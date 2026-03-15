// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Protocol adapter module for MCP.
//!
//! This module provides a thread-safe adapter implementation for the Machine Context Protocol (MCP),
//! allowing multiple components to safely interact with the protocol system concurrently.
//! The adapter pattern decouples the protocol implementation from its consumers,
//! providing a clean, stable interface while hiding internal complexity.
//!
//! # Key Features
//!
//! - Thread-safe protocol access with proper locking
//! - Lazy initialization of the protocol
//! - Support for custom protocol configurations
//! - Clean interface for message handling
//! - Handler registration and management
//! - State management
//!
//! # Examples
//!
//! Creating and using a protocol adapter:
//!
//! ```
//! use squirrel_mcp::protocol::{create_protocol_adapter, MCPProtocol};
//! use squirrel_mcp::protocol::types::MessageId;
//! use squirrel_mcp::protocol::types::MCPMessage;
//! use squirrel_mcp::protocol::types::MessageType;
//! use squirrel_mcp::protocol::types::ProtocolVersion;
//! use serde_json::json;
//! use chrono::Utc;
//!
//! async fn example() {
//!     // Create a protocol adapter
//!     let adapter = create_protocol_adapter();
//!     
//!     // Initialize the adapter
//!     adapter.initialize().await.expect("Failed to initialize");
//!     
//!     // Create a message
//!     let message = MCPMessage {
//!         id: MessageId("msg123".to_string()),
//!         type_: MessageType::Command,
//!         payload: json!({"command": "status"}),
//!         metadata: Some(json!({})),
//!         security: Default::default(),
//!         timestamp: Utc::now(),
//!         version: ProtocolVersion::new(1, 0),
//!         trace_id: Some("trace-123".to_string()),
//!     };
//!     
//!     // Handle the message
//!     let response = adapter.handle_message(message).await;
//! }
//! ```

use crate::protocol::types::{MCPMessage, MessageType, ProtocolVersion, MessageId};
use crate::types::{MCPResponse, ResponseStatus};
use crate::error::{MCPError, Result as MCPResult};
use crate::error::{MCPError, ProtocolError};
use crate::protocol::{
    MCPProtocol, MCPProtocolBase, ProtocolConfig, ProtocolResult, 
    RoutingResult, ValidationResult, RoutingDecision, CommandHandler
};
use crate::types::ProtocolState;
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Protocol adapter that provides a thread-safe interface for working with the MCP protocol.
///
/// This adapter wraps the underlying protocol implementation in a thread-safe container,
/// allowing multiple components to safely interact with the protocol system concurrently.
/// It provides methods for initializing the protocol, handling messages, registering handlers,
/// and managing protocol state.
///
/// # Examples
///
/// ```
/// use squirrel_mcp::protocol::{MCPProtocolAdapter, ProtocolConfig};
/// use squirrel_mcp::protocol::types::MessageId;
/// use squirrel_mcp::protocol::types::MCPMessage;
/// use squirrel_mcp::protocol::types::MessageType;
/// use serde_json::json;
/// use std::sync::Arc;
///
/// async fn example() {
///     // Create a new adapter
///     let adapter = Arc::new(MCPProtocolAdapter::new());
///     
///     // Initialize with custom configuration
///     let config = ProtocolConfig::default();
///     adapter.initialize_with_config(config).await.expect("Failed to initialize");
///     
///     // Use the adapter
///     let is_ready = adapter.is_initialized().await;
///     println!("Protocol ready: {}", is_ready);
/// }
/// ```
#[derive(Debug)]
pub struct MCPProtocolAdapter {
    /// Inner protocol implementation, wrapped in an Option to allow lazy initialization
    inner: Arc<RwLock<Option<MCPProtocolBase>>>,
}

impl MCPProtocolAdapter {
    /// Creates a new empty protocol adapter.
    ///
    /// This creates an uninitialized adapter that must be initialized
    /// before use with either `initialize()` or `initialize_with_config()`.
    ///
    /// # Returns
    ///
    /// A new, uninitialized `MCPProtocolAdapter`
    ///
    /// # Examples
    ///
    /// ```
    /// use squirrel_mcp::protocol::MCPProtocolAdapter;
    ///
    /// let adapter = MCPProtocolAdapter::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(None)),
        }
    }

    /// Creates a new protocol adapter with a given protocol implementation.
    ///
    /// This creates an adapter that is already initialized with the provided
    /// protocol implementation, making it ready for immediate use.
    ///
    /// # Arguments
    ///
    /// * `protocol` - The protocol implementation to use
    ///
    /// # Returns
    ///
    /// A new, initialized `MCPProtocolAdapter`
    ///
    /// # Examples
    ///
    /// ```
    /// use squirrel_mcp::protocol::{MCPProtocolAdapter, MCPProtocolBase, ProtocolConfig};
    ///
    /// let protocol = MCPProtocolBase::new_default();
    /// let adapter = MCPProtocolAdapter::with_protocol(protocol);
    /// ```
    #[must_use]
    pub fn with_protocol(protocol: MCPProtocolBase) -> Self {
        Self {
            inner: Arc::new(RwLock::new(Some(protocol))),
        }
    }

    /// Initializes the protocol adapter with default configuration.
    ///
    /// This method initializes the adapter with a default protocol configuration
    /// if it is not already initialized. If the adapter is already initialized,
    /// this method does nothing and returns success.
    ///
    /// # Returns
    ///
    /// `Ok(())` if initialization was successful or if the adapter was already initialized
    ///
    /// # Errors
    ///
    /// Returns an error if the protocol could not be initialized
    ///
    /// # Examples
    ///
    /// ```
    /// use squirrel_mcp::protocol::MCPProtocolAdapter;
    ///
    /// async fn example() {
    ///     let adapter = MCPProtocolAdapter::new();
    ///     adapter.initialize().await.expect("Failed to initialize");
    /// }
    /// ```
    pub async fn initialize(&self) -> Result<()> {
        let mut inner = self.inner.write().await;

        if inner.is_some() {
            return Ok(());
        }

        *inner = Some(MCPProtocolBase::new(ProtocolConfig::default()));
        Ok(())
    }

    /// Initializes the protocol adapter with a specific configuration.
    ///
    /// This method initializes the adapter with the provided protocol configuration
    /// if it is not already initialized. If the adapter is already initialized,
    /// this method does nothing and returns success.
    ///
    /// # Arguments
    ///
    /// * `config` - The protocol configuration to use
    ///
    /// # Returns
    ///
    /// `Ok(())` if initialization was successful or if the adapter was already initialized
    ///
    /// # Errors
    ///
    /// Returns an error if the protocol could not be initialized
    ///
    /// # Examples
    ///
    /// ```
    /// use squirrel_mcp::protocol::{MCPProtocolAdapter, ProtocolConfig};
    ///
    /// async fn example() {
    ///     let adapter = MCPProtocolAdapter::new();
    ///     let config = ProtocolConfig {
    ///         version: "1.1".to_string(),
    ///         max_message_size: 2 * 1024 * 1024, // 2MB
    ///         timeout_ms: 10000, // 10 seconds
    ///     };
    ///     adapter.initialize_with_config(config).await.expect("Failed to initialize");
    /// }
    /// ```
    pub async fn initialize_with_config(&self, config: ProtocolConfig) -> Result<()> {
        let mut inner = self.inner.write().await;

        if inner.is_some() {
            return Ok(());
        }

        *inner = Some(MCPProtocolBase::with_config(config));
        Ok(())
    }

    /// Checks if the protocol adapter is initialized.
    ///
    /// This method checks if the adapter has been initialized with a protocol
    /// implementation and is ready for use.
    ///
    /// # Returns
    ///
    /// `true` if the adapter is initialized, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```
    /// use squirrel_mcp::protocol::MCPProtocolAdapter;
    ///
    /// async fn example() {
    ///     let adapter = MCPProtocolAdapter::new();
    ///     
    ///     // Check if initialized
    ///     let initialized = adapter.is_initialized().await;
    ///     assert!(!initialized);
    ///     
    ///     // Initialize
    ///     adapter.initialize().await.expect("Failed to initialize");
    ///     
    ///     // Check again
    ///     let initialized = adapter.is_initialized().await;
    ///     assert!(initialized);
    /// }
    /// ```
    pub async fn is_initialized(&self) -> bool {
        let inner = self.inner.read().await;
        inner.is_some()
    }

    /// Handles a message according to the protocol.
    ///
    /// This method routes the message to the appropriate handler based on its type
    /// and validates the message before processing. Special handling is provided
    /// for setup messages, which are processed even without a payload.
    ///
    /// # Arguments
    ///
    /// * `msg` - The message to handle
    ///
    /// # Returns
    ///
    /// A `ProtocolResult` containing the response from the handler
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The protocol is not initialized
    /// - The message has an invalid payload
    /// - No handler is registered for the message type
    /// - The handler encounters an error while processing the message
    ///
    /// # Examples
    ///
    /// ```
    /// use squirrel_mcp::protocol::MCPProtocolAdapter;
    /// use squirrel_mcp::protocol::types::MessageId;
    /// use squirrel_mcp::protocol::types::MCPMessage;
    /// use squirrel_mcp::protocol::types::MessageType;
    /// use squirrel_mcp::protocol::types::ProtocolVersion;
    /// use serde_json::json;
    /// use chrono::Utc;
    ///
    /// async fn example() {
    ///     let adapter = MCPProtocolAdapter::new();
    ///     adapter.initialize().await.expect("Failed to initialize");
    ///     
    ///     let message = MCPMessage {
    ///         id: MessageId("msg123".to_string()),
    ///         type_: MessageType::Command,
    ///         payload: json!({"command": "status"}),
    ///         metadata: json!({}).into(),
    ///         security: Default::default(),
    ///         timestamp: Utc::now(),
    ///         version: ProtocolVersion::new(1, 0),
    ///         trace_id: Some(uuid::Uuid::new_v4().to_string()),
    ///     };
    ///     
    ///     let response = adapter.handle_message(message).await;
    /// }
    /// ```
    pub async fn handle_message(&self, msg: MCPMessage) -> ProtocolResult {
        let protocol_guard = self.inner.read().await;

        if let Some(protocol) = &*protocol_guard {
            // Special handling for setup message
            if msg.type_ == MessageType::Setup {
                // Setup messages should be processed even without payload
                return protocol.handle_protocol_message(&msg).await;
            }

            let payload = msg.payload.as_object();
            match payload {
                None => {
                    return Err(MCPError::Protocol(
                        ProtocolError::InvalidPayload("Empty or invalid payload".to_string())
                    ).into());
                }
                Some(payload) => {
                    // Validate and route the message
                    protocol.validate_message(&msg)?;

                    protocol.handle_protocol_message(&msg).await
                }
            }
        } else {
            Err(MCPError::Protocol(ProtocolError::ProtocolNotInitialized).into())
        }
    }

    /// Registers a command handler for a specific message type.
    ///
    /// This method registers a handler for a specific message type, allowing
    /// the protocol to process messages of that type.
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
    /// Returns an error if:
    /// - The protocol is not initialized
    /// - A handler is already registered for the message type
    ///
    /// # Examples
    ///
    /// ```
    /// use squirrel_mcp::protocol::{MCPProtocolAdapter, CommandHandler};
    /// use squirrel_mcp::protocol::types::MessageId;
    /// use squirrel_mcp::protocol::types::MCPMessage;
    /// use squirrel_mcp::protocol::types::MessageType;
    /// use squirrel_mcp::types::{MCPResponse, ResponseStatus};
    /// use squirrel_mcp::error::Result;
    /// use async_trait::async_trait;
    /// use std::sync::Arc;
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
    ///             message_id: message.id.clone(),
    ///             status: ResponseStatus::Success,
    ///             payload: vec![],
    ///             error_message: None,
    ///             metadata: Default::default(),
    ///         })
    ///     }
    /// }
    ///
    /// async fn example() {
    ///     let adapter = MCPProtocolAdapter::new();
    ///     adapter.initialize().await.expect("Failed to initialize");
    ///     
    ///     // Register a handler for command messages
    ///     adapter.register_handler(
    ///         MessageType::Command,
    ///         Box::new(StatusHandler)
    ///     ).await.expect("Failed to register handler");
    /// }
    /// ```
    pub async fn register_handler(
        &self,
        message_type: crate::protocol::MessageType,
        handler: Box<dyn CommandHandler>,
    ) -> Result<()> {
        let mut inner = self.inner.write().await;
        if let Some(protocol) = &mut *inner {
            protocol.register_handler(message_type, handler)
        } else {
            Err(MCPError::Protocol(ProtocolError::ProtocolNotInitialized).into())
        }
    }

    /// Unregisters a command handler for a specific message type.
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
    /// Returns an error if:
    /// - The protocol is not initialized
    /// - No handler is registered for the message type
    ///
    /// # Examples
    ///
    /// ```
    /// use squirrel_mcp::protocol::MCPProtocolAdapter;
    /// use squirrel_mcp::protocol::types::MessageId;
    /// use squirrel_mcp::protocol::types::MCPMessage;
    /// use squirrel_mcp::protocol::types::MessageType;
    ///
    /// async fn example() {
    ///     let adapter = MCPProtocolAdapter::new();
    ///     adapter.initialize().await.expect("Failed to initialize");
    ///     
    ///     // Unregister a handler for command messages
    ///     let result = adapter.unregister_handler(&MessageType::Command).await;
    ///     // Will likely error since we didn't register a handler
    /// }
    /// ```
    pub async fn unregister_handler(&self, message_type: &crate::protocol::MessageType) -> Result<()> {
        let mut inner = self.inner.write().await;
        if let Some(protocol) = &mut *inner {
            protocol.unregister_handler(message_type)
        } else {
            Err(MCPError::Protocol(ProtocolError::ProtocolNotInitialized).into())
        }
    }

    /// Gets the protocol state as a JSON value.
    ///
    /// This method retrieves the current protocol state as a JSON value,
    /// which can be used for serialization or inspection.
    ///
    /// # Returns
    ///
    /// The current protocol state as a JSON value, or an empty object if not initialized
    ///
    /// # Examples
    ///
    /// ```
    /// use squirrel_mcp::protocol::MCPProtocolAdapter;
    ///
    /// async fn example() {
    ///     let adapter = MCPProtocolAdapter::new();
    ///     adapter.initialize().await.expect("Failed to initialize");
    ///     
    ///     let state = adapter.get_state().await;
    ///     println!("Protocol state: {}", state);
    /// }
    /// ```
    pub async fn get_state(&self) -> Value {
        let inner = self.inner.read().await;
        if let Some(ref protocol) = *inner {
            protocol.get_state().clone()
        } else {
            json!({})
        }
    }

    /// Sets the protocol state.
    ///
    /// This method updates the protocol state with a new JSON value.
    ///
    /// # Arguments
    ///
    /// * `state` - The new protocol state
    ///
    /// # Returns
    ///
    /// `Ok(())` if the state was updated successfully
    ///
    /// # Errors
    ///
    /// Returns an error if the protocol is not initialized
    ///
    /// # Examples
    ///
    /// ```
    /// use squirrel_mcp::protocol::MCPProtocolAdapter;
    /// use serde_json::json;
    ///
    /// async fn example() {
    ///     let adapter = MCPProtocolAdapter::new();
    ///     adapter.initialize().await.expect("Failed to initialize");
    ///     
    ///     let new_state = json!({
    ///         "status": "ready",
    ///         "connections": 5,
    ///         "uptime": 3600
    ///     });
    ///     
    ///     adapter.set_state(new_state).await.expect("Failed to set state");
    /// }
    /// ```
    pub async fn set_state(&self, state: Value) -> Result<()> {
        let mut inner = self.inner.write().await;
        if let Some(ref mut protocol) = *inner {
            protocol.set_state(state);
            Ok(())
        } else {
            Err(MCPError::Protocol(ProtocolError::ProtocolNotInitialized).into())
        }
    }

    /// Gets the protocol configuration.
    ///
    /// This method retrieves the current protocol configuration, which includes
    /// settings like version, message size limits, and timeouts.
    ///
    /// # Returns
    ///
    /// The current protocol configuration, or the default if not initialized
    ///
    /// # Examples
    ///
    /// ```
    /// use squirrel_mcp::protocol::MCPProtocolAdapter;
    ///
    /// async fn example() {
    ///     let adapter = MCPProtocolAdapter::new();
    ///     adapter.initialize().await.expect("Failed to initialize");
    ///     
    ///     let config = adapter.get_config().await;
    ///     println!("Protocol version: {}", config.version);
    /// }
    /// ```
    pub async fn get_config(&self) -> ProtocolConfig {
        let inner = self.inner.read().await;
        if let Some(ref protocol) = *inner {
            protocol.get_config().clone()
        } else {
            ProtocolConfig::default()
        }
    }

    /// Gets the protocol version.
    ///
    /// This method retrieves the protocol version string. Since this is a static
    /// value, it does not require locking the protocol.
    ///
    /// # Returns
    ///
    /// The protocol version string
    ///
    /// # Examples
    ///
    /// ```
    /// use squirrel_mcp::protocol::MCPProtocolAdapter;
    ///
    /// let adapter = MCPProtocolAdapter::new();
    /// let version = adapter.get_version();
    /// println!("Protocol version: {}", version);
    /// ```
    #[must_use] pub fn get_version(&self) -> String {
        // The version is a static string, no need to lock
        "1.0".to_string()
    }
}

impl Clone for MCPProtocolAdapter {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl Default for MCPProtocolAdapter {
    fn default() -> Self {
        Self::new()
    }
}

/// Creates a new protocol adapter
#[must_use]
pub fn create_protocol_adapter() -> Arc<MCPProtocolAdapter> {
    Arc::new(MCPProtocolAdapter::new())
}

/// Creates a new protocol adapter with an existing protocol
#[must_use]
pub fn create_protocol_adapter_with_protocol(protocol: MCPProtocolBase) -> Arc<MCPProtocolAdapter> {
    Arc::new(MCPProtocolAdapter::with_protocol(protocol))
}

/// Creates a new protocol adapter and initializes it with default configuration
///
/// # Errors
///
/// Returns an error if initialization fails.
pub async fn create_initialized_protocol_adapter() -> Result<Arc<MCPProtocolAdapter>> {
    let adapter = MCPProtocolAdapter::new();
    adapter.initialize().await?;
    Ok(Arc::new(adapter))
}

/// Creates a new protocol adapter and initializes it with custom configuration
///
/// # Errors
///
/// Returns an error if initialization fails.
pub async fn create_protocol_adapter_with_config(
    config: ProtocolConfig,
) -> Result<Arc<MCPProtocolAdapter>> {
    let adapter = MCPProtocolAdapter::new();
    adapter.initialize_with_config(config).await?;
    Ok(Arc::new(adapter))
}

impl MCPProtocol for MCPProtocolAdapter {
    fn handle_message(&self, msg: MCPMessage) -> impl std::future::Future<Output = Result<MCPMessage>> + Send {
        async move {
            let protocol_guard = self.inner.read().await;

            if let Some(protocol) = &*protocol_guard {
                // Special handling for setup message
                if msg.type_ == MessageType::Setup {
                    // Setup messages should be processed even without payload
                    return protocol.handle_protocol_message(&msg).await;
                }

                let payload = msg.payload.as_object();
                match payload {
                    None => {
                        return Err(MCPError::Protocol(
                            ProtocolError::InvalidPayload("Empty or invalid payload".to_string())
                        ).into());
                    }
                    Some(payload) => {
                        // Validate and route the message
                        protocol.validate_message(&msg)?;

                        protocol.handle_protocol_message(&msg).await
                    }
                }
                } else {
                    Err(MCPError::Protocol(ProtocolError::ProtocolNotInitialized).into())
                }
            }
        }
    }

    fn get_version(&self) -> impl std::future::Future<Output = ProtocolVersion> + Send {
        async move {
            ProtocolVersion::default()
        }
    }
}

// Compatibility methods (non-trait)
impl MCPProtocolAdapter {
    async fn validate_message_compat(&self, msg: &MCPMessage) -> ValidationResult {
        let protocol_guard = self.inner.read().await;

        if let Some(protocol) = &*protocol_guard {
            protocol.validate_message(msg)
        } else {
            Err(MCPError::Protocol(ProtocolError::ProtocolNotInitialized).into())
        }
    }

    async fn route_message_compat(&self, _msg: &crate::protocol::types::MCPMessage) -> RoutingResult {
        let protocol_guard = self.inner.read().await;

        if let Some(ref _protocol) = *protocol_guard {
            // Check if the protocol has registered a handler for this message type
            // For now, we're just implementing a basic placeholder
            // In the future, this would delegate to protocol's handlers
            Ok(RoutingDecision::NoRouteFound)
        } else {
            Err(MCPError::Protocol(ProtocolError::ProtocolNotInitialized).into())
        }
    }

    async fn set_state(&self, new_state: ProtocolState) -> Result<()> {
        let mut protocol_guard = self.inner.write().await;

        if let Some(protocol) = &mut *protocol_guard {
            protocol.set_protocol_state(new_state);
            Ok(())
        } else {
            Err(MCPError::Protocol(ProtocolError::ProtocolNotInitialized).into())
        }
    }

    async fn get_state(&self) -> Result<ProtocolState> {
        let inner = self.inner.read().await;
        if let Some(ref _protocol) = *inner {
            // Here we need to convert from the internal state to the ProtocolState
            Ok(ProtocolState::Ready)
        } else {
            Err(MCPError::Protocol(ProtocolError::ProtocolNotInitialized).into())
        }
    }

    fn get_version(&self) -> String {
        // The version is a static string, so we can simply return it directly
        // without needing to await anything
        "1.0".to_string()
    }
}

#[cfg(test)]
mod tests {
    #![allow(unused_imports)]
    use super::*;
    use crate::protocol::types::MessageId;
    use crate::types::MCPResponse;
    use crate::protocol::MessageType;
    use crate::types::ResponseStatus;
    use crate::types::MessageMetadata;
    use chrono::Utc;
    use crate::protocol::types::ProtocolVersion;
    // BearDog handles security: // use crate::security::types::SecurityMetadata;
    use serde_json::json;

    /// Test handler implementation
    #[derive(Debug)]
    struct TestCommandHandler;

    #[async_trait::async_trait]
    impl super::super::CommandHandler for TestCommandHandler {
        async fn handle(&self, message: &MCPMessage) -> Result<MCPResponse> {
            // Simple test implementation
            Ok(MCPResponse {
                protocol_version: "1.0".to_string(),
                message_id: message.id.clone(),
                status: ResponseStatus::Success,
                metadata: MessageMetadata::default(),
                payload: vec![json!({"response": "success"})],
                error_message: None,
            })
        }
    }

    #[tokio::test]
    async fn test_adapter_initialization() {
        // Create a new adapter
        let adapter = MCPProtocolAdapter::new();

        // Should not be initialized yet
        assert!(!adapter.is_initialized().await);

        // Initialize it
        adapter.initialize().await.unwrap();

        // Now it should be initialized
        assert!(adapter.is_initialized().await);

        // Second initialization should be ok (idempotent)
        let result = adapter.initialize().await;
        assert!(result.is_ok(), "Second initialization should be idempotent");
    }

    #[tokio::test]
    async fn test_adapter_with_config() {
        // Create a custom config
        let config = ProtocolConfig {
            version: "2.0".to_string(),
            max_message_size: 2048,
            timeout_ms: 10000,
        };

        // Create and initialize with config
        let adapter = MCPProtocolAdapter::new();
        adapter
            .initialize_with_config(config.clone())
            .await
            .unwrap();

        // Check the config was set
        let adapter_config = adapter.get_config().await;
        assert_eq!(adapter_config.version, "2.0");
        assert_eq!(adapter_config.max_message_size, 2048);
        assert_eq!(adapter_config.timeout_ms, 10000);
    }

    #[tokio::test]
    async fn test_uninitialized_operations() {
        // Create a new adapter without initializing
        let adapter = MCPProtocolAdapter::new();

        // Trying to handle a message should fail
        let message = MCPMessage {
            id: MessageId("test-1".to_string()),
            type_: MessageType::Command,
            payload: json!({"command": "test"}),
            metadata: Some(json!({})),
            security: SecurityMetadata::default(),
            timestamp: Utc::now(),
            version: ProtocolVersion::default(),
            trace_id: None,
        };

        let err = adapter.handle_message(message).await.unwrap_err();
        assert!(err.to_string().contains("not initialized"));

        // Register handler should also fail
        let handler = Box::new(TestCommandHandler);
        let err = adapter
            .register_handler(MessageType::Command, handler)
            .await
            .unwrap_err();
        assert!(err.to_string().contains("not initialized"));
    }

    #[tokio::test]
    async fn test_handler_registration() {
        // Create and initialize adapter
        let adapter = MCPProtocolAdapter::new();
        adapter.initialize().await.unwrap();

        // Register a handler
        let handler = Box::new(TestCommandHandler);
        adapter
            .register_handler(MessageType::Command, handler)
            .await
            .unwrap();

        // Create a test message for the registered handler
        let message = MCPMessage {
            id: MessageId("test-1".to_string()),
            type_: MessageType::Command,
            payload: json!({"command": "test"}),
            metadata: Some(json!({})),
            security: SecurityMetadata::default(),
            timestamp: Utc::now(),
            version: ProtocolVersion::default(),
            trace_id: None,
        };

        // Handle the message
        let response = adapter.handle_message(message).await.unwrap();

        // Verify response
        assert_eq!(response.status, crate::types::ResponseStatus::Success);
        assert_eq!(response.message_id.0, "test-1");
    }

    #[tokio::test]
    async fn test_factory_functions() {
        // Test simple creation
        let adapter1 = create_protocol_adapter();
        assert!(!adapter1.is_initialized().await);

        // Test creation with an existing protocol
        let config = ProtocolConfig::default();
        let protocol = MCPProtocolBase::new(config);
        let adapter2 = create_protocol_adapter_with_protocol(protocol);
        assert!(adapter2.is_initialized().await);

        // Test initialized creation
        let adapter3 = create_initialized_protocol_adapter().await.unwrap();
        assert!(adapter3.is_initialized().await);

        // Test creation with config
        let custom_config = ProtocolConfig {
            version: "3.0".to_string(),
            max_message_size: 4096,
            timeout_ms: 15000,
        };
        let adapter4 = create_protocol_adapter_with_config(custom_config)
            .await
            .unwrap();
        assert!(adapter4.is_initialized().await);

        let config4 = adapter4.get_config().await;
        assert_eq!(config4.version, "3.0");
    }

    #[tokio::test]
    async fn test_state_management() {
        // Create and initialize adapter
        let adapter = MCPProtocolAdapter::new();
        adapter.initialize().await.unwrap();

        // Default state should be null
        let state = adapter.get_state().await;
        assert!(state.is_null());

        // Set a new state
        let new_state = json!({"status": "connected", "client_id": "test-client"});
        adapter
            .set_state(new_state.clone())
            .await
            .expect("Failed to set state");

        // Get the state again
        let updated_state = adapter.get_state().await;
        assert_eq!(updated_state, new_state);
    }

    #[tokio::test]
    async fn test_adapter_cloning() {
        // Create and initialize adapter
        let adapter = MCPProtocolAdapter::new();
        adapter.initialize().await.unwrap();

        // Register a handler
        let handler = Box::new(TestCommandHandler);
        adapter
            .register_handler(MessageType::Command, handler)
            .await
            .unwrap();

        // Clone the adapter
        let adapter_clone = adapter.clone();

        // Both should be initialized
        assert!(adapter.is_initialized().await);
        assert!(adapter_clone.is_initialized().await);

        // Both should have the handler
        // BearDog handles security: // use crate::security::types::SecurityMetadata;
        use chrono::Utc;
        use crate::protocol::types::ProtocolVersion;
        
        let message = MCPMessage {
            id: MessageId("test-1".to_string()),
            type_: MessageType::Command,
            payload: json!({"command": "test"}),
            metadata: Some(json!({})),
            security: SecurityMetadata::default(),
            timestamp: Utc::now(),
            version: ProtocolVersion::default(),
            trace_id: None,
        };

        let response1 = adapter.handle_message(message.clone()).await.unwrap();
        let response2 = adapter_clone.handle_message(message).await.unwrap();

        assert_eq!(response1.status, response2.status);
    }
}
