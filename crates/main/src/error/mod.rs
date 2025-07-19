//! Error handling for the Squirrel primal

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Main error type for the Squirrel primal
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum PrimalError {
    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Configuration error (alias for backward compatibility)
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// MCP protocol error
    #[error("MCP protocol error: {0}")]
    McpProtocol(String),

    /// Context error
    #[error("Context error: {0}")]
    Context(String),

    /// Network error
    #[error("Network error: {0}")]
    Network(String),

    /// Network error with details
    #[error("Network error: {0}")]
    NetworkError(String),

    /// Invalid context error
    #[error("Invalid context: {0}")]
    InvalidContext(String),

    /// Shutdown error
    #[error("Shutdown error: {0}")]
    Shutdown(String),

    /// Resource error
    #[error("Resource error: {0}")]
    ResourceError(String),

    /// Security error
    #[error("Security error: {0}")]
    SecurityError(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Serialization error with details
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Registration error
    #[error("Registration error: {0}")]
    RegistrationError(String),

    /// Unsupported operation error
    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),

    /// Internal error
    #[error("Internal error: {0}")]
    InternalError(String),

    /// Internal error (shorter variant)
    #[error("Internal error: {0}")]
    Internal(String),

    /// Operation failed error
    #[error("Operation failed: {0}")]
    OperationFailed(String),

    /// Not found error
    #[error("Not found: {0}")]
    NotFoundError(String),

    /// Generic error
    #[error("Generic error: {0}")]
    Generic(String),

    /// General error
    #[error("General error: {0}")]
    General(String),

    /// Validation error
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// Operation not supported
    #[error("Operation not supported: {0}")]
    OperationNotSupported(String),

    /// IO error
    #[error("IO error: {0}")]
    IO(String),

    /// JSON error
    #[error("JSON error: {0}")]
    Json(String),

    /// URL parse error
    #[error("URL parse error: {0}")]
    UrlParse(String),

    /// Timeout error
    #[error("Timeout error")]
    Timeout,

    /// Service unavailable error
    #[error("Service unavailable")]
    ServiceUnavailable,

    /// Authentication error
    #[error("Authentication error: {0}")]
    Authentication(String),

    /// Authorization error
    #[error("Authorization error: {0}")]
    Authorization(String),

    /// Resource not found error
    #[error("Resource not found: {0}")]
    ResourceNotFound(String),

    /// Validation error
    #[error("Validation error: {0}")]
    Validation(String),
}

impl From<std::io::Error> for PrimalError {
    fn from(error: std::io::Error) -> Self {
        PrimalError::IO(error.to_string())
    }
}

impl From<serde_json::Error> for PrimalError {
    fn from(error: serde_json::Error) -> Self {
        PrimalError::Json(error.to_string())
    }
}

impl From<url::ParseError> for PrimalError {
    fn from(error: url::ParseError) -> Self {
        PrimalError::UrlParse(error.to_string())
    }
}
