// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

#![forbid(unsafe_code)]

//! Squirrel Commands Service
#![expect(dead_code, reason = "Command service awaiting full wiring")]
#![expect(
    clippy::redundant_else,
    clippy::uninlined_format_args,
    clippy::unused_async,
    clippy::needless_pass_by_ref_mut,
    clippy::missing_const_for_fn,
    clippy::missing_errors_doc,
    clippy::must_use_candidate,
    clippy::use_self,
    clippy::doc_markdown,
    clippy::significant_drop_tightening,
    clippy::implicit_clone,
    clippy::option_if_let_else,
    clippy::cloned_instead_of_copied,
    clippy::cast_precision_loss,
    clippy::if_not_else,
    clippy::return_self_not_must_use,
    clippy::suboptimal_flops,
    clippy::cast_possible_truncation,
    clippy::single_match_else,
    clippy::ignored_unit_patterns,
    clippy::missing_fields_in_debug,
    clippy::cast_possible_wrap,
    clippy::needless_pass_by_value,
    clippy::cast_sign_loss,
    clippy::map_unwrap_or,
    clippy::format_push_string,
    clippy::unnecessary_literal_bound,
    clippy::unused_self,
    reason = "Progressive lint tightening; tracked for refactor"
)]
//!
//! Core command processing functionality for the Squirrel MCP ecosystem.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]
#![warn(missing_docs)]
//! This service handles basic command execution and validation.

use std::sync::Arc;

use anyhow::Result as AnyhowResult;
use serde::{Deserialize, Serialize};
use tracing::debug;

// Core imports (keeping only what remains in Squirrel)
use squirrel_context::ContextManager;

// Modules (keeping only core functionality)
pub mod builtin;
pub mod error;
pub mod factory;
pub mod history;
/// Command hooks that run during lifecycle stages (pre/post execution, validation, etc.).
pub mod hooks;
pub mod journal;
/// Lifecycle stages and hooks for command processing.
pub mod lifecycle;
pub mod observability;
pub mod registry;
/// Resource limits and management for command execution.
pub mod resources;
pub mod suggestions;
pub mod transaction;
/// Validation rules and context for commands before execution.
pub mod validation;

// Re-export key functions and types for easier access
pub use builtin::{
    EchoCommand, ExitCommand, HelpCommand, HistoryCommand, KillCommand, VersionCommand,
};
pub use error::CommandError;
pub use factory::{
    DefaultCommandRegistryFactory, create_command_registry, create_command_registry_with_plugin,
};
pub use registry::{Command, CommandRegistry, CommandResult};

/// Result type for command operations
pub type Result<T> = std::result::Result<T, CommandError>;

/// Core command processing service
pub struct CommandsService {
    context_manager: Arc<ContextManager>,
}

impl CommandsService {
    /// Create a new commands service
    pub fn new(context_manager: Arc<ContextManager>) -> Self {
        Self { context_manager }
    }

    /// Process a basic command
    pub async fn process_command(&self, command: &str, args: Vec<String>) -> AnyhowResult<String> {
        debug!("Processing command: {command} with args: {args:?}");

        match command {
            "ping" => Ok("pong".to_string()),
            "version" => Ok(env!("CARGO_PKG_VERSION").to_string()),
            "status" => self.get_status().await,
            _ => Err(anyhow::anyhow!("Unknown command: {}", command)),
        }
    }

    /// Get service status
    async fn get_status(&self) -> AnyhowResult<String> {
        Ok("Commands service is running".to_string())
    }
}

/// Basic command metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandMetadata {
    /// Command identifier.
    pub name: String,
    /// Human-readable description of what the command does.
    pub description: String,
    /// Semantic version of the command.
    pub version: String,
}

#[cfg(test)]
mod lib_tests {
    use super::*;

    #[test]
    fn test_command_metadata_serde() {
        let metadata = CommandMetadata {
            name: "test-cmd".to_string(),
            description: "A test command".to_string(),
            version: "1.0.0".to_string(),
        };
        let json = serde_json::to_string(&metadata).unwrap();
        let deserialized: CommandMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, "test-cmd");
        assert_eq!(deserialized.description, "A test command");
        assert_eq!(deserialized.version, "1.0.0");
    }

    #[test]
    fn test_command_metadata_clone() {
        let metadata = CommandMetadata {
            name: "clone-test".to_string(),
            description: "desc".to_string(),
            version: "0.1.0".to_string(),
        };
        let cloned = metadata.clone();
        assert_eq!(cloned.name, metadata.name);
        assert_eq!(cloned.version, metadata.version);
    }

    #[tokio::test]
    async fn test_commands_service_ping() {
        let context_manager = Arc::new(ContextManager::default());
        let service = CommandsService::new(context_manager);
        let result = service.process_command("ping", vec![]).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "pong");
    }

    #[tokio::test]
    async fn test_commands_service_version() {
        let context_manager = Arc::new(ContextManager::default());
        let service = CommandsService::new(context_manager);
        let result = service.process_command("version", vec![]).await;
        assert!(result.is_ok());
        assert!(!result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_commands_service_status() {
        let context_manager = Arc::new(ContextManager::default());
        let service = CommandsService::new(context_manager);
        let result = service.process_command("status", vec![]).await;
        assert!(result.is_ok());
        assert!(result.unwrap().contains("running"));
    }

    #[tokio::test]
    async fn test_commands_service_unknown_command() {
        let context_manager = Arc::new(ContextManager::default());
        let service = CommandsService::new(context_manager);
        let result = service.process_command("nonexistent", vec![]).await;
        assert!(result.is_err());
    }
}
