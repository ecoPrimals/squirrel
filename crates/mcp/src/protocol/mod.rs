//! MCP Protocol module
//!
//! This module implements the core protocol functionality for the Machine Context Protocol (MCP).
//! It handles message processing, protocol state management, and command execution.

use std::sync::Arc;
use std::collections::HashMap;
use std::fmt::Debug;
use serde::{Serialize, Deserialize};
use serde_json::{Value, json};
use async_trait::async_trait;
use tokio::sync::RwLock;
use uuid::Uuid;
use crate::error::{Result, MCPError, ProtocolError};
use squirrel_core::error::SquirrelError;

// Import common types from types module
use crate::types::{
    MessageType,
    MCPMessage,
    MCPResponse,
    ResponseStatus,
    MessageMetadata,
    ProtocolState,
    SecurityLevel,
};

/// Protocol-specific result type for operations that return a value
pub type ProtocolResult = Result<MCPResponse>;
/// Result type for validation operations
pub type ValidationResult = Result<()>;
/// Result type for message routing operations
pub type RoutingResult = Result<()>;

/// Adapter module for protocol operations
pub mod adapter;
pub use adapter::{MCPProtocolAdapter, create_protocol_adapter, create_protocol_adapter_with_protocol};
/// Implementation module for protocol core functionality
mod impl_protocol;
pub use impl_protocol::MCPProtocolImpl;

/// Configuration for the MCP protocol
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProtocolConfig {
    /// Protocol version string (e.g., "1.0")
    pub version: String,
    /// Maximum allowed message size in bytes
    pub max_message_size: usize,
    /// Timeout for protocol operations in milliseconds
    pub timeout_ms: u64,
}

impl Default for ProtocolConfig {
    fn default() -> Self {
        Self {
            version: "1.0".to_string(),
            max_message_size: 1024 * 1024, // 1MB
            timeout_ms: 5000, // 5 seconds
        }
    }
}

/// Trait for handlers that process command messages
#[async_trait::async_trait]
pub trait CommandHandler: Send + Sync + std::fmt::Debug {
    /// Handles a command message and returns a response
    async fn handle(&self, message: &MCPMessage) -> Result<MCPResponse>;
}

/// Base implementation of the MCP protocol
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
    /// Creates a new protocol instance with the specified configuration
    #[must_use] pub fn new(config: ProtocolConfig) -> Self {
        Self {
            config,
            handlers: HashMap::new(),
            state: Value::Null,
        }
    }
    
    /// Creates a new protocol instance with default configuration
    #[must_use] pub fn new_default() -> Self {
        Self::new(ProtocolConfig::default())
    }

    /// Creates a new protocol instance with custom configuration
    #[must_use] pub fn with_config(config: ProtocolConfig) -> Self {
        Self::new(config)
    }

    /// Creates a new protocol instance with custom dependencies
    #[must_use] pub fn with_dependencies(
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

    /// Creates a response message from a request message
    #[must_use] pub fn create_response(&self, message: &MCPMessage, status: ResponseStatus) -> MCPResponse {
        MCPResponse {
            protocol_version: self.config.version.clone(),
            message_id: message.id.0.clone(),
            status,
            payload: Vec::new(),
            error_message: None,
            metadata: MessageMetadata::default(),
        }
    }

    /// Handles a message using the appropriate registered handler
    ///
    /// # Errors
    ///
    /// Returns an error if no handler is registered for the message type
    pub async fn handle_message_with_handler(&self, message: &MCPMessage) -> Result<MCPResponse> {
        let handler = self.handlers.get(&message.message_type.to_string())
            .ok_or_else(|| MCPError::Protocol(ProtocolError::HandlerNotFound(format!("No handler for message type: {:?}", message.message_type))))?;

        // Call the handler and return its response directly
        handler.handle(message).await
    }

    /// Registers a command handler for a specific message type
    ///
    /// # Errors
    ///
    /// Returns an error if a handler is already registered for the message type
    pub fn register_handler(&mut self, message_type: MessageType, handler: Box<dyn CommandHandler>) -> Result<()> {
        if self.handlers.contains_key(&message_type.to_string()) {
            return Err(MCPError::Protocol(ProtocolError::HandlerAlreadyExists(format!("Handler already exists for message type: {message_type:?}"))));
        }
        self.handlers.insert(message_type.to_string(), handler);
        Ok(())
    }

    /// Unregisters a command handler for a specific message type
    ///
    /// # Errors
    ///
    /// Returns an error if no handler is found for the message type
    pub fn unregister_handler(&mut self, message_type: &MessageType) -> Result<()> {
        self.handlers.remove(&message_type.to_string())
            .ok_or_else(|| MCPError::Protocol(ProtocolError::HandlerNotFound(format!("No handler found for message type: {message_type:?}"))))?;
        Ok(())
    }

    /// Gets the current protocol state
    #[must_use] pub fn get_state(&self) -> &Value {
        &self.state
    }

    /// Sets the protocol state
    pub fn set_state(&mut self, state: Value) {
        self.state = state;
    }

    /// Gets the protocol configuration
    #[must_use] pub fn get_config(&self) -> &ProtocolConfig {
        &self.config
    }

    /// Sets the protocol configuration
    pub fn set_config(&mut self, config: ProtocolConfig) {
        self.config = config;
    }

    /// Gets the protocol state as an enum
    #[must_use] pub fn get_protocol_state(&self) -> ProtocolState {
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
        let handler = self.handlers.get(&message.message_type.to_string())
            .ok_or_else(|| MCPError::Protocol(ProtocolError::HandlerNotFound(format!("No handler for message type {:?}", message.message_type))))?;
        
        handler.handle(message).await
    }

    /// Validates a message according to protocol rules
    pub fn validate_message(&self, message: &MCPMessage) -> ValidationResult {
        // Basic validation: ensure message has the right format
        if message.id.0.is_empty() {
            return Err(MCPError::Protocol(ProtocolError::InvalidFormat("Message ID is missing".to_string())));
        }
        
        // Check protocol version compatibility if present in the payload
        if let Some(version) = message.payload.get("protocol_version") {
            if version.as_str() != Some(&self.config.version) {
                return Err(MCPError::Protocol(ProtocolError::InvalidVersion(
                    format!("Protocol version mismatch: message has {}, expected {}", 
                        version, self.config.version)
                )));
            }
        }
        
        Ok(())
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
    #[must_use] pub fn new(config: ProtocolConfig) -> Self {
        Self { config }
    }

    /// Creates a new factory with the specified configuration
    #[must_use] pub fn with_config(config: ProtocolConfig) -> Self {
        Self { config }
    }

    /// Creates a new protocol instance
    #[must_use] pub fn create_protocol(&self) -> MCPProtocolBase {
        MCPProtocolBase::new(self.config.clone())
    }

    /// Creates a new protocol instance with custom dependencies
    #[must_use] pub fn create_protocol_with_dependencies(
        &self,
        handlers: HashMap<String, Box<dyn CommandHandler>>,
        initial_state: Value,
    ) -> MCPProtocolBase {
        MCPProtocolBase::with_dependencies(
            self.config.clone(),
            handlers,
            initial_state,
        )
    }

    /// Creates a new protocol adapter
    #[must_use] pub fn create_protocol_adapter(&self) -> Arc<MCPProtocolAdapter> {
        let protocol = self.create_protocol_with_dependencies(
            HashMap::new(),
            Value::Null,
        );
        Arc::new(MCPProtocolAdapter::with_protocol(protocol))
    }
}

impl Default for MCPProtocolFactory {
    fn default() -> Self {
        Self::new(ProtocolConfig::default())
    }
}

/// Core trait for MCP protocol implementation
#[async_trait::async_trait]
pub trait MCPProtocol: Send + Sync {
    /// Handles an incoming message according to the protocol
    async fn handle_message(&self, msg: MCPMessage) -> ProtocolResult;
    
    /// Validates a message according to protocol rules
    async fn validate_message(&self, msg: &MCPMessage) -> ValidationResult;
    
    /// Routes a message to its appropriate handler
    async fn route_message(&self, msg: &MCPMessage) -> RoutingResult;
    
    /// Sets the protocol state
    async fn set_state(&self, new_state: ProtocolState) -> Result<()>;
    
    /// Gets the current protocol state
    async fn get_state(&self) -> Result<ProtocolState>;
    
    /// Gets the protocol version
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