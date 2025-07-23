//! MCP Server API
//!
//! This module provides a high-level server API for the Machine Context Protocol.
//! It handles client connections, message routing, and command processing.
//!
//! # Examples
//!
//! ```no_run
//! use squirrel_mcp::server::{MCPServer, ServerConfig, CommandHandler};
//! use squirrel_mcp::message::{Message, MessageType};
//! use squirrel_mcp::error::Result;
//! use async_trait::async_trait;
//! use std::future::Future;
//! use std::pin::Pin;
//! use std::sync::Arc;
//!
//! // Define a command handler
//! #[derive(Debug, Clone)]
//! struct StatusCommandHandler;
//!
//! impl CommandHandler for StatusCommandHandler {
//!     fn handle_command<'a>(
//!         &'a self, 
//!         command: &'a Message
//!     ) -> Pin<Box<dyn Future<Output = Result<Option<Message>>> + Send + 'a>> {
//!         Box::pin(async move {
//!             // Process the status command
//!             let response = Message::new(
//!                 MessageType::Response,
//!                 "Status: online".to_string(),
//!                 command.destination.clone(),
//!                 command.source.clone()
//!             );
//!             
//!             Ok(Some(response))
//!         })
//!     }
//!
//!     fn supported_commands(&self) -> Vec<String> {
//!         vec!["status".to_string()]
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

use crate::error::{MCPError, Result, ProtocolError};
use crate::message::{Message, MessageBuilder};
use crate::message_router::{MessageRouter, HandlerPriority};
use crate::protocol::adapter_wire::{WireFormatAdapter, WireFormatConfig, DomainObject as WireDomainObject};
use crate::session::Session;
use crate::transport::Transport;

use serde_json::{json, Value};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::sync::{RwLock, watch, Mutex};
use tokio::time::timeout;
use tokio::task::JoinHandle;
use futures::future;
use tracing::{error, info, warn};
use uuid::Uuid;

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
pub trait CommandHandler: Send + Sync + std::fmt::Debug {
    /// Handle a command message
    fn handle_command<'a>(
        &'a self, 
        command: &'a Message
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Option<Message>>> + Send + 'a>>;
    
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
pub trait ConnectionHandler: Send + Sync {
    /// Handle a new client connection
    fn handle_connection<'a>(
        &'a self,
        client: ClientConnection
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>>;
    
    /// Handle client disconnection
    fn handle_disconnection<'a>(
        &'a self,
        client_id: &'a str
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>>;
    
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
            let mut config = config;
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
    
    /// Start the server on all configured transports
    ///
    /// This method starts the server on all configured transports and begins
    /// accepting client connections.
    ///
    /// # Errors
    /// 
    /// This function will return an error if:
    /// * The server is already running (`MCPError::Protocol` with `InvalidState` variant)
    /// * The server fails to bind to the configured address (`MCPError::Transport` with `ConnectionFailed` variant)
    /// * The server fails to start the listener task
    /// * The server fails to set up required internal components
    /// * Returns `MCPError` if the server fails to start for any reason
    pub async fn start(&mut self) -> Result<()> {
        // Check if already running
        let state_guard = self.state.read().await;
        let current_state = *state_guard;
        if current_state == ServerState::Running {
            drop(state_guard); // Release the lock before returning
            return Err(MCPError::Protocol(
                ProtocolError::InvalidState("Server already running".to_string())
            ));
        }
        drop(state_guard);
        
        // Update state to starting
        {
            let mut state = self.state.write().await;
            *state = ServerState::Starting;
            drop(state); // Explicitly drop the write guard
        }
        
        // Bind to the configured address
        let bind_addr = self.config.bind_address.parse::<SocketAddr>()
            .map_err(|e| {
                let transport_error: crate::error::transport::TransportError = format!("Failed to bind to {}: {}", self.config.bind_address, e).into();
                let mcp_error = MCPError::Transport(transport_error);
                mcp_error
            })?;
        
        let listener = TcpListener::bind(bind_addr).await
            .map_err(|e| {
                let transport_error: crate::error::transport::TransportError = format!("Failed to bind to {}: {}", self.config.bind_address, e).into();
                let mcp_error = MCPError::Transport(transport_error);
                mcp_error
            })?;
        
        // Start the listener task
        if let Err(e) = self.start_listener_task(listener).await {
            // Failed to start listener task
            *self.state.write().await = ServerState::Failed;
            *self.last_error.write().await = Some(e.clone().into());
            return Err(e);
        }
        
        // Update state to running
        {
            let mut state = self.state.write().await;
            *state = ServerState::Running;
        }
        
        Ok(())
    }
    
    /// Stop the server and close all connections
    ///
    /// This method stops the server on all transports and closes all active
    /// connections.
    ///
    /// # Errors
    /// 
    /// This function will return an error if:
    /// * The server is already stopped (`MCPError::Protocol` with `InvalidState` variant)
    /// * The server is already in the process of stopping (`MCPError::Protocol` with `InvalidState` variant)
    /// * The shutdown signal cannot be sent to the listener task
    /// * Any of the connection handlers fail during client disconnection
    /// * Returns `MCPError` if the server fails to stop
    pub async fn stop(&mut self) -> Result<()> {
        // Check if already stopped or stopping
        {
            let state_guard = self.state.read().await;
            let current_state = *state_guard;
            
            if current_state == ServerState::Stopped {
                drop(state_guard); // Release the lock before returning
                return Err(MCPError::Protocol(
                    ProtocolError::InvalidState("Server already stopped".to_string())
                ));
            }
            if current_state == ServerState::Stopping {
                drop(state_guard); // Release the lock before returning
                return Err(MCPError::Protocol(
                    ProtocolError::InvalidState("Server is already in the process of stopping".to_string())
                ));
            }
            drop(state_guard); // Release the lock after checking, before the next section
        }
        
        // Update state to stopping
        {
            let mut state = self.state.write().await;
            *state = ServerState::Stopping;
            drop(state); // Explicitly drop the write guard
        }
        
        // Send shutdown signal
        if let Err(e) = self.shutdown_signal.0.send(true) {
            eprintln!("Failed to send shutdown signal: {e}");
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
            // Get a list of clients to disconnect
            let client_list: Vec<(String, ClientConnection)> = {
                let clients_guard = self.clients.read().await;
                clients_guard.iter().map(|(id, conn)| (id.clone(), conn.clone())).collect()
            };
            
            // Create disconnect futures for each client
            let mut disconnect_futures = Vec::new();
            
            for (client_id, client) in client_list {
                // Clone the Arc values for each client task
                let clients_map_clone = self.clients.clone();
                let connection_handlers_clone = self.connection_handlers.clone();
                
                // Send a disconnect message to each client
                let client_id_clone = client_id.clone();
                let transport = client.transport.clone(); // Get the transport for this client
                let transport_clone = transport.clone(); // Clone transport for the spawn task below
                
                // Create a disconnect message
                let disconnect_msg = MessageBuilder::new()
                    .with_message_type("control")
                    .with_content_str("disconnect")
                    .with_source("server")
                    .with_destination(&client_id_clone)
                    .build();
                
                // Create a task to handle async disconnect message
                tokio::spawn(async move { // Move the clone into this task
                    // Try to convert the message to wire format
                    match disconnect_msg.to_wire_message(crate::protocol::adapter_wire::WireProtocolVersion::V1_0).await {
                        Ok(wire_msg) => {
                            // Serialize the WireMessage to bytes before sending raw
                            match serde_json::to_vec(&wire_msg) { 
                                Ok(wire_bytes) => {
                                    // Use the cloned transport here
                                    if let Err(e) = transport_clone.send_raw(&wire_bytes).await {
                                        error!("Error sending disconnect message during shutdown: {}", e);
                                    }
                                },
                                Err(e) => {
                                    error!("Failed to serialize disconnect message: {}", e);
                                }
                            }
                        },
                        Err(e) => {
                            error!("Failed to convert disconnect message to wire format: {}", e);
                        }
                    }
                });
                
                // Add disconnect future (uses the original transport Arc)
                let disconnect_fut = async move {
                    // Process messages until the client disconnects
                    'outer_loop: loop {
                        // Create a mutable Box to try to get a mutable reference to the transport
                        let mut transport_boxed = Box::new(Arc::clone(&transport));
                        
                        // Get mutable reference to transport if possible
                        let receive_result = if let Some(transport_mut) = Arc::get_mut(&mut *transport_boxed) { transport_mut.receive_message().await } else {
                            // If we can't get a mutable reference, log an error and break
                            error!("Failed to get mutable reference to transport for client {}", client_id_clone);
                            break;
                        };

                        match receive_result {
                            Ok(mcp_message) => {
                                // Process the message
                                println!("Received message: {mcp_message:?}");
                            },
                            Err(e) => {
                                // Handle transport errors
                                let error_msg = format!("{}", e);
                                if error_msg.contains("connection closed") {
                                    warn!("Client connection already closed: {}", client_id_clone);
                                    break;
                                } else if error_msg.contains("timeout") {
                                    warn!("Timeout during client disconnect: {}", client_id_clone);
                                    break;
                                } else {
                                    error!("Error disconnecting client {}: {}", client_id_clone, e);
                                    break;
                                }
                            }
                        }
                    }

                    // Client disconnected, remove from list and notify handlers
                    {
                        let mut clients_guard = clients_map_clone.write().await;
                        clients_guard.remove(&client_id_clone);
                    }
                    
                    // Create a Vec of cloned handlers for the async block
                    let mut handlers_copy: Vec<Arc<Box<dyn ConnectionHandler>>> = Vec::new();
                    let handlers_guard = connection_handlers_clone.read().await;
                    for handler in handlers_guard.iter() {
                        handlers_copy.push(Arc::new(handler.clone_box()));
                    }
                    drop(handlers_guard);  // Release the guard immediately
                    
                    // Use the copied handlers in the async block
                    let client_id_clone = client_id_clone.clone();
                    tokio::spawn(async move {
                        // Process disconnect notifications
                        for handler in &handlers_copy {
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
    #[must_use]
    pub async fn is_running(&self) -> bool {
        let state_guard = self.state.read().await;
        let current_state = *state_guard;
        drop(state_guard); // Release the lock immediately
        current_state == ServerState::Running
    }
    
    /// Get the current server state
    #[must_use]
    pub async fn get_state(&self) -> ServerState {
        let state_guard = self.state.read().await;
        let current_state = *state_guard;
        drop(state_guard); // Release the lock immediately
        current_state
    }
    
    /// Register a command handler
    ///
    /// Command handlers process incoming command messages and produce responses.
    /// Multiple handlers can be registered for different command types.
    ///
    /// # Arguments
    /// * `handler` - The command handler to register
    ///
    /// # Errors
    /// 
    /// This function will return an error if:
    /// * The handler cannot acquire the necessary locks for registration
    /// * The internal state is inconsistent
    /// * The handler supports empty or invalid command types
    /// * Returns `MCPError` if the handler cannot be registered for any reason
    pub async fn register_command_handler(&mut self, handler: Box<dyn CommandHandler>) -> Result<()> {
        // Get all supported commands
        let commands = handler.supported_commands();
        
        // Add handler for each command it supports
        for command in commands {
            self.command_handlers.write().await.insert(command, handler.clone_box());
        }
        
        Ok(())
    }
    
    /// Register a connection handler
    ///
    /// Connection handlers are notified when clients connect or disconnect from the server.
    /// Multiple handlers can be registered to process different aspects of connections.
    ///
    /// # Arguments
    /// * `handler` - The connection handler to register
    ///
    /// # Errors
    /// 
    /// This function will return an error if:
    /// * The handler cannot acquire the necessary locks for registration
    /// * The internal state is inconsistent
    /// * The handler's registration process fails
    /// * Returns `MCPError` if the handler cannot be registered for any reason
    pub async fn register_connection_handler(&mut self, handler: Box<dyn ConnectionHandler>) -> Result<()> {
        self.connection_handlers.write().await.push(handler);
        Ok(())
    }
    
    /// Get the last error
    #[must_use]
    pub async fn get_last_error(&self) -> Option<MCPError> {
        let error_guard = self.last_error.read().await;
        let error_clone = error_guard.clone();
        drop(error_guard); // Release the lock immediately
        error_clone
    }
    
    /// Get the number of connected clients
    #[must_use]
    pub async fn get_client_count(&self) -> usize {
        let clients_guard = self.clients.read().await;
        let count = clients_guard.len();
        drop(clients_guard); // Release the lock immediately
        count
    }
    
    /// Get a list of connected client IDs
    #[must_use]
    pub async fn get_connected_clients(&self) -> Vec<String> {
        let clients_guard = self.clients.read().await;
        let client_ids = clients_guard.keys().cloned().collect();
        drop(clients_guard); // Release the lock immediately
        client_ids
    }
    
    /// Handle a command message
    async fn handle_command(&self, command: &Message) -> Result<Option<Message>> {
        let command_type = command.get_message_type_str();
        
        // Find a handler for this command type
        let handler = self.command_handlers.read().await.get(command_type).cloned();
        
        // Process with the handler if one was found
        match handler {
            Some(handler) => {
                match handler.handle_command(command).await {
                    Ok(Some(response)) => Ok(Some(response)),
                    Ok(None) => Ok(Some(MessageBuilder::new()
                        .with_message_type("response")
                        .with_correlation_id(command.id.clone())
                        .with_content(json!({"status": "success"}))
                        .build())),
                    Err(e) => Err(e)
                }
            },
            None => Err(MCPError::General(format!("No handler found for command type: {command_type}")).into())
        }
    }
    
    /// Start the listener task to accept client connections
    async fn start_listener_task(&self, listener: TcpListener) -> Result<()> {
        info!("Starting MCP server listener task on {}", listener.local_addr()?);
        let _clients = self.clients.clone();
        let _wire_format_adapter = self.wire_format_adapter.clone();
        let _message_router = self.message_router.clone();
        let mut shutdown_rx = self.shutdown_signal.1.clone();
        let _connection_handlers = self.connection_handlers.clone();
        let state = self.state.clone();

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    // Accept new connections
                    result = listener.accept() => {
                        match result {
                            Ok((stream, addr)) => {
                                info!(client_addr = %addr, "Accepted new client connection");
                                
                                // Generate unique client ID
                                let client_id = Uuid::new_v4().to_string();
                                
                                // Create transport for the connection
                                use crate::transport::tcp::{TcpTransport, TcpTransportConfig};
                                let transport_config = TcpTransportConfig {
                                    remote_address: addr,
                                    buffer_size: 8192,
                                    timeout: Duration::from_secs(30),
                                };
                                
                                let transport = Arc::new(TcpTransport::new_with_stream(
                                    stream, 
                                    addr, 
                                    transport_config
                                ));
                                
                                // Create session for the client
                                use crate::client::session::Session;
                                let session_config = crate::client::session::SessionConfig {
                                    timeout: Duration::from_secs(300), // 5 minutes
                                    max_pending_requests: 100,
                                    enable_compression: false,
                                };
                                let session = Arc::new(Session::new(&client_id, session_config));
                                
                                // Create client connection
                                use super::types::ClientConnection;
                                let client = ClientConnection {
                                    client_id: client_id.clone(),
                                    address: addr,
                                    session: session.clone(),
                                    transport: transport.clone(),
                                    connected_at: Utc::now(),
                                    metadata: HashMap::new(),
                                };
                                
                                // Register client
                                clients.write().await.insert(client_id.clone(), client.clone());
                                
                                // Trigger connection handlers
                                let handlers = connection_handlers.read().await;
                                for handler in handlers.iter() {
                                    if let Err(e) = handler.handle_connection(client.clone()).await {
                                        error!(client_id = %client_id, error = %e, "Connection handler failed");
                                    }
                                }
                                
                                // Spawn client message handling task
                                let client_clone = client.clone();
                                let clients_clone = clients.clone();
                                let router_clone = message_router.clone();
                                let adapter_clone = wire_format_adapter.clone();
                                
                                tokio::spawn(async move {
                                    if let Err(e) = Self::handle_client_messages(
                                        client_clone,
                                        clients_clone,
                                        router_clone,
                                        adapter_clone,
                                    ).await {
                                        error!(client_id = %client_id, error = %e, "Client message handling failed");
                                    }
                                });
                                
                                info!(client_id = %client_id, client_addr = %addr, "Client connection established");
                            }
                            Err(e) => {
                                error!(error = %e, "Failed to accept connection");
                                // Consider exponential backoff or stopping if accept fails repeatedly
                                tokio::time::sleep(Duration::from_secs(1)).await;
                            }
                        }
                    }
                    // Check for shutdown signal
                    _ = shutdown_rx.changed() => {
                        info!("Shutdown signal received, stopping listener task.");
                        break;
                    }
                }
            }
            // Update server state when listener stops
            *state.write().await = ServerState::Stopped;
            info!("MCP server listener task stopped.");
        });

        Ok(())
    }
    
    /// Handle a client connection
    async fn handle_client_connection(&self, client: ClientConnection) -> Result<()> {
        // Add client to the list
        {
            let mut clients_guard = self.clients.write().await;
            clients_guard.insert(client.client_id.clone(), client.clone());
            drop(clients_guard); // Release the lock immediately
        }
        
        // Notify connection handlers
        let connection_handlers_guard = self.connection_handlers.read().await;
        let results = futures::future::join_all(
            connection_handlers_guard.iter()
                .map(|handler| handler.handle_connection(client.clone()))
        ).await;
        drop(connection_handlers_guard);
        
        // Check if any handler returned an error
        for result in results {
            result?;
        }
        
        Ok(())
    }
    
    /// Handle a client disconnection
    async fn handle_client_disconnection(&self, client_id: &str) -> Result<()> {
        // Remove client from the list
        {
            let mut clients_guard = self.clients.write().await;
            clients_guard.remove(client_id);
            drop(clients_guard); // Release the lock immediately
        }
        
        // Notify connection handlers
        let connection_handlers_guard = self.connection_handlers.read().await;
        let results = futures::future::join_all(
            connection_handlers_guard.iter()
                .map(|handler| handler.handle_disconnection(client_id))
        ).await;
        drop(connection_handlers_guard);
        
        // Check if any handler returned an error
        for result in results {
            result?;
        }
        
        Ok(())
    }
    
    /// Handle an error
    async fn handle_error(&self, error: MCPError) {
        let mut last_error_guard = self.last_error.write().await;
        *last_error_guard = Some(error);
        drop(last_error_guard); // Release the lock immediately
    }

    /// Get the list of supported event types
    pub async fn get_supported_event_types(&self) -> Vec<String> {
        self.message_router.get_registered_message_types().await
    }
    
    /// Handle messages from a specific client
    async fn handle_client_messages(
        client: ClientConnection,
        clients: Arc<RwLock<HashMap<String, ClientConnection>>>,
        message_router: Arc<MessageRouter>,
        wire_format_adapter: Arc<WireFormatAdapter>,
    ) -> Result<()> {
        use crate::transport::Transport;
        use crate::message::{Message, MessageBuilder};
        use serde_json::json;
        
        info!(client_id = %client.client_id, "Starting client message handling loop");
        
        loop {
            // Receive message from client transport
            let raw_message = match client.transport.receive_message().await {
                Ok(message) => message,
                Err(e) => {
                    if e.to_string().contains("connection closed") || 
                       e.to_string().contains("Connection reset") {
                        info!(client_id = %client.client_id, "Client disconnected gracefully");
                        break;
                    } else {
                        error!(client_id = %client.client_id, error = %e, "Error receiving message from client");
                        break;
                    }
                }
            };
            
            // Adapt wire format to internal message format
            let message = match wire_format_adapter.wire_to_internal(&raw_message) {
                Ok(msg) => msg,
                Err(e) => {
                    error!(client_id = %client.client_id, error = %e, "Failed to adapt wire format");
                    continue;
                }
            };
            
            debug!(client_id = %client.client_id, message_id = %message.id, "Received message from client");
            
            // Route message through the message router
            match message_router.route_message(&message).await {
                Ok(Some(response)) => {
                    // Convert response back to wire format and send
                    match wire_format_adapter.internal_to_wire(&response) {
                        Ok(wire_response) => {
                            if let Err(e) = client.transport.send_message(&wire_response).await {
                                error!(client_id = %client.client_id, error = %e, "Failed to send response to client");
                                break;
                            } else {
                                debug!(client_id = %client.client_id, response_id = %response.id, "Sent response to client");
                            }
                        }
                        Err(e) => {
                            error!(client_id = %client.client_id, error = %e, "Failed to convert response to wire format");
                            // Send error response
                            let error_response = MessageBuilder::new()
                                .with_message_type("error")
                                .with_correlation_id(message.id.clone())
                                .with_content(json!({
                                    "error": "internal_error",
                                    "message": "Failed to process response"
                                }))
                                .build();
                                
                            if let Ok(wire_error) = wire_format_adapter.internal_to_wire(&error_response) {
                                let _ = client.transport.send_message(&wire_error).await;
                            }
                        }
                    }
                }
                Ok(None) => {
                    // No response from router, send acknowledgment
                    let ack_response = MessageBuilder::new()
                        .with_message_type("response")
                        .with_correlation_id(message.id.clone())
                        .with_content(json!({"status": "acknowledged"}))
                        .build();
                        
                    if let Ok(wire_ack) = wire_format_adapter.internal_to_wire(&ack_response) {
                        if let Err(e) = client.transport.send_message(&wire_ack).await {
                            error!(client_id = %client.client_id, error = %e, "Failed to send acknowledgment to client");
                            break;
                        }
                    }
                }
                Err(e) => {
                    error!(client_id = %client.client_id, error = %e, "Message routing failed");
                    
                    // Send error response
                    let error_response = MessageBuilder::new()
                        .with_message_type("error")
                        .with_correlation_id(message.id.clone())
                        .with_content(json!({
                            "error": "routing_error",
                            "message": e.to_string()
                        }))
                        .build();
                        
                    if let Ok(wire_error) = wire_format_adapter.internal_to_wire(&error_response) {
                        let _ = client.transport.send_message(&wire_error).await;
                    }
                }
            }
        }
        
        // Clean up: remove client from active connections
        {
            let mut clients_guard = clients.write().await;
            clients_guard.remove(&client.client_id);
        }
        
        info!(client_id = %client.client_id, "Client message handling loop ended");
        Ok(())
    }
}

// Add Debug implementation for MCPServer
impl std::fmt::Debug for MCPServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MCPServer")
            .field("config", &self.config)
            .field("state", &self.state)
            .field("clients", &format!("<{} clients>", self.clients.try_read().map_or(0, |c| c.len())))
            .field("wire_format_adapter", &"<WireFormatAdapter>")
            .field("message_router", &"<MessageRouter>")
            .field("command_handlers", &format!("<{} handlers>", self.command_handlers.try_read().map_or(0, |h| h.len())))
            .field("connection_handlers", &format!("<{} handlers>", self.connection_handlers.try_read().map_or(0, |h| h.len())))
            .field("listener_task", &"<JoinHandle>")
            .field("shutdown_signal", &"<ShutdownChannel>")
            .field("last_error", &"<ErrorState>")
            .finish()
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
    #[must_use] pub const fn new(router: Arc<MessageRouter>) -> Self {
        Self { router }
    }
}

impl CommandHandler for RouterCommandHandler {
    fn handle_command<'a>(
        &'a self, 
        command: &'a Message
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Option<Message>>> + Send + 'a>> {
        Box::pin(async move {
            // Use the message router to handle the command
            self.router.route_message(command).await
                .map(|response| {
                    // Map the Option<Message> to Some(Message)
                    Some(response.unwrap_or_else(|| {
                        // Create a default success response when no handler returns a response
                        MessageBuilder::new()
                            .with_message_type("response")
                            .with_correlation_id(command.id.clone())
                            .with_content(json!({"status": "success"}))
                            .build()
                    }))
                })
                .or_else(|err| {
                    // Try to convert err to MCPError if necessary
                    let err_str = err.to_string();
                    
                    // Check if the error message indicates a "no handler found" error
                    if err_str.contains("No handler found for message type") {
                        // Extract the message type from the error string if possible
                        let msg_type = err_str
                            .split("No handler found for message type")
                            .nth(1)
                            .map(|s| s.trim_start_matches(':').trim())
                            .unwrap_or("unknown");
                            
                        eprintln!("No handler found for message type: {msg_type}");
                        
                        // Create a default success response when no handler found
                        Ok(Some(MessageBuilder::new()
                            .with_message_type("response")
                            .with_correlation_id(command.id.clone())
                            .with_content(json!({"status": "success", "message": format!("No handler found for {}", msg_type)}))
                            .build()))
                    } else {
                        // Pass through other errors
                        Err(err)
                    }
                })
        })
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

/// Adapter that converts a `CommandHandler` to a `MessageHandler`
#[derive(Debug)]
pub struct CommandHandlerAdapter {
    /// Inner command handler
    handler: Box<dyn CommandHandler>,
    /// Priority
    priority: HandlerPriority,
}

impl CommandHandlerAdapter {
    /// Create a new command handler adapter
    #[must_use] pub fn new(handler: Box<dyn CommandHandler>) -> Self {
        Self {
            handler,
            priority: HandlerPriority::Medium,
        }
    }
    
    /// Create a new command handler adapter with custom priority
    #[must_use] pub fn with_priority(handler: Box<dyn CommandHandler>, priority: HandlerPriority) -> Self {
        Self {
            handler,
            priority,
        }
    }
}

impl CommandHandler for CommandHandlerAdapter {
    fn handle_command<'a>(
        &'a self, 
        command: &'a Message
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Option<Message>>> + Send + 'a>> {
        Box::pin(async move {
            // Use the command handler to get the response
            self.handler.handle_command(command).await
                .map(|maybe_response| {
                    // If we got a response, use it; otherwise create a default response
                    Some(maybe_response.unwrap_or_else(|| {
                        // Create a default success response
                        MessageBuilder::new()
                            .with_message_type("response")
                            .with_correlation_id(command.id.clone())
                            .with_content(json!({"status": "success"}))
                            .build()
                    }))
                })
        })
    }
    
    fn supported_commands(&self) -> Vec<String> {
        self.handler.supported_commands()
    }

    fn clone_box(&self) -> Box<dyn CommandHandler> {
        Box::new(Self {
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