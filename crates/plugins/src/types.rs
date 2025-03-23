//! Plugin types
//!
//! This module defines the various plugin types supported by the system.


use async_trait::async_trait;

use crate::core::Plugin;
use crate::Result;

/// Plugin type identifiers
/// Core plugin type for system extensions
pub const PLUGIN_TYPE_CORE: &str = "core";
/// Web plugin type for interface extensions
pub const PLUGIN_TYPE_WEB: &str = "web";
/// MCP (Machine Context Protocol) plugin type for protocol extensions
pub const PLUGIN_TYPE_MCP: &str = "mcp";
/// Tool plugin type for utility extensions
pub const PLUGIN_TYPE_TOOL: &str = "tool";
/// CLI plugin type for command-line interface extensions
pub const PLUGIN_TYPE_CLI: &str = "cli";

/// Core plugin trait for core system extensions
#[async_trait]
pub trait CorePlugin: Plugin {
    /// Get the core plugin name
    fn get_core_name(&self) -> &str;
    
    /// Execute core plugin functionality
    async fn execute(&self, args: &[&str]) -> Result<String>;
}

/// Web plugin trait for web interface extensions
#[cfg(feature = "web")]
#[async_trait]
pub trait WebPlugin: Plugin {
    /// Get the web plugin assets directory
    fn get_assets_dir(&self) -> Option<&str>;
    
    /// Get the web plugin routes
    fn get_routes(&self) -> Vec<crate::web::WebPluginRoute>;
    
    /// Get the web plugin UI components
    fn get_ui_components(&self) -> Vec<crate::web::WebPluginComponent>;
    
    /// Get the web plugin API endpoints
    fn get_api_endpoints(&self) -> Vec<crate::web::WebPluginEndpoint>;
    
    /// Initialize the web plugin
    async fn web_initialize(&self) -> Result<()>;
    
    /// Shutdown the web plugin
    async fn web_shutdown(&self) -> Result<()>;
}

/// MCP plugin trait for MCP protocol extensions
#[cfg(feature = "mcp")]
#[async_trait]
pub trait McpPlugin: Plugin {
    /// Get the MCP plugin name
    fn get_mcp_name(&self) -> &str;
    
    /// Register MCP plugin commands
    async fn register_commands(&self) -> Result<()>;
    
    /// Handle MCP plugin message
    async fn handle_message(&self, message: crate::mcp::McpMessage) -> Result<crate::mcp::McpMessage>;
    
    /// Initialize the MCP plugin
    async fn mcp_initialize(&self) -> Result<()>;
    
    /// Shutdown the MCP plugin
    async fn mcp_shutdown(&self) -> Result<()>;
}

/// Tool plugin trait for tool implementations
#[async_trait]
pub trait ToolPlugin: Plugin {
    /// Get the tool plugin name
    fn get_tool_name(&self) -> &str;
    
    /// Get the tool plugin description
    fn get_tool_description(&self) -> &str;
    
    /// Get the tool plugin version
    fn get_tool_version(&self) -> &str;
    
    /// Execute the tool plugin
    async fn execute_tool(&self, args: &[&str]) -> Result<String>;
}

/// CLI plugin trait for CLI interface extensions
#[cfg(feature = "cli")]
#[async_trait]
pub trait CliPlugin: Plugin {
    /// Get the CLI plugin name
    fn get_cli_name(&self) -> &str;
    
    /// Get the CLI plugin commands
    fn get_cli_commands(&self) -> Vec<crate::cli::CliCommand>;
    
    /// Initialize the CLI plugin
    async fn cli_initialize(&self) -> Result<()>;
    
    /// Shutdown the CLI plugin
    async fn cli_shutdown(&self) -> Result<()>;
} 