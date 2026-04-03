// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]
#![warn(missing_docs)]

//! Squirrel CLI library
//!
//! This crate provides the command-line interface components for the Squirrel platform.
//! It includes commands, formatters, and configuration management.

#![expect(
    clippy::uninlined_format_args,
    clippy::missing_errors_doc,
    clippy::use_self,
    clippy::must_use_candidate,
    clippy::missing_const_for_fn,
    clippy::option_if_let_else,
    clippy::doc_markdown,
    clippy::single_match_else,
    clippy::ignored_unit_patterns,
    clippy::unused_self,
    clippy::significant_drop_tightening,
    clippy::redundant_closure_for_method_calls,
    clippy::unnecessary_wraps,
    clippy::return_self_not_must_use,
    clippy::needless_pass_by_value,
    clippy::map_unwrap_or,
    clippy::match_same_arms,
    clippy::from_iter_instead_of_collect,
    clippy::cloned_instead_of_copied,
    reason = "CLI crate; progressive lint tightening"
)]

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

/// Runtime status helpers for the `squirrel status` CLI command.
pub mod status;

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
pub type Result<T> = std::result::Result<T, anyhow::Error>;

pub use squirrel_commands as commands_crate;

// Re-export the Command and CommandResult from squirrel_commands crate
pub use squirrel_commands::Command;
pub use squirrel_commands::CommandResult;

/// Re-export command registration function
pub use crate::commands::register_commands;

// Re-export from local modules
pub use crate::commands::context::CommandContext;
pub use crate::commands::executor::ExecutionContext;
