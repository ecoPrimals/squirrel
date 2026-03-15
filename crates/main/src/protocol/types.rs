// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_type_serde() {
        for variant in [
            MessageType::Request,
            MessageType::Response,
            MessageType::Notification,
        ] {
            let json = serde_json::to_string(&variant).expect("serialize");
            let _: MessageType = serde_json::from_str(&json).expect("deserialize");
        }
    }

    #[test]
    fn test_request_serde() {
        let req = Request {
            id: "1".to_string(),
            method: "system.ping".to_string(),
            params: Some(serde_json::json!({"key": "value"})),
        };
        let json = serde_json::to_string(&req).expect("serialize");
        let deser: Request = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deser.id, "1");
        assert_eq!(deser.method, "system.ping");
        assert!(deser.params.is_some());
    }

    #[test]
    fn test_response_serde() {
        let resp = Response {
            id: "1".to_string(),
            result: Some(serde_json::json!("ok")),
            error: None,
        };
        let json = serde_json::to_string(&resp).expect("serialize");
        let deser: Response = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deser.id, "1");
        assert!(deser.result.is_some());
        assert!(deser.error.is_none());
    }

    #[test]
    fn test_response_error_serde() {
        let err = ResponseError {
            code: -32600,
            message: "Invalid Request".to_string(),
            data: None,
        };
        let json = serde_json::to_string(&err).expect("serialize");
        let deser: ResponseError = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deser.code, -32600);
        assert_eq!(deser.message, "Invalid Request");
    }

    #[test]
    fn test_tool_serde() {
        let tool = Tool {
            name: "test_tool".to_string(),
            description: "A test tool".to_string(),
            input_schema: serde_json::json!({"type": "object"}),
        };
        let json = serde_json::to_string(&tool).expect("serialize");
        let deser: Tool = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deser.name, "test_tool");
    }

    #[test]
    fn test_resource_serde() {
        let resource = Resource {
            uri: "file:///test".to_string(),
            name: "test".to_string(),
            description: Some("A test resource".to_string()),
            mime_type: Some("text/plain".to_string()),
        };
        let json = serde_json::to_string(&resource).expect("serialize");
        let deser: Resource = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deser.uri, "file:///test");
        assert_eq!(deser.mime_type, Some("text/plain".to_string()));
    }

    #[test]
    fn test_capabilities_default() {
        let caps = Capabilities::default();
        assert!(caps.experimental.is_empty());
        assert!(caps.sampling.as_ref().expect("sampling").supported);
        assert!(caps.logging.as_ref().expect("logging").supported);
    }

    #[test]
    fn test_capabilities_serde() {
        let caps = Capabilities::default();
        let json = serde_json::to_string(&caps).expect("serialize");
        let deser: Capabilities = serde_json::from_str(&json).expect("deserialize");
        assert!(deser.sampling.is_some());
        assert!(deser.logging.is_some());
    }

    #[test]
    fn test_protocol_metadata_default() {
        let meta = ProtocolMetadata::default();
        assert_eq!(meta.version, "2.0");
        assert!(meta.created_at <= meta.updated_at);
    }

    #[test]
    fn test_protocol_metadata_serde() {
        let meta = ProtocolMetadata::default();
        let json = serde_json::to_string(&meta).expect("serialize");
        let deser: ProtocolMetadata = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deser.version, "2.0");
    }
}
