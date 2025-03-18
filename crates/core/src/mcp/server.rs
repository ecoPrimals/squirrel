//! MCP Server module
//!
//! This module implements the server functionality for the Machine Context Protocol (MCP).
//! It handles client connections, message routing, and protocol state management.

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

// Define server specific result type
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub bind_address: SocketAddr,
    pub max_connections: usize,
    pub connection_timeout_ms: u64,
    pub keep_alive_interval_ms: u64,
    pub max_message_size: usize,
}

impl Default for ServerConfig {
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

pub struct MCPServer {
    config: ServerConfig,
    protocol: Arc<MCPProtocolAdapter>,
    transport: Arc<RwLock<Transport>>,
    connections: Arc<RwLock<HashMap<SocketAddr, DateTime<Utc>>>>,
}

impl MCPServer {
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

    pub async fn start(&self) -> Result<()> {
        let mut transport = self.transport.write().await;
        transport.start().await?;
        
        // Start connection handling loop
        self.handle_connections().await
    }

    pub async fn stop(&self) -> Result<()> {
        let mut transport = self.transport.write().await;
        transport.stop().await?;
        
        // Clean up connections
        let mut connections = self.connections.write().await;
        connections.clear();
        
        Ok(())
    }

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

    pub fn get_config(&self) -> &ServerConfig {
        &self.config
    }

    pub async fn get_connections(&self) -> Result<Vec<SocketAddr>> {
        let connections = self.connections.read().await;
        Ok(connections.keys().cloned().collect())
    }
}

pub struct MCPServerFactory {
    config: ServerConfig,
}

impl MCPServerFactory {
    pub fn new(config: ServerConfig) -> Self {
        Self { config }
    }

    pub fn with_config(config: ServerConfig) -> Self {
        Self { config }
    }

    pub async fn create_server(&self) -> Result<Arc<MCPServer>> {
        Ok(Arc::new(MCPServer::new(self.config.clone()).await?))
    }

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
    fn default() -> Self {
        Self::new(ServerConfig::default())
    }
}

pub struct ClientConnection {
    id: String,
    protocol: Arc<RwLock<MCPProtocol>>,
    state: Arc<RwLock<ProtocolState>>,
}

impl ClientConnection {
    pub fn new(id: String, protocol: Arc<RwLock<MCPProtocol>>) -> Self {
        Self {
            id,
            protocol,
            state: Arc::new(RwLock::new(ProtocolState::Initializing)),
        }
    }

    pub async fn handle_message(&self, message: MCPMessage) -> Result<MCPResponse> {
        let protocol = self.protocol.read().await;
        protocol.handle_message(message).await
    }

    pub async fn get_state(&self) -> ProtocolState {
        self.state.read().await.clone()
    }

    pub async fn set_state(&self, state: ProtocolState) {
        *self.state.write().await = state;
    }
}

// Export common types
pub type ServerResult<T> = std::result::Result<T, MCPError>; 