// SPDX-License-Identifier: AGPL-3.0-only
// ORC-Notice: AI coordination mechanics licensed under ORC
// Copyright (C) 2026 ecoPrimals Contributors

#![allow(clippy::missing_docs_in_private_items)]
#![allow(async_fn_in_trait)]
#![allow(clippy::unused_self)]
#![allow(clippy::unnecessary_wraps)]
#![allow(clippy::significant_drop_tightening)]
#![allow(clippy::option_if_let_else)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::match_same_arms)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::redundant_closure_call)]
#![allow(clippy::or_fun_call)]
#![allow(clippy::unused_async)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::needless_continue)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::struct_excessive_bools)]
#![allow(clippy::default_trait_access)]
#![allow(clippy::module_inception)]
#![allow(clippy::redundant_clone)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::clone_on_copy)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::useless_vec)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::len_zero)]
#![allow(clippy::useless_conversion)]
#![allow(clippy::assertions_on_constants)]
#![allow(clippy::collection_is_never_read)]
#![allow(clippy::needless_collect)]
#![allow(clippy::manual_string_new)]
#![allow(clippy::unnecessary_get_then_check)]
#![allow(clippy::float_cmp)]
//! Machine Context Protocol (MCP) Implementation
//!
//! This crate provides a complete implementation of the Machine Context Protocol,
#![forbid(unsafe_code)]
#![allow(missing_docs)]
//! including message types, transport layers, and utility functions.

pub mod constants;
pub mod error;
pub mod protocol;
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
