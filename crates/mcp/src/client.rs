//! MCP Client API
//!
//! This module provides a high-level client API for interacting with the Machine Context Protocol.
//! It handles connection management, message sending/receiving, and event subscription.
//!
//! # Examples
//!
//! ```
//! use mcp::client::{MCPClient, ClientConfig};
//! use mcp::message::MessageBuilder;
//! use serde_json::json;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Create client with default configuration
//!     let client = MCPClient::new(ClientConfig::default());
//!     
//!     // Connect to server
//!     client.connect().await?;
//!     
//!     // Send a command
//!     let response = client.send_command(
//!         "get_status",
//!         json!({
//!             "detail_level": "full"
//!         })
//!     ).await?;
//!     
//!     // Process response
//!     println!("Response: {:?}", response);
//!     
//!     // Disconnect
//!     client.disconnect().await?;
//!     
//!     Ok(())
//! }
//! ```

use crate::error::{MCPError, Result, ClientError};
use crate::transport::{Transport, TransportMetadata};
use crate::transport::memory::{MemoryTransport, MemoryTransportConfig};
use crate::transport::stdio::{StdioTransport, StdioConfig};
use crate::transport::tcp::{TcpTransport, TcpTransportConfig};
use crate::transport::websocket::{WebSocketTransport, WebSocketConfig};
use crate::message::{Message, MessageBuilder, MessageType, MessagePriority};
use crate::protocol::adapter_wire::{WireFormatAdapter, WireFormatConfig, DomainObject, WireMessage};
use crate::types::{MessageId, MCPMessage};
use crate::session::Session;
use crate::error::transport::TransportError;
use crate::error::ProtocolError;
use crate::session::SessionManager;
use crate::security::SecurityManager;

use async_trait::async_trait;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{RwLock, mpsc, watch, Mutex, broadcast, oneshot};
use tokio::time::timeout;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::future::Future;
use std::pin::Pin;
use futures::future::{AbortHandle, Abortable};

/// MCP Client configuration
#[derive(Clone)]
pub struct ClientConfig {
    /// Server address to connect to
    pub server_address: String,
    
    /// Connection timeout in milliseconds
    pub connection_timeout_ms: u64,
    
    /// Request timeout in milliseconds
    pub request_timeout_ms: u64,
    
    /// Maximum number of reconnect attempts
    pub max_reconnect_attempts: u32,
    
    /// Delay between reconnect attempts in milliseconds
    pub reconnect_delay_ms: u64,
    
    /// Keep-alive interval in milliseconds
    pub keep_alive_interval_ms: Option<u64>,
    
    /// Client ID (generated automatically if not provided)
    pub client_id: Option<String>,
    
    /// Authentication token
    pub auth_token: Option<String>,
    
    /// Custom transport implementation to use
    pub transport: Option<Arc<dyn Transport>>,
    
    /// Wire format adapter configuration
    pub wire_format_config: Option<WireFormatConfig>,
    
    /// Additional client parameters
    pub parameters: HashMap<String, Value>,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            server_address: "127.0.0.1:8080".to_string(),
            connection_timeout_ms: 5000,
            request_timeout_ms: 30000,
            max_reconnect_attempts: 3,
            reconnect_delay_ms: 1000,
            keep_alive_interval_ms: Some(30000),
            client_id: None,
            auth_token: None,
            transport: None,
            wire_format_config: None,
            parameters: HashMap::new(),
        }
    }
}

impl std::fmt::Debug for ClientConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClientConfig")
            .field("server_address", &self.server_address)
            .field("connection_timeout_ms", &self.connection_timeout_ms)
            .field("request_timeout_ms", &self.request_timeout_ms)
            .field("max_reconnect_attempts", &self.max_reconnect_attempts)
            .field("reconnect_delay_ms", &self.reconnect_delay_ms)
            .field("keep_alive_interval_ms", &self.keep_alive_interval_ms)
            .field("client_id", &self.client_id)
            .field("auth_token", &self.auth_token)
            .field("parameters", &self.parameters)
            .finish()
    }
}

/// MCP Client state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientState {
    /// Client is disconnected
    Disconnected,
    /// Client is connecting
    Connecting,
    /// Client is connected
    Connected,
    /// Client is disconnecting
    Disconnecting,
    /// Client connection failed
    Failed,
}

/// Client-side handler for processing event messages
#[async_trait]
pub trait EventHandler: Send + Sync {
    /// Handle an event message
    async fn handle_event(&self, event: &Message) -> Result<()>;
    
    /// Get the event types this handler can process
    fn supported_event_types(&self) -> Vec<String>;
}

/// MCP Client implementation
///
/// This client handles all communication with the MCP server, including:
/// - Connection management
/// - Request/response handling
/// - Event subscription and publishing
pub struct MCPClient {
    /// Configuration
    pub config: ClientConfig,
    
    /// Current transport
    transport: Arc<RwLock<Option<Arc<dyn Transport>>>>,
    
    /// Last error encountered
    last_error: Arc<RwLock<Option<MCPError>>>,
    
    /// Message channel sender
    message_tx: mpsc::Sender<Message>,
    
    /// Message channel receiver
    message_rx: Arc<RwLock<Option<mpsc::Receiver<Message>>>>,
    
    /// Event subscription channel
    event_channel: Arc<broadcast::Sender<Option<Message>>>,
    
    /// Map of pending request IDs to response channels
    pending_requests: Arc<RwLock<HashMap<String, oneshot::Sender<Result<Message>>>>>,
    
    /// Map of event topics to event handlers
    event_handlers: Arc<RwLock<HashMap<String, Vec<Arc<dyn EventHandler>>>>>,
    
    /// Message processing task handle (if started)
    message_task: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
    
    /// Reader task handle (if started)
    reader_task: Arc<Mutex<Option<AbortHandle>>>,
    
    /// Client state
    state: Arc<RwLock<ClientState>>,
    
    /// Current session information
    session: Arc<RwLock<Option<Session>>>,
}

impl MCPClient {
    /// Create a new MCPClient with the provided configuration
    pub fn new(config: ClientConfig) -> Self {
        let (tx, rx) = mpsc::channel(100);
        let (event_tx, _) = broadcast::channel(100);
        
        Self {
            config,
            transport: Arc::new(RwLock::new(None)),
            last_error: Arc::new(RwLock::new(None)),
            message_tx: tx,
            message_rx: Arc::new(RwLock::new(Some(rx))),
            event_channel: Arc::new(event_tx),
            pending_requests: Arc::new(RwLock::new(HashMap::new())),
            event_handlers: Arc::new(RwLock::new(HashMap::new())),
            message_task: Arc::new(RwLock::new(None)),
            reader_task: Arc::new(Mutex::new(None)),
            state: Arc::new(RwLock::new(ClientState::Disconnected)),
            session: Arc::new(RwLock::new(None)),
        }
    }
    
    /// Get the last error that occurred
    pub async fn get_last_error(&self) -> Option<MCPError> {
        self.last_error.read().await.clone()
    }
    
    /// Check if the client is connected
    pub async fn is_connected(&self) -> bool {
        self.transport.read().await.is_some()
    }
    
    /// Connect to the server
    pub async fn connect(&mut self) -> Result<()> {
        // If already connected, return
        if self.is_connected().await {
            return Ok(());
        }
        
        // Create and connect the transport
        let mut transport_guard = self.transport.write().await;
        *transport_guard = Some(create_transport_from_config(&self.config)?);
        
        // Start message processing task
        let mut message_task_guard = self.message_task.write().await;
        let mut message_rx_guard = self.message_rx.write().await;
        
        if message_task_guard.is_none() {
            if let Some(rx) = message_rx_guard.take() {
                // Clone references to pass to the task
                let pending_requests = self.pending_requests.clone();
                let event_handlers = self.event_handlers.clone();
                let event_channel = self.event_channel.clone();
                let last_error = self.last_error.clone();
                
                // Spawn a new task to process messages
                *message_task_guard = Some(tokio::spawn(async move {
                    process_messages(
                        rx,
                        pending_requests,
                        event_handlers,
                        event_channel,
                        last_error
                    ).await;
                }));
            }
        }
        
        // Start the reader task
        self.start_reader_task().await?;
        
        Ok(())
    }
    
    /// Disconnect from the MCP server
    pub async fn disconnect(&self) -> Result<()> {
        // Check if already disconnected or disconnecting
        {
            let state = self.state.read().await;
            match *state {
                ClientState::Disconnected => return Ok(()),
                ClientState::Disconnecting => return Err(MCPError::General("Already disconnecting".to_string())),
                _ => {}
            }
        }
        
        // Update state to disconnecting
        {
            let mut state = self.state.write().await;
            *state = ClientState::Disconnecting;
        }
        
        // Cancel reader task if running
        {
            let mut task_guard = self.reader_task.lock().await;
            if let Some(task) = task_guard.take() {
                task.abort();
            }
        }
        
        // Disconnect transport
        let mut disconnect_result = Ok(());
        {
            let transport_guard = self.transport.read().await;
            if let Some(transport) = &*transport_guard {
                disconnect_result = transport.disconnect().await.map_err(|e| {
                    MCPError::Transport(crate::error::transport::TransportError::ConnectionClosed(format!("Disconnect failed: {}", e)).into())
                });
            }
        }
        
        // Clear session
        {
            let mut session = self.session.write().await;
            *session = None;
        }
        
        // Clear pending requests
        {
            let mut pending_requests = self.pending_requests.write().await;
            pending_requests.clear();
        }
        
        // Update state to disconnected
        {
            let mut state = self.state.write().await;
            *state = ClientState::Disconnected;
        }
        
        disconnect_result
    }
    
    /// Get the current client state
    pub async fn get_state(&self) -> ClientState {
        match self.state.read().await {
            guard => *guard,
        }
    }
    
    /// Send a command to the server and wait for a response
    pub async fn send_command(&self, command: Message) -> Result<Message> {
        // Get the transport
        let transport_guard = self.transport.read().await;

        // Verify transport exists
        if transport_guard.is_none() {
            return Err(MCPError::Client(ClientError::NotConnected(
                "Client is not connected".to_string(),
            )));
        }

        let transport = transport_guard.as_ref().unwrap();
        
        // Create response channel
        let (response_tx, response_rx) = oneshot::channel();
        
        // Store sender in pending requests
        {
            let mut pending_requests = self.pending_requests.write().await;
            pending_requests.insert(command.id.clone(), response_tx);
        }
        
        // Convert Message to MCPMessage and send it
        let mcp_message = MCPMessage::try_from(&command)
            .map_err(|e| MCPError::Client(ClientError::SerializationError(
                format!("Failed to convert Message to MCPMessage: {}", e)
            )))?;
        
        let send_result = transport.send_message(mcp_message).await;
        
        if let Err(e) = send_result {
            // Clean up pending request
            let mut pending_requests = self.pending_requests.write().await;
            pending_requests.remove(&command.id);
            return Err(e.into());
        }
        
        // Wait for response with timeout
        match timeout(
            Duration::from_millis(self.config.request_timeout_ms),
            response_rx
        ).await {
            Ok(res) => match res {
                Ok(msg_result) => msg_result,
                Err(_) => Err(MCPError::Client(ClientError::ResponseChannelClosed(
                    format!("Response channel closed for command {}", command.id),
                ))),
            },
            Err(_) => Err(MCPError::Client(ClientError::Timeout(
                format!("Timeout waiting for response to command {}", command.id),
            ))),
        }
    }
    
    /// Send a command to the server with the given name and content, and wait for a response
    pub async fn send_command_with_content<T>(&self, command_name: &str, content: T) -> Result<Message> 
    where
        T: Into<serde_json::Value>
    {
        // Create a message with the command
        let mut message = Message::request(
            serde_json::to_string(&content.into()).unwrap_or_default(),
            self.config.client_id.clone().unwrap_or_else(|| "unknown".to_string()),
            "*".to_string(),
        );
        
        // Set the message type and other properties
        message.message_type = MessageType::Request;
        message.metadata.insert("command".to_string(), command_name.to_string());
        
        self.send_command(message).await
    }
    
    /// Send an event message to the server (no response expected)
    pub async fn send_event(&self, event: Message) -> Result<()> {
        let transport_guard = self.transport.read().await;

        // Verify transport exists
        if transport_guard.is_none() {
            return Err(MCPError::Client(ClientError::NotConnected(
                "Client is not connected".to_string(),
            )));
        }

        let transport = transport_guard.as_ref().unwrap();
        
        // Convert Message to MCPMessage and send it
        let mcp_message = MCPMessage::try_from(&event)
            .map_err(|e| MCPError::Client(ClientError::SerializationError(
                format!("Failed to convert Message to MCPMessage: {}", e)
            )))?;
        
        transport.send_message(mcp_message).await?;
        
        Ok(())
    }
    
    /// Also add a helper method to send events with content
    pub async fn send_event_with_content<T>(&self, event_name: &str, content: T) -> Result<()> 
    where
        T: Into<serde_json::Value>
    {
        // Create a message with the event
        let mut message = Message::notification(
            serde_json::to_string(&content.into()).unwrap_or_default(),
            self.config.client_id.clone().unwrap_or_else(|| "unknown".to_string()),
            "*".to_string(),
        );
        
        // Set the topic (which is the event name)
        message.topic = Some(event_name.to_string());
        
        self.send_event(message).await
    }
    
    /// Register an event handler for events
    pub async fn register_event_handler(&self, handler: Arc<dyn EventHandler>) -> Result<()> {
        let mut handlers = self.event_handlers.write().await;
        
        // Register handler for each supported event type
        for event_type in handler.supported_event_types() {
            if !handlers.contains_key(&event_type) {
                handlers.insert(event_type.clone(), Vec::new());
            }
            
            if let Some(handlers_for_type) = handlers.get_mut(&event_type) {
                handlers_for_type.push(handler.clone());
            }
        }
        
        Ok(())
    }
    
    /// Subscribe to events
    pub async fn subscribe_to_events(&self) -> broadcast::Receiver<Option<Message>> {
        self.event_channel.subscribe()
    }
    
    /// Start the reader task that continuously reads from the transport
    async fn start_reader_task(&self) -> Result<()> {
        // First check if the reader task is already running
        {
            let task_guard = self.reader_task.lock().await;
            if task_guard.is_some() {
                return Ok(());
            }
        }
        
        // Get the transport
        let transport_guard = self.transport.read().await;
        if transport_guard.is_none() {
            return Err(MCPError::Client(ClientError::NotConnected(
                "Client is not connected".to_string(),
            )));
        }
        
        let transport = transport_guard.as_ref().unwrap().clone();
        let message_tx = self.message_tx.clone();
        
        // Create abortable reader task
        let (abort_handle, abort_registration) = AbortHandle::new_pair();
        
        // Store the abort handle
        {
            let mut task_guard = self.reader_task.lock().await;
            *task_guard = Some(abort_handle);
        }
        
        // Spawn the reader task
        tokio::spawn(Abortable::new(async move {
            loop {
                match transport.receive_message().await {
                    Ok(msg) => {
                        // Convert to domain message and send to message channel
                        match Message::try_from(&msg) {
                            Ok(message) => {
                                if let Err(e) = message_tx.send(message).await {
                                    log::error!("Failed to send message to channel: {}", e);
                                    break;
                                }
                            },
                            Err(e) => {
                                log::error!("Failed to convert message: {}", e);
                            }
                        }
                    },
                    Err(e) => {
                        log::error!("Error receiving message: {}", e);
                        break;
                    }
                }
            }
        }, abort_registration));
        
        Ok(())
    }
    
    /// Process an incoming message
    async fn process_message(&self, message: Message) -> Result<()> {
        match message.message_type {
            MessageType::Response => {
                // Get the response channel from pending requests
                if let Some(in_reply_to) = &message.in_reply_to {
                    let sender = {
                        let mut pending = self.pending_requests.write().await;
                        pending.remove(in_reply_to)
                    };
                    
                    if let Some(tx) = sender {
                        let _ = tx.send(Ok(message));
                    }
                }
            },
            MessageType::Error => {
                // Check if it's a response to a command
                if let Some(in_reply_to) = &message.in_reply_to {
                    let sender = {
                        let mut pending = self.pending_requests.write().await;
                        pending.remove(in_reply_to)
                    };
                    
                    if let Some(tx) = sender {
                        let error = MCPError::from_message(&message);
                        let _ = tx.send(Err(error.clone()));
                        
                        // Store as last error
                        {
                            let mut last_error = self.last_error.write().await;
                            *last_error = Some(MCPError::Remote(message.content.clone()));
                        }
                        
                        return Ok(());
                    }
                }
                
                // If not a response, convert to error and store
                let error = MCPError::from_message(&message);
                let mut last_error = self.last_error.write().await;
                *last_error = Some(error);
            },
            MessageType::Notification => {
                // Notify event handlers
                if let Some(topic) = &message.topic {
                    let handler_list = {
                        let handlers = self.event_handlers.read().await;
                        handlers.get(topic).cloned()
                    };
                    
                    if let Some(handlers) = handler_list {
                        for handler in handlers {
                            if let Err(e) = handler.handle_event(&message).await {
                                log::error!("Error handling event: {}", e);
                            }
                        }
                    }
                }
                
                // Also broadcast the event to anyone listening
                let _ = self.event_channel.send(Some(message));
            },
            MessageType::Request => {
                // Handle requests that might need special handling
                log::debug!("Received request message: {}", message.id);
                // Forward to request handlers if implemented
            },
            MessageType::StreamChunk => {
                // Handle stream chunks - usually processed by a stream handler
                log::debug!("Received stream chunk: {}", message.id);
                // Process the stream data
            },
            MessageType::Control => {
                // Handle control messages for the protocol
                log::debug!("Received control message: {}", message.id);
                // Process protocol control messages
            },
            MessageType::System => {
                // Handle system messages
                log::debug!("Received system message: {}", message.id);
                // Process system messages
            },
            MessageType::Any => {
                // This is a wildcard type, clients should not typically receive this message type
                log::warn!("Received message with wildcard type Any: {}", message.id);
                // No specific handling
            },
        }
        
        Ok(())
    }
    
    /// Handle an error
    async fn handle_error(&self, error: MCPError) -> Result<()> {
        let mut last_error = self.last_error.write().await;
        *last_error = Some(error);
        Ok(())
    }
}

/// Composite event handler for combining multiple handlers
pub struct CompositeEventHandler {
    /// Mapping of event type to handlers
    handlers: HashMap<String, Vec<Arc<dyn EventHandler>>>,
}

impl CompositeEventHandler {
    /// Create a new CompositeEventHandler
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }
    
    /// Add a handler for a specific event type
    pub fn add_handler(&mut self, event_type: String, handler: Arc<dyn EventHandler>) {
        if !self.handlers.contains_key(&event_type) {
            self.handlers.insert(event_type.clone(), Vec::new());
        }
        
        if let Some(handlers) = self.handlers.get_mut(&event_type) {
            handlers.push(handler);
        }
    }
}

#[async_trait]
impl EventHandler for CompositeEventHandler {
    async fn handle_event(&self, event: &Message) -> Result<()> {
        // Find handlers that support this event type
        let event_type = event.message_type.to_string();
        if let Some(handlers) = self.handlers.get(&event_type) {
            for handler in handlers {
                handler.handle_event(event).await?;
            }
        }
        
        Ok(())
    }
    
    fn supported_event_types(&self) -> Vec<String> {
        self.handlers.keys().cloned().collect()
    }
}

/// Event handler that forwards events to a channel
struct ChannelEventHandler {
    event_type: String,
}

#[async_trait]
impl EventHandler for ChannelEventHandler {
    async fn handle_event(&self, _event: &Message) -> Result<()> {
        // We don't actually handle events internally
        Ok(())
    }
    
    fn supported_event_types(&self) -> Vec<String> {
        vec![self.event_type.clone()]
    }
}

/// Process incoming messages from the channel
async fn process_messages(
    mut rx: mpsc::Receiver<Message>,
    pending_requests: Arc<RwLock<HashMap<String, oneshot::Sender<Result<Message>>>>>,
    event_handlers: Arc<RwLock<HashMap<String, Vec<Arc<dyn EventHandler>>>>>,
    event_channel: Arc<broadcast::Sender<Option<Message>>>,
    last_error: Arc<RwLock<Option<MCPError>>>,
) {
    while let Some(message) = rx.recv().await {
        if let Err(e) = process_message_internal(
            message,
            pending_requests.clone(),
            event_handlers.clone(),
            event_channel.clone(),
            last_error.clone()
        ).await {
            log::error!("Error processing message: {}", e);
        }
    }
}

/// Internal function to process a message
async fn process_message_internal(
    message: Message,
    pending_requests: Arc<RwLock<HashMap<String, oneshot::Sender<Result<Message>>>>>,
    event_handlers: Arc<RwLock<HashMap<String, Vec<Arc<dyn EventHandler>>>>>,
    event_channel: Arc<broadcast::Sender<Option<Message>>>,
    last_error: Arc<RwLock<Option<MCPError>>>,
) -> Result<()> {
    match message.message_type {
        MessageType::Response => {
            // Get the in_reply_to value
            if let Some(ref in_reply_to) = message.in_reply_to {
                let sender = {
                    let mut pending = pending_requests.write().await;
                    pending.remove(in_reply_to)
                };
                
                if let Some(tx) = sender {
                    let _ = tx.send(Ok(message));
                }
            }
        },
        MessageType::Error => {
            // Check if it's a response to a command
            if let Some(ref in_reply_to) = message.in_reply_to {
                let sender = {
                    let mut pending = pending_requests.write().await;
                    pending.remove(in_reply_to)
                };
                
                if let Some(tx) = sender {
                    let error = MCPError::from_message(&message);
                    let _ = tx.send(Err(error.clone()));
                    
                    // Store as last error
                    {
                        let mut last_error = last_error.write().await;
                        *last_error = Some(MCPError::Remote(message.content.clone()));
                    }
                    
                    return Ok(());
                }
            }
            
            // If not a response, convert to error and store
            let error = MCPError::from_message(&message);
            let mut last_error_val = last_error.write().await;
            *last_error_val = Some(error);
        },
        MessageType::Notification => {
            // Notify event handlers
            if let Some(topic) = &message.topic {
                let handler_list = {
                    let handlers = event_handlers.read().await;
                    handlers.get(topic).cloned()
                };
                
                if let Some(handlers) = handler_list {
                    for handler in handlers {
                        if let Err(e) = handler.handle_event(&message).await {
                            log::error!("Error handling event: {}", e);
                        }
                    }
                }
            }
            
            // Also broadcast the event to anyone listening
            let _ = event_channel.send(Some(message));
        },
        MessageType::Request | MessageType::StreamChunk | MessageType::Control | MessageType::System | MessageType::Any => {
            // These message types are typically handled by the client directly
            // We just log them here
            log::debug!("Received {} message: {}", message.message_type, message.id);
        },
    }
    
    Ok(())
}

/// Create a transport instance from the client configuration
fn create_transport_from_config(config: &ClientConfig) -> Result<Arc<dyn Transport>> {
    // Default to TCP transport using the server address
    let tcp_config = TcpTransportConfig {
        remote_address: config.server_address.clone(),
        connection_timeout: config.connection_timeout_ms / 1000, // Convert to seconds
        keep_alive_interval: config.keep_alive_interval_ms.map(|ms| ms / 1000), // Convert to seconds
        max_reconnect_attempts: config.max_reconnect_attempts,
        reconnect_delay_ms: config.reconnect_delay_ms,
        ..Default::default()
    };
    
    let transport = TcpTransport::new(tcp_config);
    Ok(Arc::new(transport))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // Test client configuration
    #[test]
    fn test_client_config_default() {
        let config = ClientConfig::default();
        assert_eq!(config.server_address, "127.0.0.1:8080");
        assert_eq!(config.connection_timeout_ms, 5000);
        assert_eq!(config.request_timeout_ms, 30000);
        assert_eq!(config.max_reconnect_attempts, 3);
        assert_eq!(config.reconnect_delay_ms, 1000);
        assert_eq!(config.keep_alive_interval_ms, Some(30000));
    }
    
    // Additional tests will be added as implementation progresses
} 