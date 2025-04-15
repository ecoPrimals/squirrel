use serde::{Deserialize, Serialize};
use crate::mcp::context::McpContext;

/// MCP Message structure for command communications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpMessage {
    #[serde(rename = "type")]
    pub type_: String,
    pub payload: serde_json::Value,
    pub context: McpContext,
}

/// Connection status for MCP client
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionStatus {
    /// Connected to MCP server
    Connected,
    
    /// Disconnected from MCP server
    Disconnected,
    
    /// Connecting to MCP server
    Connecting,
    
    /// Connection error
    Error,
}

/// MCP client configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpClientConfig {
    /// MCP server host
    pub host: String,
    
    /// MCP server port
    pub port: u16,
    
    /// Connection timeout in milliseconds
    pub timeout_ms: u64,
    
    /// Number of retry attempts
    pub retry_attempts: u32,
    
    /// Delay between retries in milliseconds
    pub retry_delay_ms: u64,
}

impl Default for McpClientConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 8080,
            timeout_ms: 5000,
            retry_attempts: 3,
            retry_delay_ms: 1000,
        }
    }
}

/// WebSocket server message for event broadcasting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketServerMessage {
    pub event: String,
    pub data: serde_json::Value,
    pub time: String,
} 