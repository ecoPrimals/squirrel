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

pub use squirrel_core::error::Result;
pub use commands as commands_crate;

// Re-export the Command and CommandResult from the commands crate
pub use ::commands::Command;
pub use ::commands::CommandResult;

/// Re-export command registration function
pub use crate::commands::register_commands; 
