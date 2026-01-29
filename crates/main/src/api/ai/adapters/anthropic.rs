//! Anthropic AI Provider Adapter with Capability-Based HTTP Delegation
//!
//! ⚠️ **DEPRECATED**: This vendor-specific adapter is deprecated.
//!
//! **Migration Path**:
//! - The router now uses universal capability discovery
//! - No code changes needed - providers are auto-discovered
//! - See `crates/main/src/api/ai/universal.rs` for the new interface
//!
//! **For direct use**:
//! ```rust,ignore
//! // OLD (deprecated):
//! let adapter = AnthropicAdapter::new()?;
//!
//! // NEW (universal):
//! let providers = discover_ai_providers().await;
//! ```
//!
//! This adapter will be removed in a future release.

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

/// Anthropic-specific request format
#[derive(Debug, Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<AnthropicMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

/// Anthropic API response
#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    id: String,
    model: String,
    content: Vec<AnthropicContent>,
    usage: AnthropicUsage,
}

#[derive(Debug, Deserialize)]
struct AnthropicContent {
    #[serde(rename = "type")]
    content_type: String,
    text: String,
}

#[derive(Debug, Deserialize)]
struct AnthropicUsage {
    input_tokens: u32,
    output_tokens: u32,
}

/// Anthropic AI Provider Adapter
///
/// Uses capability discovery to find HTTP provider (TRUE PRIMAL!)
///
/// ⚠️ **DEPRECATED**: Use universal capability discovery instead.
/// See module documentation for migration guide.
#[deprecated(
    since = "0.2.0",
    note = "Use universal capability discovery (discover_ai_providers) instead. \
            This vendor-specific adapter will be removed in 0.3.0."
)]
pub struct AnthropicAdapter {
    api_key: String,
    default_model: String,
}

impl AnthropicAdapter {
    /// Create new Anthropic adapter
    ///
    /// Reads API key from ANTHROPIC_API_KEY environment variable
    ///
    /// ⚠️ **DEPRECATED**: Use `discover_ai_providers()` instead.
    #[deprecated(
        since = "0.2.0",
        note = "Use discover_ai_providers() for automatic provider discovery"
    )]
    pub fn new() -> Result<Self, PrimalError> {
        let api_key = std::env::var("ANTHROPIC_API_KEY")
            .map_err(|_| PrimalError::ConfigError("ANTHROPIC_API_KEY not set".to_string()))?;

        Ok(Self {
            api_key,
            default_model: "claude-3-opus-20240229".to_string(),
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

    /// Call Anthropic API
    async fn call_anthropic(
        &self,
        request: TextGenerationRequest,
    ) -> Result<TextGenerationResponse, PrimalError> {
        // Build Anthropic-specific request
        let anthropic_request = AnthropicRequest {
            model: request.model.unwrap_or_else(|| self.default_model.clone()),
            max_tokens: request.max_tokens,
            messages: vec![AnthropicMessage {
                role: "user".to_string(),
                content: request.prompt.clone(),
            }],
            temperature: Some(request.temperature),
            system: None,
        };

        // Build headers with API key
        let mut headers = HashMap::new();
        headers.insert("x-api-key".to_string(), self.api_key.clone());
        headers.insert("anthropic-version".to_string(), "2023-06-01".to_string());
        headers.insert("content-type".to_string(), "application/json".to_string());

        // Delegate HTTP to discovered provider (TRUE PRIMAL!)
        let response_json = self
            .delegate_http(
                "POST",
                "https://api.anthropic.com/v1/messages",
                headers,
                serde_json::to_value(&anthropic_request)?,
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

        // Parse Anthropic response
        let anthropic_response: AnthropicResponse = serde_json::from_value(http_response)?;

        // Convert to universal format
        Ok(TextGenerationResponse {
            text: anthropic_response.content[0].text.clone(),
            provider_id: "anthropic".to_string(),
            model: anthropic_response.model,
            usage: Some(crate::api::ai::types::TokenUsage {
                prompt_tokens: anthropic_response.usage.input_tokens,
                completion_tokens: anthropic_response.usage.output_tokens,
                total_tokens: anthropic_response.usage.input_tokens
                    + anthropic_response.usage.output_tokens,
            }),
            cost_usd: None, // TODO: Calculate cost based on usage
            latency_ms: 0,  // TODO: Track request time
        })
    }
}

#[async_trait]
impl AiProviderAdapter for AnthropicAdapter {
    fn provider_id(&self) -> &str {
        "anthropic"
    }

    fn provider_name(&self) -> &str {
        "Anthropic (Claude)"
    }

    fn is_local(&self) -> bool {
        false // Cloud provider
    }

    fn cost_per_unit(&self) -> Option<f64> {
        Some(0.000015) // ~$0.015 per 1K tokens (opus)
    }

    fn avg_latency_ms(&self) -> u64 {
        2000 // Cloud API, ~2s average
    }

    fn quality_tier(&self) -> QualityTier {
        QualityTier::Premium
    }

    fn supports_text_generation(&self) -> bool {
        true
    }

    fn supports_image_generation(&self) -> bool {
        false // Claude doesn't generate images (yet)
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
            "Generating text with Anthropic (model: {})",
            request.model.as_deref().unwrap_or(&self.default_model)
        );

        let result = self.call_anthropic(request).await;

        match &result {
            Ok(response) => {
                info!(
                    "✅ Anthropic generated {} chars (tokens: {:?})",
                    response.text.len(),
                    response.usage.as_ref().map(|u| u.total_tokens)
                );
            }
            Err(e) => {
                error!("❌ Anthropic generation failed: {}", e);
            }
        }

        result
    }

    async fn generate_image(
        &self,
        _request: ImageGenerationRequest,
    ) -> Result<ImageGenerationResponse, PrimalError> {
        Err(PrimalError::OperationNotSupported(
            "Anthropic does not support image generation".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anthropic_adapter_creation() {
        // Without API key, should fail
        std::env::remove_var("ANTHROPIC_API_KEY");
        assert!(AnthropicAdapter::new().is_err());

        // With API key, should succeed
        std::env::set_var("ANTHROPIC_API_KEY", "test-key");
        let adapter = AnthropicAdapter::new().unwrap();
        assert_eq!(adapter.provider_id(), "anthropic");
        assert_eq!(adapter.provider_name(), "Anthropic (Claude)");
        assert!(!adapter.is_local());
        assert!(adapter.supports_text_generation());
        assert!(!adapter.supports_image_generation());
    }
}
