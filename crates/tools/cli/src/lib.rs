// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Squirrel CLI library
//!
//! This crate provides the command-line interface components for the Squirrel platform.
#![cfg_attr(not(test), forbid(unsafe_code))]
#![warn(missing_docs)]
//! It includes commands, formatters, and configuration management.

// Allow missing docs for internal implementation details
#![allow(missing_docs)]
#![allow(
    clippy::missing_docs_in_private_items,
    clippy::unnecessary_wraps,
    clippy::ignored_unit_patterns,
    clippy::unused_async,
    clippy::needless_pass_by_ref_mut,
    clippy::missing_errors_doc,
    clippy::doc_markdown,
    clippy::must_use_candidate,
    clippy::missing_const_for_fn,
    clippy::use_self,
    clippy::uninlined_format_args,
    clippy::redundant_closure_for_method_calls,
    clippy::needless_pass_by_value,
    clippy::module_name_repetitions,
    clippy::redundant_else,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::significant_drop_tightening,
    clippy::option_if_let_else,
    clippy::single_match_else,
    clippy::manual_string_new,
    clippy::or_fun_call,
    clippy::return_self_not_must_use,
    clippy::derive_partial_eq_without_eq,
    clippy::struct_excessive_bools,
    clippy::match_same_arms,
    clippy::unused_self,
    clippy::unnecessary_literal_bound,
    clippy::branches_sharing_code,
    clippy::cloned_instead_of_copied,
    clippy::map_unwrap_or,
    clippy::semicolon_if_nothing_returned,
    clippy::used_underscore_binding,
    clippy::wildcard_imports,
    clippy::needless_continue,
    clippy::too_many_lines,
    clippy::from_iter_instead_of_collect,
    clippy::unnested_or_patterns
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
