use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use uuid::Uuid;
use chrono::Utc;
use thiserror::Error;
use tracing::error;

use crate::api::{ApiResponse, ApiError};
use crate::api::commands::CommandServiceError;
use crate::mcp::McpError;

/// Application error type
#[derive(Debug, Error)]
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
    Database(String),
    
    /// Generic error with custom status code
    #[error("{0}")]
    Custom(StatusCode, String),

    /// Validation error
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// Command service error
    #[error("Command service error: {0}")]
    CommandService(#[from] CommandServiceError),

    /// MCP error
    #[error("MCP error: {0}")]
    Mcp(#[from] McpError),
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::Internal(err.to_string())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_code, message) = match &self {
            AppError::InvalidRequest(msg) => (
                StatusCode::BAD_REQUEST, 
                "invalid_request", 
                msg.clone()
            ),
            AppError::NotFound(msg) => (
                StatusCode::NOT_FOUND, 
                "not_found", 
                msg.clone()
            ),
            AppError::Unauthorized(msg) => (
                StatusCode::UNAUTHORIZED, 
                "unauthorized", 
                msg.clone()
            ),
            AppError::Forbidden(msg) => (
                StatusCode::FORBIDDEN, 
                "forbidden", 
                msg.clone()
            ),
            AppError::Conflict(msg) => (
                StatusCode::CONFLICT, 
                "conflict", 
                msg.clone()
            ),
            AppError::RateLimitExceeded(msg) => (
                StatusCode::TOO_MANY_REQUESTS, 
                "rate_limit_exceeded", 
                msg.clone()
            ),
            AppError::Internal(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR, 
                "internal_error", 
                msg.clone()
            ),
            AppError::Database(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR, 
                "database_error", 
                msg.clone()
            ),
            AppError::Custom(status, msg) => (
                *status, 
                "custom_error", 
                msg.clone()
            ),
            AppError::ValidationError(msg) => (
                StatusCode::BAD_REQUEST,
                "VALIDATION_ERROR",
                msg.clone(),
            ),
            AppError::CommandService(err) => match err {
                CommandServiceError::CommandNotFound(msg) => (
                    StatusCode::NOT_FOUND,
                    "COMMAND_NOT_FOUND",
                    format!("Command not found: {}", msg),
                ),
                CommandServiceError::InvalidParameters(msg) => (
                    StatusCode::BAD_REQUEST,
                    "INVALID_PARAMETERS",
                    format!("Invalid parameters: {}", msg),
                ),
                CommandServiceError::ExecutionFailed(msg) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "EXECUTION_FAILED",
                    format!("Command execution failed: {}", msg),
                ),
                CommandServiceError::RepositoryError(_) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "DATABASE_ERROR",
                    "A database error occurred".to_string(),
                ),
                CommandServiceError::McpError(msg) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "MCP_ERROR",
                    format!("MCP error: {}", msg),
                ),
                CommandServiceError::InvalidExecutionId(msg) => (
                    StatusCode::NOT_FOUND,
                    "EXECUTION_NOT_FOUND",
                    format!("Execution not found: {}", msg),
                ),
                CommandServiceError::Unauthorized(msg) => (
                    StatusCode::FORBIDDEN,
                    "UNAUTHORIZED",
                    format!("Unauthorized: {}", msg),
                ),
            },
            AppError::Mcp(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "MCP_ERROR",
                format!("MCP error: {}", err),
            ),
        };

        // Log the error with more details for internal debugging
        error!(
            error_code = %error_code,
            status_code = %status.as_u16(),
            error = %self,
            "API error"
        );

        // Generate a request ID for tracking
        let _request_id = Uuid::new_v4().to_string();
        let _timestamp = Utc::now().to_rfc3339();

        // Create a standard API response with error details
        let body = Json(ApiResponse::<()> {
            success: false,
            data: None,
            error: Some(ApiError {
                code: error_code.to_string(),
                message,
                details: None,
            }),
            meta: Default::default(),
        });

        (status, body).into_response()
    }
} 