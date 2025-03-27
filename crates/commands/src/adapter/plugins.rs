//! Plugin system adapter for commands crate
//!
//! This module provides adapters to integrate the commands crate with the
//! unified plugin system architecture.

use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, Mutex, RwLock};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
use thiserror::Error;
use tracing::{debug, error, info, trace};

use squirrel_interfaces::plugins::{Plugin, PluginMetadata, CommandsPlugin, CommandMetadata};

use crate::registry::{Command, CommandRegistry};

/// Errors that can occur during plugin adapter operations
#[derive(Debug, Error)]
pub enum PluginAdapterError {
    /// Command registry error
    #[error("Command registry error: {0}")]
    RegistryError(String),

    /// Command not found error
    #[error("Command not found: {0}")]
    CommandNotFound(String),

    /// Command execution error
    #[error("Command execution error: {0}")]
    ExecutionError(#[from] anyhow::Error),

    /// Invalid input error
    #[error("Invalid command input: {0}")]
    InvalidInput(String),

    /// Plugin system error
    #[error("Plugin system error: {0}")]
    PluginSystemError(String),
}

/// Result type for plugin adapter operations
pub type PluginAdapterResult<T> = std::result::Result<T, PluginAdapterError>;

/// Adapter that converts Command Registry to the Plugin System interface
#[derive(Debug)]
pub struct CommandsPluginAdapter {
    /// Plugin metadata
    metadata: PluginMetadata,
    
    /// Command registry instance
    registry: Arc<Mutex<CommandRegistry>>,
    
    /// Cache of command metadata
    command_metadata: RwLock<HashMap<String, CommandMetadata>>,
}

impl CommandsPluginAdapter {
    /// Create a new Commands Plugin Adapter
    pub fn new(registry: Arc<Mutex<CommandRegistry>>) -> Self {
        let metadata = PluginMetadata::new(
            "commands",
            env!("CARGO_PKG_VERSION"),
            "Command system plugin adapter",
            "Squirrel Team",
        )
        .with_capability("command_execution")
        .with_capability("command_management");

        

        Self {
            metadata,
            registry,
            command_metadata: RwLock::new(HashMap::new()),
        }
    }

    /// Converts a Command trait object to CommandMetadata
    fn convert_to_metadata(&self, cmd: &dyn Command) -> CommandMetadata {
        // Generate a stable ID for the command based on its name
        let id = format!("command.{}", cmd.name());

        CommandMetadata {
            id,
            name: cmd.name().to_string(),
            description: cmd.description().to_string(),
            input_schema: Self::generate_input_schema(cmd),
            output_schema: Self::generate_output_schema(),
            permissions: vec!["command.execute".to_string()],
        }
    }

    /// Generate input schema for a command
    fn generate_input_schema(cmd: &dyn Command) -> Value {
        // Extract parameters from the clap Command
        let _parser = cmd.parser();  // Add underscore to indicate intentionally unused variable
        let mut schema = serde_json::json!({
            "type": "object",
            "required": [],
            "properties": {}
        });

        // This is a simplified schema generator - in a real implementation,
        // we would parse the clap command structure to build a proper JSON schema.
        let properties = schema.as_object_mut().unwrap().get_mut("properties").unwrap();
        
        // Add args property
        properties.as_object_mut().unwrap().insert(
            "args".to_string(), 
            serde_json::json!({
                "type": "array",
                "items": {
                    "type": "string"
                }
            })
        );

        schema
    }

    /// Generate output schema for commands
    fn generate_output_schema() -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "success": {
                    "type": "boolean"
                },
                "output": {
                    "type": "string"
                },
                "error": {
                    "type": "string"
                }
            }
        })
    }

    /// Rebuild the command metadata cache
    fn rebuild_metadata_cache(&self) -> PluginAdapterResult<()> {
        debug!("Rebuilding command metadata cache");
        let registry_lock = self.registry.lock().map_err(|e| {
            PluginAdapterError::RegistryError(format!("Failed to acquire registry lock: {}", e))
        })?;

        let command_names = registry_lock.list_commands().map_err(|e| {
            PluginAdapterError::RegistryError(format!("Failed to list commands: {}", e))
        })?;

        // Create new metadata map
        let mut new_metadata = HashMap::new();

        // For each command, create metadata
        for name in command_names {
            let cmd = registry_lock.get_command(&name).map_err(|e| {
                PluginAdapterError::RegistryError(format!("Failed to get command '{}': {}", name, e))
            })?;

            let metadata = self.convert_to_metadata(cmd.as_ref());
            new_metadata.insert(metadata.id.clone(), metadata);
        }

        // Release registry lock before acquiring write lock on metadata
        drop(registry_lock);

        // Update the metadata cache
        let mut metadata_cache = self.command_metadata.write().map_err(|e| {
            PluginAdapterError::RegistryError(format!("Failed to acquire metadata write lock: {}", e))
        })?;

        *metadata_cache = new_metadata;
        debug!("Command metadata cache rebuilt with {} commands", metadata_cache.len());

        Ok(())
    }
}

#[async_trait]
impl Plugin for CommandsPluginAdapter {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    async fn initialize(&self) -> Result<()> {
        debug!("Initializing CommandsPluginAdapter");
        
        // Build the initial metadata cache
        self.rebuild_metadata_cache()
            .map_err(|e| anyhow::anyhow!("Failed to initialize command metadata: {}", e))?;
            
        trace!("CommandsPluginAdapter initialized successfully");
        Ok(())
    }

    async fn shutdown(&self) -> Result<()> {
        debug!("Shutting down CommandsPluginAdapter");
        Ok(())
    }
}

#[async_trait]
impl CommandsPlugin for CommandsPluginAdapter {
    fn get_available_commands(&self) -> Vec<CommandMetadata> {
        match self.command_metadata.read() {
            Ok(cache) => cache.values().cloned().collect(),
            Err(e) => {
                error!("Failed to read command metadata cache: {}", e);
                Vec::new()
            }
        }
    }
    
    fn get_command_metadata(&self, command_id: &str) -> Option<CommandMetadata> {
        match self.command_metadata.read() {
            Ok(cache) => cache.get(command_id).cloned(),
            Err(e) => {
                error!("Failed to read command metadata for '{}': {}", command_id, e);
                None
            }
        }
    }

    async fn execute_command(&self, command_id: &str, input: Value) -> Result<Value> {
        info!("Executing command '{}' via plugin adapter", command_id);
        
        // Extract command name from ID (remove the "command." prefix)
        let command_name = command_id.strip_prefix("command.").ok_or_else(|| {
            anyhow::anyhow!("Invalid command ID format: {}", command_id)
        })?;

        // Extract args from input
        let args = if let Some(args_value) = input.get("args") {
            if let Some(args_array) = args_value.as_array() {
                args_array.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect::<Vec<String>>()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };
        
        // Execute the command
        let registry_lock = self.registry.lock().map_err(|e| {
            anyhow::anyhow!("Failed to acquire registry lock: {}", e)
        })?;

        let result = registry_lock.execute(command_name, &args);
        drop(registry_lock);

        // Convert result to plugin response format
        match result {
            Ok(output) => Ok(serde_json::json!({
                "success": true,
                "output": output
            })),
            Err(e) => Ok(serde_json::json!({
                "success": false,
                "error": e.to_string()
            })),
        }
    }

    fn get_command_help(&self, command_id: &str) -> Option<String> {
        // Extract command name from ID
        if let Some(command_name) = command_id.strip_prefix("command.") {
            let registry_lock = match self.registry.lock() {
                Ok(lock) => lock,
                Err(e) => {
                    error!("Failed to acquire registry lock: {}", e);
                    return None;
                }
            };

            match registry_lock.get_help(command_name) {
                Ok(help) => Some(help),
                Err(e) => {
                    debug!("Failed to get help for command '{}': {}", command_name, e);
                    None
                }
            }
        } else {
            None
        }
    }
}

/// Create a commands plugin adapter from a registry
///
/// # Arguments
/// * `registry` - The command registry
///
/// # Returns
/// A commands plugin adapter that implements the CommandsPlugin trait
pub fn create_commands_plugin_adapter(
    registry: Arc<Mutex<CommandRegistry>>,
) -> Arc<dyn CommandsPlugin> {
    Arc::new(CommandsPluginAdapter::new(registry))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::factory::create_command_registry;
    use crate::registry::CommandResult;
    use std::error::Error;

    // Test command for unit tests
    struct TestCommand;

    impl Command for TestCommand {
        fn name(&self) -> &str {
            "test"
        }

        fn description(&self) -> &str {
            "Test command for unit tests"
        }

        fn execute(&self, args: &[String]) -> CommandResult<String> {
            Ok(format!("Test command executed with args: {:?}", args))
        }

        fn parser(&self) -> clap::Command {
            clap::Command::new("test")
                .about("Test command for unit tests")
                .arg(clap::Arg::new("value")
                    .help("A test value")
                    .required(false))
        }

        fn clone_box(&self) -> Box<dyn Command> {
            Box::new(TestCommand)
        }
    }

    #[tokio::test]
    async fn test_plugin_adapter_initialization() -> Result<(), Box<dyn Error>> {
        // Create a command registry
        let registry = create_command_registry()?;
        
        // Register a test command
        {
            let registry_guard = registry.lock().unwrap();
            registry_guard.register("test", Arc::new(TestCommand))?;
        }

        // Create the adapter
        let adapter = CommandsPluginAdapter::new(Arc::clone(&registry));
        
        // Initialize the adapter
        adapter.rebuild_metadata_cache()?;
        
        // Check available commands
        let commands = adapter.get_available_commands();
        assert!(commands.iter().any(|cmd| cmd.name == "test"), "Test command should be available");
        
        Ok(())
    }

    #[tokio::test]
    async fn test_command_execution() -> Result<(), Box<dyn Error>> {
        // Create command registry
        let registry = Arc::new(Mutex::new(CommandRegistry::new()));
        
        // Register test command
        {
            let registry_guard = registry.lock().unwrap();
            registry_guard.register("test", Arc::new(TestCommand)).unwrap();
        }
        
        // Create the adapter
        let adapter = CommandsPluginAdapter::new(registry);
        
        // Initialize the adapter
        adapter.rebuild_metadata_cache()?;
        
        // Execute the command
        let input = serde_json::json!({
            "args": ["arg1", "arg2"]
        });
        
        // Use the correct command ID format with "command." prefix
        let result = adapter.execute_command("command.test", input).await?;
        
        // Check result
        assert!(result.get("success").unwrap().as_bool().unwrap(), "Command execution should succeed");
        assert!(
            result.get("output").unwrap().as_str().unwrap().contains("arg1"),
            "Output should contain arg1"
        );
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_command_metadata_conversion() -> Result<(), Box<dyn Error>> {
        // Create command registry with test command
        let registry = Arc::new(Mutex::new(CommandRegistry::new()));
        {
            let registry_guard = registry.lock().unwrap();
            registry_guard.register("test", Arc::new(TestCommand)).unwrap();
        }
        
        // Create the adapter
        let adapter = CommandsPluginAdapter::new(registry);
        
        // Initialize metadata cache
        adapter.rebuild_metadata_cache()?;
        
        // Get command metadata
        let commands = adapter.get_available_commands();
        
        // Find test command
        let test_cmd = commands.iter()
            .find(|cmd| cmd.name == "test")
            .expect("Test command should be in metadata");
            
        // Verify metadata
        assert_eq!(test_cmd.id, "command.test");
        assert_eq!(test_cmd.description, "Test command for unit tests");
        assert!(test_cmd.input_schema.as_object().unwrap().contains_key("properties"));
        assert!(test_cmd.output_schema.as_object().unwrap().contains_key("properties"));
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_help_system() -> Result<(), Box<dyn Error>> {
        // Create command registry with test command
        let registry = Arc::new(Mutex::new(CommandRegistry::new()));
        {
            let registry_guard = registry.lock().unwrap();
            registry_guard.register("test", Arc::new(TestCommand)).unwrap();
        }
        
        // Create the adapter
        let adapter = CommandsPluginAdapter::new(registry);
        
        // Get help for command
        let help = adapter.get_command_help("command.test");
        
        // Verify help exists
        assert!(help.is_some(), "Help should be available for test command");
        assert!(help.unwrap().contains("Test command"), "Help should contain command description");
        
        Ok(())
    }
} 