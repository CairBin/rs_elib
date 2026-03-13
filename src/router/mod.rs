pub mod auth;
pub mod book;
pub mod group;
pub mod user;
pub mod settings;
pub mod comment;

use axum::Router;
use crate::state::AppState;

pub fn create_public_routes() -> Router<AppState> {
    Router::new()
        .merge(auth::public_routes())
        .merge(book::public_routes())
        .merge(comment::public_routes())
}

pub fn create_protected_routes() -> Router<AppState> {
    Router::new()
        .merge(auth::protected_routes())
        .merge(book::protected_routes())
        .merge(group::protected_routes())
        .merge(user::protected_routes())
        .merge(settings::protected_routes())
        .merge(comment::protected_routes())
}
