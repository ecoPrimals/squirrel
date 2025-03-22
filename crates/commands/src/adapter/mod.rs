//! Command system adapters for external integration
//!
//! This module provides adapters for integrating the command system
//! with external components and protocols.

pub mod mcp;
pub mod helper;
pub mod tests;

// Re-export adapter types for convenience
pub use self::mcp::{McpCommandAdapter, McpCommandRequest, McpCommandResponse, McpExecutionContext, McpIntegrationError};
pub use self::helper::{CommandRegistryAdapter, create_initialized_registry_adapter, create_empty_registry_adapter, AdapterHelperError, AdapterHelperResult}; 