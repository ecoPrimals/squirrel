//! WebSocket models and data structures
//!
//! This module defines the data structures used for WebSocket communication,
//! including messages, contexts, and related types.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::collections::HashSet;
use tokio::sync::mpsc;

use super::error::WebSocketError;

/// WebSocket command sent by clients
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketCommand {
    /// Command name (e.g., "subscribe", "unsubscribe", "ping")
    pub command: String,
    
    /// Optional ID for correlating responses with requests
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    
    /// Command parameters
    #[serde(default)]
    pub params: HashMap<String, Value>,
}

/// WebSocket response sent to clients
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketResponse {
    /// Whether the operation was successful
    pub success: bool,
    
    /// Event type or command result
    pub event: String,
    
    /// Response data
    pub data: Value,
    
    /// Error message if not successful
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    
    /// Response to command ID if provided
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

/// Channel categories for subscriptions
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChannelCategory {
    /// Job status updates
    Job,
    
    /// Command status updates
    Command,
    
    /// System notifications
    Notification,
    
    /// User-specific events
    User,
    
    /// General system events
    System,
}

impl ChannelCategory {
    /// Convert a string to a ChannelCategory
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "job" => Some(Self::Job),
            "command" => Some(Self::Command),
            "notification" => Some(Self::Notification),
            "user" => Some(Self::User),
            "system" => Some(Self::System),
            _ => None,
        }
    }
    
    /// Convert a ChannelCategory to a string
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Job => "job",
            Self::Command => "command",
            Self::Notification => "notification",
            Self::User => "user",
            Self::System => "system",
        }
    }
}

/// WebSocket event structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketEvent {
    /// Event type
    pub event: String,
    
    /// Channel category
    pub category: ChannelCategory,
    
    /// Channel name
    pub channel: String,
    
    /// Event data
    pub data: Value,
    
    /// Timestamp of the event
    pub timestamp: String,
}

/// WebSocket subscription
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    /// User ID that owns the subscription
    pub user_id: String,
    
    /// Channel category
    pub category: ChannelCategory,
    
    /// Channel name
    pub channel: String,
    
    /// Optional filter criteria
    #[serde(default, skip_serializing_if = "Value::is_null")]
    pub filter: Value,
}

/// WebSocket message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketMessage {
    /// Action to perform
    pub action: String,
    /// Message data
    pub data: Value,
}

/// WebSocket connection context
pub struct WebSocketContext {
    /// Connection ID
    pub connection_id: String,
    /// User ID (if authenticated)
    pub user_id: Option<String>,
    /// User roles (if authenticated)
    pub roles: Vec<String>,
    /// Subscribed channels
    pub subscriptions: HashSet<String>,
    /// Message sender
    pub sender: mpsc::Sender<Result<String, WebSocketError>>,
}

impl WebSocketContext {
    /// Create a new WebSocket context
    pub fn new(
        connection_id: String,
        user_id: Option<String>,
        roles: Vec<String>,
        sender: mpsc::Sender<Result<String, WebSocketError>>,
    ) -> Self {
        Self {
            connection_id,
            user_id,
            roles,
            subscriptions: HashSet::new(),
            sender,
        }
    }

    /// Subscribe to a channel
    pub async fn subscribe(&self, channel: &str) -> Result<(), WebSocketError> {
        // In a real implementation, this would update the connection manager
        // For now, just log the subscription
        tracing::info!("Connection {} subscribed to channel {}", self.connection_id, channel);
        Ok(())
    }

    /// Unsubscribe from a channel
    pub async fn unsubscribe(&self, channel: &str) -> Result<(), WebSocketError> {
        // In a real implementation, this would update the connection manager
        // For now, just log the unsubscription
        tracing::info!("Connection {} unsubscribed from channel {}", self.connection_id, channel);
        Ok(())
    }

    /// Send a message to the client
    pub async fn send(&self, message: WebSocketMessage) -> Result<(), WebSocketError> {
        let json = serde_json::to_string(&message)
            .map_err(WebSocketError::JsonError)?;
        
        self.sender.send(Ok(json)).await.map_err(|_| {
            WebSocketError::SendError("Failed to send message to client".to_string())
        })
    }
} 