//! Error types for API client operations
//!
//! This module defines the error types used by the API clients.

use thiserror::Error;

/// API client error type
#[derive(Debug, Error)]
pub enum Error {
    /// HTTP request error
    #[error("HTTP request error: {0}")]
    RequestError(String),

    /// API response error
    #[error("API error {code}: {message}")]
    ResponseError {
        /// HTTP status code
        code: u16,
        /// Error message
        message: String,
    },

    /// Authentication error
    #[error("Authentication error: {0}")]
    AuthError(String),

    /// Rate limit error
    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),

    /// URL parsing error
    #[error("URL parsing error: {0}")]
    UrlError(String),

    /// JSON serialization/deserialization error
    #[error("JSON error: {0}")]
    JsonError(String),

    /// I/O error
    #[error("I/O error: {0}")]
    IoError(String),

    /// Other error
    #[error("{0}")]
    Other(String),
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Error::RequestError(e.to_string())
    }
}

/// Common error type for API client operations
#[derive(Debug, Error)]
pub enum ApiError {
    /// HTTP status code
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
}
