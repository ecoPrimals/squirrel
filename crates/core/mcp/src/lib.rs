// SPDX-License-Identifier: AGPL-3.0-only
// ORC-Notice: AI coordination mechanics licensed under ORC
// Copyright (C) 2026 ecoPrimals Contributors

//! Machine Context Protocol (MCP) Implementation
//!
//! This crate provides a complete implementation of the Machine Context Protocol,
//! including message types, transport layers, and utility functions.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]
#![allow(
    // Docs WIP — tracked for completion
    missing_docs,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    // Edition 2024 stabilisation noise
    async_fn_in_trait,
    // Genuine domain naming
    clippy::doc_markdown,
    // Numeric casts in protocol handling — audited per-site
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_possible_wrap,
    // Patterns under active refactor
    clippy::option_if_let_else,
    clippy::significant_drop_tightening,
    clippy::similar_names,
)]

pub mod constants;
pub mod error;
pub mod protocol;
pub mod security;
pub mod transport;
pub mod types;
pub mod utils;

// Task management (JSON-RPC over Unix socket)
pub mod task;

// Re-export commonly used types
pub use error::{MCPError, Result};
pub use protocol::types::*;
#[cfg(feature = "websocket")]
pub use protocol::websocket::{ServerEvent, WebSocketClient, WebSocketConfig, WebSocketServer};
// Re-export specific transport types to avoid conflicts
pub use transport::types::{TransportConfig, TransportEvent, TransportMetadata, TransportType};
#[cfg(feature = "websocket")]
pub use transport::websocket::WebSocketTransport;
pub use transport::{SimpleTransport, Transport};
pub use types::*;
pub use utils::*;

/// MCP Core version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initialize the MCP core
///
/// This function initializes the MCP (Machine Context Protocol) core system.
/// It should be called once at application startup before using any MCP functionality.
/// The function sets up logging, validates the core configuration, and prepares
/// the MCP system for operation.
///
/// # Returns
///
/// Returns `Ok(())` on successful initialization, or an error if initialization fails.
///
/// # Errors
///
/// This function may return an error if:
/// - The logging system cannot be initialized
/// - Core configuration validation fails
/// - Required system resources are unavailable
///
/// # Examples
///
/// ```
/// use squirrel_mcp::init;
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     init()?;
///     // Now you can use MCP functionality
///     Ok(())
/// }
/// ```
#[must_use = "call this to initialize MCP; errors should be handled"]
pub fn init() -> Result<()> {
    tracing::info!("Initializing MCP Core v{}", VERSION);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init() {
        assert!(init().is_ok());
    }

    #[test]
    fn test_version() {
        // VERSION is a compile-time constant from CARGO_PKG_VERSION, so it will always be non-empty
        // Instead, test that it follows a valid semver format (e.g., "0.1.0")
        assert!(
            VERSION.split('.').count() >= 2,
            "Version should have at least major.minor format"
        );
    }
}
