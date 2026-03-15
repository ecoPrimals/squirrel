// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! MCP (Machine Context Protocol) plugin integration
//!
//! This module provides integration between plugins and the MCP system.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// MCP message type for plugin communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpMessage {
    /// Unique message identifier.
    pub id: String,
    /// MCP method name (e.g. "tools/list", "resources/read").
    pub method: String,
    /// JSON-RPC style parameters.
    pub params: serde_json::Value,
    /// Additional metadata key-value pairs.
    pub metadata: HashMap<String, String>,
}
