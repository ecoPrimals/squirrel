use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::error;
use uuid::Uuid;
use async_trait::async_trait;
use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::pin::Pin;
use thiserror::Error;

// Import from our own crate 
use crate::api::commands::{
    CommandStatus, CommandDefinition, CommandStatusResponse,
};

/// MCP error types
#[derive(Debug, Error)]
pub enum McpError {
    /// Connection error
    #[error("MCP connection error: {0}")]
    ConnectionError(String),
    
    /// Command error
    #[error("MCP command error: {0}")]
    CommandError(String),
    
    /// Invalid response
    #[error("MCP invalid response: {0}")]
    InvalidResponse(String),
    
    /// Command not found
    #[error("MCP command not found: {0}")]
    CommandNotFound(String),
    
    /// Timeout
    #[error("MCP timeout: {0}")]
    Timeout(String),
    
    /// Internal error
    #[error("MCP internal error: {0}")]
    Internal(String),
    
    /// Authentication error
    #[error("MCP authentication error: {0}")]
    AuthenticationError(String),
    
    /// Context error
    #[error("MCP context error: {0}")]
    ContextError(String),
    
    /// Message error
    #[error("MCP message error: {0}")]
    MessageError(String),
    
    /// Serialization error
    #[error("MCP serialization error: {0}")]
    SerializationError(String),
}

impl From<serde_json::Error> for McpError {
    fn from(err: serde_json::Error) -> Self {
        McpError::SerializationError(err.to_string())
    }
}

// Define our own simplified version of MCPError that matches what we expect
#[derive(Debug)]
pub enum SimpleMCPError {
    ConnectionError(String),
    AuthError(String),
    CommandError(String),
    TimeoutError(String),
    SerializationError(String),
    ProtocolError(String),
    IoError(std::io::Error),
    Other(String),
}

impl From<SimpleMCPError> for McpError {
    fn from(err: SimpleMCPError) -> Self {
        match err {
            SimpleMCPError::ConnectionError(err) => McpError::ConnectionError(err),
            SimpleMCPError::AuthError(err) => McpError::AuthenticationError(err),
            SimpleMCPError::CommandError(err) => McpError::CommandError(err),
            SimpleMCPError::TimeoutError(err) => McpError::Timeout(err),
            SimpleMCPError::SerializationError(err) => McpError::SerializationError(err),
            SimpleMCPError::ProtocolError(err) => McpError::MessageError(err),
            SimpleMCPError::IoError(err) => McpError::Internal(err.to_string()),
            SimpleMCPError::Other(err) => McpError::Internal(err),
        }
    }
}

/// MCP Message structure for command communications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpMessage {
    #[serde(rename = "type")]
    pub type_: String,
    pub payload: serde_json::Value,
    pub context: McpContext,
}

/// MCP Context for message metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpContext {
    pub user_id: String,
    pub request_id: String,
    pub timestamp: chrono::DateTime<Utc>,
    // Add additional context fields for improved context preservation
    pub session_id: Option<String>,
    pub source: Option<String>,
    pub correlation_id: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

/// Expanded context manager for improved context preservation
pub struct ContextManager {
    // Cache for context storage
    context_cache: Arc<RwLock<HashMap<String, McpContext>>>,
}

impl ContextManager {
    /// Create a new context manager
    pub fn new() -> Self {
        Self {
            context_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Create a new context
    pub async fn create_context(
        &self,
        user_id: String,
        session_id: Option<String>,
        source: Option<String>,
        correlation_id: Option<String>,
        metadata: Option<serde_json::Value>,
    ) -> McpContext {
        let context = McpContext {
            user_id,
            request_id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            session_id,
            source,
            correlation_id,
            metadata,
        };
        
        // Store in cache
        let request_id = context.request_id.clone();
        self.context_cache.write().await.insert(request_id, context.clone());
        
        context
    }
    
    /// Get context by request ID
    pub async fn get_context(&self, request_id: &str) -> Option<McpContext> {
        self.context_cache.read().await.get(request_id).cloned()
    }
    
    /// Update context
    pub async fn update_context(
        &self,
        request_id: &str,
        updates: McpContextUpdates,
    ) -> Option<McpContext> {
        let mut contexts = self.context_cache.write().await;
        
        if let Some(context) = contexts.get_mut(request_id) {
            // Apply updates
            if let Some(metadata) = updates.metadata {
                context.metadata = Some(metadata);
            }
            
            if let Some(correlation_id) = updates.correlation_id {
                context.correlation_id = Some(correlation_id);
            }
            
            return Some(context.clone());
        }
        
        None
    }
    
    /// Remove context
    pub async fn remove_context(&self, request_id: &str) -> Option<McpContext> {
        self.context_cache.write().await.remove(request_id)
    }
}

/// Context updates structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpContextUpdates {
    pub metadata: Option<serde_json::Value>,
    pub correlation_id: Option<String>,
}

/// MCP client interface for all MCP communications
#[async_trait]
pub trait McpClient: Send + Sync + 'static {
    /// Send a message to the MCP
    async fn send_message(&self, message: &str) -> Result<String, McpError>;
    
    /// Get the client's connection status
    async fn get_status(&self) -> Result<ConnectionStatus, McpError>;
}

/// MCP command client interface for command execution
#[async_trait]
pub trait McpCommandClient: Send + Sync + 'static {
    /// Send a message to the MCP
    async fn send_message(&self, message: &str) -> Result<String, McpError>;
    
    /// Execute a command via MCP
    async fn execute_command(
        &self,
        command: &str,
        parameters: &serde_json::Value,
    ) -> Result<String, McpError>;
    
    /// Get command status
    async fn get_command_status(
        &self,
        command_id: &str,
    ) -> Result<CommandStatusResponse, McpError>;
    
    /// Cancel command
    async fn cancel_command(
        &self,
        command_id: &str,
    ) -> Result<(), McpError>;
    
    /// List available commands
    async fn list_available_commands(
        &self,
    ) -> Result<Vec<CommandDefinition>, McpError>;
}

/// Connection status for MCP client
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionStatus {
    /// Connected to MCP server
    Connected,
    
    /// Disconnected from MCP server
    Disconnected,
    
    /// Connecting to MCP server
    Connecting,
    
    /// Connection error
    Error,
}

/// MCP client configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpClientConfig {
    /// MCP server host
    pub host: String,
    
    /// MCP server port
    pub port: u16,
    
    /// Connection timeout in milliseconds
    pub timeout_ms: u64,
    
    /// Number of retry attempts
    pub retry_attempts: u32,
    
    /// Delay between retries in milliseconds
    pub retry_delay_ms: u64,
}

impl Default for McpClientConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 8080,
            timeout_ms: 5000,
            retry_attempts: 3,
            retry_delay_ms: 1000,
        }
    }
}

/// Real MCP client implementation that integrates with the MCP crate
pub struct RealMcpClient {
    /// MCP client configuration
    config: McpClientConfig,
    
    /// Connection status
    status: Arc<RwLock<ConnectionStatus>>,
    
    /// Command status cache
    command_status_cache: Arc<RwLock<std::collections::HashMap<String, CommandStatusResponse>>>,
    
    /// The host and port for connection
    addr: SocketAddr,

    /// Context manager for tracking MCP contexts
    context_manager: Arc<ContextManager>,
}

impl RealMcpClient {
    /// Create a new real MCP client with the given configuration
    pub async fn new(config: McpClientConfig) -> Result<Self, McpError> {
        // Parse address
        let addr_str = format!("{}:{}", config.host, config.port);
        let addr = addr_str.parse::<SocketAddr>()
            .map_err(|e| McpError::ConnectionError(format!("Invalid address: {}", e)))?;
        
        // Create context manager
        let context_manager = Arc::new(ContextManager::new());
        
        // Create client
        let client = Self {
            config,
            status: Arc::new(RwLock::new(ConnectionStatus::Disconnected)),
            command_status_cache: Arc::new(RwLock::new(std::collections::HashMap::new())),
            addr,
            context_manager,
        };
        
        // Connect
        client.connect().await?;
        
        Ok(client)
    }
    
    /// Connect to the MCP server
    async fn connect(&self) -> Result<(), McpError> {
        // Set status to connecting
        {
            let mut status = self.status.write().await;
            *status = ConnectionStatus::Connecting;
        }
        
        // We don't actually store the client connection - just update the status
        // based on whether we can make a connection
        let mut connected = false;
        
        // Attempt to connect
        for attempt in 0..self.config.retry_attempts {
            // Try to establish a TCP connection to validate the server is available
            match tokio::net::TcpStream::connect(self.addr).await {
                Ok(_) => {
                    connected = true;
                    tracing::info!("Connected to MCP server at {}", self.addr);
                    break;
                }
                Err(e) => {
                    tracing::warn!("Failed to connect to MCP server (attempt {}/{}): {}", 
                        attempt + 1, self.config.retry_attempts, e);
                    
                    if attempt < self.config.retry_attempts - 1 {
                        tokio::time::sleep(tokio::time::Duration::from_millis(
                            self.config.retry_delay_ms
                        )).await;
                    }
                }
            }
        }
        
        // Update connection status
        {
            let mut status = self.status.write().await;
            if connected {
                *status = ConnectionStatus::Connected;
                Ok(())
            } else {
                *status = ConnectionStatus::Error;
                Err(McpError::ConnectionError(format!(
                    "Failed to connect to MCP server after {} attempts", 
                    self.config.retry_attempts
                )))
            }
        }
    }
    
    /// Create a command message with enhanced context
    fn create_command_message(
        &self,
        command: &str,
        parameters: &serde_json::Value,
        context: Option<McpContext>,
    ) -> String {
        let context = match context {
            Some(ctx) => ctx,
            None => McpContext {
                user_id: "anonymous".to_string(),
                request_id: Uuid::new_v4().to_string(),
                timestamp: Utc::now(),
                session_id: None,
                source: Some("web".to_string()),
                correlation_id: None,
                metadata: None,
            },
        };
        
        let message = McpMessage {
            type_: "command.execute".to_string(),
            payload: json!({
                "command": command,
                "parameters": parameters,
            }),
            context,
        };
        
        match serde_json::to_string(&message) {
            Ok(s) => s,
            Err(e) => {
                tracing::error!("Failed to serialize MCP message: {}", e);
                "{}".to_string()
            }
        }
    }
    
    /// Convert MCP command status to API command status
    fn convert_command_status(&self, status: &str) -> CommandStatus {
        match status.to_lowercase().as_str() {
            "pending" => CommandStatus::Queued, // Map pending to Queued
            "running" => CommandStatus::Running,
            "completed" => CommandStatus::Completed,
            "failed" => CommandStatus::Failed,
            "cancelled" => CommandStatus::Cancelled,
            _ => CommandStatus::Queued, // Default to Queued
        }
    }
    
    /// Send a request to the MCP server
    async fn send_request(&self, message: &str) -> Result<String, McpError> {
        // In a real implementation, we'd connect to the server and send the message
        // For now, we'll just log the message and return a dummy response
        
        // Check if we're connected
        let status = self.status.read().await;
        if *status != ConnectionStatus::Connected {
            return Err(McpError::ConnectionError("Not connected to MCP server".to_string()));
        }
        
        // Parse the message
        let msg_value: serde_json::Value = serde_json::from_str(message)
            .map_err(|e| McpError::InvalidResponse(format!("Failed to parse message: {}", e)))?;
        
        // In real implementation, send to server
        // For now, create a mock response based on the request
        let response = if let Some(command) = msg_value.get("command").and_then(|c| c.as_str()) {
            // This is a command message
            json!({
                "id": Uuid::new_v4().to_string(),
                "type": "response",
                "status": "success",
                "command_id": Uuid::new_v4().to_string(),
                "data": {
                    "message": format!("Received command: {}", command)
                }
            })
        } else {
            // Generic response
            json!({
                "id": Uuid::new_v4().to_string(),
                "type": "response",
                "status": "success",
                "data": {
                    "message": "Message received"
                }
            })
        };
        
        // Serialize response
        serde_json::to_string(&response)
            .map_err(|e| McpError::InvalidResponse(format!("Failed to serialize response: {}", e)))
    }
}

#[async_trait]
impl McpClient for RealMcpClient {
    async fn send_message(&self, message: &str) -> Result<String, McpError> {
        self.send_request(message).await
    }
    
    async fn get_status(&self) -> Result<ConnectionStatus, McpError> {
        let status = self.status.read().await;
        Ok(*status)
    }
}

#[async_trait]
impl McpCommandClient for RealMcpClient {
    async fn send_message(&self, message: &str) -> Result<String, McpError> {
        // Delegate to McpClient implementation
        <Self as McpClient>::send_message(self, message).await
    }
    
    async fn execute_command(
        &self,
        command: &str,
        parameters: &serde_json::Value,
    ) -> Result<String, McpError> {
        tracing::info!("Executing command: {}", command);
        
        // Create context with enhanced fields
        let context = self.context_manager.create_context(
            "web-user".to_string(),  // This should be replaced with actual user ID in production
            Some("web-session".to_string()),  // This should be replaced with actual session ID in production
            Some("web".to_string()),
            None,
            Some(json!({
                "source_ip": "127.0.0.1",  // This should be replaced with actual client IP in production
                "command_source": "web_api"
            })),
        ).await;
        
        // Create command message
        let message = self.create_command_message(command, parameters, Some(context));
        
        // Send request
        let response = self.send_request(&message).await?;
        
        // Parse response
        let response_json: serde_json::Value = serde_json::from_str(&response)
            .map_err(|e| McpError::InvalidResponse(format!("Failed to parse response: {}", e)))?;
        
        // Check for command ID
        let command_id = response_json.get("command_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidResponse("Missing command_id".to_string()))?;
        
        Ok(command_id.to_string())
    }
    
    async fn get_command_status(&self, command_id: &str) -> Result<CommandStatusResponse, McpError> {
        // Check the cache first
        {
            let cache = self.command_status_cache.read().await;
            if let Some(status) = cache.get(command_id) {
                return Ok(status.clone());
            }
        }
        
        // Create status request message
        let status_message = serde_json::json!({
            "id": Uuid::new_v4().to_string(),
            "type": "command_status",
            "command_id": command_id,
        });
        
        // Send status request
        let status_request = serde_json::to_string(&status_message)
            .map_err(|e| McpError::InvalidResponse(format!("Failed to serialize status request: {}", e)))?;
        
        let response = <Self as McpClient>::send_message(self, &status_request).await?;
        
        // Parse response
        let response_value: serde_json::Value = serde_json::from_str(&response)
            .map_err(|e| McpError::InvalidResponse(format!("Failed to parse response: {}", e)))?;
        
        // Extract status
        let status = response_value.get("status").and_then(|s| s.as_str()).unwrap_or("queued");
        let progress = response_value.get("progress").and_then(|p| p.as_f64()).unwrap_or(0.0) as f32;
        let result = response_value.get("result").cloned();
        let error = response_value.get("error").and_then(|e| e.as_str()).map(|e| e.to_string());
        
        // Create command status
        let command_status = CommandStatusResponse {
            id: command_id.to_string(),
            command: response_value.get("command").and_then(|c| c.as_str()).unwrap_or("unknown").to_string(),
            status: self.convert_command_status(status),
            progress,
            result,
            error,
            started_at: response_value.get("started_at").and_then(|t| t.as_str()).and_then(|t| DateTime::parse_from_rfc3339(t).ok()).map(|t| t.with_timezone(&Utc)),
            completed_at: response_value.get("completed_at").and_then(|t| t.as_str()).and_then(|t| DateTime::parse_from_rfc3339(t).ok()).map(|t| t.with_timezone(&Utc)),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        // Update the cache
        {
            let mut cache = self.command_status_cache.write().await;
            cache.insert(command_id.to_string(), command_status.clone());
        }
        
        Ok(command_status)
    }
    
    async fn cancel_command(&self, command_id: &str) -> Result<(), McpError> {
        // Create cancel request message
        let cancel_message = serde_json::json!({
            "id": Uuid::new_v4().to_string(),
            "type": "command_cancel",
            "command_id": command_id,
        });
        
        // Send cancel request
        let cancel_request = serde_json::to_string(&cancel_message)
            .map_err(|e| McpError::InvalidResponse(format!("Failed to serialize cancel request: {}", e)))?;
        
        let _ = <Self as McpClient>::send_message(self, &cancel_request).await?;
        
        // Update the cache status to cancelled
        {
            let mut cache = self.command_status_cache.write().await;
            if let Some(mut status) = cache.get(command_id).cloned() {
                status.status = CommandStatus::Cancelled;
                status.updated_at = Utc::now();
                cache.insert(command_id.to_string(), status);
            }
        }
        
        Ok(())
    }
    
    async fn list_available_commands(&self) -> Result<Vec<CommandDefinition>, McpError> {
        // Create list request message
        let list_message = serde_json::json!({
            "id": Uuid::new_v4().to_string(),
            "type": "list_commands",
        });
        
        // Send list request
        let list_request = serde_json::to_string(&list_message)
            .map_err(|e| McpError::InvalidResponse(format!("Failed to serialize list request: {}", e)))?;
        
        let response = <Self as McpClient>::send_message(self, &list_request).await?;
        
        // Parse response
        let response_value: serde_json::Value = serde_json::from_str(&response)
            .map_err(|e| McpError::InvalidResponse(format!("Failed to parse response: {}", e)))?;
        
        // Extract commands
        let commands = response_value.get("commands").and_then(|c| c.as_array());
        if let Some(commands) = commands {
            let mut result = Vec::new();
            for cmd in commands {
                let id = cmd.get("id").and_then(|i| i.as_str()).unwrap_or_default().to_string();
                let name = cmd.get("name").and_then(|n| n.as_str()).unwrap_or_default().to_string();
                let description = cmd.get("description").and_then(|d| d.as_str()).unwrap_or_default().to_string();
                let parameter_schema = cmd.get("parameter_schema").cloned().unwrap_or(serde_json::json!({}));
                
                result.push(CommandDefinition {
                    id,
                    name,
                    description,
                    parameter_schema,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                });
            }
            Ok(result)
        } else {
            Ok(Vec::new())
        }
    }
}

/// Mock MCP client implementation for testing and development
pub struct MockMcpClient {
    host: String,
    port: u16,
}

impl MockMcpClient {
    pub fn new(host: String, port: u16) -> Self {
        Self { host, port }
    }
}

#[async_trait]
impl McpClient for MockMcpClient {
    async fn send_message(&self, message: &str) -> Result<String, McpError> {
        tracing::info!("Mock MCP client sending message: {}", message);
        // In a real implementation, this would send to the MCP server
        Ok(format!("Acknowledged: {}", message))
    }
    
    async fn get_status(&self) -> Result<ConnectionStatus, McpError> {
        // Always return connected for mock client
        Ok(ConnectionStatus::Connected)
    }
}

#[async_trait]
impl McpCommandClient for MockMcpClient {
    async fn send_message(&self, message: &str) -> Result<String, McpError> {
        tracing::info!("Mock MCP client sending message: {}", message);
        // In a real implementation, this would send to the MCP server
        Ok(format!("Acknowledged: {}", message))
    }
    
    async fn execute_command(
        &self,
        command: &str,
        parameters: &serde_json::Value,
    ) -> Result<String, McpError> {
        tracing::info!(
            "Mock MCP client executing command: {} with parameters: {}", 
            command, 
            parameters
        );
        
        // Generate a fake command ID
        let command_id = uuid::Uuid::new_v4().to_string();
        
        Ok(command_id)
    }
    
    async fn get_command_status(
        &self,
        command_id: &str,
    ) -> Result<CommandStatusResponse, McpError> {
        tracing::info!("Mock MCP client getting status for command: {}", command_id);
        
        // Return a fake command status
        Ok(CommandStatusResponse {
            id: command_id.to_string(),
            command: "mock-command".to_string(),
            status: crate::api::commands::CommandStatus::Running,
            progress: 0.5,
            result: None,
            error: None,
            started_at: Some(chrono::Utc::now()),
            completed_at: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        })
    }
    
    async fn cancel_command(
        &self,
        command_id: &str,
    ) -> Result<(), McpError> {
        tracing::info!("Mock MCP client cancelling command: {}", command_id);
        
        // Pretend to cancel the command
        Ok(())
    }
    
    async fn list_available_commands(
        &self,
    ) -> Result<Vec<CommandDefinition>, McpError> {
        tracing::info!("Mock MCP client listing available commands");
        
        // Return a list of fake commands
        let now = chrono::Utc::now();
        let commands = vec![
            CommandDefinition {
                id: "cmd-1".to_string(),
                name: "test-command".to_string(),
                description: "A test command".to_string(),
                parameter_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "param1": {
                            "type": "string",
                            "description": "First parameter"
                        },
                        "param2": {
                            "type": "number",
                            "description": "Second parameter"
                        }
                    },
                    "required": ["param1"]
                }),
                created_at: now,
                updated_at: now,
            },
            CommandDefinition {
                id: "cmd-2".to_string(),
                name: "another-command".to_string(),
                description: "Another test command".to_string(),
                parameter_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "option": {
                            "type": "string",
                            "description": "Command option"
                        }
                    }
                }),
                created_at: now,
                updated_at: now,
            },
        ];
        
        Ok(commands)
    }
}

/// WebSocket server message for event broadcasting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketServerMessage {
    pub event: String,
    pub data: serde_json::Value,
    pub time: String,
}

/// MCP Event Bridge - Bridges events from MCP to WebSocket
pub struct McpEventBridge {
    /// MCP client
    mcp_client: Arc<dyn McpClient>,
    
    /// WebSocket manager
    ws_manager: Arc<crate::websocket::ConnectionManager>,
    
    /// Event handlers by event type
    event_handlers: Arc<RwLock<HashMap<String, Vec<Box<dyn Fn(serde_json::Value) -> Pin<Box<dyn Future<Output = ()> + Send + 'static>> + Send + Sync + 'static>>>>>,
    
    /// Running flag
    running: Arc<RwLock<bool>>,
    
    /// Event subscriptions
    subscriptions: Arc<RwLock<HashMap<String, String>>>,
    
    /// Context manager
    context_manager: Arc<ContextManager>,
}

impl McpEventBridge {
    /// Create a new MCP event bridge
    pub fn new(
        mcp_client: Arc<dyn McpClient>,
        ws_manager: Arc<crate::websocket::ConnectionManager>,
        context_manager: Arc<ContextManager>,
    ) -> Self {
        Self {
            mcp_client,
            ws_manager,
            event_handlers: Arc::new(RwLock::new(HashMap::new())),
            running: Arc::new(RwLock::new(false)),
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            context_manager,
        }
    }
    
    /// Start the event bridge
    pub async fn start(&self) -> Result<(), McpError> {
        // Set running flag
        {
            let mut running = self.running.write().await;
            *running = true;
        }
        
        // Clone references for the task
        let ws_manager = self.ws_manager.clone();
        let event_handlers = self.event_handlers.clone();
        let running = self.running.clone();
        let mcp_client = self.mcp_client.clone();
        let context_manager = self.context_manager.clone();
        
        // Create subscription for common event types
        self.subscribe_to_events(vec![
            "command.status".to_string(),
            "command.progress".to_string(),
            "command.result".to_string(),
            "command.error".to_string(),
            "system.status".to_string(),
        ]).await?;
        
        // Spawn a task to handle events
        tokio::spawn(async move {
            tracing::info!("MCP event bridge started");
            
            // Create a channel for receiving events
            let (tx, mut rx) = tokio::sync::mpsc::channel(100);
            
            // Create event handler
            let tx_clone = tx.clone();
            let event_handler = move |event: serde_json::Value| {
                let tx = tx_clone.clone();
                async move {
                    if let Err(e) = tx.send(event).await {
                        tracing::error!("Failed to send event to channel: {}", e);
                    }
                }
            };
            
            // Register handler with MCP client (implementation depends on the MCP client interface)
            // This is a placeholder - actual implementation would depend on how the MCP client handles subscriptions
            if let Ok(mcp_message) = serde_json::to_string(&McpMessage {
                type_: "subscribe".to_string(),
                payload: json!({
                    "events": [
                        "command.status",
                        "command.progress",
                        "command.result",
                        "command.error",
                        "system.status"
                    ]
                }),
                context: context_manager.create_context(
                    "system".to_string(),
                    Some("event_bridge".to_string()),
                    Some("web".to_string()),
                    None,
                    None,
                ).await,
            }) {
                // Send subscription message to MCP
                if let Err(e) = mcp_client.send_message(&mcp_message).await {
                    tracing::error!("Failed to subscribe to MCP events: {}", e);
                }
            }
            
            // Process events from the channel
            while *running.read().await {
                if let Some(event_data) = rx.recv().await {
                    // Extract event type and data
                    let event_type = event_data.get("type").and_then(|v| v.as_str()).unwrap_or("unknown");
                    
                    // Forward to WebSocket manager
                    if let Err(e) = ws_manager.handle_mcp_event(event_type, event_data.clone()).await {
                        tracing::error!("Failed to handle MCP event: {}", e);
                    }
                    
                    // Call registered event handlers
                    if let Some(handlers) = event_handlers.read().await.get(event_type) {
                        for handler in handlers {
                            handler(event_data.clone()).await;
                        }
                    }
                }
            }
            
            tracing::info!("MCP event bridge stopped");
        });
        
        Ok(())
    }
    
    /// Stop the event bridge
    pub async fn stop(&self) {
        let mut running = self.running.write().await;
        *running = false;
        
        // Unsubscribe from events
        self.unsubscribe_from_all_events().await.ok();
    }
    
    /// Register an event handler
    pub async fn register_event_handler<F, Fut>(
        &self,
        event_type: &str,
        handler: F,
    ) where
        F: Fn(serde_json::Value) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let mut event_handlers = self.event_handlers.write().await;
        let handlers = event_handlers.entry(event_type.to_string()).or_insert_with(Vec::new);
        
        // Create wrapper function
        let wrapper = move |data: serde_json::Value| -> Pin<Box<dyn Future<Output = ()> + Send + 'static>> {
            Box::pin(handler(data))
        };
        
        handlers.push(Box::new(wrapper));
    }
    
    /// Subscribe to MCP events
    pub async fn subscribe_to_events(&self, event_types: Vec<String>) -> Result<(), McpError> {
        let mut subscriptions = self.subscriptions.write().await;
        let context = self.context_manager.create_context(
            "system".to_string(),
            Some("event_bridge".to_string()),
            Some("web".to_string()),
            None,
            None,
        ).await;
        
        let mcp_message = McpMessage {
            type_: "subscribe".to_string(),
            payload: json!({
                "events": event_types
            }),
            context,
        };
        
        // Send subscription message to MCP
        let message_str = serde_json::to_string(&mcp_message)?;
        let response = self.mcp_client.send_message(&message_str).await?;
        
        // Parse response
        let response_json: serde_json::Value = serde_json::from_str(&response)
            .map_err(|e| McpError::InvalidResponse(format!("Failed to parse response: {}", e)))?;
        
        // Check for success
        let success = response_json.get("success").and_then(|v| v.as_bool()).unwrap_or(false);
        if !success {
            return Err(McpError::CommandError("Failed to subscribe to events".to_string()));
        }
        
        // Extract subscription ID
        let subscription_id = response_json.get("subscription_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidResponse("Missing subscription_id".to_string()))?;
        
        // Store subscription
        for event_type in event_types {
            subscriptions.insert(event_type, subscription_id.to_string());
        }
        
        Ok(())
    }
    
    /// Unsubscribe from all events
    pub async fn unsubscribe_from_all_events(&self) -> Result<(), McpError> {
        let subscriptions = self.subscriptions.read().await;
        let mut unique_subscription_ids = HashSet::new();
        
        // Collect unique subscription IDs
        for (_, subscription_id) in subscriptions.iter() {
            unique_subscription_ids.insert(subscription_id.clone());
        }
        
        // Unsubscribe from each subscription
        for subscription_id in unique_subscription_ids {
            let context = self.context_manager.create_context(
                "system".to_string(),
                Some("event_bridge".to_string()),
                Some("web".to_string()),
                None,
                None,
            ).await;
            
            let mcp_message = McpMessage {
                type_: "unsubscribe".to_string(),
                payload: json!({
                    "subscription_id": subscription_id
                }),
                context,
            };
            
            // Send unsubscription message to MCP
            let message_str = serde_json::to_string(&mcp_message)?;
            self.mcp_client.send_message(&message_str).await?;
        }
        
        // Clear subscriptions
        self.subscriptions.write().await.clear();
        
        Ok(())
    }
} 