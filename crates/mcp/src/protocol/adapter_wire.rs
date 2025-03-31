//! Wire Format Protocol Adapter module for MCP.
//!
//! This module provides a protocol adapter specifically for handling wire format translation
//! between different versions of the protocol. It handles serialization, deserialization,
//! and translation between wire format messages and domain objects.
//!
//! # Key Features
//!
//! - Protocol version negotiation
//! - Wire format serialization/deserialization
//! - Domain object translation
//! - Backward compatibility with older protocol versions
//! - Forward compatibility with newer protocol versions when possible
//!
//! # Examples
//!
//! ```
//! use mcp::protocol::adapter_wire::{WireFormatAdapter, WireFormatConfig};
//! use mcp::protocol::wire::{WireMessage, DomainObject};
//!
//! async fn example() {
//!     // Create a wire format adapter with default config
//!     let adapter = WireFormatAdapter::new(WireFormatConfig::default());
//!
//!     // Convert a domain object to wire format
//!     let domain_obj = DomainObject::new();
//!     let wire_message = adapter.to_wire_format(domain_obj).await.unwrap();
//!
//!     // Convert wire format back to domain object
//!     let deserialized = adapter.from_wire_format(&wire_message).await.unwrap();
//! }
//! ```

use crate::error::{MCPError, ProtocolError, Result};
use crate::message::{Message, MessageType};
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use thiserror::Error;
use uuid::Uuid;
use base64;
use base64::engine::general_purpose;
use base64::Engine;
use serde::{Serialize, Deserialize};

/// Errors specific to wire format protocol adapter operations.
#[derive(Debug, Error)]
pub enum WireFormatError {
    /// Unsupported protocol version
    #[error("Unsupported protocol version: {0}")]
    UnsupportedVersion(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Deserialization error
    #[error("Deserialization error: {0}")]
    Deserialization(String),

    /// Domain object translation error
    #[error("Domain object translation error: {0}")]
    Translation(String),

    /// Missing required field
    #[error("Missing required field: {0}")]
    MissingField(String),

    /// Invalid field value
    #[error("Invalid field value for {0}: {1}")]
    InvalidFieldValue(String, String),
    
    /// Invalid field type
    #[error("Invalid field type for {0}, expected {1}")]
    InvalidFieldType(String, String),
}

/// Wire format protocol versions supported by the adapter
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum WireProtocolVersion {
    #[serde(rename = "0.9")]
    V0_9,
    /// Version 1.0 (current stable)
    #[serde(rename = "1.0")]
    V1_0,
    /// Latest version (alias for the latest stable version)
    #[default]
    Latest,
}

impl WireProtocolVersion {
    /// Get the string representation of the protocol version
    #[must_use] pub const fn as_str(&self) -> &'static str {
        match self {
            Self::V1_0 => "1.0",
            Self::V0_9 => "0.9",
            Self::Latest => "1.0", // Latest is currently 1.0
        }
    }

    /// Parse a protocol version from a string
    pub fn from_str(version: &str) -> crate::error::Result<Self> {
        match version {
            "1.0" => Ok(Self::V1_0),
            "0.9" => Ok(Self::V0_9),
            "latest" => Ok(Self::Latest),
            _ => Err(MCPError::from(WireFormatError::UnsupportedVersion(version.to_string()))),
        }
    }
}

/// Format of the wire message
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WireFormat {
    /// JSON format
    Json,
    /// Binary format
    Binary,
    /// CBOR format
    Cbor,
}

/// Configuration for the wire format adapter
#[derive(Debug, Clone)]
pub struct WireFormatConfig {
    /// Default protocol version to use
    pub default_version: WireProtocolVersion,
    /// Supported protocol versions
    pub supported_versions: Vec<WireProtocolVersion>,
    /// Wire format to use
    pub format: WireFormat,
    /// Maximum message size in bytes
    pub max_message_size: usize,
    /// Enable schema validation
    pub schema_validation: bool,
}

impl Default for WireFormatConfig {
    fn default() -> Self {
        Self {
            default_version: WireProtocolVersion::V1_0,
            supported_versions: vec![WireProtocolVersion::V1_0, WireProtocolVersion::V0_9],
            format: WireFormat::Json,
            max_message_size: 10 * 1024 * 1024, // 10MB
            schema_validation: true,
        }
    }
}

/// A wire format message that can be serialized for transport
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WireMessage {
    /// Protocol version
    pub version: String,
    /// Message data
    pub data: Vec<u8>,
    /// Message format
    pub format: WireFormat,
    /// Message metadata
    pub metadata: HashMap<String, Value>,
}

impl WireMessage {
    /// Create a new wire message
    #[must_use] pub fn new(version: WireProtocolVersion, data: Vec<u8>, format: WireFormat) -> Self {
        Self {
            version: version.as_str().to_string(),
            data,
            format,
            metadata: HashMap::new(),
        }
    }

    /// Add metadata to the wire message
    #[must_use] pub fn with_metadata(mut self, key: &str, value: Value) -> Self {
        self.metadata.insert(key.to_string(), value);
        self
    }

    /// Create a wire message from a JSON value
    pub fn from_json(version: WireProtocolVersion, value: Value) -> crate::error::Result<Self> {
        let data = serde_json::to_vec(&value)
            .map_err(|e| MCPError::from(WireFormatError::Serialization(e.to_string())))?;

        Ok(Self::new(version, data, WireFormat::Json))
    }

    /// Create a wire message from a string
    #[must_use] pub fn from_string(version: WireProtocolVersion, content: &str) -> Self {
        Self::new(
            version,
            content.as_bytes().to_vec(),
            WireFormat::Json,
        )
    }
}

/// Domain object trait that can be converted to/from wire format
#[async_trait]
pub trait DomainObject: Send + Sync {
    /// Convert the domain object to a wire message
    async fn to_wire_message(&self, version: WireProtocolVersion) -> crate::error::Result<WireMessage>;

    /// Create a domain object from a wire message
    async fn from_wire_message(message: &WireMessage) -> crate::error::Result<Self>
    where
        Self: Sized;
}

/// Adapter for translating between wire format and domain objects
#[derive(Clone)]
pub struct WireFormatAdapter {
    /// Configuration
    pub config: WireFormatConfig,
    /// Protocol version mappings for translation
    version_mappings: Arc<RwLock<HashMap<String, HashMap<String, Arc<dyn Fn(Value) -> Result<Value> + Send + Sync>>>>>,
}

impl std::fmt::Debug for WireFormatAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WireFormatAdapter")
            .field("config", &self.config)
            .field("version_mappings", &format!("<mappings>"))
            .finish()
    }
}

impl WireFormatAdapter {
    /// Create a new wire format adapter with the given configuration
    #[must_use] pub fn new(config: WireFormatConfig) -> Self {
        Self {
            config,
            version_mappings: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Get the current configuration
    #[must_use] pub const fn get_config(&self) -> &WireFormatConfig {
        &self.config
    }

    /// Register a version mapping function that translates between protocol versions
    pub async fn register_version_mapping<F>(&self, from_version: WireProtocolVersion, to_version: WireProtocolVersion, mapper: F)
    where
        F: Fn(Value) -> crate::error::Result<Value> + Send + Sync + 'static,
    {
        let mut mappings = self.version_mappings.write().await;
        
        let from_key = from_version.as_str().to_string();
        let to_key = to_version.as_str().to_string();
        
        if !mappings.contains_key(&from_key) {
            mappings.insert(from_key.clone(), HashMap::new());
        }
        
        if let Some(version_map) = mappings.get_mut(&from_key) {
            version_map.insert(to_key, Arc::new(mapper));
        }
    }

    /// Convert an MCP message to wire format
    pub async fn to_wire_format(&self, message: &Message) -> crate::error::Result<WireMessage> {
        match self.config.format {
            WireFormat::Json => {
                // Convert the message to JSON - using content instead of payload
                let mut json_obj = serde_json::Map::new();
                
                // Add basic fields
                json_obj.insert("id".to_string(), serde_json::Value::String(message.id.clone()));
                json_obj.insert("message_type".to_string(), serde_json::Value::String(message.message_type.as_str().to_string()));
                json_obj.insert("content".to_string(), serde_json::Value::String(message.content.clone()));
                json_obj.insert("source".to_string(), serde_json::Value::String(message.source.clone()));
                json_obj.insert("destination".to_string(), serde_json::Value::String(message.destination.clone()));
                
                // Add optional fields
                if let Some(topic) = &message.topic {
                    let topic_str: String = topic.to_string();
                    json_obj.insert("topic".to_string(), serde_json::Value::String(topic_str));
                }
                
                if let Some(context_id) = &message.context_id {
                    let context_id_str: String = context_id.to_string();
                    json_obj.insert("context_id".to_string(), serde_json::Value::String(context_id_str));
                }
                
                if let Some(in_reply_to) = &message.in_reply_to {
                    let in_reply_to_str: String = in_reply_to.to_string();
                    json_obj.insert("in_reply_to".to_string(), serde_json::Value::String(in_reply_to_str));
                }
                
                if !message.metadata.is_empty() {
                    let metadata = serde_json::to_value(&message.metadata)
                        .map_err(|e| MCPError::from(WireFormatError::Serialization(e.to_string())))?;
                    json_obj.insert("metadata".to_string(), metadata);
                }
                
                // Add binary payload if it exists
                if let Some(ref binary_payload) = message.binary_payload {
                    // Encode binary data to base64 if the field value is binary
                    let encoded = general_purpose::STANDARD.encode(binary_payload);
                    json_obj.insert("binary_payload".to_string(), serde_json::Value::String(encoded));
                }
                
                let json = serde_json::Value::Object(json_obj);
                
                // Apply version-specific transformations if needed
                let version = self.config.default_version;
                let transformed = self.apply_version_transform(json, WireProtocolVersion::Latest, version).await?;
                
                // Serialize to bytes
                let data = serde_json::to_vec(&transformed)
                    .map_err(|e| MCPError::from(WireFormatError::Serialization(e.to_string())))?;
                
                Ok(WireMessage::new(version, data, WireFormat::Json))
            },
            WireFormat::Binary => {
                // For binary format, create a structured representation
                let mut json_obj = serde_json::Map::new();
                
                // Add basic fields
                json_obj.insert("id".to_string(), serde_json::Value::String(message.id.clone()));
                json_obj.insert("message_type".to_string(), serde_json::Value::String(message.message_type.as_str().to_string()));
                json_obj.insert("content".to_string(), serde_json::Value::String(message.content.clone()));
                json_obj.insert("source".to_string(), serde_json::Value::String(message.source.clone()));
                json_obj.insert("destination".to_string(), serde_json::Value::String(message.destination.clone()));
                
                let json = serde_json::Value::Object(json_obj);
                let data = serde_json::to_vec(&json)
                    .map_err(|e| MCPError::from(WireFormatError::Serialization(e.to_string())))?;
                
                Ok(WireMessage::new(self.config.default_version, data, WireFormat::Binary))
            },
            WireFormat::Cbor => {
                // CBOR implementation would go here - simplified for now
                let mut json_obj = serde_json::Map::new();
                
                // Add basic fields
                json_obj.insert("id".to_string(), serde_json::Value::String(message.id.clone()));
                json_obj.insert("message_type".to_string(), serde_json::Value::String(message.message_type.as_str().to_string()));
                json_obj.insert("content".to_string(), serde_json::Value::String(message.content.clone()));
                
                let json = serde_json::Value::Object(json_obj);
                let data = serde_json::to_vec(&json)
                    .map_err(|e| MCPError::from(WireFormatError::Serialization(e.to_string())))?;
                
                Ok(WireMessage::new(self.config.default_version, data, WireFormat::Cbor))
            },
        }
    }

    /// Convert from wire format to an MCP message
    pub async fn from_wire_format(&self, wire_message: &WireMessage) -> crate::error::Result<Message> {
        // Parse the message version
        let source_version = WireProtocolVersion::from_str(&wire_message.version)?;
        
        match wire_message.format {
            WireFormat::Json => {
                // Parse the JSON data
                let json: Value = serde_json::from_slice(&wire_message.data)
                    .map_err(|e| MCPError::from(WireFormatError::Deserialization(e.to_string())))?;
                
                // Apply version-specific transformations to convert to latest format
                let transformed = self.apply_version_transform(json, source_version, WireProtocolVersion::Latest).await?;
                
                // Extract fields manually to avoid payload field mismatch
                if let Some(obj) = transformed.as_object() {
                    // Required fields
                    let id = Self::extract_string_or_default(obj, "id", || Uuid::new_v4().to_string());
                    let message_type_str = Self::extract_string_or_default(obj, "message_type", || "notification".to_string());
                    let content = Self::extract_string_or_default(obj, "content", || String::new());
                    let source = Self::extract_string_or_default(obj, "source", || "unknown".to_string());
                    let destination = Self::extract_string_or_default(obj, "destination", || "*".to_string());
                    
                    // Parse message type
                    let message_type = match message_type_str.as_str() {
                        "request" => MessageType::Request,
                        "response" => MessageType::Response,
                        "notification" => MessageType::Notification,
                        "error" => MessageType::Error,
                        "control" => MessageType::Control,
                        "system" => MessageType::System,
                        "stream_chunk" => MessageType::StreamChunk,
                        _ => MessageType::Notification,
                    };
                    
                    // Optional fields
                    let in_reply_to = obj.get("in_reply_to").and_then(|v| v.as_str()).map(std::string::ToString::to_string);
                    let context_id = obj.get("context_id").and_then(|v| v.as_str()).map(std::string::ToString::to_string);
                    let topic = obj.get("topic").and_then(|v| v.as_str()).map(std::string::ToString::to_string);
                    
                    // Decode base64 to binary
                    let binary_payload = obj.get("binary_payload").and_then(|v| v.as_str()).map(|s| {
                        general_purpose::STANDARD.decode(s).unwrap_or_default()
                    });
                    
                    // Metadata
                    let metadata = if let Some(meta) = obj.get("metadata") {
                        if let Some(meta_obj) = meta.as_object() {
                            meta_obj
                                .iter()
                                .filter_map(|(k, v)| {
                                    v.as_str().map(|s| (k.clone(), s.to_string()))
                                })
                                .collect()
                        } else {
                            HashMap::new()
                        }
                    } else {
                        HashMap::new()
                    };
                    
                    // Create the message
                    let mut message = Message::new(message_type, content, source, destination);
                    message.id = id;
                    message.in_reply_to = in_reply_to;
                    message.context_id = context_id;
                    message.topic = topic;
                    message.binary_payload = binary_payload;
                    message.metadata = metadata;
                    message.timestamp = chrono::Utc::now(); // Default to current time
                    
                    Ok(message)
                } else {
                    Err(MCPError::from(WireFormatError::InvalidFieldValue(
                        "root".to_string(),
                        "Expected JSON object".to_string(),
                    )))
                }
            },
            WireFormat::Binary | WireFormat::Cbor => {
                // Simplified implementation for now
                let json: Value = serde_json::from_slice(&wire_message.data)
                    .map_err(|e| MCPError::from(WireFormatError::Deserialization(e.to_string())))?;
                
                if let Some(obj) = json.as_object() {
                    let id = Self::extract_string_or_default(obj, "id", || Uuid::new_v4().to_string());
                    let message_type_str = Self::extract_string_or_default(obj, "message_type", || "notification".to_string());
                    let content = Self::extract_string_or_default(obj, "content", || String::new());
                    let source = Self::extract_string_or_default(obj, "source", || "unknown".to_string());
                    let destination = Self::extract_string_or_default(obj, "destination", || "*".to_string());
                    
                    // Parse message type
                    let message_type = match message_type_str.as_str() {
                        "request" => MessageType::Request,
                        "response" => MessageType::Response,
                        "notification" => MessageType::Notification,
                        "error" => MessageType::Error,
                        "control" => MessageType::Control,
                        "system" => MessageType::System,
                        "stream_chunk" => MessageType::StreamChunk,
                        _ => MessageType::Notification,
                    };
                    
                    let mut message = Message::new(message_type, content, source, destination);
                    message.id = id;
                    
                    Ok(message)
                } else {
                    Err(MCPError::from(WireFormatError::InvalidFieldValue(
                        "root".to_string(),
                        "Expected JSON object".to_string(),
                    )))
                }
            },
        }
    }

    /// Apply version-specific transformations to the message data
    async fn apply_version_transform(
        &self,
        data: Value,
        from_version: WireProtocolVersion,
        to_version: WireProtocolVersion,
    ) -> crate::error::Result<Value> {
        // If versions are the same, no transformation needed
        if from_version == to_version {
            return Ok(data);
        }
        
        // Check if we have a direct mapping
        let mappings = self.version_mappings.read().await;
        let from_key = from_version.as_str().to_string();
        let to_key = to_version.as_str().to_string();
        
        if let Some(version_map) = mappings.get(&from_key) {
            if let Some(mapper) = version_map.get(&to_key) {
                return mapper(data).map_err(|e| MCPError::from(e));
            }
        }
        
        // Otherwise, try to chain transformations
        // For simplicity, we'll just handle the direct case now
        // A more sophisticated implementation would find a path through the version graph
        
        // Default to passing through unchanged
        Ok(data)
    }

    /// Validate a wire message against schema (if schema validation is enabled)
    pub fn validate_schema(&self, message: &WireMessage) -> crate::error::Result<()> {
        if !self.config.schema_validation {
            return Ok(());
        }
        
        // Basic validation for now
        // In a full implementation, we would validate against a proper schema
        if message.format == WireFormat::Json {
            let json: Value = serde_json::from_slice(&message.data)
                .map_err(|e| MCPError::from(WireFormatError::Deserialization(e.to_string())))?;
            
            // Simple validation: check that it's an object with required fields
            if !json.is_object() {
                return Err(MCPError::from(WireFormatError::InvalidFieldValue("root".to_string(), "not an object".to_string())));
            }
            
            let obj = json.as_object().unwrap();
            
            // Validate required fields based on message type
            if let Some(message_type) = obj.get("message_type") {
                if let Some(message_type) = message_type.as_str() {
                    // Check fields based on message type
                    match message_type {
                        "command" => {
                            if !obj.contains_key("id") {
                                return Err(MCPError::from(WireFormatError::MissingField("id".to_string())));
                            }
                            if !obj.contains_key("payload") {
                                return Err(MCPError::from(WireFormatError::MissingField("payload".to_string())));
                            }
                        },
                        "event" => {
                            if !obj.contains_key("id") {
                                return Err(MCPError::from(WireFormatError::MissingField("id".to_string())));
                            }
                            if !obj.contains_key("payload") {
                                return Err(MCPError::from(WireFormatError::MissingField("payload".to_string())));
                            }
                        },
                        // Add validation for other message types
                        _ => {},
                    }
                }
            } else {
                return Err(MCPError::from(WireFormatError::MissingField("message_type".to_string())));
            }
        } else {
            // For other formats, we'd implement appropriate validation
            // For now, we'll skip validation for non-JSON formats
        }
        
        Ok(())
    }

    // Helper function to extract a string from a JSON object with a default
    fn extract_string_or_default<F>(obj: &serde_json::Map<String, Value>, key: &str, default: F) -> String 
    where 
        F: FnOnce() -> String
    {
        obj.get(key)
            .and_then(|v| v.as_str()).map_or_else(default, std::string::ToString::to_string)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::MessageBuilder;
    use serde_json::json;

    #[tokio::test]
    async fn test_wire_format_roundtrip() {
        // Create a wire format adapter
        let adapter = WireFormatAdapter::new(WireFormatConfig::default());
        
        // Create a test message
        let message = MessageBuilder::new()
            .with_message_type("request")
            .with_content("Test message content")
            .with_source("client-123")
            .with_destination("server-456")
            .with_metadata("version", "1.0")
            .with_metadata("app", "test-suite")
            .build();
        
        // Convert to wire format
        let wire_message = adapter.to_wire_format(&message).await.unwrap();
        
        // Convert back to message
        let deserialized = adapter.from_wire_format(&wire_message).await.unwrap();
        
        // Verify core fields
        assert_eq!(message.message_type, deserialized.message_type);
        assert_eq!(message.content, deserialized.content);
        assert_eq!(message.source, deserialized.source);
        assert_eq!(message.destination, deserialized.destination);
        
        // Verify metadata
        assert_eq!(message.metadata.get("version"), deserialized.metadata.get("version"));
        assert_eq!(message.metadata.get("app"), deserialized.metadata.get("app"));
    }

    #[tokio::test]
    async fn test_version_translation() {
        // Create a wire format adapter
        let adapter = WireFormatAdapter::new(WireFormatConfig::default());
        
        // Register a version mapping from 0.9 to 1.0
        adapter.register_version_mapping(
            WireProtocolVersion::V0_9,
            WireProtocolVersion::V1_0,
            |value| {
                let mut obj = value.as_object().unwrap().clone();
                
                // In v0.9, "msg_type" was used instead of "message_type"
                if let Some(msg_type) = obj.remove("msg_type") {
                    obj.insert("message_type".to_string(), msg_type);
                }
                
                Ok(Value::Object(obj))
            }
        ).await;
        
        // Create a v0.9 format message
        let v09_data = json!({
            "id": "test-123",
            "msg_type": "command",
            "payload": {
                "action": "get_status"
            }
        });
        
        let wire_message = WireMessage::from_json(WireProtocolVersion::V0_9, v09_data).unwrap();
        
        // Convert to v1.0 message
        let message = adapter.from_wire_format(&wire_message).await.unwrap();
        
        // Verify translation happened correctly
        assert_eq!(message.message_type, "command");
    }

    #[tokio::test]
    async fn test_schema_validation() {
        // Create a wire format adapter with schema validation
        let mut config = WireFormatConfig::default();
        config.schema_validation = true;
        let adapter = WireFormatAdapter::new(config);
        
        // Valid message
        let valid_data = json!({
            "id": "test-123",
            "message_type": "command",
            "payload": {
                "action": "get_status"
            }
        });
        
        let valid_wire = WireMessage::from_json(WireProtocolVersion::V1_0, valid_data).unwrap();
        
        // Should validate successfully
        assert!(adapter.validate_schema(&valid_wire).is_ok());
        
        // Invalid message (missing required field)
        let invalid_data = json!({
            "message_type": "command",
            // Missing id
            "payload": {
                "action": "get_status"
            }
        });
        
        let invalid_wire = WireMessage::from_json(WireProtocolVersion::V1_0, invalid_data).unwrap();
        
        // Should fail validation
        assert!(adapter.validate_schema(&invalid_wire).is_err());
    }
} 