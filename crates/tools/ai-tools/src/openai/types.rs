//! OpenAI API types
//!
//! This module contains types for the OpenAI API requests and responses.

use serde::{Deserialize, Serialize};

use crate::common::{ChatMessage, Tool, ToolType};

/// OpenAI API configuration
#[derive(Debug, Clone)]
pub struct OpenAIConfig {
    /// API base URL
    pub api_base: String,
    /// Organization ID
    pub organization: Option<String>,
    /// Default model to use
    pub default_model: String,
    /// Request timeout in seconds
    pub timeout_seconds: u64,
    /// Rate limit (requests per minute)
    pub rate_limit: Option<u32>,
    /// Whether to retry on rate limit errors
    pub retry_on_rate_limit: Option<bool>,
    /// Maximum number of retries
    pub max_retries: Option<u32>,
    /// Delay between retries in milliseconds
    pub retry_delay_ms: Option<u64>,
}

impl Default for OpenAIConfig {
    fn default() -> Self {
        Self {
            api_base: "https://api.openai.com/v1".to_string(),
            organization: None,
            default_model: super::models::DEFAULT_MODEL.to_string(),
            timeout_seconds: 30,
            rate_limit: None,
            retry_on_rate_limit: None,
            max_retries: None,
            retry_delay_ms: None,
        }
    }
}

/// Role of a message in a conversation (OpenAI version)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OpenAIMessageRole {
    /// System message
    System,
    /// User message
    User,
    /// Assistant message
    Assistant,
    /// Tool message
    Tool,
}

/// OpenAI message type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIMessage {
    /// Role of the message
    pub role: OpenAIMessageRole,

    /// Content of the message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,

    /// Name of the message sender
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Tool calls in the message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<OpenAIToolCall>>,

    /// Tool call ID for tool messages
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

/// Chat request to the OpenAI API
#[derive(Debug, Clone, Serialize)]
pub struct OpenAIChatRequest {
    /// The model to use
    pub model: String,

    /// The messages
    pub messages: Vec<ChatMessage>,

    /// Temperature
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    /// Top p
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,

    /// Maximum tokens to generate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,

    /// Stream the response
    pub stream: bool,

    /// Tools the model can use
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,

    /// Tool choice
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<String>,

    /// Frequency penalty
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f32>,

    /// Presence penalty
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f32>,

    /// Response format
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<OpenAIResponseFormat>,

    /// User ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
}

/// Response format
#[derive(Debug, Clone, Serialize)]
pub struct OpenAIResponseFormat {
    /// The type of response format
    #[serde(rename = "type")]
    pub type_field: String,
}

/// Chat response from the OpenAI API
#[derive(Debug, Clone, Deserialize)]
pub struct OpenAIChatResponse {
    /// The ID of the response
    pub id: String,

    /// The type of object
    pub object: String,

    /// The timestamp of the response
    pub created: u64,

    /// The model used
    pub model: String,

    /// The choices
    pub choices: Vec<OpenAIChatChoice>,

    /// Token usage information
    pub usage: Option<OpenAIUsage>,
}

/// A choice in a chat response
#[derive(Debug, Clone, Deserialize)]
pub struct OpenAIChatChoice {
    /// The index of the choice
    pub index: u32,

    /// The message
    pub message: ChatMessage,

    /// The reason the generation stopped
    pub finish_reason: Option<String>,
}

/// Usage information
#[derive(Debug, Clone, Deserialize)]
pub struct OpenAIUsage {
    /// Tokens used for the prompt
    pub prompt_tokens: u32,

    /// Tokens used for the completion
    pub completion_tokens: u32,

    /// Total tokens used
    pub total_tokens: u32,
}

/// A delta in a streaming response
#[derive(Debug, Clone, Deserialize)]
pub struct OpenAIChatMessageDelta {
    /// The role of the message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<OpenAIMessageRole>,

    /// The content of the message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,

    /// Tool calls
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<OpenAIToolCall>>,
}

/// A streaming response
#[derive(Debug, Clone, Deserialize)]
pub struct OpenAIChatStreamResponse {
    /// The ID of the response
    pub id: String,

    /// The type of object
    pub object: String,

    /// The timestamp of the response
    pub created: u64,

    /// The model used
    pub model: String,

    /// The choices
    pub choices: Vec<OpenAIChatStreamChoice>,
}

/// A choice in a streaming response
#[derive(Debug, Clone, Deserialize)]
pub struct OpenAIChatStreamChoice {
    /// The index of the choice
    pub index: u32,

    /// The delta of the message
    pub delta: OpenAIChatMessageDelta,

    /// The reason the generation stopped
    pub finish_reason: Option<String>,
}

/// A tool call in a streaming response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIToolCall {
    /// The ID of the tool call
    pub id: String,

    /// The type of tool
    #[serde(rename = "type")]
    pub r#type: ToolType,

    /// The function call
    pub function: crate::common::FunctionCall,
}

/// OpenAI API error response
#[derive(Debug, Clone, Deserialize)]
pub struct OpenAIErrorResponse {
    /// The error details
    pub error: OpenAIError,
}

/// OpenAI API error details
#[derive(Debug, Clone, Deserialize)]
pub struct OpenAIError {
    /// The error message
    pub message: String,
    /// The error type
    #[serde(rename = "type")]
    pub error_type: String,
    /// The error code
    pub code: Option<String>,
    /// The error parameter
    pub param: Option<String>,
}

/// Models response
#[derive(Debug, Clone, Deserialize)]
pub struct OpenAIModelsResponse {
    /// The list of models
    pub data: Vec<OpenAIModel>,
}

/// Model information
#[derive(Debug, Clone, Deserialize)]
pub struct OpenAIModel {
    /// The ID of the model
    pub id: String,

    /// The type of object
    pub object: String,

    /// The owner of the model
    pub owned_by: String,
}
