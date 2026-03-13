use axum::{
    routing::{get, post, put, delete},
    Router,
};
use crate::handler;
use crate::state::AppState;

pub fn public_routes() -> Router<AppState> {
    Router::new()
}

pub fn protected_routes() -> Router<AppState> {
    Router::new()
        .route("/books", get(handler::book::get_books))
        .route("/books", post(handler::book::create_book))
        .route("/books/search", post(handler::book::search_books))
        .route("/books/categories", get(handler::book::get_categories))
        .route("/books/:id", get(handler::book::get_book))
        .route("/books/:id", put(handler::book::update_book))
        .route("/books/:id", delete(handler::book::delete_book))
        .route("/books/:id/read", get(handler::book::read_book))
        .route("/books/:id/file", get(handler::book::serve_book_file))
        .route("/books/:id/resources/:filename", get(handler::book::serve_book_resource))
        .route("/books/:id/chunks/:chunk_index", get(handler::book::get_book_chunk))
        .route("/books/:id/chapters", get(handler::book::get_chapters))
        .route("/books/:id/chapters/:chapter_number", get(handler::book::get_chapter))
        .route("/books/pending", get(handler::book::get_pending_books))
        .route("/books/:id/review", put(handler::book::review_book))
}
