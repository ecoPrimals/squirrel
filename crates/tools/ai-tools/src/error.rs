//! Error handling for AI tools
//!
//! **DEPRECATED**: This error module is being replaced by the unified error system.
//! Please migrate to `universal-error` for all new code.
//!
//! Migration guide:
//! ```ignore
//! // Old:
//! use crate::error::{AIError, Result};
//! // New:
//! use universal_error::{Result, tools::AIToolsError};
//! ```
//!
//! For detailed migration instructions, see: `crates/universal-error/README.md`
//!
//! This module defines the common error types that can occur when
//! interacting with AI services.

use std::fmt;

/// AI Tools error type
///
/// **DEPRECATED**: Use `universal_error::tools::AIToolsError` instead.
#[deprecated(
    since = "0.2.0",
    note = "Use `universal_error::tools::AIToolsError` instead"
)]
#[derive(Debug)]
pub enum AIError {
    /// Configuration error
    Configuration(String),

    /// Network/HTTP error
    Network(String),

    /// HTTP client error
    Http(String),

    /// Provider-specific error
    Provider(String),

    /// Model-related error
    Model(String),

    /// Parsing error
    Parse(String),

    /// Rate limiting error
    RateLimit(String),

    /// Streaming error
    Streaming(String),

    /// Runtime error
    Runtime(String),

    /// Invalid response error
    InvalidResponse(String),

    /// Parsing error
    Parsing(String),

    /// API error
    ApiError(String),

    /// Authentication error
    Authentication(String),

    /// Timeout error
    Timeout(String),

    /// Validation error
    Validation(String),

    /// Invalid request error
    InvalidRequest(String),

    /// Network error (legacy alias)
    NetworkError(String),

    /// Parse error (legacy alias)
    ParseError(String),

    /// Unsupported provider error
    UnsupportedProvider(String),

    /// Generic error
    Generic(String),
}

impl fmt::Display for AIError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AIError::Configuration(msg) => write!(f, "Configuration error: {msg}"),
            AIError::Network(msg) => write!(f, "Network error: {msg}"),
            AIError::Http(msg) => write!(f, "HTTP error: {msg}"),
            AIError::Provider(msg) => write!(f, "Provider error: {msg}"),
            AIError::Model(msg) => write!(f, "Model error: {msg}"),
            AIError::Parse(msg) => write!(f, "Parse error: {msg}"),
            AIError::RateLimit(msg) => write!(f, "Rate limit error: {msg}"),
            AIError::Streaming(msg) => write!(f, "Streaming error: {msg}"),
            AIError::Runtime(msg) => write!(f, "Runtime error: {msg}"),
            AIError::InvalidResponse(msg) => write!(f, "Invalid response: {msg}"),
            AIError::ApiError(msg) => write!(f, "API error: {msg}"),
            AIError::Authentication(msg) => write!(f, "Authentication error: {msg}"),
            AIError::Timeout(msg) => write!(f, "Timeout error: {msg}"),
            AIError::Validation(msg) => write!(f, "Validation error: {msg}"),
            AIError::InvalidRequest(msg) => write!(f, "Invalid request: {msg}"),
            AIError::NetworkError(msg) => write!(f, "Network error: {msg}"),
            AIError::ParseError(msg) => write!(f, "Parse error: {msg}"),
            AIError::UnsupportedProvider(msg) => write!(f, "Unsupported provider: {msg}"),
            AIError::Generic(msg) => write!(f, "Error: {msg}"),
            AIError::Parsing(msg) => write!(f, "Parsing error: {msg}"),
        }
    }
}

impl std::error::Error for AIError {}

impl From<reqwest::Error> for AIError {
    fn from(err: reqwest::Error) -> Self {
        AIError::Http(err.to_string())
    }
}

impl From<serde_json::Error> for AIError {
    fn from(err: serde_json::Error) -> Self {
        AIError::Parse(err.to_string())
    }
}

impl From<std::io::Error> for AIError {
    fn from(err: std::io::Error) -> Self {
        AIError::Network(err.to_string())
    }
}

impl From<tokio::time::error::Elapsed> for AIError {
    fn from(err: tokio::time::error::Elapsed) -> Self {
        AIError::Timeout(err.to_string())
    }
}

// Bridge from new unified error system to deprecated AIError
// This allows gradual migration while maintaining compatibility
impl From<universal_error::tools::AIToolsError> for AIError {
    fn from(err: universal_error::tools::AIToolsError) -> Self {
        use universal_error::tools::AIToolsError;
        match err {
            AIToolsError::Provider(s) => AIError::Provider(s),
            AIToolsError::Router(s) => AIError::Provider(s),
            AIToolsError::Local(s) => AIError::Provider(s),
            AIToolsError::ModelNotFound(s) => AIError::Model(s),
            AIToolsError::RateLimitExceeded(s) => AIError::RateLimit(s),
            AIToolsError::InvalidResponse(s) => AIError::InvalidResponse(s),
            AIToolsError::Network(s) => AIError::NetworkError(s),
            AIToolsError::Api(s) => AIError::ApiError(s),
            AIToolsError::Configuration(s) => AIError::Configuration(s),
            AIToolsError::Parse(s) => AIError::ParseError(s),
            AIToolsError::UnsupportedProvider(s) => AIError::UnsupportedProvider(s),
            AIToolsError::InvalidRequest(s) => AIError::InvalidRequest(s),
            AIToolsError::Authentication(s) => AIError::Authentication(s),
        }
    }
}

// Result type alias
pub type Result<T> = std::result::Result<T, AIError>;

// Error type alias for convenience
pub type Error = AIError;
