#![deny(missing_docs)]
#![deny(clippy::missing_docs_in_private_items)]
#![warn(clippy::undocumented_unsafe_blocks)]
#![doc(html_root_url = "https://docs.rs/squirrel-core")]
#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![allow(clippy::needless_raw_string_hashes)]

//! Core functionality for the Squirrel command system.
//!
//! This crate provides the foundational components for building and managing commands,
//! including command registration, validation, lifecycle management, and resource control.
//!
//! Squirrel is a high-performance data processing and machine learning framework.
//! This library provides core functionality for data processing, machine learning,
//! and distributed computing using the Machine Context Protocol (MCP).

pub mod app;
pub mod mcp;
pub mod error;

/// Command system implementation including command traits, registry, validation, and lifecycle management.
/// 
/// This module contains the core components for:
/// - Command trait definition and basic implementations
/// - Command registry for storing and retrieving commands
/// - Validation rules and validation system
/// - Lifecycle management and hooks
pub mod commands;

pub use app::{Core, Result};
pub use mcp::MCP;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_initialization() {
        let core = Core::new();
        assert_eq!(core.version(), VERSION);
    }

    #[tokio::test]
    async fn test_mcp_initialization() {
        let mcp = MCP::default();
        let config = mcp.get_config().await.unwrap();
        assert_eq!(config.version, "1.0");
    }
} 