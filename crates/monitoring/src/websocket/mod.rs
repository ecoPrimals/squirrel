// WebSocket API for the monitoring system
// Provides real-time data access to monitoring metrics and component status

use std::fmt::Debug;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use squirrel_core::error::Result;
use async_trait::async_trait;
use std::sync::Arc;
use tracing::error;
use std::net::SocketAddr;
use tokio::task::JoinHandle;

/// Configuration for WebSocket connections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketConfig {
    /// WebSocket server host
    pub host: String,
    /// WebSocket server port
    pub port: u16,
    /// Data update interval in seconds
    pub update_interval: u64,
    /// Maximum allowed connections
    pub max_connections: usize,
    /// Whether to enable message compression
    pub enable_compression: bool,
    /// Whether authentication is required
    pub auth_required: bool,
}

impl Default for WebSocketConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8765,
            update_interval: 1000,
            max_connections: 100,
            enable_compression: false,
            auth_required: false,
        }
    }
}

/// Interface for WebSocket servers
#[async_trait]
pub trait WebSocketInterface: Send + Sync + Debug {
    /// Get a list of available components
    async fn get_available_components(&self) -> Result<Vec<String>>;
    
    /// Get data for a specific component
    async fn get_component_data(&self, component_id: &str) -> Result<Value>;
    
    /// Check the server's health status
    async fn get_health_status(&self) -> Result<Value>;
    
    /// Start the WebSocket server
    async fn start(&self) -> Result<()>;
    
    /// Stop the WebSocket server
    async fn stop(&self) -> Result<()>;
    
    /// Update component data and notify subscribers
    async fn update_component_data(&self, component_id: &str, data: Value) -> Result<()>;
}

/// Module for WebSocket connections
pub mod connection;

/// Module for WebSocket messages
pub mod messages;

/// Module for WebSocket protocol handling
pub mod protocol;

/// Module for WebSocket server implementation
pub mod server;

// Client message type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientMessage {
    pub action: String,
    pub data: Value,
}

// Server message type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerMessage {
    pub event: String,
    pub data: Value,
}

// Client action enum
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientAction {
    Subscribe,
    Unsubscribe,
    Query,
    Ping,
}

// Re-export the connection type
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ConnectionId(pub String);

// Websocket protocol trait
#[async_trait]
pub trait WebSocketProtocol: Send + Sync {
    async fn handle_message(&self, message: ClientMessage) -> Option<ServerMessage>;
}

// Error type for WebSocket operations
#[derive(Debug, thiserror::Error)]
pub enum WebSocketError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("WebSocket error: {0}")]
    WebSocketError(String),
    
    #[error("Protocol error: {0}")]
    ProtocolError(String),
}

// Re-export the core server type
pub use server::WebSocketServer;

/// Monitoring system WebSocket API documentation.
/// 
/// This module provides a WebSocket server for real-time monitoring data access.
/// It allows dashboard and UI applications to:
/// 
/// 1. Subscribe to component updates
/// 2. Query current component status
/// 3. List available components
/// 4. Check monitoring system health
/// 
/// The WebSocket server is configured using `WebSocketConfig` and implements
/// the `WebSocketInterface` trait.
/// 
/// For message format specifications, see the `protocol` module documentation.
pub struct WebSocketApiDocumentation;

// Define the message processor and handler traits
pub trait MessageProcessor: Send + Sync {
    fn process_message(&self, msg: &str) -> Result<Option<String>>;
}

#[async_trait]
pub trait WebSocketMessageHandler: Send + Sync {
    async fn handle_message(&self, message: ClientMessage) -> Option<ServerMessage>;
}

// Message type enum
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    Subscribe,
    Unsubscribe,
    Data,
    Query,
    Error,
}

/// Start the WebSocket server on the specified port
///
/// # Examples
///
/// ```rust,no_run
/// use squirrel_monitoring::websocket;
/// use std::net::SocketAddr;
///
/// #[tokio::main]
/// async fn main() {
///     let addr = "127.0.0.1:9000".parse::<SocketAddr>().unwrap();
///     let server = websocket::start_websocket_server(addr).await.unwrap();
/// }
/// ```
pub async fn start_websocket_server(addr: SocketAddr) -> Result<JoinHandle<()>> {
    let config = WebSocketConfig {
        host: addr.ip().to_string(),
        port: addr.port(),
        ..WebSocketConfig::default()
    };
    
    let server = WebSocketServer::new(config);
    
    let handle = tokio::spawn(async move {
        if let Err(e) = server.start().await {
            error!("WebSocket server error: {:?}", e);
        }
    });
    
    Ok(handle)
}

/// Starts a websocket server at the specified address with a custom message handler
#[allow(dead_code)]
pub async fn start_websocket_server_with_handler<H>(addr: SocketAddr, _handler: Arc<H>)
where
    H: WebSocketMessageHandler + 'static,
{
    let _ = start_websocket_server(addr).await;
} 