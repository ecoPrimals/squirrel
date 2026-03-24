// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

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
    #[allow(
        clippy::unused_async,
        reason = "Async trait method; required for future implementations"
    )]
    pub async fn register_command(&self, command: Arc<dyn Command>) -> CommandResult<()> {
        let mut adapter = self.adapter.write().unwrap();
        adapter.register(command.name(), command.clone())
    }

    /// Get list of registered commands
    #[allow(
        clippy::unused_async,
        reason = "Async trait method; required for future implementations"
    )]
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::McpAdapter;
    use crate::commands::RegistryAdapter;
    use crate::types::TestCommand;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_plugin_adapter_default() {
        let adapter = PluginAdapter::default();
        assert_eq!(adapter.plugin_id(), "commands");
        assert_eq!(adapter.version(), "1.0.0");
    }

    #[tokio::test]
    async fn test_plugin_adapter_new() {
        let adapter = PluginAdapter::new();
        assert_eq!(adapter.plugin_id(), "commands");
        assert_eq!(adapter.version(), "1.0.0");
    }

    #[tokio::test]
    async fn test_plugin_adapter_register_and_get_commands() {
        let adapter = PluginAdapter::new();
        let cmd = Arc::new(TestCommand::new("test-cmd", "Test", "result"));
        adapter.register_command(cmd).await.unwrap();

        let cmds = adapter.get_commands().await.unwrap();
        assert_eq!(cmds.len(), 1);
        assert_eq!(cmds[0], "test-cmd");
    }

    #[tokio::test]
    async fn test_plugin_adapter_command_adapter_execute() {
        let adapter = PluginAdapter::new();
        let cmd = Arc::new(TestCommand::new("test", "Test", "output"));
        adapter.register_command(cmd).await.unwrap();

        let result = <PluginAdapter as CommandAdapter>::execute(&adapter, "test", vec![]).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "output");
    }

    #[tokio::test]
    async fn test_plugin_adapter_command_adapter_execute_with_args() {
        let adapter = PluginAdapter::new();
        let cmd = Arc::new(TestCommand::new("echo", "Echo", "Echo"));
        adapter.register_command(cmd).await.unwrap();

        let result = <PluginAdapter as CommandAdapter>::execute(
            &adapter,
            "echo",
            vec!["x".to_string(), "y".to_string()],
        )
        .await;
        assert!(result.is_ok());
        assert!(result.unwrap().contains('x'));
    }

    #[tokio::test]
    async fn test_plugin_adapter_command_adapter_get_help() {
        let adapter = PluginAdapter::new();
        let cmd = Arc::new(TestCommand::new("help-cmd", "Help description", "help"));
        adapter.register_command(cmd).await.unwrap();

        let help = <PluginAdapter as CommandAdapter>::get_help(&adapter, "help-cmd")
            .await
            .unwrap();
        assert_eq!(help, "help-cmd: Help description");
    }

    #[tokio::test]
    async fn test_plugin_adapter_command_adapter_execute_not_found() {
        let adapter = PluginAdapter::new();
        let result = <PluginAdapter as CommandAdapter>::execute(&adapter, "missing", vec![]).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_mock_adapter_registry() {
        let mut registry = RegistryAdapter::new();
        let cmd = Arc::new(TestCommand::new("mock-cmd", "Mock", "mock result"));
        registry.register("mock-cmd", cmd).unwrap();

        let result = <RegistryAdapter as MockAdapter>::execute(&registry, "mock-cmd", vec![]).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "mock result");
    }

    #[tokio::test]
    async fn test_mock_adapter_mcp() {
        let adapter = McpAdapter::new();
        let cmd = Arc::new(TestCommand::new("mcp-cmd", "MCP", "mcp result"));
        adapter.register_command(cmd).await.unwrap();

        let result = <McpAdapter as MockAdapter>::execute(&adapter, "mcp-cmd", vec![]).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "mcp result");
    }

    #[tokio::test]
    async fn test_mock_adapter_plugin() {
        let adapter = PluginAdapter::new();
        let cmd = Arc::new(TestCommand::new("plugin-mock", "Plugin", "plugin result"));
        adapter.register_command(cmd).await.unwrap();

        let result = <PluginAdapter as MockAdapter>::execute(&adapter, "plugin-mock", vec![]).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "plugin result");
    }

    #[tokio::test]
    async fn test_test_polymorphic_adapter() {
        let mut registry = RegistryAdapter::new();
        let cmd = Arc::new(TestCommand::new("poly", "Poly", "polymorphic"));
        registry.register("poly", cmd).unwrap();

        let result = test_polymorphic_adapter(&registry, "poly", vec![]).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "polymorphic");
    }

    #[tokio::test]
    async fn test_test_polymorphic_adapter_not_found() {
        let registry = RegistryAdapter::new();
        let result = test_polymorphic_adapter(&registry, "nonexistent", vec![]).await;
        assert!(result.is_err());
    }
}
