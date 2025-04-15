//! Common types and traits for AI clients
//!
//! This module defines the interfaces and data structures that are shared across
//! different AI providers.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::Result;

mod message;
mod parameters;
pub mod tool;
mod usage;

pub use message::*;
pub use parameters::*;
pub use tool::*;
pub use usage::*;

/// Core interface for AI clients
#[async_trait]
pub trait AIClient: Send + Sync + 'static {
    /// Get the provider name
    fn provider_name(&self) -> &str;
    
    /// Get the default model name
    fn default_model(&self) -> &str;
    
    /// Get available models
    async fn list_models(&self) -> Result<Vec<String>>;
    
    /// Send a chat request and get a chat response
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse>;
    
    /// Send a chat request and get a streaming response
    async fn chat_stream(&self, request: ChatRequest) -> Result<ChatResponseStream>;
}

/// A streaming response from an AI client
pub struct ChatResponseStream {
    /// The inner stream of chunks
    pub inner: Box<dyn futures::Stream<Item = Result<ChatResponseChunk>> + Send + Unpin>,
}

/// A chunk of a streaming chat response
#[derive(Debug, Clone)]
pub struct ChatResponseChunk {
    /// The message role (if this is a new message)
    pub role: Option<String>,
    
    /// The content delta
    pub content: Option<String>,
    
    /// Tool calls in this chunk
    pub tool_calls: Option<Vec<ToolCall>>,
}

/// Request to the AI service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    /// The chat messages
    pub messages: Vec<ChatMessage>,
    
    /// The model to use
    pub model: Option<String>,
    
    /// The model parameters
    pub parameters: Option<ModelParameters>,
    
    /// Tools that the model can use
    pub tools: Option<Vec<Tool>>,
}

impl ChatRequest {
    /// Create a new chat request
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            model: None,
            parameters: None,
            tools: None,
        }
    }
    
    /// Add a message to the request
    pub fn add_message(mut self, role: MessageRole, content: impl Into<String>) -> Self {
        self.messages.push(ChatMessage {
            role,
            content: Some(content.into()),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        });
        self
    }
    
    /// Add a system message
    pub fn add_system(self, content: impl Into<String>) -> Self {
        self.add_message(MessageRole::System, content)
    }
    
    /// Add a user message
    pub fn add_user(self, content: impl Into<String>) -> Self {
        self.add_message(MessageRole::User, content)
    }
    
    /// Add an assistant message
    pub fn add_assistant(self, content: impl Into<String>) -> Self {
        self.add_message(MessageRole::Assistant, content)
    }
    
    /// Set the model
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }
    
    /// Set the parameters
    pub fn with_parameters(mut self, parameters: ModelParameters) -> Self {
        self.parameters = Some(parameters);
        self
    }
    
    /// Add tools
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

/// Response from the AI service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    /// The response messages
    pub choices: Vec<ChatMessage>,
    
    /// Usage statistics for the request
    pub usage: Option<UsageInfo>,
}

/// Create a new chat client for the specified provider
pub fn create_client(provider: &str, api_key: impl Into<String>) -> Result<Arc<dyn AIClient>> {
    let api_key = api_key.into();
    
    match provider.to_lowercase().as_str() {
        "openai" => {
            #[cfg(feature = "openai")]
            {
                use crate::openai::OpenAIClient;
                Ok(Arc::new(OpenAIClient::new(api_key)))
            }
            #[cfg(not(feature = "openai"))]
            {
                Err(crate::Error::Configuration("OpenAI support not enabled".to_string()))
            }
        }
        "anthropic" => {
            #[cfg(feature = "anthropic")]
            {
                use crate::anthropic::AnthropicClient;
                Ok(Arc::new(AnthropicClient::new(api_key)))
            }
            #[cfg(not(feature = "anthropic"))]
            {
                Err(crate::Error::Configuration("Anthropic support not enabled".to_string()))
            }
        }
        "gemini" => {
            #[cfg(feature = "gemini")]
            {
                use crate::gemini::GeminiClient;
                Ok(Arc::new(GeminiClient::new(api_key)))
            }
            #[cfg(not(feature = "gemini"))]
            {
                Err(crate::Error::Configuration("Gemini support not enabled".to_string()))
            }
        }
        _ => Err(crate::Error::Configuration(format!("Unknown provider: {}", provider))),
    }
} 