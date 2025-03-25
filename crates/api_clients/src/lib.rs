//! API Clients for interfacing with external services
//!
//! This crate provides a collection of client implementations for various
//! external APIs that Squirrel interacts with. Each client is designed to
//! provide a Rust-friendly interface to the corresponding service.

#![warn(missing_docs)]
#![warn(rustdoc::missing_doc_code_examples)]

pub mod auth;
pub mod github;
pub mod http;

mod error;

pub use error::Error;

/// Common result type for API client operations
pub type Result<T> = std::result::Result<T, Error>;

/// Re-export of key traits and structures that define the API client interface
pub mod prelude {
    pub use crate::auth::Authenticator;
    pub use crate::error::Error;
    pub use crate::http::HttpClient;
    pub use crate::Result;
}

/// Version of the API clients crate
pub const VERSION: &str = env!("CARGO_PKG_VERSION"); 