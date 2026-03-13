use serde::Serialize;


#[derive(Serialize)]
pub struct PageResponse<T> {
    pub items: Vec<T>,
    pub page: u64,
    pub page_size: u64,
    pub total: u64,
    pub total_pages: u64
}

impl<T> PageResponse<T> {
    pub fn new(items: Vec<T>, page: u64, page_size: u64, total: u64) -> Self {
        let total_pages = if total == 0 {
            0
        } else {
            (total + page_size - 1) / page_size
        };
        Self {
            items,
            page,
            page_size,
            total,
            total_pages,
        }
    }
}

