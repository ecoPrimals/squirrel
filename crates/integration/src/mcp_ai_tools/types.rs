//! MCP-AI Tools integration types
//!
//! Common type definitions for the MCP-AI Tools integration.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Type of AI message
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AiMessageType {
    /// Human message
    Human,
    /// Assistant message
    Assistant,
    /// System message
    System,
    /// Tool result message
    ToolResult,
    /// Function call message
    FunctionCall,
}

impl fmt::Display for AiMessageType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AiMessageType::Human => write!(f, "human"),
            AiMessageType::Assistant => write!(f, "assistant"),
            AiMessageType::System => write!(f, "system"),
            AiMessageType::ToolResult => write!(f, "tool_result"),
            AiMessageType::FunctionCall => write!(f, "function_call"),
        }
    }
}

/// AI tool invocation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiToolInvocation {
    /// Name of the tool
    pub name: String,
    
    /// Arguments for the tool
    pub arguments: serde_json::Value,
    
    /// Unique ID for the invocation
    pub id: String,
    
    /// Timestamp when the invocation was created
    pub timestamp: chrono::DateTime<chrono::Utc>,
    
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl AiToolInvocation {
    /// Create a new tool invocation
    pub fn new(name: impl Into<String>, arguments: serde_json::Value) -> Self {
        let id = uuid::Uuid::new_v4().to_string();
        Self {
            name: name.into(),
            arguments,
            id,
            timestamp: chrono::Utc::now(),
            metadata: HashMap::new(),
        }
    }
    
    /// Add metadata to the invocation
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Serialize) -> Self {
        if let Ok(value) = serde_json::to_value(value) {
            self.metadata.insert(key.into(), value);
        }
        self
    }
}

/// AI tool response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiToolResponse {
    /// Result of the tool invocation
    pub result: serde_json::Value,
    
    /// Invocation this response is for
    pub invocation_id: String,
    
    /// Status of the response
    pub status: AiToolResponseStatus,
    
    /// Error message if present
    pub error: Option<String>,
    
    /// Timestamp when the response was created
    pub timestamp: chrono::DateTime<chrono::Utc>,
    
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl AiToolResponse {
    /// Create a new successful tool response
    pub fn success(invocation_id: impl Into<String>, result: impl Serialize) -> Self {
        let result = serde_json::to_value(result).unwrap_or(serde_json::Value::Null);
        Self {
            result,
            invocation_id: invocation_id.into(),
            status: AiToolResponseStatus::Success,
            error: None,
            timestamp: chrono::Utc::now(),
            metadata: HashMap::new(),
        }
    }
    
    /// Create a new error tool response
    pub fn error(invocation_id: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            result: serde_json::Value::Null,
            invocation_id: invocation_id.into(),
            status: AiToolResponseStatus::Error,
            error: Some(error.into()),
            timestamp: chrono::Utc::now(),
            metadata: HashMap::new(),
        }
    }
    
    /// Add metadata to the response
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Serialize) -> Self {
        if let Ok(value) = serde_json::to_value(value) {
            self.metadata.insert(key.into(), value);
        }
        self
    }
}

/// Status of an AI tool response
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AiToolResponseStatus {
    /// Tool executed successfully
    Success,
    /// Tool execution failed
    Error,
    /// Tool execution is in progress
    InProgress,
    /// Tool execution is partially complete
    Partial,
}

impl fmt::Display for AiToolResponseStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AiToolResponseStatus::Success => write!(f, "success"),
            AiToolResponseStatus::Error => write!(f, "error"),
            AiToolResponseStatus::InProgress => write!(f, "in_progress"),
            AiToolResponseStatus::Partial => write!(f, "partial"),
        }
    }
} 