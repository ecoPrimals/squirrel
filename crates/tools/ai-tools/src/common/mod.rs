//! Common types and traits for AI clients
//!
//! This module provides a unified interface for interacting with various AI providers,
//! including OpenAI, Anthropic, Ollama, and others. The module has been reorganized
//! into focused sub-modules for better maintainability.
//!
//! ## Architecture
//!
//! The AI tools common module is organized into several focused areas:
//!
//! * **types**: Core types and message structures
//! * **client**: AI client trait and related functionality
//! * **clients**: Concrete implementations (OpenAI, Anthropic, Ollama, Mock)
//! * **capability**: AI capabilities and task management
//! * **message**: Message types and chat structures
//! * **parameters**: Model parameters and configuration
//! * **rate_limiter**: Rate limiting functionality
//! * **registry**: Client registry for managing AI clients
//! * **tool**: Tool definitions and tool calling
//! * **usage**: Usage tracking and metrics
//!
//! ## Features
//!
//! * **Multi-provider support**: OpenAI, Anthropic, Ollama, and more
//! * **Intelligent routing**: Route requests based on cost, performance, or health
//! * **Retry logic**: Automatic retry with exponential backoff
//! * **Rate limiting**: Configurable rate limits per provider
//! * **Tool calling**: Support for function/tool calling across providers
//! * **Streaming support**: Streaming responses for real-time applications
//! * **Health monitoring**: Provider health checks and failover
//! * **Mock clients**: Testing support with mock implementations
//!
//! ## Usage
//!
//! ```rust,no_run
//! use crate::common::{AIClient, ChatRequest, MessageRole};
//! use crate::common::clients::{OpenAIClient, AnthropicClient, OllamaClient};
//!
//! // Create an OpenAI client
//! let openai_client = OpenAIClient::new("your-api-key".to_string());
//!
//! // Create a chat request
//! let request = ChatRequest::new()
//!     .add_system("You are a helpful assistant")
//!     .add_user("Hello, how are you?");
//!
//! // Process the request
//! let response = openai_client.chat(request).await?;
//! println!("Response: {:?}", response);
//! ```

// Core modules
pub mod client;
pub mod clients;
pub mod types;

// Existing sub-modules
pub mod capability;
pub mod client_registry;
pub mod message;
pub mod parameters;
pub mod providers;
pub mod rate_limiter;
pub mod registry;
pub mod tool;
pub mod usage;

// Re-export main types for convenience
pub use client::AIClient;
pub use clients::mock::MockAIClient;
pub use clients::{AnthropicClient, OllamaClient, OpenAIClient};
pub use types::{
    ChatChoice, ChatChoiceChunk, ChatMessage, ChatRequest, ChatResponse, ChatResponseChunk,
    ChatResponseStream, MessageRole, UsageInfo,
};

// Re-export existing types for backward compatibility (but avoid conflicts)
pub use capability::{AICapabilities, AITask, RoutingPreferences};
pub use client_registry::{AIRouterClient, ClientRegistry, ProviderStats};
// Note: Not re-exporting message::* to avoid conflicts with types::*
pub use parameters::ModelParameters;
pub use providers::{AICapability, AIProvider, AnthropicProvider, OllamaProvider, OpenAIProvider};
pub use rate_limiter::{RateLimiter, RateLimiterConfig};
pub use registry::ModelRegistry;
// Note: Not re-exporting tool::* and usage::* to avoid conflicts with types::*
// Individual exports from tool module (excluding ToolCall and Tool to avoid conflicts)
pub use tool::{FunctionCall, FunctionDefinition, ParameterSchema, PropertySchema, ToolType};
// Individual exports from usage module (excluding UsageInfo)
pub use usage::TokenCounter;

// Additional utility functions for backward compatibility

/// Create a provider client (factory function) - backward compatibility
pub fn create_provider_client(provider: &str, api_key: &str) -> crate::Result<Box<dyn AIClient>> {
    match provider.to_lowercase().as_str() {
        "openai" => {
            let client = clients::OpenAIClient::new(api_key.to_string());
            Ok(Box::new(client))
        }
        "anthropic" => {
            let client = clients::AnthropicClient::new(api_key.to_string());
            Ok(Box::new(client))
        }
        "ollama" => {
            let endpoint = std::env::var("OLLAMA_ENDPOINT")
                .unwrap_or_else(|_| crate::config::DefaultEndpoints::ollama_endpoint());
            let client = clients::OllamaClient::new(endpoint);
            Ok(Box::new(client))
        }
        #[cfg(test)]
        "mock" => {
            let client = clients::MockAIClient::new();
            Ok(Box::new(client))
        }
        _ => Err(crate::error::AIError::Configuration(format!(
            "Unsupported provider: {provider}"
        ))),
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

    #[test]
    fn test_create_chat_request_from_messages() {
        let messages = vec![
            create_system_message("You are a helpful assistant"),
            create_text_message("Hello"),
        ];
        let request = create_chat_request(messages, Some("gpt-4".to_string()));

        assert_eq!(request.messages.len(), 2);
        assert_eq!(request.model, Some("gpt-4".to_string()));
        assert_eq!(request.messages[0].role, MessageRole::System);
        assert_eq!(request.messages[1].role, MessageRole::User);
    }

    #[test]
    fn test_create_text_message() {
        let message = create_text_message("Hello world");
        assert_eq!(message.role, MessageRole::User);
        assert_eq!(message.content, Some("Hello world".to_string()));
    }

    #[test]
    fn test_create_system_message() {
        let message = create_system_message("You are a helpful assistant");
        assert_eq!(message.role, MessageRole::System);
        assert_eq!(
            message.content,
            Some("You are a helpful assistant".to_string())
        );
    }

    #[test]
    fn test_create_assistant_message() {
        let message = create_assistant_message("I can help you");
        assert_eq!(message.role, MessageRole::Assistant);
        assert_eq!(message.content, Some("I can help you".to_string()));
    }
}
