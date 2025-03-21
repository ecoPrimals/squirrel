//! Machine Context Protocol (MCP) implementation for Squirrel
//!
//! This crate provides the implementation of the Machine Context Protocol,
//! a system for secure communication and context management between systems.

#![allow(dead_code)] // Temporarily allow dead code during migration


/// MCP context manager
pub mod context_manager;

/// Error types and error handling
pub mod error;

/// Protocol-related functionality
pub mod protocol;

/// Monitoring and metrics
pub mod monitoring;

/// Security and authentication
pub mod security;

/// Persistence layer
pub mod persistence;

/// Synchronization
pub mod sync;

/// Common types
pub mod types;

/// Configuration module
pub mod config;

/// Re-export common types from the error module
pub use error::{MCPError, Result};

/// Re-export commonly used types
pub use protocol::ProtocolConfig;
pub use security::{SecurityManager, Credentials, Session};
pub use context_manager::Context;
pub use types::{EncryptionFormat, SecurityLevel};

/// Adapter for MCP operations
pub mod adapter;
pub use adapter::{MCPAdapter, MCPInterface};

/// Re-export the configuration type
pub use config::McpConfig as MCPConfig;

#[cfg(test)]
mod tests; 