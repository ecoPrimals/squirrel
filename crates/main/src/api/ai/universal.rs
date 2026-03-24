// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Universal AI Interface - Vendor-Agnostic
//!
//! This module defines a universal, capability-based interface for AI providers.
//! NO vendor-specific code. Works with ANY AI provider (cloud or local).
//!
//! TRUE PRIMAL Architecture:
//! - Zero hardcoding
//! - Capability-based discovery
//! - Runtime flexibility
//! - Vendor-agnostic

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use crate::error::PrimalError;

/// Universal AI capability interface
///
/// This trait defines the interface that ALL AI providers must implement,
/// whether cloud-based, local, or custom -- Squirrel is agnostic.
///
/// TRUE PRIMAL: Providers are discovered at runtime via capability discovery,
/// not hardcoded at compile time.
#[async_trait]
pub trait AiCapability: Send + Sync {
    /// Generate text completion
    ///
    /// This is the primary method for all AI providers. It accepts a universal
    /// request format and returns a universal response format.
    async fn complete(
        &self,
        request: UniversalAiRequest,
    ) -> Result<UniversalAiResponse, PrimalError>;

    /// Check if provider is available
    ///
    /// Returns true if the provider is reachable and ready to accept requests.
    async fn is_available(&self) -> bool;

    /// Get provider capabilities
    ///
    /// Returns a list of capability identifiers this provider supports.
    /// Examples: "ai.complete", "ai.chat", "ai.inference", "ai.embedding"
    fn capabilities(&self) -> Vec<String>;

    /// Get provider metadata
    ///
    /// Returns information about the provider, such as supported models,
    /// pricing tier, latency characteristics, etc.
    fn metadata(&self) -> ProviderMetadata;

    /// Get provider ID
    ///
    /// Returns a unique identifier for this provider instance.
    fn provider_id(&self) -> &str;
}

/// Universal AI request format (vendor-agnostic)
///
/// This struct represents a request to any AI provider. It contains only
/// universal fields that are meaningful across all providers.
///
/// Vendor-specific features should be handled by the provider primal,
/// not by Squirrel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalAiRequest {
    /// The prompt or main input text
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,

    /// Chat messages (for conversational models)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub messages: Option<Vec<ChatMessage>>,

    /// Maximum tokens to generate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,

    /// Temperature (0.0 = deterministic, 1.0+ = creative)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    /// Top-p sampling (nucleus sampling)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,

    /// Model identifier (provider-specific, optional)
    ///
    /// If not specified, provider chooses default model.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    /// Stop sequences
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,

    /// Enable streaming responses
    #[serde(default)]
    pub stream: bool,

    /// Additional provider-specific parameters
    ///
    /// This is an escape hatch for provider-specific features.
    /// Use sparingly - prefer universal fields above.
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Default for UniversalAiRequest {
    fn default() -> Self {
        Self {
            prompt: None,
            messages: None,
            max_tokens: Some(1024),
            temperature: Some(0.7),
            top_p: None,
            model: None,
            stop: None,
            stream: false,
            metadata: HashMap::new(),
        }
    }
}

/// Chat message for conversational AI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    /// Message role (user, assistant, system)
    pub role: MessageRole,

    /// Message content
    pub content: String,

    /// Optional message name (for multi-turn)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// Message role in conversation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    /// System prompt (instructions)
    System,

    /// User message (input)
    User,

    /// Assistant message (AI response)
    Assistant,
}

/// Universal AI response format (vendor-agnostic)
///
/// This struct represents a response from any AI provider. It contains only
/// universal fields that are meaningful across all providers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalAiResponse {
    /// Generated text
    pub text: String,

    /// Provider that generated this response (`Arc<str>` for O(1) clone in routing)
    pub provider_id: Arc<str>,

    /// Model that generated this response (`Arc<str>` for O(1) clone in routing)
    pub model: Arc<str>,

    /// Token usage statistics
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<TokenUsage>,

    /// Stop reason (finished normally, max tokens, stop sequence, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<String>,

    /// Response latency in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<u64>,

    /// Cost in USD (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost_usd: Option<f64>,

    /// Additional provider-specific response data
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Token usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    /// Tokens in the prompt
    pub prompt_tokens: u32,

    /// Tokens in the completion
    pub completion_tokens: u32,

    /// Total tokens (prompt + completion)
    pub total_tokens: u32,
}

/// Provider metadata
///
/// Information about an AI provider, such as supported models,
/// pricing tier, latency characteristics, etc.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderMetadata {
    /// Provider name (human-readable)
    pub name: String,

    /// Provider type (cloud, local, custom)
    pub provider_type: ProviderType,

    /// Supported models
    pub models: Vec<String>,

    /// Supported capabilities
    pub capabilities: Vec<String>,

    /// Average latency in milliseconds (if known)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_latency_ms: Option<u64>,

    /// Cost tier (free, low, medium, high)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost_tier: Option<CostTier>,

    /// Additional metadata
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Provider type classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ProviderType {
    /// Cloud-based provider (e.g., Anthropic, OpenAI)
    Cloud,

    /// Local provider (any local inference engine)
    Local,

    /// Custom provider (e.g., fine-tuned models)
    Custom,
}

/// Cost tier classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CostTier {
    /// Free tier
    Free,

    /// Low cost (< $0.01 per 1K tokens)
    Low,

    /// Medium cost ($0.01 - $0.10 per 1K tokens)
    Medium,

    /// High cost (> $0.10 per 1K tokens)
    High,
}

/// Helper to create a simple text completion request
impl UniversalAiRequest {
    /// Create a simple prompt-based request
    pub fn from_prompt(prompt: impl Into<String>) -> Self {
        Self {
            prompt: Some(prompt.into()),
            ..Default::default()
        }
    }

    /// Create a chat-based request
    pub fn from_messages(messages: Vec<ChatMessage>) -> Self {
        Self {
            messages: Some(messages),
            ..Default::default()
        }
    }

    /// Set the model
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    /// Set max tokens
    pub const fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    /// Set temperature
    pub const fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Enable streaming
    pub const fn with_streaming(mut self) -> Self {
        self.stream = true;
        self
    }
}

/// Type alias for boxed AI capability trait object
pub type BoxedAiCapability = Arc<dyn AiCapability>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_universal_request_from_prompt() {
        let request = UniversalAiRequest::from_prompt("Hello, world!");
        assert_eq!(request.prompt, Some("Hello, world!".to_string()));
        assert_eq!(request.max_tokens, Some(1024));
        assert_eq!(request.temperature, Some(0.7));
    }

    #[test]
    fn test_universal_request_builder() {
        let request = UniversalAiRequest::from_prompt("Test")
            .with_model("test-model")
            .with_max_tokens(500)
            .with_temperature(0.5)
            .with_streaming();

        assert_eq!(request.model, Some("test-model".to_string()));
        assert_eq!(request.max_tokens, Some(500));
        assert_eq!(request.temperature, Some(0.5));
        assert!(request.stream);
    }

    #[test]
    fn test_chat_message_creation() {
        let message = ChatMessage {
            role: MessageRole::User,
            content: "Hello!".to_string(),
            name: None,
        };

        assert_eq!(message.role, MessageRole::User);
        assert_eq!(message.content, "Hello!");
    }

    #[test]
    fn test_universal_response_creation() {
        let response = UniversalAiResponse {
            text: "Hello!".to_string(),
            provider_id: Arc::from("test-provider"),
            model: Arc::from("test-model"),
            usage: Some(TokenUsage {
                prompt_tokens: 10,
                completion_tokens: 5,
                total_tokens: 15,
            }),
            stop_reason: Some("finished".to_string()),
            latency_ms: Some(100),
            cost_usd: Some(0.001),
            metadata: HashMap::new(),
        };

        assert_eq!(response.text, "Hello!");
        assert_eq!(response.provider_id.as_ref(), "test-provider");
        assert!(response.usage.is_some());
    }

    #[test]
    fn test_provider_metadata_creation() {
        let metadata = ProviderMetadata {
            name: "Test Provider".to_string(),
            provider_type: ProviderType::Local,
            models: vec!["model-1".to_string(), "model-2".to_string()],
            capabilities: vec!["ai.complete".to_string()],
            avg_latency_ms: Some(50),
            cost_tier: Some(CostTier::Free),
            extra: HashMap::new(),
        };

        assert_eq!(metadata.name, "Test Provider");
        assert_eq!(metadata.provider_type, ProviderType::Local);
        assert_eq!(metadata.models.len(), 2);
    }

    #[test]
    fn test_universal_request_default() {
        let req = UniversalAiRequest::default();
        assert_eq!(req.max_tokens, Some(1024));
        assert_eq!(req.temperature, Some(0.7));
        assert!(!req.stream);
    }

    #[test]
    fn test_universal_request_from_messages() {
        let messages = vec![ChatMessage {
            role: MessageRole::User,
            content: "Hi".to_string(),
            name: None,
        }];
        let req = UniversalAiRequest::from_messages(messages);
        assert!(req.messages.is_some());
        assert_eq!(req.messages.as_ref().expect("should succeed").len(), 1);
    }

    #[test]
    fn test_message_role_serialization() {
        let roles = vec![
            MessageRole::System,
            MessageRole::User,
            MessageRole::Assistant,
        ];
        for role in roles {
            let json = serde_json::to_string(&role).expect("should succeed");
            let _: MessageRole = serde_json::from_str(&json).expect("should succeed");
        }
    }

    #[test]
    fn test_provider_type_serialization() {
        let types = vec![
            ProviderType::Cloud,
            ProviderType::Local,
            ProviderType::Custom,
        ];
        for t in types {
            let json = serde_json::to_string(&t).expect("should succeed");
            let _: ProviderType = serde_json::from_str(&json).expect("should succeed");
        }
    }

    #[test]
    fn test_cost_tier_serialization() {
        let tiers = vec![
            CostTier::Free,
            CostTier::Low,
            CostTier::Medium,
            CostTier::High,
        ];
        for t in tiers {
            let json = serde_json::to_string(&t).expect("should succeed");
            let _: CostTier = serde_json::from_str(&json).expect("should succeed");
        }
    }

    #[test]
    fn test_token_usage_creation() {
        let usage = TokenUsage {
            prompt_tokens: 10,
            completion_tokens: 5,
            total_tokens: 15,
        };
        assert_eq!(usage.total_tokens, 15);
    }
}
