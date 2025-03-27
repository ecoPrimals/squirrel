use std::sync::Arc;
use std::net::SocketAddr;
use std::collections::HashMap;
use std::fmt::Debug;
use futures_util::{SinkExt, StreamExt, stream::SplitSink};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Mutex, RwLock};
use tokio_tungstenite::{accept_async, WebSocketStream};
use tungstenite::protocol::Message;
use serde_json::Value;
use tracing::{info, error, debug, warn};
use async_trait::async_trait;

use squirrel_core::error::{Result, SquirrelError};
use super::{WebSocketConfig, WebSocketInterface};
use super::connection::WebSocketConnection;

/// WebSocket writer for sending messages
type WebSocketWriter = SplitSink<WebSocketStream<TcpStream>, Message>;

/// WebSocket server implementation
///
/// This server provides a real-time data streaming API for UI clients.
/// It supports:
/// - Component data subscription via topics
/// - Real-time updates when data changes
/// - Health status monitoring
/// - Dynamic component discovery
///
/// ## Usage for UI & Dashboard Teams
///
/// ```rust
/// use squirrel_monitoring::websocket::{WebSocketConfig, WebSocketServer};
/// use serde_json::json;
///
/// async fn init_websocket() -> Result<(), Box<dyn std::error::Error>> {
///     // 1. Create a server with custom configuration
///     let config = WebSocketConfig {
///         host: "0.0.0.0".to_string(), // Listen on all interfaces
///         port: 8765,
///         ..Default::default()
///     };
///     let server = WebSocketServer::new(config);
///
///     // 2. Start the server
///     server.start().await?;
///
///     // 3. Update data for components - triggers subscriber notifications
///     server.update_component_data("cpu_usage", json!({
///         "value": 45.2,
///         "timestamp": chrono::Utc::now().timestamp(),
///     })).await?;
///
///     // 4. When done, stop the server 
///     // server.stop().await?;
///
///     Ok(())
/// }
/// ```
///
/// ## Client Connection Protocol
///
/// Clients connect via standard WebSocket:
/// ```
/// ws://hostname:port
/// ```
///
/// ### Subscription Message Format
/// ```json
/// {
///     "action": "subscribe",
///     "topic": "component_name"
/// }
/// ```
///
/// ### Unsubscribe Message Format
/// ```json
/// {
///     "action": "unsubscribe",
///     "topic": "component_name"
/// }
/// ```
///
/// ### Server Update Message Format
/// ```json
/// {
///     "topic": "component_name",
///     "payload": { /* component data */ },
///     "timestamp": 1624553600
/// }
/// ```
#[derive(Debug)]
pub struct WebSocketServer {
    config: WebSocketConfig,
    /// Active client connections
    active_connections: Arc<RwLock<HashMap<String, Arc<WebSocketConnection>>>>,
    /// Client message senders
    connection_writers: Arc<RwLock<HashMap<String, Arc<Mutex<WebSocketWriter>>>>>,
    /// Component data
    component_data: Arc<RwLock<HashMap<String, Value>>>,
    /// Server running status
    running: Arc<Mutex<bool>>,
}

impl WebSocketServer {
    pub fn new(config: WebSocketConfig) -> Self {
        Self {
            config,
            active_connections: Arc::new(RwLock::new(HashMap::new())),
            connection_writers: Arc::new(RwLock::new(HashMap::new())),
            component_data: Arc::new(RwLock::new(HashMap::new())),
            running: Arc::new(Mutex::new(false)),
        }
    }

    pub async fn is_running(&self) -> bool {
        *self.running.lock().await
    }

    pub async fn update_component_data(&self, id: &str, data: Value) -> Result<()> {
        let mut component_data = self.component_data.write().await;
        component_data.insert(id.to_string(), data.clone());
        
        // Broadcast update to subscribed clients
        let connections = self.active_connections.read().await;
        let writers = self.connection_writers.read().await;
        
        for (client_id, connection) in connections.iter() {
            // Check if the client is subscribed to this topic
            if connection.is_subscribed(id).await {
                let message = serde_json::json!({
                    "topic": id,
                    "payload": data,
                    "timestamp": chrono::Utc::now().timestamp(),
                });
                
                if let Some(writer) = writers.get(client_id) {
                    let mut writer = writer.lock().await;
                    if let Err(e) = writer.send(Message::Text(message.to_string())).await {
                        warn!("Error sending update to client {}: {}", client_id, e);
                    }
                }
            }
        }
        
        Ok(())
    }

    async fn send_message_to_client(&self, client_id: &str, message: Message) -> Result<()> {
        let writers = self.connection_writers.read().await;
        if let Some(writer) = writers.get(client_id) {
            let mut writer = writer.lock().await;
            writer.send(message).await
                .map_err(|e| SquirrelError::Generic(format!("Failed to send WebSocket message: {}", e)))?;
        }
        Ok(())
    }

    async fn handle_connection(&self, stream: TcpStream, addr: SocketAddr) -> Result<()> {
        let ws_stream = accept_async(stream).await
            .map_err(|e| SquirrelError::Generic(format!("Error accepting WebSocket connection: {}", e)))?;
        
        let client_id = format!("{}", addr);
        
        // Create connection
        let (write_half, read_half) = ws_stream.split();
        
        // Create a new connection with the client ID
        let connection = Arc::new(WebSocketConnection::new(client_id.clone()));
        let writer = Arc::new(Mutex::new(write_half));
        
        // Add to active connections
        {
            let mut connections = self.active_connections.write().await;
            connections.insert(client_id.clone(), Arc::clone(&connection));
            
            let mut writers = self.connection_writers.write().await;
            writers.insert(client_id.clone(), Arc::clone(&writer));
        }
        
        // Process incoming messages
        let connection_clone = Arc::clone(&connection);
        let server_clone = self.clone();
        
        tokio::spawn(async move {
            let mut read = read_half;
            
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        if let Ok(request) = serde_json::from_str::<Value>(&text) {
                            if let Some(action) = request.get("action").and_then(|a| a.as_str()) {
                                match action {
                                    "subscribe" => {
                                        if let Some(topic) = request.get("topic").and_then(|t| t.as_str()) {
                                            connection_clone.subscribe(topic.to_string()).await;
                                            
                                            // Send initial data if available
                                            let component_data = server_clone.component_data.read().await;
                                            if let Some(data) = component_data.get(topic) {
                                                let response = serde_json::json!({
                                                    "topic": topic,
                                                    "payload": data,
                                                    "timestamp": chrono::Utc::now().timestamp(),
                                                });
                                                
                                                // Send response using the client's writer
                                                if let Err(e) = server_clone.send_message_to_client(
                                                    &client_id, 
                                                    Message::Text(response.to_string())
                                                ).await {
                                                    warn!("Error sending initial data: {}", e);
                                                }
                                            }
                                        }
                                    },
                                    "unsubscribe" => {
                                        if let Some(topic) = request.get("topic").and_then(|t| t.as_str()) {
                                            connection_clone.unsubscribe(topic).await;
                                        }
                                    },
                                    _ => {
                                        debug!("Unknown WebSocket action: {}", action);
                                    }
                                }
                            }
                        }
                    },
                    Ok(Message::Close(_)) => {
                        debug!("WebSocket connection closed by client: {}", client_id);
                        break;
                    },
                    Ok(_) => { /* Ignore other message types */ },
                    Err(e) => {
                        error!("WebSocket error for client {}: {}", client_id, e);
                        break;
                    }
                }
            }
            
            // Remove from active connections
            {
                let mut connections = server_clone.active_connections.write().await;
                connections.remove(&client_id);
                
                let mut writers = server_clone.connection_writers.write().await;
                writers.remove(&client_id);
            }
            
            debug!("WebSocket connection closed: {}", client_id);
        });
        
        Ok(())
    }

    pub async fn start(&self) -> Result<()> {
        let mut running = self.running.lock().await;
        if *running {
            return Err(SquirrelError::Generic("WebSocket server is already running".to_string()));
        }

        *running = true;
        
        let address = format!("{}:{}", self.config.host, self.config.port);
        let addr: SocketAddr = address.parse()
            .map_err(|e| SquirrelError::Generic(format!("Invalid address {}: {}", address, e)))?;
        
        let server = self.clone();
        tokio::spawn(async move {
            match TcpListener::bind(&addr).await {
                Ok(listener) => {
                    info!("WebSocket server listening on {}", addr);
                    
                    while server.is_running().await {
                        match listener.accept().await {
                            Ok((stream, addr)) => {
                                debug!("New WebSocket connection from {}", addr);
                                let server_clone = server.clone();
                                
                                tokio::spawn(async move {
                                    if let Err(e) = server_clone.handle_connection(stream, addr).await {
                                        error!("Error handling WebSocket connection: {}", e);
                                    }
                                });
                            }
                            Err(e) => {
                                error!("Error accepting WebSocket connection: {}", e);
                                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to bind WebSocket server to {}: {}", addr, e);
                }
            }
        });

        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        let mut running = self.running.lock().await;
        if !*running {
            return Ok(()); // Already stopped
        }

        *running = false;
        
        // Close all active connections
        let writers = self.connection_writers.read().await;
        for (client_id, writer) in writers.iter() {
            let mut writer = writer.lock().await;
            if let Err(e) = writer.send(Message::Close(None)).await {
                warn!("Error closing WebSocket connection {}: {}", client_id, e);
            }
        }
        
        // Clear connections
        {
            let mut connections = self.active_connections.write().await;
            connections.clear();
            
            let mut writers = self.connection_writers.write().await;
            writers.clear();
        }
        
        info!("WebSocket server stopped");
        
        Ok(())
    }

    pub fn config(&self) -> &WebSocketConfig {
        &self.config
    }
}

impl Clone for WebSocketServer {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            active_connections: Arc::clone(&self.active_connections),
            connection_writers: Arc::clone(&self.connection_writers),
            component_data: Arc::clone(&self.component_data),
            running: Arc::clone(&self.running),
        }
    }
}

#[async_trait]
impl WebSocketInterface for WebSocketServer {
    async fn get_available_components(&self) -> Result<Vec<String>> {
        let data = self.component_data.read().await;
        Ok(data.keys().cloned().collect())
    }
    
    async fn get_component_data(&self, component_id: &str) -> Result<Value> {
        let data = self.component_data.read().await;
        match data.get(component_id) {
            Some(value) => Ok(value.clone()),
            None => Err(SquirrelError::Generic(format!("Component data not found for: {}", component_id)))
        }
    }
    
    async fn get_health_status(&self) -> Result<Value> {
        let is_running = self.is_running().await;
        let connection_count = self.active_connections.read().await.len();
        
        Ok(serde_json::json!({
            "running": is_running,
            "connection_count": connection_count,
            "uptime": "not implemented", // Would be implemented in a real server
            "status": if is_running { "healthy" } else { "stopped" }
        }))
    }
    
    async fn start(&self) -> Result<()> {
        WebSocketServer::start(self).await
    }
    
    async fn stop(&self) -> Result<()> {
        WebSocketServer::stop(self).await
    }
    
    async fn update_component_data(&self, component_id: &str, data: Value) -> Result<()> {
        WebSocketServer::update_component_data(self, component_id, data).await
    }
} 