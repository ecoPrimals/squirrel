//! MCP (Machine Context Protocol) module
//!
//! This module implements the core functionality for the Machine Context Protocol.

use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use thiserror::Error;

/// Error type for MCP operations
#[derive(Debug, Error)]
pub enum MCPError {
    /// Lock acquisition failed
    #[error("Failed to acquire lock: {0}")]
    LockError(String),
}

/// A Result type alias for MCP operations
pub type Result<T> = std::result::Result<T, MCPError>;

/// Configuration for the MCP system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPConfig {
    /// The protocol version
    pub version: String,
    /// Maximum message size in bytes
    pub max_message_size: u64,
    /// Timeout in milliseconds
    pub timeout_ms: u64,
}

/// The main MCP system controller
pub struct MCP {
    /// The MCP configuration, wrapped in a thread-safe read-write lock
    config: Arc<RwLock<MCPConfig>>,
}

impl MCP {
    /// Creates a new MCP instance with the specified configuration
    #[must_use]
    pub fn new(config: MCPConfig) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
        }
    }

    /// Gets the current configuration
    ///
    /// # Errors
    ///
    /// Returns a `MCPError::LockError` if the configuration lock cannot be acquired
    #[must_use = "This returns the current MCP configuration which may be needed for further operations"]
    pub async fn get_config(&self) -> Result<MCPConfig> {
        let config = self.config.read().await;
        Ok(config.clone())
    }
}

impl Default for MCPConfig {
    fn default() -> Self {
        Self {
            version: "1.0".to_string(),
            max_message_size: 1024 * 1024 * 10, // 10MB
            timeout_ms: 30000, // 30 seconds
        }
    }
}

impl Default for MCP {
    fn default() -> Self {
        Self::new(MCPConfig::default())
    }
} 