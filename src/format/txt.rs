use super::{ChapterParsed, FormatParser, FormatParserError, Result};

use futures::future::BoxFuture;
use once_cell::sync::Lazy;
use regex::Regex;

use std::path::PathBuf;

use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncSeekExt, BufReader, SeekFrom};

static CHAPTER_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"^(第\s*[零一二三四五六七八九十百千万\d]+\s*[章节卷篇集部节]|卷\s*[零一二三四五六七八九十百千万\d]+\s*[：:]|[零一二三四五六七八九十百千万\d]+\s*[章节卷篇集部节]\s*[：:])"
    )
    .unwrap()
});

#[derive(Debug)]
struct ChapterIndex {
    title: String,
    start: u64,
    end: u64,
}

#[derive(Debug)]
enum TxtParserState {
    Uninitialized,
    Initialized {
        chapters: Vec<ChapterIndex>,
        current_index: usize,
    },
    Finished,
}

pub struct TxtParser {
    path: Option<PathBuf>,
    state: TxtParserState,
}

impl TxtParser {
    pub fn new(path: &str) -> Self {
        Self {
            path: Some(PathBuf::from(path)),
            state: TxtParserState::Uninitialized,
        }
    }

    async fn build_index(&mut self) -> Result<()> {
        if matches!(
            self.state,
            TxtParserState::Initialized { .. } | TxtParserState::Finished
        ) {
            return Ok(());
        }

        let path = self
            .path
            .as_ref()
            .ok_or_else(|| FormatParserError::ParseError("TXT path missing".into()))?;

        let file = File::open(path).await?;

        let mut reader = BufReader::new(file);

        let mut chapters: Vec<ChapterIndex> = Vec::new();

        let mut offset: u64 = 0;

        let mut current_title = "正文".to_string();
        let mut current_start = 0;

        let mut buf = String::new();

        loop {
            buf.clear();

            let bytes = reader.read_line(&mut buf).await?;

            if bytes == 0 {
                break;
            }

            let trimmed = buf.trim();

            if CHAPTER_RE.is_match(trimmed) {
                if !chapters.is_empty() || offset != 0 {
                    chapters.push(ChapterIndex {
                        title: current_title,
                        start: current_start,
                        end: offset,
                    });
                }

                current_title = trimmed.to_string();
                current_start = offset;
            }

            offset += bytes as u64;
        }

        chapters.push(ChapterIndex {
            title: current_title,
            start: current_start,
            end: offset,
        });

        self.state = TxtParserState::Initialized {
            chapters,
            current_index: 0,
        };

        Ok(())
    }

    async fn read_chapter_content(&self, start: u64, end: u64) -> Result<String> {
        let path = self.path.as_ref().unwrap();

        let mut file = File::open(path).await?;

        file.seek(SeekFrom::Start(start)).await?;

        let size = (end - start) as usize;

        let mut buffer = vec![0u8; size];

        file.read_exact(&mut buffer).await?;

        let text = String::from_utf8_lossy(&buffer);

        let html = text
            .lines()
            .map(|l| l.trim())
            .collect::<Vec<_>>()
            .join("<br>");

        Ok(format!("<div>{}</div>", html))
    }
}

impl FormatParser for TxtParser {
    fn new(path: &str) -> Self {
        Self {
            path: Some(PathBuf::from(path)),
            state: TxtParserState::Uninitialized,
        }
    }

    fn parse_chapters<'a>(&'a mut self, book_id: i32) -> BoxFuture<'a, Result<ChapterParsed>> {
        Box::pin(async move {
            self.build_index().await?;

            let (start, end, title, chapter_number, new_index, finished) = match &mut self.state {
                TxtParserState::Initialized {
                    chapters,
                    current_index,
                } => {
                    if *current_index >= chapters.len() {
                        self.state = TxtParserState::Finished;
                        return Err(FormatParserError::EndOfChapters);
                    }

                    let chapter_idx = &chapters[*current_index];

                    let start = chapter_idx.start;
                    let end = chapter_idx.end;
                    let title = chapter_idx.title.clone();

                    let chapter_number = *current_index + 1;

                    let new_index = *current_index + 1;

                    let finished = new_index >= chapters.len();

                    (start, end, title, chapter_number, new_index, finished)
                }

                TxtParserState::Finished => return Err(FormatParserError::EndOfChapters),

                TxtParserState::Uninitialized => {
                    return Err(FormatParserError::ParseError(
                        "Parser not initialized".into(),
                    ));
                }
            };

            let content = self.read_chapter_content(start, end).await?;

            let chapter = ChapterParsed {
                book_id,
                chapter_number: chapter_number as i32,
                title,
                content,
            };

            if let TxtParserState::Initialized { current_index, .. } = &mut self.state {
                *current_index = new_index;

                if finished {
                    self.state = TxtParserState::Finished;
                }
            }

            Ok(chapter)
        })
    }
}
