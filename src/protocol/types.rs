//! Protocol types for squirrel MCP implementation

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Session identifier type
pub type SessionId = String;

/// MCP message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    Request,
    Response,
    Notification,
}

/// MCP request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub id: String,
    pub method: String,
    pub params: Option<serde_json::Value>,
}

/// MCP response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub id: String,
    pub result: Option<serde_json::Value>,
    pub error: Option<ResponseError>,
}

/// Response error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseError {
    pub code: i32,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

/// Tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

/// Resource definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    pub uri: String,
    pub name: String,
    pub description: Option<String>,
    pub mime_type: Option<String>,
}

/// Protocol capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capabilities {
    pub experimental: HashMap<String, serde_json::Value>,
    pub sampling: Option<SamplingCapabilities>,
    pub logging: Option<LoggingCapabilities>,
}

/// Sampling capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingCapabilities {
    pub supported: bool,
}

/// Logging capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingCapabilities {
    pub supported: bool,
}

/// Protocol metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolMetadata {
    pub version: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Default for Capabilities {
    fn default() -> Self {
        Self {
            experimental: HashMap::new(),
            sampling: Some(SamplingCapabilities { supported: true }),
            logging: Some(LoggingCapabilities { supported: true }),
        }
    }
}

impl Default for ProtocolMetadata {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            version: "2.0".to_string(),
            created_at: now,
            updated_at: now,
        }
    }
}
