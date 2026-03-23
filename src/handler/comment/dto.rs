use serde::{Deserialize, Serialize};
use chrono::NaiveDateTime;

#[derive(Debug, Serialize)]
pub struct CommentDto {
    pub id: i32,
    pub book_id: Option<i32>,
    pub chapter_id: Option<i32>,
    pub user_id: i32,
    pub username: String,
    pub content: String,
    pub status: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
