// crates/mcp/src/config/mod.rs
//! Configuration structures for MCP components.

mod encryption;
mod scaling;
mod service_security;

pub use encryption::EncryptionConfig;
pub use scaling::ScalingConfig;
pub use service_security::{
    ServiceSecurityConfig,
    AccessControlConfig,
};

use serde::{Deserialize, Serialize};
use crate::protocol::domain_objects::EncryptionFormat;
 // Assuming EncryptionFormat is needed

/// Configuration for security-related functionality
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Default encryption format to use
    pub encryption_default_format: String,
    // Add other security configuration fields as needed
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            encryption_default_format: "AES256GCM".to_string(),
        }
    }
}

/// Configuration for RBAC functionality
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RBACConfig {
    // Add RBAC configuration fields as needed
}

impl Default for RBACConfig {
    fn default() -> Self {
        Self {}
    }
}

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
    /// Security configuration
    pub security: SecurityConfig,
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
            security: SecurityConfig::default(),
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
    #[must_use] pub const fn with_max_connections(mut self, max_connections: usize) -> Self {
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
    #[must_use] pub const fn with_timeout(mut self, timeout: u64) -> Self {
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
    #[must_use] pub const fn with_buffer_size(mut self, buffer_size: usize) -> Self {
        self.buffer_size = buffer_size;
        self
    }
    
    /// Sets the security configuration.
    ///
    /// # Arguments
    ///
    /// * `security` - Security configuration
    ///
    /// # Returns
    ///
    /// Updated configuration with the new security settings
    #[must_use] pub fn with_security(mut self, security: SecurityConfig) -> Self {
        self.security = security;
        self
    }
}

/// Configuration for Memory Transport.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryTransportConfig {
    /// Transport name, useful for debugging
    pub name: String,
    
    /// Channel buffer size
    pub buffer_size: usize,
    
    /// Maximum message count in history
    pub max_history: Option<usize>,
    
    /// Simulate latency in milliseconds
    pub simulated_latency_ms: Option<u64>,
    
    /// Simulate random connection failures
    pub simulate_failures: bool,
    
    /// Encryption format (for metadata only)
    pub encryption: EncryptionFormat,
    
    /// Compression format (for metadata only)
    pub compression: crate::types::CompressionFormat,
    
    /// Channel size option (from original definition)
    pub channel_size: Option<usize>,
}

impl Default for MemoryTransportConfig {
    fn default() -> Self {
        Self {
            name: "memory".to_string(),
            buffer_size: 100,
            max_history: Some(1000),
            simulated_latency_ms: None,
            simulate_failures: false,
            encryption: EncryptionFormat::default(),
            compression: crate::types::CompressionFormat::None,
            channel_size: None,
        }
    }
}

// Add other config structs as needed (e.g., TcpTransportConfig, WebSocketConfig, StdioConfig, PersistenceConfig, SyncConfig) 