// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Configuration management for plugins
//!
//! This module provides configuration loading and management capabilities for WASM plugins.
//! Configuration is validated and access is controlled through sandbox permissions.

use crate::error::{PluginError, PluginResult};
use serde::{Deserialize, Serialize};
#[cfg(target_arch = "wasm32")]
use universal_constants::network::get_bind_address;
use universal_constants::network::get_service_port;
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
    /// Server URL resolution:
    /// 1. `MCP_SERVER_URL` if set
    /// 2. Otherwise: native targets default to `unix://` + Songbird-style socket path
    ///    ([`universal_constants::network::resolve_capability_unix_socket`]); WASM defaults to
    ///    `ws://{bind_address}:{get_service_port("websocket")}` for browser `WebSocket`.
    pub fn from_env() -> Self {
        let server_url = std::env::var("MCP_SERVER_URL").unwrap_or_else(|_| {
            #[cfg(target_arch = "wasm32")]
            {
                let port = std::env::var("MCP_SERVER_PORT")
                    .ok()
                    .and_then(|p| p.parse::<u16>().ok())
                    .unwrap_or_else(|| get_service_port("websocket"));
                let host = get_bind_address();
                format!("ws://{host}:{port}")
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                let path = universal_constants::network::resolve_capability_unix_socket(
                    "MCP_SERVER_SOCKET",
                    "squirrel-mcp",
                );
                format!("unix://{}", path.display())
            }
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

        #[cfg(not(target_arch = "wasm32"))]
        {
            let u = self.server_url.trim();
            if u.starts_with("ws://") || u.starts_with("wss://") {
                return Err(PluginError::InvalidConfiguration {
                    message: "MCP_SERVER_URL cannot use ws:// or wss:// on native targets; use unix://… IPC (Tower Atomic / Songbird).".to_string(),
                });
            }
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
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or_else(|| get_service_port("websocket")),
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

// Plugin manifest and sandbox types live in their own module for modularity.
// Re-exported here to maintain backward compatibility.
pub use super::plugin_config::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::Permission;

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

        config
            .set_setting("test_key", "test_value")
            .expect("should succeed");
        let value: Option<String> = config.get_setting("test_key").expect("should succeed");
        assert_eq!(value, Some("test_value".to_string()));

        let missing: Option<String> = config.get_setting("missing_key").expect("should succeed");
        assert_eq!(missing, None);
    }

    #[test]
    fn test_get_setting_wrong_type_returns_error() {
        let mut config = PluginConfig::default();
        config
            .settings
            .insert("n".to_string(), serde_json::json!(42));
        let err: Result<Option<String>, _> = config.get_setting("n");
        assert!(err.is_err());
    }

    #[test]
    fn test_capabilities() {
        let mut config = PluginConfig::default();
        config.permissions.push(Permission::NetworkAccess);
        assert!(config.has_capability("network"));
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
    #[cfg(not(target_arch = "wasm32"))]
    fn test_mcp_client_config_validation_rejects_ws_url() {
        let mut config = McpClientConfig::from_env();
        config.server_url = "ws://127.0.0.1:8080".to_string();
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
