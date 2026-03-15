// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! AI API request and response types
#![allow(dead_code)] // Public API types awaiting consumer activation
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
    #[must_use]
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

#[cfg(test)]
mod tests {
    use super::*;

    // ========== ActionRequirements Tests ==========

    #[test]
    fn test_action_requirements_default() {
        let req = ActionRequirements::default();
        assert_eq!(req.quality.as_deref(), Some("medium"));
        assert_eq!(req.cost_preference.as_deref(), Some("balanced"));
        assert!(req.max_latency_ms.is_none());
        assert!(req.privacy_level.is_none());
        assert!(req.preferred_provider.is_none());
    }

    // ========== UniversalAiRequest/Response Serde Tests ==========

    #[test]
    fn test_universal_ai_request_serde() {
        let req = UniversalAiRequest {
            action: "text.generation".to_string(),
            input: serde_json::json!({"prompt": "Hello"}),
            requirements: Some(ActionRequirements::default()),
            metadata: HashMap::new(),
        };
        let json = serde_json::to_string(&req).unwrap();
        let deser: UniversalAiRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.action, "text.generation");
    }

    #[test]
    fn test_universal_ai_request_without_requirements() {
        let json = r#"{"action":"test","input":"hello"}"#;
        let req: UniversalAiRequest = serde_json::from_str(json).unwrap();
        assert!(req.requirements.is_none());
        assert!(req.metadata.is_empty());
    }

    // ========== ImageGenerationRequest Tests ==========

    #[test]
    fn test_image_generation_request_defaults() {
        let json = r#"{"prompt":"a cat"}"#;
        let req: ImageGenerationRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.prompt, "a cat");
        assert_eq!(req.size, "512x512");
        assert_eq!(req.n, 1);
        assert!(req.negative_prompt.is_none());
        assert!(req.constraints.is_empty());
    }

    #[test]
    fn test_image_generation_response_serde() {
        let resp = ImageGenerationResponse {
            images: vec![GeneratedImage {
                url: Some("https://example.com/img.png".to_string()),
                data: None,
                mime_type: "image/png".to_string(),
                width: 512,
                height: 512,
            }],
            provider_id: "test-provider".to_string(),
            cost_usd: Some(0.02),
            latency_ms: 3000,
        };
        let json = serde_json::to_string(&resp).unwrap();
        let deser: ImageGenerationResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.images.len(), 1);
        assert_eq!(deser.images[0].width, 512);
    }

    // ========== TextGenerationRequest Tests ==========

    #[test]
    fn test_text_generation_request_defaults() {
        let json = r#"{"prompt":"Hello world"}"#;
        let req: TextGenerationRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.prompt, "Hello world");
        assert_eq!(req.max_tokens, 1024);
        assert!((req.temperature - 0.7).abs() < f32::EPSILON);
        assert!(req.model.is_none());
        assert!(req.system.is_none());
    }

    #[test]
    fn test_text_generation_response_serde() {
        let resp = TextGenerationResponse {
            text: "Generated text".to_string(),
            provider_id: "openai".to_string(),
            model: "gpt-4".to_string(),
            usage: Some(TokenUsage {
                prompt_tokens: 10,
                completion_tokens: 20,
                total_tokens: 30,
            }),
            cost_usd: Some(0.001),
            latency_ms: 500,
        };
        let json = serde_json::to_string(&resp).unwrap();
        let deser: TextGenerationResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.model, "gpt-4");
        assert_eq!(deser.usage.as_ref().unwrap().total_tokens, 30);
    }

    // ========== TokenUsage Tests ==========

    #[test]
    fn test_token_usage_serde() {
        let usage = TokenUsage {
            prompt_tokens: 100,
            completion_tokens: 200,
            total_tokens: 300,
        };
        let json = serde_json::to_string(&usage).unwrap();
        let deser: TokenUsage = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.prompt_tokens, 100);
        assert_eq!(deser.completion_tokens, 200);
        assert_eq!(deser.total_tokens, 300);
    }

    // ========== AiErrorResponse Tests ==========

    #[test]
    fn test_ai_error_response_new() {
        let err = AiErrorResponse::new("E001", "Something failed");
        assert_eq!(err.code, "E001");
        assert_eq!(err.message, "Something failed");
        assert!(err.provider_id.is_none());
        assert!(!err.retryable);
        assert!(err.details.is_empty());
    }

    #[test]
    fn test_ai_error_response_builder() {
        let err = AiErrorResponse::new("E002", "Timeout")
            .retryable()
            .with_provider("openai");
        assert!(err.retryable);
        assert_eq!(err.provider_id.as_deref(), Some("openai"));
    }

    #[test]
    fn test_ai_error_response_serde() {
        let err = AiErrorResponse::new("E003", "Rate limited")
            .retryable()
            .with_provider("anthropic");
        let json = serde_json::to_string(&err).unwrap();
        let deser: AiErrorResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.code, "E003");
        assert!(deser.retryable);
    }
}
