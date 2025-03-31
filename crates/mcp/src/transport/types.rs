// transport/types.rs

use crate::security::types::EncryptionFormat;
use crate::types::CompressionFormat;
use std::net::SocketAddr;
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

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
    Connected(TransportMetadata),
    Disconnected(Option<String>), // Optional reason
    MessageReceived(Vec<u8>), // Raw bytes received
    Error(String),
}

/// Represents the type of transport.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransportType {
    Tcp,
    WebSocket,
    Stdio,
    Memory,
    Unknown,
}