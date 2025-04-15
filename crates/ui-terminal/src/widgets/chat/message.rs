//! Chat message module
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

/// Represents a chat message in the AI chat interface
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    /// The content of the message
    pub content: String,
    /// Whether this message is from the user or the AI
    pub is_user: bool,
    /// When the message was created
    pub timestamp: DateTime<Utc>,
}

impl ChatMessage {
    /// Create a new user message
    pub fn new_user(content: String) -> Self {
        Self {
            content,
            is_user: true,
            timestamp: Utc::now(),
        }
    }

    /// Create a new AI message
    pub fn new_ai(content: String) -> Self {
        Self {
            content,
            is_user: false,
            timestamp: Utc::now(),
        }
    }
} 