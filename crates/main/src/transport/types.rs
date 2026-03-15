// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Transport types for MCP
//!
//! Basic transport types for MCP protocol.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Connection metadata for transport
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionMetadata {
    /// Connection ID
    pub connection_id: String,
    /// Remote address
    pub remote_address: Option<String>,
    /// Local address
    pub local_address: Option<String>,
    /// Connected timestamp
    pub connected_at: DateTime<Utc>,
    /// Protocol version
    pub protocol_version: String,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl Default for ConnectionMetadata {
    fn default() -> Self {
        Self {
            connection_id: Uuid::new_v4().to_string(),
            remote_address: None,
            local_address: None,
            connected_at: Utc::now(),
            protocol_version: "1.0".to_string(),
            metadata: HashMap::new(),
        }
    }
}

/// Transport configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportConfig {
    /// Buffer size for transport
    pub buffer_size: usize,
    /// Connection timeout in seconds
    pub connection_timeout: u64,
    /// Keep-alive enabled
    pub keep_alive: bool,
    /// Maximum frame size
    pub max_frame_size: usize,
    /// Compression enabled
    pub compression: bool,
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            buffer_size: 8192,
            connection_timeout: 30,
            keep_alive: true,
            max_frame_size: 65536,
            compression: false,
        }
    }
}

/// Frame metadata for transport
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameMetadata {
    /// Frame ID
    pub frame_id: String,
    /// Frame type
    pub frame_type: String,
    /// Frame size
    pub frame_size: usize,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Sequence number
    pub sequence_number: u64,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl Default for FrameMetadata {
    fn default() -> Self {
        Self {
            frame_id: Uuid::new_v4().to_string(),
            frame_type: "data".to_string(),
            frame_size: 0,
            timestamp: Utc::now(),
            sequence_number: 0,
            metadata: HashMap::new(),
        }
    }
}
