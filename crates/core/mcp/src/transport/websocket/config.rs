// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! WebSocket transport configuration.

use crate::types::{CompressionFormat, EncryptionFormat};

/// Configuration for the WebSocket transport
///
/// This struct contains all the configuration parameters for
/// establishing and maintaining a WebSocket connection.
#[derive(Debug, Clone)]
pub struct WebSocketConfig {
    /// WebSocket URL to connect to
    pub url: String,

    /// Maximum message size in bytes
    pub max_message_size: usize,

    /// Connection timeout in seconds
    pub connection_timeout: u64,

    /// Ping interval in seconds
    pub ping_interval: Option<u64>,

    /// Encryption format
    pub encryption: EncryptionFormat,

    /// Compression format
    pub compression: CompressionFormat,

    /// Maximum number of reconnection attempts
    pub max_reconnect_attempts: u32,

    /// Reconnection delay in milliseconds
    pub reconnect_delay_ms: u64,
}

impl Default for WebSocketConfig {
    fn default() -> Self {
        // Using universal-constants for all configuration values
        use universal_constants::limits::DEFAULT_MAX_MESSAGE_SIZE;
        use universal_constants::network::get_service_port;
        use universal_constants::timeouts::{
            DEFAULT_CONNECTION_TIMEOUT, DEFAULT_INITIAL_DELAY, DEFAULT_PING_INTERVAL,
        };

        Self {
            url: format!("ws://localhost:{}", get_service_port("websocket")),
            max_message_size: DEFAULT_MAX_MESSAGE_SIZE,
            connection_timeout: DEFAULT_CONNECTION_TIMEOUT.as_secs(),
            ping_interval: Some(DEFAULT_PING_INTERVAL.as_secs()),
            encryption: EncryptionFormat::None,
            compression: CompressionFormat::None,
            max_reconnect_attempts: 5,
            reconnect_delay_ms: DEFAULT_INITIAL_DELAY.as_millis() as u64,
        }
    }
}
