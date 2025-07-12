//! Error types for MCP Core

use std::fmt;
use thiserror::Error;

/// Main error type for MCP operations
#[derive(Error, Debug, Clone)]
pub enum MCPError {
    /// General error
    #[error("General error: {0}")]
    General(String),

    /// Input validation errors
    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    /// Operation failed with specific reason
    #[error("Operation failed: {0}")]
    OperationFailed(String),

    /// Internal system error
    #[error("Internal error: {0}")]
    InternalError(String),

    /// Network communication error
    #[error("Network error: {0}")]
    Network(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Invalid argument provided
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    /// Resource not found
    #[error("Not found: {0}")]
    NotFound(String),

    /// Permission denied
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Authentication errors
    #[error("Authentication failed")]
    InvalidCredentials,

    #[error("Invalid token")]
    InvalidToken,

    #[error("Account locked")]
    AccountLocked,

    #[error("Missing context")]
    MissingContext,

    #[error("Provider error: {0}")]
    ProviderError(String),
}

impl MCPError {
    /// Get error code for this error type
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::General(_) => "MCP-000",
            Self::ValidationFailed(_) => "MCP-001",
            Self::OperationFailed(_) => "MCP-002",
            Self::InternalError(_) => "MCP-003",
            Self::Network(_) => "MCP-024",
            Self::Configuration(_) => "MCP-030",
            Self::InvalidArgument(_) => "MCP-035",
            Self::NotFound(_) => "MCP-036",
            Self::PermissionDenied(_) => "MCP-037",
            Self::InvalidCredentials => "MCP-040",
            Self::InvalidToken => "MCP-041",
            Self::AccountLocked => "MCP-042",
            Self::MissingContext => "MCP-043",
            Self::ProviderError(_) => "MCP-044",
        }
    }
}

/// Result type alias for MCP operations
pub type Result<T, E = MCPError> = std::result::Result<T, E>; 