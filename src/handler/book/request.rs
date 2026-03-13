use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct CreateBookRequest {
    pub title: String,
    pub author: String,
    pub isbn: Option<String>,
    pub category: String,
}


#[derive(Serialize, Deserialize)]
pub struct UpdateBookRequest {
    pub title: Option<String>,
    pub author: Option<String>,
    pub description: Option<String>,
    pub isbn: Option<String>,
    pub category: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct SearchBooksRequest {
    pub keyword: Option<String>,
    pub category: Option<String>,
    pub format: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct PaginationQuery {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
}

impl PaginationQuery {
    pub fn get_page(&self) -> u64 {
        self.page.unwrap_or(1).max(1)
    }

    pub fn get_page_size(&self) -> u64 {
        self.page_size.unwrap_or(20).clamp(1, 100)
    }
}

#[derive(Debug, Deserialize)]
pub struct SearchBooksWithPagination {
    pub keyword: Option<String>,
    pub category: Option<String>,
    pub format: Option<String>,
    pub page: Option<u64>,
    pub page_size: Option<u64>,
}


#[derive(Debug, Deserialize)]
pub struct ReviewBookRequest {
    pub status: String,
}