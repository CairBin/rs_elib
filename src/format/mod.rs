pub mod epub;
pub mod txt;
pub mod markdown;

use futures::future::BoxFuture;
use tokio::sync::Mutex;
use tokio::time;
use std::{collections::HashMap, fmt};
use std::sync::Arc;

#[derive(Debug)]
pub enum FormatParserError {
    IoError(std::io::Error),
    ZipError(zip::result::ZipError),
    RegexError(regex::Error),
    ParseError(String),
    InvalidPath(String),
    TimeError(time::error::Error),
    EndOfChapters
}

// 解析器的Result
pub type Result<T> = std::result::Result<T, FormatParserError>;

pub trait FormatParser: Send + Sync{
    fn new(path: &str) -> Self
        where Self: Sized;

    /// 多次调用直到返回`FormatParserError::EndOfChapters`
    fn parse_chapters<'a>(
        &'a mut self,
        book_id: i32,
    ) -> BoxFuture<'a, Result<ChapterParsed>>;
}

impl fmt::Display for FormatParserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FormatParserError::ZipError(e) => write!(f, "Zip error: {}", e),
            FormatParserError::ParseError(msg) => write!(f, "Failed to parse file: {}", msg),
            FormatParserError::IoError(e) => write!(f, "IO error：{}", e),
            FormatParserError::RegexError(e) => write!(f, "Regex error: {}", e),
            FormatParserError::InvalidPath(msg) => write!(f, "Invalid path: {}", msg),
            FormatParserError::TimeError(e) => write!(f, "Time error: {}", e),
            FormatParserError::EndOfChapters => write!(f, "End of chapters"),
        }
    }
}

impl std::error::Error for FormatParserError {}

// 实现从 IO 错误到自定义错误的转换（支持 ? 操作符）
impl From<std::io::Error> for FormatParserError {
    fn from(e: std::io::Error) -> Self {
        FormatParserError::IoError(e)
    }
}
// 实现从 ZIP 错误到自定义错误的转换
impl From<zip::result::ZipError> for FormatParserError {
    fn from(e: zip::result::ZipError) -> Self {
        FormatParserError::ZipError(e)
    }
}

impl From<regex::Error> for FormatParserError {
    fn from(e: regex::Error) -> Self {
        FormatParserError::RegexError(e)
    }
}

impl From<time::error::Error> for FormatParserError {
    fn from(e: time::error::Error) -> Self {
        FormatParserError::TimeError(e)
    }
}


/// 解析后的章节，用于作为Result参数，作为正常情况下的通用返回结果
pub struct ChapterParsed {
    pub book_id: i32,
    pub chapter_number: i32,
    pub title: String,
    pub content: String,
}

/// 解析器工厂 trait，用于创建新的解析器实例
pub trait FormatParserFactory: Send + Sync {
    fn create(&self, path: &str) -> Arc<Mutex<dyn FormatParser + Send + Sync>>;
}

/// 泛型解析器工厂
pub struct GenericParserFactory<T> {
    _marker: std::marker::PhantomData<T>,
}

impl<T: FormatParser + 'static> GenericParserFactory<T> {
    pub fn new() -> Self {
        Self {
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T: FormatParser + 'static> FormatParserFactory for GenericParserFactory<T> {
    fn create(&self, path: &str) -> Arc<Mutex<dyn FormatParser + Send + Sync>> {
        Arc::new(Mutex::new(T::new(path)))
    }
}

/// 解析器注册器
#[derive(Clone)]
pub struct FormatParserRegister{
    parsers: HashMap<&'static str, Arc<dyn FormatParserFactory + Send + Sync>>,
}

impl FormatParserRegister {
    pub fn new() -> Self {
        Self {
            parsers: HashMap::new(),
        }
    }

    pub fn register<T: FormatParser + 'static>(
        &mut self,
        book_format: &'static str,
    ) {
        let factory = Arc::new(GenericParserFactory::<T>::new());
        self.parsers.insert(book_format, factory);
    }

    pub fn get(&self, book_format: &str, path: &str) -> Option<Arc<Mutex<dyn FormatParser + Send + Sync>>> {
        self.parsers.get(book_format).map(|factory| factory.create(path))
    }
}