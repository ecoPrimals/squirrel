//! Domain object implementations for the MCP protocol.
//!
//! This module provides implementations of the `DomainObject` trait for various
//! domain objects used in the MCP protocol. These implementations enable translation
//! between domain objects and wire format messages.

use crate::error::Result;
use crate::message::Message;
use crate::types::MCPMessage;
use crate::protocol::adapter_wire::{DomainObject, WireFormatError, WireMessage, ProtocolVersion, WireFormat};
use async_trait::async_trait;
use chrono::Utc;
use serde_json::{json, Value};
use std::collections::HashMap;
use uuid::Uuid;
use crate::error::MCPError;
use serde_json::Map;
use std::convert::TryFrom;
use crate::types::ResponseStatus;
use base64::Engine;

// ==========================================
// Message Domain Object Implementation
// ==========================================

#[async_trait]
impl DomainObject for Message {
    async fn to_wire_message(&self, version: ProtocolVersion) -> Result<WireMessage> {
        // Convert Message to Value based on protocol version
        let json_value = match version {
            ProtocolVersion::V1_0 | ProtocolVersion::Latest => {
                // Current version format - direct serialization
                serde_json::to_value(self)
                    .map_err(|e| MCPError::from(WireFormatError::Serialization(e.to_string())))?
            },
            ProtocolVersion::V0_9 => {
                // Legacy format (v0.9)
                // Here we adapt our current model to the old wire format
                let mut obj = serde_json::Map::new();
                
                // Map fields from current to legacy format
                obj.insert("id".to_string(), json!(self.id));
                obj.insert("msg_type".to_string(), json!(format!("{:?}", self.message_type).to_lowercase()));
                obj.insert("content".to_string(), json!(self.content));
                obj.insert("timestamp".to_string(), json!(self.timestamp.timestamp()));
                obj.insert("source".to_string(), json!(self.source));
                obj.insert("destination".to_string(), json!(self.destination));
                
                // Handle optional fields
                if let Some(reply_id) = &self.in_reply_to {
                    obj.insert("reply_to".to_string(), json!(reply_id));
                }
                
                if let Some(binary) = &self.binary_payload {
                    obj.insert("binary".to_string(), json!(base64::engine::general_purpose::STANDARD.encode(binary)));
                }
                
                // Add metadata
                let metadata = json!(self.metadata);
                obj.insert("metadata".to_string(), metadata);
                
                Value::Object(obj)
            }
        };
        
        // Create wire message with the JSON value
        WireMessage::from_json(version, json_value)
    }
    
    async fn from_wire_message(message: &WireMessage) -> Result<Self>
    where
        Self: Sized,
    {
        // Parse the version to determine how to decode
        let version = ProtocolVersion::from_str(&message.version)?;
        
        match message.format {
            WireFormat::Json => {
                // Parse the JSON data
                let json: Value = serde_json::from_slice(&message.data)
                    .map_err(|e| MCPError::from(WireFormatError::Deserialization(e.to_string())))?;
                
                match version {
                    ProtocolVersion::V1_0 | ProtocolVersion::Latest => {
                        // Current version format - direct deserialization
                        serde_json::from_value(json)
                            .map_err(|e| MCPError::from(WireFormatError::Deserialization(e.to_string())))
                    },
                    ProtocolVersion::V0_9 => {
                        // Legacy format (v0.9) - need to adapt to current model
                        if let Some(obj) = json.as_object() {
                            // Extract required fields with validation
                            let id = extract_string(obj, "id")?;
                            let msg_type_str = extract_string(obj, "msg_type")?;
                            let content = extract_string(obj, "content")?;
                            let source = extract_string(obj, "source")?;
                            let destination = extract_string(obj, "destination")?;
                            
                            // Convert message type string to enum
                            let message_type = match msg_type_str.as_str() {
                                "request" => crate::message::MessageType::Request,
                                "response" => crate::message::MessageType::Response,
                                "notification" => crate::message::MessageType::Notification,
                                "error" => crate::message::MessageType::Error,
                                "control" => crate::message::MessageType::Control,
                                "system" => crate::message::MessageType::System,
                                _ => return Err(MCPError::from(WireFormatError::InvalidFieldValue(
                                    "msg_type".to_string(), 
                                    format!("Unknown message type: {}", msg_type_str)
                                ))),
                            };
                            
                            // Extract optional fields
                            let in_reply_to = obj.get("reply_to")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string());
                            
                            // Extract binary payload if present
                            let binary_payload = obj.get("binary")
                                .and_then(|v| v.as_str())
                                .map(|s| base64::engine::general_purpose::STANDARD.decode(s))
                                .transpose()
                                .map_err(|e| MCPError::from(WireFormatError::Deserialization(
                                    format!("Invalid base64 in binary field: {}", e)
                                )))?;
                            
                            // Extract timestamp or use current time
                            let timestamp = obj.get("timestamp")
                                .and_then(|v| v.as_i64())
                                .map(|ts| chrono::DateTime::from_timestamp(ts, 0))
                                .flatten()
                                .unwrap_or_else(Utc::now);
                            
                            // Extract topic if present
                            let topic = obj.get("topic")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string());
                            
                            // Extract context ID if present
                            let context_id = obj.get("context_id")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string());
                            
                            // Extract metadata
                            let metadata = if let Some(meta_value) = obj.get("metadata") {
                                if let Some(meta_obj) = meta_value.as_object() {
                                    meta_obj.iter().filter_map(|(k, v)| {
                                        v.as_str().map(|s| (k.clone(), s.to_string()))
                                    }).collect()
                                } else {
                                    std::collections::HashMap::new()
                                }
                            } else {
                                std::collections::HashMap::new()
                            };
                            
                            // Create the Message
                            Ok(Message {
                                id,
                                message_type,
                                priority: crate::message::MessagePriority::Normal, // Default value
                                content,
                                binary_payload,
                                timestamp,
                                in_reply_to,
                                source,
                                destination,
                                context_id,
                                topic,
                                metadata,
                            })
                        } else {
                            Err(MCPError::from(WireFormatError::InvalidFieldValue(
                                "root".to_string(), 
                                "Expected JSON object".to_string()
                            )))
                        }
                    }
                }
            },
            WireFormat::Binary | WireFormat::Cbor => {
                // For now, treat binary and CBOR as JSON
                // In a full implementation, these would have separate decoders
                let json: Value = serde_json::from_slice(&message.data)
                    .map_err(|e| MCPError::from(WireFormatError::Deserialization(e.to_string())))?;
                
                serde_json::from_value(json)
                    .map_err(|e| MCPError::from(WireFormatError::Deserialization(e.to_string())))
            }
        }
    }
}

// ==========================================
// MCPMessage Domain Object Implementation
// ==========================================

#[async_trait]
impl DomainObject for MCPMessage {
    async fn to_wire_message(&self, version: ProtocolVersion) -> Result<WireMessage> {
        // Convert MCPMessage to Value based on protocol version
        let json_value = match version {
            ProtocolVersion::V1_0 | ProtocolVersion::Latest => {
                // Current version format
                serde_json::to_value(self)
                    .map_err(|e| MCPError::from(WireFormatError::Serialization(e.to_string())))?
            },
            ProtocolVersion::V0_9 => {
                // Legacy format (v0.9)
                let mut obj = serde_json::Map::new();
                
                // Map core fields
                obj.insert("id".to_string(), json!(self.id.0));
                obj.insert("type".to_string(), json!(format!("{:?}", self.type_).to_lowercase()));
                
                // Add payload
                obj.insert("payload".to_string(), self.payload.clone());
                
                // Add metadata if present
                if let Some(metadata) = &self.metadata {
                    obj.insert("metadata".to_string(), metadata.clone());
                }
                
                // Add security information
                obj.insert("security".to_string(), json!({
                    "security_level": format!("{:?}", self.security.security_level).to_lowercase(),
                    "has_signature": self.security.signature.is_some(),
                }));
                
                // Add timestamp
                obj.insert("timestamp".to_string(), json!(self.timestamp.timestamp()));
                
                // Add version
                obj.insert("version".to_string(), json!(format!("{}.{}", self.version.major, self.version.minor)));
                
                // Add trace ID if present
                if let Some(trace_id) = &self.trace_id {
                    obj.insert("trace_id".to_string(), json!(trace_id));
                }
                
                Value::Object(obj)
            }
        };
        
        // Create wire message with the JSON value
        WireMessage::from_json(version, json_value)
    }
    
    async fn from_wire_message(message: &WireMessage) -> Result<Self>
    where
        Self: Sized,
    {
        // Parse the version to determine how to decode
        let version = ProtocolVersion::from_str(&message.version)?;
        
        match message.format {
            WireFormat::Json => {
                // Parse the JSON data
                let json: Value = serde_json::from_slice(&message.data)
                    .map_err(|e| MCPError::from(WireFormatError::Deserialization(e.to_string())))?;
                
                match version {
                    ProtocolVersion::V1_0 | ProtocolVersion::Latest => {
                        // Current version format - direct deserialization
                        serde_json::from_value(json)
                            .map_err(|e| MCPError::from(WireFormatError::Deserialization(e.to_string())))
                    },
                    ProtocolVersion::V0_9 => {
                        // Legacy format (v0.9) - need to adapt to current model
                        if let Some(obj) = json.as_object() {
                            // Extract required fields
                            let id_str = extract_string(obj, "id")?;
                            let type_str = extract_string(obj, "type")?;
                            
                            // Parse message type
                            let msg_type = match type_str.as_str() {
                                "command" => crate::types::MessageType::Command,
                                "response" => crate::types::MessageType::Response,
                                "event" => crate::types::MessageType::Event,
                                "error" => crate::types::MessageType::Error,
                                "setup" => crate::types::MessageType::Setup,
                                "heartbeat" => crate::types::MessageType::Heartbeat,
                                "sync" => crate::types::MessageType::Sync,
                                _ => return Err(MCPError::from(WireFormatError::InvalidFieldValue(
                                    "type".to_string(), 
                                    format!("Unknown message type: {}", type_str)
                                ))),
                            };
                            
                            // Extract payload
                            let payload = obj.get("payload")
                                .cloned()
                                .unwrap_or(Value::Null);
                            
                            // Extract metadata if present
                            let metadata = obj.get("metadata").cloned();
                            
                            // Extract security information
                            let security = if let Some(sec) = obj.get("security") {
                                if let Some(sec_obj) = sec.as_object() {
                                    // Extract security level
                                    let security_level = sec_obj.get("security_level")
                                        .and_then(|v| v.as_str())
                                        .map(|s| match s {
                                            "low" => crate::types::SecurityLevel::Low,
                                            "medium" => crate::types::SecurityLevel::Standard,
                                            "high" => crate::types::SecurityLevel::High,
                                            "critical" => crate::types::SecurityLevel::Critical,
                                            _ => crate::types::SecurityLevel::default(),
                                        })
                                        .unwrap_or_default();
                                    
                                    // Extract encryption info
                                    let encryption_info = sec_obj.get("encryption_info")
                                        .and_then(|v| v.as_object())
                                        .map(|_| None) // Simplified for now
                                        .unwrap_or(None);
                                    
                                    // Extract signature
                                    let signature = sec_obj.get("signature")
                                        .and_then(|v| v.as_str())
                                        .map(|s| s.to_string());
                                    
                                    // Extract auth token
                                    let auth_token = sec_obj.get("auth_token")
                                        .and_then(|v| v.as_str())
                                        .map(|s| s.to_string());
                                    
                                    crate::types::SecurityMetadata {
                                        security_level,
                                        encryption_info,
                                        signature,
                                        auth_token,
                                        permissions: None,
                                        roles: None,
                                    }
                                } else {
                                    crate::types::SecurityMetadata::default()
                                }
                            } else {
                                crate::types::SecurityMetadata::default()
                            };
                            
                            // Extract timestamp or use current time
                            let timestamp = obj.get("timestamp")
                                .and_then(|v| v.as_i64())
                                .map(|ts| chrono::DateTime::from_timestamp(ts, 0))
                                .flatten()
                                .unwrap_or_else(Utc::now);
                            
                            // Extract version
                            let version_str = obj.get("version")
                                .and_then(|v| v.as_str())
                                .unwrap_or("0.9");
                                
                            // Extract trace ID if present
                            let trace_id = obj.get("trace_id")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string());
                            
                            // Create the MCPMessage
                            Ok(MCPMessage {
                                id: crate::types::MessageId(id_str),
                                type_: msg_type,
                                payload,
                                metadata,
                                security,
                                timestamp,
                                version: crate::types::ProtocolVersion { major: 1, minor: 0 },
                                trace_id,
                            })
                        } else {
                            Err(MCPError::from(WireFormatError::InvalidFieldValue(
                                "root".to_string(), 
                                "Expected JSON object".to_string()
                            )))
                        }
                    }
                }
            },
            WireFormat::Binary | WireFormat::Cbor => {
                // For now, treat binary and CBOR as JSON
                // In a full implementation, these would have separate decoders
                let json: Value = serde_json::from_slice(&message.data)
                    .map_err(|e| MCPError::from(WireFormatError::Deserialization(e.to_string())))?;
                
                serde_json::from_value(json)
                    .map_err(|e| MCPError::from(WireFormatError::Deserialization(e.to_string())))
            }
        }
    }
}

// ==========================================
// Helper Functions
// ==========================================

/// Helper function to extract a string field from a JSON object
fn extract_string(obj: &serde_json::Map<String, Value>, field: &str) -> Result<String> {
    Ok(obj.get(field)
        .ok_or_else(|| MCPError::from(WireFormatError::MissingField(field.to_string())))?
        .as_str()
        .ok_or_else(|| MCPError::from(WireFormatError::InvalidFieldValue(field.to_string(), "not a string".to_string())))?
        .to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::{MessageType, MessagePriority};
    
    #[tokio::test]
    async fn test_message_to_wire_format_v1() {
        // Create a test message
        let message = Message {
            id: Uuid::new_v4().to_string(),
            message_type: MessageType::Request,
            priority: MessagePriority::Normal,
            content: "Test content".to_string(),
            binary_payload: None,
            timestamp: Utc::now(),
            in_reply_to: None,
            source: "test-client".to_string(),
            destination: "test-server".to_string(),
            context_id: Some("test-context".to_string()),
            topic: Some("test-topic".to_string()),
            metadata: {
                let mut map = HashMap::new();
                map.insert("key1".to_string(), "value1".to_string());
                map
            },
        };
        
        // Convert to wire format
        let wire_message = message.to_wire_message(ProtocolVersion::V1_0).await.unwrap();
        
        // Verify wire message properties
        assert_eq!(wire_message.version, "1.0");
        assert_eq!(wire_message.format, WireFormat::Json);
        
        // Convert back to message
        let decoded = Message::from_wire_message(&wire_message).await.unwrap();
        
        // Verify round-trip conversion
        assert_eq!(decoded.id, message.id);
        assert_eq!(decoded.message_type, message.message_type);
        assert_eq!(decoded.content, message.content);
        assert_eq!(decoded.source, message.source);
        assert_eq!(decoded.destination, message.destination);
    }
    
    #[tokio::test]
    async fn test_message_to_wire_format_v09() {
        // Create a test message
        let message = Message {
            id: "test-123".to_string(),
            message_type: MessageType::Notification,
            priority: MessagePriority::High,
            content: "Legacy message".to_string(),
            binary_payload: Some(vec![1, 2, 3, 4]),
            timestamp: Utc::now(),
            in_reply_to: Some("ref-456".to_string()),
            source: "client-a".to_string(),
            destination: "server-b".to_string(),
            context_id: None,
            topic: Some("updates".to_string()),
            metadata: {
                let mut map = HashMap::new();
                map.insert("version".to_string(), "0.9".to_string());
                map
            },
        };
        
        // Convert to wire format using v0.9
        let wire_message = message.to_wire_message(ProtocolVersion::V0_9).await.unwrap();
        
        // Verify wire message properties
        assert_eq!(wire_message.version, "0.9");
        assert_eq!(wire_message.format, WireFormat::Json);
        
        // Parse the JSON to verify format-specific adaptations
        let json: Value = serde_json::from_slice(&wire_message.data).unwrap();
        let obj = json.as_object().unwrap();
        
        // Verify legacy format specific fields
        assert_eq!(obj.get("id").unwrap().as_str().unwrap(), "test-123");
        assert_eq!(obj.get("msg_type").unwrap().as_str().unwrap(), "notification");
        assert_eq!(obj.get("content").unwrap().as_str().unwrap(), "Legacy message");
        assert_eq!(obj.get("reply_to").unwrap().as_str().unwrap(), "ref-456");
        
        // Verify binary encoding
        assert!(obj.get("binary").is_some());
        
        // Convert back to message
        let decoded = Message::from_wire_message(&wire_message).await.unwrap();
        
        // Verify core fields match
        assert_eq!(decoded.id, message.id);
        assert_eq!(decoded.message_type, message.message_type);
        assert_eq!(decoded.content, message.content);
        assert_eq!(decoded.in_reply_to, message.in_reply_to);
        assert_eq!(decoded.source, message.source);
        assert_eq!(decoded.destination, message.destination);
        
        // Verify binary payload
        assert_eq!(decoded.binary_payload, message.binary_payload);
    }
    
    #[tokio::test]
    async fn test_mcpmessage_to_wire_format() {
        use crate::types::{MessageId, SecurityMetadata, ProtocolVersion as MCPProtocolVersion};
        
        // Create a test MCPMessage
        let message = MCPMessage {
            id: MessageId("mcpmsg-123".to_string()),
            type_: crate::types::MessageType::Command,
            payload: json!({"action": "get_status", "detail": "full"}),
            metadata: Some(json!({"client_version": "2.1.0"})),
            security: SecurityMetadata {
                security_level: crate::types::SecurityLevel::High,
                encryption_info: None,
                signature: Some("sig123".to_string()),
                auth_token: None,
                permissions: None,
                roles: None,
            },
            timestamp: Utc::now(),
            version: MCPProtocolVersion("1.0".to_string()),
            trace_id: Some("trace-abc".to_string()),
        };
        
        // Convert to wire format
        let wire_message = message.to_wire_message(ProtocolVersion::V1_0).await.unwrap();
        
        // Verify wire message properties
        assert_eq!(wire_message.version, "1.0");
        assert_eq!(wire_message.format, WireFormat::Json);
        
        // Convert back to message
        let decoded = MCPMessage::from_wire_message(&wire_message).await.unwrap();
        
        // Verify round-trip conversion
        assert_eq!(decoded.id.0, message.id.0);
        assert_eq!(decoded.type_, message.type_);
        assert_eq!(decoded.payload, message.payload);
        assert_eq!(decoded.security.security_level, message.security.security_level);
        assert_eq!(decoded.security.signature, message.security.signature);
        assert_eq!(decoded.security.encrypted, message.security.encrypted);
        assert_eq!(decoded.security.signed, message.security.signed);
        assert_eq!(decoded.trace_id, message.trace_id);
    }
    
    #[tokio::test]
    async fn test_mcpmessage_legacy_conversion() {
        use crate::types::{MessageId, SecurityMetadata, ProtocolVersion as MCPProtocolVersion};
        
        // Create a test MCPMessage
        let message = MCPMessage {
            id: MessageId("legacy-id-123".to_string()),
            type_: crate::types::MessageType::Event,
            payload: json!({"event_type": "status_changed", "status": "online"}),
            metadata: None,
            security: SecurityMetadata {
                encrypted: true,
                signed: true,
                key_id: None,
                signature: None,
            },
            timestamp: Utc::now(),
            version: MCPProtocolVersion("0.9".to_string()),
            trace_id: None,
        };
        
        // Convert to legacy wire format
        let wire_message = message.to_wire_message(ProtocolVersion::V0_9).await.unwrap();
        
        // Verify wire message properties
        assert_eq!(wire_message.version, "0.9");
        
        // Parse the JSON to verify format-specific adaptations
        let json: Value = serde_json::from_slice(&wire_message.data).unwrap();
        let obj = json.as_object().unwrap();
        
        // Verify legacy format fields
        assert_eq!(obj.get("id").unwrap().as_str().unwrap(), "legacy-id-123");
        assert_eq!(obj.get("type").unwrap().as_str().unwrap(), "event");
        
        // Convert back to message
        let decoded = MCPMessage::from_wire_message(&wire_message).await.unwrap();
        
        // Verify core fields match
        assert_eq!(decoded.id.0, message.id.0);
        assert_eq!(decoded.type_, message.type_);
        assert_eq!(decoded.payload, message.payload);
        assert_eq!(decoded.security.encrypted, message.security.encrypted);
        assert_eq!(decoded.security.signed, message.security.signed);
    }
} 