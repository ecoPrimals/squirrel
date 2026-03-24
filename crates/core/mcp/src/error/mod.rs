// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

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
/// Alert-related errors.
pub mod alert;
/// Client connection and request errors.
pub mod client;
/// Configuration errors.
pub mod config;
/// Connection and transport errors.
pub mod connection;
/// Context-related errors.
pub mod context;
/// Context error types.
pub mod context_err;
/// Standardized error context trait.
pub mod context_trait;
/// Error usage examples.
pub mod examples;
/// Handler and request processing errors.
pub mod handler;
/// Integration errors.
pub mod integration;
/// Plugin errors.
pub mod plugin;
/// Port and binding errors.
pub mod port;
/// Production environment errors.
pub mod production;
/// Protocol-level errors.
pub mod protocol_err;
/// RBAC and authorization errors.
pub mod rbac;
/// Registry errors.
pub mod registry;
/// Session errors.
pub mod session;
/// Task execution errors.
pub mod task;
/// Tool invocation errors.
pub mod tool;
/// Transport layer errors.
pub mod transport;
/// Error types and context trait usage.
pub mod types;

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
