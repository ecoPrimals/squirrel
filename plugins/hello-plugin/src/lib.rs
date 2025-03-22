//! Hello World plugin for Squirrel CLI
//!
//! This is a simple plugin that adds a "hello" command to the CLI.

use std::sync::Arc;
use async_trait::async_trait;
use clap::{Command as ClapCommand, Arg};
use log::{debug, info};

use squirrel_commands::{Command, CommandRegistry, CommandResult, CommandContext};
use squirrel_cli::plugins::{Plugin, PluginError, PluginFactory};

/// Hello command implementation
#[derive(Debug, Clone)]
pub struct HelloCommand;

impl HelloCommand {
    /// Create a new hello command
    pub fn new() -> Self {
        Self
    }

    /// Create the command parser
    pub fn parser(&self) -> ClapCommand {
        ClapCommand::new("hello")
            .about("Hello world command from a plugin")
            .arg(
                Arg::new("name")
                    .long("name")
                    .help("Name to greet")
                    .default_value("world")
            )
    }
}

#[async_trait]
impl Command for HelloCommand {
    /// Get the command name
    fn name(&self) -> &str {
        "hello"
    }
    
    /// Get the command description
    fn description(&self) -> &str {
        "Say hello from a plugin"
    }
    
    /// Execute the command
    async fn execute(&self, context: &CommandContext) -> CommandResult {
        debug!("Executing hello command");
        
        let matches = context.matches();
        let name = matches.get_one::<String>("name").unwrap_or(&"world".to_string());
        
        Ok(format!("Hello, {}! This is coming from a plugin.", name))
    }
}

/// Hello plugin implementation
pub struct HelloPlugin;

#[async_trait]
impl Plugin for HelloPlugin {
    /// Get the plugin name
    fn name(&self) -> &str {
        "hello"
    }
    
    /// Get the plugin version
    fn version(&self) -> &str {
        "1.0.0"
    }
    
    /// Get the plugin description
    fn description(&self) -> Option<&str> {
        Some("A simple hello world plugin")
    }
    
    /// Initialize the plugin
    async fn initialize(&self) -> Result<(), PluginError> {
        info!("Initializing hello plugin");
        Ok(())
    }
    
    /// Register commands provided by this plugin
    fn register_commands(&self, registry: &mut CommandRegistry) -> Result<(), PluginError> {
        info!("Registering hello command");
        let command = HelloCommand::new();
        registry.register("hello", Arc::new(command))
            .map_err(|e| PluginError::RegisterError(format!("Failed to register hello command: {}", e)))
    }
    
    /// Return the list of commands provided by this plugin
    fn commands(&self) -> Vec<Arc<dyn Command>> {
        vec![Arc::new(HelloCommand::new())]
    }
    
    /// Execute plugin functionality
    async fn execute(&self, args: &[String]) -> Result<String, PluginError> {
        debug!("Executing hello plugin with args: {:?}", args);
        
        // Just return a simple message in this example
        Ok("Hello plugin executed successfully".to_string())
    }
    
    /// Clean up plugin resources
    async fn cleanup(&self) -> Result<(), PluginError> {
        info!("Cleaning up hello plugin");
        Ok(())
    }
}

/// Hello plugin factory implementation
#[no_mangle]
pub fn create_plugin() -> Result<Arc<dyn Plugin>, PluginError> {
    Ok(Arc::new(HelloPlugin))
}

/// Plugin factory registration
#[no_mangle]
pub fn register_plugin_factory() -> Arc<dyn PluginFactory> {
    struct HelloPluginFactory;
    
    impl PluginFactory for HelloPluginFactory {
        fn create(&self) -> Result<Arc<dyn Plugin>, PluginError> {
            create_plugin()
        }
    }
    
    Arc::new(HelloPluginFactory)
} 