//! API client functionality for Squirrel
//!
//! This crate provides common functionality for implementing API clients
//! including HTTP client implementations, authentication, and error handling.

#![warn(missing_docs)]
#![warn(rustdoc::missing_doc_code_examples)]

use std::result;
use thiserror::Error;
use serde::Serialize;

pub mod auth;
pub mod github;
pub mod http;
mod error;

/// A type alias for Results from API client operations
pub type Result<T> = result::Result<T, Error>;

/// Pagination parameters for API requests
#[derive(Debug, Clone, Copy, Serialize)]
pub struct Pagination {
    /// Page number (1-indexed)
    pub page: u32,
    /// Number of items per page
    pub per_page: u32,
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            page: 1,
            per_page: 30,
        }
    }
}

impl Pagination {
    /// Create a new pagination with the given page and per_page
    pub fn new(page: u32, per_page: u32) -> Self {
        Self { page, per_page }
    }
    
    /// Get the next page of results
    pub fn next_page(&self) -> Self {
        Self {
            page: self.page + 1,
            per_page: self.per_page,
        }
    }
}

/// Error type for API client operations
#[derive(Debug, Error)]
pub enum Error {
    /// HTTP request error
    #[error("HTTP request error: {0}")]
    RequestError(#[from] reqwest::Error),
    
    /// API response error
    #[error("API error {0}: {1}")]
    ResponseError(u16, String),
    
    /// Authentication error
    #[error("Authentication error: {0}")]
    AuthError(String),
    
    /// Rate limit error
    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),
    
    /// URL parsing error
    #[error("URL parsing error: {0}")]
    Url(#[from] url::ParseError),
    
    /// JSON serialization/deserialization error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    
    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    /// Other error
    #[error("{0}")]
    Other(String),
}

/// Re-export of key traits and structures that define the API client interface
pub mod prelude {
    pub use crate::auth::Authenticator;
    pub use crate::error::Error;
    pub use crate::http::HttpClient;
    pub use crate::Result;
}

/// Version of the API clients crate
pub const VERSION: &str = env!("CARGO_PKG_VERSION"); 