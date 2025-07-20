//! Configuration module for MCP server and client
//!
//! This module provides configurable settings to replace hardcoded values
//! and improve maintainability of the MCP implementation.

use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr};
use std::time::Duration;

/// Default MCP server configuration values
pub const DEFAULT_MCP_PORT: u16 = 8778;
pub const DEFAULT_DEV_HOST: &str = "127.0.0.1";
pub const DEFAULT_PROD_HOST: &str = "0.0.0.0";
pub const DEFAULT_CLIENT_TIMEOUT_SECS: u64 = 30;
pub const DEFAULT_CONNECT_TIMEOUT_SECS: u64 = 5;
pub const DEFAULT_HEARTBEAT_INTERVAL_SECS: u64 = 30;
pub const DEFAULT_MAX_CONNECTIONS: usize = 100;
pub const DEFAULT_BUFFER_SIZE: usize = 8192;

/// MCP Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPServerConfig {
    /// Server bind address
    pub host: String,
    /// Server port
    pub port: u16,
    /// Maximum concurrent connections
    pub max_connections: usize,
    /// Connection timeout
    pub connection_timeout: Duration,
    /// Heartbeat interval for client health checks
    pub heartbeat_interval: Duration,
    /// Buffer size for message processing
    pub buffer_size: usize,
    /// Enable debug logging
    pub debug_logging: bool,
    /// Environment (development/production)
    pub environment: String,
}

impl Default for MCPServerConfig {
    fn default() -> Self {
        Self {
            host: Self::default_host_for_env(),
            port: DEFAULT_MCP_PORT,
            max_connections: DEFAULT_MAX_CONNECTIONS,
            connection_timeout: Duration::from_secs(DEFAULT_CONNECT_TIMEOUT_SECS),
            heartbeat_interval: Duration::from_secs(DEFAULT_HEARTBEAT_INTERVAL_SECS),
            buffer_size: DEFAULT_BUFFER_SIZE,
            debug_logging: false,
            environment: "development".to_string(),
        }
    }
}

/// MCP Client configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPClientConfig {
    /// Server host to connect to
    pub host: String,
    /// Server port to connect to
    pub port: u16,
    /// Connection timeout
    pub connect_timeout: Duration,
    /// Request timeout
    pub request_timeout: Duration,
    /// Number of connection retry attempts
    pub max_retries: u32,
    /// Delay between retry attempts
    pub retry_delay: Duration,
    /// Enable keep-alive
    pub keep_alive: bool,
}

impl Default for MCPClientConfig {
    fn default() -> Self {
        Self {
            host: DEFAULT_DEV_HOST.to_string(),
            port: DEFAULT_MCP_PORT,
            connect_timeout: Duration::from_secs(DEFAULT_CONNECT_TIMEOUT_SECS),
            request_timeout: Duration::from_secs(DEFAULT_CLIENT_TIMEOUT_SECS),
            max_retries: 3,
            retry_delay: Duration::from_millis(1000),
            keep_alive: true,
        }
    }
}

impl MCPServerConfig {
    /// Create configuration from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::default();

        // Host configuration
        if let Ok(host) = std::env::var("MCP_HOST") {
            config.host = host;
        }

        // Port configuration
        if let Ok(port) = std::env::var("MCP_PORT") {
            if let Ok(port_num) = port.parse::<u16>() {
                config.port = port_num;
            }
        }

        // Environment
        if let Ok(env) = std::env::var("ENVIRONMENT") {
            config.environment = env;
            // Auto-adjust host for production
            if config.environment.eq_ignore_ascii_case("production")
                && config.host == DEFAULT_DEV_HOST
            {
                config.host = DEFAULT_PROD_HOST.to_string();
            }
        }

        // Max connections
        if let Ok(max_conn) = std::env::var("MCP_MAX_CONNECTIONS") {
            if let Ok(max_connections) = max_conn.parse::<usize>() {
                config.max_connections = max_connections;
            }
        }

        // Connection timeout
        if let Ok(timeout) = std::env::var("MCP_CONNECTION_TIMEOUT_SECS") {
            if let Ok(secs) = timeout.parse::<u64>() {
                config.connection_timeout = Duration::from_secs(secs);
            }
        }

        // Heartbeat interval
        if let Ok(interval) = std::env::var("MCP_HEARTBEAT_INTERVAL_SECS") {
            if let Ok(secs) = interval.parse::<u64>() {
                config.heartbeat_interval = Duration::from_secs(secs);
            }
        }

        // Debug logging
        if let Ok(debug) = std::env::var("MCP_DEBUG") {
            config.debug_logging = debug.eq_ignore_ascii_case("true") || debug == "1";
        }

        config
    }

    /// Get appropriate default host for environment
    fn default_host_for_env() -> String {
        let is_production = std::env::var("ENVIRONMENT")
            .unwrap_or_else(|_| "development".to_string())
            .eq_ignore_ascii_case("production");

        if is_production {
            std::env::var("MCP_HOST").unwrap_or_else(|_| DEFAULT_PROD_HOST.to_string())
        } else {
            std::env::var("MCP_HOST").unwrap_or_else(|_| DEFAULT_DEV_HOST.to_string())
        }
    }

    /// Validate configuration values
    pub fn validate(&self) -> Result<(), String> {
        if self.port == 0 {
            return Err("Port cannot be zero".to_string());
        }

        if self.max_connections == 0 {
            return Err("Max connections cannot be zero".to_string());
        }

        if self.connection_timeout.as_secs() == 0 {
            return Err("Connection timeout cannot be zero".to_string());
        }

        // Validate host is a valid IP address or hostname
        if let Ok(addr) = self.host.parse::<IpAddr>() {
            if addr.is_unspecified() && !self.environment.eq_ignore_ascii_case("production") {
                return Err("Binding to 0.0.0.0 is only allowed in production".to_string());
            }
        } else if self.host.is_empty() {
            return Err("Host cannot be empty".to_string());
        }

        Ok(())
    }

    /// Get the full server address string
    pub fn address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

impl MCPClientConfig {
    /// Create configuration from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::default();

        // Host configuration
        if let Ok(host) = std::env::var("MCP_CLIENT_HOST") {
            config.host = host;
        } else if let Ok(host) = std::env::var("MCP_HOST") {
            config.host = host;
        }

        // Port configuration
        if let Ok(port) = std::env::var("MCP_CLIENT_PORT") {
            if let Ok(port_num) = port.parse::<u16>() {
                config.port = port_num;
            }
        } else if let Ok(port) = std::env::var("MCP_PORT") {
            if let Ok(port_num) = port.parse::<u16>() {
                config.port = port_num;
            }
        }

        // Timeouts
        if let Ok(timeout) = std::env::var("MCP_CLIENT_CONNECT_TIMEOUT_SECS") {
            if let Ok(secs) = timeout.parse::<u64>() {
                config.connect_timeout = Duration::from_secs(secs);
            }
        }

        if let Ok(timeout) = std::env::var("MCP_CLIENT_REQUEST_TIMEOUT_SECS") {
            if let Ok(secs) = timeout.parse::<u64>() {
                config.request_timeout = Duration::from_secs(secs);
            }
        }

        // Retries
        if let Ok(retries) = std::env::var("MCP_CLIENT_MAX_RETRIES") {
            if let Ok(max_retries) = retries.parse::<u32>() {
                config.max_retries = max_retries;
            }
        }

        config
    }

    /// Validate configuration values
    pub fn validate(&self) -> Result<(), String> {
        if self.port == 0 {
            return Err("Port cannot be zero".to_string());
        }

        if self.connect_timeout.as_secs() == 0 {
            return Err("Connect timeout cannot be zero".to_string());
        }

        if self.request_timeout.as_secs() == 0 {
            return Err("Request timeout cannot be zero".to_string());
        }

        if self.max_retries > 10 {
            return Err("Max retries cannot exceed 10".to_string());
        }

        if self.host.is_empty() {
            return Err("Host cannot be empty".to_string());
        }

        Ok(())
    }

    /// Get the full server address string
    pub fn address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_server_config() {
        let config = MCPServerConfig::default();
        assert_eq!(config.port, DEFAULT_MCP_PORT);
        assert_eq!(config.max_connections, DEFAULT_MAX_CONNECTIONS);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_default_client_config() {
        let config = MCPClientConfig::default();
        assert_eq!(config.port, DEFAULT_MCP_PORT);
        assert_eq!(config.host, DEFAULT_DEV_HOST);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validation() {
        let mut config = MCPServerConfig::default();
        assert!(config.validate().is_ok());

        config.port = 0;
        assert!(config.validate().is_err());

        config.port = DEFAULT_MCP_PORT;
        config.max_connections = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_address_formatting() {
        let config = MCPServerConfig::default();
        assert_eq!(
            config.address(),
            format!("{}:{}", DEFAULT_DEV_HOST, DEFAULT_MCP_PORT)
        );
    }
}
