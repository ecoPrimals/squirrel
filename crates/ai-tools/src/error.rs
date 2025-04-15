//! Error handling for AI tools
//!
//! This module defines the common error types that can occur when
//! interacting with AI services.

use thiserror::Error;

/// Common error type for AI operations
#[derive(Debug, Error)]
pub enum Error {
    /// HTTP request failed
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// Authentication error
    #[error("Authentication error: {0}")]
    Authentication(String),

    /// Rate limiting error
    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),

    /// Model error (model not found, incompatible parameters, etc.)
    #[error("Model error: {0}")]
    Model(String),

    /// Content moderation error
    #[error("Content moderation error: {0}")]
    ContentModeration(String),

    /// Context length exceeded
    #[error("Context length exceeded: {0}")]
    ContextLength(String),

    /// Token limit exceeded
    #[error("Token limit exceeded: {0}")]
    TokenLimit(String),

    /// Tool execution error
    #[error("Tool execution error: {0}")]
    ToolExecution(String),

    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// API returned an error
    #[error("API error ({status}): {message}")]
    ApiError {
        /// HTTP status code
        status: u16,
        /// Error message from the API
        message: String,
    },

    /// Streaming error
    #[error("Streaming error: {0}")]
    Streaming(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Feature not supported by the model
    #[error("Feature not supported: {0}")]
    UnsupportedFeature(String),

    /// Invalid tool call format
    #[error("Invalid tool call: {0}")]
    InvalidToolCall(String),

    /// Invalid function call format
    #[error("Invalid function call: {0}")]
    InvalidFunctionCall(String),

    /// Invalid response format
    #[error("Invalid response format: {0}")]
    InvalidResponse(String),

    /// Service-specific error
    #[error("Service error: {0}")]
    Service(String),

    /// Other error
    #[error("{0}")]
    Other(String),
} 