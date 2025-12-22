//! OpenAI adapter for API-based AI
//!
//! Provides integration with OpenAI GPT models.

use super::{AiProviderAdapter, QualityTier};
use crate::api::ai::types::*;
use crate::error::PrimalError;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Instant;
use tracing::{debug, info};

/// OpenAI API adapter
pub struct OpenAIAdapter {
    client: Client,
    api_key: String,
    base_url: String,
}

#[derive(Debug, Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    choices: Vec<Choice>,
    usage: Usage,
    model: String,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: Message,
}

#[derive(Debug, Deserialize)]
struct Usage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

impl OpenAIAdapter {
    /// Create a new OpenAI adapter
    pub fn new() -> Result<Self, PrimalError> {
        let api_key = std::env::var("OPENAI_API_KEY")
            .map_err(|_| PrimalError::ConfigurationError("OPENAI_API_KEY not set".to_string()))?;

        let base_url = std::env::var("OPENAI_BASE_URL")
            .unwrap_or_else(|_| "https://api.openai.com/v1".to_string());

        Ok(Self {
            client: Client::new(),
            api_key,
            base_url,
        })
    }
}

#[async_trait]
impl AiProviderAdapter for OpenAIAdapter {
    fn provider_id(&self) -> &str {
        "openai"
    }

    fn provider_name(&self) -> &str {
        "OpenAI GPT"
    }

    fn is_local(&self) -> bool {
        false
    }

    fn cost_per_unit(&self) -> Option<f64> {
        Some(0.002) // GPT-3.5-turbo approximate
    }

    fn avg_latency_ms(&self) -> u64 {
        2000
    }

    fn quality_tier(&self) -> QualityTier {
        QualityTier::High
    }

    fn supports_text_generation(&self) -> bool {
        true
    }

    fn supports_image_generation(&self) -> bool {
        true
    }

    async fn generate_text(
        &self,
        request: TextGenerationRequest,
    ) -> Result<TextGenerationResponse, PrimalError> {
        let start = Instant::now();

        let model = request.model.unwrap_or_else(|| "gpt-3.5-turbo".to_string());

        debug!("Generating text with OpenAI model '{}'", model);

        let openai_req = OpenAIRequest {
            model: model.clone(),
            messages: vec![Message {
                role: "user".to_string(),
                content: request.prompt.clone(),
            }],
            max_tokens: Some(request.max_tokens),
            temperature: Some(request.temperature),
        };

        let url = format!("{}/chat/completions", self.base_url);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&openai_req)
            .send()
            .await
            .map_err(|e| PrimalError::NetworkError(format!("OpenAI request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "unknown error".to_string());
            return Err(PrimalError::NetworkError(format!(
                "OpenAI returned error {}: {}",
                status, error_text
            )));
        }

        let openai_response: OpenAIResponse = response.json().await.map_err(|e| {
            PrimalError::OperationFailed(format!("Failed to parse OpenAI response: {}", e))
        })?;

        let latency_ms = start.elapsed().as_millis() as u64;

        let text = openai_response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default();

        // Calculate cost (GPT-3.5-turbo pricing)
        let cost = (openai_response.usage.total_tokens as f64 * 0.002) / 1000.0;

        info!(
            "OpenAI generation complete: model={}, tokens={}, latency={}ms, cost=${:.6}",
            model, openai_response.usage.total_tokens, latency_ms, cost
        );

        Ok(TextGenerationResponse {
            text,
            provider_id: self.provider_id().to_string(),
            model,
            usage: Some(TokenUsage {
                prompt_tokens: openai_response.usage.prompt_tokens,
                completion_tokens: openai_response.usage.completion_tokens,
                total_tokens: openai_response.usage.total_tokens,
            }),
            cost_usd: Some(cost),
            latency_ms,
        })
    }

    async fn generate_image(
        &self,
        _request: ImageGenerationRequest,
    ) -> Result<ImageGenerationResponse, PrimalError> {
        // Image generation already works via existing endpoint
        // This would call DALL-E API
        Err(PrimalError::OperationFailed(
            "Image generation via adapter not yet implemented (use existing endpoint)".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openai_is_not_local() {
        std::env::set_var("OPENAI_API_KEY", "test");
        let adapter = OpenAIAdapter::new().unwrap();
        assert!(!adapter.is_local());
        assert!(adapter.cost_per_unit().is_some());
    }
}
