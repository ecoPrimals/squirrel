#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)] // Temporarily allow while we fix systematically
#![allow(clippy::must_use_candidate)] // Temporarily allow while we fix systematically
#![allow(clippy::use_self)] // Temporarily allow while we fix systematically
#![allow(clippy::significant_drop_tightening)] // Temporarily allow while we fix systematically
#![allow(clippy::missing_const_for_fn)] // Temporarily allow while we fix systematically
#![allow(clippy::redundant_clone)] // Temporarily allow while we fix systematically
#![allow(clippy::needless_pass_by_ref_mut)] // Temporarily allow while we fix systematically
#![allow(clippy::option_if_let_else)] // Temporarily allow while we fix systematically
#![allow(clippy::significant_drop_in_scrutinee)] // Temporarily allow while we fix systematically
#![allow(clippy::derive_partial_eq_without_eq)] // Temporarily allow while we fix systematically
#![allow(clippy::suboptimal_flops)] // Temporarily allow while we fix systematically
#![allow(clippy::future_not_send)] // Temporarily allow while we fix systematically
#![allow(missing_docs)]
#![warn(clippy::undocumented_unsafe_blocks)]

//! Squirrel is a high-performance data processing and machine learning framework.
//!
//! This library provides core functionality for data processing, machine learning,
//! and distributed computing using the Machine Context Protocol (MCP).

// Explicitly use external crates to avoid unused dependency warnings
use async_trait as _;
use axum as _;
use chrono as _;
use futures as _;
use handlebars as _;
use hyper as _;
use lazy_static as _;
use lettre as _;
use metrics as _;
use metrics_exporter_prometheus as _;
use prometheus as _;
use reqwest as _;
use serde as _;
use serde_json as _;
use sysinfo as _;
use tempfile as _;
use thiserror as _;
use time as _;
use tokio_tungstenite as _;
use tower as _;
use tower_http as _;
use tracing as _;
use uuid as _;

pub mod app;
pub mod commands;
pub mod context;
pub mod error;
pub mod monitoring;
pub mod mcp;

pub use error::Result;
pub use mcp::MCP;
pub use error::SquirrelError;
pub use crate::app::context::Context;
pub use commands::CommandHandler;
pub use monitoring::MonitoringService;

/// The current version of the library
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The authors of the library
pub const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");

/// A brief description of the library
pub const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

/// Core functionality for the Squirrel library
pub struct Core {
    version: String,
}

impl Core {
    /// Create a new Core instance
    pub fn new() -> Self {
        Self {
            version: VERSION.to_string(),
        }
    }

    /// Get the version of the Core
    pub fn version(&self) -> &str {
        &self.version
    }
}

impl Default for Core {
    fn default() -> Self {
        Self::new()
    }
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