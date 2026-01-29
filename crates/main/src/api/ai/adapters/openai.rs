//! OpenAI AI Provider Adapter with Capability-Based HTTP Delegation
//!
//! TRUE PRIMAL Pattern: Discovers HTTP capability provider at runtime (no hardcoding!)
//!
//! This adapter:
//! - Reads API key from environment (OPENAI_API_KEY)
//! - Builds OpenAI-specific HTTP requests
//! - Discovers HTTP provider via capability discovery
//! - Delegates HTTP to whoever provides "http.request" capability
//! - NO knowledge of Songbird, BearDog, or any other primal names!

use super::{AiProviderAdapter, QualityTier};
use crate::api::ai::types::{
    ImageGenerationRequest, ImageGenerationResponse, TextGenerationRequest, TextGenerationResponse,
};
use crate::capabilities::discover_capability;
use crate::error::PrimalError;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tracing::{debug, error, info};
use uuid::Uuid;

/// OpenAI chat completion request format
#[derive(Debug, Serialize)]
struct OpenAiRequest {
    model: String,
    messages: Vec<OpenAiMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAiMessage {
    role: String,
    content: String,
}

/// OpenAI API response
#[derive(Debug, Deserialize)]
struct OpenAiResponse {
    id: String,
    model: String,
    choices: Vec<OpenAiChoice>,
    usage: OpenAiUsage,
}

#[derive(Debug, Deserialize)]
struct OpenAiChoice {
    message: OpenAiMessage,
    finish_reason: String,
}

#[derive(Debug, Deserialize)]
struct OpenAiUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

/// OpenAI AI Provider Adapter
///
/// Uses capability discovery to find HTTP provider (TRUE PRIMAL!)
pub struct OpenAiAdapter {
    api_key: String,
    default_model: String,
}

impl OpenAiAdapter {
    /// Create new OpenAI adapter
    ///
    /// Reads API key from OPENAI_API_KEY environment variable
    pub fn new() -> Result<Self, PrimalError> {
        let api_key = std::env::var("OPENAI_API_KEY")
            .map_err(|_| PrimalError::ConfigError("OPENAI_API_KEY not set".to_string()))?;

        Ok(Self {
            api_key,
            default_model: "gpt-4".to_string(),
        })
    }

    /// Send HTTP request via discovered HTTP capability provider
    ///
    /// TRUE PRIMAL: Discovers "http.request" provider at runtime
    /// Could be Songbird, or any other primal providing HTTP!
    async fn delegate_http(
        &self,
        method: &str,
        url: &str,
        headers: HashMap<String, String>,
        body: serde_json::Value,
    ) -> Result<serde_json::Value, PrimalError> {
        // Discover who provides HTTP capability (TRUE PRIMAL!)
        let http_provider = discover_capability("http.request")
            .await
            .map_err(|e| PrimalError::NetworkError(format!("No HTTP provider found: {}", e)))?;

        debug!(
            "Delegating HTTP to {} (discovered via capability)",
            http_provider.id
        );

        // Connect to HTTP provider
        let stream = UnixStream::connect(&http_provider.socket).await?;

        // Build JSON-RPC request for HTTP delegation
        // BIOME OS FIX (Jan 28, 2026): Songbird expects body as STRING, not object
        let body_string = match body {
            serde_json::Value::String(s) => serde_json::Value::String(s),
            serde_json::Value::Null => serde_json::Value::Null,
            other => serde_json::Value::String(serde_json::to_string(&other)?),
        };

        let rpc_request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "http.request",
            "params": {
                "method": method,
                "url": url,
                "headers": headers,
                "body": body_string,
            },
            "id": Uuid::new_v4().to_string(),
        });

        // Send request
        let request_json = serde_json::to_string(&rpc_request)?;
        let mut request_bytes = request_json.as_bytes().to_vec();
        request_bytes.push(b'\n');

        let (read_half, mut write_half) = stream.into_split();
        write_half.write_all(&request_bytes).await?;

        // Read response
        let mut reader = BufReader::new(read_half);
        let mut response_line = String::new();
        reader.read_line(&mut response_line).await?;

        // Parse JSON-RPC response
        let rpc_response: serde_json::Value = serde_json::from_str(&response_line)?;

        if let Some(error) = rpc_response.get("error") {
            return Err(PrimalError::NetworkError(format!(
                "HTTP delegation error: {}",
                error
            )));
        }

        rpc_response
            .get("result")
            .cloned()
            .ok_or_else(|| PrimalError::NetworkError("No result in HTTP response".to_string()))
    }

    /// Call OpenAI API
    async fn call_openai(
        &self,
        request: TextGenerationRequest,
    ) -> Result<TextGenerationResponse, PrimalError> {
        // Build OpenAI-specific request
        let openai_request = OpenAiRequest {
            model: request.model.unwrap_or_else(|| self.default_model.clone()),
            messages: vec![OpenAiMessage {
                role: "user".to_string(),
                content: request.prompt.clone(),
            }],
            temperature: Some(request.temperature),
            max_tokens: Some(request.max_tokens),
        };

        // Build headers with API key
        let mut headers = HashMap::new();
        headers.insert(
            "Authorization".to_string(),
            format!("Bearer {}", self.api_key),
        );
        headers.insert("content-type".to_string(), "application/json".to_string());

        // Delegate HTTP to discovered provider (TRUE PRIMAL!)
        let response_json = self
            .delegate_http(
                "POST",
                "https://api.openai.com/v1/chat/completions",
                headers,
                serde_json::to_value(&openai_request)?,
            )
            .await?;

        // Parse HTTP response - body comes as string from Songbird
        let body_value = response_json
            .get("body")
            .cloned()
            .ok_or_else(|| PrimalError::ParsingError("No body in HTTP response".to_string()))?;

        // BIOME OS FIX (Jan 29, 2026): Songbird returns body as string, parse it
        let http_response: serde_json::Value = match body_value {
            serde_json::Value::String(s) => serde_json::from_str(&s).map_err(|e| {
                PrimalError::ParsingError(format!("Failed to parse body JSON: {}", e))
            })?,
            other => other, // Already parsed (for future compatibility)
        };

        // Parse OpenAI response
        let openai_response: OpenAiResponse = serde_json::from_value(http_response)?;

        // Convert to universal format
        Ok(TextGenerationResponse {
            text: openai_response.choices[0].message.content.clone(),
            provider_id: "openai".to_string(),
            model: openai_response.model,
            usage: Some(crate::api::ai::types::TokenUsage {
                prompt_tokens: openai_response.usage.prompt_tokens,
                completion_tokens: openai_response.usage.completion_tokens,
                total_tokens: openai_response.usage.total_tokens,
            }),
            cost_usd: None, // TODO: Calculate cost based on usage
            latency_ms: 0,  // TODO: Track request time
        })
    }
}

#[async_trait]
impl AiProviderAdapter for OpenAiAdapter {
    fn provider_id(&self) -> &str {
        "openai"
    }

    fn provider_name(&self) -> &str {
        "OpenAI (GPT)"
    }

    fn is_local(&self) -> bool {
        false // Cloud provider
    }

    fn cost_per_unit(&self) -> Option<f64> {
        Some(0.00003) // ~$0.03 per 1K tokens (gpt-4)
    }

    fn avg_latency_ms(&self) -> u64 {
        1500 // Cloud API, ~1.5s average
    }

    fn quality_tier(&self) -> QualityTier {
        QualityTier::Premium
    }

    fn supports_text_generation(&self) -> bool {
        true
    }

    fn supports_image_generation(&self) -> bool {
        true // OpenAI has DALL-E
    }

    async fn is_available(&self) -> bool {
        // Check if API key is set and HTTP provider is available
        if self.api_key.is_empty() {
            return false;
        }

        // Check if we can discover HTTP provider
        discover_capability("http.request").await.is_ok()
    }

    async fn generate_text(
        &self,
        request: TextGenerationRequest,
    ) -> Result<TextGenerationResponse, PrimalError> {
        info!(
            "Generating text with OpenAI (model: {})",
            request.model.as_deref().unwrap_or(&self.default_model)
        );

        let result = self.call_openai(request).await;

        match &result {
            Ok(response) => {
                info!(
                    "✅ OpenAI generated {} chars (tokens: {:?})",
                    response.text.len(),
                    response.usage.as_ref().map(|u| u.total_tokens)
                );
            }
            Err(e) => {
                error!("❌ OpenAI generation failed: {}", e);
            }
        }

        result
    }

    async fn generate_image(
        &self,
        _request: ImageGenerationRequest,
    ) -> Result<ImageGenerationResponse, PrimalError> {
        // TODO: Implement DALL-E image generation
        // For now, return unsupported
        Err(PrimalError::OperationNotSupported(
            "OpenAI image generation not yet implemented (TODO: DALL-E)".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openai_adapter_creation() {
        // Without API key, should fail
        std::env::remove_var("OPENAI_API_KEY");
        assert!(OpenAiAdapter::new().is_err());

        // With API key, should succeed
        std::env::set_var("OPENAI_API_KEY", "test-key");
        let adapter = OpenAiAdapter::new().unwrap();
        assert_eq!(adapter.provider_id(), "openai");
        assert_eq!(adapter.provider_name(), "OpenAI (GPT)");
        assert!(!adapter.is_local());
        assert!(adapter.supports_text_generation());
        assert!(adapter.supports_image_generation());
    }
}
