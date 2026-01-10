//! Ollama adapter for local AI execution
//!
//! Provides integration with Ollama for running models locally.

use super::AiProviderAdapter;
use crate::api::ai::types::{
    ImageGenerationRequest, ImageGenerationResponse, TextGenerationRequest, TextGenerationResponse,
    TokenUsage,
};
use crate::error::PrimalError;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Instant;
use tracing::{debug, info, warn};

/// Ollama API adapter
pub struct OllamaAdapter {
    client: Client,
    base_url: String,
    default_model: String,
}

#[derive(Debug, Serialize)]
struct OllamaGenerateRequest {
    model: String,
    prompt: String,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<OllamaOptions>,
}

#[derive(Debug, Serialize)]
struct OllamaOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    num_predict: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
}

#[derive(Debug, Deserialize)]
struct OllamaGenerateResponse {
    #[allow(dead_code)] // Model name from Ollama API response
    model: String,
    response: String,
    #[allow(dead_code)] // Completion status flag
    done: bool,
    #[serde(default)]
    #[allow(dead_code)] // Duration metrics from Ollama
    total_duration: Option<u64>,
    #[serde(default)]
    prompt_eval_count: Option<i32>,
    #[serde(default)]
    eval_count: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct OllamaModel {
    name: String,
    #[serde(default)]
    size: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct OllamaListResponse {
    models: Vec<OllamaModel>,
}

impl OllamaAdapter {
    /// Create a new Ollama adapter
    pub fn new() -> Self {
        let base_url = std::env::var("OLLAMA_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:11434".to_string());

        let default_model =
            std::env::var("OLLAMA_DEFAULT_MODEL").unwrap_or_else(|_| "llama3.2:3b".to_string());

        Self {
            client: Client::new(),
            base_url,
            default_model,
        }
    }

    /// Check if Ollama is available
    pub async fn is_available(&self) -> bool {
        let url = format!("{}/api/tags", self.base_url);

        match self.client.get(&url).send().await {
            Ok(response) if response.status().is_success() => {
                debug!("Ollama is available at {}", self.base_url);
                true
            }
            Ok(response) => {
                warn!("Ollama returned non-success status: {}", response.status());
                false
            }
            Err(e) => {
                debug!("Ollama not available: {}", e);
                false
            }
        }
    }

    /// List available models
    pub async fn list_models(&self) -> Result<Vec<String>, PrimalError> {
        let url = format!("{}/api/tags", self.base_url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| PrimalError::OperationFailed(format!("Ollama request failed: {e}")))?;

        if !response.status().is_success() {
            return Err(PrimalError::OperationFailed(format!(
                "Ollama returned error: {}",
                response.status()
            )));
        }

        let list: OllamaListResponse = response.json().await.map_err(|e| {
            PrimalError::OperationFailed(format!("Failed to parse Ollama response: {e}"))
        })?;

        Ok(list.models.into_iter().map(|m| m.name).collect())
    }

    /// Select best available model for request
    async fn select_model(&self, _request: &TextGenerationRequest) -> String {
        // TODO: Implement smart model selection based on request
        // For now, use default
        self.default_model.clone()
    }
}

impl Default for OllamaAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AiProviderAdapter for OllamaAdapter {
    fn provider_id(&self) -> &'static str {
        "ollama"
    }

    fn provider_name(&self) -> &'static str {
        "Ollama (Local)"
    }

    fn is_local(&self) -> bool {
        true
    }

    fn cost_per_unit(&self) -> Option<f64> {
        Some(0.0) // Local models are free
    }

    fn avg_latency_ms(&self) -> u64 {
        1500 // Typical local inference time
    }

    fn quality_tier(&self) -> super::QualityTier {
        super::QualityTier::Standard // Most Ollama models are standard quality
    }

    fn supports_text_generation(&self) -> bool {
        true
    }

    fn supports_image_generation(&self) -> bool {
        false // Most Ollama models don't support image gen
    }

    async fn generate_text(
        &self,
        request: TextGenerationRequest,
    ) -> Result<TextGenerationResponse, PrimalError> {
        let start = Instant::now();

        let model = self.select_model(&request).await;

        debug!(
            "Generating text with Ollama model '{}' for prompt: '{}'",
            model,
            &request.prompt[..request.prompt.len().min(50)]
        );

        let ollama_req = OllamaGenerateRequest {
            model: model.clone(),
            prompt: request.prompt.clone(),
            stream: false,
            options: Some(OllamaOptions {
                num_predict: Some(request.max_tokens as i32),
                temperature: Some(request.temperature),
                top_p: None, // Not all requests have top_p
            }),
        };

        let url = format!("{}/api/generate", self.base_url);

        let response = self
            .client
            .post(&url)
            .json(&ollama_req)
            .send()
            .await
            .map_err(|e| PrimalError::OperationFailed(format!("Ollama request failed: {e}")))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "unknown error".to_string());
            return Err(PrimalError::OperationFailed(format!(
                "Ollama returned error {status}: {error_text}"
            )));
        }

        let ollama_response: OllamaGenerateResponse = response.json().await.map_err(|e| {
            PrimalError::OperationFailed(format!("Failed to parse Ollama response: {e}"))
        })?;

        let latency_ms = start.elapsed().as_millis() as u64;

        // Estimate token counts (Ollama provides eval counts)
        let prompt_tokens = ollama_response.prompt_eval_count.unwrap_or(0) as u32;
        let completion_tokens = ollama_response.eval_count.unwrap_or(0) as u32;

        info!(
            "Ollama generation complete: model={}, tokens={}/{}, latency={}ms",
            model, prompt_tokens, completion_tokens, latency_ms
        );

        Ok(TextGenerationResponse {
            text: ollama_response.response,
            provider_id: self.provider_id().to_string(),
            model: model.clone(),
            usage: Some(TokenUsage {
                prompt_tokens,
                completion_tokens,
                total_tokens: prompt_tokens + completion_tokens,
            }),
            cost_usd: Some(0.0), // Local is free
            latency_ms,
        })
    }

    async fn generate_image(
        &self,
        _request: ImageGenerationRequest,
    ) -> Result<ImageGenerationResponse, PrimalError> {
        Err(PrimalError::OperationFailed(
            "Image generation not supported by Ollama adapter".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ollama_availability() {
        let adapter = OllamaAdapter::new();
        // This will fail in CI but works locally
        let _available = adapter.is_available().await;
    }

    #[test]
    fn test_ollama_is_local() {
        let adapter = OllamaAdapter::new();
        assert!(adapter.is_local());
        assert_eq!(adapter.cost_per_unit(), Some(0.0));
    }
}
