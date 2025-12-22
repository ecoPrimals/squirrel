//! AI API request and response types
//!
//! Modern, idiomatic Rust types for the universal AI capability endpoint.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Universal AI request for any action type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalAiRequest {
    /// The action to perform (e.g., "image.generation", "text.generation")
    pub action: String,

    /// Input data for the action (flexible JSON)
    pub input: serde_json::Value,

    /// Optional requirements for provider selection
    #[serde(default)]
    pub requirements: Option<ActionRequirements>,

    /// Request metadata
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

/// Requirements for provider selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionRequirements {
    /// Desired quality level ("low", "medium", "high")
    pub quality: Option<String>,

    /// Cost preference ("optimize", "balanced", "premium")
    pub cost_preference: Option<String>,

    /// Maximum acceptable latency in milliseconds
    pub max_latency_ms: Option<u64>,

    /// Privacy level requirement ("public", "private", "local")
    pub privacy_level: Option<String>,

    /// Preferred provider (optional override)
    pub preferred_provider: Option<String>,
}

impl Default for ActionRequirements {
    fn default() -> Self {
        Self {
            quality: Some("medium".to_string()),
            cost_preference: Some("balanced".to_string()),
            max_latency_ms: None,
            privacy_level: None,
            preferred_provider: None,
        }
    }
}

/// Universal AI response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalAiResponse {
    /// The action that was performed
    pub action: String,

    /// Output data from the action (flexible JSON)
    pub output: serde_json::Value,

    /// Response metadata
    pub metadata: ResponseMetadata,
}

/// Response metadata for transparency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMetadata {
    /// Provider that fulfilled the request
    pub provider_id: String,

    /// Provider name for display
    pub provider_name: String,

    /// Cost in USD (if available)
    pub cost_usd: Option<f64>,

    /// Actual latency in milliseconds
    pub latency_ms: u64,

    /// Request timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// Additional provider-specific metadata
    #[serde(default)]
    pub extras: HashMap<String, serde_json::Value>,
}

/// Image generation request (specific action type)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageGenerationRequest {
    /// Text prompt describing the image
    pub prompt: String,

    /// Optional negative prompt
    pub negative_prompt: Option<String>,

    /// Image size (e.g., "512x512", "1024x1024")
    #[serde(default = "default_image_size")]
    pub size: String,

    /// Number of images to generate
    #[serde(default = "default_n_images")]
    pub n: u32,

    /// Quality preference
    pub quality_preference: Option<String>,

    /// Routing constraints for provider selection
    #[serde(default)]
    pub constraints: Vec<super::constraints::RoutingConstraint>,

    /// Additional parameters
    #[serde(default)]
    pub params: HashMap<String, serde_json::Value>,
}

fn default_image_size() -> String {
    "512x512".to_string()
}

fn default_n_images() -> u32 {
    1
}

/// Image generation response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageGenerationResponse {
    /// Generated image URLs or data
    pub images: Vec<GeneratedImage>,

    /// Provider information
    pub provider_id: String,

    /// Cost if available
    pub cost_usd: Option<f64>,

    /// Generation time in milliseconds
    pub latency_ms: u64,
}

/// A single generated image
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedImage {
    /// Image URL (if hosted)
    pub url: Option<String>,

    /// Base64 encoded image data (if embedded)
    pub data: Option<String>,

    /// MIME type
    pub mime_type: String,

    /// Image dimensions
    pub width: u32,
    pub height: u32,
}

/// Text generation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextGenerationRequest {
    /// The prompt or input text
    pub prompt: String,

    /// Optional system message
    pub system: Option<String>,

    /// Maximum tokens to generate
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,

    /// Temperature for randomness (0.0 - 2.0)
    #[serde(default = "default_temperature")]
    pub temperature: f32,

    /// Model preference (optional)
    pub model: Option<String>,

    /// Routing constraints for provider selection
    #[serde(default)]
    pub constraints: Vec<super::constraints::RoutingConstraint>,

    /// Additional parameters
    #[serde(default)]
    pub params: HashMap<String, serde_json::Value>,
}

fn default_max_tokens() -> u32 {
    1024
}

fn default_temperature() -> f32 {
    0.7
}

/// Text generation response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextGenerationResponse {
    /// Generated text
    pub text: String,

    /// Provider information
    pub provider_id: String,

    /// Model used
    pub model: String,

    /// Token usage information
    pub usage: Option<TokenUsage>,

    /// Cost if available
    pub cost_usd: Option<f64>,

    /// Generation time in milliseconds
    pub latency_ms: u64,
}

/// Token usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Error response for AI operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiErrorResponse {
    /// Error code
    pub code: String,

    /// Human-readable error message
    pub message: String,

    /// Provider that failed (if applicable)
    pub provider_id: Option<String>,

    /// Whether retry might succeed
    pub retryable: bool,

    /// Additional error details
    #[serde(default)]
    pub details: HashMap<String, serde_json::Value>,
}

impl AiErrorResponse {
    /// Create a new error response
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            provider_id: None,
            retryable: false,
            details: HashMap::new(),
        }
    }

    /// Mark as retryable
    pub fn retryable(mut self) -> Self {
        self.retryable = true;
        self
    }

    /// Set provider ID
    pub fn with_provider(mut self, provider_id: impl Into<String>) -> Self {
        self.provider_id = Some(provider_id.into());
        self
    }
}
