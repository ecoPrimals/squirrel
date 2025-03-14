use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::error::{MCPError, Result};
use crate::mcp::protocol::MCPProtocol;
use crate::mcp::transport::Transport;
use crate::mcp::types::{MCPMessage, ProtocolState};
use serde_json::Value;

pub struct MCPClient {
    protocol: Arc<RwLock<MCPProtocol>>,
    transport: Transport,
}

impl MCPClient {
    pub async fn new(addr: SocketAddr) -> Result<Self> {
        let mut transport = Transport::new();
        transport.connect(addr).await?;
        
        Ok(Self {
            protocol: Arc::new(RwLock::new(MCPProtocol::new())),
            transport,
        })
    }

    pub async fn send_message(&mut self, message: &MCPMessage) -> Result<MCPMessage> {
        self.transport.send_message(message).await?;
        self.transport.receive_message().await
    }

    pub async fn get_state(&self) -> Result<Value> {
        let protocol = self.protocol.read().await;
        Ok(protocol.get_state().clone())
    }

    pub async fn set_state(&self, state: Value) -> Result<()> {
        let mut protocol = self.protocol.write().await;
        protocol.set_state(state);
        Ok(())
    }
} 