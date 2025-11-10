//! Plugin types
//!
//! This module defines the various plugin types supported by the system.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::plugin::Plugin;
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

/// Plugin type enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginType {
    /// Built-in plugin
    Builtin,
    /// Native (shared library) plugin
    Native,
    /// WebAssembly plugin
    WebAssembly,
    /// Script plugin
    Script,
}

/// Plugin state enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginState {
    /// Plugin is loaded and ready
    Loaded,
    /// Plugin is currently loading
    Loading,
    /// Plugin failed to load
    Failed,
    /// Plugin is unloading
    Unloading,
    /// Plugin is unloaded
    Unloaded,
}

/// Plugin data format enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginDataFormat {
    /// JSON format
    Json,
    /// Binary format
    Binary,
    /// Text format
    Text,
    /// Custom format
    Custom(String),
}

/// Plugin resource usage information
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PluginResources {
    /// Memory usage in bytes
    pub memory_usage: u64,
    /// CPU usage percentage
    pub cpu_usage: f64,
    /// Number of open file handles
    pub file_handles: u32,
    /// Number of network connections
    pub network_connections: u32,
}

/// Plugin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    /// Configuration settings
    pub settings: HashMap<String, serde_json::Value>,
    /// Plugin-specific environment variables
    pub environment: HashMap<String, String>,
    /// Resource limits
    pub limits: ResourceLimits,
}

/// Resource limits for plugins
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum memory usage in bytes
    pub max_memory_bytes: Option<u64>,
    /// Maximum CPU usage percentage
    pub max_cpu_percent: Option<f64>,
    /// Maximum execution time in seconds
    pub max_execution_time_secs: Option<u64>,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            settings: HashMap::new(),
            environment: HashMap::new(),
            limits: ResourceLimits::default(),
        }
    }
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_bytes: Some(100 * 1024 * 1024), // 100MB default
            max_cpu_percent: Some(50.0),               // 50% CPU default
            max_execution_time_secs: Some(300),        // 5 minutes default
        }
    }
}

/// Plugin status enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginStatus {
    /// Plugin is inactive
    Inactive,
    /// Plugin is registered but not loaded
    Registered,
    /// Plugin is loaded but not running
    Loaded,
    /// Plugin is initialized and ready
    Initialized,
    /// Plugin is running
    Running,
    /// Plugin is stopped
    Stopped,
    /// Plugin failed to start
    Failed,
    /// Plugin is stopping
    Stopping,
    /// Plugin is unloaded
    Unloaded,
}

// PluginMetadata removed - use squirrel_interfaces::plugins::PluginMetadata instead
// This was duplicate/unused code. The canonical version is in squirrel-interfaces crate.

/// Core plugin trait for core system extensions
pub trait CorePlugin: Plugin {
    /// Get the core plugin name
    fn get_core_name(&self) -> &str;

    /// Execute core plugin functionality
    async fn execute(&self, args: &[&str]) -> Result<String>;
}

/// Web plugin trait for web interface extensions
#[cfg(feature = "web")]
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
pub trait McpPlugin: Plugin {
    /// Get the MCP plugin name
    fn get_mcp_name(&self) -> &str;

    /// Register MCP plugin commands
    async fn register_commands(&self) -> Result<()>;

    /// Handle MCP plugin message
    async fn handle_message(
        &self,
        message: crate::mcp::McpMessage,
    ) -> Result<crate::mcp::McpMessage>;

    /// Initialize the MCP plugin
    async fn mcp_initialize(&self) -> Result<()>;

    /// Shutdown the MCP plugin
    async fn mcp_shutdown(&self) -> Result<()>;
}

/// Tool plugin trait for tool implementations
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
