// Plugin Examples Module
//
// This module provides example plugin implementations for testing and demonstration.

mod command_example;
mod tool_example;
mod dynamic_example;

use std::sync::Arc;
use async_trait::async_trait;

use uuid::Uuid;

use crate::plugins::interfaces::{Plugin, CommandsPlugin, ToolPlugin};
use crate::plugins::errors::Result;
use crate::plugins::discovery::PluginLoader;

pub use command_example::create_command_example_plugin;
pub use tool_example::create_tool_example_plugin;
pub use dynamic_example::create_dynamic_example_plugin;

/// Example plugin loader for testing and demonstration
#[derive(Debug)]
pub struct ExamplePluginLoader;

#[async_trait]
impl PluginLoader for ExamplePluginLoader {
    async fn load_plugins(&self) -> Result<Vec<Box<dyn Plugin>>> {
        let mut plugins: Vec<Box<dyn Plugin>> = Vec::new();
        
        // Create example command plugin
        let command_plugin = create_command_example_plugin();
        plugins.push(command_plugin);
        
        // Create example tool plugin
        let tool_plugin = create_tool_example_plugin();
        plugins.push(tool_plugin);
        
        Ok(plugins)
    }
}

/// Create an example plugin loader
pub fn create_example_plugin_loader() -> Arc<dyn PluginLoader> {
    Arc::new(ExamplePluginLoader)
}

/// Get a command plugin for testing
pub fn get_test_command_plugin() -> Box<dyn CommandsPlugin> {
    let plugin = create_command_example_plugin();
    plugin
}

/// Get a tool plugin for testing
pub fn get_test_tool_plugin() -> Box<dyn ToolPlugin> {
    let plugin = create_tool_example_plugin();
    plugin
}

/// Get a dynamic plugin for testing
pub fn get_test_dynamic_plugin() -> Box<dyn Plugin> {
    let plugin = create_dynamic_example_plugin();
    plugin
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};
    
    #[tokio::test]
    async fn test_command_example_plugin() {
        let plugin = get_test_command_plugin();
        
        // Test initialization
        plugin.initialize().await.expect("Failed to initialize");
        
        // Test commands
        let commands = plugin.get_commands();
        assert!(!commands.is_empty());
        
        // Test hello command
        let result = plugin.execute_command("hello", json!({"name": "World"})).await.expect("Failed to execute");
        let message = result["message"].as_str().expect("Missing message");
        assert_eq!(message, "Hello, World!");
        
        // Test help
        let help = plugin.get_command_help("hello").expect("Missing help");
        assert_eq!(help.name, "hello");
        
        // Test schema
        let schema = plugin.get_command_schema("hello").expect("Missing schema");
        assert!(schema.is_object());
        
        // Test shutdown
        plugin.shutdown().await.expect("Failed to shutdown");
    }
    
    #[tokio::test]
    async fn test_tool_example_plugin() {
        let plugin = get_test_tool_plugin();
        
        // Test initialization
        plugin.initialize().await.expect("Failed to initialize");
        
        // Test tools
        let tools = plugin.get_tools();
        assert!(!tools.is_empty());
        
        // Test analyze tool
        let result = plugin.execute_tool("analyze", json!({"text": "Sample text"})).await.expect("Failed to execute");
        let word_count = result["word_count"].as_u64().expect("Missing word count");
        assert_eq!(word_count, 2);
        
        // Test availability
        let availability = plugin.check_tool_availability("analyze").await.expect("Failed to check");
        assert!(availability.available);
        
        // Test metadata
        let metadata = plugin.get_tool_metadata("analyze").expect("Missing metadata");
        assert_eq!(metadata.name, "analyze");
        
        // Test shutdown
        plugin.shutdown().await.expect("Failed to shutdown");
    }
    
    #[tokio::test]
    async fn test_dynamic_example_plugin() {
        let plugin = get_test_dynamic_plugin();
        
        // Test initialization
        plugin.initialize().await.expect("Failed to initialize");
        plugin.start().await.expect("Failed to start");
        
        // Test shutdown sequence
        plugin.stop().await.expect("Failed to stop");
        plugin.shutdown().await.expect("Failed to shutdown");
    }
} 