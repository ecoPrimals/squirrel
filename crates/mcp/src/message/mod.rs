use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::fmt;
use crate::error::Result as MCPResult;
use crate::protocol::types::{MCPMessage, ProtocolVersion};
use crate::security::types::SecurityMetadata;
use serde_json::Value;
use crate::protocol::types::MessageId;
use crate::error::MCPError;
use std::str::FromStr;

/// `MessageType` enum defines the different types of messages that can be sent in the MCP protocol
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
    /// Convert `MessageType` to a string representation
    #[must_use]
    pub const fn as_str(&self) -> &str {
        match self {
            Self::Request => "request",
            Self::Response => "response",
            Self::Notification => "notification",
            Self::StreamChunk => "stream_chunk",
            Self::Error => "error",
            Self::Control => "control",
            Self::System => "system",
            Self::Any => "any",
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
        Self::Notification
    }
}

impl std::str::FromStr for MessageType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "request" => Ok(MessageType::Request),
            "response" => Ok(MessageType::Response),
            "notification" => Ok(MessageType::Notification),
            "stream_chunk" => Ok(MessageType::StreamChunk),
            "error" => Ok(MessageType::Error),
            "control" => Ok(MessageType::Control),
            "system" => Ok(MessageType::System),
            "any" => Ok(MessageType::Any),
            _ => Err(format!("Unknown message type: {}", s)),
        }
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
        Self::Normal
    }
}

impl AsRef<str> for MessagePriority {
    fn as_ref(&self) -> &str {
        match self {
            Self::Low => "low",
            Self::Normal => "normal",
            Self::High => "high",
            Self::Urgent => "urgent",
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
    #[must_use]
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
    #[must_use]
    pub fn builder() -> MessageBuilder {
        MessageBuilder::new()
    }
    
    /// Create a new request message
    #[must_use]
    pub fn request(content: String, source: String, destination: String) -> Self {
        Self::new(MessageType::Request, content, source, destination)
    }
    
    /// Create a new response message
    #[must_use]
    pub fn response(content: String, source: String, destination: String, request_id: &str) -> Self {
        let mut msg = Self::new(MessageType::Response, content, source, destination);
        msg.in_reply_to = Some(request_id.to_string());
        msg
    }
    
    /// Create a notification message
    #[must_use]
    pub fn notification(content: String, source: String, destination: String) -> Self {
        Self::new(MessageType::Notification, content, source, destination)
    }
    
    /// Create an error message
    #[must_use]
    pub fn error(content: String, source: String, destination: String) -> Self {
        Self::new(MessageType::Error, content, source, destination)
    }
    
    /// Create a system message
    #[must_use]
    pub fn system(content: String, source: String, destination: String) -> Self {
        Self::new(MessageType::System, content, source, destination)
    }
    
    /// Set message priority
    #[must_use]
    pub const fn with_priority(mut self, priority: MessagePriority) -> Self {
        self.priority = priority;
        self
    }
    
    /// Set binary payload
    #[must_use] pub fn with_binary_payload(mut self, payload: Vec<u8>) -> Self {
        self.binary_payload = Some(payload);
        self
    }
    
    /// Set context ID
    #[must_use] pub fn with_context(mut self, context_id: String) -> Self {
        self.context_id = Some(context_id);
        self
    }
    
    /// Set topic
    #[must_use] pub fn with_topic(mut self, topic: String) -> Self {
        self.topic = Some(topic);
        self
    }
    
    /// Add metadata entry
    #[must_use] pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
    
    /// Clone this message as a response to itself
    #[must_use] pub fn create_response(&self, content: String) -> Self {
        Self::response(content, self.destination.clone(), self.source.clone(), &self.id)
    }
    
    /// Clone this message as an error response to itself
    #[must_use] pub fn create_error_response(&self, error_message: String) -> Self {
        let mut response = Self::error(error_message, self.destination.clone(), self.source.clone());
        response.in_reply_to = Some(self.id.clone());
        if let Some(context) = &self.context_id {
            response.context_id = Some(context.clone());
        }
        response
    }

    /// Get the message type as a string
    pub fn get_message_type_str(&self) -> &str {
        match self.message_type {
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

    /// Convert a Message to an `MCPMessage`
    pub async fn to_mcp_message(&self) -> MCPResult<MCPMessage> {
        // Convert message::MessageType to protocol::types::MessageType
        let protocol_message_type = match self.message_type {
            MessageType::Request => crate::protocol::types::MessageType::Command,
            MessageType::Response => crate::protocol::types::MessageType::Response,
            MessageType::Notification => crate::protocol::types::MessageType::Event,
            MessageType::Error => crate::protocol::types::MessageType::Error,
            MessageType::StreamChunk => crate::protocol::types::MessageType::Event, // Map to Event as fallback
            MessageType::Control => crate::protocol::types::MessageType::Heartbeat, // Map to Heartbeat as closest match
            MessageType::System => crate::protocol::types::MessageType::Setup, // Map to Setup as closest match
            MessageType::Any => crate::protocol::types::MessageType::Unknown, // Map to Unknown
        };

        Ok(MCPMessage {
            id: MessageId(self.id.clone()),
            type_: protocol_message_type,
            version: ProtocolVersion::default(),
            timestamp: self.timestamp,
            payload: serde_json::to_value(&self.content)?,
            metadata: if self.metadata.is_empty() {
                None
            } else {
                Some(serde_json::to_value(&self.metadata)?)
            },
            security: SecurityMetadata::default(),
            trace_id: None,
        })
    }

    /// Create a Message from an `MCPMessage`
    ///
    /// # Arguments
    /// * `message` - The MCP protocol message to convert
    ///
    /// # Errors
    /// * Returns `MCPError::Protocol` if the message is not a valid response or error
    /// * Returns `MCPError::Protocol` if the message content cannot be parsed
    pub fn from_mcp_message(message: &crate::protocol::types::MCPMessage) -> Result<Self, MCPError> {
        // Map protocol::types::MessageType to message::MessageType
        let message_type = match message.type_ {
            crate::protocol::types::MessageType::Command => MessageType::Request,
            crate::protocol::types::MessageType::Response => MessageType::Response,
            crate::protocol::types::MessageType::Event => MessageType::Notification,
            crate::protocol::types::MessageType::Error => MessageType::Error,
            crate::protocol::types::MessageType::Sync => MessageType::Request, // Mapping Sync to Request
            // Map other protocol types to appropriate domain types (e.g., System)
            crate::protocol::types::MessageType::Setup => MessageType::System,
            crate::protocol::types::MessageType::Heartbeat => MessageType::System,
            crate::protocol::types::MessageType::Unknown => MessageType::System, // Or handle as error?
        };

        // Extract payload as content string (assuming JSON - potentially lossy)
        // TODO: Review payload handling - should content be Value or handle binary?
        let content = serde_json::to_string(&message.payload).unwrap_or_else(|_| "{}".to_string());

        // Extract metadata from MCPMessage.metadata (Option<Value>)
        let mut metadata_map = HashMap::new();
        if let Some(Value::Object(map)) = &message.metadata {
             for (k, v) in map {
                 if let Value::String(s) = v {
                     metadata_map.insert(k.clone(), s.clone());
                 }
            }
        }

        // Extract other fields. 
        // TODO: Review source/destination mapping based on specs. Should it come from metadata or security context?
        let source = metadata_map.get("source").cloned().unwrap_or_else(|| "unknown_source".to_string());
        let destination = metadata_map.get("destination").cloned().unwrap_or_else(|| "unknown_destination".to_string());
        // TODO: Review in_reply_to logic. Should only be set if explicitly present?
        let in_reply_to = metadata_map.get("in_reply_to").cloned(); // Attempt to get from metadata, no fallback for now.

        Ok(Self {
            id: message.id.0.clone(),
            message_type,
            priority: MessagePriority::Normal, // TODO: Map priority if available in MCPMessage
            content,
            binary_payload: None, // TODO: Handle binary payload if needed
            timestamp: message.timestamp, // Use timestamp from MCPMessage
            in_reply_to, // Use extracted value (or None)
            source,
            destination,
            context_id: metadata_map.get("context_id").cloned(),
            topic: metadata_map.get("topic").cloned(),
            metadata: metadata_map,
        })
    }
}

/// Builder for creating Message objects
pub struct MessageBuilder {
    message_type: MessageType,
    content: String,
    binary_payload: Option<Vec<u8>>,
    source: String,
    destination: String,
    priority: MessagePriority,
    context_id: Option<String>,
    topic: Option<String>,
    metadata: HashMap<String, String>,
    in_reply_to: Option<String>,
}

impl MessageBuilder {
    /// Create a new message builder
    pub fn new() -> Self {
        Self {
            message_type: MessageType::Notification,
            content: String::new(),
            binary_payload: None,
            source: String::new(),
            destination: String::new(),
            priority: MessagePriority::Normal,
            context_id: None,
            topic: None,
            metadata: HashMap::new(),
            in_reply_to: None,
        }
    }

    /// Set the message type
    pub fn with_message_type(mut self, message_type: &str) -> Self {
        self.message_type = MessageType::from_str(message_type).unwrap_or(MessageType::Notification);
        self
    }

    /// Set the content
    pub fn with_content(mut self, content: serde_json::Value) -> Self {
        self.content = content.to_string();
        self
    }

    /// Set the content as string directly
    pub fn with_content_str(mut self, content: &str) -> Self {
        self.content = content.to_string();
        self
    }

    /// Set the payload (alias for with_content)
    pub fn with_payload(mut self, payload: serde_json::Value) -> Self {
        self.content = payload.to_string();
        self
    }

    /// Set binary payload
    pub fn with_binary_payload(mut self, payload: Vec<u8>) -> Self {
        self.binary_payload = Some(payload);
        self
    }

    /// Set the source
    pub fn with_source(mut self, source: &str) -> Self {
        self.source = source.to_string();
        self
    }

    /// Set the destination
    pub fn with_destination(mut self, destination: &str) -> Self {
        self.destination = destination.to_string();
        self
    }

    /// Set the priority
    pub fn with_priority(mut self, priority: MessagePriority) -> Self {
        self.priority = priority;
        self
    }

    /// Set the priority from string
    pub fn with_priority_str(mut self, priority: &str) -> Self {
        self.priority = match priority {
            "low" => MessagePriority::Low,
            "high" => MessagePriority::High,
            "urgent" => MessagePriority::Urgent,
            _ => MessagePriority::Normal,
        };
        self
    }

    /// Set the context ID
    pub fn with_context_id(mut self, context_id: String) -> Self {
        self.context_id = Some(context_id);
        self
    }
    
    /// Set the correlation ID (alternative name for context_id to maintain compatibility)
    pub fn with_correlation_id(mut self, correlation_id: String) -> Self {
        self.context_id = Some(correlation_id);
        self
    }

    /// Set the topic
    pub fn with_topic(mut self, topic: &str) -> Self {
        self.topic = Some(topic.to_string());
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }

    /// Set in_reply_to
    pub fn in_reply_to(mut self, message_id: &str) -> Self {
        self.in_reply_to = Some(message_id.to_string());
        self
    }

    /// Build the message
    #[must_use]
    pub fn build(self) -> Message {
        Message {
            id: Uuid::new_v4().to_string(),
            message_type: self.message_type,
            priority: self.priority,
            content: self.content,
            binary_payload: self.binary_payload,
            timestamp: Utc::now(),
            in_reply_to: self.in_reply_to,
            source: self.source,
            destination: self.destination,
            context_id: self.context_id,
            topic: self.topic,
            metadata: self.metadata,
        }
    }
}

/// Builder for creating Message objects with a fluent API
///
/// # Examples
///
/// ```
/// use squirrel_mcp::message::MessageBuilder;
/// use serde_json::json;
/// 
/// let message = MessageBuilder::new()
///     .with_message_type("command")
///     .with_content(json!("Hello world"))
///     .with_source("client-1")
///     .with_destination("server-1")
///     .with_priority_str("high")
///     .with_metadata("version", "1.0")
///     .build();
/// ```
///
/// This concludes the documentation for the message module.
#[doc(hidden)]
pub struct MessageDocumentation;