// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Squirrel CLI library
//!
//! This crate provides the command-line interface components for the Squirrel platform.
#![deny(unsafe_code)]
//! It includes commands, formatters, and configuration management.

// Allow missing docs for internal implementation details
#![allow(missing_docs)]
#![allow(clippy::missing_docs_in_private_items)]

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
    pub use crate::commands::adapter::registry::CommandRegistryAdapter as RegistryAdapter;
    pub use crate::commands::adapter::{CommandAdapter, CommandAdapterTrait};
    pub use crate::commands::registry::CommandRegistry;
    pub use async_trait::async_trait;
}

/// Error handling re-exports
pub mod error {
    pub use crate::commands::adapter::error::AdapterError;
    pub use crate::commands::adapter::error::AdapterResult;
    pub use crate::commands::error::CommandError;
}

// Define our own Result type
/// Result type alias for CLI operations
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub use squirrel_commands as commands_crate;

// Re-export the Command and CommandResult from squirrel_commands crate
pub use squirrel_commands::Command;
pub use squirrel_commands::CommandResult;

/// Re-export command registration function
pub use crate::commands::register_commands;

// Re-export from local modules
pub use crate::commands::context::CommandContext;
pub use crate::commands::executor::ExecutionContext;
