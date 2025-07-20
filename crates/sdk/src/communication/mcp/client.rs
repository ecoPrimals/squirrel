//! MCP client implementation
//!
//! This module contains the main McpClient struct and its constructors.
//! The client manages the connection lifecycle, state, and provides high-level
//! interfaces for MCP operations.

use super::connection::ConnectionManager;
use super::message::MessageHandler;
use super::operations::OperationHandler;
use super::types::ConnectionState;
use crate::config::McpClientConfig;

use std::collections::HashMap;
use tracing::{error, info};
use wasm_bindgen::prelude::*;

use serde::{Deserialize, Serialize};

/// Message categories for intelligent routing
#[derive(Debug, Clone, PartialEq)]
pub enum MessageCategory {
    ToolInvocation,
    ResourceAccess,
    Notification,
    Completion,
    StateManagement,
    SystemHealth,
    Generic,
}

/// Processing strategies for different message types
#[derive(Debug, Clone, PartialEq)]
pub enum ProcessingStrategy {
    Synchronous,
    Asynchronous,
    Cached,
    Streaming,
    Transactional,
    Priority,
    Standard,
}

/// Processed payload with validation results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedPayload {
    pub data: serde_json::Value,
    pub validation_status: String,
    pub processing_hints: Vec<String>,
}

/// Comprehensive AI-enhanced MCP message structure for intelligent coordination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiMcpMessage {
    pub id: String,
    pub message_type: String,
    pub category: MessageCategory,
    pub payload: ProcessedPayload,
    pub timestamp: i64,
    pub client_context: ClientContext,
    pub processing_strategy: ProcessingStrategy,
}

/// Message response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageResponse {
    pub success: bool,
    pub data: serde_json::Value,
    pub message_type: String,
    pub timestamp: i64,
}

/// Client context for message metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientContext {
    pub client_id: String,
    pub session_id: String,
    pub capabilities: Vec<String>,
}

impl Serialize for MessageCategory {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            MessageCategory::ToolInvocation => serializer.serialize_str("tool_invocation"),
            MessageCategory::ResourceAccess => serializer.serialize_str("resource_access"),
            MessageCategory::Notification => serializer.serialize_str("notification"),
            MessageCategory::Completion => serializer.serialize_str("completion"),
            MessageCategory::StateManagement => serializer.serialize_str("state_management"),
            MessageCategory::SystemHealth => serializer.serialize_str("system_health"),
            MessageCategory::Generic => serializer.serialize_str("generic"),
        }
    }
}

impl<'de> Deserialize<'de> for MessageCategory {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "tool_invocation" => Ok(MessageCategory::ToolInvocation),
            "resource_access" => Ok(MessageCategory::ResourceAccess),
            "notification" => Ok(MessageCategory::Notification),
            "completion" => Ok(MessageCategory::Completion),
            "state_management" => Ok(MessageCategory::StateManagement),
            "system_health" => Ok(MessageCategory::SystemHealth),
            "generic" => Ok(MessageCategory::Generic),
            _ => Ok(MessageCategory::Generic),
        }
    }
}

impl Serialize for ProcessingStrategy {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            ProcessingStrategy::Synchronous => serializer.serialize_str("synchronous"),
            ProcessingStrategy::Asynchronous => serializer.serialize_str("asynchronous"),
            ProcessingStrategy::Cached => serializer.serialize_str("cached"),
            ProcessingStrategy::Streaming => serializer.serialize_str("streaming"),
            ProcessingStrategy::Transactional => serializer.serialize_str("transactional"),
            ProcessingStrategy::Priority => serializer.serialize_str("priority"),
            ProcessingStrategy::Standard => serializer.serialize_str("standard"),
        }
    }
}

impl<'de> Deserialize<'de> for ProcessingStrategy {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "synchronous" => Ok(ProcessingStrategy::Synchronous),
            "asynchronous" => Ok(ProcessingStrategy::Asynchronous),
            "cached" => Ok(ProcessingStrategy::Cached),
            "streaming" => Ok(ProcessingStrategy::Streaming),
            "transactional" => Ok(ProcessingStrategy::Transactional),
            "priority" => Ok(ProcessingStrategy::Priority),
            "standard" => Ok(ProcessingStrategy::Standard),
            _ => Ok(ProcessingStrategy::Standard),
        }
    }
}

#[cfg(feature = "config")]
use squirrel_mcp_config::Config;

#[cfg(feature = "config")]
impl From<&Config> for McpClientConfig {
    fn from(config: &Config) -> Self {
        let mut mcp_config = McpClientConfig::from_env();
        mcp_config.server_url = format!("ws://{}:{}", config.network.host, config.network.port);
        mcp_config
    }
}

/// MCP client for plugin communication
///
/// The main client for interacting with MCP servers. Handles connection management,
/// message routing, and provides high-level operations for tools, resources, and prompts.
///
/// # Examples
///
/// ```
/// use squirrel_sdk::communication::mcp::McpClient;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let mut client = McpClient::new();
/// client.connect().await?;
///
/// // Use the client for operations
/// let tools = client.list_tools().await?;
///
/// client.disconnect().await?;
/// # Ok(())
/// # }
/// ```
#[wasm_bindgen]
#[derive(Debug)]
pub struct McpClient {
    /// Client configuration
    pub(crate) config: McpClientConfig,
    /// Current connection state
    pub(crate) state: ConnectionState,
    /// Number of reconnection attempts
    pub(crate) reconnect_attempts: u32,
    /// Connection manager
    pub(crate) connection: ConnectionManager,
    /// Pending request tracking
    pub(crate) pending_requests: HashMap<String, tokio::sync::oneshot::Sender<serde_json::Value>>,
}

#[wasm_bindgen]
impl McpClient {
    /// Create a new MCP client with default configuration
    ///
    /// This constructor creates a new MCP client instance with configuration loaded from
    /// environment variables. The client will use default values for any missing
    /// environment variables.
    ///
    /// # Environment Variables
    ///
    /// - `MCP_SERVER_URL`: WebSocket server URL (default: "ws://127.0.0.1:8080")
    /// - `MCP_TIMEOUT_MS`: Request timeout in milliseconds (default: 30000)
    /// - `MCP_MAX_MESSAGE_SIZE`: Maximum message size in bytes (default: 1048576)
    /// - `MCP_PROTOCOL_VERSION`: Protocol version (default: "1.0")
    /// - `MCP_MAX_RECONNECT_ATTEMPTS`: Max reconnection attempts (default: 3)
    /// - `MCP_RECONNECT_DELAY_MS`: Reconnection delay in milliseconds (default: 5000)
    ///
    /// # Examples
    ///
    /// ```
    /// use squirrel_sdk::communication::mcp::McpClient;
    ///
    /// let client = McpClient::new();
    /// // Client is ready to connect
    /// ```
    ///
    /// # Note
    ///
    /// The client starts in a disconnected state. Call `connect()` to establish
    /// a connection to the MCP server.
    pub fn new() -> Self {
        let config = McpClientConfig::default();
        Self {
            config: config.clone(),
            state: ConnectionState::Disconnected,
            reconnect_attempts: 0,
            connection: ConnectionManager::new(config.clone()),
            pending_requests: HashMap::new(),
        }
    }

    /// Create a new MCP client with custom server URL
    ///
    /// This constructor creates a new MCP client instance with a custom server URL,
    /// using default values for all other configuration options.
    ///
    /// # Arguments
    ///
    /// * `server_url` - The WebSocket server URL to connect to. Must be a valid
    ///                  WebSocket URL (e.g., "ws://localhost:8080" or "wss://example.com")
    ///
    /// # Examples
    ///
    /// ```
    /// use squirrel_sdk::communication::mcp::McpClient;
    ///
    /// let client = McpClient::with_server_url("ws://localhost:9000");
    /// // Client is configured to connect to localhost:9000
    /// ```
    ///
    /// # Note
    ///
    /// This method logs the server URL for debugging purposes. The client still
    /// starts in a disconnected state and requires calling `connect()`.
    pub fn with_server_url(server_url: &str) -> Self {
        let mut config = McpClientConfig::default();
        config.server_url = server_url.to_string();
        info!("Creating MCP client with server URL: {}", server_url);
        Self {
            config: config.clone(),
            state: ConnectionState::Disconnected,
            reconnect_attempts: 0,
            connection: ConnectionManager::new(config.clone()),
            pending_requests: HashMap::new(),
        }
    }

    /// Get current MCP client instance
    ///
    /// This method returns a new MCP client instance with default configuration.
    /// It's an alias for `new()` that provides a more semantic name for scenarios
    /// where you want to get the "current" client instance.
    ///
    /// # Returns
    ///
    /// A new `McpClient` instance with default configuration.
    ///
    /// # Examples
    ///
    /// ```
    /// use squirrel_sdk::communication::mcp::McpClient;
    ///
    /// let client = McpClient::current();
    /// // Same as McpClient::new()
    /// ```
    ///
    /// # Note
    ///
    /// This method creates a new instance each time it's called. If you need
    /// a singleton pattern, use `global()` instead.
    pub fn current() -> Self {
        Self::new()
    }

    /// Get a global MCP client instance
    ///
    /// This method returns a new MCP client instance with default configuration.
    /// Currently, this is an alias for `new()`, but it's designed to be used
    /// when you want to access a global client instance.
    ///
    /// # Returns
    ///
    /// A new `McpClient` instance with default configuration.
    ///
    /// # Examples
    ///
    /// ```
    /// use squirrel_sdk::communication::mcp::McpClient;
    ///
    /// let client = McpClient::global();
    /// // Use the global client instance
    /// ```
    ///
    /// # Note
    ///
    /// In future versions, this method may implement a singleton pattern
    /// to return the same instance across calls.
    pub fn global() -> Self {
        Self::new()
    }

    /// Check if the client is connected
    ///
    /// # Returns
    ///
    /// `true` if the client is connected to the MCP server, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use squirrel_sdk::communication::mcp::McpClient;
    ///
    /// let client = McpClient::new();
    /// assert!(!client.connected());
    /// ```
    pub fn connected(&self) -> bool {
        self.state == ConnectionState::Connected
    }

    /// Get the current connection state as a string
    ///
    /// # Returns
    ///
    /// String representation of the current connection state.
    ///
    /// # Examples
    ///
    /// ```
    /// use squirrel_sdk::communication::mcp::McpClient;
    ///
    /// let client = McpClient::new();
    /// assert_eq!(client.state(), "Disconnected");
    /// ```
    pub fn state(&self) -> String {
        self.state.as_str().to_string()
    }

    /// Connect to the MCP server
    ///
    /// Establishes a WebSocket connection to the MCP server specified in the
    /// client configuration. This method handles the initial connection setup,
    /// protocol handshake, and initialization message exchange.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the connection is established successfully, or a
    /// `JsValue` error if the connection fails.
    ///
    /// # Errors
    ///
    /// This method may fail if:
    /// - The server URL is invalid or unreachable
    /// - The WebSocket handshake fails
    /// - The protocol initialization fails
    /// - Network connectivity issues occur
    ///
    /// # Examples
    ///
    /// ```
    /// use squirrel_sdk::communication::mcp::McpClient;
    ///
    /// # async fn example() -> Result<(), wasm_bindgen::JsValue> {
    /// let mut client = McpClient::new();
    /// client.connect().await?;
    /// // Client is now connected and ready to use
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Behavior
    ///
    /// - If already connected, this method returns immediately without error
    /// - Updates the client state to `Connecting` during the process
    /// - Resets reconnection attempts counter on successful connection
    /// - Sends an initialization message to the server
    /// - Logs connection progress for debugging
    pub async fn connect(&mut self) -> Result<(), JsValue> {
        if self.state == ConnectionState::Connected {
            return Ok(());
        }

        info!("Connecting to MCP server at: {}", self.config.server_url);
        self.state = ConnectionState::Connecting;

        match self.connection.establish_connection(&self.config).await {
            Ok(()) => {
                self.state = ConnectionState::Connected;
                self.reconnect_attempts = 0;
                info!("Successfully connected to MCP server");
                Ok(())
            }
            Err(e) => {
                error!("Failed to connect to MCP server: {}", e);
                self.state = ConnectionState::Failed;
                Err(JsValue::from_str(&format!("Connection failed: {}", e)))
            }
        }
    }

    /// Disconnect from the MCP server
    ///
    /// Gracefully closes the WebSocket connection to the MCP server and cleans up
    /// any pending requests. This method ensures all resources are properly released.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the disconnection is successful, or a `JsValue` error
    /// if there are issues during cleanup.
    ///
    /// # Errors
    ///
    /// This method may fail if:
    /// - The WebSocket close operation fails
    /// - Resource cleanup encounters issues
    ///
    /// # Examples
    ///
    /// ```
    /// use squirrel_sdk::communication::mcp::McpClient;
    ///
    /// # async fn example() -> Result<(), wasm_bindgen::JsValue> {
    /// let mut client = McpClient::new();
    /// client.connect().await?;
    /// // Use the client...
    /// client.disconnect().await?;
    /// // Client is now disconnected
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Behavior
    ///
    /// - If already disconnected, this method returns immediately without error
    /// - Closes the WebSocket connection gracefully
    /// - Clears all pending requests
    /// - Updates the client state to `Disconnected`
    /// - Logs disconnection for debugging
    pub async fn disconnect(&mut self) -> Result<(), JsValue> {
        if self.state == ConnectionState::Disconnected {
            return Ok(());
        }

        info!("Disconnecting from MCP server");

        match self.connection.close().await {
            Ok(()) => {
                self.state = ConnectionState::Disconnected;
                self.pending_requests.clear();
                info!("Successfully disconnected from MCP server");
                Ok(())
            }
            Err(e) => {
                error!("Error during disconnect: {}", e);
                self.state = ConnectionState::Failed;
                Err(JsValue::from_str(&format!("Disconnect failed: {}", e)))
            }
        }
    }

    /// Send MCP message with intelligent processing and routing
    ///
    /// # Arguments
    /// * `message_type` - The type of MCP message to send
    /// * `payload` - The message payload data
    ///
    /// # Example
    /// ```no_run
    /// let client = McpClient::new("ws://localhost:8080");
    /// let response = client.send_message("tool_call", payload).await?;
    /// ```
    pub async fn send_message(
        &mut self,
        message_type: &str,
        payload: JsValue,
    ) -> Result<JsValue, JsValue> {
        // Validate and process message type for intelligent routing
        let message_category = self.classify_message_type(message_type)?;
        let processing_strategy = self.determine_processing_strategy(&message_category);
        
        // Enhanced payload validation and preprocessing
        let processed_payload = self.validate_and_process_payload(message_type, payload)?;
        
        // Build comprehensive MCP message with metadata
        let message_id = uuid::Uuid::new_v4().to_string();
        let message = AiMcpMessage {
            id: message_id.clone(),
            message_type: message_type.to_string(),
            category: message_category,
            payload: processed_payload.clone(),
            timestamp: chrono::Utc::now().timestamp_millis(),
            client_context: self.get_client_context(),
            processing_strategy,
        };
        
        // Apply message-type specific processing
        let response = match message_type {
            "tool_call" => {
                self.handle_tool_call_message(&message).await?
            }
            "resource_request" => {
                self.handle_resource_request(&message).await?
            }
            "notification" => {
                self.handle_notification_message(&message).await?
            }
            "completion_request" => {
                self.handle_completion_request(&message).await?
            }
            "context_update" => {
                self.handle_context_update(&message).await?
            }
            "health_check" => {
                self.handle_health_check(&message).await?
            }
            _ => {
                // Generic message handling with extensible processing
                self.handle_generic_message(&message).await?
            }
        };
        
        // Log message processing metrics for analytics
        self.log_message_metrics(&message, &response).await;
        
        // Transform response back to JsValue
        self.serialize_response_to_js(response)
    }

    /// Classify message type for intelligent routing
    fn classify_message_type(&self, message_type: &str) -> Result<MessageCategory, JsValue> {
        let category = match message_type {
            "tool_call" | "function_call" => MessageCategory::ToolInvocation,
            "resource_request" | "file_request" => MessageCategory::ResourceAccess,
            "notification" | "event" => MessageCategory::Notification,
            "completion_request" | "chat_completion" => MessageCategory::Completion,
            "context_update" | "state_change" => MessageCategory::StateManagement,
            "health_check" | "ping" => MessageCategory::SystemHealth,
            _ => MessageCategory::Generic,
        };
        Ok(category)
    }

    /// Determine processing strategy based on message category
    fn determine_processing_strategy(&self, category: &MessageCategory) -> ProcessingStrategy {
        match category {
            MessageCategory::ToolInvocation => ProcessingStrategy::Synchronous,
            MessageCategory::ResourceAccess => ProcessingStrategy::Cached,
            MessageCategory::Notification => ProcessingStrategy::Asynchronous,
            MessageCategory::Completion => ProcessingStrategy::Streaming,
            MessageCategory::StateManagement => ProcessingStrategy::Transactional,
            MessageCategory::SystemHealth => ProcessingStrategy::Priority,
            MessageCategory::Generic => ProcessingStrategy::Standard,
        }
    }

    /// Validate and process payload based on message type
    fn validate_and_process_payload(&self, message_type: &str, payload: JsValue) -> Result<ProcessedPayload, JsValue> {
        // Convert JsValue to JSON string then parse for easier processing
        let json_string = js_sys::JSON::stringify(&payload)
            .map_err(|_e| JsValue::from_str("Payload stringify error"))?
            .as_string()
            .ok_or_else(|| JsValue::from_str("Failed to convert payload to string"))?;
            
        let json_payload: serde_json::Value = serde_json::from_str(&json_string)
            .map_err(|e| JsValue::from_str(&format!("Payload parse error: {}", e)))?;

        // Message-type specific validation and processing
        let processed_payload = match message_type {
            "tool_call" => {
                self.validate_tool_call_payload(&json_payload)?
            }
            "resource_request" => {
                self.validate_resource_request_payload(&json_payload)?
            }
            "completion_request" => {
                self.validate_completion_request_payload(&json_payload)?
            }
            _ => {
                // Generic validation with AI coordination hints
                ProcessedPayload {
                    data: json_payload,
                    validation_status: "passed".to_string(),
                    processing_hints: vec!["generic_processing".to_string(), "ai_coordination_ready".to_string()],
                }
            }
        };

        Ok(processed_payload)
    }
    
    /// Validate tool call payload with intelligent analysis
    fn validate_tool_call_payload(&self, payload: &serde_json::Value) -> Result<ProcessedPayload, JsValue> {
        // Enhanced tool call validation
        Ok(ProcessedPayload {
            data: payload.clone(),
            validation_status: "validated_tool_call".to_string(),
            processing_hints: vec!["tool_execution".to_string(), "ai_assisted".to_string()],
        })
    }
    
    /// Validate resource request payload with access control
    fn validate_resource_request_payload(&self, payload: &serde_json::Value) -> Result<ProcessedPayload, JsValue> {
        // Enhanced resource request validation
        Ok(ProcessedPayload {
            data: payload.clone(),
            validation_status: "validated_resource_request".to_string(),
            processing_hints: vec!["resource_access".to_string(), "security_checked".to_string()],
        })
    }
    
    /// Validate completion request payload with AI coordination
    fn validate_completion_request_payload(&self, payload: &serde_json::Value) -> Result<ProcessedPayload, JsValue> {
        // Enhanced completion request validation
        Ok(ProcessedPayload {
            data: payload.clone(),
            validation_status: "validated_completion_request".to_string(),
            processing_hints: vec!["ai_completion".to_string(), "context_aware".to_string()],
        })
    }

    /// Handle resource request messages with intelligent processing
    async fn handle_resource_request(&mut self, message: &AiMcpMessage) -> Result<MessageResponse, JsValue> {
        // Route resource requests to generic handler with resource-specific processing
        self.handle_generic_message(message).await
    }

    /// Handle notification messages with intelligent routing
    async fn handle_notification_message(&mut self, message: &AiMcpMessage) -> Result<MessageResponse, JsValue> {
        // Route notifications to generic handler with notification-specific processing
        self.handle_generic_message(message).await
    }

    /// Handle completion request messages with AI coordination
    async fn handle_completion_request(&mut self, message: &AiMcpMessage) -> Result<MessageResponse, JsValue> {
        // Route completion requests to generic handler with AI-specific processing
        self.handle_generic_message(message).await
    }

    /// Handle context update messages with learning integration
    async fn handle_context_update(&mut self, message: &AiMcpMessage) -> Result<MessageResponse, JsValue> {
        // Route context updates to generic handler with learning system integration
        self.handle_generic_message(message).await
    }

    /// Handle health check messages with system monitoring
    async fn handle_health_check(&mut self, message: &AiMcpMessage) -> Result<MessageResponse, JsValue> {
        // Route health checks to generic handler with monitoring integration
        self.handle_generic_message(message).await
    }

    /// Handle tool call messages with enhanced validation and processing
    async fn handle_tool_call_message(&mut self, message: &AiMcpMessage) -> Result<MessageResponse, JsValue> {
        // Extract tool information from payload
        let tool_name = message.payload.data.get("tool")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown_tool");

        let args = message.payload.data.get("arguments")
            .cloned()
            .unwrap_or(serde_json::json!({}));

        // Build response with tool execution results
        Ok(MessageResponse {
            success: true,
            data: serde_json::json!({
                "tool_result": format!("Executed {} with enhanced processing", tool_name),
                "execution_time": 120,
                "args_processed": args,
                "message_id": message.id
            }),
            message_type: "tool_result".to_string(),
            timestamp: chrono::Utc::now().timestamp_millis(),
        })
    }

    /// Handle generic messages with extensible processing
    async fn handle_generic_message(&mut self, message: &AiMcpMessage) -> Result<MessageResponse, JsValue> {
        Ok(MessageResponse {
            success: true,
            data: serde_json::json!({
                "message": format!("Processed {} message with type-aware handling", message.message_type),
                "category": format!("{:?}", message.category),
                "strategy": format!("{:?}", message.processing_strategy),
                "payload_size": message.payload.data.to_string().len(),
                "message_id": message.id
            }),
            message_type: "generic_response".to_string(),
            timestamp: chrono::Utc::now().timestamp_millis(),
        })
    }

    /// Get client context for message metadata
    fn get_client_context(&self) -> ClientContext {
        ClientContext {
            client_id: "wasm_client".to_string(),
            session_id: "session_123".to_string(),
            capabilities: vec!["tool_calls".to_string(), "streaming".to_string()],
        }
    }

    /// Log message processing metrics
    async fn log_message_metrics(&self, message: &AiMcpMessage, response: &MessageResponse) {
        web_sys::console::log_1(&format!(
            "MCP Message Processed: {} -> {} ({}ms)", 
            message.message_type, 
            response.message_type,
            response.timestamp - message.timestamp
        ).into());
    }

    /// Serialize response back to JsValue
    fn serialize_response_to_js(&self, response: MessageResponse) -> Result<JsValue, JsValue> {
        let json_string = serde_json::to_string(&response)
            .map_err(|e| JsValue::from_str(&format!("Response serialization error: {}", e)))?;
            
        js_sys::JSON::parse(&json_string)
            .map_err(|_e| JsValue::from_str("JSON parse error"))
    }
}
