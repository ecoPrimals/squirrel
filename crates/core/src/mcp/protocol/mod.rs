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