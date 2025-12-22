//! Tool execution types
//!
//! This module contains types related to tool execution.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;

/// Tool execution status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionStatus {
    /// Execution was successful
    Success,
    /// Execution failed
    Failure,
    /// Execution was cancelled
    Cancelled,
    /// Execution timed out
    Timeout,
}

/// Tool execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecutionResult {
    /// Tool ID
    pub tool_id: String,
    /// Capability name
    pub capability: String,
    /// Request ID
    pub request_id: String,
    /// Execution status
    pub status: ExecutionStatus,
    /// Execution output
    pub output: Option<JsonValue>,
    /// Error message if execution failed
    pub error_message: Option<String>,
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
    /// Timestamp when the execution completed
    pub timestamp: DateTime<Utc>,
}

/// Tool execution context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolContext {
    /// Tool ID
    pub tool_id: String,
    /// Capability name
    pub capability: String,
    /// Capability parameters
    pub parameters: HashMap<String, JsonValue>,
    /// Security token
    pub security_token: Option<String>,
    /// Session ID
    pub session_id: Option<String>,
    /// Request ID
    pub request_id: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

