// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Server Configuration
//!
//! Configuration types for the MCP server.

use serde_json::Value;
use std::collections::HashMap;

use crate::protocol::adapter_wire::WireFormatConfig;

/// MCP Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Address to bind the server to
    pub bind_address: String,
    
    /// Maximum number of concurrent clients
    pub max_clients: usize,
    
    /// Client connection timeout in milliseconds
    pub client_timeout_ms: u64,
    
    /// Keep-alive interval in milliseconds
    pub keep_alive_interval_ms: Option<u64>,
    
    /// Maximum message size in bytes
    pub max_message_size: usize,
    
    /// Server ID (generated automatically if not provided)
    pub server_id: Option<String>,
    
    /// Wire format adapter configuration
    pub wire_format_config: Option<WireFormatConfig>,
    
    /// Additional server parameters
    pub parameters: HashMap<String, Value>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        use universal_constants::{limits, network, timeouts};
        
        Self {
            bind_address: network::socket_addr(
                &network::get_bind_address(),
                network::get_service_port("http"),
            ),
            max_clients: limits::DEFAULT_MAX_CONNECTIONS,
            client_timeout_ms: timeouts::duration_to_millis(timeouts::DEFAULT_CONNECTION_TIMEOUT),
            keep_alive_interval_ms: Some(timeouts::duration_to_millis(
                timeouts::DEFAULT_HEARTBEAT_INTERVAL,
            )),
            max_message_size: limits::DEFAULT_MAX_MESSAGE_SIZE,
            server_id: None,
            wire_format_config: None,
            parameters: HashMap::new(),
        }
    }
}

/// MCP Server state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerState {
    /// Server is stopped
    Stopped,
    /// Server is starting
    Starting,
    /// Server is running
    Running,
    /// Server is stopping
    Stopping,
    /// Server encountered an error
    Error,
}
