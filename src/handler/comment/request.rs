use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CreateCommentRequest {
    pub book_id: Option<i32>,
    pub chapter_id: Option<i32>,
    pub content: String,
}


#[derive(Debug, Deserialize)]
pub struct ReviewCommentRequest {
    pub status: String,
}