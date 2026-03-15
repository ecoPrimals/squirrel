// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Command registry and registry adapter implementation

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

use crate::types::{Command, CommandError, CommandResult};

/// Command registry to store and execute commands
#[derive(Debug)]
pub struct CommandRegistry {
    commands: HashMap<String, Arc<dyn Command>>,
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandRegistry {
    /// Create a new command registry
    #[must_use]
    pub fn new() -> Self {
        Self {
            commands: HashMap::new(),
        }
    }

    /// Register a command in the registry
    pub fn register(&mut self, name: &str, command: Arc<dyn Command>) -> CommandResult<()> {
        self.commands.insert(name.to_string(), command);
        Ok(())
    }

    /// Execute a registered command
    pub fn execute(&self, name: &str, args: Vec<String>) -> CommandResult<String> {
        self.commands.get(name).map_or_else(
            || Err(CommandError::NotFound(name.to_string())),
            |cmd| cmd.execute(args),
        )
    }

    /// Get help text for a command
    pub fn get_help(&self, name: &str) -> CommandResult<String> {
        self.commands.get(name).map_or_else(
            || Err(CommandError::NotFound(name.to_string())),
            |cmd| Ok(format!("{}: {}", cmd.name(), cmd.description())),
        )
    }

    /// List all registered commands
    pub fn list_commands(&self) -> CommandResult<Vec<String>> {
        Ok(self.commands.keys().cloned().collect())
    }
}

/// Adapter interface for command operations
#[async_trait]
pub trait CommandAdapter: Send + Sync {
    /// Execute a command with given arguments
    async fn execute(&self, command: &str, args: Vec<String>) -> CommandResult<String>;

    /// Get help information for a command
    async fn get_help(&self, command: &str) -> CommandResult<String>;

    /// List all available commands
    async fn list_commands(&self) -> CommandResult<Vec<String>>;
}

/// Registry adapter implementation
#[derive(Debug)]
pub struct RegistryAdapter {
    commands: HashMap<String, Arc<dyn Command>>,
}

impl Default for RegistryAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl RegistryAdapter {
    /// Create a new registry adapter
    #[must_use]
    pub fn new() -> Self {
        Self {
            commands: HashMap::new(),
        }
    }

    /// Register a command in the registry
    pub fn register(&mut self, name: &str, command: Arc<dyn Command>) -> CommandResult<()> {
        self.commands.insert(name.to_string(), command);
        Ok(())
    }

    /// Execute a registered command by name
    pub fn execute(&self, name: &str, args: Vec<String>) -> CommandResult<String> {
        self.commands.get(name).map_or_else(
            || Err(CommandError::NotFound(name.to_string())),
            |cmd| cmd.execute(args),
        )
    }

    /// Get help information for a registered command
    pub fn get_help(&self, name: &str) -> CommandResult<String> {
        self.commands.get(name).map_or_else(
            || Err(CommandError::NotFound(name.to_string())),
            |cmd| Ok(format!("{}: {}", cmd.name(), cmd.description())),
        )
    }

    /// List all registered command names
    pub fn list_commands(&self) -> CommandResult<Vec<String>> {
        Ok(self.commands.keys().cloned().collect())
    }
}

#[async_trait]
impl CommandAdapter for RegistryAdapter {
    async fn execute(&self, command: &str, args: Vec<String>) -> CommandResult<String> {
        self.execute(command, args)
    }

    async fn get_help(&self, command: &str) -> CommandResult<String> {
        self.get_help(command)
    }

    async fn list_commands(&self) -> CommandResult<Vec<String>> {
        self.list_commands()
    }
}
