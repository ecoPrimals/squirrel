use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::error::{MCPError, Result};
use crate::mcp::protocol::{MCPProtocolAdapter, create_protocol_adapter};
use crate::mcp::transport::Transport;
use crate::mcp::types::{MCPMessage, ProtocolState};
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub connect_timeout_ms: u64,
    pub retry_attempts: u32,
    pub retry_delay_ms: u64,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            connect_timeout_ms: 5000,
            retry_attempts: 3,
            retry_delay_ms: 1000,
        }
    }
}

pub struct MCPClient {
    config: ClientConfig,
    protocol: Arc<MCPProtocolAdapter>,
    transport: Arc<RwLock<Transport>>,
}

impl MCPClient {
    pub async fn new(addr: SocketAddr) -> Result<Self> {
        Self::with_config(addr, ClientConfig::default()).await
    }

    pub async fn with_config(addr: SocketAddr, config: ClientConfig) -> Result<Self> {
        let mut transport = Transport::new();
        transport.connect(addr).await?;
        
        Ok(Self {
            config,
            protocol: create_protocol_adapter(),
            transport: Arc::new(RwLock::new(transport)),
        })
    }

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

    pub async fn send_message(&self, message: &MCPMessage) -> Result<MCPMessage> {
        let mut transport = self.transport.write().await;
        transport.send_message(message).await?;
        transport.receive_message().await
    }

    pub async fn get_state(&self) -> Result<Value> {
        Ok(self.protocol.get_state())
    }

    pub async fn set_state(&self, state: Value) -> Result<()> {
        let mut protocol = self.protocol.clone();
        protocol.set_state(state);
        Ok(())
    }

    pub fn get_config(&self) -> &ClientConfig {
        &self.config
    }
}

pub struct MCPClientFactory {
    config: ClientConfig,
}

impl MCPClientFactory {
    pub fn new(config: ClientConfig) -> Self {
        Self { config }
    }

    pub fn with_config(config: ClientConfig) -> Self {
        Self { config }
    }

    pub async fn create_client(&self, addr: SocketAddr) -> Result<Arc<MCPClient>> {
        Ok(Arc::new(MCPClient::with_config(addr, self.config.clone()).await?))
    }

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
    fn default() -> Self {
        Self::new(ClientConfig::default())
    }
} 