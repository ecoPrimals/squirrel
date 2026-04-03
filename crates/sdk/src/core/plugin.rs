// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Plugin trait and core functionality for the Squirrel Plugin SDK
//!
//! This module provides the core plugin trait and related types that WASM plugins
//! must implement to integrate with the Squirrel plugin system.

use crate::config::PluginConfig;
use crate::mcp::McpCapabilities;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

/// Command information structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandInfo {
    /// Command name
    pub name: String,
    /// Command description
    pub description: String,
    /// Command category
    pub category: Option<String>,
    /// Parameters schema
    pub parameters: Option<serde_json::Value>,
}

/// Plugin lifecycle states
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginStatus {
    /// Plugin is not yet initialized
    Uninitialized,
    /// Plugin is being initialized
    Initializing,
    /// Plugin is active and ready
    Active,
    /// Plugin is paused
    Paused,
    /// Plugin is being stopped
    Stopping,
    /// Plugin has stopped
    Stopped,
    /// Plugin encountered an error
    Error(String),
}

/// Plugin metadata and status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    /// Plugin unique identifier
    pub id: String,
    /// Plugin name
    pub name: String,
    /// Plugin version
    pub version: String,
    /// Current state
    pub state: PluginStatus,
    /// Configuration
    pub config: PluginConfig,
    /// Statistics
    pub stats: PluginStats,
    /// Capabilities
    pub capabilities: Vec<String>,
    /// Plugin description
    pub description: String,
    /// Plugin author
    pub author: String,
    /// Plugin license
    pub license: String,
    /// Repository URL
    pub repository: Option<String>,
    /// Keywords for discovery
    pub keywords: Vec<String>,
    /// Additional metadata
    pub metadata: serde_json::Value,
}

/// Plugin execution statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginStats {
    /// Number of commands executed
    pub commands_executed: u64,
    /// Total execution time in milliseconds
    pub total_execution_time: u64,
    /// Number of errors encountered
    pub error_count: u64,
    /// Memory usage in bytes
    pub memory_usage: u64,
    /// CPU time used in milliseconds
    pub cpu_time: u64,
    /// Start time
    pub start_time: String,
    /// Last activity time
    pub last_activity: String,
}

impl Default for PluginStats {
    fn default() -> Self {
        let now = crate::utils::current_timestamp();
        Self {
            commands_executed: 0,
            total_execution_time: 0,
            error_count: 0,
            memory_usage: 0,
            cpu_time: 0,
            start_time: now.to_string(),
            last_activity: now.to_string(),
        }
    }
}

/// Simple permission enum for plugin capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Permission {
    /// Access to local storage
    LocalStorage,
    /// Access to session storage
    SessionStorage,
    /// File system read access
    FileSystemRead(String),
    /// File system write access
    FileSystemWrite(String),
    /// Network access
    NetworkAccess,
}

/// Plugin capabilities
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PluginCapabilities {
    /// Commands that this plugin provides
    pub commands: Vec<CommandInfo>,
    /// Events that this plugin can emit
    pub events: Vec<String>,
    /// Resources that this plugin can provide
    pub resources: Vec<String>,
    /// UI components that this plugin provides
    pub ui_components: Vec<String>,
    /// Permissions that this plugin requires
    pub permissions: Vec<Permission>,
}

impl PluginCapabilities {
    /// Create a new capabilities instance with all capabilities
    pub fn all() -> Self {
        Self {
            commands: vec![],
            events: vec![],
            resources: vec![],
            ui_components: vec![],
            permissions: vec![Permission::LocalStorage, Permission::SessionStorage],
        }
    }

    /// Create a new capabilities instance with no capabilities
    pub fn none() -> Self {
        Self::default()
    }
}

/// Plugin runtime context for execution
#[derive(Debug)]
pub struct RuntimeContext {
    /// Plugin ID
    pub plugin_id: String,
    /// Plugin configuration
    pub config: PluginConfig,
    /// Plugin capabilities
    pub capabilities: PluginCapabilities,
    /// MCP-specific capabilities
    pub mcp_capabilities: McpCapabilities,
}

/// Plugin execution context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginContext {
    /// Plugin ID
    pub plugin_id: String,

    /// Current working directory
    pub working_directory: String,

    /// Environment variables
    pub environment: HashMap<String, String>,

    /// Plugin configuration
    pub config: serde_json::Value,

    /// User data
    pub user_data: serde_json::Value,
}

impl PluginContext {
    /// Create a new plugin context
    pub fn new(plugin_id: String) -> Self {
        Self {
            plugin_id,
            working_directory: "/".to_string(),
            environment: HashMap::new(),
            config: serde_json::json!({}),
            user_data: serde_json::json!({}),
        }
    }

    /// Get environment variable
    pub fn get_env(&self, key: &str) -> Option<&String> {
        self.environment.get(key)
    }

    /// Set environment variable
    pub fn set_env(&mut self, key: String, value: String) {
        self.environment.insert(key, value);
    }

    /// Get config value
    pub fn get_config(&self, key: &str) -> Option<&serde_json::Value> {
        self.config.get(key)
    }
}

/// Result of executing a plugin command
#[derive(Debug, Clone)]
pub struct PluginCommandResult {
    /// Whether the command succeeded
    pub success: bool,
    /// Result data
    pub data: String,
    /// Error message if failed
    pub error: Option<String>,
    /// Additional metadata
    pub metadata: String,
}

impl PluginCommandResult {
    /// Create a successful result
    pub fn success(data: String) -> Self {
        Self {
            success: true,
            data,
            error: None,
            metadata: "{}".to_string(),
        }
    }

    /// Create a failed result
    pub fn error(error: String) -> Self {
        Self {
            success: false,
            data: "{}".to_string(),
            error: Some(error),
            metadata: "{}".to_string(),
        }
    }

    /// Add metadata
    pub fn with_metadata(mut self, metadata: String) -> Self {
        self.metadata = metadata;
        self
    }
}

/// WebAssembly plugin trait
pub trait WasmPlugin: Send + Sync {
    /// Get plugin information
    fn get_info(&self) -> PluginInfo;

    /// Initialize the plugin
    fn initialize(&mut self, config: JsValue) -> Result<(), JsValue>;

    /// Start the plugin
    fn start(&mut self) -> Result<(), JsValue>;

    /// Stop the plugin
    fn stop(&mut self) -> Result<(), JsValue>;

    /// Pause the plugin
    fn pause(&mut self) -> Result<(), JsValue>;

    /// Resume the plugin
    fn resume(&mut self) -> Result<(), JsValue>;

    /// Handle a command
    fn handle_command(&self, command: &str, params: JsValue) -> Result<JsValue, JsValue>;

    /// Handle an event
    fn handle_event(&self, event: JsValue) -> Result<(), JsValue>;

    /// Get plugin statistics
    fn get_stats(&self) -> PluginStats;

    /// Get plugin capabilities
    fn get_capabilities(&self) -> PluginCapabilities;

    /// Shutdown the plugin
    fn shutdown(&mut self) -> Result<(), JsValue>;

    /// Check if plugin is initialized
    fn is_initialized(&self) -> bool;

    /// Get plugin status
    fn get_status(&self) -> PluginStatus;
}

/// Base plugin implementation
#[wasm_bindgen]
pub struct BasePlugin {
    info: PluginInfo,
    state: PluginStatus,
    capabilities: PluginCapabilities,
    initialized: bool,
}

#[wasm_bindgen]
impl BasePlugin {
    /// Create a new base plugin
    #[wasm_bindgen(constructor)]
    pub fn new(name: String, version: String) -> Self {
        let id = crate::utils::generate_uuid();
        let config = PluginConfig::default();

        Self {
            info: PluginInfo {
                id,
                name,
                version,
                state: PluginStatus::Uninitialized,
                config,
                stats: PluginStats::default(),
                capabilities: Vec::new(),
                description: String::new(),
                author: String::new(),
                license: "MIT".to_string(),
                repository: None,
                keywords: Vec::new(),
                metadata: serde_json::json!({}),
            },
            state: PluginStatus::Uninitialized,
            capabilities: PluginCapabilities::default(),
            initialized: false,
        }
    }

    /// Get the plugin ID
    #[wasm_bindgen(getter)]
    pub fn id(&self) -> String {
        self.info.id.clone()
    }

    /// Get the plugin name
    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.info.name.clone()
    }

    /// Get the plugin version
    #[wasm_bindgen(getter)]
    pub fn version(&self) -> String {
        self.info.version.clone()
    }

    /// Get the current state
    #[wasm_bindgen(getter)]
    pub fn state(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.state).unwrap_or(JsValue::NULL)
    }

    /// Initialize the plugin
    #[wasm_bindgen]
    pub async fn initialize(&mut self, config: JsValue) -> Result<(), JsValue> {
        self.set_state(PluginStatus::Initializing);

        // Parse configuration
        let config: PluginConfig = serde_wasm_bindgen::from_value(config)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        // Validate configuration
        config
            .validate()
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        self.info.config = config;
        self.set_state(PluginStatus::Active);

        Ok(())
    }

    /// Start the plugin
    #[wasm_bindgen]
    pub async fn start(&mut self) -> Result<(), JsValue> {
        match self.state {
            PluginStatus::Uninitialized => {
                return Err(JsValue::from_str(
                    "Plugin must be initialized before starting",
                ));
            }
            PluginStatus::Active => {
                return Ok(()); // Already active
            }
            _ => {}
        }

        self.set_state(PluginStatus::Active);
        Ok(())
    }

    /// Stop the plugin
    #[wasm_bindgen]
    pub async fn stop(&mut self) -> Result<(), JsValue> {
        self.set_state(PluginStatus::Stopping);

        // Cleanup resources here

        self.set_state(PluginStatus::Stopped);
        Ok(())
    }

    /// Pause the plugin
    #[wasm_bindgen]
    pub async fn pause(&mut self) -> Result<(), JsValue> {
        if self.state != PluginStatus::Active {
            return Err(JsValue::from_str("Can only pause active plugins"));
        }

        self.set_state(PluginStatus::Paused);
        Ok(())
    }

    /// Resume the plugin
    #[wasm_bindgen]
    pub async fn resume(&mut self) -> Result<(), JsValue> {
        if self.state != PluginStatus::Paused {
            return Err(JsValue::from_str("Can only resume paused plugins"));
        }

        self.set_state(PluginStatus::Active);
        Ok(())
    }

    /// Handle a command (default implementation)
    #[wasm_bindgen]
    pub async fn handle_command(
        &self,
        command: &str,
        _params: JsValue,
    ) -> Result<JsValue, JsValue> {
        match command {
            "ping" => {
                let response = serde_json::json!({
                    "pong": true,
                    "timestamp": crate::utils::current_timestamp()
                });
                Ok(serde_wasm_bindgen::to_value(&response)?)
            }
            "info" => self.get_info(),
            "health" => {
                let healthy = self.health_check().await?;
                let response = serde_json::json!({ "healthy": healthy });
                Ok(serde_wasm_bindgen::to_value(&response)?)
            }
            _ => Err(JsValue::from_str(&format!("Unknown command: {}", command))),
        }
    }

    /// Get plugin information
    #[wasm_bindgen]
    pub fn get_info(&self) -> Result<JsValue, JsValue> {
        serde_wasm_bindgen::to_value(&self.info).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Health check
    #[wasm_bindgen]
    pub async fn health_check(&self) -> Result<bool, JsValue> {
        // Basic health check - plugin is healthy if it's active
        Ok(self.state == PluginStatus::Active)
    }

    /// Add a capability
    #[wasm_bindgen]
    pub fn add_capability(&mut self, capability: String) {
        if !self.info.capabilities.contains(&capability) {
            self.info.capabilities.push(capability);
        }
    }

    /// Check if plugin has a capability
    #[wasm_bindgen]
    pub fn has_capability(&self, capability: &str) -> bool {
        self.info.capabilities.contains(&capability.to_string())
    }

    /// Update statistics
    #[wasm_bindgen]
    pub fn update_stats(&mut self, execution_time: u64, success: bool) {
        self.info.stats.commands_executed += 1;
        self.info.stats.total_execution_time += execution_time;
        if !success {
            self.info.stats.error_count += 1;
        }
        self.info.stats.last_activity = crate::utils::current_timestamp().to_string();
    }

    /// Set capabilities
    #[wasm_bindgen]
    pub fn set_capabilities(&mut self, capabilities: JsValue) -> Result<(), JsValue> {
        self.capabilities = serde_wasm_bindgen::from_value(capabilities)?;
        Ok(())
    }

    /// Check if plugin is initialized
    #[wasm_bindgen]
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
}

impl BasePlugin {
    /// Set the plugin state
    fn set_state(&mut self, state: PluginStatus) {
        self.state = state.clone();
        self.info.state = state;
    }
}

impl WasmPlugin for BasePlugin {
    fn get_info(&self) -> PluginInfo {
        self.info.clone()
    }

    fn initialize(&mut self, config: JsValue) -> Result<(), JsValue> {
        self.set_state(PluginStatus::Initializing);

        // Parse configuration
        let config: PluginConfig = serde_wasm_bindgen::from_value(config)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        // Validate configuration
        config
            .validate()
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        self.info.config = config;
        self.set_state(PluginStatus::Active);

        Ok(())
    }

    fn start(&mut self) -> Result<(), JsValue> {
        match self.state {
            PluginStatus::Uninitialized => {
                return Err(JsValue::from_str(
                    "Plugin must be initialized before starting",
                ));
            }
            PluginStatus::Active => {
                return Ok(()); // Already active
            }
            _ => {}
        }

        self.set_state(PluginStatus::Active);
        Ok(())
    }

    fn stop(&mut self) -> Result<(), JsValue> {
        self.set_state(PluginStatus::Stopping);

        // Cleanup resources here

        self.set_state(PluginStatus::Stopped);
        Ok(())
    }

    fn pause(&mut self) -> Result<(), JsValue> {
        if self.state != PluginStatus::Active {
            return Err(JsValue::from_str("Can only pause active plugins"));
        }

        self.set_state(PluginStatus::Paused);
        Ok(())
    }

    fn resume(&mut self) -> Result<(), JsValue> {
        if self.state != PluginStatus::Paused {
            return Err(JsValue::from_str("Can only resume paused plugins"));
        }

        self.set_state(PluginStatus::Active);
        Ok(())
    }

    fn handle_command(&self, command: &str, _params: JsValue) -> Result<JsValue, JsValue> {
        // Default implementation - can be overridden by specific plugins
        Ok(JsValue::from_str(&format!(
            "Command '{}' not implemented",
            command
        )))
    }

    fn handle_event(&self, _event: JsValue) -> Result<(), JsValue> {
        // Default implementation - can be overridden by specific plugins
        Ok(())
    }

    fn get_stats(&self) -> PluginStats {
        self.info.stats.clone()
    }

    fn get_capabilities(&self) -> PluginCapabilities {
        self.capabilities.clone()
    }

    fn shutdown(&mut self) -> Result<(), JsValue> {
        self.set_state(PluginStatus::Stopping);

        // Cleanup resources here synchronously

        self.set_state(PluginStatus::Stopped);
        Ok(())
    }

    fn is_initialized(&self) -> bool {
        !matches!(self.state, PluginStatus::Uninitialized)
    }

    fn get_status(&self) -> PluginStatus {
        self.state.clone()
    }
}

/// Plugin manager for handling multiple plugins
#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "Plugin tests use expect on wasm and manager APIs"
)]
mod tests {
    use super::super::manager::{PluginManager, utils};
    use super::*;
    use crate::config::PluginConfig;
    use crate::mcp::McpCapabilities;

    #[test]
    fn test_plugin_creation() {
        let plugin = BasePlugin::new("test".to_string(), "1.0.0".to_string());
        let info = WasmPlugin::get_info(&plugin);
        assert_eq!(info.name, "test");
        assert_eq!(info.version, "1.0.0");
    }

    #[tokio::test]
    #[cfg(target_arch = "wasm32")]
    async fn test_plugin_lifecycle() {
        let mut plugin = BasePlugin::new("test".to_string(), "1.0.0".to_string());
        let config = PluginConfig::default();
        let js_config = serde_wasm_bindgen::to_value(&config).expect("config to wasm");

        // Test initialization
        assert!(plugin.initialize(js_config).await.is_ok());
        let info = WasmPlugin::get_info(&plugin);
        assert_eq!(info.name, "test");

        // Test state
        let info = WasmPlugin::get_info(&plugin);
        assert_eq!(info.name, "test");
    }

    #[test]
    fn test_plugin_validation() {
        let plugin = BasePlugin::new("test".to_string(), "1.0.0".to_string());
        let info = WasmPlugin::get_info(&plugin);
        assert_eq!(info.name, "test");
    }

    #[test]
    fn test_plugin_stats_default() {
        let stats = PluginStats::default();
        assert_eq!(stats.commands_executed, 0);
        assert_eq!(stats.error_count, 0);
        assert!(!stats.start_time.is_empty());
        assert!(!stats.last_activity.is_empty());
    }

    #[test]
    fn test_plugin_capabilities_all() {
        let caps = PluginCapabilities::all();
        assert_eq!(caps.permissions.len(), 2);
        // Check we have LocalStorage and SessionStorage (Permission doesn't impl PartialEq)
        let has_local = caps
            .permissions
            .iter()
            .any(|p| matches!(p, Permission::LocalStorage));
        let has_session = caps
            .permissions
            .iter()
            .any(|p| matches!(p, Permission::SessionStorage));
        assert!(has_local);
        assert!(has_session);
    }

    #[test]
    fn test_plugin_capabilities_none() {
        let caps = PluginCapabilities::none();
        assert!(caps.commands.is_empty());
        assert!(caps.events.is_empty());
        assert!(caps.permissions.is_empty());
    }

    #[test]
    fn test_plugin_context_new() {
        let ctx = PluginContext::new("plugin-123".to_string());
        assert_eq!(ctx.plugin_id, "plugin-123");
        assert_eq!(ctx.working_directory, "/");
        assert!(ctx.environment.is_empty());
    }

    #[test]
    fn test_plugin_context_get_set_env() {
        let mut ctx = PluginContext::new("test".to_string());
        ctx.set_env("KEY".to_string(), "value".to_string());
        assert_eq!(ctx.get_env("KEY"), Some(&"value".to_string()));
        assert_eq!(ctx.get_env("MISSING"), None);
    }

    #[test]
    fn test_plugin_command_result_success() {
        let result = PluginCommandResult::success("data".to_string());
        assert!(result.success);
        assert_eq!(result.data, "data");
        assert!(result.error.is_none());
    }

    #[test]
    fn test_plugin_command_result_error() {
        let result = PluginCommandResult::error("failed".to_string());
        assert!(!result.success);
        assert_eq!(result.error, Some("failed".to_string()));
    }

    #[test]
    fn test_plugin_command_result_with_metadata() {
        let result = PluginCommandResult::success("data".to_string())
            .with_metadata(r#"{"key":"value"}"#.to_string());
        assert_eq!(result.metadata, r#"{"key":"value"}"#);
    }

    #[test]
    fn test_plugin_status_equality() {
        assert_eq!(PluginStatus::Uninitialized, PluginStatus::Uninitialized);
        assert_eq!(PluginStatus::Active, PluginStatus::Active);
        assert_ne!(PluginStatus::Uninitialized, PluginStatus::Active);
        assert_eq!(
            PluginStatus::Error("msg".to_string()),
            PluginStatus::Error("msg".to_string())
        );
    }

    #[test]
    fn test_command_info_creation() {
        let cmd = CommandInfo {
            name: "test_cmd".to_string(),
            description: "A test".to_string(),
            category: Some("utils".to_string()),
            parameters: Some(serde_json::json!({"type": "string"})),
        };
        assert_eq!(cmd.name, "test_cmd");
        assert_eq!(cmd.description, "A test");
        assert_eq!(cmd.category, Some("utils".to_string()));
    }

    #[test]
    fn test_plugin_context_get_config() {
        let mut ctx = PluginContext::new("test".to_string());
        ctx.config = serde_json::json!({"key": "value"});
        assert_eq!(ctx.get_config("key"), Some(&serde_json::json!("value")));
        assert_eq!(ctx.get_config("missing"), None);
    }

    #[test]
    fn test_plugin_manager_lifecycle() {
        let manager = PluginManager::new();
        assert!(manager.list_plugins().expect("list_plugins").is_empty());
        assert!(!manager.has_plugin("nonexistent").expect("has_plugin"));
    }

    #[test]
    fn test_plugin_manager_has_plugin_empty() {
        let manager = PluginManager::new();
        assert!(!manager.has_plugin("x").expect("has_plugin"));
    }

    #[test]
    fn test_validate_plugin_info_valid() {
        let info = PluginInfo {
            id: "test-id".to_string(),
            name: "Test".to_string(),
            version: "1.0.0".to_string(),
            state: PluginStatus::Uninitialized,
            config: PluginConfig::default(),
            stats: PluginStats::default(),
            capabilities: vec![],
            description: String::new(),
            author: String::new(),
            license: "MIT".to_string(),
            repository: None,
            keywords: vec![],
            metadata: serde_json::json!({}),
        };
        assert!(utils::validate_plugin_info(&info).is_ok());
    }

    #[test]
    fn test_validate_plugin_info_empty_id() {
        let mut info = PluginInfo {
            id: String::new(),
            name: "Test".to_string(),
            version: "1.0.0".to_string(),
            state: PluginStatus::Uninitialized,
            config: PluginConfig::default(),
            stats: PluginStats::default(),
            capabilities: vec![],
            description: String::new(),
            author: String::new(),
            license: "MIT".to_string(),
            repository: None,
            keywords: vec![],
            metadata: serde_json::json!({}),
        };
        assert!(utils::validate_plugin_info(&info).is_err());
        info.id = "x".to_string();
        info.name = String::new();
        assert!(utils::validate_plugin_info(&info).is_err());
        info.name = "Test".to_string();
        info.version = String::new();
        assert!(utils::validate_plugin_info(&info).is_err());
        info.version = "invalid".to_string();
        assert!(utils::validate_plugin_info(&info).is_err());
    }

    #[test]
    fn test_plugin_utils_create_default_context() {
        let ctx = utils::create_default_context("my-plugin".to_string());
        assert_eq!(ctx.plugin_id, "my-plugin");
    }

    #[test]
    fn runtime_context_stores_mcp_capabilities() {
        let ctx = RuntimeContext {
            plugin_id: "pid".to_string(),
            config: PluginConfig::default(),
            capabilities: PluginCapabilities::none(),
            mcp_capabilities: McpCapabilities::default(),
        };
        assert_eq!(ctx.plugin_id, "pid");
    }

    #[test]
    fn wasm_plugin_trait_stats_capabilities_and_events_no_js() {
        let p = BasePlugin::new("stats".to_string(), "1.0.0".to_string());
        let s = WasmPlugin::get_stats(&p);
        assert_eq!(s.commands_executed, 0);
        let caps = WasmPlugin::get_capabilities(&p);
        assert!(caps.commands.is_empty());
        assert!(WasmPlugin::handle_event(&p, wasm_bindgen::JsValue::NULL).is_ok());
    }
}
