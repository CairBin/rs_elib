use axum::{
    routing::{get, post, put, delete},
    Router,
};
use crate::handler;
use crate::state::AppState;

pub fn public_routes() -> Router<AppState> {
    Router::new()
        .route("/books/:book_id/comments", get(handler::comment::get_book_comments))
        .route("/chapters/:chapter_id/comments", get(handler::comment::get_chapter_comments))
}

pub fn protected_routes() -> Router<AppState> {
    Router::new()
        .route("/comments", post(handler::comment::create_comment))
        .route("/comments/:id", delete(handler::comment::delete_comment))
        .route("/comments/pending", get(handler::comment::get_pending_comments))
        .route("/comments/:id/review", put(handler::comment::review_comment))
}
