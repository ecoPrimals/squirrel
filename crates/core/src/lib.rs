#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]
#![warn(clippy::undocumented_unsafe_blocks)]
#![doc(html_root_url = "https://docs.rs/squirrel-core")]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![allow(clippy::needless_raw_string_hashes)]
#![warn(clippy::todo)]
#![cfg_attr(test, allow(clippy::unwrap_used))]
#![cfg_attr(test, allow(clippy::expect_used))]

//! Core functionality for the Squirrel project.
//!
//! This crate provides the core functionality for the Squirrel project,
//! including application state management, messaging, and command processing.

// Only include the modules needed for DI testing
#[cfg(feature = "di-tests")]
pub mod error;

#[cfg(feature = "di-tests")]
pub mod app;

#[cfg(feature = "di-tests")]
pub mod mcp;

// Include all modules when not using the di-tests feature
#[cfg(not(feature = "di-tests"))]
pub mod error;

#[cfg(not(feature = "di-tests"))]
pub mod app;

#[cfg(not(feature = "di-tests"))]
/// Command-line interface and command handling functionality
///
/// This module provides the command-line interface for Squirrel,
/// including command parsing, execution, and result handling.
pub mod commands;

#[cfg(not(feature = "di-tests"))]
pub mod context;

#[cfg(not(feature = "di-tests"))]
pub mod context_adapter;

#[cfg(not(feature = "di-tests"))]
pub mod mcp;

#[cfg(not(feature = "di-tests"))]
pub mod monitoring;

#[cfg(not(feature = "di-tests"))]
#[cfg(test)]
pub mod test_utils;

// Only include these pub use statements when not using di-tests
#[cfg(not(feature = "di-tests"))]
pub use mcp::{MCP, SecurityConfig, SecurityManager, Credentials};

#[cfg(not(feature = "di-tests"))]
pub use commands::CommandRegistry;

// Include the appropriate pub use statements for di-tests
#[cfg(feature = "di-tests")]
pub use app::App as Core;

/// The current version of the library
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The authors of the library
pub const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");

/// A brief description of the library
pub const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

/// Module containing build-time information
#[allow(clippy::needless_raw_string_hashes)]
pub mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

/// MCP module tests
#[cfg(test)]
// mod adapter_tests;

#[cfg(test)]
#[cfg(not(feature = "di-tests"))]
mod tests {
    use super::VERSION;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
}

#[cfg(test)]
#[cfg(feature = "di-tests")]
mod tests {
    use super::*;
    use app::AppConfig;

    #[test]
    fn test_version() {
        let config = AppConfig::default();
        let _core = Core::new(config);
        assert_eq!(VERSION, env!("CARGO_PKG_VERSION"));
    }
}