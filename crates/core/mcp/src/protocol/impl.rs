//! Implementation of the MCP protocol.
//!
//! This module contains the core implementation of the Machine Context Protocol (MCP),
//! providing functionality for message handling, validation, routing, state management,
//! and handler registration.
//!
//! The protocol implementation maintains its own state, configuration, and registered
//! handlers for different message types. It provides methods for validating messages,
//! routing them to appropriate handlers, and managing the protocol state.
//!
//! # Key Components
//!
//! - `MCPProtocolBase`: The main protocol implementation
//! - Handler registration and management
//! - Message validation
//! - Message routing
//! - State management
//!
//! # Examples
//!
//! ```
//! use mcp::protocol::{MCPProtocolBase, ProtocolConfig, CommandHandler};
//! use mcp::types::{MCPMessage, MessageId, MessageType, MCPResponse, ResponseStatus};
//! use mcp::error::Result;
//! use async_trait::async_trait;
//! use serde_json::json;
//!
//! // Create a custom handler
//! #[derive(Debug)]
//! struct EchoHandler;
//!
//! #[async_trait]
//! impl CommandHandler for EchoHandler {
//!     async fn handle(&self, message: &MCPMessage) -> Result<MCPResponse> {
//!         Ok(MCPResponse {
//!             protocol_version: "1.0".to_string(),
//!             message_id: message.id.0.clone(),
//!             status: ResponseStatus::Success,
//!             payload: vec![message.payload.clone()],
//!             error_message: None,
//!             metadata: Default::default(),
//!         })
//!     }
//! }
//!
//! // Create and use the protocol
//! fn example() {
//!     let mut protocol = MCPProtocolBase::new_default();
//!     
//!     // Register a handler
//!     let handler = Box::new(EchoHandler);
//!     protocol.register_handler(MessageType::Command, handler).unwrap();
//!     
//!     // Create a message
//!     let message = MCPMessage {
//!         id: MessageId("msg123".to_string()),
//!         type_: MessageType::Command,
//!         payload: json!({"text": "Hello, MCP!"}),
//!     };
//!     
//!     // The message would be handled asynchronously
//!     // let response = protocol.handle_protocol_message(&message).await;
//! }
//! ```

use std::sync::Arc;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use tokio::sync::RwLock;
use crate::error::{MCPError, Result};
use crate::mcp::types::{MCPMessage, ProtocolVersion, ProtocolState, MCPCommand, MCPResponse};
use serde_json::Value;

use super::{MCPProtocol, CommandHandler};

/// Core implementation of the MCP protocol.
///
/// This struct provides the main functionality for the Machine Context Protocol,
/// including message validation, routing, and handling, as well as state management
/// and handler registration.
///
/// The protocol maintains its own state and configuration, and allows for registering
/// handlers for different message types. When a message is received, it is validated,
/// routed to the appropriate handler, and the response is returned.
///
/// # Examples
///
/// ```
/// use mcp::protocol::{MCPProtocolBase, ProtocolConfig};
/// use mcp::types::{MCPMessage, MessageId, MessageType};
/// use serde_json::json;
///
/// // Create a new protocol instance with default config
/// let protocol = MCPProtocolBase::new_default();
///
/// // Or create with custom config
/// let config = ProtocolConfig {
///     version: "1.1".to_string(),
///     max_message_size: 1048576, // 1MB
///     timeout_ms: 5000, // 5 seconds
/// };
/// let protocol = MCPProtocolBase::new(config);
/// ```
#[derive(Debug)]
pub struct MCPProtocolBase {
    /// The current protocol state, stored as a JSON value.
    state: Value,
    
    /// The protocol configuration, including version and limits.
    config: ProtocolConfig,
    
    /// Map of registered handlers for different message types.
    handlers: HashMap<MessageType, Box<dyn CommandHandler>>,
    
    /// Number of handlers registered in the protocol.
    handler_count: usize,
}

impl MCPProtocolBase {
    /// Creates a new protocol instance with the provided configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - The protocol configuration
    ///
    /// # Returns
    ///
    /// A new protocol instance with the provided configuration
    ///
    /// # Examples
    ///
    /// ```
    /// use mcp::protocol::{MCPProtocolBase, ProtocolConfig};
    ///
    /// let config = ProtocolConfig {
    ///     version: "1.0".to_string(),
    ///     max_message_size: 524288, // 512KB
    ///     timeout_ms: 3000, // 3 seconds
    /// };
    ///
    /// let protocol = MCPProtocolBase::new(config);
    /// ```
    #[must_use]
    pub fn new(config: ProtocolConfig) -> Self {
        Self {
            state: json!({
                "initialized": true,
                "version": config.version,
                "handler_count": 0,
            }),
            config,
            handlers: HashMap::new(),
            handler_count: 0,
        }
    }

    /// Creates a new protocol instance with default configuration.
    ///
    /// This is a convenience method for creating a protocol instance with
    /// the default configuration settings.
    ///
    /// # Returns
    ///
    /// A new protocol instance with default configuration
    ///
    /// # Examples
    ///
    /// ```
    /// use mcp::protocol::MCPProtocolBase;
    ///
    /// let protocol = MCPProtocolBase::new_default();
    /// ```
    #[must_use]
    pub fn new_default() -> Self {
        Self::new(ProtocolConfig::default())
    }

    /// Creates a new protocol instance with a specific protocol version.
    ///
    /// This method allows creating a protocol instance with the default
    /// configuration but a specific protocol version.
    ///
    /// # Arguments
    ///
    /// * `version` - The protocol version
    ///
    /// # Returns
    ///
    /// A new protocol instance with the specified version
    ///
    /// # Examples
    ///
    /// ```
    /// use mcp::protocol::MCPProtocolBase;
    ///
    /// let protocol = MCPProtocolBase::with_version("1.1");
    /// ```
    #[must_use]
    pub fn with_version(version: &str) -> Self {
        let mut config = ProtocolConfig::default();
        config.version = version.to_string();
        Self::new(config)
    }

    /// Creates a new protocol instance with provided configuration.
    ///
    /// This is an alias for `new()`.
    ///
    /// # Arguments
    ///
    /// * `config` - The protocol configuration
    ///
    /// # Returns
    ///
    /// A new protocol instance with the provided configuration
    #[must_use]
    pub fn with_config(config: ProtocolConfig) -> Self {
        Self::new(config)
    }

    /// Gets the current protocol state.
    ///
    /// # Returns
    ///
    /// The current protocol state as a JSON value
    ///
    /// # Examples
    ///
    /// ```
    /// use mcp::protocol::MCPProtocolBase;
    ///
    /// let protocol = MCPProtocolBase::new_default();
    /// let state = protocol.get_state();
    /// println!("Protocol state: {}", state);
    /// ```
    #[must_use]
    pub fn get_state(&self) -> &Value {
        &self.state
    }

    /// Sets the protocol state.
    ///
    /// This method updates the protocol state with the provided JSON value.
    ///
    /// # Arguments
    ///
    /// * `state` - The new protocol state
    ///
    /// # Examples
    ///
    /// ```
    /// use mcp::protocol::MCPProtocolBase;
    /// use serde_json::json;
    ///
    /// let mut protocol = MCPProtocolBase::new_default();
    ///
    /// let new_state = json!({
    ///     "initialized": true,
    ///     "version": "1.0",
    ///     "handler_count": 0,
    ///     "custom_field": "custom_value",
    /// });
    ///
    /// protocol.set_state(new_state);
    /// ```
    pub fn set_state(&mut self, state: Value) {
        self.state = state;
    }

    /// Gets the protocol configuration.
    ///
    /// # Returns
    ///
    /// The protocol configuration
    ///
    /// # Examples
    ///
    /// ```
    /// use mcp::protocol::MCPProtocolBase;
    ///
    /// let protocol = MCPProtocolBase::new_default();
    /// let config = protocol.get_config();
    /// println!("Protocol version: {}", config.version);
    /// ```
    #[must_use]
    pub fn get_config(&self) -> &ProtocolConfig {
        &self.config
    }

    /// Validates a message before processing.
    ///
    /// This method validates that the message meets the protocol requirements,
    /// including checking the message type and payload.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to validate
    ///
    /// # Returns
    ///
    /// `Ok(())` if the message is valid
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The message has an invalid payload
    /// - No handler is registered for the message type
    ///
    /// # Examples
    ///
    /// ```
    /// use mcp::protocol::{MCPProtocolBase, CommandHandler};
    /// use mcp::types::{MCPMessage, MessageId, MessageType};
    /// use serde_json::json;
    ///
    /// // Create handlers and register them...
    ///
    /// let protocol = MCPProtocolBase::new_default();
    /// // protocol.register_handler(MessageType::Command, Box::new(handler));
    ///
    /// let message = MCPMessage {
    ///     id: MessageId("msg123".to_string()),
    ///     type_: MessageType::Command,
    ///     payload: json!({"command": "status"}),
    /// };
    ///
    /// // Validate the message
    /// let result = protocol.validate_message(&message);
    /// ```
    pub fn validate_message(&self, message: &MCPMessage) -> ValidationResult {
        // Check if we have a handler for this message type
        if !matches!(message.type_, MessageType::Setup) {
            if !self.handlers.contains_key(&message.type_) {
                return Err(MCPError::Protocol(ProtocolError::NoHandlerForMessageType(
                    format!("{:?}", message.type_),
                )));
            }
        }

        // Check payload - should be a valid JSON object for command messages
        if matches!(message.type_, MessageType::Command) {
            if !message.payload.is_object() {
                return Err(MCPError::Protocol(ProtocolError::InvalidPayload(
                    "Command messages must have a JSON object payload".to_string(),
                )));
            }
            if let Some(obj) = message.payload.as_object() {
                if obj.is_empty() {
                    return Err(MCPError::Protocol(ProtocolError::InvalidPayload(
                        "Command messages must have a non-empty JSON object payload".to_string(),
                    )));
                }
            }
        }

        Ok(())
    }

    /// Handles a protocol message.
    ///
    /// This method validates the message, routes it to the appropriate handler,
    /// and returns the response from the handler.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to handle
    ///
    /// # Returns
    ///
    /// A `ProtocolResult` containing the response from the handler
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The message validation fails
    /// - No handler is registered for the message type
    /// - The handler encounters an error while processing the message
    ///
    /// # Examples
    ///
    /// ```
    /// use mcp::protocol::{MCPProtocolBase, CommandHandler};
    /// use mcp::types::{MCPMessage, MessageId, MessageType, MCPResponse, ResponseStatus};
    /// use mcp::error::Result;
    /// use async_trait::async_trait;
    /// use serde_json::json;
    ///
    /// #[derive(Debug)]
    /// struct StatusHandler;
    ///
    /// #[async_trait]
    /// impl CommandHandler for StatusHandler {
    ///     async fn handle(&self, message: &MCPMessage) -> Result<MCPResponse> {
    ///         // Handle status command
    ///         Ok(MCPResponse {
    ///             protocol_version: "1.0".to_string(),
    ///             message_id: message.id.0.clone(),
    ///             status: ResponseStatus::Success,
    ///             payload: vec![],
    ///             error_message: None,
    ///             metadata: Default::default(),
    ///         })
    ///     }
    /// }
    ///
    /// async fn example() {
    ///     let mut protocol = MCPProtocolBase::new_default();
    ///     protocol.register_handler(MessageType::Command, Box::new(StatusHandler)).unwrap();
    ///     
    ///     let message = MCPMessage {
    ///         id: MessageId("msg123".to_string()),
    ///         type_: MessageType::Command,
    ///         payload: json!({"command": "status"}),
    ///     };
    ///     
    ///     let response = protocol.handle_protocol_message(&message).await;
    /// }
    /// ```
    pub async fn handle_protocol_message(&self, message: &MCPMessage) -> ProtocolResult {
        // Special case for setup messages - these have special handling
        if message.type_ == MessageType::Setup {
            // For setup, we just return success and the protocol version
            return Ok(MCPResponse {
                protocol_version: self.config.version.clone(),
                message_id: message.id.0.clone(),
                status: ResponseStatus::Success,
                payload: vec![json!({
                    "protocol_version": self.config.version,
                    "max_message_size": self.config.max_message_size,
                    "timeout_ms": self.config.timeout_ms,
                })],
                error_message: None,
                metadata: Default::default(),
            });
        }

        // Route to the appropriate handler based on message type
        if let Some(handler) = self.handlers.get(&message.type_) {
            handler.handle(message).await
        } else {
            Err(MCPError::Protocol(ProtocolError::NoHandlerForMessageType(
                format!("{:?}", message.type_),
            )))
        }
    }

    /// Registers a handler for a specific message type.
    ///
    /// This method registers a handler for a specific message type,
    /// allowing the protocol to process messages of that type.
    ///
    /// # Arguments
    ///
    /// * `message_type` - The message type to register the handler for
    /// * `handler` - The handler to register
    ///
    /// # Returns
    ///
    /// `Ok(())` if the handler was registered successfully
    ///
    /// # Errors
    ///
    /// Returns an error if a handler is already registered for the message type
    ///
    /// # Examples
    ///
    /// ```
    /// use mcp::protocol::{MCPProtocolBase, CommandHandler};
    /// use mcp::types::{MCPMessage, MessageType, MCPResponse, ResponseStatus};
    /// use mcp::error::Result;
    /// use async_trait::async_trait;
    ///
    /// #[derive(Debug)]
    /// struct StatusHandler;
    ///
    /// #[async_trait]
    /// impl CommandHandler for StatusHandler {
    ///     async fn handle(&self, message: &MCPMessage) -> Result<MCPResponse> {
    ///         // Handle status command
    ///         Ok(MCPResponse {
    ///             protocol_version: "1.0".to_string(),
    ///             message_id: message.id.0.clone(),
    ///             status: ResponseStatus::Success,
    ///             payload: vec![],
    ///             error_message: None,
    ///             metadata: Default::default(),
    ///         })
    ///     }
    /// }
    ///
    /// fn example() {
    ///     let mut protocol = MCPProtocolBase::new_default();
    ///     
    ///     // Register a handler for command messages
    ///     let handler = Box::new(StatusHandler);
    ///     protocol.register_handler(MessageType::Command, handler).unwrap();
    /// }
    /// ```
    pub fn register_handler(
        &mut self,
        message_type: MessageType,
        handler: Box<dyn CommandHandler>,
    ) -> Result<()> {
        if self.handlers.contains_key(&message_type) {
            return Err(MCPError::Protocol(ProtocolError::HandlerAlreadyRegistered(
                format!("{:?}", message_type),
            )));
        }

        self.handlers.insert(message_type, handler);
        self.handler_count += 1;

        // Update state
        if let Value::Object(ref mut map) = self.state {
            map.insert("handler_count".to_string(), json!(self.handler_count));
        }

        Ok(())
    }

    /// Unregisters a handler for a specific message type.
    ///
    /// This method removes a previously registered handler for a message type.
    ///
    /// # Arguments
    ///
    /// * `message_type` - The message type to unregister the handler for
    ///
    /// # Returns
    ///
    /// `Ok(())` if the handler was unregistered successfully
    ///
    /// # Errors
    ///
    /// Returns an error if no handler is registered for the message type
    ///
    /// # Examples
    ///
    /// ```
    /// use mcp::protocol::MCPProtocolBase;
    /// use mcp::types::MessageType;
    ///
    /// fn example() {
    ///     let mut protocol = MCPProtocolBase::new_default();
    ///     // Register handlers...
    ///     
    ///     // Unregister a handler
    ///     let result = protocol.unregister_handler(&MessageType::Command);
    /// }
    /// ```
    pub fn unregister_handler(&mut self, message_type: &MessageType) -> Result<()> {
        if !self.handlers.contains_key(message_type) {
            return Err(MCPError::Protocol(ProtocolError::NoHandlerForMessageType(
                format!("{:?}", message_type),
            )));
        }

        self.handlers.remove(message_type);
        self.handler_count -= 1;

        // Update state
        if let Value::Object(ref mut map) = self.state {
            map.insert("handler_count".to_string(), json!(self.handler_count));
        }

        Ok(())
    }

    /// Get the number of registered handlers.
    ///
    /// # Returns
    ///
    /// The number of registered handlers
    ///
    /// # Examples
    ///
    /// ```
    /// use mcp::protocol::MCPProtocolBase;
    ///
    /// let protocol = MCPProtocolBase::new_default();
    /// let count = protocol.get_handler_count();
    /// assert_eq!(count, 0); // No handlers registered yet
    /// ```
    #[must_use]
    pub fn get_handler_count(&self) -> usize {
        self.handler_count
    }

    /// Generates a response for the given message.
    ///
    /// This is a helper method to create a standard response for a message.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to generate a response for
    /// * `status` - The response status
    /// * `payload` - The response payload
    /// * `error_message` - Optional error message
    ///
    /// # Returns
    ///
    /// An `MCPResponse` for the message
    ///
    /// # Examples
    ///
    /// ```
    /// use mcp::protocol::MCPProtocolBase;
    /// use mcp::types::{MCPMessage, MessageId, MessageType, ResponseStatus};
    /// use serde_json::json;
    ///
    /// fn example() {
    ///     let protocol = MCPProtocolBase::new_default();
    ///     
    ///     let message = MCPMessage {
    ///         id: MessageId("msg123".to_string()),
    ///         type_: MessageType::Command,
    ///         payload: json!({"command": "status"}),
    ///     };
    ///     
    ///     let payload = vec![json!({"status": "ok"})];
    ///     let response = protocol.generate_response(
    ///         &message,
    ///         ResponseStatus::Success,
    ///         payload,
    ///         None,
    ///     );
    ///     
    ///     assert_eq!(response.message_id, "msg123");
    ///     assert_eq!(response.status, ResponseStatus::Success);
    /// }
    /// ```
    #[must_use]
    pub fn generate_response(
        &self,
        message: &MCPMessage,
        status: ResponseStatus,
        payload: Vec<Value>,
        error_message: Option<String>,
    ) -> MCPResponse {
        MCPResponse {
            protocol_version: self.config.version.clone(),
            message_id: message.id.0.clone(),
            status,
            payload,
            error_message,
            metadata: Default::default(),
        }
    }

    /// Checks if the protocol has a handler for a specific message type.
    ///
    /// # Arguments
    ///
    /// * `message_type` - The message type to check for
    ///
    /// # Returns
    ///
    /// `true` if a handler is registered for the message type, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```
    /// use mcp::protocol::MCPProtocolBase;
    /// use mcp::types::MessageType;
    ///
    /// fn example() {
    ///     let protocol = MCPProtocolBase::new_default();
    ///     // Register handlers...
    ///     
    ///     let has_handler = protocol.has_handler_for(&MessageType::Command);
    ///     println!("Has command handler: {}", has_handler);
    /// }
    /// ```
    #[must_use]
    pub fn has_handler_for(&self, message_type: &MessageType) -> bool {
        self.handlers.contains_key(message_type)
    }
}

impl MCPProtocol {
    pub fn new() -> Self {
        Self {
            state: Value::Null,
            handlers: HashMap::new(),
        }
    }

    pub async fn handle_message(&self, message: &MCPMessage) -> Result<MCPMessage> {
        let handler = self.handlers.get(&message.command)
            .ok_or_else(|| MCPError::Protocol(format!("No handler for command: {}", message.command)))?;

        handler.handle(message).await
    }

    pub fn register_handler(&mut self, command: String, handler: Box<dyn CommandHandler>) -> Result<()> {
        if self.handlers.contains_key(&command) {
            return Err(MCPError::Protocol(format!("Handler already exists for command: {}", command)));
        }
        self.handlers.insert(command, handler);
        Ok(())
    }

    pub fn unregister_handler(&mut self, command: &str) -> Result<()> {
        self.handlers.remove(command)
            .ok_or_else(|| MCPError::Protocol(format!("No handler found for command: {}", command)))?;
        Ok(())
    }

    pub fn get_state(&self) -> &Value {
        &self.state
    }

    pub fn set_state(&mut self, state: Value) {
        self.state = state;
    }
}

impl Default for MCPProtocol {
    fn default() -> Self {
        Self::new()
    }
} 