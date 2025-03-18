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

//! Core functionality for the Squirrel command system.
//!
//! This crate provides the foundational components for building and managing commands,
//! including command registration, validation, lifecycle management, and resource control.
//!
//! Squirrel is a high-performance data processing and machine learning framework.
//! This library provides core functionality for data processing, machine learning,
//! and distributed computing using the Machine Context Protocol (MCP).

pub mod app;
/// Machine Context Protocol implementation
pub mod mcp;
pub mod error;
/// Command processing and lifecycle management
pub mod commands;
/// Context system for state and data management
pub mod context;
/// Adapter for connecting context system with MCP
pub mod context_adapter;
pub mod monitoring;

pub use app::Core;
pub use mcp::{MCP, SecurityConfig, SecurityManager, Credentials};
pub use error::SquirrelError;
pub use commands::CommandRegistry;

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
pub mod test_utils;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_initialization() {
        let core = Core::new();
        assert_eq!(core.version(), VERSION);
    }

    // Comment out the old MCP test that uses the singleton pattern
    /* 
    #[tokio::test]
    async fn test_mcp_initialization() {
        let mcp = MCP::default();
        let config = mcp.get_config().await
            .expect("Failed to get MCP configuration");
        assert_eq!(config.version, "1.0");
    }
    */
}