use std::sync::Arc;
use tokio::sync::Mutex;
use log::debug;
use serde_json::Value;

use commands::{Command, CommandRegistry};
use crate::commands::adapter::error::{AdapterError, AdapterResult};
use crate::commands::adapter::CommandAdapterTrait;

/// Metadata for a command
#[derive(Debug, Clone)]
pub struct CommandMetadata {
    /// Command name
    pub name: String,
    /// Command description
    pub description: String,
    /// Command usage information
    pub usage: String,
}

/// Metadata for a plugin
#[derive(Debug, Clone)]
pub struct PluginMetadata {
    /// Plugin name
    pub name: String,
    /// Plugin version
    pub version: String,
    /// Plugin description
    pub description: String,
    /// Plugin author
    pub author: String,
    /// Plugin ID
    pub id: String,
    /// Plugin capabilities
    pub capabilities: Vec<String>,
}

/// Plugin interface for command execution
pub trait Plugin: Send + Sync {
    /// Get the plugin metadata
    fn metadata(&self) -> PluginMetadata;
    
    /// Get the available commands in this plugin
    fn get_available_commands(&self) -> Vec<CommandMetadata>;
    
    /// Execute a command with the given ID and input
    fn execute_command(&self, command_id: &str, input: Value) -> Result<Value, String>;
    
    /// Get help for a command
    fn get_command_help(&self, command_id: &str) -> Option<String>;
    
    /// Initialize the plugin
    fn initialize(&self) -> Result<(), String>;
    
    /// Shutdown the plugin
    fn shutdown(&self) -> Result<(), String>;
}

/// A registry of plugins
pub struct PluginRegistry {
    plugins: Vec<Arc<dyn Plugin>>,
}

impl PluginRegistry {
    /// Create a new plugin registry
    pub fn new() -> Self {
        Self { plugins: Vec::new() }
    }
    
    /// Register a plugin
    pub fn register_plugin(&mut self, plugin: Arc<dyn Plugin>) {
        self.plugins.push(plugin);
    }
    
    /// Get a plugin by name
    pub fn get_plugin(&self, name: &str) -> Option<&Arc<dyn Plugin>> {
        self.plugins.iter().find(|p| p.metadata().name == name)
    }
    
    /// Get all plugins
    pub fn get_plugins(&self) -> &[Arc<dyn Plugin>] {
        &self.plugins
    }
}

/// Adapter for plugin-based command execution
pub struct CommandsPluginAdapter {
    registry: Arc<Mutex<CommandRegistry>>,
    plugin_registry: Arc<Mutex<PluginRegistry>>,
    command_cache: Mutex<Vec<CommandMetadata>>,
}

impl CommandsPluginAdapter {
    /// Create a new plugin adapter
    pub fn new(registry: Arc<Mutex<CommandRegistry>>) -> Self {
        debug!("Creating new plugin adapter");
        Self {
            registry,
            plugin_registry: Arc::new(Mutex::new(PluginRegistry::new())),
            command_cache: Mutex::new(Vec::new()),
        }
    }
    
    /// Register a plugin
    pub async fn register_plugin(&self, plugin: Arc<dyn Plugin>) -> AdapterResult<()> {
        debug!("Registering plugin: {}", plugin.metadata().name);
        let mut plugin_registry = self.plugin_registry.lock().await;
        plugin_registry.register_plugin(plugin);
        self.rebuild_metadata_cache().await?;
        Ok(())
    }
    
    /// Rebuild the command metadata cache
    async fn rebuild_metadata_cache(&self) -> AdapterResult<()> {
        debug!("Rebuilding command metadata cache");
        let plugin_registry = self.plugin_registry.lock().await;
        let mut command_cache = self.command_cache.lock().await;
        command_cache.clear();
        
        for plugin in plugin_registry.get_plugins() {
            for cmd in plugin.get_available_commands() {
                command_cache.push(cmd);
            }
        }
        
        debug!("Command metadata cache rebuilt with {} commands", command_cache.len());
        Ok(())
    }

    /// Get a list of loaded plugin names
    pub async fn get_loaded_plugins(&self) -> AdapterResult<Vec<String>> {
        let plugin_registry = self.plugin_registry.lock().await;
        
        let plugins = plugin_registry.get_plugins();
        Ok(plugins.iter().map(|p| p.metadata().name.to_string()).collect())
    }
    
    /// Check if a plugin has a specific command
    pub async fn has_command(&self, plugin_name: &str, command_name: &str) -> AdapterResult<bool> {
        let plugin_registry = self.plugin_registry.lock().await;
        
        for plugin in plugin_registry.get_plugins() {
            if plugin.metadata().name == plugin_name {
                return Ok(plugin.get_available_commands().iter().any(|cmd| cmd.name == command_name));
            }
        }
        
        Ok(false)
    }
    
    /// Execute a command from a specific plugin
    pub async fn execute_plugin_command(&self, plugin_name: &str, command_name: &str, args: Vec<String>) -> AdapterResult<String> {
        let plugin_registry = self.plugin_registry.lock().await;
        
        for plugin in plugin_registry.get_plugins() {
            if plugin.metadata().name == plugin_name {
                if plugin.get_available_commands().iter().any(|cmd| cmd.name == command_name) {
                    let input = serde_json::json!({
                        "args": args
                    });
                    
                    match plugin.execute_command(command_name, input) {
                        Ok(output) => {
                            // Extract result from output
                            if let Some(result) = output.get("result") {
                                if let Some(result_str) = result.as_str() {
                                    return Ok(result_str.to_string());
                                }
                            }
                            
                            return Ok("Command executed successfully".to_string());
                        },
                        Err(e) => {
                            return Err(AdapterError::ExecutionFailed(format!(
                                "Plugin execution failed: {}", e
                            )));
                        }
                    }
                }
            }
        }
        
        Err(AdapterError::NotFound(format!("Command '{}' not found in plugin '{}'", command_name, plugin_name)))
    }
    
    /// Get help text for a command from a specific plugin
    pub async fn get_plugin_command_help(&self, plugin_name: &str, command_name: &str) -> AdapterResult<String> {
        let plugin_registry = self.plugin_registry.lock().await;
        
        for plugin in plugin_registry.get_plugins() {
            if plugin.metadata().name == plugin_name {
                if let Some(help) = plugin.get_command_help(command_name) {
                    return Ok(help);
                }
            }
        }
        
        Err(AdapterError::NotFound(format!("Help for command '{}' not found in plugin '{}'", command_name, plugin_name)))
    }
    
    /// List commands available in a specific plugin
    pub async fn list_plugin_commands(&self, plugin_name: &str) -> AdapterResult<Vec<String>> {
        let plugin_registry = self.plugin_registry.lock().await;
        
        for plugin in plugin_registry.get_plugins() {
            if plugin.metadata().name == plugin_name {
                let commands = plugin.get_available_commands()
                    .iter()
                    .map(|cmd| cmd.name.clone())
                    .collect();
                return Ok(commands);
            }
        }
        
        Err(AdapterError::NotFound(format!("Plugin '{}' not found", plugin_name)))
    }
}

#[async_trait::async_trait]
impl CommandAdapterTrait for CommandsPluginAdapter {
    async fn execute_command(&self, command: &str, args: Vec<String>) -> AdapterResult<String> {
        debug!("Plugin adapter executing command: {} with args: {:?}", command, args);
        
        // Check if the command contains a namespace
        let parts: Vec<&str> = command.split('.').collect();
        if parts.len() > 1 {
            let plugin_name = parts[0];
            let command_name = parts[1];
            
            // Execute command through plugin
            self.execute_plugin_command(plugin_name, command_name, args).await
        } else {
            // Try to find in any loaded plugin
            for plugin_name in self.get_loaded_plugins().await? {
                if self.has_command(&plugin_name, command).await? {
                    return self.execute_plugin_command(&plugin_name, command, args).await;
                }
            }
            
            Err(AdapterError::NotFound(format!("Command '{}' not found in any plugin", command)))
        }
    }
    
    async fn get_help(&self, command: &str) -> AdapterResult<String> {
        debug!("Plugin adapter getting help for: {}", command);
        
        // Check if the command contains a namespace
        let parts: Vec<&str> = command.split('.').collect();
        if parts.len() > 1 {
            let plugin_name = parts[0];
            let command_name = parts[1];
            
            // Get help through plugin
            self.get_plugin_command_help(plugin_name, command_name).await
        } else {
            // Try to find in any loaded plugin
            for plugin_name in self.get_loaded_plugins().await? {
                if self.has_command(&plugin_name, command).await? {
                    return self.get_plugin_command_help(&plugin_name, command).await;
                }
            }
            
            Err(AdapterError::NotFound(format!("Command '{}' not found in any plugin", command)))
        }
    }
    
    async fn list_commands(&self) -> AdapterResult<Vec<String>> {
        debug!("Plugin adapter listing commands");
        
        let mut commands = Vec::new();
        for plugin_name in self.get_loaded_plugins().await? {
            let plugin_commands = self.list_plugin_commands(&plugin_name).await?;
            for cmd in plugin_commands {
                commands.push(format!("{}.{}", plugin_name, cmd));
            }
        }
        
        Ok(commands)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Command as ClapCommand;
    use commands::{Command, CommandResult};
    
    #[derive(Debug, Clone)]
    struct TestCommand;
    
    impl Command for TestCommand {
        fn name(&self) -> &str {
            "test_command"
        }
        
        fn description(&self) -> &str {
            "Test command for unit tests"
        }
        
        fn parser(&self) -> ClapCommand {
            ClapCommand::new("test_command")
                .about("Test command for unit tests")
        }
        
        fn execute(&self, args: &[String]) -> CommandResult<String> {
            if args.is_empty() {
                Ok("Test command executed successfully".to_string())
            } else {
                Ok(format!("Test command executed with args: {:?}", args))
            }
        }
        
        fn clone_box(&self) -> Box<dyn Command> {
            Box::new(self.clone())
        }
    }
    
    #[tokio::test]
    async fn test_commands_plugin_adapter() {
        // Create registry and register commands
        let registry = Arc::new(Mutex::new(CommandRegistry::new()));
        
        // Register a test command
        {
            let mut registry_lock = registry.lock().await;
            registry_lock.register("test_command", Arc::new(TestCommand)).unwrap();
        }
        
        // Create plugin adapter
        let adapter = CommandsPluginAdapter::new(registry.clone());
        
        // Get loaded plugins
        let plugins = adapter.get_loaded_plugins().await.unwrap();
        assert!(plugins.is_empty()); // No plugins loaded yet
        
        // Check if command exists in a plugin
        let has_command = adapter.has_command("test_plugin", "test_command").await.unwrap();
        assert!(!has_command); // Should be false as no plugins are loaded
        
        // Execute command
        let result = adapter.execute_plugin_command("test_plugin", "test_command", vec!["arg1".to_string(), "arg2".to_string()]).await;
        assert!(result.is_err()); // Should fail as plugin doesn't exist
        
        // Get command help
        let help = adapter.get_plugin_command_help("test_plugin", "test_command").await;
        assert!(help.is_err()); // Should fail as plugin doesn't exist
        
        // List commands
        let commands = adapter.list_plugin_commands("test_plugin").await;
        assert!(commands.is_err()); // Should fail as plugin doesn't exist
    }
} 