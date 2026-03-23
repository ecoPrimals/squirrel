// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Common types used throughout the MCP system.
//!
//! This module contains the core data structures and enumerations that define the Machine Context Protocol (MCP).
//! It includes message types, security levels, protocol versioning, and other fundamental structures
//! that are used across the MCP system for communication between components.
//!
//! # Core Message Types
//!
//! The primary message types in this module include:
//! - `MCPMessage`: The core message structure for MCP communications
//! - `CommandRequestMessage`: For issuing commands to the system
//! - `CommandResponseMessage`: For returning results from commands
//! - `EventMessage`: For publishing one-way event notifications
//! - `HandshakeMessage`: For establishing and negotiating connections
//!
//! # Identity Types
//!
//! MCP uses several wrapper types to provide type safety for identifiers:
//! - `MessageId`: Uniquely identifies messages within the system
//! - `UserId`: Identifies users across the system
//! - `AccountId`: Identifies accounts within the system
//! - `SessionToken`: Represents an active user session
//! - `AuthToken`: Used for authentication purposes
//!
//! # Protocol Configuration
//!
//! The protocol configuration is managed through:
//! - `ProtocolVersion`: Represents the protocol version with major and minor components
//! - `ProtocolState`: Tracks the current state of the protocol (ready, error, etc.)
//!
//! # Security
//!
//! Security-related types include:
//! - `SecurityLevel`: Defines the required security level for operations
//! - `EncryptionFormat`: Specifies encryption algorithms for secure communications
//! - `CompressionFormat`: Defines compression methods for efficient data transfer
//! - `UserRole`: Defines access control roles for authorization
//!
//! # Examples
//!
//! Creating a basic MCP message:
//!
//! ```ignore
//! use squirrel_mcp::protocol::types::{MCPMessage, MessageId, MessageType};
//! use squirrel_mcp::protocol::types::ProtocolVersion;
//! // BearDog handles security: // use crate::mcp::security::types::SecurityMetadata;
//! use serde_json::json;
//! use chrono::Utc;
//!
//! let message = MCPMessage {
//!     id: MessageId("msg123".to_string()),
//!     type_: MessageType::Command,
//!     payload: json!({
//!         "command": "execute",
//!         "parameters": {
//!             "tool": "file_reader",
//!             "args": ["path/to/file"]
//!         }
//!     }),
//!     metadata: Some(json!({})),
//!     security: SecurityMetadata::default(),
//!     timestamp: Utc::now(),
//!     version: ProtocolVersion::new(1, 0),
//!     trace_id: Some("trace-123".to_string()),
//! };
//! ```
//!
//! Processing a response:
//!
//! ```ignore
//! use squirrel_mcp::types::{MCPResponse, ResponseStatus};
//! use serde_json::json;
//!
//! // Assuming we received a response from the system
//! fn process_response(response: MCPResponse) {
//!     match response.status {
//!         ResponseStatus::Success => {
//!             // Process successful response
//!             println!("Operation succeeded!");
//!         },
//!         ResponseStatus::Error => {
//!             // Handle error
//!             println!("Error: {}", response.error_message.unwrap_or_default());
//!         },
//!         ResponseStatus::Pending => {
//!             // Operation is still in progress
//!             println!("Operation is pending, check back later");
//!         },
//!     }
//! }
//! ```

use crate::protocol::types::MessageId;
use serde::{Deserialize, Serialize};
use serde_json;

/// Compression formats supported by the MCP protocol
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CompressionFormat {
    /// No compression
    #[default]
    None,
    /// GZIP compression
    Gzip,
    /// LZ4 compression
    Lz4,
    /// Zstandard compression
    Zstd,
    /// Custom compression format
    Custom(u8),
}

impl std::fmt::Display for CompressionFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "none"),
            Self::Gzip => write!(f, "gzip"),
            Self::Lz4 => write!(f, "lz4"),
            Self::Zstd => write!(f, "zstd"),
            Self::Custom(id) => write!(f, "custom-{id}"),
        }
    }
}

/// Encryption formats supported by the MCP protocol
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum EncryptionFormat {
    /// No encryption
    #[default]
    None,
    /// AES-256-GCM encryption
    Aes256Gcm,
    /// ChaCha20-Poly1305 encryption
    ChaCha20Poly1305,
    /// RSA encryption
    Rsa,
    /// Custom encryption format
    Custom(u8),
}

impl std::fmt::Display for EncryptionFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "none"),
            Self::Aes256Gcm => write!(f, "aes256-gcm"),
            Self::ChaCha20Poly1305 => write!(f, "chacha20-poly1305"),
            Self::Rsa => write!(f, "rsa"),
            Self::Custom(id) => write!(f, "custom-{id}"),
        }
    }
}

/// Message metadata for MCP messages.
///
/// This structure provides additional context for messages transmitted
/// through the MCP system. It includes timing information and routing
/// details that can be used for debugging, auditing, and message delivery.
///
/// # Usage
///
/// Message metadata is attached to all MCP messages and can be used to:
/// - Track message flow through the system
/// - Measure performance and latency
/// - Debug message routing issues
/// - Implement message TTL (time-to-live) logic
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MessageMetadata {
    /// Timestamp of the message (Unix timestamp in milliseconds)
    pub timestamp: u64,
    /// Source of the message (sender identifier)
    pub source: String,
    /// Destination of the message (recipient identifier)
    pub destination: String,
}

/// Status of a response message.
///
/// This enumeration represents the possible states of a response
/// to a command or request. It indicates whether the operation
/// succeeded, failed, or is still in progress.
///
/// # Usage
///
/// Response status is used to determine how to handle a response:
/// - Success: The operation completed successfully
/// - Error: The operation failed
/// - Pending: The operation is still being processed
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ResponseStatus {
    /// Success: The operation completed successfully
    #[default]
    Success,
    /// Error: The operation failed
    Error,
    /// Pending: The operation is still being processed
    Pending,
}

/// Command Request Message for executing commands via MCP.
///
/// This message type is used to request execution of a command with optional arguments.
///
/// # Examples
///
/// ```
/// use squirrel_mcp::protocol::types::MessageId;
/// use squirrel_mcp::types::CommandRequestMessage;
///
/// let request = CommandRequestMessage {
///     id: MessageId("cmd123".to_string()),
///     command: "read_file".to_string(),
///     args: Some(serde_json::json!({"path": "example.txt"})),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandRequestMessage {
    /// Unique identifier for the message
    pub id: MessageId,
    /// Command to execute
    pub command: String,
    /// Optional arguments for the command
    pub args: Option<serde_json::Value>,
}

/// Command Response Message for returning command execution results.
///
/// This message type is sent in response to a `CommandRequestMessage` and
/// includes the status of the command execution and optional result data.
///
/// # Examples
///
/// ```
/// use squirrel_mcp::protocol::types::MessageId;
/// use squirrel_mcp::types::CommandResponseMessage;
///
/// let response = CommandResponseMessage {
///     id: MessageId("resp123".to_string()),
///     command_id: MessageId("cmd123".to_string()),
///     status: "success".to_string(),
///     result: Some(serde_json::json!({"content": "file contents..."})),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResponseMessage {
    /// Unique identifier for the response
    pub id: MessageId,
    /// ID of the command request this is responding to
    pub command_id: MessageId,
    /// Status of the command execution
    pub status: String,
    /// Optional result data from the command
    pub result: Option<serde_json::Value>,
}

/// Event Message for publishing events in the MCP system.
///
/// This message type is used for one-way event notifications that don't require responses.
///
/// # Examples
///
/// ```
/// use squirrel_mcp::protocol::types::MessageId;
/// use squirrel_mcp::types::EventMessage;
///
/// let event = EventMessage {
///     id: MessageId("evt123".to_string()),
///     event_type: "file_change".to_string(),
///     data: serde_json::json!({
///         "path": "example.txt",
///         "operation": "write"
///     }),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMessage {
    /// Unique identifier for the event
    pub id: MessageId,
    /// Type of event being published
    pub event_type: String,
    /// Event data payload
    pub data: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::{
        CommandRequestMessage, CommandResponseMessage, CompressionFormat, EncryptionFormat,
        EventMessage, MessageMetadata, ResponseStatus,
    };
    use crate::protocol::types::MessageId;

    #[test]
    fn compression_format_roundtrip_display_default() {
        let variants = [
            CompressionFormat::None,
            CompressionFormat::Gzip,
            CompressionFormat::Lz4,
            CompressionFormat::Zstd,
            CompressionFormat::Custom(9),
        ];
        for v in variants {
            let s = format!("{v}");
            assert!(!s.is_empty());
            let json = serde_json::to_string(&v).unwrap();
            let back: CompressionFormat = serde_json::from_str(&json).unwrap();
            assert_eq!(v, back);
            let c2 = v;
            assert_eq!(v, c2);
        }
        assert_eq!(CompressionFormat::default(), CompressionFormat::None);
    }

    #[test]
    fn encryption_format_roundtrip_display_default() {
        let variants = [
            EncryptionFormat::None,
            EncryptionFormat::Aes256Gcm,
            EncryptionFormat::ChaCha20Poly1305,
            EncryptionFormat::Rsa,
            EncryptionFormat::Custom(3),
        ];
        for v in variants {
            let _ = format!("{v}");
            let json = serde_json::to_string(&v).unwrap();
            let back: EncryptionFormat = serde_json::from_str(&json).unwrap();
            assert_eq!(v, back);
        }
        assert_eq!(EncryptionFormat::default(), EncryptionFormat::None);
    }

    #[test]
    fn message_metadata_default_and_serde() {
        let m = MessageMetadata::default();
        let json = serde_json::to_string(&m).unwrap();
        let back: MessageMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(back.timestamp, m.timestamp);
        let _ = format!("{m:?}");
        let c = m.clone();
        assert_eq!(c.source, m.source);
    }

    #[test]
    fn response_status_variants_and_serde() {
        for s in [
            ResponseStatus::Success,
            ResponseStatus::Error,
            ResponseStatus::Pending,
        ] {
            let json = serde_json::to_string(&s).unwrap();
            let back: ResponseStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(s, back);
            let _ = format!("{s:?}");
        }
        assert_eq!(ResponseStatus::default(), ResponseStatus::Success);
    }

    #[test]
    fn command_request_response_and_event_messages_serde() {
        let mid = MessageId("m1".into());
        let req = CommandRequestMessage {
            id: mid.clone(),
            command: "cmd".into(),
            args: Some(serde_json::json!({"a": 1})),
        };
        let json = serde_json::to_string(&req).unwrap();
        let r2: CommandRequestMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(r2.command, req.command);

        let resp = CommandResponseMessage {
            id: MessageId("r1".into()),
            command_id: mid,
            status: "ok".into(),
            result: Some(serde_json::json!({})),
        };
        let json = serde_json::to_string(&resp).unwrap();
        let r2: CommandResponseMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(r2.status, resp.status);

        let ev = EventMessage {
            id: MessageId("e1".into()),
            event_type: "evt".into(),
            data: serde_json::json!({"x": true}),
        };
        let json = serde_json::to_string(&ev).unwrap();
        let e2: EventMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(e2.event_type, ev.event_type);
        let _ = format!("{ev:?}");
        assert_eq!(ev.clone().id, ev.id);
    }
}
