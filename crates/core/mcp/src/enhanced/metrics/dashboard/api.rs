//! Dashboard API structures and error handling

use serde::{Deserialize, Serialize};

/// API error types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApiError {
    /// Authentication error
    AuthenticationError(String),
    
    /// Authorization error
    AuthorizationError(String),
    
    /// Validation error
    ValidationError(String),
    
    /// Internal server error
    InternalError(String),
    
    /// Not found error
    NotFound(String),
    
    /// Bad request error
    BadRequest(String),
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::AuthenticationError(msg) => write!(f, "Authentication error: {}", msg),
            ApiError::AuthorizationError(msg) => write!(f, "Authorization error: {}", msg),
            ApiError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            ApiError::InternalError(msg) => write!(f, "Internal error: {}", msg),
            ApiError::NotFound(msg) => write!(f, "Not found: {}", msg),
            ApiError::BadRequest(msg) => write!(f, "Bad request: {}", msg),
        }
    }
}

impl std::error::Error for ApiError {}

// Implement warp::reject::Reject trait if warp is available
impl warp::reject::Reject for ApiError {}

/// API response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    /// Success flag
    pub success: bool,
    
    /// Response data
    pub data: Option<T>,
    
    /// Error message
    pub error: Option<String>,
    
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl<T> ApiResponse<T> {
    /// Create success response
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            timestamp: chrono::Utc::now(),
        }
    }
    
    /// Create error response
    pub fn error(error: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
            timestamp: chrono::Utc::now(),
        }
    }
} 