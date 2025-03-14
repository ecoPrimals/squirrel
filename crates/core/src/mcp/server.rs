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

// Export server types
pub struct MCPServer {
    protocol: Arc<RwLock<MCPProtocol>>,
    transport: Transport,
}

impl MCPServer {
    pub fn new(protocol: Arc<RwLock<MCPProtocol>>, transport: Transport) -> Self {
        Self {
            protocol,
            transport,
        }
    }

    pub async fn start(&mut self) -> Result<()> {
        loop {
            let message = self.transport.receive_message().await?;
            let response = self.handle_message(&message).await?;
            self.transport.send_message(&response).await?;
        }
    }

    async fn handle_message(&self, message: &MCPMessage) -> Result<MCPMessage> {
        let protocol = self.protocol.read().await;
        protocol.handle_message(message).await
    }

    pub async fn get_state(&self) -> Result<Value> {
        let protocol = self.protocol.read().await;
        protocol.get_state()
    }

    pub async fn set_state(&self, state: Value) -> Result<()> {
        let mut protocol = self.protocol.write().await;
        protocol.set_state(state);
        Ok(())
    }

    pub async fn handle_client_connection(&self, client_id: String, connection: ClientConnection) -> Result<()> {
        let mut clients = self.clients.write().await;
        clients.insert(client_id, connection);
        Ok(())
    }

    pub async fn remove_client(&self, client_id: &str) -> Result<()> {
        let mut clients = self.clients.write().await;
        clients.remove(client_id);
        Ok(())
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