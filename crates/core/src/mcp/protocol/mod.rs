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

// Import common types
use crate::mcp::types::{
    CompressionFormat,
    EncryptionFormat,
    MessageMetadata,
    ResponseStatus,
};

// Define protocol specific result type
pub type ProtocolResult<T> = Result<T>;

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

use std::pin::Pin;
use std::future::Future;

pub trait CommandHandler: Send + Sync {
    fn handle<'a>(&'a self, message: &'a MCPMessage) -> Pin<Box<dyn Future<Output = Result<MCPMessage>> + Send + 'a>>;
}

pub struct MCPProtocol {
    version: ProtocolVersion,
    state: ProtocolState,
    security_level: SecurityLevel,
    compression: CompressionFormat,
    encryption: EncryptionFormat,
    handlers: HashMap<String, Box<dyn CommandHandler>>,
}

impl MCPProtocol {
    pub fn new(version: ProtocolVersion, security_level: SecurityLevel) -> Self {
        Self {
            version,
            state: ProtocolState::Initialized,
            security_level,
            compression: CompressionFormat::None,
            encryption: EncryptionFormat::None,
            handlers: HashMap::new(),
        }
    }

    pub fn version(&self) -> &ProtocolVersion {
        &self.version
    }

    pub fn state(&self) -> ProtocolState {
        self.state
    }

    pub fn security_level(&self) -> SecurityLevel {
        self.security_level
    }

    pub fn compression(&self) -> CompressionFormat {
        self.compression
    }

    pub fn encryption(&self) -> EncryptionFormat {
        self.encryption
    }

    pub fn set_state(&mut self, state: ProtocolState) {
        self.state = state;
    }

    pub fn set_compression(&mut self, compression: CompressionFormat) {
        self.compression = compression;
    }

    pub fn set_encryption(&mut self, encryption: EncryptionFormat) {
        self.encryption = encryption;
    }

    pub fn get_state(&self) -> Result<Value> {
        Ok(serde_json::to_value(&self)?)
    }

    pub async fn handle_message(&self, message: &MCPMessage) -> Result<MCPMessage> {
        let command = message.command.as_deref().ok_or_else(|| MCPError::Command("No command specified".to_string()))?;
        let handler = self.handlers.get(command).ok_or_else(|| MCPError::Command(format!("Unknown command: {}", command)))?;
        handler.handle(message).await
    }

    pub fn register_handler(&mut self, command: String, handler: Box<dyn CommandHandler>) {
        self.handlers.insert(command, handler);
    }
}

pub trait MessageHandler: Send + Sync {
    fn handle(&self, message: &MCPMessage) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<MCPResponse>> + Send + '_>>;
}

#[derive(Debug, Clone)]
pub enum ProtocolError {
    UnknownCommand(String),
    InvalidMessage(String),
    InvalidState(String),
    ConnectionError(String),
    Other(String),
}

// Export common types
pub type ProtocolResult<T> = std::result::Result<T, ProtocolError>;