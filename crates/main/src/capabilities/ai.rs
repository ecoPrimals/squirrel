// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! AI-related capability traits

use crate::error::PrimalError;
// Native async traits (Rust 1.75+) - no async_trait needed!
use serde::{Deserialize, Serialize};

/// Request for AI inference (text generation, completion, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceRequest {
    /// The prompt or input text
    pub prompt: String,

    /// Optional model identifier (e.g., "gpt-4", "claude-3")
    /// If None, provider chooses best available model
    pub model: Option<String>,

    /// Temperature for randomness (0.0 = deterministic, 2.0 = very random)
    pub temperature: Option<f32>,

    /// Maximum tokens to generate
    pub max_tokens: Option<usize>,

    /// System prompt / context
    pub system: Option<String>,
}

/// Response from AI inference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceResponse {
    /// Generated text
    pub text: String,

    /// Model that generated the response
    pub model: String,

    /// Number of tokens used
    pub tokens_used: usize,

    /// Latency in milliseconds
    pub latency_ms: u64,

    /// Additional metadata
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

/// Request for text embeddings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingsRequest {
    /// Text to embed
    pub text: String,

    /// Optional model for embeddings
    pub model: Option<String>,
}

/// Response with embeddings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingsResponse {
    /// The embedding vector
    pub embedding: Vec<f32>,

    /// Model used
    pub model: String,

    /// Dimensionality of the embedding
    pub dimensions: usize,
}

/// Capability for AI inference (text generation)
///
/// Any provider implementing this trait can satisfy "ai.inference" capability requests.
/// This could be Songbird, OpenAI, Anthropic, local LLM, etc.

pub trait AiInferenceCapability: Send + Sync {
    /// Perform inference with the given request
    async fn infer(&self, request: InferenceRequest) -> Result<InferenceResponse, PrimalError>;

    /// Get available models
    async fn list_models(&self) -> Result<Vec<String>, PrimalError>;
}

/// Capability for generating embeddings

pub trait EmbeddingsCapability: Send + Sync {
    /// Generate embeddings for text
    async fn embed(&self, request: EmbeddingsRequest) -> Result<EmbeddingsResponse, PrimalError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inference_request_serialization() {
        let request = InferenceRequest {
            prompt: "Hello, world!".to_string(),
            model: Some("gpt-4".to_string()),
            temperature: Some(0.7),
            max_tokens: Some(100),
            system: None,
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: InferenceRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.prompt, "Hello, world!");
        assert_eq!(deserialized.model, Some("gpt-4".to_string()));
    }
}
