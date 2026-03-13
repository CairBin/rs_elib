use axum::{
    routing::{get, post},
    Router,
};
use crate::{handler, state::AppState};

pub fn public_routes() -> Router<AppState> {
    Router::new()
        .route("/auth/register", post(handler::auth::register))
        .route("/auth/login", post(handler::auth::login))
}

pub fn protected_routes() -> Router<AppState> {
    Router::new()
        .route("/auth/me", get(handler::auth::get_current_user))
}
