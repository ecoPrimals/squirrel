//! MCP plugin module
//!
//! This module provides functionality for MCP plugins.

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt::Debug;
use uuid::Uuid;

use crate::plugin::Plugin;

/// MCP plugin trait
#[async_trait]
pub trait McpPlugin: Plugin {
    /// Handle MCP message
    async fn handle_message(&self, message: Value) -> Result<Value>;

    /// Get supported protocol extensions
    fn get_protocol_extensions(&self) -> Vec<String>;

    /// Check if the plugin supports a specific protocol extension
    fn supports_protocol_extension(&self, extension: &str) -> bool {
        self.get_protocol_extensions()
            .contains(&extension.to_string())
    }

    /// Handle protocol extension
    async fn handle_protocol_extension(&self, extension: &str, data: Value) -> Result<Value>;

    /// Validate message schema
    fn validate_message_schema(&self, message: &Value) -> bool;

    /// Get plugin capabilities
    fn get_capabilities(&self) -> Vec<String> {
        self.metadata().capabilities.clone()
    }
}

/// MCP message type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum McpMessageType {
    /// Request message
    Request,
    /// Response message
    Response,
    /// Notification message
    Notification,
    /// Error message
    Error,
}

/// MCP message context
#[derive(Debug, Clone)]
pub struct McpMessageContext {
    /// Message ID
    pub id: String,

    /// Message type
    pub message_type: McpMessageType,

    /// Protocol version
    pub protocol_version: String,

    /// Source ID
    pub source_id: Option<Uuid>,

    /// Target ID
    pub target_id: Option<Uuid>,

    /// Additional context properties
    pub properties: std::collections::HashMap<String, String>,
}

impl McpMessageContext {
    /// Create a new MCP message context
    pub fn new(id: impl Into<String>, message_type: McpMessageType) -> Self {
        Self {
            id: id.into(),
            message_type,
            protocol_version: "1.0".to_string(),
            source_id: None,
            target_id: None,
            properties: std::collections::HashMap::new(),
        }
    }

    /// Add a property to the context
    pub fn with_property(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.properties.insert(key.into(), value.into());
        self
    }

    /// Set the source ID
    pub fn with_source(mut self, source_id: Uuid) -> Self {
        self.source_id = Some(source_id);
        self
    }

    /// Set the target ID
    pub fn with_target(mut self, target_id: Uuid) -> Self {
        self.target_id = Some(target_id);
        self
    }

    /// Set the protocol version
    pub fn with_protocol_version(mut self, version: impl Into<String>) -> Self {
        self.protocol_version = version.into();
        self
    }
}
