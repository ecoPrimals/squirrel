//! MCP Protocol module
//!
//! This module implements the core protocol functionality for the Machine Context Protocol (MCP).
//! It handles message processing, protocol state management, and command execution.

use std::sync::Arc;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use tokio::sync::RwLock;
use crate::mcp::error::{MCPError, Result};
use crate::mcp::types::{
    MCPMessage,
    ProtocolVersion,
    ProtocolState,
    MCPCommand,
    MCPResponse,
    MessageType,
    SecurityLevel,
    CompressionFormat,
    EncryptionFormat,
    MessageMetadata,
    ResponseStatus,
};
use serde_json::Value;

// Import core error types
use crate::core::error::{CoreError, CoreResult};

// Define protocol specific result types
pub type ProtocolResult<T> = Result<T>;
pub type ValidationResult = Result<()>;
pub type RoutingResult = Result<()>;

pub mod adapter;
pub use adapter::{MCPProtocolAdapter, create_protocol_adapter, create_protocol_adapter_with_protocol};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolConfig {
    pub version: ProtocolVersion,
    pub state: ProtocolState,
    pub max_message_size: usize,
    pub timeout_ms: u64,
}

impl Default for ProtocolConfig {
    fn default() -> Self {
        Self {
            version: ProtocolVersion::V1,
            state: ProtocolState::Initialized,
            max_message_size: 1024 * 1024, // 1MB
            timeout_ms: 5000, // 5 seconds
        }
    }
}

#[async_trait::async_trait]
pub trait CommandHandler: Send + Sync {
    async fn handle(&self, message: &MCPMessage) -> Result<MCPMessage>;
}

pub struct MCPProtocol {
    config: ProtocolConfig,
    handlers: HashMap<String, Box<dyn CommandHandler>>,
    state: Value,
}

impl MCPProtocol {
    pub fn new(config: ProtocolConfig) -> Self {
        Self {
            config,
            handlers: HashMap::new(),
            state: Value::Null,
        }
    }

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

    pub fn get_config(&self) -> &ProtocolConfig {
        &self.config
    }
}

pub struct MCPProtocolFactory {
    config: ProtocolConfig,
}

impl MCPProtocolFactory {
    pub fn new(config: ProtocolConfig) -> Self {
        Self { config }
    }

    pub fn with_config(config: ProtocolConfig) -> Self {
        Self { config }
    }

    pub fn create_protocol_with_dependencies(
        &self,
        handlers: HashMap<String, Box<dyn CommandHandler>>,
        initial_state: Value,
    ) -> Arc<MCPProtocol> {
        Arc::new(MCPProtocol::with_dependencies(
            self.config.clone(),
            handlers,
            initial_state,
        ))
    }

    pub fn create_protocol_adapter(&self) -> Arc<MCPProtocolAdapter> {
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

#[async_trait::async_trait]
pub trait MCPProtocol: Send + Sync {
    /// Handles an incoming message according to the protocol
    async fn handle_message(&self, msg: MCPMessage) -> ProtocolResult<MCPResponse>;
    
    /// Validates a message according to protocol rules
    async fn validate_message(&self, msg: &MCPMessage) -> ValidationResult;
    
    /// Routes a message to its appropriate handler
    async fn route_message(&self, msg: MCPMessage) -> RoutingResult;
}

#[async_trait::async_trait]
pub trait MessageHandler: Send + Sync {
    /// Handles a specific type of message
    async fn handle(&self, message: &MCPMessage) -> ProtocolResult<MCPResponse>;
    
    /// Returns the message types this handler can process
    fn supported_types(&self) -> Vec<MessageType>;
}

#[derive(Debug)]
pub struct MCPProtocolImpl {
    version: ProtocolVersion,
    state: Arc<RwLock<ProtocolState>>,
    security_level: SecurityLevel,
    compression: CompressionFormat,
    encryption: EncryptionFormat,
    handlers: Arc<RwLock<HashMap<MessageType, Box<dyn MessageHandler>>>>,
}

impl MCPProtocolImpl {
    pub fn new(version: ProtocolVersion, security_level: SecurityLevel) -> Self {
        Self {
            version,
            state: Arc::new(RwLock::new(ProtocolState::Initialized)),
            security_level,
            compression: CompressionFormat::None,
            encryption: EncryptionFormat::None,
            handlers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register_handler(&self, handler: Box<dyn MessageHandler>) -> Result<()> {
        let mut handlers = self.handlers.write().await;
        for msg_type in handler.supported_types() {
            handlers.insert(msg_type, handler.clone());
        }
        Ok(())
    }

    async fn get_handler(&self, msg_type: MessageType) -> Result<Box<dyn MessageHandler>> {
        let handlers = self.handlers.read().await;
        handlers
            .get(&msg_type)
            .cloned()
            .ok_or_else(|| MCPError::UnknownMessageType(format!("No handler for message type: {:?}", msg_type)))
    }
}

#[async_trait::async_trait]
impl MCPProtocol for MCPProtocolImpl {
    async fn handle_message(&self, msg: MCPMessage) -> ProtocolResult<MCPResponse> {
        // Validate message first
        self.validate_message(&msg).await?;
        
        // Route and handle the message
        self.route_message(msg).await?;
        
        // Get appropriate handler
        let handler = self.get_handler(msg.message_type).await?;
        
        // Handle the message
        handler.handle(&msg).await
    }
    
    async fn validate_message(&self, msg: &MCPMessage) -> ValidationResult {
        // Check protocol version compatibility
        if msg.protocol_version != self.version {
            return Err(MCPError::VersionMismatch {
                expected: self.version.clone(),
                received: msg.protocol_version.clone(),
            });
        }
        
        // Validate message structure
        if msg.payload.is_empty() {
            return Err(MCPError::InvalidMessage("Empty message payload".to_string()));
        }
        
        // Validate security requirements
        if msg.security_level < self.security_level {
            return Err(MCPError::SecurityLevelTooLow {
                required: self.security_level,
                provided: msg.security_level,
            });
        }
        
        Ok(())
    }
    
    async fn route_message(&self, msg: MCPMessage) -> RoutingResult {
        // Check protocol state
        let state = self.state.read().await;
        if *state != ProtocolState::Ready {
            return Err(MCPError::InvalidState(format!(
                "Protocol not ready. Current state: {:?}",
                *state
            )));
        }
        
        // Verify handler exists
        if !self.handlers.read().await.contains_key(&msg.message_type) {
            return Err(MCPError::UnknownMessageType(format!(
                "No handler registered for message type: {:?}",
                msg.message_type
            )));
        }
        
        Ok(())
    }
}

// Re-export common types
pub use crate::mcp::types::{
    MCPMessage,
    ProtocolVersion,
    ProtocolState,
    CompressionFormat,
    EncryptionFormat,
    MessageMetadata,
    ResponseStatus,
};

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_protocol_validation() {
        let protocol = MCPProtocolImpl::new(
            ProtocolVersion::new(1, 0),
            SecurityLevel::Standard,
        );
        
        let valid_msg = MCPMessage {
            protocol_version: ProtocolVersion::new(1, 0),
            message_type: MessageType::Command,
            security_level: SecurityLevel::Standard,
            payload: vec![1, 2, 3],
            metadata: MessageMetadata::default(),
        };
        
        assert!(protocol.validate_message(&valid_msg).await.is_ok());
        
        let invalid_msg = MCPMessage {
            protocol_version: ProtocolVersion::new(2, 0),
            message_type: MessageType::Command,
            security_level: SecurityLevel::Standard,
            payload: vec![],
            metadata: MessageMetadata::default(),
        };
        
        assert!(protocol.validate_message(&invalid_msg).await.is_err());
    }
}
}