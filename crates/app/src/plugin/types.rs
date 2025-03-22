use async_trait::async_trait;
use serde_json::Value;
use crate::error::Result;
use super::{Plugin, PluginMetadata};
use std::sync::Arc;
use squirrel_commands::CommandRegistry;
use crate::plugin::{PluginState};
use std::sync::RwLock;
use futures::future::BoxFuture;
use std::any::Any;

/// Command plugin for extending command functionality
#[async_trait]
pub trait CommandPlugin: Plugin {
    /// Execute a command with the given arguments
    async fn execute_command(&self, command: &str, args: Value) -> Result<Value>;
    
    /// Get the list of commands provided by this plugin
    async fn get_commands(&self) -> Result<Vec<String>>;
    
    /// Get command help information
    fn get_command_help(&self, command: &str) -> Option<String>;
    
    /// List available commands
    fn list_commands(&self) -> Vec<String>;

    /// Get the command registry
    fn registry(&self) -> Arc<CommandRegistry>;
}

/// UI plugin for extending user interface components
#[async_trait]
pub trait UiPlugin: Plugin {
    /// Get UI component by name
    async fn get_component(&self, name: &str) -> Result<Value>;
    
    /// Update UI component state
    async fn update_component(&self, name: &str, state: Value) -> Result<()>;
    
    /// List available components
    fn list_components(&self) -> Vec<String>;
}

/// Tool plugin for extending tool functionality
#[async_trait]
pub trait ToolPlugin: Plugin {
    /// Execute a tool with arguments
    async fn execute_tool(&self, tool: &str, args: Value) -> Result<Value>;
    
    /// Get tool configuration
    fn get_tool_config(&self, tool: &str) -> Option<Value>;
    
    /// List available tools
    fn list_tools(&self) -> Vec<String>;
}

/// MCP plugin for extending Machine Context Protocol
#[async_trait]
pub trait McpPlugin: Plugin {
    /// Handle MCP message
    async fn handle_message(&self, message: Value) -> Result<Value>;
    
    /// Get protocol extensions
    fn get_protocol_extensions(&self) -> Vec<String>;
    
    /// Get message handlers
    fn get_message_handlers(&self) -> Vec<String>;
}

/// A plugin that provides command functionality
#[derive(Debug)]
pub struct CommandPluginImpl {
    /// Plugin metadata
    pub metadata: PluginMetadata,
    /// Command registry
    pub registry: Arc<CommandRegistry>,
    /// Command state
    pub state: RwLock<Option<PluginState>>,
}

impl Plugin for CommandPluginImpl {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    fn initialize<'a>(&'a self) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { Ok(()) })
    }

    fn shutdown<'a>(&'a self) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { Ok(()) })
    }

    fn get_state<'a>(&'a self) -> BoxFuture<'a, Result<Option<PluginState>>> {
        Box::pin(async move { 
            match self.state.read() {
                Ok(guard) => Ok(guard.clone()),
                Err(_) => Err(std::io::Error::new(std::io::ErrorKind::Other, "Failed to acquire read lock").into())
            }
        })
    }

    fn set_state<'a>(&'a self, state: PluginState) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            match self.state.write() {
                Ok(mut guard) => {
                    *guard = Some(state);
                    Ok(())
                },
                Err(_) => Err(std::io::Error::new(std::io::ErrorKind::Other, "Failed to acquire write lock").into())
            }
        })
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn Plugin> {
        Box::new(Self {
            metadata: self.metadata.clone(),
            registry: self.registry.clone(),
            state: RwLock::new(self.state.read().ok().and_then(|guard| guard.clone())),
        })
    }
}

#[async_trait]
impl CommandPlugin for CommandPluginImpl {
    async fn execute_command(&self, command: &str, args: Value) -> Result<Value> {
        // Example implementation
        Ok(Value::String(format!("Executed command {command} with args {args:?}")))
    }
    
    async fn get_commands(&self) -> Result<Vec<String>> {
        // For now, return an empty list since we don't have a way to get registered commands
        Ok(Vec::new())
    }
    
    fn get_command_help(&self, command: &str) -> Option<String> {
        Some(format!("Help for command: {command}"))
    }
    
    fn list_commands(&self) -> Vec<String> {
        Vec::new()
    }

    fn registry(&self) -> Arc<CommandRegistry> {
        self.registry.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    
    #[tokio::test]
    async fn test_example_command_plugin() {
        let plugin = CommandPluginImpl {
            metadata: PluginMetadata {
                id: Uuid::new_v4(),
                name: "example".to_string(),
                version: "0.1.0".to_string(),
                description: "Example command plugin".to_string(),
                author: "Test Author".to_string(),
                dependencies: vec![],
                capabilities: vec!["command".to_string()],
            },
            registry: Arc::new(CommandRegistry::new()),
            state: RwLock::new(None),
        };
        
        // Test plugin interface
        assert_eq!(plugin.metadata().name, "example");
        
        // Test command plugin interface
        let result = plugin.execute_command("test", Value::Null).await.unwrap();
        assert!(result.is_string());
        
        let commands = plugin.get_commands().await.unwrap();
        assert!(commands.is_empty());
        
        assert!(plugin.get_command_help("test").is_some());
    }
} 