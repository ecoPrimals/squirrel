/// Port management module for MCP (Machine Context Protocol)
///
/// This module provides functionality for managing network ports used by MCP services.
/// It handles port configuration, connection tracking, and port lifecycle management.
/// The implementation is designed to be async-compatible and thread-safe.

use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use crate::error::{MCPError, Result};
use crate::types::{ProtocolVersion, ProtocolState};

/// Configuration for an MCP port
///
/// Contains all the parameters needed to configure a network port for MCP communication,
/// including the port number, hostname, connection limits, and timeouts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortConfig {
    /// The port number to listen on
    pub port: u16,
    
    /// The hostname or IP address to bind to
    pub host: String,
    
    /// Maximum number of simultaneous connections allowed
    pub max_connections: u32,
    
    /// Connection timeout in milliseconds
    pub timeout_ms: u64,
}

/// State information for an MCP port
///
/// Contains runtime state information about a port, including whether it's actively
/// listening, current connection count, and connection statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortState {
    /// Whether the port is currently listening for connections
    pub is_listening: bool,
    
    /// Number of connections currently active
    pub active_connections: u32,
    
    /// Total number of connections handled since started
    pub total_connections: u64,
    
    /// Timestamp of the last connection attempt
    pub last_connection: chrono::DateTime<chrono::Utc>,
}

/// An MCP port manager
///
/// Manages a network port used for MCP communication, providing methods for
/// starting/stopping the port, tracking connections, and monitoring port state.
pub struct MCPPort {
    /// Port configuration, protected by a read-write lock for thread safety
    config: Arc<RwLock<PortConfig>>,
    
    /// Port state, protected by a read-write lock for thread safety
    state: Arc<RwLock<PortState>>,
}

impl MCPPort {
    /// Creates a new port manager with the specified configuration
    ///
    /// # Arguments
    ///
    /// * `config` - The configuration for the port
    ///
    /// # Returns
    ///
    /// A new `MCPPort` instance initialized with the given configuration
    #[must_use] pub fn new(config: PortConfig) -> Self {
        let state = PortState {
            is_listening: false,
            active_connections: 0,
            total_connections: 0,
            last_connection: chrono::Utc::now(),
        };

        Self {
            config: Arc::new(RwLock::new(config)),
            state: Arc::new(RwLock::new(state)),
        }
    }

    /// Start the port
    ///
    /// Begins listening for incoming connections on the configured port.
    ///
    /// # Returns
    ///
    /// A Result indicating success or an error
    ///
    /// # Errors
    ///
    /// Returns an error if the port cannot be started or if the internal
    /// state lock cannot be acquired
    pub async fn start(&self) -> Result<()> {
        // TODO: Implement actual port listening
        self.state.write().await.is_listening = true;
        Ok(())
    }

    /// Stop the port
    ///
    /// Stops listening for incoming connections on the configured port.
    ///
    /// # Returns
    ///
    /// A Result indicating success or an error
    ///
    /// # Errors
    ///
    /// Returns an error if the port cannot be stopped or if the internal
    /// state lock cannot be acquired
    pub async fn stop(&self) -> Result<()> {
        // TODO: Implement actual port stopping
        self.state.write().await.is_listening = false;
        Ok(())
    }

    /// Update port configuration
    ///
    /// Updates the configuration of the port.
    ///
    /// # Arguments
    ///
    /// * `config` - The new port configuration
    ///
    /// # Returns
    ///
    /// A Result indicating success or an error
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration cannot be updated or if the internal
    /// configuration lock cannot be acquired
    pub async fn update_config(&self, config: PortConfig) -> Result<()> {
        *self.config.write().await = config;
        Ok(())
    }

    /// Get the current port configuration
    ///
    /// # Returns
    ///
    /// A Result containing the current port configuration
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration cannot be read or if the internal
    /// configuration lock cannot be acquired
    pub async fn get_config(&self) -> Result<PortConfig> {
        let config = self.config.read().await;
        Ok(config.clone())
    }

    /// Retrieves the current state of the port
    ///
    /// # Returns
    ///
    /// The current port state wrapped in a Result
    ///
    /// # Errors
    /// 
    /// Returns an error if the internal state lock cannot be acquired
    pub async fn get_state(&self) -> Result<PortState> {
        let state = self.state.read().await;
        Ok(state.clone())
    }

    /// Records a new connection to the port
    ///
    /// Updates the connection statistics and timestamps when a new connection is established.
    ///
    /// # Returns
    ///
    /// A Result indicating success or an error
    ///
    /// # Errors
    ///
    /// Returns an error if the internal state lock cannot be acquired
    pub async fn record_connection(&self) -> Result<()> {
        let mut state = self.state.write().await;
        state.active_connections += 1;
        state.total_connections += 1;
        state.last_connection = chrono::Utc::now();
        drop(state); // Early drop the state lock
        Ok(())
    }

    /// Records a disconnection from the port
    ///
    /// Updates the connection statistics when a connection is closed.
    ///
    /// # Returns
    ///
    /// A Result indicating success or an error
    ///
    /// # Errors
    ///
    /// Returns an error if the internal state lock cannot be acquired
    pub async fn record_disconnection(&self) -> Result<()> {
        let mut state = self.state.write().await;
        if state.active_connections > 0 {
            state.active_connections -= 1;
        }
        drop(state); // Early drop the state lock
        Ok(())
    }
}

impl Default for PortConfig {
    /// Creates a default port configuration
    ///
    /// Default values:
    /// - Port: 8080
    /// - Host: 127.0.0.1
    /// - Max connections: 100
    /// - Timeout: 30 seconds
    fn default() -> Self {
        Self {
            port: 8080,
            host: "127.0.0.1".to_string(),
            max_connections: 100,
            timeout_ms: 30000, // 30 seconds
        }
    }
}

impl Default for MCPPort {
    /// Creates a default port manager
    ///
    /// Uses the default port configuration.
    fn default() -> Self {
        Self::new(PortConfig::default())
    }
} 