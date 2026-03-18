// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

// transport/types.rs

// BearDog handles security: // use crate::security::types::EncryptionFormat;
use crate::types::CompressionFormat;
use crate::types::EncryptionFormat;
use bytes::Bytes;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;

/// Metadata associated with a transport connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportMetadata {
    /// Unique connection ID
    pub connection_id: String,
    /// Remote address of the connection
    pub remote_address: Option<SocketAddr>,
    /// Local address of the connection
    pub local_address: Option<SocketAddr>,
    /// Timestamp when the connection was established
    pub connected_at: DateTime<Utc>,
    /// Last activity timestamp
    pub last_activity: DateTime<Utc>,
    /// Encryption format used, if any
    pub encryption_format: Option<EncryptionFormat>,
    /// Compression format used, if any
    pub compression_format: Option<CompressionFormat>,
    /// Additional metadata
    pub additional_info: HashMap<String, String>,
}

/// State of a transport connection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionState {
    /// Connection is disconnected
    Disconnected,
    /// Connection attempt is in progress
    Connecting,
    /// Connection is established and active
    Connected,
    /// Connection is being closed
    Disconnecting,
    /// Connection encountered an error
    Error,
}

/// Represents events that can occur on a transport.
#[derive(Debug, Clone)]
pub enum TransportEvent {
    /// Connection was established
    Connected(TransportMetadata),
    /// Connection was closed, optionally with a reason
    Disconnected(Option<String>),
    /// Raw message bytes were received
    MessageReceived(Bytes),
    /// An error occurred
    Error(String),
}

/// Represents the type of transport.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransportType {
    /// TCP socket transport
    Tcp,
    /// WebSocket transport
    WebSocket,
    /// Standard I/O transport
    Stdio,
    /// In-memory transport for testing
    Memory,
    /// Unknown or unspecified transport type
    Unknown,
}

/// Transport configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportConfig {
    /// Encryption format to use
    pub encryption: EncryptionFormat,
    /// Connection timeout in milliseconds
    pub timeout_ms: u64,
    /// Maximum allowed message size in bytes
    pub max_message_size: usize,
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            encryption: EncryptionFormat::None,
            timeout_ms: 30000,
            max_message_size: 1024 * 1024, // 1MB
        }
    }
}

/// Transport message wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportMessage {
    /// Unique message identifier
    pub id: String,
    /// Raw message payload
    pub payload: Bytes,
    /// Message metadata
    pub metadata: TransportMessageMetadata,
}

/// Transport message metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportMessageMetadata {
    /// MIME content type of the message
    pub content_type: String,
    /// Character encoding, if applicable
    pub encoding: Option<String>,
    /// Compression format, if applicable
    pub compression: Option<String>,
    /// Additional headers
    pub headers: HashMap<String, String>,
}

impl Default for TransportMessageMetadata {
    fn default() -> Self {
        Self {
            content_type: "application/json".to_string(),
            encoding: None,
            compression: None,
            headers: HashMap::new(),
        }
    }
}
