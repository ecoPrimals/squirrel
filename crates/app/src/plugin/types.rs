use async_trait::async_trait;
use serde_json::Value;
use crate::error::Result;
use super::{Plugin, PluginMetadata};
use std::sync::Arc;
use crate::commands_crate::CommandRegistry;
use crate::plugin::{PluginState};
use tokio::sync::RwLock;
use futures::future::BoxFuture;
use std::any::Any;
use std::collections::HashMap;

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
    /// Available commands
    pub commands: Vec<String>,
    /// Command help texts
    pub command_help: HashMap<String, String>,
}

impl Clone for CommandPluginImpl {
    fn clone(&self) -> Self {
        Self {
            metadata: self.metadata.clone(),
            registry: self.registry.clone(),
            state: RwLock::new(None), // Create a new RwLock for the clone
            commands: self.commands.clone(),
            command_help: self.command_help.clone(),
        }
    }
}

impl Plugin for CommandPluginImpl {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    fn initialize(&self) -> BoxFuture<'_, Result<()>> {
        Box::pin(async move { Ok(()) })
    }

    fn shutdown(&self) -> BoxFuture<'_, Result<()>> {
        Box::pin(async move { Ok(()) })
    }

    fn get_state(&self) -> BoxFuture<'_, Result<Option<super::PluginState>>> {
        Box::pin(async move { 
            let guard = self.state.read().await;
            Ok(guard.clone())
        })
    }

    fn set_state(&self, state: super::PluginState) -> BoxFuture<'_, Result<()>> {
        Box::pin(async move {
            let mut guard = self.state.write().await;
            *guard = Some(state);
            Ok(())
        })
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn Plugin> {
        // Create a new CommandPluginImpl with a fresh state RwLock
        // This avoids blocking on async operations in a sync context
        Box::new(Self {
            metadata: self.metadata.clone(),
            registry: self.registry.clone(),
            state: RwLock::new(None),
            commands: self.commands.clone(),
            command_help: self.command_help.clone(),
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
        Ok(self.commands.clone())
    }
    
    fn get_command_help(&self, command: &str) -> Option<String> {
        self.command_help.get(command).cloned()
    }
    
    fn list_commands(&self) -> Vec<String> {
        self.commands.clone()
    }

    fn registry(&self) -> Arc<CommandRegistry> {
        self.registry.clone()
    }
}

/// A base implementation for tool plugins
#[derive(Debug)]
pub struct ToolPluginImpl {
    /// Plugin metadata
    pub metadata: PluginMetadata,
    /// Tool configurations
    pub tool_configs: HashMap<String, Value>,
    /// Available tools
    pub tools: Vec<String>,
    /// Plugin state
    pub state: RwLock<Option<PluginState>>,
}

impl Clone for ToolPluginImpl {
    fn clone(&self) -> Self {
        Self {
            metadata: self.metadata.clone(),
            tool_configs: self.tool_configs.clone(),
            tools: self.tools.clone(),
            state: RwLock::new(None), // Create a new RwLock for the clone
        }
    }
}

impl ToolPluginImpl {
    /// Create a new tool plugin
    #[must_use]
    pub fn new(metadata: PluginMetadata) -> Self {
        Self {
            metadata,
            tool_configs: HashMap::new(),
            tools: Vec::new(),
            state: RwLock::new(None),
        }
    }

    /// Register a tool with its configuration
    pub fn register_tool(&mut self, name: &str, config: Value) {
        self.tool_configs.insert(name.to_string(), config);
        if !self.tools.contains(&name.to_string()) {
            self.tools.push(name.to_string());
        }
    }
}

impl Plugin for ToolPluginImpl {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    fn initialize(&self) -> BoxFuture<'_, Result<()>> {
        Box::pin(async move { Ok(()) })
    }

    fn shutdown(&self) -> BoxFuture<'_, Result<()>> {
        Box::pin(async move { Ok(()) })
    }

    fn get_state(&self) -> BoxFuture<'_, Result<Option<PluginState>>> {
        Box::pin(async move { 
            let guard = self.state.read().await;
            Ok(guard.clone())
        })
    }

    fn set_state(&self, state: PluginState) -> BoxFuture<'_, Result<()>> {
        Box::pin(async move {
            let mut guard = self.state.write().await;
            *guard = Some(state);
            Ok(())
        })
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn Plugin> {
        Box::new(Self {
            metadata: self.metadata.clone(),
            tool_configs: self.tool_configs.clone(),
            tools: self.tools.clone(),
            state: RwLock::new(None),
        })
    }
}

#[async_trait]
impl ToolPlugin for ToolPluginImpl {
    async fn execute_tool(&self, tool: &str, args: Value) -> Result<Value> {
        // Default implementation simply echoes the tool name and args
        // Actual implementations should override this with real functionality
        Ok(Value::String(format!("Executed tool {tool} with args {args:?}")))
    }
    
    fn get_tool_config(&self, tool: &str) -> Option<Value> {
        self.tool_configs.get(tool).cloned()
    }
    
    fn list_tools(&self) -> Vec<String> {
        self.tools.clone()
    }
}

/// A base implementation for MCP plugins
pub struct McpPluginImpl {
    /// Plugin metadata
    pub metadata: PluginMetadata,
    /// Protocol extensions
    pub extensions: Vec<String>,
    /// Message handlers
    pub handlers: Vec<String>,
    /// Plugin state
    pub state: RwLock<Option<PluginState>>,
    /// Message handling callbacks
    message_handlers: RwLock<HashMap<String, Box<dyn Fn(Value) -> BoxFuture<'static, Result<Value>> + Send + Sync>>>,
}

impl Clone for McpPluginImpl {
    fn clone(&self) -> Self {
        Self {
            metadata: self.metadata.clone(),
            extensions: self.extensions.clone(),
            handlers: self.handlers.clone(),
            state: RwLock::new(None), // Create a new RwLock for the clone
            message_handlers: RwLock::new(HashMap::new()), // Create a new empty handlers map
        }
    }
}

impl std::fmt::Debug for McpPluginImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("McpPluginImpl")
            .field("metadata", &self.metadata)
            .field("extensions", &self.extensions)
            .field("handlers", &self.handlers)
            .field("state", &self.state)
            .field("message_handlers", &format!("<{} handlers>", self.handlers.len()))
            .finish()
    }
}

impl McpPluginImpl {
    /// Create a new MCP plugin
    #[must_use]
    pub fn new(metadata: PluginMetadata) -> Self {
        Self {
            metadata,
            extensions: Vec::new(),
            handlers: Vec::new(),
            state: RwLock::new(None),
            message_handlers: RwLock::new(HashMap::new()),
        }
    }

    /// Register a message handler
    pub async fn register_handler<F, Fut>(&mut self, message_type: &str, handler: F)
    where
        F: Fn(Value) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<Value>> + Send + 'static,
    {
        let mut handlers = self.message_handlers.write().await;
        
        // Convert the handler to a BoxFuture-returning closure
        let boxed_handler = Box::new(move |value: Value| -> BoxFuture<'static, Result<Value>> {
            Box::pin(handler(value))
        });
        
        handlers.insert(message_type.to_string(), boxed_handler);
        
        // Add to the handlers list if not already present
        if !self.handlers.contains(&message_type.to_string()) {
            self.handlers.push(message_type.to_string());
        }
    }

    /// Register a protocol extension
    pub fn register_extension(&mut self, extension: &str) {
        if !self.extensions.contains(&extension.to_string()) {
            self.extensions.push(extension.to_string());
        }
    }
}

impl Plugin for McpPluginImpl {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    fn initialize(&self) -> BoxFuture<'_, Result<()>> {
        Box::pin(async move { Ok(()) })
    }

    fn shutdown(&self) -> BoxFuture<'_, Result<()>> {
        Box::pin(async move { Ok(()) })
    }

    fn get_state(&self) -> BoxFuture<'_, Result<Option<PluginState>>> {
        Box::pin(async move { 
            let guard = self.state.read().await;
            Ok(guard.clone())
        })
    }

    fn set_state(&self, state: PluginState) -> BoxFuture<'_, Result<()>> {
        Box::pin(async move {
            let mut guard = self.state.write().await;
            *guard = Some(state);
            Ok(())
        })
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn Plugin> {
        // Note: This doesn't clone the message handlers, which need to be re-registered
        Box::new(Self {
            metadata: self.metadata.clone(),
            extensions: self.extensions.clone(),
            handlers: self.handlers.clone(),
            state: RwLock::new(None),
            message_handlers: RwLock::new(HashMap::new()),
        })
    }
}

#[async_trait]
impl McpPlugin for McpPluginImpl {
    async fn handle_message(&self, message: Value) -> Result<Value> {
        // Extract the message type
        let message_type = match message.get("type").and_then(Value::as_str) {
            Some(t) => t,
            None => return Ok(Value::String("Error: Missing message type".to_string())),
        };
        
        // Find the handler for this message type
        let handlers = self.message_handlers.read().await;
        
        if let Some(handler) = handlers.get(message_type) {
            // Call the handler
            handler(message).await
        } else {
            // No handler found
            Ok(Value::String(format!("No handler for message type: {message_type}")))
        }
    }
    
    fn get_protocol_extensions(&self) -> Vec<String> {
        self.extensions.clone()
    }
    
    fn get_message_handlers(&self) -> Vec<String> {
        self.handlers.clone()
    }
}

/// A builder to simplify creating command plugins
#[derive(Debug)]
pub struct CommandPluginBuilder {
    metadata: PluginMetadata,
    registry: Option<Arc<CommandRegistry>>,
    commands: HashMap<String, String>, // command -> help text
}

impl CommandPluginBuilder {
    /// Create a new command plugin builder
    #[must_use]
    pub fn new(metadata: PluginMetadata) -> Self {
        Self {
            metadata,
            registry: None,
            commands: HashMap::new(),
        }
    }
    
    /// Set the command registry
    #[must_use]
    pub fn with_registry(mut self, registry: Arc<CommandRegistry>) -> Self {
        self.registry = Some(registry);
        self
    }
    
    /// Add a command with help text
    #[must_use]
    pub fn with_command(mut self, name: &str, help: &str) -> Self {
        self.commands.insert(name.to_string(), help.to_string());
        self
    }
    
    /// Build the command plugin
    #[must_use]
    pub fn build(self) -> Box<dyn CommandPlugin> {
        let registry = self.registry.unwrap_or_else(|| Arc::new(CommandRegistry::new()));
        
        let commands: Vec<String> = self.commands.keys().cloned().collect();
        
        let impl_obj = CommandPluginImpl {
            metadata: self.metadata,
            registry,
            state: RwLock::new(None),
            commands,
            command_help: self.commands,
        };
        
        Box::new(impl_obj)
    }
}

/// A builder to simplify creating tool plugins
#[derive(Debug)]
pub struct ToolPluginBuilder {
    metadata: PluginMetadata,
    tools: HashMap<String, Value>, // tool name -> config
}

impl ToolPluginBuilder {
    /// Create a new tool plugin builder
    #[must_use]
    pub fn new(metadata: PluginMetadata) -> Self {
        Self {
            metadata,
            tools: HashMap::new(),
        }
    }
    
    /// Add a tool with configuration
    #[must_use]
    pub fn with_tool(mut self, name: &str, config: Value) -> Self {
        self.tools.insert(name.to_string(), config);
        self
    }
    
    /// Build the tool plugin
    #[must_use]
    pub fn build(self) -> Box<dyn ToolPlugin> {
        let mut plugin = ToolPluginImpl::new(self.metadata);
        
        for (name, config) in self.tools {
            plugin.register_tool(&name, config);
        }
        
        Box::new(plugin)
    }
}

/// A builder to simplify creating MCP plugins
#[derive(Debug)]
pub struct McpPluginBuilder {
    metadata: PluginMetadata,
    extensions: Vec<String>,
}

impl McpPluginBuilder {
    /// Create a new MCP plugin builder
    #[must_use]
    pub fn new(metadata: PluginMetadata) -> Self {
        Self {
            metadata,
            extensions: Vec::new(),
        }
    }
    
    /// Add a protocol extension
    #[must_use]
    pub fn with_extension(mut self, extension: &str) -> Self {
        self.extensions.push(extension.to_string());
        self
    }
    
    /// Build the MCP plugin
    #[must_use]
    pub fn build(self) -> Arc<McpPluginImpl> {
        let mut plugin = McpPluginImpl::new(self.metadata);
        
        for extension in self.extensions {
            plugin.register_extension(&extension);
        }
        
        Arc::new(plugin)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    use serde_json::json;
    
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
            commands: Vec::new(),
            command_help: {
                let mut map = HashMap::new();
                map.insert("test".to_string(), "Test command help".to_string());
                map
            },
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
    
    #[tokio::test]
    async fn test_tool_plugin() {
        let plugin = ToolPluginBuilder::new(PluginMetadata {
            id: Uuid::new_v4(),
            name: "test-tool".to_string(),
            version: "0.1.0".to_string(),
            description: "Test tool plugin".to_string(),
            author: "Test Author".to_string(),
            dependencies: vec![],
            capabilities: vec!["tool".to_string()],
        })
        .with_tool("analyze", json!({"language": "rust"}))
        .with_tool("format", json!({"style": "default"}))
        .build();
        
        // Test plugin interface
        assert_eq!(plugin.metadata().name, "test-tool");
        
        // Test tool plugin interface
        let tools = plugin.list_tools();
        assert_eq!(tools.len(), 2);
        assert!(tools.contains(&"analyze".to_string()));
        assert!(tools.contains(&"format".to_string()));
        
        // Test tool config
        let config = plugin.get_tool_config("analyze").unwrap();
        assert_eq!(config.get("language").unwrap(), "rust");
    }
    
    #[tokio::test]
    async fn test_mcp_plugin() {
        let plugin = McpPluginBuilder::new(PluginMetadata {
            id: Uuid::new_v4(),
            name: "test-mcp".to_string(),
            version: "0.1.0".to_string(),
            description: "Test MCP plugin".to_string(),
            author: "Test Author".to_string(),
            dependencies: vec![],
            capabilities: vec!["mcp".to_string()],
        })
        .with_extension("context")
        .with_extension("commands")
        .build();
        
        // Test plugin interface
        assert_eq!(plugin.metadata().name, "test-mcp");
        
        // Test MCP plugin interface
        let extensions = plugin.get_protocol_extensions();
        assert_eq!(extensions.len(), 2);
        assert!(extensions.contains(&"context".to_string()));
        assert!(extensions.contains(&"commands".to_string()));
        
        // Note: We can't register a handler on an Arc<McpPluginImpl>
        // Handler registration should happen before creating the Arc
        
        // Test basic plugin functionality
        assert!(plugin.metadata().capabilities.contains(&"mcp".to_string()));
    }
} 
