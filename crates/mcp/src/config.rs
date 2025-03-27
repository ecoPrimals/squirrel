//! MCP configuration types and utilities.
//!
//! This module provides configuration structures for the Machine Context Protocol (MCP)
//! system, allowing customization of connection parameters, performance settings,
//! and operational limits.

use serde::{Deserialize, Serialize};

/// Configuration for MCP server and client operations.
///
/// This structure contains all configurable parameters for MCP operations,
/// including network settings, connection limits, and performance tuning options.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConfig {
    /// Host address to bind to or connect to
    pub host: String,
    /// Port number for MCP communications
    pub port: u16,
    /// Maximum number of concurrent connections allowed
    pub max_connections: usize,
    /// Connection timeout in seconds
    pub timeout: u64,
    /// Size of internal communication buffers in bytes
    pub buffer_size: usize,
}

impl Default for McpConfig {
    /// Creates a default configuration with reasonable values.
    ///
    /// # Returns
    ///
    /// A new `McpConfig` instance with default settings:
    /// - Host: 127.0.0.1
    /// - Port: 8080
    /// - Max connections: 100
    /// - Timeout: 30 seconds
    /// - Buffer size: 8192 bytes
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            max_connections: 100,
            timeout: 30,
            buffer_size: 8192,
        }
    }
}

impl McpConfig {
    /// Creates a new configuration with custom host and port.
    ///
    /// # Arguments
    ///
    /// * `host` - Host address to bind to or connect to
    /// * `port` - Port number for MCP communications
    ///
    /// # Returns
    ///
    /// A new `McpConfig` instance with the specified host and port,
    /// and default values for other settings.
    pub fn new(host: impl Into<String>, port: u16) -> Self {
        Self {
            host: host.into(),
            port,
            ..Default::default()
        }
    }

    /// Sets the maximum number of concurrent connections.
    ///
    /// # Arguments
    ///
    /// * `max_connections` - Maximum number of connections to allow
    ///
    /// # Returns
    ///
    /// Updated configuration with the new max connections setting
    pub fn with_max_connections(mut self, max_connections: usize) -> Self {
        self.max_connections = max_connections;
        self
    }

    /// Sets the connection timeout in seconds.
    ///
    /// # Arguments
    ///
    /// * `timeout` - Timeout value in seconds
    ///
    /// # Returns
    ///
    /// Updated configuration with the new timeout setting
    pub fn with_timeout(mut self, timeout: u64) -> Self {
        self.timeout = timeout;
        self
    }

    /// Sets the buffer size for network operations.
    ///
    /// # Arguments
    ///
    /// * `buffer_size` - Buffer size in bytes
    ///
    /// # Returns
    ///
    /// Updated configuration with the new buffer size setting
    pub fn with_buffer_size(mut self, buffer_size: usize) -> Self {
        self.buffer_size = buffer_size;
        self
    }
}
