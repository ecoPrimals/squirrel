//! Client configuration for MCP connections
//!
//! This module provides configuration options for MCP clients including connection settings,
//! timeouts, authentication, and transport configuration.
//!
//! ## Environment Variables (Unified Config)
//! 
//! Timeouts now use the unified configuration system with SQUIRREL_* prefix:
//! - `SQUIRREL_CONNECTION_TIMEOUT_SECS`: Connection timeout (default: 30)
//! - `SQUIRREL_REQUEST_TIMEOUT_SECS`: Request timeout (default: 60)
//! - `SQUIRREL_HEARTBEAT_INTERVAL_SECS`: Keep-alive interval (default: 30)
//! - `SQUIRREL_CUSTOM_TIMEOUT_MCP_RECONNECT_SECS`: Reconnection delay (default: 1)
//!
//! Legacy MCP_* environment variables:
//! - `MCP_SERVER_ADDRESS`: Server address (default: "127.0.0.1:8080")
//! - `MCP_CLIENT_ID`: Client ID (auto-generated if not provided)
//! - `MCP_AUTH_TOKEN`: Authentication token

use crate::error::Result;
use crate::transport::Transport;
use crate::protocol::WireFormatConfig;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

// Import unified config for timeout management
use squirrel_mcp_config::unified::ConfigLoader;

// Import configuration if available
#[cfg(feature = "config")]
use squirrel_mcp_config::unified::SquirrelUnifiedConfig;  // Migrated from deprecated Config type (ADR-008)

/// Client configuration for MCP connections
///
/// This configuration can be created from environment variables or a configuration file.
/// 
/// ## Example
///
/// ```rust
/// use squirrel_mcp::client::ClientConfig;
/// 
/// // Create from environment
/// let config = ClientConfig::from_env();
/// 
/// // Create with custom values
/// let config = ClientConfig::new()
///     .with_server_address("localhost:9000")
///     .with_connection_timeout(Duration::from_secs(10))
///     .with_request_timeout(Duration::from_secs(60));
/// ```
#[derive(Clone)]
pub struct ClientConfig {
    /// Server address to connect to
    pub server_address: String,
    
    /// Connection timeout
    pub connection_timeout: Duration,
    
    /// Request timeout
    pub request_timeout: Duration,
    
    /// Maximum number of reconnect attempts
    pub max_reconnect_attempts: u32,
    
    /// Delay between reconnect attempts
    pub reconnect_delay: Duration,
    
    /// Keep-alive interval
    pub keep_alive_interval: Option<Duration>,
    
    /// Client ID (generated automatically if not provided)
    pub client_id: Option<String>,
    
    /// Authentication token
    pub auth_token: Option<String>,
    
    /// Custom transport implementation to use
    pub transport: Option<Arc<dyn Transport>>,
    
    /// Wire format adapter configuration
    pub wire_format_config: Option<WireFormatConfig>,
    
    /// Additional client parameters
    pub parameters: HashMap<String, Value>,
}

impl ClientConfig {
    /// Create a new client configuration with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Create configuration from environment variables using unified config
    ///
    /// Timeouts are now loaded from the unified configuration system:
    /// - Connection timeout: SQUIRREL_CONNECTION_TIMEOUT_SECS
    /// - Request timeout: SQUIRREL_REQUEST_TIMEOUT_SECS
    /// - Keep-alive interval: SQUIRREL_HEARTBEAT_INTERVAL_SECS
    /// - Reconnect delay: SQUIRREL_CUSTOM_TIMEOUT_MCP_RECONNECT_SECS
    pub fn from_env() -> Self {
        // Load unified config for timeouts
        let unified_config = ConfigLoader::load()
            .map(|c| c.into_config())
            .ok();
        
        let connection_timeout = unified_config.as_ref()
            .map(|c| c.timeouts.connection_timeout())
            .unwrap_or(Duration::from_secs(5));
        
        let request_timeout = unified_config.as_ref()
            .map(|c| c.timeouts.request_timeout())
            .unwrap_or(Duration::from_secs(30));
        
        let keep_alive_interval = unified_config.as_ref()
            .map(|c| Some(c.timeouts.heartbeat_interval()))
            .unwrap_or(Some(Duration::from_secs(30)));
        
        let reconnect_delay = unified_config.as_ref()
            .map(|c| c.timeouts.get_custom_timeout("mcp_reconnect"))
            .unwrap_or(Duration::from_secs(1));
        
        Self {
            server_address: std::env::var("MCP_SERVER_ADDRESS")
                .unwrap_or_else(|_| "127.0.0.1:8080".to_string()),
            connection_timeout,
            request_timeout,
            max_reconnect_attempts: std::env::var("MCP_MAX_RECONNECT_ATTEMPTS")
                .unwrap_or_else(|_| "3".to_string())
                .parse()
                .unwrap_or(3),
            reconnect_delay,
            keep_alive_interval,
            client_id: std::env::var("MCP_CLIENT_ID").ok(),
            auth_token: std::env::var("MCP_AUTH_TOKEN").ok(),
            transport: None,
            wire_format_config: None,
            parameters: HashMap::new(),
        }
    }

    /// Create configuration from global config
    #[cfg(feature = "config")]
    pub fn from_global_config(config: &Config) -> Self {
        let mut client_config = Self::default();
        
        // Override with values from global config
        if let Some(server_addr) = config.get::<String>("server_address") {
            client_config.server_address = server_addr;
        }
        if let Some(timeout) = config.get::<u64>("connection_timeout_ms") {
            client_config.connection_timeout = Duration::from_millis(timeout);
        }
        if let Some(timeout) = config.get::<u64>("request_timeout_ms") {
            client_config.request_timeout = Duration::from_millis(timeout);
        }
        
        client_config
    }

    /// Set server address
    pub fn with_server_address(mut self, address: impl Into<String>) -> Self {
        self.server_address = address.into();
        self
    }

    /// Set connection timeout
    pub fn with_connection_timeout(mut self, timeout: Duration) -> Self {
        self.connection_timeout = timeout;
        self
    }

    /// Set request timeout
    pub fn with_request_timeout(mut self, timeout: Duration) -> Self {
        self.request_timeout = timeout;
        self
    }

    /// Set maximum reconnect attempts
    pub fn with_max_reconnect_attempts(mut self, attempts: u32) -> Self {
        self.max_reconnect_attempts = attempts;
        self
    }

    /// Set reconnect delay
    pub fn with_reconnect_delay(mut self, delay: Duration) -> Self {
        self.reconnect_delay = delay;
        self
    }

    /// Set keep-alive interval
    pub fn with_keep_alive_interval(mut self, interval: Option<Duration>) -> Self {
        self.keep_alive_interval = interval;
        self
    }

    /// Set client ID
    pub fn with_client_id(mut self, client_id: impl Into<String>) -> Self {
        self.client_id = Some(client_id.into());
        self
    }

    /// Set authentication token
    pub fn with_auth_token(mut self, token: impl Into<String>) -> Self {
        self.auth_token = Some(token.into());
        self
    }

    /// Set custom transport
    pub fn with_transport(mut self, transport: Arc<dyn Transport>) -> Self {
        self.transport = Some(transport);
        self
    }

    /// Set wire format configuration
    pub fn with_wire_format_config(mut self, config: WireFormatConfig) -> Self {
        self.wire_format_config = Some(config);
        self
    }

    /// Add a parameter
    pub fn with_parameter(mut self, key: impl Into<String>, value: Value) -> Self {
        self.parameters.insert(key.into(), value);
        self
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        if self.server_address.is_empty() {
            return Err(crate::error::ClientError::InvalidConfiguration(
                "Server address cannot be empty".to_string(),
            ).into());
        }
        
        if self.connection_timeout.is_zero() {
            return Err(crate::error::ClientError::InvalidConfiguration(
                "Connection timeout must be greater than zero".to_string(),
            ).into());
        }
        
        if self.request_timeout.is_zero() {
            return Err(crate::error::ClientError::InvalidConfiguration(
                "Request timeout must be greater than zero".to_string(),
            ).into());
        }
        
        Ok(())
    }

    /// Get connection timeout in milliseconds
    pub fn connection_timeout_ms(&self) -> u64 {
        self.connection_timeout.as_millis() as u64
    }

    /// Get request timeout in milliseconds
    pub fn request_timeout_ms(&self) -> u64 {
        self.request_timeout.as_millis() as u64
    }

    /// Get reconnect delay in milliseconds
    pub fn reconnect_delay_ms(&self) -> u64 {
        self.reconnect_delay.as_millis() as u64
    }

    /// Get keep-alive interval in milliseconds
    pub fn keep_alive_interval_ms(&self) -> Option<u64> {
        self.keep_alive_interval.map(|d| d.as_millis() as u64)
    }
}

impl Default for ClientConfig {
    fn default() -> Self {
        // Load unified config for environment-aware timeouts
        let unified_config = ConfigLoader::load()
            .map(|c| c.into_config())
            .ok();
        
        let connection_timeout = unified_config.as_ref()
            .map(|c| c.timeouts.connection_timeout())
            .unwrap_or(Duration::from_secs(5));
        
        let request_timeout = unified_config.as_ref()
            .map(|c| c.timeouts.request_timeout())
            .unwrap_or(Duration::from_secs(30));
        
        let keep_alive_interval = unified_config.as_ref()
            .map(|c| Some(c.timeouts.heartbeat_interval()))
            .unwrap_or(Some(Duration::from_secs(30)));
        
        let reconnect_delay = unified_config.as_ref()
            .map(|c| c.timeouts.get_custom_timeout("mcp_reconnect"))
            .unwrap_or(Duration::from_secs(1));
        
        Self {
            server_address: "127.0.0.1:8080".to_string(),
            connection_timeout,
            request_timeout,
            max_reconnect_attempts: 3,
            reconnect_delay,
            keep_alive_interval,
            client_id: None,
            auth_token: None,
            transport: None,
            wire_format_config: None,
            parameters: HashMap::new(),
        }
    }
}

impl std::fmt::Debug for ClientConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClientConfig")
            .field("server_address", &self.server_address)
            .field("connection_timeout", &self.connection_timeout)
            .field("request_timeout", &self.request_timeout)
            .field("max_reconnect_attempts", &self.max_reconnect_attempts)
            .field("reconnect_delay", &self.reconnect_delay)
            .field("keep_alive_interval", &self.keep_alive_interval)
            .field("client_id", &self.client_id)
            .field("auth_token", &"[REDACTED]")
            .field("transport", &self.transport.is_some())
            .field("wire_format_config", &self.wire_format_config.is_some())
            .field("parameters", &self.parameters)
            .finish()
    }
} 