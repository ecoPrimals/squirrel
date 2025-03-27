//! Adapter Pattern Implementation for Testing
//!
//! This crate provides a standalone implementation of the adapter pattern in Rust,
//! with a focus on testing and demonstration purposes. It's designed to be completely
//! independent of the main codebase, allowing it to be used for learning and testing
//! the adapter pattern without any dependencies on the main code.
//!
//! # Overview
//!
//! The adapter pattern is a structural design pattern that allows objects with
//! incompatible interfaces to collaborate. This crate demonstrates several adaptations:
//!
//! 1. **Command Registry Adapter** - Adapts a command registry for asynchronous operations
//! 2. **MCP Command Adapter** - Adapts commands for Machine Context Protocol (MCP) with authentication
//! 3. **Plugin Adapter** - Adapts commands for a plugin system
//!
//! # Features
//!
//! - Thread-safe command registry with Arc/Mutex
//! - Asynchronous execution of commands
//! - Authentication and authorization for MCP commands
//! - Comprehensive test suite
//!
//! # Example
//!
//! ```
//! use adapter_tests::{TestCommand, CommandRegistryAdapter, Auth, McpCommandAdapter};
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a registry adapter
//! let adapter = CommandRegistryAdapter::new();
//!
//! // Create and register a test command
//! let cmd = TestCommand::new("hello", "Says hello", "Hello, world!");
//! adapter.register_command(Arc::new(cmd))?;
//!
//! // Execute the command
//! let result = adapter.execute("hello", vec![]).await?;
//! println!("Result: {}", result);  // Output: "Hello, world!"
//!
//! // Using the MCP adapter with authentication
//! let mcp_adapter = McpCommandAdapter::new();
//! let cmd = TestCommand::new("secure", "Secure command", "Secret data");
//! mcp_adapter.register_command(Arc::new(cmd))?;
//!
//! // Execute with authentication
//! let result = mcp_adapter.execute_with_auth(
//!     "secure",
//!     vec![],
//!     Auth::User("admin".to_string(), "password".to_string())
//! ).await?;
//! println!("Authenticated result: {}", result);
//! # Ok(())
//! # }
//! ```

// Modules
mod command;
mod registry;
mod error;
pub mod adapter;

// Re-exports for public API
pub use command::{MockCommand, TestCommand};
pub use registry::MockCommandRegistry;
pub use error::{AdapterError, AdapterResult};
pub use adapter::{
    CommandRegistryAdapter,
    McpCommandAdapter,
    CommandsPluginAdapter,
    MockAdapter,
    Auth,
    create_registry_adapter,
    create_mcp_adapter,
    create_plugin_adapter,
};
