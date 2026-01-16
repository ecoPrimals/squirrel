//! `HuggingFace` adapter for API-based AI
//!
//! Provides integration with `HuggingFace` Inference API.
//!
//! **v1.1.0**: This adapter is only available with the `dev-direct-http` feature.
//! Production builds use `UniversalAiAdapter` with Unix sockets only.
//!
//! # Environment Variables
//!
//! - `HUGGINGFACE_API_KEY` (required): Your HuggingFace API token
//! - `HUGGINGFACE_MODEL` (optional): Default model to use (defaults to Mistral-7B)
//! - `HUGGINGFACE_BASE_URL` (optional): Base URL for API (defaults to official API)
//!
//! # Example
//!
//! ```no_run
//! use squirrel::api::ai::adapters::HuggingFaceAdapter;
//! use squirrel::api::ai::types::TextGenerationRequest;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Set API key
//! std::env::set_var("HUGGINGFACE_API_KEY", "hf_your_token_here");
//!
//! let adapter = HuggingFaceAdapter::new();
//!
//! let request = TextGenerationRequest {
//!     prompt: "Explain quantum computing in simple terms".to_string(),
//!     model: Some("mistralai/Mistral-7B-Instruct-v0.2".to_string()),
//!     max_tokens: 200,
//!     temperature: 0.7,
//! };
//!
//! let response = adapter.generate_text(request).await?;
//! println!("Generated: {}", response.text);
//! # Ok(())
//! # }
//! ```

#![cfg(feature = "dev-direct-http")]

use super::{AiProviderAdapter, QualityTier};
use crate::api::ai::types::{
    ImageGenerationRequest, ImageGenerationResponse, TextGenerationRequest, TextGenerationResponse,
};
use crate::error::PrimalError;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

/// `HuggingFace` API adapter
pub struct HuggingFaceAdapter {
    client: Client,
    api_key: Option<String>,
    base_url: String,
    default_model: String,
    max_retries: u32,
}

/// HuggingFace Inference API request
#[derive(Debug, Serialize)]
struct HuggingFaceRequest {
    inputs: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    parameters: Option<HuggingFaceParameters>,
}

/// HuggingFace request parameters
#[derive(Debug, Serialize)]
struct HuggingFaceParameters {
    #[serde(skip_serializing_if = "Option::is_none")]
    max_new_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    do_sample: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    return_full_text: Option<bool>,
}

/// HuggingFace Inference API response (single item)
#[derive(Debug, Deserialize)]
struct HuggingFaceResponseItem {
    #[serde(default)]
    generated_text: Option<String>,
}

/// HuggingFace error response
#[derive(Debug, Deserialize)]
struct HuggingFaceError {
    error: String,
    #[serde(default)]
    estimated_time: Option<f64>,
}

impl HuggingFaceAdapter {
    /// Create a new `HuggingFace` adapter
    pub fn new() -> Self {
        let api_key = std::env::var("HUGGINGFACE_API_KEY").ok();

        let base_url = std::env::var("HUGGINGFACE_BASE_URL")
            .unwrap_or_else(|_| "https://api-inference.huggingface.co".to_string());

        let default_model = std::env::var("HUGGINGFACE_MODEL")
            .unwrap_or_else(|_| "mistralai/Mistral-7B-Instruct-v0.2".to_string());

        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(120))
                .build()
                .unwrap_or_else(|_| Client::new()),
            api_key,
            base_url,
            default_model,
            max_retries: 3,
        }
    }

    /// Check if `HuggingFace` is available
    pub async fn is_available(&self) -> bool {
        self.api_key.is_some()
    }

    /// Send request with retry logic
    async fn send_with_retry(
        &self,
        url: &str,
        request: &HuggingFaceRequest,
        api_key: &str,
    ) -> Result<Vec<HuggingFaceResponseItem>, PrimalError> {
        let mut last_error = None;

        for attempt in 0..self.max_retries {
            if attempt > 0 {
                let backoff = Duration::from_millis(1000 * 2_u64.pow(attempt - 1));
                debug!("Retry attempt {} after {:?}", attempt + 1, backoff);
                tokio::time::sleep(backoff).await;
            }

            let response = self
                .client
                .post(url)
                .header("Authorization", format!("Bearer {}", api_key))
                .header("Content-Type", "application/json")
                .json(request)
                .send()
                .await
                .map_err(|e| {
                    PrimalError::NetworkError(format!("HuggingFace request failed: {}", e))
                })?;

            let status = response.status();

            if status.is_success() {
                // Success - parse and return
                let items: Vec<HuggingFaceResponseItem> = response.json().await.map_err(|e| {
                    PrimalError::ParsingError(format!(
                        "Failed to parse HuggingFace response: {}",
                        e
                    ))
                })?;
                return Ok(items);
            } else if status.as_u16() == 503 {
                // Model loading - might need retry
                if let Ok(error) = response.json::<HuggingFaceError>().await {
                    if let Some(wait_time) = error.estimated_time {
                        warn!(
                            "Model loading, estimated wait: {:.2}s. Error: {}",
                            wait_time, error.error
                        );
                        last_error = Some(PrimalError::OperationFailed(format!(
                            "Model loading (estimated {}s): {}",
                            wait_time, error.error
                        )));
                        continue;
                    }
                }
            } else if status.as_u16() == 429 {
                // Rate limit - retry with backoff
                warn!("Rate limited by HuggingFace API");
                last_error = Some(PrimalError::OperationFailed(
                    "Rate limit exceeded".to_string(),
                ));
                continue;
            } else {
                // Other error - try to get error message
                let error_text = response.text().await.unwrap_or_default();
                return Err(PrimalError::OperationFailed(format!(
                    "HuggingFace API error {}: {}",
                    status, error_text
                )));
            }
        }

        Err(last_error
            .unwrap_or_else(|| PrimalError::OperationFailed("Max retries exceeded".to_string())))
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
        request: TextGenerationRequest,
    ) -> Result<TextGenerationResponse, PrimalError> {
        let start = Instant::now();

        // Check if API key is configured
        let api_key = self.api_key.as_ref().ok_or_else(|| {
            PrimalError::ConfigurationError("HUGGINGFACE_API_KEY not set".to_string())
        })?;

        // Get model from request or use default
        let model = request
            .model
            .as_ref()
            .unwrap_or(&self.default_model)
            .clone();

        debug!("Generating text with HuggingFace model '{}'", model);

        // Build request
        let hf_req = HuggingFaceRequest {
            inputs: request.prompt.clone(),
            parameters: Some(HuggingFaceParameters {
                max_new_tokens: Some(request.max_tokens),
                temperature: Some(request.temperature),
                top_p: None,
                do_sample: Some(request.temperature > 0.0),
                return_full_text: Some(false), // Only return generated text, not prompt
            }),
        };

        // Build URL
        let url = format!("{}/models/{}", self.base_url, model);

        info!("Calling HuggingFace Inference API: {}", url);

        // Send request with retry logic
        let items = self.send_with_retry(&url, &hf_req, api_key).await?;

        // Extract generated text
        let text = items
            .first()
            .and_then(|item| item.generated_text.clone())
            .ok_or_else(|| {
                PrimalError::OperationFailed(
                    "No generated text in HuggingFace response".to_string(),
                )
            })?;

        let latency_ms = start.elapsed().as_millis() as u64;

        info!(
            "HuggingFace generation complete: {} chars, {}ms",
            text.len(),
            latency_ms
        );

        // Build response
        Ok(TextGenerationResponse {
            text: text.clone(),
            model: model.clone(),
            provider_id: "huggingface".to_string(),
            usage: None, // HF Inference API doesn't return token counts
            cost_usd: Some(0.001 * (text.len() as f64 / 1000.0)), // Estimate based on chars
            latency_ms,
        })
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::ai::types::TextGenerationRequest;

    #[tokio::test]
    async fn test_huggingface_adapter_creation() {
        let adapter = HuggingFaceAdapter::new();

        assert_eq!(adapter.provider_id(), "huggingface");
        assert_eq!(adapter.provider_name(), "HuggingFace");
        assert!(!adapter.is_local());
        assert_eq!(adapter.quality_tier(), QualityTier::Standard);
        assert!(adapter.supports_text_generation());
        assert!(!adapter.supports_image_generation());
    }

    #[tokio::test]
    async fn test_huggingface_availability_without_api_key() {
        // Temporarily unset API key
        let original_key = std::env::var("HUGGINGFACE_API_KEY").ok();
        std::env::remove_var("HUGGINGFACE_API_KEY");

        let adapter = HuggingFaceAdapter::new();
        assert!(!adapter.is_available().await);

        // Restore original key if it existed
        if let Some(key) = original_key {
            std::env::set_var("HUGGINGFACE_API_KEY", key);
        }
    }

    #[test]
    fn test_huggingface_availability_with_api_key() {
        // Set a dummy API key
        std::env::set_var("HUGGINGFACE_API_KEY", "hf_test_key");

        let adapter = HuggingFaceAdapter::new();
        assert!(adapter.api_key.is_some());

        // Clean up
        std::env::remove_var("HUGGINGFACE_API_KEY");
    }

    #[tokio::test]
    async fn test_huggingface_text_generation_missing_api_key() {
        // Ensure no API key is set
        let original_key = std::env::var("HUGGINGFACE_API_KEY").ok();
        std::env::remove_var("HUGGINGFACE_API_KEY");

        let adapter = HuggingFaceAdapter::new();

        let request = TextGenerationRequest {
            prompt: "Hello, world!".to_string(),
            system: None,
            model: None,
            max_tokens: 100,
            temperature: 0.7,
            constraints: vec![],
            params: Default::default(),
        };

        let result = adapter.generate_text(request).await;
        assert!(result.is_err());

        if let Err(e) = result {
            assert!(matches!(e, PrimalError::ConfigurationError(_)));
        }

        // Restore original key if it existed
        if let Some(key) = original_key {
            std::env::set_var("HUGGINGFACE_API_KEY", key);
        }
    }

    #[tokio::test]
    async fn test_huggingface_image_generation_not_supported() {
        let adapter = HuggingFaceAdapter::new();

        let request = ImageGenerationRequest {
            prompt: "A beautiful sunset".to_string(),
            negative_prompt: None,
            size: "512x512".to_string(),
            n: 1,
            quality_preference: None,
            constraints: vec![],
            params: Default::default(),
        };

        let result = adapter.generate_image(request).await;
        assert!(result.is_err());

        if let Err(e) = result {
            assert!(matches!(e, PrimalError::OperationFailed(_)));
        }
    }

    #[test]
    fn test_huggingface_default_model_from_env() {
        // Set custom model
        std::env::set_var("HUGGINGFACE_MODEL", "meta-llama/Llama-2-7b-chat-hf");

        let adapter = HuggingFaceAdapter::new();
        assert_eq!(adapter.default_model, "meta-llama/Llama-2-7b-chat-hf");

        // Clean up
        std::env::remove_var("HUGGINGFACE_MODEL");
    }

    #[test]
    fn test_huggingface_cost_per_unit() {
        let adapter = HuggingFaceAdapter::new();
        assert_eq!(adapter.cost_per_unit(), Some(0.001));
    }

    #[test]
    fn test_huggingface_avg_latency() {
        let adapter = HuggingFaceAdapter::new();
        assert_eq!(adapter.avg_latency_ms(), 3000);
    }
}
