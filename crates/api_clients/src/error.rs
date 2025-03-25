//! Error handling for API clients
//!
//! This module defines the common error types that can occur when
//! interacting with external APIs.

use thiserror::Error;

/// Common error type for API client operations
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

    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// URL parsing error
    #[error("URL error: {0}")]
    Url(#[from] url::ParseError),

    /// Unexpected response format
    #[error("Unexpected response format: {0}")]
    UnexpectedResponse(String),

    /// API returned an error
    #[error("API error ({status}): {message}")]
    ApiError {
        /// HTTP status code
        status: u16,
        /// Error message from the API
        message: String,
    },

    /// Connection error
    #[error("Connection error: {0}")]
    Connection(String),

    /// JWT token error
    #[error("JWT token error: {0}")]
    JwtToken(#[from] jsonwebtoken::errors::Error),

    /// OAuth2 error
    #[error("OAuth2 error: {0}")]
    OAuth(String),

    /// Other error
    #[error("{0}")]
    Other(String),
} 