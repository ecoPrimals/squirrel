// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Error types for the MCP system
//!
//! This module contains all error types used throughout the MCP system.
//!
//! # Main Types
//!
//! - [`MCPError`] - The primary error type for all MCP operations
//! - [`Result<T>`] - Canonical Result type alias for MCP operations  
//! - [`ErrorSeverity`] - Severity levels for error categorization
//! - [`ErrorContext`] - Additional context metadata for errors
//!
//! # Usage
//!
//! ```ignore
//! use crate::error::{Result, MCPError};
//!
//! fn perform_operation() -> Result<String> {
//!     Ok("success".to_string())
//! }
//! ```

// Core error modules
pub mod alert;
pub mod client;
pub mod config;
pub mod connection;
pub mod context;
pub mod context_err;
pub mod context_trait; // NEW: Standardized error context trait
pub mod examples;
pub mod handler;
pub mod integration;
pub mod plugin;
pub mod port;
pub mod production;
pub mod protocol_err;
pub mod rbac;
pub mod registry;
pub mod session;
pub mod task;
pub mod tool;
pub mod transport;
pub mod types; // NEW: Error context trait usage examples

// Re-export the main error types for convenient access
pub use crate::error::alert::AlertError;
pub use crate::error::client::ClientError;
pub use crate::error::config::ConfigError;
pub use crate::error::connection::ConnectionError;
pub use crate::error::context_err::ContextError;
pub use crate::error::handler::HandlerError;
pub use crate::error::integration::IntegrationError;
pub use crate::error::plugin::PluginError;
pub use crate::error::port::PortErrorKind;
pub use crate::error::protocol_err::ProtocolError;
pub use crate::error::rbac::RBACError;
pub use crate::error::registry::RegistryError;
pub use crate::error::session::SessionError;
pub use crate::error::task::TaskError;
pub use crate::error::tool::ToolError;
pub use crate::error::transport::TransportError;

// Re-export all types from the types module (includes MCPError, Result, ErrorSeverity, etc.)
pub use crate::error::types::*;

// Re-export error context trait for convenient access
pub use context_trait::{ErrorContextTrait, ResultContextExt, WithContext};

// Export test utilities
#[cfg(test)]
pub mod tests;
