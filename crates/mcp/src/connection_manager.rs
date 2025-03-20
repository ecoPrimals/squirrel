use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock as TokioRwLock};
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::{error, info, instrument, warn};
use thiserror::Error;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use anyhow::Result;
use std::time::Duration;
use serde_json::Value;
use std::net::SocketAddr;

use crate::mcp::{
    MCPMessage, PortManager, SecurityManager, ErrorHandler, StateManager, ContextManager,
    MessageHandler, MessageHandlerConfig,
    protocol::{MessageType, ProtocolVersion},
    security_manager::SecurityManager,
    error::{MCPError, ConnectionError},
    port_manager::{PortConfig, PortAccessControl},
};

#[derive(Debug, thiserror::Error)]
pub enum ConnectionError {
    #[error("Connection {0} already exists")]
    AlreadyExists(String),
    #[error("Connection {0} not found")]
    NotFound(String),
    #[error("Invalid connection state: {0}")]
    InvalidState(String),
    #[error("Security error: {0}")]
    SecurityError(String),
    #[error("Protocol error: {0}")]
    ProtocolError(String),
    #[error("IO error: {0}")]
    IOError(String),
    #[error("Connection closed: {0}")]
    Closed(String),
    #[error("Invalid port: {0}")]
    InvalidPort(u16),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    pub max_connections: usize,
    pub connection_timeout: chrono::Duration,
    pub keep_alive_interval: chrono::Duration,
    pub max_message_queue: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionStatus {
    Connected,
    Disconnected,
    Connecting,
    Disconnecting,
    Error(String),
}

#[derive(Debug)]
pub struct Connection {
    pub id: String,
    pub status: ConnectionStatus,
    pub created_at: DateTime<Utc>,
    pub last_active: DateTime<Utc>,
    pub message_tx: mpsc::Sender<MCPMessage>,
    pub message_rx: mpsc::Receiver<MCPMessage>,
}

impl Clone for Connection {
    fn clone(&self) -> Self {
        let (tx, rx) = mpsc::channel(100);
        Self {
            id: self.id.clone(),
            status: self.status.clone(),
            created_at: self.created_at,
            last_active: self.last_active,
            message_tx: tx,
            message_rx: rx,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConnectionManagerConfig {
    pub max_connections: u32,
    pub connection_timeout: Duration,
    pub keep_alive_interval: Duration,
    pub max_message_queue: usize,
}

impl Default for ConnectionManagerConfig {
    fn default() -> Self {
        Self {
            max_connections: 100,
            connection_timeout: Duration::from_secs(30),
            keep_alive_interval: Duration::from_secs(10),
            max_message_queue: 1000,
        }
    }
}

pub struct ConnectionManager {
    pub connections: Arc<TokioRwLock<HashMap<String, Connection>>>,
    pub config: ConnectionConfig,
    port_config: PortConfig,
    security_manager: Arc<SecurityManager>,
    error_handler: Arc<ErrorHandler>,
}

impl ConnectionManager {
    pub fn new(
        config: ConnectionManagerConfig,
        port_config: PortConfig,
        security_manager: Arc<SecurityManager>,
        error_handler: Arc<ErrorHandler>,
    ) -> Self {
        Self {
            config,
            port_config,
            security_manager,
            error_handler,
            connections: Arc::new(TokioRwLock::new(HashMap::new())),
        }
    }

    #[instrument(skip(self))]
    pub async fn create_connection(&self, addr: SocketAddr) -> Result<Connection> {
        let mut connections = self.connections.write().await;
        
        if connections.len() >= self.config.max_connections as usize {
            return Err(anyhow::anyhow!("Maximum connections reached"));
        }

        let (message_tx, message_rx) = mpsc::channel(self.config.max_message_queue);
        let connection = Connection {
            id: Uuid::new_v4().to_string(),
            addr,
            created_at: Utc::now(),
            last_active: Utc::now(),
            status: ConnectionStatus::Connected,
            message_tx,
            message_rx,
        };

        connections.insert(connection.id.clone(), connection.clone());
        Ok(connection)
    }

    #[instrument(skip(self))]
    pub async fn close_connection(&self, id: String) -> Result<()> {
        let mut connections = self.connections.write().await;
        
        if let Some(connection) = connections.remove(&id) {
            if let Err(e) = connection.message_tx.send(MCPMessage {
                version: ProtocolVersion::default(),
                message_type: MessageType::Event,
                payload: serde_json::Value::Null,
                metadata: None,
            }).await {
                error!("Failed to send close message: {}", e);
            }
        }
        
        Ok(())
    }

    pub async fn get_connection(&self, id: &str) -> Result<Option<Connection>> {
        let connections = self.connections.read().await;
        Ok(connections.get(id).cloned())
    }

    #[instrument(skip(self))]
    pub async fn update_connection_status(&self, id: String, status: ConnectionStatus) -> Result<()> {
        let mut connections = self.connections.write().await;
        
        if let Some(connection) = connections.get_mut(&id) {
            connection.status = status;
            connection.last_active = Utc::now();
        }
        
        Ok(())
    }

    pub async fn add_connection(&self, connection: Connection) -> Result<()> {
        let mut connections = self.connections.write().await;
        connections.insert(connection.id.clone(), connection);
        Ok(())
    }

    pub async fn remove_connection(&self, id: &str) -> Result<()> {
        let mut connections = self.connections.write().await;
        connections.remove(id);
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn handle_message(&self, message: MCPMessage) -> Result<()> {
        // Implementation for handling messages
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::SocketAddr;
    use std::str::FromStr;

    #[tokio::test]
    async fn test_connection_creation() {
        let config = ConnectionManagerConfig::default();
        let port_config = PortConfig::default();
        let security_manager = Arc::new(SecurityManager::new());
        let error_handler = Arc::new(ErrorHandler::new(100));

        let manager = ConnectionManager::new(
            config,
            port_config,
            security_manager,
            error_handler,
        );

        let addr = SocketAddr::from_str("127.0.0.1:8080").unwrap();
        let connection = manager.create_connection(addr).await.unwrap();

        assert_eq!(connection.addr, addr);
        assert_eq!(connection.status, ConnectionStatus::Connected);
    }

    #[tokio::test]
    async fn test_connection_cleanup() {
        let config = ConnectionManagerConfig::default();
        let port_config = PortConfig::default();
        let security_manager = Arc::new(SecurityManager::new());
        let error_handler = Arc::new(ErrorHandler::new(100));

        let manager = ConnectionManager::new(
            config,
            port_config,
            security_manager,
            error_handler,
        );

        let addr = SocketAddr::from_str("127.0.0.1:8080").unwrap();
        let connection = manager.create_connection(addr).await.unwrap();
        
        manager.close_connection(connection.id).await.unwrap();
    }
} 