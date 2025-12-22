//! Dynamic port types.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::hash::Hash;

/// Dynamic port information for songbird-managed ports
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DynamicPortInfo {
    /// Port assigned by songbird
    pub assigned_port: u16,
    /// Port type (HTTP, HTTPS, WebSocket, etc.)
    pub port_type: PortType,
    /// Port status
    pub status: PortStatus,
    /// Port assignment timestamp
    pub assigned_at: DateTime<Utc>,
    /// Port lease duration
    pub lease_duration: Duration,
}

/// Port type enumeration
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PortType {
    /// HTTP port
    Http,
    /// HTTPS port
    Https,
    /// WebSocket port
    WebSocket,
    /// gRPC port
    Grpc,
    /// Custom port type
    Custom(String),
}

/// Port status
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PortStatus {
    /// Port is active and available
    Active,
    /// Port is reserved but not yet active
    Reserved,
    /// Port is being releasing
    Releasing,
    /// Port is expired and should be cleaned up
    Expired,
}
