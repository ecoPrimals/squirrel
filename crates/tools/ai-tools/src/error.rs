//! Error handling for AI tools
//!
//! This module defines the common error types that can occur when
//! interacting with AI services.

use std::fmt;

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

// Result type alias
pub type Result<T> = std::result::Result<T, AIError>;

// Error type alias for convenience
pub type Error = AIError;
