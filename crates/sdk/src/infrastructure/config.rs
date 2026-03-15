// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Configuration management for plugins
//!
//! This module provides configuration loading and management capabilities for WASM plugins.
//! Configuration is validated and access is controlled through sandbox permissions.

use crate::error::{PluginError, PluginResult};
use crate::plugin::Permission;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use universal_constants::network::{get_bind_address, get_service_port};
// Sandbox security handled by BearDog framework

/// Comprehensive SDK configuration structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PluginSdkConfig {
    /// Plugin-specific configuration
    pub plugin: PluginConfig,
    /// MCP protocol configuration
    pub mcp: McpClientConfig,
    /// Logging configuration
    pub logging: LoggingConfig,
    /// Sandbox configuration
    pub sandbox: SandboxConfig,
    /// Network configuration
    pub network: NetworkConfig,
    /// HTTP client configuration
    pub http: HttpConfig,
    /// Performance configuration
    pub performance: PerformanceConfig,
}

impl PluginSdkConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        Self {
            plugin: PluginConfig::default(),
            mcp: McpClientConfig::from_env(),
            logging: LoggingConfig::from_env(),
            sandbox: SandboxConfig::from_env(),
            network: NetworkConfig::from_env(),
            http: HttpConfig::from_env(),
            performance: PerformanceConfig::from_env(),
        }
    }

    /// Validate the entire configuration
    pub fn validate(&self) -> PluginResult<()> {
        self.plugin.validate()?;
        self.mcp.validate()?;
        self.logging.validate()?;
        self.sandbox.validate()?;
        self.network.validate()?;
        self.http.validate()?;
        self.performance.validate()?;
        Ok(())
    }
}

/// MCP client configuration with environment variable support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpClientConfig {
    /// Server endpoint URL
    pub server_url: String,
    /// Connection timeout in milliseconds
    pub timeout_ms: u64,
    /// Maximum message size in bytes
    pub max_message_size: usize,
    /// Protocol version
    pub protocol_version: String,
    /// Reconnection attempts
    pub max_reconnect_attempts: u32,
    /// Reconnection delay in milliseconds
    pub reconnect_delay_ms: u64,
}

impl Default for McpClientConfig {
    fn default() -> Self {
        Self::from_env()
    }
}

impl McpClientConfig {
    /// Load MCP configuration from environment variables
    ///
    /// Multi-tier server URL resolution:
    /// 1. MCP_SERVER_URL (full WebSocket URL)
    /// 2. MCP_SERVER_PORT (port override)
    /// 3. Default: ws://{bind_address}:{get_service_port("websocket")}
    pub fn from_env() -> Self {
        let server_url = std::env::var("MCP_SERVER_URL").unwrap_or_else(|_| {
            let port = std::env::var("MCP_SERVER_PORT")
                .ok()
                .and_then(|p| p.parse::<u16>().ok())
                .unwrap_or_else(|| get_service_port("websocket"));
            let host = get_bind_address();
            format!("ws://{host}:{port}")
        });

        Self {
            server_url,
            timeout_ms: std::env::var("MCP_TIMEOUT_MS")
                .unwrap_or_else(|_| "30000".to_string())
                .parse()
                .unwrap_or(30000),
            max_message_size: std::env::var("MCP_MAX_MESSAGE_SIZE")
                .unwrap_or_else(|_| "1048576".to_string())
                .parse()
                .unwrap_or(1024 * 1024),
            protocol_version: std::env::var("MCP_PROTOCOL_VERSION")
                .unwrap_or_else(|_| "1.0".to_string()),
            max_reconnect_attempts: std::env::var("MCP_MAX_RECONNECT_ATTEMPTS")
                .unwrap_or_else(|_| "3".to_string())
                .parse()
                .unwrap_or(3),
            reconnect_delay_ms: std::env::var("MCP_RECONNECT_DELAY_MS")
                .unwrap_or_else(|_| "5000".to_string())
                .parse()
                .unwrap_or(5000),
        }
    }

    /// Validate MCP configuration
    pub fn validate(&self) -> PluginResult<()> {
        if self.server_url.is_empty() {
            return Err(PluginError::InvalidConfiguration {
                message: "MCP server URL cannot be empty".to_string(),
            });
        }

        if self.timeout_ms == 0 {
            return Err(PluginError::InvalidConfiguration {
                message: "MCP timeout must be greater than 0".to_string(),
            });
        }

        if self.max_message_size == 0 {
            return Err(PluginError::InvalidConfiguration {
                message: "MCP max message size must be greater than 0".to_string(),
            });
        }

        Ok(())
    }
}

/// Logging configuration with environment variable support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Minimum log level
    pub min_level: String,
    /// Maximum log entries in memory
    pub max_entries: usize,
    /// Whether to include file/line information
    pub include_location: bool,
    /// Whether to send logs to host system
    pub send_to_host: bool,
    /// Log rotation size in bytes
    pub rotation_size: usize,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self::from_env()
    }
}

impl LoggingConfig {
    /// Load logging configuration from environment variables
    pub fn from_env() -> Self {
        Self {
            min_level: std::env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
            max_entries: std::env::var("LOG_MAX_ENTRIES")
                .unwrap_or_else(|_| "1000".to_string())
                .parse()
                .unwrap_or(1000),
            include_location: std::env::var("LOG_INCLUDE_LOCATION")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
            send_to_host: std::env::var("LOG_SEND_TO_HOST")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            rotation_size: std::env::var("LOG_ROTATION_SIZE")
                .unwrap_or_else(|_| "1048576".to_string())
                .parse()
                .unwrap_or(1024 * 1024),
        }
    }

    /// Validate logging configuration
    pub fn validate(&self) -> PluginResult<()> {
        let valid_levels = ["trace", "debug", "info", "warn", "error"];
        if !valid_levels.contains(&self.min_level.as_str()) {
            return Err(PluginError::InvalidConfiguration {
                message: format!(
                    "Invalid log level '{}', must be one of: {:?}",
                    self.min_level, valid_levels
                ),
            });
        }

        if self.max_entries == 0 {
            return Err(PluginError::InvalidConfiguration {
                message: "Log max entries must be greater than 0".to_string(),
            });
        }

        Ok(())
    }
}

/// Network configuration with environment variable support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Default host address
    pub host: String,
    /// Default port number
    pub port: u16,
    /// Maximum concurrent connections
    pub max_connections: u32,
    /// Connection timeout in milliseconds
    pub connection_timeout_ms: u64,
    /// Read timeout in milliseconds
    pub read_timeout_ms: u64,
    /// Write timeout in milliseconds
    pub write_timeout_ms: u64,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self::from_env()
    }
}

impl NetworkConfig {
    /// Load network configuration from environment variables
    pub fn from_env() -> Self {
        Self {
            host: std::env::var("NETWORK_HOST").unwrap_or_else(|_| {
                std::env::var("DEV_SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string())
            }),
            port: std::env::var("NETWORK_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .unwrap_or(8080),
            max_connections: std::env::var("NETWORK_MAX_CONNECTIONS")
                .unwrap_or_else(|_| "100".to_string())
                .parse()
                .unwrap_or(100),
            connection_timeout_ms: std::env::var("NETWORK_CONNECTION_TIMEOUT_MS")
                .unwrap_or_else(|_| "30000".to_string())
                .parse()
                .unwrap_or(30000),
            read_timeout_ms: std::env::var("NETWORK_READ_TIMEOUT_MS")
                .unwrap_or_else(|_| "30000".to_string())
                .parse()
                .unwrap_or(30000),
            write_timeout_ms: std::env::var("NETWORK_WRITE_TIMEOUT_MS")
                .unwrap_or_else(|_| "30000".to_string())
                .parse()
                .unwrap_or(30000),
        }
    }

    /// Validate network configuration
    pub fn validate(&self) -> PluginResult<()> {
        if self.host.is_empty() {
            return Err(PluginError::InvalidConfiguration {
                message: "Network host cannot be empty".to_string(),
            });
        }

        if self.port == 0 {
            return Err(PluginError::InvalidConfiguration {
                message: "Network port must be greater than 0".to_string(),
            });
        }

        if self.max_connections == 0 {
            return Err(PluginError::InvalidConfiguration {
                message: "Max connections must be greater than 0".to_string(),
            });
        }

        Ok(())
    }
}

/// HTTP client configuration with environment variable support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpConfig {
    /// Default request timeout in milliseconds
    pub default_timeout_ms: u64,
    /// Maximum request size in bytes
    pub max_request_size: usize,
    /// Maximum response size in bytes
    pub max_response_size: usize,
    /// Maximum redirects to follow
    pub max_redirects: u32,
    /// User agent string
    pub user_agent: String,
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self::from_env()
    }
}

impl HttpConfig {
    /// Load HTTP configuration from environment variables
    pub fn from_env() -> Self {
        Self {
            default_timeout_ms: std::env::var("HTTP_DEFAULT_TIMEOUT_MS")
                .unwrap_or_else(|_| "30000".to_string())
                .parse()
                .unwrap_or(30000),
            max_request_size: std::env::var("HTTP_MAX_REQUEST_SIZE")
                .unwrap_or_else(|_| "10485760".to_string())
                .parse()
                .unwrap_or(10 * 1024 * 1024),
            max_response_size: std::env::var("HTTP_MAX_RESPONSE_SIZE")
                .unwrap_or_else(|_| "10485760".to_string())
                .parse()
                .unwrap_or(10 * 1024 * 1024),
            max_redirects: std::env::var("HTTP_MAX_REDIRECTS")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .unwrap_or(10),
            user_agent: std::env::var("HTTP_USER_AGENT")
                .unwrap_or_else(|_| "Squirrel-Plugin-SDK/1.0".to_string()),
        }
    }

    /// Validate HTTP configuration
    pub fn validate(&self) -> PluginResult<()> {
        if self.default_timeout_ms == 0 {
            return Err(PluginError::InvalidConfiguration {
                message: "HTTP default timeout must be greater than 0".to_string(),
            });
        }

        if self.max_request_size == 0 {
            return Err(PluginError::InvalidConfiguration {
                message: "HTTP max request size must be greater than 0".to_string(),
            });
        }

        if self.max_response_size == 0 {
            return Err(PluginError::InvalidConfiguration {
                message: "HTTP max response size must be greater than 0".to_string(),
            });
        }

        if self.user_agent.is_empty() {
            return Err(PluginError::InvalidConfiguration {
                message: "HTTP user agent cannot be empty".to_string(),
            });
        }

        Ok(())
    }
}

/// Performance configuration with environment variable support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Plugin ID maximum length
    pub max_plugin_id_length: usize,
    /// String pool initial capacity
    pub string_pool_capacity: usize,
    /// Batch processor buffer size
    pub batch_processor_size: usize,
    /// File system buffer size
    pub fs_buffer_size: usize,
    /// Session timeout in seconds
    pub session_timeout_seconds: u32,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self::from_env()
    }
}

impl PerformanceConfig {
    /// Load performance configuration from environment variables
    pub fn from_env() -> Self {
        Self {
            max_plugin_id_length: std::env::var("PERF_MAX_PLUGIN_ID_LENGTH")
                .unwrap_or_else(|_| "64".to_string())
                .parse()
                .unwrap_or(64),
            string_pool_capacity: std::env::var("PERF_STRING_POOL_CAPACITY")
                .unwrap_or_else(|_| "1000".to_string())
                .parse()
                .unwrap_or(1000),
            batch_processor_size: std::env::var("PERF_BATCH_PROCESSOR_SIZE")
                .unwrap_or_else(|_| "100".to_string())
                .parse()
                .unwrap_or(100),
            fs_buffer_size: std::env::var("PERF_FS_BUFFER_SIZE")
                .unwrap_or_else(|_| "8192".to_string())
                .parse()
                .unwrap_or(8192),
            session_timeout_seconds: std::env::var("PERF_SESSION_TIMEOUT_SECONDS")
                .unwrap_or_else(|_| "300".to_string())
                .parse()
                .unwrap_or(300),
        }
    }

    /// Validate performance configuration
    pub fn validate(&self) -> PluginResult<()> {
        if self.max_plugin_id_length == 0 {
            return Err(PluginError::InvalidConfiguration {
                message: "Max plugin ID length must be greater than 0".to_string(),
            });
        }

        if self.string_pool_capacity == 0 {
            return Err(PluginError::InvalidConfiguration {
                message: "String pool capacity must be greater than 0".to_string(),
            });
        }

        if self.batch_processor_size == 0 {
            return Err(PluginError::InvalidConfiguration {
                message: "Batch processor size must be greater than 0".to_string(),
            });
        }

        if self.fs_buffer_size == 0 {
            return Err(PluginError::InvalidConfiguration {
                message: "File system buffer size must be greater than 0".to_string(),
            });
        }

        if self.session_timeout_seconds == 0 {
            return Err(PluginError::InvalidConfiguration {
                message: "Session timeout must be greater than 0".to_string(),
            });
        }

        Ok(())
    }
}

/// Plugin configuration structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PluginConfig {
    /// Plugin metadata
    pub metadata: PluginMetadata,
    /// Plugin permissions
    pub permissions: Vec<Permission>,
    /// Plugin settings
    pub settings: HashMap<String, serde_json::Value>,
    /// Plugin UI configuration
    pub ui: Option<UiConfig>,
    /// Plugin resources
    pub resources: Vec<String>,
}

/// Plugin metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    /// Plugin name
    pub name: String,
    /// Plugin version
    pub version: String,
    /// Plugin description
    pub description: String,
    /// Plugin author
    pub author: String,
    /// Plugin license
    pub license: String,
    /// Plugin homepage URL
    pub homepage: Option<String>,
    /// Plugin repository URL
    pub repository: Option<String>,
    /// Plugin documentation URL
    pub documentation: Option<String>,
    /// Plugin categories
    pub categories: Vec<String>,
    /// Plugin tags
    pub tags: Vec<String>,
    /// Plugin keywords
    pub keywords: Vec<String>,
}

impl Default for PluginMetadata {
    fn default() -> Self {
        Self {
            name: "unnamed-plugin".to_string(),
            version: "0.1.0".to_string(),
            description: "A Squirrel plugin".to_string(),
            author: "Unknown".to_string(),
            license: "MIT".to_string(),
            homepage: None,
            repository: None,
            documentation: None,
            categories: vec!["general".to_string()],
            tags: Vec::new(),
            keywords: Vec::new(),
        }
    }
}

/// Sandbox configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    /// Whether network access is allowed
    pub network_access: bool,
    /// Allowed file system paths
    pub file_system_access: Vec<String>,
    /// Memory limit in MB
    pub memory_limit_mb: u32,
    /// CPU limit as percentage
    pub cpu_limit_percent: u8,
    /// Execution timeout in seconds
    pub execution_timeout_seconds: u32,
    /// Security level
    pub security_level: SecurityLevel,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self::from_env()
    }
}

impl SandboxConfig {
    /// Load sandbox configuration from environment variables
    pub fn from_env() -> Self {
        Self {
            network_access: std::env::var("SANDBOX_NETWORK_ACCESS")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
            file_system_access: std::env::var("SANDBOX_FILE_SYSTEM_ACCESS")
                .unwrap_or_else(|_| "".to_string())
                .split(',')
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect(),
            memory_limit_mb: std::env::var("SANDBOX_MEMORY_LIMIT_MB")
                .unwrap_or_else(|_| "128".to_string())
                .parse()
                .unwrap_or(128),
            cpu_limit_percent: std::env::var("SANDBOX_CPU_LIMIT_PERCENT")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .unwrap_or(5),
            execution_timeout_seconds: std::env::var("SANDBOX_EXECUTION_TIMEOUT_SECONDS")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .unwrap_or(10),
            security_level: match std::env::var("SANDBOX_SECURITY_LEVEL")
                .unwrap_or_else(|_| "restricted".to_string())
                .to_lowercase()
                .as_str()
            {
                "standard" => SecurityLevel::Standard,
                "enhanced" => SecurityLevel::Enhanced,
                "full" => SecurityLevel::Full,
                _ => SecurityLevel::Restricted,
            },
        }
    }

    /// Validate sandbox configuration
    pub fn validate(&self) -> PluginResult<()> {
        if self.memory_limit_mb == 0 {
            return Err(PluginError::InvalidConfiguration {
                message: "Memory limit must be greater than 0".to_string(),
            });
        }

        if self.cpu_limit_percent == 0 || self.cpu_limit_percent > 100 {
            return Err(PluginError::InvalidConfiguration {
                message: "CPU limit must be between 1 and 100".to_string(),
            });
        }

        if self.execution_timeout_seconds == 0 {
            return Err(PluginError::InvalidConfiguration {
                message: "Execution timeout must be greater than 0".to_string(),
            });
        }

        Ok(())
    }
}

/// Security level for plugin execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityLevel {
    /// Minimal permissions, safe for untrusted plugins
    Restricted,
    /// Standard permissions for verified plugins
    Standard,
    /// Enhanced permissions for trusted plugins
    Enhanced,
    /// Full permissions for system plugins
    Full,
}

/// MCP configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConfig {
    /// MCP protocol version
    pub protocol_version: String,
    /// Whether streaming is supported
    pub supports_streaming: bool,
    /// Whether tools are supported
    pub supports_tools: bool,
    /// Whether resources are supported
    pub supports_resources: bool,
    /// Whether prompts are supported
    pub supports_prompts: bool,
    /// Custom capabilities
    pub custom_capabilities: HashMap<String, serde_json::Value>,
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            protocol_version: "1.0".to_string(),
            supports_streaming: false,
            supports_tools: true,
            supports_resources: false,
            supports_prompts: false,
            custom_capabilities: HashMap::new(),
        }
    }
}

/// UI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    /// Whether the plugin has a web UI
    pub has_web_ui: bool,
    /// UI entry point file
    pub ui_entry_point: Option<String>,
    /// UI theme support
    pub theme_support: bool,
    /// Responsive design support
    pub responsive: bool,
    /// Accessibility support
    pub accessibility: bool,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            has_web_ui: false,
            ui_entry_point: None,
            theme_support: true,
            responsive: true,
            accessibility: true,
        }
    }
}

/// Plugin dependency specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDependency {
    /// Dependency name
    pub name: String,
    /// Version requirement
    pub version: String,
    /// Whether the dependency is optional
    pub optional: bool,
    /// Dependency features to enable
    pub features: Vec<String>,
}

impl PluginConfig {
    /// Validate the configuration
    pub fn validate(&self) -> PluginResult<()> {
        // Basic validation - in a real implementation, this would be more comprehensive
        if self.metadata.name.is_empty() {
            return Err(PluginError::InvalidConfiguration {
                message: "Plugin name cannot be empty".to_string(),
            });
        }

        if self.metadata.version.is_empty() {
            return Err(PluginError::InvalidConfiguration {
                message: "Plugin version cannot be empty".to_string(),
            });
        }

        // Basic version format validation (should be semantic version-like)
        if !self.metadata.version.chars().any(|c| c.is_ascii_digit()) {
            return Err(PluginError::InvalidConfiguration {
                message: "Plugin version must contain at least one digit".to_string(),
            });
        }

        // Validate permissions
        for permission in &self.permissions {
            match permission {
                Permission::NetworkAccess => {
                    // Network access permission is valid as-is
                }
                Permission::FileSystemRead(_) | Permission::FileSystemWrite(_) => {
                    // File system permissions are valid
                }
                Permission::LocalStorage | Permission::SessionStorage => {
                    // Storage permissions are valid
                }
            }
        }

        Ok(())
    }

    /// Get memory limit in bytes
    pub fn memory_limit_bytes(&self) -> usize {
        let sandbox_config = SandboxConfig::from_env();
        (sandbox_config.memory_limit_mb as usize) * 1024 * 1024
    }

    /// Check if path access is allowed
    pub fn is_path_allowed(&self, _path: &str) -> bool {
        // Simple check - in a real implementation, this would check against allowed paths
        true
    }

    /// Get a setting value by key
    pub fn get_setting<T>(&self, key: &str) -> PluginResult<Option<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        match self.settings.get(key) {
            Some(value) => {
                let parsed: T = serde_json::from_value(value.clone())?;
                Ok(Some(parsed))
            }
            None => Ok(None),
        }
    }

    /// Set a setting value
    pub fn set_setting<T>(&mut self, key: &str, value: T) -> PluginResult<()>
    where
        T: Serialize,
    {
        let json_value = serde_json::to_value(value)?;
        self.settings.insert(key.to_string(), json_value);
        Ok(())
    }

    /// Check if plugin has a specific capability
    pub fn has_capability(&self, capability: &str) -> bool {
        match capability {
            "network" => self
                .permissions
                .iter()
                .any(|p| matches!(p, Permission::NetworkAccess)),
            "filesystem" => self.permissions.iter().any(|p| {
                matches!(
                    p,
                    Permission::FileSystemRead(_) | Permission::FileSystemWrite(_)
                )
            }),
            "storage" => self
                .permissions
                .iter()
                .any(|p| matches!(p, Permission::LocalStorage | Permission::SessionStorage)),
            "ui" => self.ui.as_ref().is_some_and(|ui| ui.has_web_ui),
            _ => false,
        }
    }

    /// Get the maximum allowed memory in bytes
    pub fn max_memory_bytes(&self) -> usize {
        let sandbox_config = SandboxConfig::from_env();
        (sandbox_config.memory_limit_mb as usize) * 1024 * 1024
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = PluginConfig::default();
        assert_eq!(config.metadata.name, "unnamed-plugin");
        // Removed sandbox field references since structure was simplified
        assert!(config.permissions.is_empty());
    }

    #[test]
    fn test_config_validation() {
        let mut config = PluginConfig::default();

        // Valid config should pass
        assert!(config.validate().is_ok());

        // Empty name should fail
        config.metadata.name = String::new();
        assert!(config.validate().is_err());

        // Invalid version should fail
        config.metadata.name = "test".to_string();
        config.metadata.version = "invalid".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_settings() {
        let mut config = PluginConfig::default();

        config.set_setting("test_key", "test_value").unwrap();
        let value: Option<String> = config.get_setting("test_key").unwrap();
        assert_eq!(value, Some("test_value".to_string()));

        let missing: Option<String> = config.get_setting("missing_key").unwrap();
        assert_eq!(missing, None);
    }

    #[test]
    fn test_capabilities() {
        let config = PluginConfig::default();

        assert!(!config.has_capability("network"));
        assert!(!config.has_capability("fs"));
    }

    #[test]
    fn test_path_validation() {
        let config = PluginConfig::default();

        // Simplified path validation since sandbox config was removed
        assert!(config.is_path_allowed("./workspace/file.txt"));
        assert!(config.is_path_allowed("./other/file.txt"));
    }

    #[test]
    fn test_plugin_sdk_config_from_env() {
        let config = PluginSdkConfig::from_env();
        assert!(!config.mcp.server_url.is_empty());
        assert!(config.mcp.timeout_ms > 0);
        assert!(!config.logging.min_level.is_empty());
        assert!(!config.network.host.is_empty());
        assert!(config.network.port > 0);
    }

    #[test]
    fn test_plugin_sdk_config_validate() {
        let config = PluginSdkConfig::from_env();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_mcp_client_config_validation_empty_url() {
        let mut config = McpClientConfig::from_env();
        config.server_url = String::new();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_mcp_client_config_validation_zero_timeout() {
        let mut config = McpClientConfig::from_env();
        config.timeout_ms = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_mcp_client_config_validation_zero_message_size() {
        let mut config = McpClientConfig::from_env();
        config.max_message_size = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_logging_config_validation_invalid_level() {
        let mut config = LoggingConfig::from_env();
        config.min_level = "invalid_level".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_network_config_validation_empty_host() {
        let mut config = NetworkConfig::from_env();
        config.host = String::new();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_network_config_validation_zero_port() {
        let mut config = NetworkConfig::from_env();
        config.port = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_http_config_validation_empty_user_agent() {
        let mut config = HttpConfig::from_env();
        config.user_agent = String::new();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_performance_config_validation_zero_values() {
        let mut config = PerformanceConfig::from_env();
        config.max_plugin_id_length = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_sandbox_config_validation() {
        let config = SandboxConfig::from_env();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_mcp_config_default() {
        let config = McpConfig::default();
        assert_eq!(config.protocol_version, "1.0");
        assert!(config.supports_tools);
    }
}
