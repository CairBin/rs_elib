use axum::{
    routing::{get, post, put, delete},
    Router,
};
use crate::handler;
use crate::state::AppState;

pub fn protected_routes() -> Router<AppState> {
    Router::new()
        .route("/users", get(handler::user::get_users))
        .route("/users", post(handler::user::create_user))
        .route("/users/me/profile", put(handler::user::update_user_profile))
        .route("/users/:id", get(handler::user::get_user))
        .route("/users/:id/role", put(handler::user::update_user_role))
        .route("/users/:id/password", put(handler::user::update_user_password))
        .route("/users/:id/disabled", put(handler::user::update_user_disabled))
        .route("/users/:id", delete(handler::user::delete_user))
}
