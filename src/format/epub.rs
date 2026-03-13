use super::{ChapterParsed, FormatParser, FormatParserError, Result};

use futures::future::BoxFuture;
use once_cell::sync::Lazy;
use regex::Regex;
use std::{
    collections::HashMap,
    io::{Read, Write},
    path::{Path, PathBuf},
};
use tokio::task;
use zip::ZipArchive;

static CONTAINER_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r#"full-path="([^"]+)""#).unwrap());
static SPINE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r#"<itemref idref="([^"]+)""#).unwrap());
static ITEM_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"<item id="([^"]+)" href="([^"]+)""#).unwrap());
static TITLE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r#"<h[1-3][^>]*>(.*?)</h[1-3]>"#).unwrap());
static TAG_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r#"<[^>]+>"#).unwrap());
static SRC_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"(src|href)\s*=\s*["']([^"']+)["']"#).unwrap());

#[derive(Debug)]
enum EpubParserState {
    Uninitialized,
    Initialized {
        book_path: String,
        opf_dir: PathBuf,
        spine_items: Vec<String>,
        id_to_href: HashMap<String, String>,
        resource_map: HashMap<String, String>,
        current_index: usize,
        book_id: i32,
    },
    Finished,
}

pub struct EpubParser {
    path: Option<String>,
    state: EpubParserState,
}

impl EpubParser {
    pub fn new(upload_dir: &str) -> Self {
        Self {
            path: Some(upload_dir.to_string()),
            state: EpubParserState::Uninitialized,
        }
    }

    fn extract_and_save_resources(
        archive: &mut ZipArchive<std::fs::File>,
        opf_dir: &Path,
        book_id: i32,
    ) -> Result<HashMap<String, String>> {
        let mut resource_map = HashMap::new();
        let resources_dir = PathBuf::from("uploads").join(format!("book_{}_resources", book_id));
        
        std::fs::create_dir_all(&resources_dir)?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let file_name = file.name().to_string();
            
            let ext = Path::new(&file_name)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();
            
            let is_resource = matches!(ext.as_str(), 
                "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" | "svg" |
                "css" | "js" | "woff" | "woff2" | "ttf" | "otf"
            );

            if is_resource {
                let file_name_simple = Path::new(&file_name)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(&file_name);
                
                let mut save_name = file_name_simple.to_string();
                if resources_dir.join(&save_name).exists() {
                    let stem = Path::new(file_name_simple).file_stem().and_then(|s| s.to_str()).unwrap_or("resource");
                    let ext = Path::new(file_name_simple).extension().and_then(|e| e.to_str()).unwrap_or("");
                    let mut counter = 1;
                    loop {
                        let candidate = format!("{}_{}.{}", stem, counter, ext);
                        if !resources_dir.join(&candidate).exists() {
                            save_name = candidate;
                            break;
                        }
                        counter += 1;
                    }
                }

                let save_path = resources_dir.join(&save_name);
                let mut buf = Vec::new();
                file.read_to_end(&mut buf)?;
                
                let mut output_file = std::fs::File::create(&save_path)?;
                output_file.write_all(&buf)?;

                let epub_relative_path = opf_dir.join(&file_name);
                let epub_relative_str = epub_relative_path
                    .to_str()
                    .unwrap_or(&file_name)
                    .replace("\\", "/");
                
                let api_path = format!("/api/books/{}/resources/{}", book_id, save_name);
                
                resource_map.insert(epub_relative_str.clone(), api_path.clone());
                resource_map.insert(file_name.clone(), api_path.clone());
                
                if let Some(file_name_only) = Path::new(&file_name).file_name().and_then(|n| n.to_str()) {
                    resource_map.insert(file_name_only.to_string(), api_path.clone());
                }
                
                if let Some(stripped) = epub_relative_str.strip_prefix('/') {
                    resource_map.insert(stripped.to_string(), api_path.clone());
                }
                
                if let Some(stripped) = file_name.strip_prefix('/') {
                    resource_map.insert(stripped.to_string(), api_path);
                }
            }
        }

        Ok(resource_map)
    }

    fn replace_resources(html: &str, map: &HashMap<String, String>) -> String {
        SRC_RE
            .replace_all(html, |caps: &regex::Captures| {
                let attr = &caps[1];
                let path = &caps[2];
                
                let path_lower = path.to_lowercase();
                if path_lower.starts_with("http://") || path_lower.starts_with("https://") {
                    return caps[0].to_string();
                }
                
                if let Some(url) = map.get(path) {
                    return format!(r#"{}="{}""#, attr, url);
                }
                
                let file_name = Path::new(path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(path);
                
                if let Some(url) = map.get(file_name) {
                    return format!(r#"{}="{}""#, attr, url);
                }
                
                for (key, url) in map.iter() {
                    if key.ends_with(file_name) {
                        return format!(r#"{}="{}""#, attr, url);
                    }
                }
                
                caps[0].to_string()
            })
            .to_string()
    }

    fn extract_title(html: &str) -> Option<String> {
        let caps = TITLE_RE.captures(html)?;
        let title = caps.get(1)?.as_str();
        Some(TAG_RE.replace_all(title, "").trim().to_string())
    }
}

impl FormatParser for EpubParser {
    fn new(path: &str) -> Self {
        Self {
            path: Some(path.to_string()),
            state: EpubParserState::Uninitialized,
        }
    }

    fn parse_chapters<'a>(&'a mut self, book_id: i32) -> BoxFuture<'a, Result<ChapterParsed>> {
        Box::pin(async move {
            if matches!(self.state, EpubParserState::Uninitialized) {
                let book_path = self.path.clone().ok_or_else(|| {
                    FormatParserError::ParseError("EPUB path missing".into())
                })?;

                let init_state = task::spawn_blocking(move || -> Result<EpubParserState> {
                    let file = std::fs::File::open(&book_path)?;
                    let mut archive = ZipArchive::new(file)?;

                    let mut container = String::new();
                    archive.by_name("META-INF/container.xml")?
                        .read_to_string(&mut container)?;

                    let opf_path = CONTAINER_RE.captures(&container)
                        .and_then(|c| c.get(1))
                        .map(|m| m.as_str().to_string())
                        .unwrap_or_else(|| "content.opf".to_string());

                    let mut opf_content = String::new();
                    archive.by_name(&opf_path)?
                        .read_to_string(&mut opf_content)?;

                    let spine_items = SPINE_RE.captures_iter(&opf_content)
                        .filter_map(|c| c.get(1).map(|m| m.as_str().to_string()))
                        .collect();

                    let mut id_to_href = HashMap::new();
                    for cap in ITEM_RE.captures_iter(&opf_content) {
                        let id = cap.get(1).unwrap().as_str().to_string();
                        let href = cap.get(2).unwrap().as_str().to_string();
                        id_to_href.insert(id, href);
                    }

                    let opf_dir = Path::new(&opf_path)
                        .parent()
                        .unwrap_or(Path::new(""))
                        .to_path_buf();

                    let resource_map = Self::extract_and_save_resources(&mut archive, &opf_dir, book_id)?;

                    Ok(EpubParserState::Initialized {
                        book_path,
                        opf_dir,
                        spine_items,
                        id_to_href,
                        resource_map,
                        current_index: 0,
                        book_id,
                    })
                })
                .await
                .map_err(|_| FormatParserError::ParseError("Init failed".to_string()))??;

                self.state = init_state;
            }

            if matches!(self.state, EpubParserState::Finished) {
                return Err(FormatParserError::EndOfChapters);
            }

            let chapter = task::spawn_blocking({
                let state = match &self.state {
                    EpubParserState::Initialized {
                        book_path,
                        opf_dir,
                        spine_items,
                        id_to_href,
                        resource_map,
                        current_index,
                        book_id,
                    } => (
                        book_path.clone(),
                        opf_dir.clone(),
                        spine_items.clone(),
                        id_to_href.clone(),
                        resource_map.clone(),
                        *current_index,
                        *book_id,
                    ),
                    _ => return Err(FormatParserError::ParseError("Invalid state".to_string())),
                };

                move || -> Result<ChapterParsed> {
                    let (path, opf_dir, spine, id_map, res_map, idx, book_id) = state;

                    if idx >= spine.len() {
                        return Err(FormatParserError::EndOfChapters);
                    }

                    let file = std::fs::File::open(&path)?;
                    let mut archive = ZipArchive::new(file)?;

                    let idref = &spine[idx];
                    let href = id_map.get(idref)
                        .ok_or_else(|| FormatParserError::ParseError("Missing chapter href".to_string()))?;

                    let full_path = opf_dir.join(href);
                    let full_str = full_path
                        .to_str()
                        .ok_or_else(|| FormatParserError::InvalidPath(href.clone()))?
                        .replace("\\", "/");

                    let mut content = String::new();
                    archive.by_name(&full_str)?
                        .read_to_string(&mut content)?;

                    let content = Self::replace_resources(&content, &res_map);
                    let chapter_num = (idx + 1) as i32;
                    let title = Self::extract_title(&content)
                        .unwrap_or_else(|| format!("第{}章", chapter_num));

                    Ok(ChapterParsed {
                        book_id,
                        chapter_number: chapter_num,
                        title,
                        content,
                    })
                }
            })
            .await
            .map_err(|_| FormatParserError::ParseError("Read chapter failed".to_string()))??;

            if let EpubParserState::Initialized { current_index, spine_items, .. } = &mut self.state {
                *current_index += 1;
                if *current_index >= spine_items.len() {
                    self.state = EpubParserState::Finished;
                }
            }

            Ok(chapter)
        })
    }
}