//! `HuggingFace` adapter for API-based AI
//!
//! Provides integration with `HuggingFace` Inference API.

use super::{AiProviderAdapter, QualityTier};
use crate::api::ai::types::{
    ImageGenerationRequest, ImageGenerationResponse, TextGenerationRequest, TextGenerationResponse,
};
use crate::error::PrimalError;
use async_trait::async_trait;
use reqwest::Client;

/// `HuggingFace` API adapter
pub struct HuggingFaceAdapter {
    client: Client,
    api_key: Option<String>,
}

impl HuggingFaceAdapter {
    /// Create a new `HuggingFace` adapter
    pub fn new() -> Self {
        let api_key = std::env::var("HUGGINGFACE_API_KEY").ok();

        Self {
            client: Client::new(),
            api_key,
        }
    }

    /// Check if `HuggingFace` is available
    pub async fn is_available(&self) -> bool {
        self.api_key.is_some()
    }
}

impl Default for HuggingFaceAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AiProviderAdapter for HuggingFaceAdapter {
    fn provider_id(&self) -> &'static str {
        "huggingface"
    }

    fn provider_name(&self) -> &'static str {
        "HuggingFace"
    }

    fn is_local(&self) -> bool {
        false
    }

    fn cost_per_unit(&self) -> Option<f64> {
        Some(0.001) // Approximate
    }

    fn avg_latency_ms(&self) -> u64 {
        3000
    }

    fn quality_tier(&self) -> QualityTier {
        QualityTier::Standard
    }

    fn supports_text_generation(&self) -> bool {
        true
    }

    fn supports_image_generation(&self) -> bool {
        false
    }

    async fn generate_text(
        &self,
        _request: TextGenerationRequest,
    ) -> Result<TextGenerationResponse, PrimalError> {
        // Placeholder - would implement HuggingFace Inference API calls
        Err(PrimalError::OperationFailed(
            "HuggingFace adapter not yet fully implemented".to_string(),
        ))
    }

    async fn generate_image(
        &self,
        _request: ImageGenerationRequest,
    ) -> Result<ImageGenerationResponse, PrimalError> {
        Err(PrimalError::OperationFailed(
            "Image generation not supported by HuggingFace adapter".to_string(),
        ))
    }
}
