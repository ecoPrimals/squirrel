// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Types and serde glue for the MCP client message layer.

use serde::{Deserialize, Serialize};

/// Message categories for intelligent routing
#[derive(Debug, Clone, PartialEq)]
pub enum MessageCategory {
    /// Tool invocation messages for executing tools
    ToolInvocation,
    /// Resource access messages for retrieving resources
    ResourceAccess,
    /// Notification messages for event broadcasting
    Notification,
    /// Completion messages for auto-completion requests
    Completion,
    /// State management messages for managing application state
    StateManagement,
    /// System health messages for monitoring system status
    SystemHealth,
    /// Generic messages that don't fit other categories
    Generic,
}

/// Processing strategies for different message types
#[derive(Debug, Clone, PartialEq)]
pub enum ProcessingStrategy {
    /// Process messages synchronously with immediate response
    Synchronous,
    /// Process messages asynchronously in the background
    Asynchronous,
    /// Process messages with caching for improved performance
    Cached,
    /// Process messages with streaming support for large payloads
    Streaming,
    /// Process messages with transactional guarantees
    Transactional,
    /// Process messages with high priority
    Priority,
    /// Process messages with standard priority and handling
    Standard,
}

/// Processed payload with validation results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedPayload {
    /// The actual data payload content
    pub data: serde_json::Value,
    /// Status of payload validation (e.g., "valid", "invalid", "pending")
    pub validation_status: String,
    /// Hints for processing the payload effectively
    pub processing_hints: Vec<String>,
}

/// Comprehensive AI-enhanced MCP message structure for intelligent coordination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiMcpMessage {
    /// Unique identifier for the message
    pub id: String,
    /// Type of the message (e.g., "request", "response", "notification")
    pub message_type: String,
    /// Category for intelligent message routing
    pub category: MessageCategory,
    /// Processed message payload with validation status
    pub payload: ProcessedPayload,
    /// Unix timestamp when the message was created
    pub timestamp: i64,
    /// Client context information for the message
    pub client_context: ClientContext,
    /// Strategy for processing this message
    pub processing_strategy: ProcessingStrategy,
}

/// Message response structure for MCP protocol communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageResponse {
    /// Whether the operation was successful
    pub success: bool,
    /// Response data payload
    pub data: serde_json::Value,
    /// Type of the response message
    pub message_type: String,
    /// Unix timestamp when the response was created
    pub timestamp: i64,
}

/// Client context for message metadata and session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientContext {
    /// Unique identifier for the client instance
    pub client_id: String,
    /// Session identifier for the current connection
    pub session_id: String,
    /// List of capabilities supported by the client
    pub capabilities: Vec<String>,
}

impl Serialize for MessageCategory {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            MessageCategory::ToolInvocation => serializer.serialize_str("tool_invocation"),
            MessageCategory::ResourceAccess => serializer.serialize_str("resource_access"),
            MessageCategory::Notification => serializer.serialize_str("notification"),
            MessageCategory::Completion => serializer.serialize_str("completion"),
            MessageCategory::StateManagement => serializer.serialize_str("state_management"),
            MessageCategory::SystemHealth => serializer.serialize_str("system_health"),
            MessageCategory::Generic => serializer.serialize_str("generic"),
        }
    }
}

impl<'de> Deserialize<'de> for MessageCategory {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "tool_invocation" => Ok(MessageCategory::ToolInvocation),
            "resource_access" => Ok(MessageCategory::ResourceAccess),
            "notification" => Ok(MessageCategory::Notification),
            "completion" => Ok(MessageCategory::Completion),
            "state_management" => Ok(MessageCategory::StateManagement),
            "system_health" => Ok(MessageCategory::SystemHealth),
            "generic" => Ok(MessageCategory::Generic),
            _ => Ok(MessageCategory::Generic),
        }
    }
}

impl Serialize for ProcessingStrategy {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            ProcessingStrategy::Synchronous => serializer.serialize_str("synchronous"),
            ProcessingStrategy::Asynchronous => serializer.serialize_str("asynchronous"),
            ProcessingStrategy::Cached => serializer.serialize_str("cached"),
            ProcessingStrategy::Streaming => serializer.serialize_str("streaming"),
            ProcessingStrategy::Transactional => serializer.serialize_str("transactional"),
            ProcessingStrategy::Priority => serializer.serialize_str("priority"),
            ProcessingStrategy::Standard => serializer.serialize_str("standard"),
        }
    }
}

impl<'de> Deserialize<'de> for ProcessingStrategy {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "synchronous" => Ok(ProcessingStrategy::Synchronous),
            "asynchronous" => Ok(ProcessingStrategy::Asynchronous),
            "cached" => Ok(ProcessingStrategy::Cached),
            "streaming" => Ok(ProcessingStrategy::Streaming),
            "transactional" => Ok(ProcessingStrategy::Transactional),
            "priority" => Ok(ProcessingStrategy::Priority),
            "standard" => Ok(ProcessingStrategy::Standard),
            _ => Ok(ProcessingStrategy::Standard),
        }
    }
}

#[cfg(feature = "config")]
use crate::config::McpClientConfig;
#[cfg(feature = "config")]
use squirrel_mcp_config::unified::SquirrelUnifiedConfig; // Migrated from deprecated Config type (ADR-008)

#[cfg(feature = "config")]
impl From<&SquirrelUnifiedConfig> for McpClientConfig {
    fn from(_config: &SquirrelUnifiedConfig) -> Self {
        let mut mcp_config = McpClientConfig::from_env();
        mcp_config.server_url = format!(
            "unix://{}",
            universal_constants::network::resolve_capability_unix_socket(
                "MCP_SERVER_SOCKET",
                "squirrel-mcp",
            )
            .display()
        );
        mcp_config
    }
}
