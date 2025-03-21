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

/// Re-export types from dependencies
pub use squirrel_commands::{Command, CommandResult};

/// Re-export command registration function
pub use commands::register_commands; 