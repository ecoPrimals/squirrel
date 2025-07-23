//! MCP (Machine Context Protocol) plugin integration
//!
//! This module provides integration between plugins and the MCP system.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// MCP message type for plugin communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpMessage {
    pub id: String,
    pub method: String,
    pub params: serde_json::Value,
    pub metadata: HashMap<String, String>,
}
