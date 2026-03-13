use axum::{
    routing::{get, post, put, delete},
    Router,
};
use crate::handler;
use crate::state::AppState;

pub fn protected_routes() -> Router<AppState> {
    Router::new()
        .route("/groups", get(handler::group::get_groups))
        .route("/groups", post(handler::group::create_group))
        .route("/groups/:id", get(handler::group::get_group))
        .route("/groups/:id", put(handler::group::update_group))
        .route("/groups/:id", delete(handler::group::delete_group))
        .route("/groups/:id/users", post(handler::group::add_user_to_group))
        .route("/groups/:id/users", get(handler::group::get_group_users))
        .route("/groups/:id/users/:user_id", delete(handler::group::remove_user_from_group))
        .route("/groups/:id/books", post(handler::group::add_book_to_group))
        .route("/groups/:id/books", get(handler::group::get_group_books))
        .route("/groups/:id/books/:book_id", delete(handler::group::remove_book_from_group))
        .route("/groups/:id/invite-codes", post(handler::group::create_invite_code))
        .route("/groups/:id/invite-codes", get(handler::group::get_group_invite_codes))
        .route("/groups/:id/invite-codes/:code_id", put(handler::group::deactivate_invite_code))
        .route("/groups/join-with-invite", post(handler::group::join_with_invite_code))
}
