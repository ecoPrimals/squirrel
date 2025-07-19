//! Squirrel Commands Service
//!
//! Core command processing functionality for the Squirrel MCP ecosystem.
//! This service handles basic command execution and validation.

use std::sync::Arc;

use anyhow::Result as AnyhowResult;
use log::debug;
use serde::{Deserialize, Serialize};

// Core imports (keeping only what remains in Squirrel)
use squirrel_context::ContextManager;

// Modules (keeping only core functionality)
pub mod builtin;
pub mod error;
pub mod factory;
pub mod history;
pub mod hooks;
pub mod journal;
pub mod lifecycle;
pub mod observability;
pub mod registry;
pub mod resources;
pub mod suggestions;
pub mod transaction;
pub mod validation;

// Re-export key functions and types for easier access
pub use builtin::{
    EchoCommand, ExitCommand, HelpCommand, HistoryCommand, KillCommand, VersionCommand,
};
pub use error::CommandError;
pub use factory::{
    create_command_registry, create_command_registry_with_plugin, DefaultCommandRegistryFactory,
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
    pub name: String,
    pub description: String,
    pub version: String,
}
