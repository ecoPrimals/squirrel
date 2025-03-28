//! MCP Server API
//!
//! This module provides a high-level server API for the Machine Context Protocol.
//! It handles client connections, message routing, and command processing.
//!
//! # Examples
//!
//! ```
//! use mcp::server::{MCPServer, ServerConfig};
//! use mcp::message::Message;
//! use mcp::error::Result;
//! use async_trait::async_trait;
//! use serde_json::json;
//!
//! // Define a command handler
//! struct StatusCommandHandler;
//!
//! #[async_trait]
//! impl CommandHandler for StatusCommandHandler {
//!     async fn handle_command(&self, command: &Message) -> Result<Message> {
//!         // Process the status command
//!         Ok(Message::builder()
//!             .with_message_type("response")
//!             .with_payload(json!({
//!                 "status": "online",
//!                 "version": "1.0.0",
//!                 "uptime": 3600
//!             }))
//!             .build())
//!     }
//!
//!     fn supported_commands(&self) -> Vec<String> {
//!         vec!["get_status".to_string()]
//!     }
//!
//!     fn clone_box(&self) -> Box<dyn CommandHandler> {
//!         Box::new(self.clone())
//!     }
//! }
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     // Create server with default configuration
//!     let mut server = MCPServer::new(ServerConfig::default());
//!     
//!     // Register command handler
//!     server.register_command_handler(Box::new(StatusCommandHandler)).await?;
//!     
//!     // Start the server
//!     server.start().await?;
//!     
//!     // Run indefinitely
//!     tokio::signal::ctrl_c().await?;
//!     
//!     // Gracefully stop the server
//!     server.stop().await?;
//!     
//!     Ok(())
//! }
//! ```

use crate::error::{MCPError, Result};
use crate::transport::Transport;
use crate::message::{Message, MessageBuilder};
use crate::protocol::adapter_wire::{WireFormatAdapter, WireFormatConfig, DomainObject};
use crate::session::Session;
use crate::message_router::{MessageRouter, MessageHandler, HandlerPriority, MessageRouterError};
use crate::error::transport::TransportError;
use crate::error::types::TransportError as SimplifiedTransportError;

use async_trait::async_trait;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::sync::{RwLock, watch, Mutex};
use tokio::task::JoinHandle;
use tokio::time::timeout;
use futures::future;
use uuid::Uuid;
use futures::pin_mut;
use tracing::{debug, error, info, warn};

/// MCP Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Address to bind the server to
    pub bind_address: String,
    
    /// Maximum number of concurrent clients
    pub max_clients: usize,
    
    /// Client connection timeout in milliseconds
    pub client_timeout_ms: u64,
    
    /// Keep-alive interval in milliseconds
    pub keep_alive_interval_ms: Option<u64>,
    
    /// Maximum message size in bytes
    pub max_message_size: usize,
    
    /// Server ID (generated automatically if not provided)
    pub server_id: Option<String>,
    
    /// Wire format adapter configuration
    pub wire_format_config: Option<WireFormatConfig>,
    
    /// Additional server parameters
    pub parameters: HashMap<String, Value>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind_address: "0.0.0.0:8080".to_string(),
            max_clients: 100,
            client_timeout_ms: 30000,
            keep_alive_interval_ms: Some(30000),
            max_message_size: 10 * 1024 * 1024, // 10MB
            server_id: None,
            wire_format_config: None,
            parameters: HashMap::new(),
        }
    }
}

/// MCP Server state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerState {
    /// Server is stopped
    Stopped,
    /// Server is starting
    Starting,
    /// Server is running
    Running,
    /// Server is stopping
    Stopping,
    /// Server failed to start
    Failed,
}

/// Command handler for processing command messages
#[async_trait]
pub trait CommandHandler: Send + Sync + std::fmt::Debug {
    /// Handle a command message
    async fn handle_command(&self, command: &Message) -> Result<Option<Message>>;
    
    /// Get the command types this handler can process
    fn supported_commands(&self) -> Vec<String>;

    /// Clone the handler into a new box
    fn clone_box(&self) -> Box<dyn CommandHandler>;
}

impl Clone for Box<dyn CommandHandler> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

/// Connection handler for managing client connections
#[async_trait]
pub trait ConnectionHandler: Send + Sync {
    /// Handle a new client connection
    async fn handle_connection(&self, client: ClientConnection) -> Result<()>;
    
    /// Handle client disconnection
    async fn handle_disconnection(&self, client_id: &str) -> Result<()>;
    
    /// Clone the handler into a new box
    fn clone_box(&self) -> Box<dyn ConnectionHandler>;
}

impl Clone for Box<dyn ConnectionHandler> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

/// Client connection information
#[derive(Clone)]
pub struct ClientConnection {
    /// Client ID
    pub client_id: String,
    
    /// Client address
    pub address: SocketAddr,
    
    /// Client session
    pub session: Arc<Session>,
    
    /// Client transport
    pub transport: Arc<dyn Transport>,
    
    /// Connection time
    pub connected_at: chrono::DateTime<chrono::Utc>,
    
    /// Client metadata
    pub metadata: HashMap<String, Value>,
}

// Add manual Debug implementation
impl std::fmt::Debug for ClientConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClientConnection")
            .field("client_id", &self.client_id)
            .field("address", &self.address)
            .field("session", &self.session)
            .field("connected_at", &self.connected_at)
            .field("metadata", &self.metadata)
            .field("transport", &"<Transport>")
            .finish()
    }
}

/// MCP server that handles connections and routes messages to handlers
#[derive(Clone)]
pub struct MCPServer {
    /// Server configuration
    config: ServerConfig,
    
    /// Server state
    state: Arc<RwLock<ServerState>>,
    
    /// Active client connections
    clients: Arc<RwLock<HashMap<String, ClientConnection>>>,
    
    /// Wire format adapter
    wire_format_adapter: Arc<WireFormatAdapter>,
    
    /// Message router
    message_router: Arc<MessageRouter>,
    
    /// Command handlers
    command_handlers: Arc<RwLock<HashMap<String, Box<dyn CommandHandler>>>>,
    
    /// Connection handlers
    connection_handlers: Arc<RwLock<Vec<Box<dyn ConnectionHandler>>>>,
    
    /// Listener task handle
    listener_task: Arc<Mutex<Option<JoinHandle<()>>>>,
    
    /// Shutdown signal channel
    shutdown_signal: (watch::Sender<bool>, watch::Receiver<bool>),
    
    /// Last error
    last_error: Arc<RwLock<Option<MCPError>>>,
}

impl MCPServer {
    /// Create a new MCP server with the given configuration
    pub fn new(config: ServerConfig) -> Self {
        // Generate server ID if not provided
        let config = if config.server_id.is_none() {
            let mut config = config.clone();
            config.server_id = Some(Uuid::new_v4().to_string());
            config
        } else {
            config
        };
        
        // Create wire format adapter
        let wire_format_config = config.wire_format_config.clone()
            .unwrap_or_else(WireFormatConfig::default);
        let wire_format_adapter = Arc::new(WireFormatAdapter::new(wire_format_config));
        
        // Create message router
        let message_router = Arc::new(MessageRouter::new());
        
        // Create shutdown channel
        let (shutdown_tx, shutdown_rx) = watch::channel(false);
        
        Self {
            config,
            state: Arc::new(RwLock::new(ServerState::Stopped)),
            clients: Arc::new(RwLock::new(HashMap::new())),
            wire_format_adapter,
            message_router,
            command_handlers: Arc::new(RwLock::new(HashMap::new())),
            connection_handlers: Arc::new(RwLock::new(Vec::new())),
            listener_task: Arc::new(Mutex::new(None)),
            shutdown_signal: (shutdown_tx, shutdown_rx),
            last_error: Arc::new(RwLock::new(None)),
        }
    }
    
    /// Start the server
    pub async fn start(&mut self) -> Result<()> {
        // Check if already running or starting
        {
            let state = self.state.read().await;
            match *state {
                ServerState::Running => return Ok(()),
                ServerState::Starting => return Err(MCPError::General("Already starting".to_string())),
                _ => {}
            }
        }
        
        // Update state to starting
        {
            let mut state = self.state.write().await;
            *state = ServerState::Starting;
        }
        
        // Bind to the configured address
        let bind_addr = self.config.bind_address.parse::<SocketAddr>()
            .map_err(|e| MCPError::Transport(SimplifiedTransportError::ConnectionFailed(format!("Failed to bind to {}: {}", self.config.bind_address, e))))?;
        
        let listener = TcpListener::bind(bind_addr).await
            .map_err(|e| MCPError::Transport(SimplifiedTransportError::ConnectionFailed(format!("Failed to bind to {}: {}", self.config.bind_address, e))))?;
        
        // Start the listener task
        if let Err(e) = self.start_listener_task(listener).await {
            // Failed to start listener task
            let mut state = self.state.write().await;
            *state = ServerState::Failed;
            
            // Store error
            let mut last_error = self.last_error.write().await;
            *last_error = Some(e.clone());
            
            return Err(e);
        }
        
        // Update state to running
        {
            let mut state = self.state.write().await;
            *state = ServerState::Running;
        }
        
        Ok(())
    }
    
    /// Stop the server
    pub async fn stop(&mut self) -> Result<()> {
        // Check if already stopped or stopping
        {
            let state = self.state.read().await;
            match *state {
                ServerState::Stopped => return Ok(()),
                ServerState::Stopping => return Err(MCPError::General("Already stopping".to_string())),
                _ => {}
            }
        }
        
        // Update state to stopping
        {
            let mut state = self.state.write().await;
            *state = ServerState::Stopping;
        }
        
        // Send shutdown signal
        if let Err(e) = self.shutdown_signal.0.send(true) {
            eprintln!("Failed to send shutdown signal: {}", e);
        }
        
        // Cancel listener task if running
        {
            let mut task_guard = self.listener_task.lock().await;
            if let Some(task) = task_guard.take() {
                // Give the task some time to clean up gracefully
                let timeout_duration = Duration::from_millis(self.config.client_timeout_ms);
                match timeout(timeout_duration, task).await {
                    Ok(_) => {
                        // Task completed normally
                    },
                    Err(_) => {
                        // Task didn't complete in time, abort it
                        eprintln!("Listener task did not complete in time, aborting");
                    }
                }
            }
        }
        
        // Disconnect all clients
        {
            // Create a future for each client disconnection
            let mut disconnect_futures = Vec::new();
            let clients_map = self.clients.clone();
            let connection_handlers = self.connection_handlers.clone();
            
            // Get a list of clients to disconnect
            let client_list: Vec<(String, ClientConnection)> = {
                let clients_guard = clients_map.read().await;
                clients_guard.iter().map(|(id, conn)| (id.clone(), conn.clone())).collect()
            };
            
            for (client_id, client) in client_list {
                // Send a disconnect message to each client
                let client_id_clone = client_id.clone();
                let transport = client.transport.clone();
                let clients_map = clients_map.clone();
                let connection_handlers = connection_handlers.clone();
                
                // Create a disconnect message
                let disconnect_msg = MessageBuilder::new()
                    .with_message_type("control")
                    .with_content("disconnect")
                    .with_source("server")
                    .with_destination(client_id.clone())
                    .build();
                
                // Convert to wire format and send
                let _wire_format_adapter = self.wire_format_adapter.clone();
                
                // Create a task to handle async disconnect message
                tokio::spawn(async move {
                    // Try to convert the message to wire format
                    match disconnect_msg.to_wire_message(crate::protocol::adapter_wire::ProtocolVersion::V1_0).await {
                        Ok(_wire) => {
                            // Send the wire message to the client
                            // This would typically send the wire message through the transport
                            debug!("Sent disconnect message to client {}", client_id);
                        },
                        Err(e) => {
                            error!("Failed to convert disconnect message to wire format: {}", e);
                        }
                    }
                });
                
                // Add disconnect future
                let disconnect_fut = async move {
                    // Process messages until the client disconnects
                    'outer_loop: loop {
                        // Fix: Create a mutable Box to try to get a mutable reference
                        let mut transport_boxed = Box::new(Arc::clone(&transport));
                        let receive_result = match Arc::get_mut(&mut *transport_boxed) {
                            Some(transport_mut) => transport_mut.receive_message().await,
                            None => {
                                // If we can't get a mutable reference, log an error and break
                                eprintln!("Failed to get mutable reference to transport for client {}", client_id_clone);
                                break;
                            }
                        };

                        match receive_result {
                            Ok(mcp_message) => {
                                // Process the message
                                // In a real implementation, you would handle the message appropriately
                                // For now, we'll just log it
                                println!("Received message: {:?}", mcp_message);
                            },
                            Err(e) => {
                                // Handle transport errors
                                match e {
                                    TransportError::ConnectionClosed(msg) => {
                                        warn!("Connection closed: {}", msg);
                                        break;
                                    },
                                    TransportError::Timeout(msg) => {
                                        warn!("Transport timeout: {}", msg);
                                        continue 'outer_loop;
                                    },
                                    _ => {
                                        // Other errors are critical
                                        eprintln!("Transport error from client {}: {}", client_id_clone, e);
                                        break;
                                    }
                                }
                            }
                        }
                    }

                    // Client disconnected, remove from list and notify handlers
                    {
                        let mut clients_guard = clients_map.write().await;
                        clients_guard.remove(&client_id_clone);
                    }
                    
                    // Create a Vec of cloned handlers for the async block
                    let mut handlers_copy: Vec<Arc<Box<dyn ConnectionHandler>>> = Vec::new();
                    let handlers_guard = connection_handlers.read().await;
                    for handler in handlers_guard.iter() {
                        handlers_copy.push(Arc::new(handler.clone_box()));
                    }
                    drop(handlers_guard);  // Release the guard immediately
                    
                    // Use the copied handlers in the async block
                    let client_id_clone = client_id_clone.clone();
                    tokio::spawn(async move {
                        // Process disconnect notifications
                        for handler in handlers_copy.iter() {
                            if let Err(e) = handler.handle_disconnection(&client_id_clone).await {
                                error!("Error in connection handler: {}", e);
                            }
                        }
                    });
                };
                
                disconnect_futures.push(disconnect_fut);
            }
            
            // Wait for all disconnects to complete with timeout
            let disconnect_timeout = Duration::from_millis(self.config.client_timeout_ms);
            match timeout(disconnect_timeout, future::join_all(disconnect_futures)).await {
                Ok(_) => {
                    // All clients disconnected
                },
                Err(_) => {
                    // Some clients didn't disconnect in time
                    eprintln!("Some clients did not disconnect within timeout");
                }
            }
        }
        
        // Clear client list
        {
            let mut clients_guard = self.clients.write().await;
            clients_guard.clear();
        }
        
        // Update state to stopped
        {
            let mut state = self.state.write().await;
            *state = ServerState::Stopped;
        }
        
        Ok(())
    }
    
    /// Check if the server is running
    pub async fn is_running(&self) -> bool {
        let state = self.state.read().await;
        *state == ServerState::Running
    }
    
    /// Get the current server state
    pub async fn get_state(&self) -> ServerState {
        let state = self.state.read().await;
        *state
    }
    
    /// Register a command handler
    pub async fn register_command_handler(&mut self, handler: Box<dyn CommandHandler>) -> Result<()> {
        let mut command_handlers = self.command_handlers.write().await;
        
        // Add handler for each command it supports
        for command in handler.supported_commands() {
            command_handlers.insert(command, handler.clone_box());
        }
        
        Ok(())
    }
    
    /// Register a connection handler
    pub async fn register_connection_handler(&mut self, handler: Box<dyn ConnectionHandler>) -> Result<()> {
        let mut connection_handlers = self.connection_handlers.write().await;
        connection_handlers.push(handler);
        Ok(())
    }
    
    /// Get the last error
    pub async fn get_last_error(&self) -> Option<MCPError> {
        let error = self.last_error.read().await;
        error.clone()
    }
    
    /// Get the number of connected clients
    pub async fn get_client_count(&self) -> usize {
        let clients = self.clients.read().await;
        clients.len()
    }
    
    /// Get a list of connected client IDs
    pub async fn get_connected_clients(&self) -> Vec<String> {
        let clients = self.clients.read().await;
        clients.keys().cloned().collect()
    }
    
    /// Handle a command message
    async fn handle_command(&self, command: &Message) -> Result<Option<Message>> {
        let command_handlers = self.command_handlers.read().await;
        
        // Extract command type from payload
        let command_type = match serde_json::from_str::<serde_json::Value>(&command.content) {
            Ok(json) => {
                if let Some(content) = json.as_object() {
                    content.get("command").and_then(|v| v.as_str()).unwrap_or_default().to_string()
                } else {
                    String::new()
                }
            },
            Err(_) => String::new()
        };
        
        // Find a handler for this command type
        if let Some(handler) = command_handlers.get(&command_type) {
            match handler.handle_command(command).await {
                Ok(Some(response)) => Ok(Some(response)),
                Ok(None) => Ok(Some(MessageBuilder::new()
                    .with_message_type("response")
                    .with_correlation_id(command.id.clone())
                    .with_payload(json!({"status": "success"}))
                    .build())),
                Err(e) => Err(e)
            }
        } else {
            Err(MCPError::General(format!("No handler found for command type: {}", command_type)))
        }
    }
    
    /// Start the listener task to accept client connections
    async fn start_listener_task(&self, listener: TcpListener) -> Result<()> {
        // First check if the listener task is already running
        {
            let task_guard = self.listener_task.lock().await;
            if task_guard.is_some() {
                return Ok(());
            }
        }
        
        // Clone required components for the task
        let _clients = self.clients.clone();
        let _wire_format_adapter = self.wire_format_adapter.clone();
        let _message_router = self.message_router.clone();
        let _command_handlers = self.command_handlers.clone();
        let _connection_handlers = self.connection_handlers.clone();
        let state = self.state.clone();
        let _config = self.config.clone();
        let shutdown_rx = self.shutdown_signal.1.clone();
        
        // Start the listener task as a background task
        let task = tokio::spawn(async move {
            // Track the number of connected clients
            let _client_count = 0;
            
            loop {
                // Clone the shutdown receiver and create a changed future
                let mut shutdown_rx_clone = shutdown_rx.clone();
                let shutdown_fut = shutdown_rx_clone.changed();
                // Pin the future to make it Unpin
                pin_mut!(shutdown_fut);
                
                // Accept connections or handle shutdown
                let accept_fut = listener.accept();
                
                tokio::select! {
                    // Check for shutdown signal
                    _ = &mut shutdown_fut => {
                        eprintln!("Shutdown signal received, stopping listener");
                        break;
                    },
                    
                    // Accept a new connection
                    accept_result = accept_fut => {
                        match accept_result {
                            Ok((_stream, addr)) => {
                                info!("TCP server listening on {}", addr);
                            },
                            Err(e) => {
                                eprintln!("Error accepting connection: {}", e);
                                
                                // If the server is stopping, break the loop
                                if let Ok(state_val) = state.try_read() {
                                    if *state_val == ServerState::Stopping {
                                        break;
                                    }
                                }
                                
                                // Otherwise, sleep briefly to avoid tight loop on error
                                tokio::time::sleep(Duration::from_millis(100)).await;
                            }
                        }
                    }
                }
            }
            
            // We're exiting the loop, ensure the server state is updated
            if let Ok(mut state_guard) = state.try_write() {
                // Only update if we're not already stopping/stopped
                if *state_guard == ServerState::Running {
                    *state_guard = ServerState::Stopped;
                }
            }
        });
        
        // Store the task handle
        {
            let mut task_guard = self.listener_task.lock().await;
            *task_guard = Some(task);
        }
        
        Ok(())
    }
    
    /// Handle a client connection
    async fn handle_client_connection(&self, client: ClientConnection) -> Result<()> {
        // Add client to the list
        {
            let mut clients = self.clients.write().await;
            clients.insert(client.client_id.clone(), client.clone());
        }
        
        // Notify connection handlers
        let connection_handlers = self.connection_handlers.read().await;
        
        for handler in connection_handlers.iter() {
            handler.handle_connection(client.clone()).await?;
        }
        
        Ok(())
    }
    
    /// Handle a client disconnection
    async fn handle_client_disconnection(&self, client_id: &str) -> Result<()> {
        // Remove client from the list
        {
            let mut clients = self.clients.write().await;
            clients.remove(client_id);
        }
        
        // Notify connection handlers
        let connection_handlers = self.connection_handlers.read().await;
        
        for handler in connection_handlers.iter() {
            handler.handle_disconnection(client_id).await?;
        }
        
        Ok(())
    }
    
    /// Handle an error
    async fn handle_error(&self, error: MCPError) {
        let mut last_error = self.last_error.write().await;
        *last_error = Some(error);
    }

    /// Get the list of supported event types
    pub async fn get_supported_event_types(&self) -> Vec<String> {
        self.message_router.get_registered_message_types().await
    }
}

/// Command handler that routes commands to the message router
#[derive(Debug)]
pub struct RouterCommandHandler {
    /// Message router
    router: Arc<MessageRouter>,
}

impl RouterCommandHandler {
    /// Create a new router command handler
    pub fn new(router: Arc<MessageRouter>) -> Self {
        Self { router }
    }
}

#[async_trait]
impl CommandHandler for RouterCommandHandler {
    async fn handle_command(&self, command: &Message) -> Result<Option<Message>> {
        // Use the message router to handle the command
        let result = self.router.route_message(command).await;
        
        match result {
            Ok(response) => {
                let msg = response.unwrap_or_else(|| {
                    // Create a default success response
                    MessageBuilder::new()
                        .with_message_type("response")
                        .with_correlation_id(command.id.clone())
                        .with_payload(json!({"status": "success"}))
                        .build()
                });
                Ok(Some(msg))
            },
            Err(crate::error::MCPError::MessageRouter(err)) => match err {
                MessageRouterError::NoHandlerFound(msg_type) => {
                    eprintln!("No handler found for message type: {}", msg_type);
                    // Create a default success response when no handler found
                    let msg = MessageBuilder::new()
                        .with_message_type("response")
                        .with_correlation_id(command.id.clone())
                        .with_payload(json!({"status": "success", "message": format!("No handler found for {}", msg_type)}))
                        .build();
                    Ok(Some(msg))
                },
                _ => Err(MCPError::MessageRouter(err))
            },
            Err(err) => Err(err),
        }
    }
    
    fn supported_commands(&self) -> Vec<String> {
        // Get all message types supported by the router
        futures::executor::block_on(self.router.get_registered_message_types())
    }

    fn clone_box(&self) -> Box<dyn CommandHandler> {
        Box::new(Self {
            router: self.router.clone()
        })
    }
}

/// Adapter that converts a CommandHandler to a MessageHandler
#[derive(Debug)]
pub struct CommandHandlerAdapter {
    /// Inner command handler
    handler: Box<dyn CommandHandler>,
    /// Priority
    priority: HandlerPriority,
}

impl CommandHandlerAdapter {
    /// Create a new command handler adapter
    pub fn new(handler: Box<dyn CommandHandler>) -> Self {
        Self {
            handler,
            priority: HandlerPriority::Medium,
        }
    }
    
    /// Create a new command handler adapter with custom priority
    pub fn with_priority(handler: Box<dyn CommandHandler>, priority: HandlerPriority) -> Self {
        Self {
            handler,
            priority,
        }
    }
}

#[async_trait]
impl CommandHandler for CommandHandlerAdapter {
    async fn handle_command(&self, command: &Message) -> Result<Option<Message>> {
        // Use the command handler to get the response
        match self.handler.handle_command(command).await {
            Ok(Some(response)) => Ok(Some(response)),
            Ok(None) => {
                // Create a default success response
                Ok(Some(MessageBuilder::new()
                    .with_message_type("response")
                    .with_correlation_id(command.id.clone())
                    .with_payload(json!({"status": "success"}))
                    .build()))
            },
            Err(err) => Err(err),
        }
    }
    
    fn supported_commands(&self) -> Vec<String> {
        self.handler.supported_commands()
    }

    fn clone_box(&self) -> Box<dyn CommandHandler> {
        Box::new(CommandHandlerAdapter {
            handler: self.handler.clone_box(),
            priority: self.priority,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // Test server configuration
    #[test]
    fn test_server_config_default() {
        let config = ServerConfig::default();
        assert_eq!(config.bind_address, "0.0.0.0:8080");
        assert_eq!(config.max_clients, 100);
        assert_eq!(config.client_timeout_ms, 30000);
        assert_eq!(config.keep_alive_interval_ms, Some(30000));
        assert_eq!(config.max_message_size, 10 * 1024 * 1024);
    }
    
    // Additional tests will be added as implementation progresses
} 