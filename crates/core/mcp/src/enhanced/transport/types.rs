// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Transport configuration, messages, and the [`TransportService`] trait.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

/// Transport Type enumeration
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum TransportType {
    WebSocket,
    Tarpc,
    TCP,
    HTTP,
    UDP,
}

/// Transport configuration for UnifiedTransport
#[derive(Debug, Clone)]
pub struct TransportConfig {
    /// Supported transport types
    pub supported_transports: Vec<TransportType>,
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            supported_transports: vec![
                TransportType::WebSocket,
                TransportType::Tarpc,
                TransportType::TCP,
            ],
        }
    }
}

/// Transport Service trait
#[async_trait::async_trait]
pub trait TransportService: Send + Sync {
    /// Start the transport service
    async fn start(&self, addr: SocketAddr) -> crate::error::Result<()>;

    /// Stop the transport service
    async fn stop(&self) -> crate::error::Result<()>;

    /// Send message
    async fn send_message(
        &self,
        connection_id: &str,
        message: TransportMessage,
    ) -> crate::error::Result<()>;

    /// Get service type
    fn service_type(&self) -> TransportType;

    /// Get service metrics
    async fn get_metrics(&self) -> TransportServiceMetrics;
}

/// Transport Message - Unified message format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportMessage {
    /// Message ID
    pub id: String,

    /// Message type
    pub message_type: MessageType,

    /// Payload
    pub payload: serde_json::Value,

    /// Metadata
    pub metadata: HashMap<String, String>,

    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Message Type enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    MCPRequest,
    MCPResponse,
    MCPNotification,
    StreamChunk,
    Heartbeat,
    Control,
}

/// Connection information
#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    /// Connection ID
    pub id: String,

    /// Transport type
    pub transport_type: TransportType,

    /// Remote address
    pub remote_addr: SocketAddr,

    /// Connection state
    pub state: ConnectionState,

    /// Created timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// Last activity
    pub last_activity: chrono::DateTime<chrono::Utc>,

    /// Metadata
    pub metadata: HashMap<String, String>,
}

/// Connection State enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionState {
    Connecting,
    Connected,
    Authenticating,
    Authenticated,
    Disconnecting,
    Disconnected,
    Error(String),
}

#[derive(Debug)]
pub struct ConnectionHandler {
    pub connection_id: String,
    pub transport_type: TransportType,
    pub sender: tokio::sync::mpsc::Sender<TransportMessage>,
}

#[derive(Debug, Clone)]
pub enum LoadBalancingStrategy {
    RoundRobin,
    LeastConnections,
    WeightedRoundRobin,
    Random,
}

#[derive(Debug, Clone)]
pub struct RoutingEntry {
    pub target: String,
    pub transport_type: TransportType,
    pub weight: u32,
}

#[derive(Debug, Clone, Default)]
pub struct TransportMetrics {
    pub total_connections: u64,
    pub active_connections: u64,
    pub messages_sent: u64,
    pub messages_received: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub connection_errors: u64,
    pub message_errors: u64,
}

#[derive(Debug, Clone, Default)]
pub struct TransportServiceMetrics {
    pub connections: u64,
    pub messages_sent: u64,
    pub messages_received: u64,
    pub bytes_transferred: u64,
    pub errors: u64,
    pub uptime_seconds: u64,
}

impl TransportMessage {
    pub fn new(message_type: MessageType, payload: serde_json::Value) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            message_type,
            payload,
            metadata: HashMap::new(),
            timestamp: chrono::Utc::now(),
        }
    }

    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}
