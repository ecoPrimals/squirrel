// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Commands plugin module
//!
//! This module provides functionality for creating and managing command plugins.

use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

use crate::errors::PluginError;
use crate::plugin::{Plugin, PluginMetadata};

/// Command metadata
#[derive(Clone, Debug)]
pub struct CommandMetadata {
    /// Command name
    pub name: String,

    /// Command description
    pub description: String,

    /// Command usage
    pub usage: String,

    /// Required permissions
    pub permissions: Vec<String>,
}

impl CommandMetadata {
    /// Create a new command metadata
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            usage: String::new(),
            permissions: Vec::new(),
        }
    }

    /// Add usage information
    pub fn with_usage(mut self, usage: impl Into<String>) -> Self {
        self.usage = usage.into();
        self
    }

    /// Add required permission
    pub fn with_permission(mut self, permission: impl Into<String>) -> Self {
        self.permissions.push(permission.into());
        self
    }
}

/// Command handler function
pub type CommandHandler =
    Arc<dyn Fn(Value) -> futures::future::BoxFuture<'static, Result<Value>> + Send + Sync>;

/// Command implementation
#[derive(Clone)]
pub struct Command {
    /// Command metadata
    pub metadata: CommandMetadata,

    /// Command handler
    pub handler: CommandHandler,
}

impl std::fmt::Debug for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Command")
            .field("metadata", &self.metadata)
            .field("handler", &"CommandHandler")
            .finish()
    }
}

impl Command {
    /// Create a new command
    pub fn new(
        metadata: CommandMetadata,
        handler: impl Fn(Value) -> futures::future::BoxFuture<'static, Result<Value>>
            + Send
            + Sync
            + 'static,
    ) -> Self {
        Self {
            metadata,
            handler: Arc::new(handler),
        }
    }

    /// Execute the command
    pub async fn execute(&self, args: Value) -> Result<Value> {
        (self.handler)(args).await
    }
}

/// Commands plugin implementation
#[derive(Debug)]
pub struct CommandsPlugin {
    /// Plugin metadata
    metadata: PluginMetadata,

    /// Commands provided by this plugin
    commands: HashMap<String, Command>,
}

impl CommandsPlugin {
    /// Create a new commands plugin
    pub fn new(metadata: PluginMetadata) -> Self {
        Self {
            metadata,
            commands: HashMap::new(),
        }
    }

    /// Add a command to the plugin
    pub fn with_command(mut self, command: Command) -> Self {
        self.commands.insert(command.metadata.name.clone(), command);
        self
    }

    /// Execute a command
    pub async fn execute_command(&self, name: &str, args: Value) -> Result<Value> {
        match self.commands.get(name) {
            Some(command) => command.execute(args).await,
            None => Err(PluginError::CommandNotFound(name.to_string()).into()),
        }
    }
}

#[async_trait]
impl Plugin for CommandsPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    async fn initialize(&self) -> Result<()> {
        // No initialization needed
        Ok(())
    }

    async fn shutdown(&self) -> Result<()> {
        // No shutdown needed
        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Builder for creating commands plugins
#[derive(Debug)]
pub struct CommandsPluginBuilder {
    /// Plugin metadata
    metadata: PluginMetadata,

    /// Commands to add to the plugin
    commands: Vec<Command>,
}

impl CommandsPluginBuilder {
    /// Create a new commands plugin builder
    pub fn new(metadata: PluginMetadata) -> Self {
        Self {
            metadata,
            commands: Vec::new(),
        }
    }

    /// Add a command to the plugin
    pub fn with_command(mut self, command: Command) -> Self {
        self.commands.push(command);
        self
    }

    /// Add a command with a closure
    pub fn with_command_fn(
        mut self,
        name: impl Into<String>,
        description: impl Into<String>,
        handler: impl Fn(Value) -> futures::future::BoxFuture<'static, Result<Value>>
            + Send
            + Sync
            + 'static,
    ) -> Self {
        let metadata = CommandMetadata::new(name, description);
        let command = Command::new(metadata, handler);
        self.commands.push(command);
        self
    }

    /// Build the commands plugin
    pub fn build(self) -> CommandsPlugin {
        let mut plugin = CommandsPlugin::new(self.metadata);

        for command in self.commands {
            plugin = plugin.with_command(command);
        }

        plugin
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_commands_plugin() {
        // Create a plugin with the builder
        let metadata = PluginMetadata::new(
            "test-commands",
            "1.0.0",
            "Test commands plugin",
            "Test Author",
        );

        let plugin = CommandsPluginBuilder::new(metadata)
            .with_command_fn("hello", "Say hello", |args| {
                Box::pin(async move {
                    let name = match args.get("name") {
                        Some(name) => name.as_str().unwrap_or("world"),
                        None => "world",
                    };

                    Ok(serde_json::json!({
                        "message": format!("Hello, {}!", name)
                    }))
                })
            })
            .with_command_fn("echo", "Echo back input", |args| {
                Box::pin(async move { Ok(args) })
            })
            .build();

        // Execute commands
        let result = plugin
            .execute_command("hello", serde_json::json!({ "name": "test" }))
            .await
            .expect("should succeed");

        assert_eq!(
            result.get("message").expect("should succeed").as_str().expect("should succeed"),
            "Hello, test!"
        );

        let result = plugin
            .execute_command("echo", serde_json::json!({ "value": 42 }))
            .await
            .expect("should succeed");

        assert_eq!(result.get("value").expect("should succeed").as_i64().expect("should succeed"), 42);
    }
}
