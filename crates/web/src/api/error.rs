use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use uuid::Uuid;
use chrono::Utc;

/// Application error type
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    /// Invalid request error
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
    
    /// Not found error
    #[error("Not found: {0}")]
    NotFound(String),
    
    /// Unauthorized error
    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    /// Forbidden error
    #[error("Forbidden: {0}")]
    Forbidden(String),
    
    /// Conflict error
    #[error("Conflict: {0}")]
    Conflict(String),
    
    /// Rate limit exceeded
    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),

    /// Database error
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    /// Generic error with custom status code
    #[error("{0}")]
    Custom(StatusCode, String),
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::Internal(err.to_string())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_code, message) = match self {
            AppError::InvalidRequest(msg) => (
                StatusCode::BAD_REQUEST, 
                "invalid_request", 
                msg
            ),
            AppError::NotFound(msg) => (
                StatusCode::NOT_FOUND, 
                "not_found", 
                msg
            ),
            AppError::Unauthorized(msg) => (
                StatusCode::UNAUTHORIZED, 
                "unauthorized", 
                msg
            ),
            AppError::Forbidden(msg) => (
                StatusCode::FORBIDDEN, 
                "forbidden", 
                msg
            ),
            AppError::Conflict(msg) => (
                StatusCode::CONFLICT, 
                "conflict", 
                msg
            ),
            AppError::RateLimitExceeded(msg) => (
                StatusCode::TOO_MANY_REQUESTS, 
                "rate_limit_exceeded", 
                msg
            ),
            AppError::Internal(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR, 
                "internal_error", 
                msg
            ),
            AppError::Database(err) => (
                StatusCode::INTERNAL_SERVER_ERROR, 
                "database_error", 
                format!("Database error: {}", err)
            ),
            AppError::Custom(status, msg) => (
                status, 
                "custom_error", 
                msg
            ),
        };

        // Generate a request ID for tracking
        let request_id = Uuid::new_v4().to_string();
        let timestamp = Utc::now().to_rfc3339();

        let body = Json(json!({
            "success": false,
            "error": {
                "code": error_code,
                "message": message,
                "details": null
            },
            "data": null,
            "meta": {
                "requestId": request_id,
                "timestamp": timestamp
            }
        }));

        (status, body).into_response()
    }
} 