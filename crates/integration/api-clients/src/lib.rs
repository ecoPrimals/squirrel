//! API client integrations for Squirrel
//!
//! This crate provides API clients for external services that integrate
//! with the Squirrel platform. Authentication is handled by BearDog.
//!
//! ## Features
//!
//! - Anthropic API client for Claude models
//! - OpenAI API client for GPT models
//! - Unified interface for AI model interactions
//! - Error handling and retries
//! - Configurable timeouts and connection settings

#![warn(missing_docs)]
#![warn(rustdoc::missing_doc_code_examples)]

use serde::Serialize;

pub mod auth;
pub mod config;
mod error;
pub mod github;
pub mod http;

// Re-export universal error types
pub use universal_error::Result;
pub use universal_error::integration::APIClientError as Error;

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
            per_page: config::DEFAULT_PER_PAGE,
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

/// Re-export of key traits and structures that define the API client interface
pub mod prelude {
    pub use crate::auth::Authenticator;
    pub use crate::Error;
    pub use crate::http::HttpClient;
    pub use crate::Result;
}

/// Version of the API clients crate
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

mod anthropic;
mod client;
mod openai;

pub use anthropic::AnthropicClient;
pub use client::AIClient;
pub use config::{AnthropicConfig, ApiClientConfig, OpenAIConfig};
pub use openai::OpenAIClient;
