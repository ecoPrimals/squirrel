// Plugin Interfaces Module
//
// This module defines the core interfaces for the plugin system.

use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use std::fmt::Debug;
use std::sync::Arc;
use uuid::Uuid;

use crate::plugins::errors::PluginError;
use crate::plugins::Result;

// Import base Plugin trait from squirrel_mcp
use squirrel_mcp::plugins::interfaces::Plugin;

/// Commands Plugin interface
///
/// This trait defines the interface for plugins that provide command functionality.
#[async_trait]
pub trait CommandsPlugin: Plugin {
    /// Get a list of commands provided by this plugin
    fn get_commands(&self) -> Vec<CommandInfo>;
    
    /// Execute a command
    async fn execute_command(&self, name: &str, args: Value) -> Result<Value>;
    
    /// Get help for a command
    fn get_command_help(&self, name: &str) -> Option<CommandHelp>;
    
    /// Get JSON schema for command arguments
    fn get_command_schema(&self, name: &str) -> Option<Value>;
}

/// Information about a command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandInfo {
    /// Command name
    pub name: String,
    
    /// Command description
    pub description: String,
    
    /// Command category
    pub category: Option<String>,
    
    /// Command tags
    pub tags: Vec<String>,
    
    /// Whether the command requires authentication
    pub requires_auth: bool,
}

/// Command help information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandHelp {
    /// Command name
    pub name: String,
    
    /// Command description
    pub description: String,
    
    /// Command usage
    pub usage: String,
    
    /// Command examples
    pub examples: Vec<String>,
    
    /// Command arguments
    pub arguments: Vec<CommandArgument>,
    
    /// Command options
    pub options: Vec<CommandOption>,
}

/// Command argument information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandArgument {
    /// Argument name
    pub name: String,
    
    /// Argument description
    pub description: String,
    
    /// Whether the argument is required
    pub required: bool,
    
    /// Argument data type
    pub data_type: String,
}

/// Command option information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandOption {
    /// Option name
    pub name: String,
    
    /// Option description
    pub description: String,
    
    /// Whether the option is required
    pub required: bool,
    
    /// Option data type
    pub data_type: String,
    
    /// Option short flag
    pub short_flag: Option<char>,
    
    /// Option long flag
    pub long_flag: Option<String>,
}

/// Tool Plugin interface
///
/// This trait defines the interface for plugins that provide tool functionality.
#[async_trait]
pub trait ToolPlugin: Plugin {
    /// Get a list of tools provided by this plugin
    fn get_tools(&self) -> Vec<ToolInfo>;
    
    /// Execute a tool
    async fn execute_tool(&self, name: &str, args: Value) -> Result<Value>;
    
    /// Check if a tool is available (e.g., exists and has dependencies installed)
    async fn check_tool_availability(&self, name: &str) -> Result<ToolAvailability>;
    
    /// Get tool metadata
    fn get_tool_metadata(&self, name: &str) -> Option<ToolMetadata>;
}

/// Information about a tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    /// Tool name
    pub name: String,
    
    /// Tool description
    pub description: String,
    
    /// Tool category
    pub category: Option<String>,
    
    /// Tool tags
    pub tags: Vec<String>,
    
    /// Whether the tool requires authentication
    pub requires_auth: bool,
}

/// Tool availability information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolAvailability {
    /// Whether the tool is available
    pub available: bool,
    
    /// Reason why the tool is not available (if applicable)
    pub reason: Option<String>,
    
    /// Missing dependencies (if any)
    pub missing_dependencies: Vec<String>,
    
    /// Installation instructions (if not available)
    pub installation_instructions: Option<String>,
}

/// Tool metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolMetadata {
    /// Tool name
    pub name: String,
    
    /// Tool description
    pub description: String,
    
    /// Tool version
    pub version: String,
    
    /// Tool author
    pub author: Option<String>,
    
    /// Tool homepage
    pub homepage: Option<String>,
    
    /// Tool license
    pub license: Option<String>,
    
    /// Tool dependencies
    pub dependencies: Vec<String>,
    
    /// Input schema
    pub input_schema: Option<Value>,
    
    /// Output schema
    pub output_schema: Option<Value>,
} 