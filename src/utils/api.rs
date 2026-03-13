use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;


#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
}


impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: None,
        }
    }

    pub fn error(message: &str) -> Self {
        Self {
            success: false,
            data: None,
            message: Some(message.to_string()),
        }
    }
}


impl<T: Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> Response {
        let status = if self.success {
            StatusCode::OK
        } else {
            StatusCode::BAD_REQUEST
        };
        (status, axum::Json(self)).into_response()
    }
}

pub fn internal_error(message: &str) -> Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        message.to_string(),
    ).into_response()
}

pub fn forbidden(message: &str) -> Response {
    (
        StatusCode::FORBIDDEN,
        message.to_string(),
    ).into_response()
}

pub fn not_found(message: &str) -> Response {
    (
        StatusCode::NOT_FOUND,
        message.to_string(),
    ).into_response()
}

pub fn conflict(message: &str) -> Response {
    (
        StatusCode::CONFLICT,
        message.to_string(),
    ).into_response()
}

pub fn unauthorized(message: &str) -> Response {
    (
        StatusCode::UNAUTHORIZED,
        message.to_string(),
    ).into_response()
}

pub fn bad_request(message: &str) -> Response {
    (
        StatusCode::BAD_REQUEST,
        message.to_string(),
    ).into_response()
}

pub fn created<T: Serialize>(data: T) -> Response {
    (
        StatusCode::CREATED,
        Json(data)
    ).into_response()
}

pub fn success<T: Serialize>(data: T) -> Response{
    (
        StatusCode::OK,
        Json(data)
    ).into_response()
}