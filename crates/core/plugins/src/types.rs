// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Plugin types
//!
//! This module defines the various plugin types supported by the system.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::Result;
use crate::plugin::Plugin;
use universal_constants::limits;

/// Plugin type identifiers (reserved for plugin type filtering)
#[expect(dead_code, reason = "planned feature not yet wired")]
pub const PLUGIN_TYPE_CORE: &str = "core";
#[expect(dead_code, reason = "planned feature not yet wired")]
pub const PLUGIN_TYPE_WEB: &str = "web";
#[expect(dead_code, reason = "planned feature not yet wired")]
pub const PLUGIN_TYPE_MCP: &str = "mcp";
#[expect(dead_code, reason = "planned feature not yet wired")]
pub const PLUGIN_TYPE_TOOL: &str = "tool";
#[expect(dead_code, reason = "planned feature not yet wired")]
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

/// Plugin state enumeration (reserved for plugin state management system)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(dead_code)] // Lifecycle API; serde roundtrip in tests
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

/// Plugin data format enumeration (reserved for plugin data serialization system)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(dead_code)] // Serialization format API; serde roundtrip in tests
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
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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
#[expect(
    clippy::struct_field_names,
    reason = "Domain naming convention: plugin_id, plugin_name"
)]
pub struct ResourceLimits {
    /// Maximum memory usage in bytes
    pub max_memory_bytes: Option<u64>,
    /// Maximum CPU usage percentage
    pub max_cpu_percent: Option<f64>,
    /// Maximum execution time in seconds
    pub max_execution_time_secs: Option<u64>,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_bytes: Some(limits::DEFAULT_PLUGIN_MAX_MEMORY_BYTES),
            max_cpu_percent: Some(limits::DEFAULT_PLUGIN_MAX_CPU_PERCENT),
            max_execution_time_secs: Some(limits::DEFAULT_PLUGIN_MAX_EXECUTION_TIME_SECS),
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

/// Core plugin trait for core system extensions (reserved for plugin specialization system)
#[expect(dead_code, reason = "planned feature not yet wired")]
pub trait CorePlugin: Plugin {
    /// Get the core plugin name
    fn get_core_name(&self) -> &str;

    /// Execute core plugin functionality
    async fn execute(&self, args: &[&str]) -> Result<String>;
}

/// Web plugin trait for web interface extensions
#[cfg(feature = "web")]
#[expect(dead_code, reason = "used when feature web is enabled")]
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
#[expect(dead_code, reason = "used when feature mcp is enabled")]
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

/// Tool plugin trait for tool implementations (reserved for plugin specialization system)
#[expect(dead_code, reason = "planned feature not yet wired")]
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
#[expect(dead_code, reason = "used when feature cli is enabled")]
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

#[cfg(test)]
mod tests {
    use super::*;
    use universal_constants::limits;

    fn serde_roundtrip<T: Serialize + for<'de> Deserialize<'de> + PartialEq + std::fmt::Debug>(
        value: &T,
    ) {
        let json = serde_json::to_string(value).unwrap();
        let decoded: T = serde_json::from_str(&json).unwrap();
        assert_eq!(value, &decoded);
    }

    #[test]
    fn test_plugin_type_serde() {
        serde_roundtrip(&PluginType::Builtin);
        serde_roundtrip(&PluginType::Native);
        serde_roundtrip(&PluginType::WebAssembly);
        serde_roundtrip(&PluginType::Script);
    }

    #[test]
    fn test_plugin_status_serde() {
        serde_roundtrip(&PluginStatus::Inactive);
        serde_roundtrip(&PluginStatus::Registered);
        serde_roundtrip(&PluginStatus::Loaded);
        serde_roundtrip(&PluginStatus::Initialized);
        serde_roundtrip(&PluginStatus::Running);
        serde_roundtrip(&PluginStatus::Stopped);
        serde_roundtrip(&PluginStatus::Failed);
        serde_roundtrip(&PluginStatus::Stopping);
        serde_roundtrip(&PluginStatus::Unloaded);
    }

    #[test]
    fn test_resource_limits_default() {
        let limits = ResourceLimits::default();
        assert_eq!(
            limits.max_memory_bytes,
            Some(limits::DEFAULT_PLUGIN_MAX_MEMORY_BYTES)
        );
        assert_eq!(
            limits.max_cpu_percent,
            Some(limits::DEFAULT_PLUGIN_MAX_CPU_PERCENT)
        );
        assert_eq!(
            limits.max_execution_time_secs,
            Some(limits::DEFAULT_PLUGIN_MAX_EXECUTION_TIME_SECS)
        );
    }

    #[test]
    fn test_plugin_config_default() {
        let config = PluginConfig::default();
        assert!(config.settings.is_empty());
        assert!(config.environment.is_empty());
        assert_eq!(
            config.limits.max_memory_bytes,
            Some(limits::DEFAULT_PLUGIN_MAX_MEMORY_BYTES)
        );
    }

    #[test]
    fn test_plugin_resources_default() {
        let resources = PluginResources::default();
        assert_eq!(resources.memory_usage, 0);
        assert!((resources.cpu_usage - 0.0).abs() < f64::EPSILON);
        assert_eq!(resources.file_handles, 0);
        assert_eq!(resources.network_connections, 0);
    }

    #[test]
    fn test_plugin_state_serde() {
        serde_roundtrip(&PluginState::Loaded);
        serde_roundtrip(&PluginState::Loading);
        serde_roundtrip(&PluginState::Failed);
        serde_roundtrip(&PluginState::Unloading);
        serde_roundtrip(&PluginState::Unloaded);
    }

    #[test]
    fn test_plugin_data_format_serde() {
        serde_roundtrip(&PluginDataFormat::Json);
        serde_roundtrip(&PluginDataFormat::Binary);
        serde_roundtrip(&PluginDataFormat::Text);
        serde_roundtrip(&PluginDataFormat::Custom("custom".to_string()));
    }
}
