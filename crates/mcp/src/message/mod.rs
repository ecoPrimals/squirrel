use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::fmt;
use crate::types::{MCPMessage, MessageType as MCPMessageType, MessageId, SecurityMetadata, ProtocolVersion};
use std::convert::TryFrom;
use crate::error::{Result};
use serde_json::{json, Value};

/// MessageType enum defines the different types of messages that can be sent in the MCP protocol
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum MessageType {
    /// Request message requiring a response
    Request,
    
    /// Response to a request
    Response,
    
    /// Notification message not requiring a response
    Notification,
    
    /// Stream data chunk
    StreamChunk,
    
    /// Error message
    Error,
    
    /// Control message for protocol operation
    Control,
    
    /// System internal message
    System,
    
    /// Wildcard for handling any message type
    Any,
}

impl MessageType {
    /// Convert MessageType to a string representation
    pub fn as_str(&self) -> &str {
        match self {
            MessageType::Request => "request",
            MessageType::Response => "response",
            MessageType::Notification => "notification",
            MessageType::StreamChunk => "stream_chunk",
            MessageType::Error => "error",
            MessageType::Control => "control",
            MessageType::System => "system",
            MessageType::Any => "any",
        }
    }
}

impl fmt::Display for MessageType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Default for MessageType {
    fn default() -> Self {
        MessageType::Notification
    }
}

/// Message priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessagePriority {
    /// Lowest priority
    Low = 0,
    
    /// Normal priority (default)
    Normal = 1,
    
    /// High priority messages
    High = 2,
    
    /// Urgent messages (highest priority)
    Urgent = 3,
}

impl Default for MessagePriority {
    fn default() -> Self {
        MessagePriority::Normal
    }
}

impl AsRef<str> for MessagePriority {
    fn as_ref(&self) -> &str {
        match self {
            MessagePriority::Low => "low",
            MessagePriority::Normal => "normal",
            MessagePriority::High => "high",
            MessagePriority::Urgent => "urgent",
        }
    }
}

/// Core MCP Message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Unique message ID
    pub id: String,
    
    /// Message type
    pub message_type: MessageType,
    
    /// Message priority
    pub priority: MessagePriority,
    
    /// Message content
    pub content: String,
    
    /// Binary payload (if any, base64 encoded when serialized)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub binary_payload: Option<Vec<u8>>,
    
    /// Timestamp when the message was created
    #[serde(with = "chrono::serde::ts_seconds")]
    pub timestamp: DateTime<Utc>,
    
    /// For responses, the ID of the request this is responding to
    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_reply_to: Option<String>,
    
    /// Source identifier (e.g., client ID, service name)
    pub source: String,
    
    /// Destination identifier (e.g., client ID, service name, or "*" for broadcast)
    pub destination: String,
    
    /// Optional context ID to group related messages
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_id: Option<String>,
    
    /// Optional topic for pub/sub style messaging
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topic: Option<String>,
    
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl Message {
    /// Create a new message
    pub fn new(
        message_type: MessageType,
        content: String,
        source: String,
        destination: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            message_type,
            priority: MessagePriority::Normal,
            content,
            binary_payload: None,
            timestamp: Utc::now(),
            in_reply_to: None,
            source,
            destination,
            context_id: None,
            topic: None,
            metadata: HashMap::new(),
        }
    }
    
    /// Create a builder for creating a new Message
    pub fn builder() -> MessageBuilder {
        MessageBuilder::new()
    }
    
    /// Create a new request message
    pub fn request(content: String, source: String, destination: String) -> Self {
        Self::new(MessageType::Request, content, source, destination)
    }
    
    /// Create a new response message
    pub fn response(content: String, source: String, destination: String, request_id: &str) -> Self {
        let mut msg = Self::new(MessageType::Response, content, source, destination);
        msg.in_reply_to = Some(request_id.to_string());
        msg
    }
    
    /// Create a notification message
    pub fn notification(content: String, source: String, destination: String) -> Self {
        Self::new(MessageType::Notification, content, source, destination)
    }
    
    /// Create an error message
    pub fn error(content: String, source: String, destination: String) -> Self {
        Self::new(MessageType::Error, content, source, destination)
    }
    
    /// Create a system message
    pub fn system(content: String, source: String, destination: String) -> Self {
        Self::new(MessageType::System, content, source, destination)
    }
    
    /// Set message priority
    pub fn with_priority(mut self, priority: MessagePriority) -> Self {
        self.priority = priority;
        self
    }
    
    /// Set binary payload
    pub fn with_binary_payload(mut self, payload: Vec<u8>) -> Self {
        self.binary_payload = Some(payload);
        self
    }
    
    /// Set context ID
    pub fn with_context(mut self, context_id: String) -> Self {
        self.context_id = Some(context_id);
        self
    }
    
    /// Set topic
    pub fn with_topic(mut self, topic: String) -> Self {
        self.topic = Some(topic);
        self
    }
    
    /// Add metadata entry
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
    
    /// Clone this message as a response to itself
    pub fn create_response(&self, content: String) -> Self {
        Self::response(content, self.destination.clone(), self.source.clone(), &self.id)
    }
    
    /// Clone this message as an error response to itself
    pub fn create_error_response(&self, error_message: String) -> Self {
        let mut response = Self::error(error_message, self.destination.clone(), self.source.clone());
        response.in_reply_to = Some(self.id.clone());
        if let Some(context) = &self.context_id {
            response.context_id = Some(context.clone());
        }
        response
    }

    /// Convert a Message to an MCPMessage
    pub async fn to_mcp_message(&self) -> MCPMessage {
        let message_type = match self.message_type {
            MessageType::Request => MCPMessageType::Command,
            MessageType::Response => MCPMessageType::Response,
            MessageType::Notification => MCPMessageType::Event,
            MessageType::Error => MCPMessageType::Error,
            MessageType::Control => MCPMessageType::Setup,
            MessageType::System => MCPMessageType::Setup,
            MessageType::StreamChunk => MCPMessageType::Sync,
            MessageType::Any => MCPMessageType::Sync,
        };

        // Create a JSON payload from the message content and binary payload
        let mut payload = serde_json::json!({
            "content": self.content,
            "source": self.source,
            "destination": self.destination
        });

        // Add topic if present
        if let Some(topic) = &self.topic {
            payload["topic"] = serde_json::Value::String(topic.clone());
        }

        // Add context_id if present
        if let Some(context_id) = &self.context_id {
            payload["context_id"] = serde_json::Value::String(context_id.clone());
        }

        // Add in_reply_to if present
        if let Some(in_reply_to) = &self.in_reply_to {
            payload["in_reply_to"] = serde_json::Value::String(in_reply_to.clone());
        }

        // Add metadata
        if !self.metadata.is_empty() {
            let metadata_obj = serde_json::to_value(&self.metadata).unwrap_or(serde_json::Value::Null);
            payload["metadata"] = metadata_obj;
        }

        // Create the MCPMessage
        MCPMessage {
            id: MessageId(self.id.clone()),
            type_: message_type,
            payload,
            metadata: None,
            security: SecurityMetadata::default(),
            timestamp: self.timestamp,
            version: ProtocolVersion::new(1, 0),
            trace_id: Some(self.id.clone()),
        }
    }

    /// Create a Message from an MCPMessage
    pub async fn from_mcp_message(msg: &MCPMessage) -> crate::error::Result<Self> {
        // Extract content from payload
        let content = match msg.payload.get("content") {
            Some(serde_json::Value::String(content)) => content.clone(),
            Some(content) => content.to_string(),
            None => String::new(),
        };

        // Extract topic from payload
        let topic = msg.payload.get("topic").and_then(|v| {
            if let serde_json::Value::String(s) = v {
                Some(s.clone())
            } else {
                None
            }
        });

        // Extract context_id from payload
        let context_id = msg.payload.get("context_id").and_then(|v| {
            if let serde_json::Value::String(s) = v {
                Some(s.clone())
            } else {
                None
            }
        });

        // Extract in_reply_to from payload
        let in_reply_to = msg.payload.get("in_reply_to").and_then(|v| {
            if let serde_json::Value::String(s) = v {
                Some(s.clone())
            } else {
                None
            }
        });

        // Extract source and destination
        let source = match msg.payload.get("source") {
            Some(serde_json::Value::String(src)) => src.clone(),
            _ => "unknown".to_string(),
        };

        let destination = match msg.payload.get("destination") {
            Some(serde_json::Value::String(dest)) => dest.clone(),
            _ => "unknown".to_string(),
        };

        // Extract metadata
        let mut metadata = HashMap::new();
        if let Some(serde_json::Value::Object(meta_obj)) = msg.payload.get("metadata") {
            for (key, value) in meta_obj {
                if let serde_json::Value::String(val_str) = value {
                    metadata.insert(key.clone(), val_str.clone());
                } else {
                    metadata.insert(key.clone(), value.to_string());
                }
            }
        }

        // Map message type
        let message_type = match msg.type_ {
            MCPMessageType::Command => MessageType::Request,
            MCPMessageType::Response => MessageType::Response,
            MCPMessageType::Event => MessageType::Notification,
            MCPMessageType::Error => MessageType::Error,
            MCPMessageType::Setup => MessageType::Control,
            MCPMessageType::Heartbeat => MessageType::System,
            MCPMessageType::Sync => MessageType::StreamChunk,
        };

        Ok(Self {
            id: msg.id.0.clone(),
            message_type,
            priority: MessagePriority::Normal,
            content,
            binary_payload: None,
            timestamp: msg.timestamp,
            in_reply_to,
            source,
            destination,
            context_id,
            topic,
            metadata,
        })
    }
}

/// Builder for creating Message objects with a fluent API
///
/// # Examples
///
/// ```
/// use mcp::message::MessageBuilder;
/// 
/// let message = MessageBuilder::new()
///     .with_message_type("command")
///     .with_content("Hello world")
///     .with_source("client-1")
///     .with_destination("server-1")
///     .with_priority("high")
///     .with_metadata("version", "1.0")
///     .build();
/// ```
pub struct MessageBuilder {
    message_type: Option<MessageType>,
    content: Option<String>,
    binary_payload: Option<Vec<u8>>,
    source: Option<String>,
    destination: Option<String>,
    priority: MessagePriority,
    in_reply_to: Option<String>,
    context_id: Option<String>,
    topic: Option<String>,
    metadata: HashMap<String, String>,
}

impl MessageBuilder {
    /// Create a new MessageBuilder with default values
    pub fn new() -> Self {
        Self {
            message_type: None,
            content: None,
            binary_payload: None,
            source: None,
            destination: None,
            priority: MessagePriority::Normal,
            in_reply_to: None,
            context_id: None,
            topic: None,
            metadata: HashMap::new(),
        }
    }
    
    /// Set message ID
    pub fn with_id<T: Into<String>>(mut self, id: T) -> Self {
        let id_string = id.into();
        self.metadata.insert("id".to_string(), id_string.clone());
        self
    }
    
    /// Set correlation ID (same as in_reply_to for compatibility)
    pub fn with_correlation_id<T: Into<String>>(mut self, correlation_id: T) -> Self {
        self.in_reply_to = Some(correlation_id.into());
        self
    }
    
    /// Set the message type
    pub fn with_message_type<T: AsRef<str>>(mut self, message_type: T) -> Self {
        let type_str = message_type.as_ref().to_lowercase();
        self.message_type = Some(match type_str.as_str() {
            "request" => MessageType::Request,
            "response" => MessageType::Response,
            "notification" => MessageType::Notification,
            "error" => MessageType::Error,
            "control" => MessageType::Control,
            "system" => MessageType::System,
            "stream_chunk" => MessageType::StreamChunk,
            "any" => MessageType::Any,
            _ => MessageType::Notification,
        });
        self
    }
    
    /// Set the message content
    pub fn with_content<T: Into<String>>(mut self, content: T) -> Self {
        self.content = Some(content.into());
        self
    }
    
    /// Set the binary payload
    pub fn with_binary_payload(mut self, payload: Vec<u8>) -> Self {
        self.binary_payload = Some(payload);
        self
    }
    
    /// Set the source
    pub fn with_source<T: Into<String>>(mut self, source: T) -> Self {
        self.source = Some(source.into());
        self
    }
    
    /// Set the destination
    pub fn with_destination<T: Into<String>>(mut self, destination: T) -> Self {
        self.destination = Some(destination.into());
        self
    }
    
    /// Set the message priority
    pub fn with_priority<T: AsRef<str>>(mut self, priority: T) -> Self {
        let priority_str = priority.as_ref().to_lowercase();
        self.priority = match priority_str.as_str() {
            "low" => MessagePriority::Low,
            "normal" => MessagePriority::Normal,
            "high" => MessagePriority::High,
            "urgent" => MessagePriority::Urgent,
            _ => MessagePriority::Normal,
        };
        self
    }
    
    /// Set the in_reply_to field
    pub fn in_reply_to<T: Into<String>>(mut self, message_id: T) -> Self {
        self.in_reply_to = Some(message_id.into());
        self
    }
    
    /// Alias for in_reply_to for backward compatibility
    pub fn with_in_reply_to<T: Into<String>>(mut self, message_id: T) -> Self {
        self.in_reply_to = Some(message_id.into());
        self
    }
    
    /// Set the context ID
    pub fn with_context<T: Into<String>>(mut self, context_id: T) -> Self {
        self.context_id = Some(context_id.into());
        self
    }
    
    /// Set the topic
    pub fn with_topic<T: Into<String>>(mut self, topic: T) -> Self {
        self.topic = Some(topic.into());
        self
    }
    
    /// Add metadata
    pub fn with_metadata<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
    
    /// Add multiple metadata entries
    pub fn with_metadata_map(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata.extend(metadata);
        self
    }
    
    /// Set the payload as a JSON value
    pub fn with_payload<T: Into<serde_json::Value>>(mut self, payload: T) -> Self {
        let payload_json = payload.into();
        self.content = Some(payload_json.to_string());
        self
    }
    
    /// Build the Message
    pub fn build(self) -> Message {
        let message_type = self.message_type.unwrap_or(MessageType::Notification);
        let content = self.content.unwrap_or_else(|| String::new());
        let source = self.source.unwrap_or_else(|| "unknown".to_string());
        let destination = self.destination.unwrap_or_else(|| "*".to_string());
        
        let mut message = Message::new(message_type, content, source, destination);
        
        // Set the id from metadata if it exists
        if let Some(id) = self.metadata.get("id") {
            message.id = id.clone();
        }
        
        if let Some(binary_payload) = self.binary_payload {
            message.binary_payload = Some(binary_payload);
        }
        
        if let Some(in_reply_to) = self.in_reply_to {
            message.in_reply_to = Some(in_reply_to);
        }
        
        if let Some(context_id) = self.context_id {
            message.context_id = Some(context_id);
        }
        
        if let Some(topic) = self.topic {
            message.topic = Some(topic);
        }
        
        message.priority = self.priority;
        message.metadata = self.metadata;
        
        message
    }
}

impl Default for MessageBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Message codec for serialization and deserialization
pub mod codec {
    use super::*;
    use crate::error::{Result, MCPError};
    use crate::error::transport::TransportError;
    
    /// Serialize a message to JSON
    pub fn serialize_message(message: &Message) -> Result<String> {
        serde_json::to_string(message)
            .map_err(|e| TransportError::SerializationError(e).into())
    }
    
    /// Deserialize a message from JSON
    pub fn deserialize_message(json: &str) -> Result<Message> {
        serde_json::from_str(json)
            .map_err(|e| TransportError::SerializationError(e).into())
    }
    
    /// Serialize a message to binary format (JSON in this implementation)
    pub fn serialize_message_binary(message: &Message) -> Result<Vec<u8>> {
        serde_json::to_vec(message)
            .map_err(|e| TransportError::SerializationError(e).into())
    }
    
    /// Deserialize a message from binary format
    pub fn deserialize_message_binary(data: &[u8]) -> Result<Message> {
        serde_json::from_slice(data)
            .map_err(|e| TransportError::SerializationError(e).into())
    }
}

/// Implementation of TryFrom<Message> for MCPMessage
impl TryFrom<Message> for MCPMessage {
    type Error = crate::error::MCPError;

    fn try_from(message: Message) -> std::result::Result<Self, Self::Error> {
        // Simply call the to_mcp_message method which already exists
        Ok(async_std::task::block_on(message.to_mcp_message()))
    }
}

/// Implementation of TryFrom<MCPMessage> for Message
impl TryFrom<MCPMessage> for Message {
    type Error = crate::error::MCPError;

    fn try_from(msg: MCPMessage) -> std::result::Result<Self, Self::Error> {
        // Call the from_mcp_message method which handles the conversion
        Ok(async_std::task::block_on(Self::from_mcp_message(&msg))?)
    }
}

/// Implementation of TryFrom<&Message> for MCPMessage
impl TryFrom<&Message> for MCPMessage {
    type Error = crate::error::MCPError;

    fn try_from(message: &Message) -> std::result::Result<Self, Self::Error> {
        // Clone and then use the implementation for owned Message
        MCPMessage::try_from(message.clone())
    }
}

/// Implementation of TryFrom<&MCPMessage> for Message
impl TryFrom<&MCPMessage> for Message {
    type Error = crate::error::MCPError;

    fn try_from(msg: &MCPMessage) -> std::result::Result<Self, Self::Error> {
        // Clone and then use the implementation for owned MCPMessage
        Message::try_from(msg.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_message_creation() {
        let msg = Message::request(
            "Hello world".to_string(),
            "client-1".to_string(),
            "server-1".to_string(),
        );
        
        assert_eq!(msg.message_type, MessageType::Request);
        assert_eq!(msg.content, "Hello world");
        assert_eq!(msg.source, "client-1");
        assert_eq!(msg.destination, "server-1");
        assert_eq!(msg.priority, MessagePriority::Normal);
    }
    
    #[test]
    fn test_message_builder_pattern() {
        let msg = Message::notification(
            "Notification".to_string(),
            "system".to_string(),
            "*".to_string(),
        )
        .with_priority(MessagePriority::High)
        .with_context("ctx-123".to_string())
        .with_topic("alerts".to_string())
        .with_metadata("category".to_string(), "system".to_string());
        
        assert_eq!(msg.message_type, MessageType::Notification);
        assert_eq!(msg.priority, MessagePriority::High);
        assert_eq!(msg.context_id, Some("ctx-123".to_string()));
        assert_eq!(msg.topic, Some("alerts".to_string()));
        assert_eq!(msg.metadata.get("category"), Some(&"system".to_string()));
    }
    
    #[test]
    fn test_message_response() {
        let request = Message::request(
            "Query data".to_string(),
            "client-1".to_string(),
            "server-1".to_string(),
        );
        
        let response = request.create_response("Response data".to_string());
        
        assert_eq!(response.message_type, MessageType::Response);
        assert_eq!(response.in_reply_to, Some(request.id.clone()));
        assert_eq!(response.source, "server-1");
        assert_eq!(response.destination, "client-1");
    }
    
    #[test]
    fn test_message_serialization() {
        let msg = Message::notification(
            "Test message".to_string(),
            "client-1".to_string(),
            "server-1".to_string(),
        )
        .with_metadata("version".to_string(), "1.0".to_string());
        
        let json = codec::serialize_message(&msg).unwrap();
        let decoded = codec::deserialize_message(&json).unwrap();
        
        assert_eq!(decoded.id, msg.id);
        assert_eq!(decoded.content, msg.content);
        assert_eq!(decoded.metadata.get("version"), Some(&"1.0".to_string()));
    }
} 