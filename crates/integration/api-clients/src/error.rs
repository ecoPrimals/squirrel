//! Error types for API client operations
//!
//! **DEPRECATED**: This error module is being replaced by the unified error system.
//! Please migrate to `universal-error` for all new code.
//!
//! Migration guide:
//! ```ignore
//! // Old:
//! use crate::error::Error;
//! // New:
//! use universal_error::{Result, integration::APIClientError};
//! ```
//!
//! For detailed migration instructions, see: `crates/universal-error/README.md`
//!
//! This module defines the error types used by the API clients.

use thiserror::Error;

/// API client error type
///
/// **DEPRECATED**: Use `universal_error::integration::APIClientError` instead.
#[deprecated(since = "0.2.0", note = "Use `universal_error::integration::APIClientError` instead")]
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
///
/// **DEPRECATED**: Use `universal_error::integration::APIClientError` instead.
#[deprecated(since = "0.2.0", note = "Use `universal_error::integration::APIClientError` instead")]
#[derive(Debug, Error)]
pub enum ApiError {
    /// HTTP response error with status code and message
    #[error("API error ({status}): {message}")]
    HttpResponse {
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
