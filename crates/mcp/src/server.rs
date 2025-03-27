//! MCP Server module
//!
//! This module implements the server functionality for the Machine Context Protocol (MCP).
//! It handles client connections, message routing, and protocol state management.
//! The server provides a WebSocket-based interface for clients to connect and communicate
//! using the MCP protocol.

use std::sync::Arc;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use tokio::sync::RwLock;
use serde_json::Value;
use std::net::SocketAddr;

// Import core error types
use crate::core::error::{CoreError, CoreResult};
use crate::error::{MCPError, ProtocolError};

// Import common types
use crate::mcp::types::{
    MCPMessage,
    ProtocolVersion,
    ProtocolState,
    MCPCommand,
    CompressionFormat,
    EncryptionFormat,
    MessageMetadata,
    ResponseStatus,
    MCPResponse,
};

// Import protocol types
use crate::mcp::protocol::{
    MCPProtocol,
    CommandHandler,
    MCPProtocolAdapter,
    create_protocol_adapter,
};

// Import transport types
use crate::mcp::transport::Transport;

/// Server-specific result type for MCP operations
pub type Result<T> = std::result::Result<T, MCPError>;

// Re-export common types
pub use crate::mcp::types::{
    MCPMessage,
    ProtocolVersion,
    ProtocolState,
    MCPCommand,
    CompressionFormat,
    EncryptionFormat,
    MessageMetadata,
    ResponseStatus,
    MCPResponse,
};

// Re-export error types
pub use crate::error::{MCPError, ProtocolError};

/// Configuration for the MCP server.
///
/// This structure contains parameters for controlling server behavior,
/// including network settings, connection limits, and timeouts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Network address to bind the server to
    pub bind_address: SocketAddr,
    /// Maximum number of concurrent connections
    pub max_connections: usize,
    /// Connection timeout in milliseconds
    pub connection_timeout_ms: u64,
    /// Keep-alive interval in milliseconds
    pub keep_alive_interval_ms: u64,
    /// Maximum message size in bytes
    pub max_message_size: usize,
}

impl Default for ServerConfig {
    /// Creates a default server configuration with reasonable values.
    ///
    /// # Returns
    ///
    /// A new `ServerConfig` with default settings:
    /// - Bind address: 127.0.0.1:8080
    /// - Max connections: 1000
    /// - Connection timeout: 30 seconds
    /// - Keep-alive interval: 5 seconds
    /// - Max message size: 1MB
    fn default() -> Self {
        Self {
            bind_address: "127.0.0.1:8080".parse().unwrap(),
            max_connections: 1000,
            connection_timeout_ms: 30000,
            keep_alive_interval_ms: 5000,
            max_message_size: 1024 * 1024, // 1MB
        }
    }
}

/// MCP Server that manages connections and message processing.
///
/// This struct provides functionality for running an MCP server,
/// handling client connections, and processing MCP protocol messages.
pub struct MCPServer {
    /// Server configuration
    config: ServerConfig,
    /// Protocol adapter for message handling
    protocol: Arc<MCPProtocolAdapter>,
    /// Transport layer for network communication
    transport: Arc<RwLock<Transport>>,
    /// Active connections mapped by socket address to connection time
    connections: Arc<RwLock<HashMap<SocketAddr, DateTime<Utc>>>>,
}

impl MCPServer {
    /// Creates a new server with the specified configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - Server configuration
    ///
    /// # Returns
    ///
    /// A new `MCPServer` instance
    ///
    /// # Errors
    ///
    /// Returns an error if the server initialization fails
    pub async fn new(config: ServerConfig) -> Result<Self> {
        let mut transport = Transport::new();
        transport.bind(config.bind_address).await?;
        
        Ok(Self {
            config,
            protocol: create_protocol_adapter(),
            transport: Arc::new(RwLock::new(transport)),
            connections: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Creates a server with explicitly provided dependencies.
    ///
    /// This method is primarily used for testing or when integrating
    /// with custom transport or protocol implementations.
    ///
    /// # Arguments
    ///
    /// * `config` - Server configuration
    /// * `protocol` - Protocol adapter implementation
    /// * `transport` - Transport layer implementation
    /// * `connections` - Connection tracking map
    ///
    /// # Returns
    ///
    /// A new `MCPServer` using the provided dependencies
    pub fn with_dependencies(
        config: ServerConfig,
        protocol: Arc<MCPProtocolAdapter>,
        transport: Arc<RwLock<Transport>>,
        connections: Arc<RwLock<HashMap<SocketAddr, DateTime<Utc>>>>,
    ) -> Self {
        Self {
            config,
            protocol,
            transport,
            connections,
        }
    }

    /// Starts the server and begins accepting connections.
    ///
    /// # Returns
    ///
    /// Nothing on success
    ///
    /// # Errors
    ///
    /// Returns an error if the server fails to start
    pub async fn start(&self) -> Result<()> {
        let mut transport = self.transport.write().await;
        transport.start().await?;
        
        // Start connection handling loop
        self.handle_connections().await
    }

    /// Stops the server and closes all connections.
    ///
    /// # Returns
    ///
    /// Nothing on success
    ///
    /// # Errors
    ///
    /// Returns an error if the server fails to stop gracefully
    pub async fn stop(&self) -> Result<()> {
        let mut transport = self.transport.write().await;
        transport.stop().await?;
        
        // Clean up connections
        let mut connections = self.connections.write().await;
        connections.clear();
        
        Ok(())
    }

    /// Handles incoming connections and processes messages.
    ///
    /// This method runs in a loop accepting connections and processing
    /// messages from clients.
    ///
    /// # Returns
    ///
    /// Nothing if the loop is terminated gracefully
    ///
    /// # Errors
    ///
    /// Returns an error if connection handling fails
    async fn handle_connections(&self) -> Result<()> {
        loop {
            let mut transport = self.transport.write().await;
            let (message, addr) = transport.accept_connection().await?;
            
            // Update connection tracking
            let mut connections = self.connections.write().await;
            connections.insert(addr, Utc::now());
            
            // Handle message
            let response = self.protocol.handle_message(&message).await?;
            transport.send_message_to(&response, addr).await?;
        }
    }

    /// Returns the server configuration.
    ///
    /// # Returns
    ///
    /// Reference to the server's configuration
    pub fn get_config(&self) -> &ServerConfig {
        &self.config
    }

    /// Gets the list of currently active connections.
    ///
    /// # Returns
    ///
    /// Vector of socket addresses for all active connections
    ///
    /// # Errors
    ///
    /// Returns an error if the connection list cannot be retrieved
    pub async fn get_connections(&self) -> Result<Vec<SocketAddr>> {
        let connections = self.connections.read().await;
        Ok(connections.keys().cloned().collect())
    }
}

/// Factory for creating MCP server instances.
///
/// Provides methods for creating servers with consistent configuration
/// and dependency management.
pub struct MCPServerFactory {
    /// Server configuration to use for created servers
    config: ServerConfig,
}

impl MCPServerFactory {
    /// Creates a new server factory with the specified configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - Server configuration to use for created servers
    ///
    /// # Returns
    ///
    /// A new `MCPServerFactory` instance
    pub fn new(config: ServerConfig) -> Self {
        Self { config }
    }

    /// Creates a new server factory with custom configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - Server configuration to use for created servers
    ///
    /// # Returns
    ///
    /// A new `MCPServerFactory` instance
    pub fn with_config(config: ServerConfig) -> Self {
        Self { config }
    }

    /// Creates a new server with the factory's configuration.
    ///
    /// # Returns
    ///
    /// A new `MCPServer` instance wrapped in an `Arc`
    ///
    /// # Errors
    ///
    /// Returns an error if the server initialization fails
    pub async fn create_server(&self) -> Result<Arc<MCPServer>> {
        Ok(Arc::new(MCPServer::new(self.config.clone()).await?))
    }

    /// Creates a server with explicitly provided dependencies.
    ///
    /// # Arguments
    ///
    /// * `protocol` - Protocol adapter implementation
    /// * `transport` - Transport layer implementation
    /// * `connections` - Connection tracking map
    ///
    /// # Returns
    ///
    /// A new `MCPServer` instance wrapped in an `Arc`
    pub fn create_server_with_dependencies(
        &self,
        protocol: Arc<MCPProtocolAdapter>,
        transport: Arc<RwLock<Transport>>,
        connections: Arc<RwLock<HashMap<SocketAddr, DateTime<Utc>>>>,
    ) -> Arc<MCPServer> {
        Arc::new(MCPServer::with_dependencies(
            self.config.clone(),
            protocol,
            transport,
            connections,
        ))
    }
}

impl Default for MCPServerFactory {
    /// Creates a new server factory with default configuration.
    ///
    /// # Returns
    ///
    /// A new `MCPServerFactory` instance with default server settings
    fn default() -> Self {
        Self::new(ServerConfig::default())
    }
}

/// Represents a client connection to the MCP server.
///
/// Tracks connection state and handles protocol operations for a single client.
pub struct ClientConnection {
    /// Unique identifier for the connection
    id: String,
    /// Protocol instance for message handling
    protocol: Arc<RwLock<MCPProtocol>>,
    /// Current state of the protocol for this connection
    state: Arc<RwLock<ProtocolState>>,
}

impl ClientConnection {
    /// Creates a new client connection.
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier for the connection
    /// * `protocol` - Protocol instance for message handling
    ///
    /// # Returns
    ///
    /// A new `ClientConnection` instance
    pub fn new(id: String, protocol: Arc<RwLock<MCPProtocol>>) -> Self {
        Self {
            id,
            protocol,
            state: Arc::new(RwLock::new(ProtocolState::Initializing)),
        }
    }

    /// Handles an incoming message from the client.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to process
    ///
    /// # Returns
    ///
    /// Response message to send back to the client
    ///
    /// # Errors
    ///
    /// Returns an error if message handling fails
    pub async fn handle_message(&self, message: MCPMessage) -> Result<MCPResponse> {
        let protocol = self.protocol.read().await;
        protocol.handle_message(message).await
    }

    /// Gets the current state of the connection.
    ///
    /// # Returns
    ///
    /// Current protocol state for this connection
    pub async fn get_state(&self) -> ProtocolState {
        self.state.read().await.clone()
    }

    /// Sets the current state of the connection.
    ///
    /// # Arguments
    ///
    /// * `state` - New state to set
    pub async fn set_state(&self, state: ProtocolState) {
        *self.state.write().await = state;
    }
}

/// Alternative name for the server result type
/// 
/// This type alias is provided for backward compatibility with code
/// that relies on the older naming convention.
pub type ServerResult<T> = std::result::Result<T, MCPError>; 