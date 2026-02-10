// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

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

#[cfg(test)]
mod tests {
    use super::*;

    // --- MessageRole tests ---
    #[test]
    fn test_message_role_serde() {
        let roles = vec![
            MessageRole::System,
            MessageRole::User,
            MessageRole::Assistant,
            MessageRole::Tool,
            MessageRole::Function,
        ];
        for role in roles {
            let json = serde_json::to_string(&role).unwrap();
            let deserialized: MessageRole = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, role);
        }
    }

    // --- ChatRequest builder tests ---
    #[test]
    fn test_chat_request_new() {
        let req = ChatRequest::new();
        assert!(req.model.is_none());
        assert!(req.messages.is_empty());
        assert!(req.parameters.is_none());
        assert!(req.tools.is_none());
    }

    #[test]
    fn test_chat_request_default() {
        let req = ChatRequest::default();
        assert!(req.messages.is_empty());
    }

    #[test]
    fn test_chat_request_add_system() {
        let req = ChatRequest::new().add_system("You are helpful");
        assert_eq!(req.messages.len(), 1);
        assert_eq!(req.messages[0].role, MessageRole::System);
        assert_eq!(req.messages[0].content.as_deref(), Some("You are helpful"));
    }

    #[test]
    fn test_chat_request_add_user() {
        let req = ChatRequest::new().add_user("Hello");
        assert_eq!(req.messages.len(), 1);
        assert_eq!(req.messages[0].role, MessageRole::User);
        assert_eq!(req.messages[0].content.as_deref(), Some("Hello"));
    }

    #[test]
    fn test_chat_request_add_assistant() {
        let req = ChatRequest::new().add_assistant("Hi there!");
        assert_eq!(req.messages.len(), 1);
        assert_eq!(req.messages[0].role, MessageRole::Assistant);
    }

    #[test]
    fn test_chat_request_builder_chain() {
        let req = ChatRequest::new()
            .with_model("gpt-4")
            .add_system("You are helpful")
            .add_user("Hello")
            .add_assistant("Hi!");

        assert_eq!(req.model.as_deref(), Some("gpt-4"));
        assert_eq!(req.messages.len(), 3);
    }

    #[test]
    fn test_chat_request_with_parameters() {
        let params = super::super::parameters::ModelParameters::new();
        let req = ChatRequest::new().with_parameters(params);
        assert!(req.parameters.is_some());
    }

    #[test]
    fn test_chat_request_with_tools() {
        let tool = Tool::new(
            "test".to_string(),
            "A test tool".to_string(),
            serde_json::json!({}),
        );
        let req = ChatRequest::new().with_tools(vec![tool]);
        assert!(req.tools.is_some());
        assert_eq!(req.tools.unwrap().len(), 1);
    }

    #[test]
    fn test_chat_request_serde() {
        let req = ChatRequest::new()
            .with_model("gpt-4")
            .add_system("System message")
            .add_user("User message");

        let json = serde_json::to_string(&req).unwrap();
        let deserialized: ChatRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.model.as_deref(), Some("gpt-4"));
        assert_eq!(deserialized.messages.len(), 2);
    }

    // --- ChatResponse tests ---
    #[test]
    fn test_chat_response_serde() {
        let resp = ChatResponse {
            choices: vec![ChatChoice {
                index: 0,
                role: MessageRole::Assistant,
                content: Some("Hello!".to_string()),
                finish_reason: Some("stop".to_string()),
                tool_calls: None,
            }],
            usage: Some(UsageInfo {
                prompt_tokens: 10,
                completion_tokens: 5,
                total_tokens: 15,
            }),
            model: "gpt-4".to_string(),
            id: "resp-1".to_string(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        let deserialized: ChatResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.choices.len(), 1);
        assert_eq!(deserialized.model, "gpt-4");
        assert!(deserialized.usage.is_some());
        assert_eq!(deserialized.usage.unwrap().total_tokens, 15);
    }

    // --- ToolCall tests ---
    #[test]
    fn test_tool_call_serde() {
        let tc = ToolCall {
            id: "tc-1".to_string(),
            name: "search".to_string(),
            arguments: serde_json::json!({"query": "test"}),
        };
        let json = serde_json::to_string(&tc).unwrap();
        let deserialized: ToolCall = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, "tc-1");
        assert_eq!(deserialized.name, "search");
    }

    // --- Tool tests ---
    #[test]
    fn test_tool_new() {
        let tool = Tool::new(
            "search".to_string(),
            "Search the web".to_string(),
            serde_json::json!({"type": "object"}),
        );
        assert_eq!(tool.name, "search");
        assert_eq!(tool.description, "Search the web");
        assert!(tool.function.is_none());
    }

    #[test]
    fn test_tool_with_function() {
        let tool = Tool::with_function(
            "calculator".to_string(),
            "Do math".to_string(),
            serde_json::json!({"type": "object"}),
        );
        assert_eq!(tool.name, "calculator");
        assert!(tool.function.is_some());
        let func = tool.function.unwrap();
        assert_eq!(func.name, "calculator");
        assert_eq!(func.description, "Do math");
    }

    #[test]
    fn test_tool_serde() {
        let tool = Tool::new(
            "test".to_string(),
            "A test tool".to_string(),
            serde_json::json!({"type": "object"}),
        );
        let json = serde_json::to_string(&tool).unwrap();
        let deserialized: Tool = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, "test");
    }

    // --- ToolChoice tests ---
    #[test]
    fn test_tool_choice_serde() {
        let choices = vec![
            ToolChoice::Auto,
            ToolChoice::None,
            ToolChoice::Required,
            ToolChoice::Specific("search".to_string()),
        ];
        for choice in choices {
            let json = serde_json::to_string(&choice).unwrap();
            let _deserialized: ToolChoice = serde_json::from_str(&json).unwrap();
        }
    }

    // --- ChatResponseChunk tests ---
    #[test]
    fn test_chat_response_chunk_serde() {
        let chunk = ChatResponseChunk {
            id: "chunk-1".to_string(),
            model: "gpt-4".to_string(),
            choices: vec![ChatChoiceChunk {
                index: 0,
                delta: ChatMessage {
                    role: MessageRole::Assistant,
                    content: Some("Hello".to_string()),
                    name: None,
                    tool_calls: None,
                    tool_call_id: None,
                },
                finish_reason: None,
            }],
        };
        let json = serde_json::to_string(&chunk).unwrap();
        let deserialized: ChatResponseChunk = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, "chunk-1");
        assert_eq!(deserialized.choices.len(), 1);
    }

    // --- UsageInfo tests ---
    #[test]
    fn test_usage_info_serde() {
        let usage = UsageInfo {
            prompt_tokens: 100,
            completion_tokens: 50,
            total_tokens: 150,
        };
        let json = serde_json::to_string(&usage).unwrap();
        let deserialized: UsageInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.prompt_tokens, 100);
        assert_eq!(deserialized.completion_tokens, 50);
        assert_eq!(deserialized.total_tokens, 150);
    }

    // --- Helper function tests ---
    #[test]
    fn test_create_chat_request_fn() {
        let messages = vec![
            create_system_message("You are helpful"),
            create_text_message("Hello"),
        ];
        let req = create_chat_request(messages, Some("gpt-4".to_string()));
        assert_eq!(req.messages.len(), 2);
        assert_eq!(req.model.as_deref(), Some("gpt-4"));
    }

    #[test]
    fn test_create_chat_request_no_model() {
        let messages = vec![create_text_message("Hello")];
        let req = create_chat_request(messages, None);
        assert!(req.model.is_none());
    }

    #[test]
    fn test_create_text_message_fn() {
        let msg = create_text_message("test");
        assert_eq!(msg.role, MessageRole::User);
        assert_eq!(msg.content.as_deref(), Some("test"));
        assert!(msg.name.is_none());
        assert!(msg.tool_calls.is_none());
        assert!(msg.tool_call_id.is_none());
    }

    #[test]
    fn test_create_system_message_fn() {
        let msg = create_system_message("sys");
        assert_eq!(msg.role, MessageRole::System);
    }

    #[test]
    fn test_create_assistant_message_fn() {
        let msg = create_assistant_message("asst");
        assert_eq!(msg.role, MessageRole::Assistant);
    }
}
