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

#[cfg(test)]
mod tests {
    use super::{
        ConnectionState, TransportConfig, TransportEvent, TransportMessage,
        TransportMessageMetadata, TransportMetadata, TransportType,
    };
    use crate::types::{CompressionFormat, EncryptionFormat};
    use bytes::Bytes;
    use chrono::Utc;
    use std::collections::HashMap;
    use std::net::SocketAddr;
    use std::str::FromStr;

    #[test]
    fn transport_metadata_serde_roundtrip() {
        let meta = TransportMetadata {
            connection_id: "c1".into(),
            remote_address: Some(SocketAddr::from_str("127.0.0.1:9000").unwrap()),
            local_address: Some(SocketAddr::from_str("127.0.0.1:9001").unwrap()),
            connected_at: Utc::now(),
            last_activity: Utc::now(),
            encryption_format: Some(EncryptionFormat::Aes256Gcm),
            compression_format: Some(CompressionFormat::Gzip),
            additional_info: HashMap::from([("k".into(), "v".into())]),
        };
        let json = serde_json::to_string(&meta).unwrap();
        let back: TransportMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(back.connection_id, meta.connection_id);
        let _ = format!("{meta:?}");
        let cloned = meta.clone();
        assert_eq!(cloned.connection_id, meta.connection_id);
    }

    #[test]
    fn connection_state_and_transport_type_serde() {
        for s in [
            ConnectionState::Disconnected,
            ConnectionState::Connecting,
            ConnectionState::Connected,
            ConnectionState::Disconnecting,
            ConnectionState::Error,
        ] {
            let json = serde_json::to_string(&s).unwrap();
            let back: ConnectionState = serde_json::from_str(&json).unwrap();
            assert_eq!(s, back);
            let _ = format!("{s:?}");
        }
        for t in [
            TransportType::Tcp,
            TransportType::WebSocket,
            TransportType::Stdio,
            TransportType::Memory,
            TransportType::Unknown,
        ] {
            let json = serde_json::to_string(&t).unwrap();
            let back: TransportType = serde_json::from_str(&json).unwrap();
            assert_eq!(t, back);
        }
    }

    #[test]
    fn transport_event_and_transport_message_roundtrip() {
        let tm = TransportMetadata {
            connection_id: "c".into(),
            remote_address: None,
            local_address: None,
            connected_at: Utc::now(),
            last_activity: Utc::now(),
            encryption_format: None,
            compression_format: None,
            additional_info: HashMap::new(),
        };
        let ev = TransportEvent::Connected(tm);
        let _ = format!("{ev:?}");
        let ev2 = TransportEvent::Disconnected(Some("bye".into()));
        let _ = format!("{ev2:?}");
        let ev3 = TransportEvent::MessageReceived(Bytes::from_static(b"hi"));
        let _ = format!("{ev3:?}");
        let ev4 = TransportEvent::Error("e".into());
        let _ = format!("{ev4:?}");
        assert!(matches!(ev3, TransportEvent::MessageReceived(_)));

        let msg = TransportMessage {
            id: "mid".into(),
            payload: Bytes::from_static(b"payload"),
            metadata: TransportMessageMetadata {
                content_type: "application/json".into(),
                encoding: Some("utf-8".into()),
                compression: Some("gzip".into()),
                headers: HashMap::from([("h".into(), "v".into())]),
            },
        };
        let json = serde_json::to_string(&msg).unwrap();
        let back: TransportMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(back.id, msg.id);
    }

    #[test]
    fn transport_config_and_message_metadata_defaults() {
        let d = TransportConfig::default();
        assert_eq!(d.encryption, EncryptionFormat::None);
        let d2 = d.clone();
        assert_eq!(d2.timeout_ms, d.timeout_ms);
        let m = TransportMessageMetadata::default();
        assert_eq!(m.content_type, "application/json");
        let json = serde_json::to_string(&m).unwrap();
        let back: TransportMessageMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(back.content_type, m.content_type);
    }
}
