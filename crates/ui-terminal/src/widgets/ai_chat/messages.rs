use chrono::{DateTime, Utc};

/// A message in the AI chat conversation
#[derive(Debug, Clone)]
pub struct ChatMessage {
    /// The role of the message sender (user, assistant, system)
    pub role: String,
    
    /// The content of the message
    pub content: String,
    
    /// When the message was created
    pub timestamp: DateTime<Utc>,
    
    /// Whether this message is currently being generated/streamed
    pub is_streaming: bool,
}

impl ChatMessage {
    /// Create a new user message
    pub fn user(content: String) -> Self {
        Self {
            role: "user".to_string(),
            content,
            timestamp: Utc::now(),
            is_streaming: false,
        }
    }
    
    /// Create a new assistant message
    pub fn assistant(content: String) -> Self {
        Self {
            role: "assistant".to_string(),
            content,
            timestamp: Utc::now(),
            is_streaming: false,
        }
    }
    
    /// Create a new system message
    pub fn system(content: String) -> Self {
        Self {
            role: "system".to_string(),
            content,
            timestamp: Utc::now(),
            is_streaming: false,
        }
    }
    
    /// Check if this is a user message
    pub fn is_user(&self) -> bool {
        self.role == "user"
    }
    
    /// Check if this is an assistant message
    pub fn is_assistant(&self) -> bool {
        self.role == "assistant"
    }
    
    /// Check if this is a system message
    pub fn is_system(&self) -> bool {
        self.role == "system"
    }
    
    /// Append content to this message (useful for streaming)
    pub fn append_content(&mut self, new_content: &str) {
        self.content.push_str(new_content);
    }
    
    /// Mark message as complete (not streaming anymore)
    pub fn mark_complete(&mut self) {
        self.is_streaming = false;
    }
    
    /// Start streaming this message
    pub fn start_streaming(&mut self) {
        self.is_streaming = true;
    }
} 