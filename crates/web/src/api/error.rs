use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

/// Application error type
#[derive(Debug)]
pub enum AppError {
    /// Internal server error
    Internal(String),
    /// Not found error
    NotFound(String),
    /// Bad request error
    BadRequest(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
        };

        (status, message).into_response()
    }
} 