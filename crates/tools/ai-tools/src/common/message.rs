// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Messages for AI chat interfaces
//!
//! This module defines the common message types used across different AI providers.

use serde::{Deserialize, Serialize};
use std::fmt;

use crate::common::tool::ToolCall;

/// Role of a message in a conversation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    /// System message (instructions to the AI)
    System,
    /// User message
    User,
    /// Assistant message (AI response)
    Assistant,
    /// Tool message (tool response)
    Tool,
    /// Function message (deprecated, use Tool instead)
    Function,
}

impl fmt::Display for MessageRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::System => write!(f, "system"),
            Self::User => write!(f, "user"),
            Self::Assistant => write!(f, "assistant"),
            Self::Tool => write!(f, "tool"),
            Self::Function => write!(f, "function"),
        }
    }
}

/// A message in a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    /// Role of the message sender
    pub role: MessageRole,

    /// Content of the message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,

    /// Name of the sender (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Tool calls made in this message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,

    /// ID of the tool call this message is responding to
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

impl ChatMessage {
    /// Create a new system message
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::System,
            content: Some(content.into()),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        }
    }

    /// Create a new user message
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::User,
            content: Some(content.into()),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        }
    }

    /// Create a new assistant message
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::Assistant,
            content: Some(content.into()),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        }
    }

    /// Create a new tool message
    pub fn tool(content: impl Into<String>, tool_call_id: impl Into<String>) -> Self {
        Self {
            role: MessageRole::Tool,
            content: Some(content.into()),
            name: None,
            tool_calls: None,
            tool_call_id: Some(tool_call_id.into()),
        }
    }

    /// Set the name field
    #[must_use]
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set the tool calls
    #[must_use]
    pub fn with_tool_calls(mut self, tool_calls: Vec<ToolCall>) -> Self {
        self.tool_calls = Some(tool_calls);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_role_display() {
        assert_eq!(MessageRole::System.to_string(), "system");
        assert_eq!(MessageRole::User.to_string(), "user");
        assert_eq!(MessageRole::Assistant.to_string(), "assistant");
        assert_eq!(MessageRole::Tool.to_string(), "tool");
        assert_eq!(MessageRole::Function.to_string(), "function");
    }

    #[test]
    fn test_message_role_serde() {
        for role in [
            MessageRole::System,
            MessageRole::User,
            MessageRole::Assistant,
            MessageRole::Tool,
            MessageRole::Function,
        ] {
            let json = serde_json::to_string(&role).expect("serialize");
            let deser: MessageRole = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(deser, role);
        }
    }

    #[test]
    fn test_chat_message_system() {
        let msg = ChatMessage::system("You are a helpful assistant");
        assert_eq!(msg.role, MessageRole::System);
        assert_eq!(msg.content.as_deref(), Some("You are a helpful assistant"));
        assert!(msg.name.is_none());
        assert!(msg.tool_calls.is_none());
        assert!(msg.tool_call_id.is_none());
    }

    #[test]
    fn test_chat_message_user() {
        let msg = ChatMessage::user("Hello");
        assert_eq!(msg.role, MessageRole::User);
        assert_eq!(msg.content.as_deref(), Some("Hello"));
    }

    #[test]
    fn test_chat_message_assistant() {
        let msg = ChatMessage::assistant("Hi there!");
        assert_eq!(msg.role, MessageRole::Assistant);
        assert_eq!(msg.content.as_deref(), Some("Hi there!"));
    }

    #[test]
    fn test_chat_message_tool() {
        let msg = ChatMessage::tool("result data", "call-123");
        assert_eq!(msg.role, MessageRole::Tool);
        assert_eq!(msg.content.as_deref(), Some("result data"));
        assert_eq!(msg.tool_call_id.as_deref(), Some("call-123"));
    }

    #[test]
    fn test_chat_message_with_name() {
        let msg = ChatMessage::user("Hello").with_name("alice");
        assert_eq!(msg.name.as_deref(), Some("alice"));
    }

    #[test]
    fn test_chat_message_serde() {
        let msg = ChatMessage::user("Hello");
        let json = serde_json::to_string(&msg).expect("serialize");
        let deser: ChatMessage = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deser.role, MessageRole::User);
        assert_eq!(deser.content.as_deref(), Some("Hello"));
    }

    #[test]
    fn test_chat_message_serde_skip_none() {
        let msg = ChatMessage::user("Hello");
        let json = serde_json::to_string(&msg).expect("serialize");
        // None fields should be skipped
        assert!(!json.contains("name"));
        assert!(!json.contains("tool_calls"));
        assert!(!json.contains("tool_call_id"));
    }
}
