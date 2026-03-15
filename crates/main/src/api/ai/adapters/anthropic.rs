// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

// Allow deprecated items for backward compatibility
#![allow(deprecated)]

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

        // NUCLEUS FIX (Feb 3, 2026): Use haiku as default (available on most API keys)
        // Can be overridden per-request via the model field
        let default_model = std::env::var("ANTHROPIC_DEFAULT_MODEL")
            .unwrap_or_else(|_| "claude-3-haiku-20240307".to_string());

        Ok(Self {
            api_key,
            default_model,
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
        // Songbird expects body as STRING, not object
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

        // Track request start time for latency calculation
        let start_time = std::time::Instant::now();

        // Delegate HTTP to discovered provider (TRUE PRIMAL!)
        let response_json = self
            .delegate_http(
                "POST",
                "https://api.anthropic.com/v1/messages",
                headers,
                serde_json::to_value(&anthropic_request)?,
            )
            .await?;

        // Calculate latency
        let latency_ms = start_time.elapsed().as_millis() as u64;

        // Parse HTTP response - body comes as string from Songbird
        let body_value = response_json
            .get("body")
            .cloned()
            .ok_or_else(|| PrimalError::ParsingError("No body in HTTP response".to_string()))?;

        // Songbird returns body as string, parse it
        let http_response: serde_json::Value = match body_value {
            serde_json::Value::String(s) => serde_json::from_str(&s).map_err(|e| {
                PrimalError::ParsingError(format!("Failed to parse body JSON: {}", e))
            })?,
            other => other, // Already parsed (for future compatibility)
        };

        // NUCLEUS FIX (Feb 3, 2026): Check for Anthropic error response before parsing
        // Anthropic errors have format: {"error": {"type": "...", "message": "..."}, ...}
        if let Some(error_obj) = http_response.get("error") {
            let error_type = error_obj
                .get("type")
                .and_then(|t| t.as_str())
                .unwrap_or("unknown");
            let error_msg = error_obj
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown error");
            return Err(PrimalError::NetworkError(format!(
                "Anthropic API error ({}): {}",
                error_type, error_msg
            )));
        }

        // Parse Anthropic response
        let anthropic_response: AnthropicResponse = serde_json::from_value(http_response)?;

        // Calculate cost based on token usage (approximate pricing per 1M tokens)
        // Cost tiers are configurable; defaults provide reasonable estimates.
        // NOTE(config): Cost tables could be loaded from squirrel.toml / env at startup
        let model = &anthropic_response.model;
        let (input_cost_per_1m, output_cost_per_1m) = Self::estimate_cost_per_1m_tokens(model);

        let cost_usd = Some(
            (anthropic_response.usage.input_tokens as f64 / 1_000_000.0 * input_cost_per_1m)
                + (anthropic_response.usage.output_tokens as f64 / 1_000_000.0
                    * output_cost_per_1m),
        );

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
            cost_usd,
            latency_ms,
        })
    }

    /// Estimate cost per 1M tokens for a given model identifier.
    ///
    /// Uses model family pattern matching with reasonable defaults.
    /// These are approximate and should eventually be loaded from
    /// configuration at startup.
    fn estimate_cost_per_1m_tokens(model: &str) -> (f64, f64) {
        let model_lower = model.to_lowercase();
        if model_lower.contains("opus") {
            (15.0, 75.0)
        } else if model_lower.contains("sonnet") {
            (3.0, 15.0)
        } else {
            // Haiku / default — conservative estimate
            (0.25, 1.25)
        }
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

    #[test]
    fn test_cost_estimation_opus() {
        let (input, output) = AnthropicAdapter::estimate_cost_per_1m_tokens("claude-3-opus");
        assert!((input - 15.0).abs() < f64::EPSILON);
        assert!((output - 75.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_cost_estimation_opus_case_insensitive() {
        let (input, output) =
            AnthropicAdapter::estimate_cost_per_1m_tokens("Claude-3-Opus-20240229");
        assert!((input - 15.0).abs() < f64::EPSILON);
        assert!((output - 75.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_cost_estimation_sonnet() {
        let (input, output) = AnthropicAdapter::estimate_cost_per_1m_tokens("claude-3-sonnet");
        assert!((input - 3.0).abs() < f64::EPSILON);
        assert!((output - 15.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_cost_estimation_haiku_default() {
        let (input, output) = AnthropicAdapter::estimate_cost_per_1m_tokens("claude-3-haiku");
        assert!((input - 0.25).abs() < f64::EPSILON);
        assert!((output - 1.25).abs() < f64::EPSILON);
    }

    #[test]
    fn test_cost_estimation_unknown_model() {
        let (input, output) = AnthropicAdapter::estimate_cost_per_1m_tokens("unknown-model-xyz");
        // Should use conservative default (Haiku pricing)
        assert!((input - 0.25).abs() < f64::EPSILON);
        assert!((output - 1.25).abs() < f64::EPSILON);
    }

    #[test]
    fn test_adapter_quality_tier() {
        std::env::set_var("ANTHROPIC_API_KEY", "test-key-qt");
        let adapter = AnthropicAdapter::new().unwrap();
        assert_eq!(adapter.quality_tier(), QualityTier::Premium);
        assert_eq!(adapter.avg_latency_ms(), 2000);
        assert!(adapter.cost_per_unit().is_some());
    }

    #[test]
    fn test_default_model() {
        std::env::remove_var("ANTHROPIC_DEFAULT_MODEL");
        std::env::set_var("ANTHROPIC_API_KEY", "test-key-dm");
        let adapter = AnthropicAdapter::new().unwrap();
        assert_eq!(adapter.default_model, "claude-3-haiku-20240307");
    }

    #[test]
    fn test_custom_default_model() {
        std::env::set_var("ANTHROPIC_API_KEY", "test-key-cdm");
        std::env::set_var("ANTHROPIC_DEFAULT_MODEL", "claude-3-opus");
        let adapter = AnthropicAdapter::new().unwrap();
        assert_eq!(adapter.default_model, "claude-3-opus");
        std::env::remove_var("ANTHROPIC_DEFAULT_MODEL");
    }
}
