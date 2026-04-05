// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! AI provider trait and related request/response types.

/// AI provider trait for Squirrel AI primal
///
/// This trait defines the interface for AI providers that can be registered
/// with the Squirrel AI coordinator for dynamic model access.
#[expect(
    async_fn_in_trait,
    reason = "internal trait — all impls are Send + Sync"
)]
pub trait AIProvider: Send + Sync {
    /// Get AI capabilities
    async fn get_capabilities(&self) -> Vec<AICapability>;

    /// Health check for the provider
    async fn health_check(&self) -> ProviderHealth;

    /// Perform inference
    async fn inference(&self, request: InferenceRequest) -> Result<InferenceResponse, AIError>;

    /// Stream inference
    async fn stream_inference(&self, request: InferenceRequest)
    -> Result<InferenceStream, AIError>;

    /// Get provider name
    fn provider_name(&self) -> &str;

    /// Get provider type
    fn provider_type(&self) -> &str;
}
/// AI capability enumeration
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AICapability {
    /// Text generation
    TextGeneration,
    /// Code generation
    CodeGeneration,
    /// Image generation
    ImageGeneration,
    /// Speech synthesis
    SpeechSynthesis,
    /// Language translation
    LanguageTranslation,
    /// Question answering
    QuestionAnswering,
    /// Summarization
    Summarization,
    /// Classification
    Classification,
    /// Sentiment analysis
    SentimentAnalysis,
    /// Multimodal processing
    MultiModal,
}

/// Provider health status
#[derive(Debug, Clone)]
pub struct ProviderHealth {
    /// Is the provider healthy?
    pub healthy: bool,

    /// Health status message
    pub message: String,

    /// Response time in milliseconds
    pub response_time_ms: u64,

    /// Error rate percentage
    pub error_rate: f64,

    /// Current load percentage
    pub load_percentage: f64,
}

impl ProviderHealth {
    /// Check if the provider is healthy
    #[must_use]
    pub const fn is_healthy(&self) -> bool {
        self.healthy
    }
}

/// AI inference request
#[derive(Debug, Clone)]
pub struct InferenceRequest {
    /// Request ID
    pub id: String,

    /// Input prompt or data
    pub input: String,

    /// Request parameters
    pub parameters: std::collections::HashMap<String, serde_json::Value>,

    /// Request context
    pub context: Option<String>,

    /// Maximum tokens to generate
    pub max_tokens: Option<u32>,

    /// Temperature for generation
    pub temperature: Option<f32>,

    /// Top-p for generation
    pub top_p: Option<f32>,
}

/// AI inference response
#[derive(Debug, Clone)]
pub struct InferenceResponse {
    /// Request ID
    pub request_id: String,

    /// Generated output
    pub output: String,

    /// Response metadata
    pub metadata: std::collections::HashMap<String, serde_json::Value>,

    /// Token usage information
    pub usage: TokenUsage,

    /// Response time in milliseconds
    pub response_time_ms: u64,
}

/// Token usage information
#[derive(Debug, Clone)]
pub struct TokenUsage {
    /// Input tokens
    pub input_tokens: u32,

    /// Output tokens
    pub output_tokens: u32,

    /// Total tokens
    pub total_tokens: u32,
}

/// AI inference stream
pub type InferenceStream =
    Box<dyn futures::Stream<Item = Result<InferenceChunk, AIError>> + Send + Unpin>;

/// AI inference chunk for streaming
#[derive(Debug, Clone)]
pub struct InferenceChunk {
    /// Request ID
    pub request_id: String,

    /// Chunk content
    pub content: String,

    /// Is this the final chunk?
    pub is_final: bool,

    /// Chunk metadata
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

/// AI request for coordination
#[derive(Debug, Clone)]
pub struct AIRequest {
    /// Request ID
    pub id: String,

    /// Request prompt
    pub prompt: String,

    /// Required capabilities
    pub capabilities: Vec<String>,

    /// Request context
    pub context: Option<String>,

    /// User preferences
    pub preferences: Option<std::collections::HashMap<String, serde_json::Value>>,
}

/// AI response from coordination
#[derive(Debug, Clone)]
pub struct AIResponse {
    /// Request ID
    pub request_id: String,

    /// Response content
    pub content: String,

    /// Provider that handled the request
    pub provider: String,

    /// Response metadata
    pub metadata: std::collections::HashMap<String, serde_json::Value>,

    /// Response time in milliseconds
    pub response_time_ms: u64,
}

/// AI error types
#[derive(Debug, thiserror::Error)]
pub enum AIError {
    /// Provider is not available
    #[error("Provider unavailable: {0}")]
    ProviderUnavailable(String),

    /// Provider is unhealthy
    #[error("Provider unhealthy: {0}")]
    ProviderUnhealthy(String),

    /// Invalid request
    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    /// Rate limit exceeded
    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),

    /// Authentication failed
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    /// Network error
    #[error("Network error: {0}")]
    NetworkError(String),

    /// Internal error
    #[error("Internal error: {0}")]
    InternalError(String),
}
