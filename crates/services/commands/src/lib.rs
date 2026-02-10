// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Squirrel Commands Service
#![allow(dead_code)] // Command service awaiting full wiring
//!
//! Core command processing functionality for the Squirrel MCP ecosystem.
#![deny(unsafe_code)]
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
