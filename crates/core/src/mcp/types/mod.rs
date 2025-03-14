use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::collections::HashMap;
use std::fmt;
use crate::error::{MCPError, Result};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPMessage {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub message_type: MessageType,
    pub metadata: MessageMetadata,
    pub payload: Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageType {
    Command,
    Response,
    Event,
    Request,
}

impl fmt::Display for MessageType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MessageType::Command => write!(f, "Command"),
            MessageType::Response => write!(f, "Response"),
            MessageType::Event => write!(f, "Event"),
            MessageType::Request => write!(f, "Request"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecurityLevel {
    None,
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageMetadata {
    pub correlation_id: Option<String>,
    pub security_level: SecurityLevel,
    pub compression: CompressionFormat,
    pub encryption: EncryptionFormat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompressionFormat {
    None,
    Gzip,
    Zstd,
    Lz4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EncryptionFormat {
    None,
    Aes256Gcm,
    ChaCha20Poly1305,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResponseStatus {
    Success,
    Error,
    Pending,
    Timeout,
}

impl MCPMessage {
    pub fn new(message_type: MessageType, command: impl Into<String>, payload: serde_json::Value) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            message_type,
            metadata: MessageMetadata::default(),
            payload,
        }
    }

    pub fn with_correlation_id(mut self, correlation_id: impl Into<String>) -> Self {
        self.metadata.correlation_id = Some(correlation_id.into());
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPCommand {
    pub name: String,
    pub args: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPResponse {
    pub status: ResponseStatus,
    pub data: Value,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl fmt::Display for ProtocolVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProtocolState {
    Initialized,
    Connected,
    Authenticated,
    Ready,
    Error,
    Closed,
}

impl ProtocolVersion {
    pub fn major(&self) -> u8 {
        self.major as u8
    }

    pub fn minor(&self) -> u8 {
        self.minor as u8
    }

    pub fn patch(&self) -> u8 {
        self.patch as u8
    }

    pub fn is_compatible(&self, other: &ProtocolVersion) -> bool {
        self.major() == other.major() && self.minor() >= other.minor()
    }
}

impl Default for ProtocolVersion {
    fn default() -> Self {
        Self {
            major: 1,
            minor: 0,
            patch: 0,
        }
    }
}

impl Default for MessageMetadata {
    fn default() -> Self {
        Self {
            correlation_id: None,
            security_level: SecurityLevel::None,
            compression: CompressionFormat::None,
            encryption: EncryptionFormat::None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolConfig {
    pub version: ProtocolVersion,
    pub max_message_size: usize,
    pub timeout_ms: u64,
    pub retry_count: u32,
}

impl Default for ProtocolConfig {
    fn default() -> Self {
        Self {
            version: ProtocolVersion::default(),
            max_message_size: 1024 * 1024, // 1MB
            timeout_ms: 5000,
            retry_count: 3,
        }
    }
} 