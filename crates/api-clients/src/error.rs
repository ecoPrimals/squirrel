//! Error handling for API clients
//!
//! This module defines the common error types that can occur when
//! interacting with external APIs.

use thiserror::Error;
use std::fmt;
use reqwest::StatusCode;

/// Error types for the API clients
#[derive(Debug)]
pub enum Error {
    /// HTTP client error
    Http(reqwest::Error),
    /// Rate limit error
    RateLimit {
        /// Time to wait in seconds before retrying
        retry_after: u64,
    },
    /// Authentication error
    Auth(String),
    /// Request validation error
    Validation(String),
    /// Response parsing error
    Parse(String),
    /// Configuration error
    Config(String),
    /// Circuit breaker is open error
    CircuitOpen,
    /// Other error type
    Other(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Http(e) => write!(f, "HTTP error: {}", e),
            Error::RateLimit { retry_after } => {
                write!(f, "Rate limit exceeded. Retry after {} seconds", retry_after)
            }
            Error::Auth(msg) => write!(f, "Authentication error: {}", msg),
            Error::Validation(msg) => write!(f, "Validation error: {}", msg),
            Error::Parse(msg) => write!(f, "Parse error: {}", msg),
            Error::Config(msg) => write!(f, "Configuration error: {}", msg),
            Error::CircuitOpen => write!(f, "Circuit breaker is open"),
            Error::Other(msg) => write!(f, "Other error: {}", msg),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Http(e) => Some(e),
            _ => None,
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Error::Http(e)
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