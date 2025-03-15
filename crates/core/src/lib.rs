#![deny(missing_docs)]
#![deny(clippy::missing_docs_in_private_items)]
#![warn(clippy::undocumented_unsafe_blocks)]

//! Squirrel is a high-performance data processing and machine learning framework.
//!
//! This library provides core functionality for data processing, machine learning,
//! and distributed computing using the Machine Context Protocol (MCP).

pub mod app;
pub mod mcp;
pub mod error;

pub use app::{Core, Result};
pub use mcp::{MCP, SecurityConfig, SecurityManager, Credentials};
pub use error::{SquirrelError, MCPError, SecurityError};

/// The current version of the library
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The authors of the library
pub const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");

/// A brief description of the library
pub const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

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