//! Command handling functionality for plugins
//!
//! This module provides command registration and execution capabilities for WASM plugins.

use crate::error::{PluginError, PluginResult};
use crate::utils::{generate_command_id, safe_lock, safe_lock_or_fallback};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

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

impl CommandContext {
    /// Create a new command context
    pub fn new() -> Self {
        Self {
            execution_id: generate_command_id(),
            user_id: None,
            session_id: None,
            timestamp: crate::utils::current_timestamp_iso(),
            environment: HashMap::new(),
        }
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
    /// Command handlers
    handlers: Arc<Mutex<HashMap<String, Box<dyn CommandHandler>>>>,
}

impl CommandRegistry {
    /// Create a new command registry
    pub fn new() -> Self {
        Self {
            commands: Arc::new(Mutex::new(HashMap::new())),
            handlers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Get the global command registry instance
    pub fn global() -> &'static CommandRegistry {
        use std::sync::OnceLock;
        static INSTANCE: OnceLock<CommandRegistry> = OnceLock::new();
        INSTANCE.get_or_init(|| CommandRegistry::new())
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

        // Register the handler
        let mut handlers = safe_lock(&self.handlers, "handlers")?;
        handlers.insert(definition.name, Box::new(handler));

        Ok(())
    }

    /// Unregister a command
    pub async fn unregister_command(&self, name: &str) -> PluginResult<()> {
        let mut commands = safe_lock(&self.commands, "commands")?;
        let mut handlers = safe_lock(&self.handlers, "handlers")?;

        commands.remove(name);
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

        // Get handler
        let handlers = safe_lock(&self.handlers, "handlers")?;
        let handler = handlers
            .get(name)
            .ok_or_else(|| PluginError::UnknownCommand {
                command: format!("No handler for command: {}", name),
            })?;

        // Execute command
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
            .filter(|cmd| cmd.category.as_ref().map_or(false, |c| c == category))
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

    /// Validate JSON schema (comprehensive)
    fn validate_json_schema(
        &self,
        schema: &serde_json::Value,
        data: &serde_json::Value,
    ) -> PluginResult<()> {
        let schema_obj = schema
            .as_object()
            .ok_or_else(|| PluginError::InvalidParameter {
                name: "schema".to_string(),
                reason: "Schema must be an object".to_string(),
            })?;

        // Validate type
        if let Some(type_value) = schema_obj.get("type") {
            if let Some(expected_type) = type_value.as_str() {
                match expected_type {
                    "string" => {
                        if !data.is_string() {
                            return Err(PluginError::InvalidParameter {
                                name: "data".to_string(),
                                reason: format!("Expected string, got {}", get_value_type(data)),
                            });
                        }
                    }
                    "number" => {
                        if !data.is_number() {
                            return Err(PluginError::InvalidParameter {
                                name: "data".to_string(),
                                reason: format!("Expected number, got {}", get_value_type(data)),
                            });
                        }
                    }
                    "boolean" => {
                        if !data.is_boolean() {
                            return Err(PluginError::InvalidParameter {
                                name: "data".to_string(),
                                reason: format!("Expected boolean, got {}", get_value_type(data)),
                            });
                        }
                    }
                    "array" => {
                        if !data.is_array() {
                            return Err(PluginError::InvalidParameter {
                                name: "data".to_string(),
                                reason: format!("Expected array, got {}", get_value_type(data)),
                            });
                        }
                    }
                    "object" => {
                        if !data.is_object() {
                            return Err(PluginError::InvalidParameter {
                                name: "data".to_string(),
                                reason: format!("Expected object, got {}", get_value_type(data)),
                            });
                        }
                    }
                    _ => {} // Unknown type, skip validation
                }
            }
        }

        // Validate object properties
        if let Some(properties) = schema_obj.get("properties") {
            if let Some(data_obj) = data.as_object() {
                if let Some(properties_obj) = properties.as_object() {
                    for (property_name, property_schema) in properties_obj {
                        if let Some(property_value) = data_obj.get(property_name) {
                            // Recursively validate nested properties
                            self.validate_json_schema(property_schema, property_value)?;
                        }
                    }
                }
            }
        }

        // Validate required fields
        if let Some(required) = schema_obj.get("required") {
            if let Some(required_array) = required.as_array() {
                if let Some(data_obj) = data.as_object() {
                    for req_field in required_array {
                        if let Some(field_name) = req_field.as_str() {
                            if !data_obj.contains_key::<str>(field_name) {
                                return Err(PluginError::MissingParameter {
                                    parameter: field_name.to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }

        // Validate string constraints
        if data.is_string() {
            if let Some(data_str) = data.as_str() {
                if let Some(min_length) = schema_obj.get("minLength").and_then(|v| v.as_u64()) {
                    if data_str.len() < min_length as usize {
                        return Err(PluginError::InvalidParameter {
                            name: "data".to_string(),
                            reason: format!(
                                "String length {} is less than minimum {}",
                                data_str.len(),
                                min_length
                            ),
                        });
                    }
                }
                if let Some(max_length) = schema_obj.get("maxLength").and_then(|v| v.as_u64()) {
                    if data_str.len() > max_length as usize {
                        return Err(PluginError::InvalidParameter {
                            name: "data".to_string(),
                            reason: format!(
                                "String length {} exceeds maximum {}",
                                data_str.len(),
                                max_length
                            ),
                        });
                    }
                }
                if let Some(pattern) = schema_obj.get("pattern").and_then(|v| v.as_str()) {
                    // Basic pattern matching - in a real implementation, use regex
                    if !data_str.contains(pattern) {
                        return Err(PluginError::InvalidParameter {
                            name: "data".to_string(),
                            reason: format!("String does not match pattern: {}", pattern),
                        });
                    }
                }
            }
        }

        // Validate numeric constraints
        if let Some(data_num) = data.as_f64() {
            if let Some(minimum) = schema_obj.get("minimum").and_then(|v| v.as_f64()) {
                if data_num < minimum {
                    return Err(PluginError::InvalidParameter {
                        name: "data".to_string(),
                        reason: format!("Value {} is less than minimum {}", data_num, minimum),
                    });
                }
            }
            if let Some(maximum) = schema_obj.get("maximum").and_then(|v| v.as_f64()) {
                if data_num > maximum {
                    return Err(PluginError::InvalidParameter {
                        name: "data".to_string(),
                        reason: format!("Value {} exceeds maximum {}", data_num, maximum),
                    });
                }
            }
        }

        // Validate array constraints
        if let Some(data_array) = data.as_array() {
            if let Some(min_items) = schema_obj.get("minItems").and_then(|v| v.as_u64()) {
                if data_array.len() < min_items as usize {
                    return Err(PluginError::InvalidParameter {
                        name: "data".to_string(),
                        reason: format!(
                            "Array length {} is less than minimum {}",
                            data_array.len(),
                            min_items
                        ),
                    });
                }
            }
            if let Some(max_items) = schema_obj.get("maxItems").and_then(|v| v.as_u64()) {
                if data_array.len() > max_items as usize {
                    return Err(PluginError::InvalidParameter {
                        name: "data".to_string(),
                        reason: format!(
                            "Array length {} exceeds maximum {}",
                            data_array.len(),
                            max_items
                        ),
                    });
                }
            }

            // Validate array items
            if let Some(items_schema) = schema_obj.get("items") {
                for item in data_array {
                    self.validate_json_schema(items_schema, item)?;
                }
            }
        }

        Ok(())
    }
}

/// Helper function to get the type name of a JSON value
fn get_value_type(value: &serde_json::Value) -> &'static str {
    match value {
        serde_json::Value::Null => "null",
        serde_json::Value::Bool(_) => "boolean",
        serde_json::Value::Number(_) => "number",
        serde_json::Value::String(_) => "string",
        serde_json::Value::Array(_) => "array",
        serde_json::Value::Object(_) => "object",
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
