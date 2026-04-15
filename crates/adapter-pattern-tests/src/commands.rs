// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Command registry and registry adapter implementation

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
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
pub trait CommandAdapter: Send + Sync {
    /// Execute a command with given arguments
    fn execute(
        &self,
        command: &str,
        args: Vec<String>,
    ) -> Pin<Box<dyn Future<Output = CommandResult<String>> + Send + '_>>;

    /// Get help information for a command
    fn get_help(
        &self,
        command: &str,
    ) -> Pin<Box<dyn Future<Output = CommandResult<String>> + Send + '_>>;

    /// List all available commands
    fn list_commands(
        &self,
    ) -> Pin<Box<dyn Future<Output = CommandResult<Vec<String>>> + Send + '_>>;
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

impl CommandAdapter for RegistryAdapter {
    fn execute(
        &self,
        command: &str,
        args: Vec<String>,
    ) -> Pin<Box<dyn Future<Output = CommandResult<String>> + Send + '_>> {
        let out = Self::execute(self, command, args);
        Box::pin(async move { out })
    }

    fn get_help(
        &self,
        command: &str,
    ) -> Pin<Box<dyn Future<Output = CommandResult<String>> + Send + '_>> {
        let out = Self::get_help(self, command);
        Box::pin(async move { out })
    }

    fn list_commands(
        &self,
    ) -> Pin<Box<dyn Future<Output = CommandResult<Vec<String>>> + Send + '_>> {
        let out = Self::list_commands(self);
        Box::pin(async move { out })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::TestCommand;
    use std::sync::Arc;

    #[test]
    fn test_command_registry_default() {
        let registry = CommandRegistry::default();
        let cmds = registry.list_commands().expect("should succeed");
        assert!(cmds.is_empty());
    }

    #[test]
    fn test_command_registry_register_and_execute() {
        let mut registry = CommandRegistry::new();
        let cmd = Arc::new(TestCommand::new("hello", "Says hello", "Hello!"));
        registry.register("hello", cmd).expect("should succeed");

        let result = registry.execute("hello", vec![]).expect("should succeed");
        assert_eq!(result, "Hello!");
    }

    #[test]
    fn test_command_registry_execute_with_args() {
        let mut registry = CommandRegistry::new();
        let cmd = Arc::new(TestCommand::new("echo", "Echoes", "Echo"));
        registry.register("echo", cmd).expect("should succeed");

        let result = registry
            .execute("echo", vec!["a".to_string(), "b".to_string()])
            .expect("should succeed");
        assert!(result.contains('a'));
        assert!(result.contains('b'));
    }

    #[test]
    fn test_command_registry_execute_not_found() {
        let registry = CommandRegistry::new();
        let result = registry.execute("missing", vec![]);
        assert!(result.is_err());
        assert!(matches!(result, Err(CommandError::NotFound(_))));
    }

    #[test]
    fn test_command_registry_get_help() {
        let mut registry = CommandRegistry::new();
        let cmd = Arc::new(TestCommand::new("hello", "Says hello", "Hello!"));
        registry.register("hello", cmd).expect("should succeed");

        let help = registry.get_help("hello").expect("should succeed");
        assert_eq!(help, "hello: Says hello");
    }

    #[test]
    fn test_command_registry_get_help_not_found() {
        let registry = CommandRegistry::new();
        let result = registry.get_help("missing");
        assert!(result.is_err());
        assert!(matches!(result, Err(CommandError::NotFound(_))));
    }

    #[test]
    fn test_command_registry_list_commands() {
        let mut registry = CommandRegistry::new();
        registry
            .register("a", Arc::new(TestCommand::new("a", "A", "a")))
            .expect("should succeed");
        registry
            .register("b", Arc::new(TestCommand::new("b", "B", "b")))
            .expect("should succeed");

        let cmds = registry.list_commands().expect("should succeed");
        assert_eq!(cmds.len(), 2);
        assert!(cmds.contains(&"a".to_string()));
        assert!(cmds.contains(&"b".to_string()));
    }

    #[tokio::test]
    async fn test_registry_adapter_default() {
        let adapter = RegistryAdapter::default();
        let cmds = adapter.list_commands().expect("should succeed");
        assert!(cmds.is_empty());
    }

    #[tokio::test]
    async fn test_registry_adapter_register_execute() {
        let mut adapter = RegistryAdapter::new();
        let cmd = Arc::new(TestCommand::new("hello", "Says hello", "Hello!"));
        adapter.register("hello", cmd).expect("should succeed");

        let result = <RegistryAdapter as CommandAdapter>::execute(&adapter, "hello", vec![]).await;
        assert!(result.is_ok());
        assert_eq!(result.expect("should succeed"), "Hello!");
    }

    #[tokio::test]
    async fn test_registry_adapter_execute_not_found() {
        let adapter = RegistryAdapter::new();
        let result =
            <RegistryAdapter as CommandAdapter>::execute(&adapter, "missing", vec![]).await;
        assert!(result.is_err());
        assert!(matches!(result, Err(CommandError::NotFound(_))));
    }

    #[tokio::test]
    async fn test_registry_adapter_get_help() {
        let mut adapter = RegistryAdapter::new();
        let cmd = Arc::new(TestCommand::new("hello", "Says hello", "Hello!"));
        adapter.register("hello", cmd).expect("should succeed");

        let help = <RegistryAdapter as CommandAdapter>::get_help(&adapter, "hello")
            .await
            .expect("should succeed");
        assert_eq!(help, "hello: Says hello");
    }

    #[tokio::test]
    async fn test_registry_adapter_get_help_not_found() {
        let adapter = RegistryAdapter::new();
        let result = <RegistryAdapter as CommandAdapter>::get_help(&adapter, "missing").await;
        assert!(result.is_err());
        assert!(matches!(result, Err(CommandError::NotFound(_))));
    }

    #[tokio::test]
    async fn test_registry_adapter_list_commands() {
        let mut adapter = RegistryAdapter::new();
        adapter
            .register("a", Arc::new(TestCommand::new("a", "A", "a")))
            .expect("should succeed");
        adapter
            .register("b", Arc::new(TestCommand::new("b", "B", "b")))
            .expect("should succeed");

        let cmds = <RegistryAdapter as CommandAdapter>::list_commands(&adapter)
            .await
            .expect("should succeed");
        assert_eq!(cmds.len(), 2);
        assert!(cmds.contains(&"a".to_string()));
        assert!(cmds.contains(&"b".to_string()));
    }
}
