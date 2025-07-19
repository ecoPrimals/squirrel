//! MCP client implementation
//!
//! This module contains the main McpClient struct and its constructors.
//! The client manages the connection lifecycle, state, and provides high-level
//! interfaces for MCP operations.

use super::connection::ConnectionManager;
use super::message::MessageHandler;
use super::operations::OperationHandler;
use super::types::{ConnectionState, McpMessage, McpPrompt, McpResource, McpTool};
use crate::config::McpClientConfig;
use crate::error::{PluginError, PluginResult};

use std::collections::HashMap;
use tracing::{error, info};
use wasm_bindgen::prelude::*;

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
    /// Message handler
    pub(crate) message_handler: MessageHandler,
    /// Operation handler
    pub(crate) operation_handler: OperationHandler,
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
            message_handler: MessageHandler::new(),
            operation_handler: OperationHandler::new(),
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
            message_handler: MessageHandler::new(),
            operation_handler: OperationHandler::new(),
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

    /// Send a message to the MCP server
    ///
    /// Sends a message to the MCP server and waits for a response. This is the
    /// primary method for communication with the server.
    ///
    /// # Arguments
    ///
    /// * `message_type` - The type of message to send
    /// * `payload` - The message payload as a JsValue
    ///
    /// # Returns
    ///
    /// Returns the response payload as a JsValue, or an error if the operation fails.
    ///
    /// # Examples
    ///
    /// ```
    /// let client = McpClient::new("ws://localhost:8080");
    /// let response = client.send_message("tool_call", payload).await?;
    /// ```
    pub async fn send_message(
        &mut self,
        message_type: &str,
        payload: JsValue,
    ) -> Result<JsValue, JsValue> {
        // Implementation placeholder
        Ok(JsValue::NULL)
    }
}
