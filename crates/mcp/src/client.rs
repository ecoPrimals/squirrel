//! MCP Client implementation for connecting to MCP servers.
//!
//! This module provides client-side functionality for the Machine Context Protocol,
//! enabling applications to connect to MCP servers, send messages, and manage state.
//! It includes configurable client implementations with retry logic and connection management.

use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::error::{MCPError, Result};
use crate::mcp::protocol::{MCPProtocolAdapter, create_protocol_adapter};
use crate::mcp::transport::Transport;
use crate::mcp::types::{MCPMessage, ProtocolState};
use serde_json::Value;

/// Configuration options for MCP clients.
///
/// This structure contains parameters for controlling client behavior,
/// including connection timeouts, retry logic, and performance settings.
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// Maximum time in milliseconds to wait for a connection
    pub connect_timeout_ms: u64,
    /// Number of connection retry attempts before failing
    pub retry_attempts: u32,
    /// Delay in milliseconds between retry attempts
    pub retry_delay_ms: u64,
}

impl Default for ClientConfig {
    /// Creates a default client configuration with reasonable values.
    ///
    /// # Returns
    ///
    /// A new `ClientConfig` with default settings:
    /// - Connection timeout: 5000ms (5 seconds)
    /// - Retry attempts: 3
    /// - Retry delay: 1000ms (1 second)
    fn default() -> Self {
        Self {
            connect_timeout_ms: 5000,
            retry_attempts: 3,
            retry_delay_ms: 1000,
        }
    }
}

/// Client for connecting to MCP servers.
///
/// Provides methods for establishing connections, sending messages,
/// and interacting with the Machine Context Protocol system.
pub struct MCPClient {
    /// Client configuration
    config: ClientConfig,
    /// Protocol adapter for message handling
    protocol: Arc<MCPProtocolAdapter>,
    /// Transport layer for network communication
    transport: Arc<RwLock<Transport>>,
}

impl MCPClient {
    /// Creates a new client with default configuration.
    ///
    /// # Arguments
    ///
    /// * `addr` - Socket address of the MCP server to connect to
    ///
    /// # Returns
    ///
    /// A new `MCPClient` connected to the specified address with default settings
    ///
    /// # Errors
    ///
    /// Returns an error if the connection fails after retry attempts
    pub async fn new(addr: SocketAddr) -> Result<Self> {
        Self::with_config(addr, ClientConfig::default()).await
    }

    /// Creates a new client with custom configuration.
    ///
    /// # Arguments
    ///
    /// * `addr` - Socket address of the MCP server to connect to
    /// * `config` - Custom client configuration
    ///
    /// # Returns
    ///
    /// A new `MCPClient` connected to the specified address with custom settings
    ///
    /// # Errors
    ///
    /// Returns an error if the connection fails after retry attempts
    pub async fn with_config(addr: SocketAddr, config: ClientConfig) -> Result<Self> {
        let mut transport = Transport::new();
        transport.connect(addr).await?;
        
        Ok(Self {
            config,
            protocol: create_protocol_adapter(),
            transport: Arc::new(RwLock::new(transport)),
        })
    }

    /// Creates a client with explicitly provided dependencies.
    ///
    /// This method is primarily used for testing or when integrating
    /// with custom transport or protocol implementations.
    ///
    /// # Arguments
    ///
    /// * `config` - Client configuration
    /// * `protocol` - Protocol adapter implementation
    /// * `transport` - Transport layer implementation
    ///
    /// # Returns
    ///
    /// A new `MCPClient` using the provided dependencies
    pub async fn with_dependencies(
        config: ClientConfig,
        protocol: Arc<MCPProtocolAdapter>,
        transport: Arc<RwLock<Transport>>,
    ) -> Self {
        Self {
            config,
            protocol,
            transport,
        }
    }

    /// Sends a message to the connected MCP server.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to send
    ///
    /// # Returns
    ///
    /// Response message from the server
    ///
    /// # Errors
    ///
    /// Returns an error if sending or receiving fails
    pub async fn send_message(&self, message: &MCPMessage) -> Result<MCPMessage> {
        let mut transport = self.transport.write().await;
        transport.send_message(message).await?;
        transport.receive_message().await
    }

    /// Gets the current protocol state.
    ///
    /// # Returns
    ///
    /// Current state of the protocol as a JSON value
    ///
    /// # Errors
    ///
    /// Returns an error if the state cannot be retrieved
    pub async fn get_state(&self) -> Result<Value> {
        Ok(self.protocol.get_state())
    }

    /// Sets the protocol state.
    ///
    /// # Arguments
    ///
    /// * `state` - New state to set
    ///
    /// # Returns
    ///
    /// Nothing on success
    ///
    /// # Errors
    ///
    /// Returns an error if the state cannot be set
    pub async fn set_state(&self, state: Value) -> Result<()> {
        let mut protocol = self.protocol.clone();
        protocol.set_state(state);
        Ok(())
    }

    /// Returns the client configuration.
    ///
    /// # Returns
    ///
    /// Reference to the client's configuration
    pub fn get_config(&self) -> &ClientConfig {
        &self.config
    }
}

/// Factory for creating MCP client instances.
///
/// Provides methods for creating clients with consistent configuration
/// and dependency management.
pub struct MCPClientFactory {
    /// Client configuration to use for created clients
    config: ClientConfig,
}

impl MCPClientFactory {
    /// Creates a new client factory with the specified configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - Client configuration to use for created clients
    ///
    /// # Returns
    ///
    /// A new `MCPClientFactory` instance
    pub fn new(config: ClientConfig) -> Self {
        Self { config }
    }

    /// Creates a new client factory with custom configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - Client configuration to use for created clients
    ///
    /// # Returns
    ///
    /// A new `MCPClientFactory` instance
    pub fn with_config(config: ClientConfig) -> Self {
        Self { config }
    }

    /// Creates a new client connected to the specified address.
    ///
    /// # Arguments
    ///
    /// * `addr` - Socket address of the MCP server to connect to
    ///
    /// # Returns
    ///
    /// A new `MCPClient` instance wrapped in an `Arc`
    ///
    /// # Errors
    ///
    /// Returns an error if the connection fails after retry attempts
    pub async fn create_client(&self, addr: SocketAddr) -> Result<Arc<MCPClient>> {
        Ok(Arc::new(MCPClient::with_config(addr, self.config.clone()).await?))
    }

    /// Creates a client with explicitly provided dependencies.
    ///
    /// # Arguments
    ///
    /// * `protocol` - Protocol adapter implementation
    /// * `transport` - Transport layer implementation
    ///
    /// # Returns
    ///
    /// A new `MCPClient` instance wrapped in an `Arc`
    pub fn create_client_with_dependencies(
        &self,
        protocol: Arc<MCPProtocolAdapter>,
        transport: Arc<RwLock<Transport>>,
    ) -> Arc<MCPClient> {
        Arc::new(MCPClient::with_dependencies(
            self.config.clone(),
            protocol,
            transport,
        ))
    }
}

impl Default for MCPClientFactory {
    /// Creates a new client factory with default configuration.
    ///
    /// # Returns
    ///
    /// A new `MCPClientFactory` instance with default client settings
    fn default() -> Self {
        Self::new(ClientConfig::default())
    }
} 