// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Plugin adapter and integration test helpers

use async_trait::async_trait;
use std::sync::{Arc, RwLock};

use crate::commands::{CommandAdapter, RegistryAdapter};
use crate::types::{Command, CommandResult};

/// Plugin adapter implementation
#[derive(Debug)]
pub struct PluginAdapter {
    adapter: Arc<RwLock<RegistryAdapter>>,
    plugin_id: String,
    version: String,
}

impl Default for PluginAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginAdapter {
    /// Create a new plugin adapter
    #[must_use]
    pub fn new() -> Self {
        Self {
            adapter: Arc::new(RwLock::new(RegistryAdapter::new())),
            plugin_id: "commands".to_string(),
            version: "1.0.0".to_string(),
        }
    }

    /// Get the plugin identifier
    #[must_use]
    pub fn plugin_id(&self) -> &str {
        &self.plugin_id
    }

    /// Get the plugin version
    #[must_use]
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Register a command in the plugin adapter
    #[allow(clippy::unused_async)]
    pub async fn register_command(&self, command: Arc<dyn Command>) -> CommandResult<()> {
        let mut adapter = self.adapter.write().unwrap();
        adapter.register(command.name(), command.clone())
    }

    /// Get list of registered commands
    #[allow(clippy::unused_async)]
    pub async fn get_commands(&self) -> CommandResult<Vec<String>> {
        let adapter = self.adapter.read().unwrap();
        adapter.list_commands()
    }
}

#[async_trait]
impl CommandAdapter for PluginAdapter {
    async fn execute(&self, command: &str, args: Vec<String>) -> CommandResult<String> {
        let adapter = self.adapter.read().unwrap();
        adapter.execute(command, args)
    }

    async fn get_help(&self, command: &str) -> CommandResult<String> {
        let adapter = self.adapter.read().unwrap();
        adapter.get_help(command)
    }

    async fn list_commands(&self) -> CommandResult<Vec<String>> {
        let adapter = self.adapter.read().unwrap();
        adapter.list_commands()
    }
}

/// `MockAdapter` trait for testing and example purposes
#[async_trait]
pub trait MockAdapter: Send + Sync {
    /// Execute a command with given arguments
    async fn execute(&self, command: &str, args: Vec<String>) -> CommandResult<String>;

    /// Get help information for a command
    async fn get_help(&self, command: &str) -> CommandResult<String>;

    /// List all available commands
    async fn list_commands(&self) -> CommandResult<Vec<String>>;
}

#[async_trait]
impl MockAdapter for RegistryAdapter {
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

#[async_trait]
impl MockAdapter for crate::auth::McpAdapter {
    async fn execute(&self, command: &str, args: Vec<String>) -> CommandResult<String> {
        CommandAdapter::execute(self, command, args).await
    }

    async fn get_help(&self, command: &str) -> CommandResult<String> {
        CommandAdapter::get_help(self, command).await
    }

    async fn list_commands(&self) -> CommandResult<Vec<String>> {
        CommandAdapter::list_commands(self).await
    }
}

#[async_trait]
impl MockAdapter for PluginAdapter {
    async fn execute(&self, command: &str, args: Vec<String>) -> CommandResult<String> {
        CommandAdapter::execute(self, command, args).await
    }

    async fn get_help(&self, command: &str) -> CommandResult<String> {
        CommandAdapter::get_help(self, command).await
    }

    async fn list_commands(&self) -> CommandResult<Vec<String>> {
        CommandAdapter::list_commands(self).await
    }
}

/// Test the adapter with different implementations
///
/// This function demonstrates how the adapter pattern allows for polymorphic
/// usage of different adapter implementations through a common interface.
pub async fn test_polymorphic_adapter<A: CommandAdapter + ?Sized>(
    adapter: &A,
    command: &str,
    args: Vec<String>,
) -> CommandResult<String> {
    adapter.execute(command, args).await
}
