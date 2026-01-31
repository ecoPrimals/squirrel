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
//! use squirrel_ai_tools::capability_ai::{AiClient, ChatMessage};
//!
//! // Create capability-based AI client (TRUE PRIMAL!)
//! let client = AiClient::from_env()?;
//!
//! // Create chat messages
//! let messages = vec![
//!     ChatMessage::system("You are a helpful assistant"),
//!     ChatMessage::user("Hello, how are you?"),
//! ];
//!
//! // Process the request via Songbird (Unix socket delegation)
//! let response = client.chat_completion("gpt-4", messages, None).await?;
//! println!("Response: {}", response.content);
//! ```

// Core modules
pub mod client;
pub mod clients;
pub mod types;

// Old capability_provider.rs removed (depended on old providers)
// Use capability_ai::AiClient directly instead

// Existing sub-modules
pub mod capability;
// client_registry.rs removed (depended on old providers)
pub mod message;
pub mod parameters;
// providers.rs removed - old HTTP-based providers deleted
pub mod rate_limiter;
pub mod registry;
pub mod tool;
pub mod usage;

// Re-export main types for convenience
pub use client::AIClient;
pub use clients::mock::MockAIClient;

// Old HTTP clients removed - use capability_ai::AiClient instead

pub use types::{
    ChatChoice, ChatChoiceChunk, ChatMessage, ChatRequest, ChatResponse, ChatResponseChunk,
    ChatResponseStream, MessageRole, UsageInfo,
};

// Re-export existing types for backward compatibility (but avoid conflicts)
pub use capability::{AICapabilities, AITask, RoutingPreferences};
// client_registry re-exports removed (module deleted)
// Note: Not re-exporting message::* to avoid conflicts with types::*
pub use parameters::ModelParameters;
// Old provider re-exports removed - use capability_ai::AiClient instead
pub use rate_limiter::{RateLimiter, RateLimiterConfig};
pub use registry::ModelRegistry;
// Note: Not re-exporting tool::* and usage::* to avoid conflicts with types::*
// Individual exports from tool module (excluding ToolCall and Tool to avoid conflicts)
pub use tool::{FunctionCall, FunctionDefinition, ParameterSchema, PropertySchema, ToolType};
// Individual exports from usage module (excluding UsageInfo)
pub use usage::TokenCounter;

// Additional utility functions for backward compatibility

/// Create a provider client (factory function) - backward compatibility
/// NOTE: HTTP clients removed. Use capability-based patterns via Universal Transport.
/// See: crates/universal-patterns/src/transport.rs (Isomorphic IPC complete Jan 31, 2026)
pub fn create_provider_client(_provider: &str, _api_key: &str) -> crate::Result<Box<dyn AIClient>> {
    // Old HTTP-based providers removed - use capability_ai::AiClient instead
    Err(universal_error::tools::AIToolsError::Configuration(
        "Old HTTP-based providers removed. Use capability_ai::AiClient::from_env() instead."
            .to_string(),
    )
    .into())
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
