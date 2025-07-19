//! Error types for the MCP system
//!
//! This module contains all error types used throughout the MCP system.

// Core error modules
pub mod alert;
pub mod client;
pub mod config;
pub mod connection;
pub mod context;
pub mod context_err;
pub mod handler;
pub mod integration;
pub mod plugin;
pub mod port;
pub mod protocol_err;
pub mod rbac;
pub mod registry;
pub mod session;
pub mod task;
pub mod tool;
pub mod transport;
pub mod types;

// Re-export the main error types
pub use crate::error::client::ClientError;
pub use crate::error::protocol_err::ProtocolError;
pub use crate::error::rbac::RBACError;
pub use crate::error::session::SessionError;
pub use crate::error::transport::TransportError;
pub use crate::error::types::*;

// Export test utilities
#[cfg(test)]
pub mod tests;
