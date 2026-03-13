use axum::{
    routing::{get, put},
    Router,
};
use crate::handler;
use crate::state::AppState;

pub fn protected_routes() -> Router<AppState> {
    Router::new()
        .route("/settings", get(handler::settings::get_settings))
        .route("/settings/:key", put(handler::settings::update_setting))
}
