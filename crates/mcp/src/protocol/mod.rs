//! MCP Protocol module for machine context exchange.
//!
//! This module implements the core protocol functionality for the Machine Context Protocol (MCP),
//! providing a robust framework for message handling, state management, and command execution
//! between components in a distributed system.
//!
//! The MCP protocol is designed to facilitate secure, efficient communication with:
//! - Message format validation
//! - Protocol state management
//! - Command routing and execution
//! - Security integration
//! - Error handling and recovery
//!
//! # Core Components
//!
//! The key components in this module include:
//!
//! - [`MCPProtocolBase`]: Base implementation of the protocol
//! - [`MCPProtocolAdapter`]: Thread-safe adapter providing a clean interface
//! - [`CommandHandler`]: Trait for handling command messages
//! - [`MCPProtocol`]: Interface for protocol operations
//!
//! # Examples
//!
//! Creating and using the MCP protocol:
//!
//! ```
//! use mcp::protocol::{MCPProtocolBase, ProtocolConfig};
//! use mcp::types::{MCPMessage, MessageId, MessageType};
//! use serde_json::json;
//!
//! // Create a new protocol instance
//! let protocol = MCPProtocolBase::new_default();
//!
//! // Create a response from a message
//! let message = MCPMessage {
//!     id: MessageId("msg123".to_string()),
//!     type_: MessageType::Command,
//!     payload: json!({"command": "status"}),
//! };
//!
//! let response = protocol.create_response(&message, mcp::types::ResponseStatus::Success);
//! ```
//!
//! Using the protocol adapter:
//!
//! ```
//! use mcp::protocol::{create_protocol_adapter, MCPProtocol};
//! use std::sync::Arc;
//!
//! async fn example() {
//!     // Create a protocol adapter
//!     let adapter = create_protocol_adapter();
//!     
//!     // Initialize the adapter
//!     adapter.initialize().await.expect("Failed to initialize");
//!     
//!     // Check protocol state
//!     let state = adapter.get_state().await.expect("Failed to get state");
//! }
//! ```

use crate::error::{MCPError, ProtocolError, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fmt::Debug;
use std::str::FromStr;
use std::sync::Arc;

// Import common types from types module
use crate::types::{
    MCPMessage, MCPResponse, MessageMetadata, MessageType, ProtocolState, ResponseStatus,
};

/// Protocol-related types module
pub mod types;
// Re-export types from the protocol::types module
pub use types::*;

/// Protocol-specific result type for operations that return a response.
///
/// This type is an alias for `Result<MCPResponse>` and is used throughout
/// the protocol module for operations that produce a response message.
pub type ProtocolResult = Result<MCPResponse>;

/// Result type for validation operations.
///
/// This type is an alias for `Result<()>` and is used for operations
/// that validate messages without producing a response.
pub type ValidationResult = Result<()>;

/// Result type for message routing operations.
///
/// This type is an alias for `Result<()>` and is used for operations
/// that route messages to their appropriate handlers.
pub type RoutingResult = Result<()>;

/// Adapter module for protocol operations
pub mod adapter;
pub use adapter::{
    create_protocol_adapter, create_protocol_adapter_with_protocol, MCPProtocolAdapter,
};

/// Wire format adapter for protocol serialization/deserialization and versioning
pub mod adapter_wire;
pub use adapter_wire::{
    WireFormatAdapter, WireFormatConfig, WireMessage, WireFormat, ProtocolVersion, DomainObject
};

/// Domain object implementations for protocol serialization/deserialization
pub mod domain_objects;

/// Implementation module for protocol core functionality
mod impl_protocol;
pub use impl_protocol::MCPProtocolImpl;

/// Serialization helpers for protocol operations
pub mod serialization_helpers;

/// Serialization utility functions
pub mod serialization_utils;

/// Module for domain object tests
#[cfg(test)]
mod domain_objects_tests;

/// Configuration for the MCP protocol.
///
/// This struct contains the configuration parameters for the MCP protocol,
/// including version information, message size limits, and timeout settings.
/// It allows customizing the behavior of the protocol to meet specific requirements
/// for different environments and use cases.
///
/// The configuration affects various aspects of protocol operation:
/// - The protocol version determines compatibility with clients
/// - Message size limits protect against resource exhaustion
/// - Timeout settings ensure operations complete in a reasonable time
///
/// # Fields
///
/// * `version` - Protocol version string, used for compatibility checking
/// * `max_message_size` - Maximum allowed message size in bytes, prevents `DoS` attacks
/// * `timeout_ms` - Timeout for protocol operations in milliseconds, ensures responsiveness
///
/// # Examples
///
/// Creating a custom configuration:
///
/// ```
/// use mcp::protocol::ProtocolConfig;
///
/// // For high-performance systems with large messages
/// let high_capacity_config = ProtocolConfig {
///     version: "1.1".to_string(),
///     max_message_size: 10 * 1024 * 1024, // 10MB
///     timeout_ms: 30000, // 30 seconds
/// };
///
/// // For resource-constrained environments
/// let lightweight_config = ProtocolConfig {
///     version: "1.0".to_string(),
///     max_message_size: 64 * 1024, // 64KB
///     timeout_ms: 3000, // 3 seconds
/// };
///
/// // Using the default configuration
/// let default_config = ProtocolConfig::default();
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProtocolConfig {
    /// Protocol version string (e.g., "1.0")
    ///
    /// This version identifier is used for compatibility checking between
    /// clients and servers. It follows semantic versioning conventions.
    pub version: String,
    
    /// Maximum allowed message size in bytes
    ///
    /// This limit prevents denial of service attacks and resource exhaustion
    /// by capping the size of messages that can be processed. Messages exceeding
    /// this size will be rejected.
    pub max_message_size: usize,
    
    /// Timeout for protocol operations in milliseconds
    ///
    /// This setting determines how long the protocol will wait for operations
    /// to complete before timing out. It ensures the system remains responsive
    /// even when facing slow or unresponsive components.
    pub timeout_ms: u64,
}

impl Default for ProtocolConfig {
    /// Creates a default configuration with balanced settings.
    ///
    /// The default configuration provides reasonable values suitable for
    /// most general-purpose applications:
    /// - Version: "1.0"
    /// - Max message size: 1MB (sufficient for most message types)
    /// - Timeout: 5 seconds (balances responsiveness with operation complexity)
    fn default() -> Self {
        Self {
            version: "1.0".to_string(),
            max_message_size: 1024 * 1024, // 1MB
            timeout_ms: 5000,              // 5 seconds
        }
    }
}

/// Trait for handlers that process command messages.
///
/// This trait defines the interface for components that handle specific types of messages
/// in the MCP system. Implementations of this trait are responsible for processing
/// incoming messages, executing commands, and producing appropriate responses.
///
/// Command handlers are the core extension point of the MCP system, allowing
/// new functionality to be added by implementing handlers for different
/// message types or commands.
///
/// # Thread Safety
///
/// All implementations of `CommandHandler` must be thread-safe, as they may be
/// called concurrently from multiple contexts. This typically means implementing
/// `Send` and `Sync` for your handler types.
///
/// # Error Handling
///
/// Handlers should return appropriate errors when they encounter issues processing
/// a message. The MCP protocol will translate these errors into appropriate response
/// messages with error details.
///
/// # Performance Considerations
///
/// Since handlers are called during message processing, they should be designed to
/// complete quickly to avoid blocking the message handling loop. For long-running
/// operations, consider:
///
/// - Delegating work to background tasks
/// - Using asynchronous processing with proper cancellation support
/// - Implementing progress reporting mechanisms
///
/// # Examples
///
/// Implementing a simple command handler:
///
/// ```
/// use async_trait::async_trait;
/// use mcp::error::Result;
/// use mcp::protocol::CommandHandler;
/// use mcp::types::{MCPMessage, MCPResponse, ResponseStatus};
/// use serde_json::json;
///
/// #[derive(Debug)]
/// struct EchoHandler;
///
/// #[async_trait]
/// impl CommandHandler for EchoHandler {
///     async fn handle(&self, message: &MCPMessage) -> Result<MCPResponse> {
///         // Echo back the message payload
///         Ok(MCPResponse {
///             protocol_version: "1.0".to_string(),
///             message_id: message.id.0.clone(),
///             status: ResponseStatus::Success,
///             payload: vec![message.payload.clone()],
///             error_message: None,
///             metadata: Default::default(),
///         })
///     }
/// }
/// ```
///
/// Implementing a handler with command routing:
///
/// ```
/// use async_trait::async_trait;
/// use mcp::error::{MCPError, Result, CommandError};
/// use mcp::protocol::CommandHandler;
/// use mcp::types::{MCPMessage, MCPResponse, ResponseStatus};
/// use serde_json::json;
///
/// #[derive(Debug)]
/// struct MultiCommandHandler {
///     // Handler state could go here
/// }
///
/// impl MultiCommandHandler {
///     pub fn new() -> Self {
///         Self {}
///     }
///
///     async fn handle_status(&self, params: &serde_json::Value) -> Result<serde_json::Value> {
///         // Process status command
///         Ok(json!({
///             "status": "operational",
///             "uptime": 3600,
///             "version": "1.0"
///         }))
///     }
///
///     async fn handle_reset(&self, params: &serde_json::Value) -> Result<serde_json::Value> {
///         // Process reset command
///         Ok(json!({
///             "reset": true,
///             "timestamp": "2023-06-10T12:00:00Z"
///         }))
///     }
/// }
///
/// #[async_trait]
/// impl CommandHandler for MultiCommandHandler {
///     async fn handle(&self, message: &MCPMessage) -> Result<MCPResponse> {
///         // Extract the command type from the payload
///         let cmd = message.payload.get("command")
///             .and_then(|v| v.as_str())
///             .ok_or_else(|| MCPError::Command(CommandError::InvalidCommand(
///                 "Missing or invalid 'command' field".to_string()
///             )))?;
///
///         // Route to the appropriate handler based on command
///         let result = match cmd {
///             "status" => self.handle_status(&message.payload).await?,
///             "reset" => self.handle_reset(&message.payload).await?,
///             _ => return Err(MCPError::Command(CommandError::UnknownCommand(
///                 format!("Unknown command: {}", cmd)
///             ))),
///         };
///
///         // Construct the response
///         Ok(MCPResponse {
///             protocol_version: "1.0".to_string(),
///             message_id: message.id.0.clone(),
///             status: ResponseStatus::Success,
///             payload: vec![result],
///             error_message: None,
///             metadata: Default::default(),
///         })
///     }
/// }
/// ```
#[async_trait::async_trait]
pub trait CommandHandler: Send + Sync + Debug {
    /// Handles a message and produces a response.
    ///
    /// This method is called by the protocol when a message of the associated type
    /// is received. The handler is responsible for processing the message, executing
    /// any required actions, and producing an appropriate response.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to handle
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing either:
    ///
    /// - An `MCPResponse` with the results of processing the message
    /// - An `MCPError` if an error occurred during processing
    ///
    /// # Error Handling
    ///
    /// The handler should return appropriate errors when it encounters issues,
    /// using the error types defined in the `error` module. These errors will
    /// be translated into response messages with appropriate error status and
    /// details by the protocol system.
    async fn handle(&self, message: &MCPMessage) -> Result<MCPResponse>;
}

/// Base implementation of the MCP protocol.
///
/// This struct provides the core functionality of the MCP protocol,
/// managing message handling, state transitions, and command execution.
/// It maintains a registry of command handlers for different message types
/// and tracks the current protocol state.
///
/// # Examples
///
/// Creating and using a protocol instance:
///
/// ```
/// use mcp::protocol::{MCPProtocolBase, ProtocolConfig, CommandHandler};
/// use mcp::types::{MCPMessage, MessageId, MessageType};
/// use async_trait::async_trait;
/// use serde_json::json;
///
/// // Create a protocol instance
/// let mut protocol = MCPProtocolBase::new_default();
///
/// // Create a message
/// let message = MCPMessage {
///     id: MessageId("msg123".to_string()),
///     type_: MessageType::Command,
///     payload: json!({"command": "get_status"}),
/// };
///
/// // Create a response
/// let response = protocol.create_response(&message, mcp::types::ResponseStatus::Success);
/// ```
#[derive(Debug)]
pub struct MCPProtocolBase {
    /// Protocol configuration
    config: ProtocolConfig,
    /// Registered command handlers
    #[allow(clippy::type_complexity)]
    handlers: HashMap<String, Box<dyn CommandHandler>>,
    /// Current protocol state
    state: Value,
}

impl MCPProtocolBase {
    /// Creates a new protocol instance with the specified configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - The protocol configuration
    ///
    /// # Returns
    ///
    /// A new `MCPProtocolBase` instance with the specified configuration
    #[must_use]
    pub fn new(config: ProtocolConfig) -> Self {
        Self {
            config,
            handlers: HashMap::new(),
            state: Value::Null,
        }
    }

    /// Creates a new protocol instance with default configuration.
    ///
    /// # Returns
    ///
    /// A new `MCPProtocolBase` instance with default configuration
    #[must_use]
    pub fn new_default() -> Self {
        Self::new(ProtocolConfig::default())
    }

    /// Creates a new protocol instance with custom configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - The protocol configuration
    ///
    /// # Returns
    ///
    /// A new `MCPProtocolBase` instance with the specified configuration
    #[must_use]
    pub fn with_config(config: ProtocolConfig) -> Self {
        Self::new(config)
    }

    /// Creates a new protocol instance with custom dependencies.
    ///
    /// # Arguments
    ///
    /// * `config` - The protocol configuration
    /// * `handlers` - Map of message types to command handlers
    /// * `initial_state` - Initial protocol state
    ///
    /// # Returns
    ///
    /// A new `MCPProtocolBase` instance with the specified dependencies
    #[must_use]
    pub fn with_dependencies(
        config: ProtocolConfig,
        handlers: HashMap<String, Box<dyn CommandHandler>>,
        initial_state: Value,
    ) -> Self {
        Self {
            config,
            handlers,
            state: initial_state,
        }
    }

    /// Creates a response message from a request message.
    ///
    /// # Arguments
    ///
    /// * `message` - The request message
    /// * `status` - The response status
    ///
    /// # Returns
    ///
    /// A new `MCPResponse` with information from the request message
    #[must_use]
    pub fn create_response(&self, message: &MCPMessage, status: ResponseStatus) -> MCPResponse {
        MCPResponse {
            protocol_version: self.config.version.clone(),
            message_id: message.id.0.clone(),
            status,
            payload: Vec::new(),
            error_message: None,
            metadata: MessageMetadata::default(),
        }
    }

    /// Handles a message using the appropriate registered handler.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to handle
    ///
    /// # Returns
    ///
    /// A `Result<MCPResponse>` containing the response from the handler
    ///
    /// # Errors
    ///
    /// Returns an error if no handler is registered for the message type
    pub async fn handle_message_with_handler(&self, message: &MCPMessage) -> Result<MCPResponse> {
        let handler = self
            .handlers
            .get(&message.type_.to_string())
            .ok_or_else(|| {
                MCPError::Protocol(ProtocolError::HandlerNotFound(format!(
                    "No handler for {}",
                    message.type_
                )))
            })?;

        // Call the handler and return its response directly
        handler.handle(message).await
    }

    /// Registers a command handler for a specific message type.
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
    pub fn register_handler(
        &mut self,
        message_type: MessageType,
        handler: Box<dyn CommandHandler>,
    ) -> Result<()> {
        if self.handlers.contains_key(&message_type.to_string()) {
            return Err(MCPError::Protocol(ProtocolError::HandlerAlreadyExists(
                format!("Handler already exists for message type: {message_type:?}"),
            )));
        }
        self.handlers.insert(message_type.to_string(), handler);
        Ok(())
    }

    /// Unregisters a command handler for a specific message type.
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
    /// Returns an error if no handler is found for the message type
    pub fn unregister_handler(&mut self, message_type: &MessageType) -> Result<()> {
        self.handlers
            .remove(&message_type.to_string())
            .ok_or_else(|| {
                MCPError::Protocol(ProtocolError::HandlerNotFound(format!(
                    "No handler found for message type: {message_type:?}"
                )))
            })?;
        Ok(())
    }

    /// Gets the current protocol state.
    ///
    /// # Returns
    ///
    /// A reference to the current protocol state
    #[must_use]
    pub const fn get_state(&self) -> &Value {
        &self.state
    }

    /// Sets the protocol state.
    ///
    /// # Arguments
    ///
    /// * `state` - The new protocol state
    pub fn set_state(&mut self, state: Value) {
        self.state = state;
    }

    /// Gets the protocol configuration.
    ///
    /// # Returns
    ///
    /// A reference to the protocol configuration
    #[must_use]
    pub const fn get_config(&self) -> &ProtocolConfig {
        &self.config
    }

    /// Sets the protocol configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - The new protocol configuration
    pub fn set_config(&mut self, config: ProtocolConfig) {
        self.config = config;
    }

    /// Gets the protocol state as an enum
    #[must_use]
    pub fn get_protocol_state(&self) -> ProtocolState {
        // Default to initialized state if we can't parse the state
        if self.state.is_null() {
            return ProtocolState::Initialized;
        }

        // Try to extract a string representation of the state
        if let Some(state_str) = self.state.get("state").and_then(|s| s.as_str()) {
            match state_str {
                "ready" => ProtocolState::Ready,
                "error" => ProtocolState::Error,
                _ => ProtocolState::Initialized,
            }
        } else {
            ProtocolState::Initialized
        }
    }

    /// Sets the protocol state from an enum
    pub fn set_protocol_state(&mut self, state: ProtocolState) {
        let state_str = match state {
            ProtocolState::Initialized => "initialized",
            ProtocolState::Ready => "ready",
            ProtocolState::Error => "error",
            ProtocolState::Uninitialized => "uninitialized",
            ProtocolState::Initializing => "initializing",
            ProtocolState::ShuttingDown => "shutting_down",
            ProtocolState::Closed => "closed",
        };

        // Update the existing state object or create a new one
        if self.state.is_object() {
            if let Some(obj) = self.state.as_object_mut() {
                obj.insert("state".to_string(), Value::String(state_str.to_string()));
            }
        } else {
            self.state = json!({ "state": state_str });
        }
    }

    /// Handle a protocol message by delegating to a registered handler.
    pub async fn handle_protocol_message(&self, message: &MCPMessage) -> ProtocolResult {
        let handler = self
            .handlers
            .get(&message.type_.to_string())
            .ok_or_else(|| {
                MCPError::Protocol(ProtocolError::HandlerNotFound(format!(
                    "No handler for {}",
                    message.type_
                )))
            })?;

        handler.handle(message).await
    }

    /// Validates a message according to protocol rules
    pub fn validate_message(&self, message: &MCPMessage) -> ValidationResult {
        // Basic validation: ensure message has the right format
        if message.id.0.is_empty() {
            return Err(MCPError::Protocol(ProtocolError::InvalidFormat(
                "Message ID is missing".to_string(),
            )));
        }

        // Check protocol version compatibility if present in the payload
        if let Some(version) = message.payload.get("protocol_version") {
            if version.as_str() != Some(&self.config.version) {
                return Err(MCPError::Protocol(ProtocolError::InvalidVersion(format!(
                    "Protocol version mismatch: message has {}, expected {}",
                    version, self.config.version
                ))));
            }
        }

        Ok(())
    }

    /// Recovers from a protocol error, if possible
    pub fn recover_from_error(
        &mut self,
        error: ProtocolError,
    ) -> std::result::Result<(), ProtocolError> {
        match error {
            // Handle recoverable errors
            ProtocolError::InvalidFormat(_)
            | ProtocolError::InvalidPayload(_)
            | ProtocolError::MessageTooLarge(_)
            | ProtocolError::InvalidTimestamp(_)
            | ProtocolError::MessageTimeout(_) => {
                // Log the error but don't take action
                tracing::warn!("Recoverable error encountered: {}", error);
                Ok(())
            }
            ProtocolError::HandlerNotFound(ref message_type) => {
                // Register a default handler for unknown message types
                tracing::warn!(
                    "Registering default handler for unhandled message type: {}",
                    message_type
                );

                // Create a default handler that returns an error response
                #[derive(Debug)]
                struct DefaultHandler;

                #[async_trait::async_trait]
                impl CommandHandler for DefaultHandler {
                    async fn handle(&self, message: &MCPMessage) -> Result<MCPResponse> {
                        let response = MCPResponse {
                            protocol_version: "1.0".to_string(),
                            message_id: message.id.0.clone(),
                            status: ResponseStatus::Error,
                            payload: Vec::new(),
                            error_message: Some(
                                "No handler registered for this message type".to_string(),
                            ),
                            metadata: MessageMetadata::default(),
                        };

                        Ok(response)
                    }
                }

                // Extract message type string
                let type_str = message_type.to_string();
                if let Ok(message_type) = MessageType::from_str(&type_str) {
                    // Register a default handler for this message type
                    let _ = self.register_handler(message_type, Box::new(DefaultHandler));
                    Ok(())
                } else {
                    Err(ProtocolError::RecoveryFailed(format!(
                        "Failed to parse message type: {message_type}"
                    )))
                }
            }
            ProtocolError::InvalidState(ref state) => {
                // Try to reset the protocol state
                tracing::warn!("Resetting protocol state from invalid state: {}", state);
                self.set_protocol_state(ProtocolState::Initialized);
                Ok(())
            }
            // Non-recoverable errors
            _ => {
                // If we get here, we couldn't recover
                tracing::error!("Non-recoverable error: {}", error);
                Err(error)
            }
        }
    }
}

/// Factory for creating protocol instances
#[derive(Debug)]
pub struct MCPProtocolFactory {
    /// Protocol configuration
    config: ProtocolConfig,
}

impl MCPProtocolFactory {
    /// Creates a new factory with the specified configuration
    #[must_use]
    pub const fn new(config: ProtocolConfig) -> Self {
        Self { config }
    }

    /// Creates a new factory with the specified configuration
    #[must_use]
    pub const fn with_config(config: ProtocolConfig) -> Self {
        Self { config }
    }

    /// Creates a new protocol instance
    #[must_use]
    pub fn create_protocol(&self) -> MCPProtocolBase {
        MCPProtocolBase::new(self.config.clone())
    }

    /// Creates a new protocol instance with custom dependencies
    #[must_use]
    pub fn create_protocol_with_dependencies(
        &self,
        handlers: HashMap<String, Box<dyn CommandHandler>>,
        initial_state: Value,
    ) -> MCPProtocolBase {
        MCPProtocolBase::with_dependencies(self.config.clone(), handlers, initial_state)
    }

    /// Creates a new protocol adapter
    #[must_use]
    pub fn create_protocol_adapter(&self) -> Arc<MCPProtocolAdapter> {
        let protocol = self.create_protocol_with_dependencies(HashMap::new(), Value::Null);
        Arc::new(MCPProtocolAdapter::with_protocol(protocol))
    }
}

impl Default for MCPProtocolFactory {
    fn default() -> Self {
        Self::new(ProtocolConfig::default())
    }
}

/// Trait defining the MCP protocol operations.
///
/// This trait specifies the core functionality of the MCP protocol, including
/// handling messages, validating message structures, routing messages, and
/// managing the protocol state. Implementations of this trait provide the
/// concrete logic for the protocol operations.
#[async_trait::async_trait]
pub trait MCPProtocol: Send + Sync + Debug {
    /// Handles an incoming message, processing it according to the protocol rules.
    ///
    /// This method is the main entry point for processing messages. It takes an
    /// incoming `MCPMessage`, validates it, determines the appropriate action
    /// based on the message type (e.g., command, event, response), and produces
    /// a response or performs the required side effects.
    ///
    /// # Arguments
    ///
    /// * `msg` - The `MCPMessage` to handle.
    ///
    /// # Returns
    ///
    /// A `ProtocolResult` containing the `MCPResponse` if successful, or an
    /// `MCPError` if processing fails.
    async fn handle_message(&self, msg: MCPMessage) -> ProtocolResult;

    /// Validates the structure and content of an incoming message.
    ///
    /// This method checks if a message conforms to the protocol specification,
    /// including verifying required fields, checking data types, and ensuring
    /// the message size is within limits.
    ///
    /// # Arguments
    ///
    /// * `msg` - The `MCPMessage` to validate.
    ///
    /// # Returns
    ///
    /// A `ValidationResult` which is `Ok(())` if validation succeeds, or an
    /// `MCPError` (typically `ProtocolError`) if validation fails.
    async fn validate_message(&self, msg: &MCPMessage) -> ValidationResult;

    /// Routes a message to the appropriate handler or destination.
    ///
    /// Based on the message type and content, this method determines where
    /// the message should be sent for further processing. This might involve
    /// looking up command handlers, dispatching events, or forwarding responses.
    ///
    /// # Arguments
    ///
    /// * `msg` - The `MCPMessage` to route.
    ///
    /// # Returns
    ///
    /// A `RoutingResult` which is `Ok(())` if routing is successful, or an
    /// `MCPError` if routing fails (e.g., handler not found).
    async fn route_message(&self, msg: &MCPMessage) -> RoutingResult;

    /// Sets the current state of the protocol.
    ///
    /// Allows external components to update the protocol's operational state,
    /// for example, setting it to `Ready` after initialization or `Error` if
    /// a critical failure occurs.
    ///
    /// # Arguments
    ///
    /// * `new_state` - The `ProtocolState` to set.
    ///
    /// # Returns
    ///
    /// A `Result<()>` indicating success or failure.
    async fn set_state(&self, new_state: ProtocolState) -> Result<()>;

    /// Gets the current state of the protocol.
    ///
    /// Returns the current operational state of the protocol handler.
    ///
    /// # Returns
    ///
    /// A `Result<ProtocolState>` containing the current state or an error.
    async fn get_state(&self) -> Result<ProtocolState>;
    
    /// Gets the protocol version string.
    ///
    /// # Returns
    ///
    /// The protocol version string (e.g., "1.0").
    fn get_version(&self) -> String;
}

/// Trait for handlers that process specific message types
#[async_trait::async_trait]
pub trait MessageHandler: Send + Sync + std::fmt::Debug {
    /// Handles a specific type of message
    async fn handle(&self, message: &MCPMessage) -> ProtocolResult;

    /// Validates a message according to handler rules
    async fn validate(&self, message: &MCPMessage) -> ValidationResult;

    /// Routes a message to its appropriate processor
    async fn route(&self, message: &MCPMessage) -> RoutingResult;

    /// Returns the message types this handler can process
    fn supported_types(&self) -> Vec<MessageType>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    pub struct ProtocolVersion {
        major: u32,
        minor: u32,
    }

    impl ProtocolVersion {
        #[allow(dead_code)]
        pub fn new(major: u32, minor: u32) -> Self {
            Self { major, minor }
        }
    }

    #[derive(Clone, Debug, PartialEq)]
    #[allow(dead_code)]
    pub enum ProtocolState {
        Initialized,
        Ready,
        Error,
    }

    #[derive(Clone, Debug)]
    #[allow(dead_code)]
    pub enum CompressionFormat {
        None,
        Gzip,
        Zstd,
    }

    #[derive(Clone, Debug)]
    #[allow(dead_code)]
    pub enum EncryptionFormat {
        None,
        Aes256Gcm,
        ChaCha20Poly1305,
    }

    #[derive(Clone, Debug, Default)]
    pub struct MessageMetadata {
        #[allow(dead_code)]
        pub timestamp: u64,
        #[allow(dead_code)]
        pub source: String,
        #[allow(dead_code)]
        pub destination: String,
    }

    #[derive(Clone, Debug)]
    #[allow(dead_code)]
    pub enum ResponseStatus {
        Success,
        Error,
        Pending,
    }
}
