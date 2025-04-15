//! Domain object implementations for the MCP protocol.
//!
//! This module provides implementations of the `DomainObject` trait for various
//! domain objects used in the MCP protocol. These implementations enable translation
//! between domain objects and wire format messages.

use crate::error::MCPError;
use crate::error::protocol_err::ProtocolError;
use crate::protocol::adapter_wire::{DomainObject, WireFormatError, WireMessage, WireProtocolVersion, WireFormat};
use crate::protocol::serialization_utils::extract_string;
use crate::protocol::types::{MCPMessage, MessageType, ProtocolVersion};
use crate::message::{Message, MessageType as DomainMessageType};
use crate::security::types::EncryptionFormat;

use async_trait::async_trait;
use base64::Engine;
use chrono::Utc;
use serde::{Deserialize, Deserializer, de::Error as SerdeError};
use serde_json::{json, Value};

// ==========================================
// Message Domain Object Implementation
// ==========================================

#[async_trait]
impl DomainObject for Message {
    async fn to_wire_message(&self, version: crate::protocol::adapter_wire::WireProtocolVersion) -> crate::error::Result<WireMessage> {
        // Convert Message to Value based on protocol version
        let json_value = match version {
            crate::protocol::adapter_wire::WireProtocolVersion::V1_0 | crate::protocol::adapter_wire::WireProtocolVersion::Latest => {
                // Current version format - direct serialization
                serde_json::to_value(self)
                    .map_err(|e| MCPError::from(WireFormatError::Serialization(e.to_string())))?
            },
            crate::protocol::adapter_wire::WireProtocolVersion::V0_9 => {
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
    
    async fn from_wire_message(message: &WireMessage) -> crate::error::Result<Self>
    where
        Self: Sized,
    {
        // Parse the version using adapter_wire::ProtocolVersion
        let version = crate::protocol::adapter_wire::WireProtocolVersion::from_str(&message.version)?;
        
        match message.format {
            WireFormat::Json => {
                // Parse the JSON data
                let json: Value = serde_json::from_slice(&message.data)
                    .map_err(|e| -> crate::error::MCPError {
                        MCPError::from(WireFormatError::Deserialization(e.to_string()))
                    })?;
                
                match version {
                    crate::protocol::adapter_wire::WireProtocolVersion::V1_0 | crate::protocol::adapter_wire::WireProtocolVersion::Latest => {
                        // Current version format - direct deserialization
                        serde_json::from_value(json)
                            .map_err(|e| -> crate::error::MCPError {
                                MCPError::from(WireFormatError::Deserialization(e.to_string()))
                            })
                    },
                    crate::protocol::adapter_wire::WireProtocolVersion::V0_9 => {
                        // Legacy format (v0.9) - need to adapt to current model
                        if let Some(obj) = json.as_object() {
                            // Extract required fields with validation
                            let id = extract_string(obj, "id")?;
                            let msg_type_str = extract_string(obj, "msg_type")?;
                            let content = extract_string(obj, "content")?;
                            let source = extract_string(obj, "source")?;
                            let destination = extract_string(obj, "destination")?;
                            
                            // Convert message type string to enum
                            let message_type: DomainMessageType = match msg_type_str.as_str() {
                                "request" => DomainMessageType::Request,
                                "response" => DomainMessageType::Response,
                                "notification" => DomainMessageType::Notification,
                                "error" => DomainMessageType::Error,
                                "control" => DomainMessageType::Control,
                                "system" => DomainMessageType::System,
                                _ => return Err(MCPError::from(WireFormatError::InvalidFieldValue(
                                    "msg_type".to_string(), 
                                    format!("Unknown message type: {}", msg_type_str)
                                )).into()),
                            };
                            
                            // Extract optional fields
                            let in_reply_to = obj.get("reply_to")
                                .and_then(|v| v.as_str())
                                .map(std::string::ToString::to_string);
                            
                            // Extract binary payload if present
                            let binary_payload = obj.get("binary")
                                .and_then(|v| v.as_str())
                                .map(|s| base64::engine::general_purpose::STANDARD.decode(s))
                                .transpose()
                                .map_err(|e| -> crate::error::MCPError {
                                    MCPError::from(WireFormatError::Deserialization(
                                        format!("Invalid base64 in binary field: {e}")
                                    ))
                                })?;
                            
                            // Extract timestamp or use current time
                            let timestamp = obj.get("timestamp")
                                .and_then(serde_json::Value::as_i64)
                                .map(|ts| chrono::DateTime::from_timestamp(ts, 0))
                                .flatten()
                                .unwrap_or_else(Utc::now);
                            
                            // Extract topic if present
                            let topic = obj.get("topic")
                                .and_then(|v| v.as_str())
                                .map(std::string::ToString::to_string);
                            
                            // Extract context ID if present
                            let context_id = obj.get("context_id")
                                .and_then(|v| v.as_str())
                                .map(std::string::ToString::to_string);
                            
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
                            Ok(Self {
                                id: id.clone(), // ID should be String for Message struct
                                message_type,
                                priority: crate::message::MessagePriority::Normal, // Default value
                                content: content.clone(), // Clone content as it might be used elsewhere
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
                            )).into())
                        }
                    }
                }
            },
            WireFormat::Binary | WireFormat::Cbor => {
                // For now, treat binary and CBOR as JSON
                // In a full implementation, these would have separate decoders
                let json: Value = serde_json::from_slice(&message.data)
                    .map_err(|e| -> crate::error::MCPError {
                        MCPError::from(WireFormatError::Deserialization(e.to_string()))
                    })?;
                
                serde_json::from_value(json)
                    .map_err(|e| -> crate::error::MCPError {
                        MCPError::from(WireFormatError::Deserialization(e.to_string()))
                    })
            }
        }
    }
}

// ==========================================
// MCPMessage Domain Object Implementation
// ==========================================

#[async_trait]
impl DomainObject for MCPMessage {
    async fn to_wire_message(&self, version: WireProtocolVersion) -> crate::error::Result<WireMessage> {
        let data = serde_json::to_vec(self)
            .map_err(|e| ProtocolError::SerializationError(format!("Failed to serialize MCPMessage to JSON bytes: {}", e)))?;
        Ok(WireMessage::new(version, data, WireFormat::Json))
    }

    async fn from_wire_message(message: &WireMessage) -> crate::error::Result<Self>
    where
        Self: Sized,
    {
        if message.format != WireFormat::Json {
            return Err(MCPError::Protocol(
                ProtocolError::InvalidFormat("Only JSON wire format supported currently".to_string())
            ).into());
        }

        // Deserialize using the helper struct which is now in another module
        // We need to make the helper structs public within the crate (pub(crate))
        // and import them here.
        use crate::protocol::serialization_helpers::MCPMessageDefinitionHelper;

        let helper: MCPMessageDefinitionHelper = serde_json::from_slice(&message.data)
            .map_err(|e| MCPError::Serialization(format!("Failed to deserialize MCPMessage helper from wire data: {}", e)))?;

        // Convert the helper into the final MCPMessage using its TryFrom impl (also moved)
        MCPMessage::try_from(helper).map_err(|e: ProtocolError| {
            MCPError::Protocol(e).into()
        })
    }
}

// ==========================================
// Helper Functions
// ==========================================

// Removed: Moved to serialization_utils.rs
// /// Helper function to extract a string field from a JSON object
// fn extract_string(obj: &serde_json::Map<String, Value>, field: &str) -> Result<String> {
//     Ok(obj.get(field)
//         .ok_or_else(|| MCPError::from(WireFormatError::MissingField(field.to_string())))?
//         .as_str()
//         .ok_or_else(|| MCPError::from(WireFormatError::InvalidFieldValue(field.to_string(), "not a string".to_string())))?
//         .to_string())
// }

// ==========================================
// Deserialization Implementation for MCPMessage
// ==========================================

// --- Serde Implementation ---

impl<'de> Deserialize<'de> for MCPMessage {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Import the helper struct
        use crate::protocol::serialization_helpers::MCPMessageDefinitionHelper;
        
        let helper = MCPMessageDefinitionHelper::deserialize(deserializer)?;
        
        // Convert ProtocolError into D::Error using serde::de::Error::custom
        MCPMessage::try_from(helper).map_err(|e: ProtocolError| {
            // Use SerdeError::custom (imported as SerdeError)
            SerdeError::custom(format!("MCPMessage::try_from failed: {}", e))
        })
    }
}

// ==========================================
// Tests
// ==========================================

// MOVED to domain_objects_tests.rs

// ------------ Core Logic ------------

// MOVED to domain_objects_tests.rs

// --- Helper Structs Definition ---
// MOVED to serialization_helpers.rs

// --- TryFrom Implementations for Core Types ---

impl TryFrom<WireProtocolVersion> for ProtocolVersion {
    type Error = ProtocolError;
    fn try_from(value: WireProtocolVersion) -> std::result::Result<Self, Self::Error> {
        match value {
            WireProtocolVersion::V1_0 => Ok(Self { major: 1, minor: 0}),
            WireProtocolVersion::V0_9 => Ok(Self { major: 0, minor: 9}),
            // Assuming Latest maps to 1.0 for now
            WireProtocolVersion::Latest => Ok(Self { major: 1, minor: 0}), 
        }
    }
}

impl TryFrom<String> for MessageType {
    type Error = crate::error::ProtocolError;
    fn try_from(value: String) -> std::result::Result<Self, crate::error::ProtocolError> {
        match value.to_lowercase().as_str() {
            "command" => Ok(MessageType::Command),
            "response" => Ok(MessageType::Response),
            "event" => Ok(MessageType::Event),
            "error" => Ok(MessageType::Error),
            "setup" => Ok(MessageType::Setup),
            "heartbeat" => Ok(MessageType::Heartbeat),
            "sync" => Ok(MessageType::Sync),
            "unknown" => Ok(MessageType::Unknown),
            _ => Err(ProtocolError::InvalidFormat(format!(
                "Unknown message type string: {}",
                value
            ))),
        }
    }
}

impl TryFrom<String> for EncryptionFormat {
     // Explicitly name the error type to resolve ambiguity
     type Error = crate::error::ProtocolError;
     // Explicitly use ProtocolError in the return type
     fn try_from(value: String) -> std::result::Result<Self, crate::error::ProtocolError> {
         match value.to_lowercase().as_str() {
             "aes-256-gcm" => Ok(Self::Aes256Gcm),
             "chacha20-poly1305" => Ok(Self::ChaCha20Poly1305),
             "none" => Ok(Self::None),
             _ => Err(ProtocolError::ValidationFailed(format!(
                 "Unknown encryption format string: {}", value))),
         }
     }
}

// -- TryFrom Implementations for Helpers --
// MOVED to serialization_helpers.rs