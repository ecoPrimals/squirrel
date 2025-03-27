//! Squirrel CLI library
//!
//! This crate provides the command-line interface components for the Squirrel platform.
//! It includes commands, formatters, and configuration management.

/// Output formatter for CLI commands
pub mod formatter;

/// CLI configuration management
pub mod config;

/// Commands module
pub mod commands;

/// MCP module
pub mod mcp;

/// Plugin system
pub mod plugins;

/// Command adapter re-exports for testing and benchmarking
pub mod command_adapter {
    pub use crate::commands::adapter::{CommandAdapter, CommandAdapterTrait};
    pub use crate::commands::adapter::registry::CommandRegistryAdapter as RegistryAdapter;
    pub use crate::commands::registry::CommandRegistry;
    pub use async_trait::async_trait;
}

/// Error handling re-exports
pub mod error {
    pub use crate::commands::adapter::error::AdapterError;
    pub use crate::commands::adapter::error::AdapterResult;
    pub use crate::commands::error::CommandError;
}

pub use squirrel_core::error::Result;
pub use commands as commands_crate;

// Re-export the Command and CommandResult from the commands crate
pub use ::commands::Command;
pub use ::commands::CommandResult;

/// Re-export command registration function
pub use crate::commands::register_commands;

// Re-export from commands module
pub use commands::adapter;
pub use commands::context::CommandContext;
pub use commands::executor::ExecutionContext; 
