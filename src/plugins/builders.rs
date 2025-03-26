// Plugin Builders Module
//
// This module provides builder patterns for creating plugins.

use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::plugins::errors::PluginError;
use crate::plugins::interfaces::{CommandInfo, CommandHelp, CommandArgument, CommandOption, CommandsPlugin, ToolInfo, ToolMetadata, ToolAvailability, ToolPlugin};
use crate::plugins::Result;

use squirrel_mcp::plugins::interfaces::{Plugin, PluginMetadata, PluginStatus};

/// Builder for creating command plugins
pub struct CommandPluginBuilder {
    /// Plugin metadata
    metadata: PluginMetadata,
    
    /// Command info
    commands: Vec<CommandInfo>,
    
    /// Command help
    command_help: HashMap<String, CommandHelp>,
    
    /// Command schema
    command_schema: HashMap<String, Value>,
    
    /// Command handlers
    command_handlers: HashMap<String, Arc<dyn Fn(Value) -> Result<Value> + Send + Sync>>,
}

impl CommandPluginBuilder {
    /// Create a new command plugin builder
    pub fn new(metadata: PluginMetadata) -> Self {
        Self {
            metadata,
            commands: Vec::new(),
            command_help: HashMap::new(),
            command_schema: HashMap::new(),
            command_handlers: HashMap::new(),
        }
    }
    
    /// Add a command
    pub fn with_command(
        mut self,
        name: &str,
        description: &str,
    ) -> Self {
        let command = CommandInfo {
            name: name.to_string(),
            description: description.to_string(),
            category: None,
            tags: Vec::new(),
            requires_auth: false,
        };
        
        self.commands.push(command);
        
        self
    }
    
    /// Add a command with category and tags
    pub fn with_command_full(
        mut self,
        name: &str,
        description: &str,
        category: Option<&str>,
        tags: Vec<&str>,
        requires_auth: bool,
    ) -> Self {
        let command = CommandInfo {
            name: name.to_string(),
            description: description.to_string(),
            category: category.map(|s| s.to_string()),
            tags: tags.into_iter().map(|s| s.to_string()).collect(),
            requires_auth,
        };
        
        self.commands.push(command);
        
        self
    }
    
    /// Add a command handler
    pub fn with_command_handler<F>(
        mut self,
        name: &str,
        handler: F,
    ) -> Self
    where
        F: Fn(Value) -> Result<Value> + Send + Sync + 'static,
    {
        self.command_handlers.insert(name.to_string(), Arc::new(handler));
        
        self
    }
    
    /// Add a command handler with help and schema
    pub fn with_command_full_handler<F>(
        mut self,
        name: &str,
        description: &str,
        usage: &str,
        examples: Vec<&str>,
        arguments: Vec<CommandArgument>,
        options: Vec<CommandOption>,
        schema: Value,
        handler: F,
    ) -> Self
    where
        F: Fn(Value) -> Result<Value> + Send + Sync + 'static,
    {
        // Add command
        let command = CommandInfo {
            name: name.to_string(),
            description: description.to_string(),
            category: None,
            tags: Vec::new(),
            requires_auth: false,
        };
        
        self.commands.push(command);
        
        // Add command help
        let help = CommandHelp {
            name: name.to_string(),
            description: description.to_string(),
            usage: usage.to_string(),
            examples: examples.into_iter().map(|s| s.to_string()).collect(),
            arguments,
            options,
        };
        
        self.command_help.insert(name.to_string(), help);
        
        // Add command schema
        self.command_schema.insert(name.to_string(), schema);
        
        // Add command handler
        self.command_handlers.insert(name.to_string(), Arc::new(handler));
        
        self
    }
    
    /// Add command help
    pub fn with_command_help(
        mut self,
        name: &str,
        description: &str,
        usage: &str,
        examples: Vec<&str>,
        arguments: Vec<CommandArgument>,
        options: Vec<CommandOption>,
    ) -> Self {
        let help = CommandHelp {
            name: name.to_string(),
            description: description.to_string(),
            usage: usage.to_string(),
            examples: examples.into_iter().map(|s| s.to_string()).collect(),
            arguments,
            options,
        };
        
        self.command_help.insert(name.to_string(), help);
        
        self
    }
    
    /// Add command schema
    pub fn with_command_schema(
        mut self,
        name: &str,
        schema: Value,
    ) -> Self {
        self.command_schema.insert(name.to_string(), schema);
        
        self
    }
    
    /// Build the command plugin
    pub fn build(self) -> Arc<dyn CommandsPlugin> {
        Arc::new(CommandPluginImpl {
            metadata: self.metadata,
            commands: self.commands,
            command_help: self.command_help,
            command_schema: self.command_schema,
            command_handlers: self.command_handlers,
        })
    }
}

/// Implementation of the CommandsPlugin trait
#[derive(Debug)]
struct CommandPluginImpl {
    /// Plugin metadata
    metadata: PluginMetadata,
    
    /// Command info
    commands: Vec<CommandInfo>,
    
    /// Command help
    command_help: HashMap<String, CommandHelp>,
    
    /// Command schema
    command_schema: HashMap<String, Value>,
    
    /// Command handlers
    command_handlers: HashMap<String, Arc<dyn Fn(Value) -> Result<Value> + Send + Sync>>,
}

#[async_trait::async_trait]
impl Plugin for CommandPluginImpl {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }
    
    async fn initialize(&self) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }
    
    async fn start(&self) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }
    
    async fn stop(&self) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl CommandsPlugin for CommandPluginImpl {
    fn get_commands(&self) -> Vec<CommandInfo> {
        self.commands.clone()
    }
    
    async fn execute_command(&self, name: &str, args: Value) -> Result<Value> {
        match self.command_handlers.get(name) {
            Some(handler) => handler(args),
            None => Err(PluginError::CommandNotFound(name.to_string())),
        }
    }
    
    fn get_command_help(&self, name: &str) -> Option<CommandHelp> {
        self.command_help.get(name).cloned()
    }
    
    fn get_command_schema(&self, name: &str) -> Option<Value> {
        self.command_schema.get(name).cloned()
    }
}

/// Builder for creating tool plugins
pub struct ToolPluginBuilder {
    /// Plugin metadata
    metadata: PluginMetadata,
    
    /// Tool info
    tools: Vec<ToolInfo>,
    
    /// Tool metadata
    tool_metadata: HashMap<String, ToolMetadata>,
    
    /// Tool handlers
    tool_handlers: HashMap<String, Arc<dyn Fn(Value) -> Result<Value> + Send + Sync>>,
    
    /// Tool availability checkers
    tool_availability_checkers: HashMap<String, Arc<dyn Fn() -> Result<ToolAvailability> + Send + Sync>>,
}

impl ToolPluginBuilder {
    /// Create a new tool plugin builder
    pub fn new(metadata: PluginMetadata) -> Self {
        Self {
            metadata,
            tools: Vec::new(),
            tool_metadata: HashMap::new(),
            tool_handlers: HashMap::new(),
            tool_availability_checkers: HashMap::new(),
        }
    }
    
    /// Add a tool
    pub fn with_tool(
        mut self,
        name: &str,
        description: &str,
    ) -> Self {
        let tool = ToolInfo {
            name: name.to_string(),
            description: description.to_string(),
            category: None,
            tags: Vec::new(),
            requires_auth: false,
        };
        
        self.tools.push(tool);
        
        self
    }
    
    /// Add a tool with category and tags
    pub fn with_tool_full(
        mut self,
        name: &str,
        description: &str,
        category: Option<&str>,
        tags: Vec<&str>,
        requires_auth: bool,
    ) -> Self {
        let tool = ToolInfo {
            name: name.to_string(),
            description: description.to_string(),
            category: category.map(|s| s.to_string()),
            tags: tags.into_iter().map(|s| s.to_string()).collect(),
            requires_auth,
        };
        
        self.tools.push(tool);
        
        self
    }
    
    /// Add a tool handler
    pub fn with_tool_handler<F>(
        mut self,
        name: &str,
        handler: F,
    ) -> Self
    where
        F: Fn(Value) -> Result<Value> + Send + Sync + 'static,
    {
        self.tool_handlers.insert(name.to_string(), Arc::new(handler));
        
        self
    }
    
    /// Add a tool availability checker
    pub fn with_tool_availability_checker<F>(
        mut self,
        name: &str,
        checker: F,
    ) -> Self
    where
        F: Fn() -> Result<ToolAvailability> + Send + Sync + 'static,
    {
        self.tool_availability_checkers.insert(name.to_string(), Arc::new(checker));
        
        self
    }
    
    /// Add tool metadata
    pub fn with_tool_metadata(
        mut self,
        name: &str,
        description: &str,
        version: &str,
        author: Option<&str>,
        homepage: Option<&str>,
        license: Option<&str>,
        dependencies: Vec<&str>,
        input_schema: Option<Value>,
        output_schema: Option<Value>,
    ) -> Self {
        let metadata = ToolMetadata {
            name: name.to_string(),
            description: description.to_string(),
            version: version.to_string(),
            author: author.map(|s| s.to_string()),
            homepage: homepage.map(|s| s.to_string()),
            license: license.map(|s| s.to_string()),
            dependencies: dependencies.into_iter().map(|s| s.to_string()).collect(),
            input_schema,
            output_schema,
        };
        
        self.tool_metadata.insert(name.to_string(), metadata);
        
        self
    }
    
    /// Build the tool plugin
    pub fn build(self) -> Arc<dyn ToolPlugin> {
        Arc::new(ToolPluginImpl {
            metadata: self.metadata,
            tools: self.tools,
            tool_metadata: self.tool_metadata,
            tool_handlers: self.tool_handlers,
            tool_availability_checkers: self.tool_availability_checkers,
        })
    }
}

/// Implementation of the ToolPlugin trait
#[derive(Debug)]
struct ToolPluginImpl {
    /// Plugin metadata
    metadata: PluginMetadata,
    
    /// Tool info
    tools: Vec<ToolInfo>,
    
    /// Tool metadata
    tool_metadata: HashMap<String, ToolMetadata>,
    
    /// Tool handlers
    tool_handlers: HashMap<String, Arc<dyn Fn(Value) -> Result<Value> + Send + Sync>>,
    
    /// Tool availability checkers
    tool_availability_checkers: HashMap<String, Arc<dyn Fn() -> Result<ToolAvailability> + Send + Sync>>,
}

#[async_trait::async_trait]
impl Plugin for ToolPluginImpl {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }
    
    async fn initialize(&self) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }
    
    async fn start(&self) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }
    
    async fn stop(&self) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl ToolPlugin for ToolPluginImpl {
    fn get_tools(&self) -> Vec<ToolInfo> {
        self.tools.clone()
    }
    
    async fn execute_tool(&self, name: &str, args: Value) -> Result<Value> {
        match self.tool_handlers.get(name) {
            Some(handler) => handler(args),
            None => Err(PluginError::CommandNotFound(name.to_string())),
        }
    }
    
    async fn check_tool_availability(&self, name: &str) -> Result<ToolAvailability> {
        match self.tool_availability_checkers.get(name) {
            Some(checker) => checker(),
            None => Ok(ToolAvailability {
                available: false,
                reason: Some(format!("Tool {} not found", name)),
                missing_dependencies: Vec::new(),
                installation_instructions: None,
            }),
        }
    }
    
    fn get_tool_metadata(&self, name: &str) -> Option<ToolMetadata> {
        self.tool_metadata.get(name).cloned()
    }
} 