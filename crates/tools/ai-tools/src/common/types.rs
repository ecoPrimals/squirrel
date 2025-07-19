//! Common types and structures for AI tools
//!
//! This module contains the core types used across the AI tools system,
//! including chat messages, requests, responses, and related structures.

use serde::{Deserialize, Serialize};
use std::pin::Pin;

/// Message role in a chat conversation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageRole {
    /// System message
    System,
    /// User message
    User,
    /// Assistant message
    Assistant,
    /// Tool message
    Tool,
    /// Function message (for backward compatibility)
    Function,
}

/// Chat message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    /// Role of the message sender
    pub role: MessageRole,
    /// Content of the message
    pub content: Option<String>,
    /// Name of the sender (optional)
    pub name: Option<String>,
    /// Tool calls in the message (optional)
    pub tool_calls: Option<Vec<ToolCall>>,
    /// Tool call ID (for tool messages)
    pub tool_call_id: Option<String>,
}

/// Chat request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    /// Model to use (optional)
    pub model: Option<String>,
    /// Messages in the conversation
    pub messages: Vec<ChatMessage>,
    /// Model parameters (optional)
    pub parameters: Option<super::parameters::ModelParameters>,
    /// Available tools (optional)
    pub tools: Option<Vec<Tool>>,
}

impl ChatRequest {
    /// Create a new chat request
    pub fn new() -> Self {
        Self {
            model: None,
            messages: Vec::new(),
            parameters: None,
            tools: None,
        }
    }

    /// Add a system message
    pub fn add_system(mut self, content: &str) -> Self {
        self.messages.push(ChatMessage {
            role: MessageRole::System,
            content: Some(content.to_string()),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        });
        self
    }

    /// Add a user message
    pub fn add_user(mut self, content: &str) -> Self {
        self.messages.push(ChatMessage {
            role: MessageRole::User,
            content: Some(content.to_string()),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        });
        self
    }

    /// Add an assistant message
    pub fn add_assistant(mut self, content: &str) -> Self {
        self.messages.push(ChatMessage {
            role: MessageRole::Assistant,
            content: Some(content.to_string()),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        });
        self
    }

    /// Set the model to use
    pub fn with_model(mut self, model: &str) -> Self {
        self.model = Some(model.to_string());
        self
    }

    /// Set the model parameters
    pub fn with_parameters(mut self, parameters: super::parameters::ModelParameters) -> Self {
        self.parameters = Some(parameters);
        self
    }

    /// Add tools to the request
    pub fn with_tools(mut self, tools: Vec<Tool>) -> Self {
        self.tools = Some(tools);
        self
    }
}

impl Default for ChatRequest {
    fn default() -> Self {
        Self::new()
    }
}

/// Chat response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    /// Response choices
    pub choices: Vec<ChatChoice>,
    /// Usage information
    pub usage: Option<UsageInfo>,
    /// Model used
    pub model: String,
    /// Unique ID for the response
    pub id: String,
}

/// Chat choice in response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatChoice {
    /// Choice index
    pub index: usize,
    /// Message role
    pub role: MessageRole,
    /// Message content
    pub content: Option<String>,
    /// Finish reason
    pub finish_reason: Option<String>,
    /// Tool calls made by the model
    pub tool_calls: Option<Vec<ToolCall>>,
}

/// Tool call made by the model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// Tool call ID
    pub id: String,
    /// Tool name
    pub name: String,
    /// Tool arguments
    pub arguments: serde_json::Value,
}

/// Chat response chunk for streaming
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponseChunk {
    /// Response ID
    pub id: String,
    /// Model used
    pub model: String,
    /// Response choices
    pub choices: Vec<ChatChoiceChunk>,
}

/// Chat choice chunk for streaming
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatChoiceChunk {
    /// Choice index
    pub index: usize,
    /// Message delta
    pub delta: ChatMessage,
    /// Finish reason
    pub finish_reason: Option<String>,
}

/// Usage information for API calls
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageInfo {
    /// Tokens used in the prompt
    pub prompt_tokens: u32,
    /// Tokens generated in the completion
    pub completion_tokens: u32,
    /// Total tokens used
    pub total_tokens: u32,
}

/// Chat response stream type
pub type ChatResponseStream =
    Pin<Box<dyn futures::Stream<Item = crate::Result<ChatResponseChunk>> + Send>>;

/// Tool choice options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ToolChoice {
    /// Let the model choose
    Auto,
    /// Don't use tools
    None,
    /// Tools are required
    Required,
    /// Use a specific tool
    Specific(String),
}

/// Tool definition for function calling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    /// Tool name
    pub name: String,
    /// Tool description
    pub description: String,
    /// Tool parameters schema
    pub parameters: serde_json::Value,
    /// Function definition (optional)
    pub function: Option<FunctionDefinition>,
}

/// Function definition structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDefinition {
    /// Function name
    pub name: String,
    /// Function description
    pub description: String,
    /// Function parameters schema
    pub parameters: serde_json::Value,
}

impl Tool {
    /// Create a new tool
    pub fn new(name: String, description: String, parameters: serde_json::Value) -> Self {
        Self {
            name,
            description,
            parameters,
            function: None,
        }
    }

    /// Create a new tool with function definition
    pub fn with_function(name: String, description: String, parameters: serde_json::Value) -> Self {
        let function = FunctionDefinition {
            name: name.clone(),
            description: description.clone(),
            parameters: parameters.clone(),
        };
        Self {
            name,
            description,
            parameters,
            function: Some(function),
        }
    }
}

/// Create a chat request from messages
pub fn create_chat_request(messages: Vec<ChatMessage>, model: Option<String>) -> ChatRequest {
    ChatRequest {
        model,
        messages,
        parameters: None,
        tools: None,
    }
}

/// Create a text message
pub fn create_text_message(content: &str) -> ChatMessage {
    ChatMessage {
        role: MessageRole::User,
        content: Some(content.to_string()),
        name: None,
        tool_calls: None,
        tool_call_id: None,
    }
}

/// Create a system message
pub fn create_system_message(content: &str) -> ChatMessage {
    ChatMessage {
        role: MessageRole::System,
        content: Some(content.to_string()),
        name: None,
        tool_calls: None,
        tool_call_id: None,
    }
}

/// Create an assistant message
pub fn create_assistant_message(content: &str) -> ChatMessage {
    ChatMessage {
        role: MessageRole::Assistant,
        content: Some(content.to_string()),
        name: None,
        tool_calls: None,
        tool_call_id: None,
    }
}
