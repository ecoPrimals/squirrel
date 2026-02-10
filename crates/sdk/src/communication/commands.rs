// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Command handling functionality for plugins
//!
//! This module provides command registration and execution capabilities for WASM plugins.

use crate::error::{PluginError, PluginResult};
use crate::utils::{generate_command_id, safe_lock, safe_lock_or_fallback};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::Mutex as TokioMutex;

/// Command definition structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandDefinition {
    /// Command name
    pub name: String,
    /// Command description
    pub description: String,
    /// JSON schema for command parameters
    pub parameters: serde_json::Value,
    /// Command category
    pub category: Option<String>,
    /// Command examples
    pub examples: Vec<CommandExample>,
    /// Whether the command supports streaming
    pub streaming: bool,
    /// Estimated execution time in seconds
    pub estimated_duration: Option<u32>,
}

/// Command usage example
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandExample {
    /// Example description
    pub description: String,
    /// Example parameters
    pub parameters: serde_json::Value,
    /// Expected result (optional)
    pub expected_result: Option<serde_json::Value>,
}

/// Command execution context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandContext {
    /// Unique command execution ID
    pub execution_id: String,
    /// User ID (if available)
    pub user_id: Option<String>,
    /// Session ID
    pub session_id: Option<String>,
    /// Execution timestamp
    pub timestamp: String,
    /// Environment variables
    pub environment: HashMap<String, String>,
}

impl Default for CommandContext {
    fn default() -> Self {
        Self {
            execution_id: generate_command_id(),
            user_id: None,
            session_id: None,
            timestamp: crate::utils::current_timestamp_iso(),
            environment: HashMap::new(),
        }
    }
}

impl CommandContext {
    /// Create a new command context
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a command context with user ID
    pub fn with_user(user_id: String) -> Self {
        Self {
            execution_id: generate_command_id(),
            user_id: Some(user_id),
            session_id: None,
            timestamp: crate::utils::current_timestamp_iso(),
            environment: HashMap::new(),
        }
    }
}

/// Command execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResult {
    /// Whether the command succeeded
    pub success: bool,
    /// Result data (JSON)
    pub data: serde_json::Value,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl CommandResult {
    /// Create a successful result
    pub fn success(data: serde_json::Value) -> Self {
        Self {
            success: true,
            data,
            error: None,
            metadata: HashMap::new(),
        }
    }

    /// Create a failed result
    pub fn error(error: String) -> Self {
        Self {
            success: false,
            data: serde_json::Value::Null,
            error: Some(error),
            metadata: HashMap::new(),
        }
    }

    /// Add metadata to the result
    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// Command handler trait
pub trait CommandHandler: Send + Sync + std::fmt::Debug {
    /// Execute the command
    fn execute(
        &self,
        params: serde_json::Value,
        context: CommandContext,
    ) -> futures::future::BoxFuture<'_, PluginResult<CommandResult>>;
}

/// Command registry for managing plugin commands
#[derive(Debug)]
pub struct CommandRegistry {
    /// Registered commands
    commands: Arc<Mutex<HashMap<String, CommandDefinition>>>,
    /// Command handlers - using tokio::Mutex for async-safe access across await points
    handlers: Arc<TokioMutex<HashMap<String, Box<dyn CommandHandler>>>>,
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self {
            commands: Arc::new(Mutex::new(HashMap::new())),
            handlers: Arc::new(TokioMutex::new(HashMap::new())),
        }
    }
}

impl CommandRegistry {
    /// Create a new command registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the global command registry instance
    pub fn global() -> &'static CommandRegistry {
        use std::sync::OnceLock;
        static INSTANCE: OnceLock<CommandRegistry> = OnceLock::new();
        INSTANCE.get_or_init(CommandRegistry::default)
    }

    /// Register a command
    pub async fn register_command(&self, definition: CommandDefinition) -> PluginResult<()> {
        let mut commands = safe_lock(&self.commands, "commands")?;

        if commands.contains_key(&definition.name) {
            return Err(PluginError::PluginAlreadyExists {
                plugin_id: format!("Command '{}' already registered", definition.name),
            });
        }

        commands.insert(definition.name.clone(), definition);
        Ok(())
    }

    /// Register a command with handler
    pub async fn register_handler<H>(
        &self,
        definition: CommandDefinition,
        handler: H,
    ) -> PluginResult<()>
    where
        H: CommandHandler + Send + Sync + 'static,
    {
        // Register the command definition
        self.register_command(definition.clone()).await?;

        // Register the handler using tokio mutex
        let mut handlers = self.handlers.lock().await;
        handlers.insert(definition.name, Box::new(handler));

        Ok(())
    }

    /// Unregister a command
    pub async fn unregister_command(&self, name: &str) -> PluginResult<()> {
        {
            let mut commands = safe_lock(&self.commands, "commands")?;
            commands.remove(name);
        }

        let mut handlers = self.handlers.lock().await;
        handlers.remove(name);

        Ok(())
    }

    /// Get a command definition
    pub fn get_command(&self, name: &str) -> Option<CommandDefinition> {
        safe_lock_or_fallback(&self.commands, HashMap::new, "commands")
            .get(name)
            .cloned()
    }

    /// List all registered commands
    pub fn list_commands(&self) -> Vec<CommandDefinition> {
        safe_lock_or_fallback(&self.commands, HashMap::new, "commands")
            .values()
            .cloned()
            .collect()
    }

    /// Execute a command
    pub async fn execute_command(
        &self,
        name: &str,
        params: serde_json::Value,
        context: CommandContext,
    ) -> PluginResult<CommandResult> {
        // Get command definition
        let command = self
            .get_command(name)
            .ok_or_else(|| PluginError::UnknownCommand {
                command: name.to_string(),
            })?;

        // Validate parameters
        self.validate_parameters(&command, &params)?;

        // Execute command using tokio mutex - safe to hold across await
        let handlers = self.handlers.lock().await;
        let handler = handlers
            .get(name)
            .ok_or_else(|| PluginError::UnknownCommand {
                command: format!("No handler for command: {}", name),
            })?;

        // Execute command - tokio::Mutex is designed to be held across await points
        handler.execute(params, context).await
    }

    /// Validate command parameters
    fn validate_parameters(
        &self,
        command: &CommandDefinition,
        params: &serde_json::Value,
    ) -> PluginResult<()> {
        // Basic validation - in a real implementation, you'd use a JSON schema validator
        if let Some(schema_obj) = command.parameters.as_object() {
            if let Some(_properties) = schema_obj.get("properties").and_then(|p| p.as_object()) {
                if let Some(required) = schema_obj.get("required").and_then(|r| r.as_array()) {
                    for req_field in required {
                        if let Some(field_name) = req_field.as_str() {
                            if !params
                                .as_object()
                                .unwrap_or(&serde_json::Map::new())
                                .contains_key(field_name)
                            {
                                return Err(PluginError::MissingParameter {
                                    parameter: field_name.to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Search commands by category
    pub fn search_by_category(&self, category: &str) -> Vec<CommandDefinition> {
        safe_lock_or_fallback(&self.commands, HashMap::new, "commands")
            .values()
            .filter(|cmd| cmd.category.as_ref().is_some_and(|c| c == category))
            .cloned()
            .collect()
    }

    /// Search commands by name pattern
    pub fn search_by_name(&self, pattern: &str) -> Vec<CommandDefinition> {
        safe_lock_or_fallback(&self.commands, HashMap::new, "commands")
            .values()
            .filter(|cmd| cmd.name.contains(pattern))
            .cloned()
            .collect()
    }
}

/// Simple command handler implementation
#[derive(Debug)]
pub struct SimpleCommandHandler<F>
where
    F: Fn(
            serde_json::Value,
            CommandContext,
        ) -> futures::future::BoxFuture<'static, PluginResult<CommandResult>>
        + Send
        + Sync
        + std::fmt::Debug,
{
    handler_fn: F,
}

impl<F> SimpleCommandHandler<F>
where
    F: Fn(
            serde_json::Value,
            CommandContext,
        ) -> futures::future::BoxFuture<'static, PluginResult<CommandResult>>
        + Send
        + Sync
        + std::fmt::Debug,
{
    /// Create a new simple command handler
    pub fn new(handler_fn: F) -> Self {
        Self { handler_fn }
    }
}

impl<F> CommandHandler for SimpleCommandHandler<F>
where
    F: Fn(
            serde_json::Value,
            CommandContext,
        ) -> futures::future::BoxFuture<'static, PluginResult<CommandResult>>
        + Send
        + Sync
        + std::fmt::Debug,
{
    fn execute(
        &self,
        params: serde_json::Value,
        context: CommandContext,
    ) -> futures::future::BoxFuture<'_, PluginResult<CommandResult>> {
        Box::pin(async move { (self.handler_fn)(params, context).await })
    }
}

/// Helper macro for creating command handlers
#[macro_export]
macro_rules! command_handler {
    ($handler:expr) => {
        SimpleCommandHandler::new(|params, context| {
            Box::pin(async move { $handler(params, context).await })
        })
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::FutureExt;

    #[derive(Debug)]
    struct TestHandler;

    impl CommandHandler for TestHandler {
        fn execute(
            &self,
            params: serde_json::Value,
            _context: CommandContext,
        ) -> futures::future::BoxFuture<'_, PluginResult<CommandResult>> {
            async move { Ok(CommandResult::success(params)) }.boxed()
        }
    }

    #[tokio::test]
    async fn test_command_registry() {
        let registry = CommandRegistry::new();

        let command = CommandDefinition {
            name: "test_command".to_string(),
            description: "Test command".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "name": {"type": "string"}
                },
                "required": ["name"]
            }),
            category: Some("test".to_string()),
            examples: vec![],
            streaming: false,
            estimated_duration: None,
        };

        registry
            .register_handler(command.clone(), TestHandler)
            .await
            .unwrap();

        let commands = registry.list_commands();
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0].name, "test_command");
    }

    #[tokio::test]
    async fn test_command_execution() {
        let registry = CommandRegistry::new();

        let command = CommandDefinition {
            name: "test_command".to_string(),
            description: "Test command".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "name": {"type": "string"}
                },
                "required": ["name"]
            }),
            category: Some("test".to_string()),
            examples: vec![],
            streaming: false,
            estimated_duration: None,
        };

        registry
            .register_handler(command, TestHandler)
            .await
            .unwrap();

        let params = serde_json::json!({"name": "test"});
        let context = CommandContext::new();

        let result = registry
            .execute_command("test_command", params.clone(), context)
            .await
            .unwrap();
        assert!(result.success);
        assert_eq!(result.data, params);
    }
}
