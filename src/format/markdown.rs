use super::{FormatParser, FormatParserError, Result, ChapterParsed};
use futures::future::BoxFuture;
use pulldown_cmark::{Parser, Options, html};
use tokio::fs;
use tokio::io::{AsyncBufReadExt, BufReader};

pub struct MarkdownParser {
    path: Option<String>,       // Markdown 文件路径
    lines: Option<Vec<String>>, // 文件行缓存
    current_index: usize,       // 当前解析行索引
    chapter_counter: i32,       // 章节计数器
}

impl MarkdownParser {
    pub fn new(path: &str) -> Self {
        Self {
            path: Some(path.to_string()),
            lines: None,
            current_index: 0,
            chapter_counter: 0,
        }
    }

    /// 异步读取文件并缓存每行
    async fn load_file(&mut self) -> Result<()> {
        if self.lines.is_some() {
            return Ok(());
        }
        let path = self.path.as_ref().ok_or_else(|| {
            FormatParserError::ParseError("Markdown 文件路径未设置".to_string())
        })?;
        let file = fs::File::open(path).await?;
        let reader = BufReader::new(file);
        let mut lines = Vec::new();
        let mut lines_stream = reader.lines();
        while let Some(line) = lines_stream.next_line().await? {
            lines.push(line);
        }
        self.lines = Some(lines);
        Ok(())
    }

    /// 判断行是否是章节标题（一至六级标题都可以作为章节分隔）
    fn is_chapter_title(line: &str) -> bool {
        let trimmed = line.trim_start();
        if trimmed.starts_with('|') {
            return false;
        }
        let mut chars = trimmed.chars().peekable();
        let mut hash_count = 0;
        while let Some(&'#') = chars.peek() {
            hash_count += 1;
            chars.next();
        }
        hash_count >= 1 && hash_count <= 6 && chars.next() == Some(' ')
    }

    /// 将 Markdown 转 HTML（使用 pulldown-cmark 完整标准 Markdown）
    fn markdown_to_html(md: &str) -> String {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_FOOTNOTES);
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TASKLISTS);
        options.insert(Options::ENABLE_SMART_PUNCTUATION);

        let parser = Parser::new_ext(md, options);
        let mut html_output = String::with_capacity(md.len() + 1024);
        html::push_html(&mut html_output, parser);
        html_output
    }
}

impl FormatParser for MarkdownParser {
    fn new(path: &str) -> Self {
        Self {
            path: Some(path.to_string()),
            lines: None,
            current_index: 0,
            chapter_counter: 0,
        }
    }

    fn parse_chapters<'a>(
        &'a mut self,
        book_id: i32,
    ) -> BoxFuture<'a, Result<ChapterParsed>> {
        Box::pin(async move {
            self.load_file().await?;
            let lines = self.lines.as_ref().unwrap();
            if self.current_index >= lines.len() {
                return Err(FormatParserError::EndOfChapters);
            }

            let mut chapter_lines = Vec::new();
            let mut chapter_title = String::from("正文");

            for i in self.current_index..lines.len() {
                let line = &lines[i];
                if Self::is_chapter_title(line) {
                    if !chapter_lines.is_empty() {
                        break; // 遇到下一章节标题，结束当前章节
                    } else {
                        chapter_title = line.trim_start_matches('#').trim().to_string();
                    }
                }
                chapter_lines.push(line.clone());
                self.current_index = i + 1;
            }

            let content_md = chapter_lines.join("\n");
            let html_content = Self::markdown_to_html(&content_md);

            self.chapter_counter += 1;

            let chapter = ChapterParsed {
                book_id,
                chapter_number: self.chapter_counter,
                title: chapter_title,
                content: html_content,
            };

            Ok(chapter)
        })
    }
}