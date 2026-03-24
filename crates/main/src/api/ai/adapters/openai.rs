// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

// Allow deprecated items for backward compatibility
#![expect(deprecated, reason = "Deprecated adapter; removal planned")]

//! OpenAI AI Provider Adapter with Capability-Based HTTP Delegation
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
//! let adapter = OpenAiAdapter::new()?;
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
use universal_constants::ai_providers;
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
#[expect(dead_code, reason = "Fields used by serde deserialization")]
struct OpenAiResponse {
    id: String,
    model: String,
    choices: Vec<OpenAiChoice>,
    usage: OpenAiUsage,
}

#[derive(Debug, Deserialize)]
#[expect(dead_code, reason = "Fields used by serde deserialization")]
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
///
/// ⚠️ **DEPRECATED**: Use universal capability discovery instead.
/// See module documentation for migration guide.
#[deprecated(
    since = "0.2.0",
    note = "Use universal capability discovery (discover_ai_providers) instead. \
            This vendor-specific adapter will be removed in 0.3.0."
)]
pub struct OpenAiAdapter {
    api_key: String,
    default_model: String,
}

impl OpenAiAdapter {
    /// Create new OpenAI adapter
    ///
    /// Reads API key from OPENAI_API_KEY environment variable
    ///
    /// ⚠️ **DEPRECATED**: Use `discover_ai_providers()` instead.
    #[deprecated(
        since = "0.2.0",
        note = "Use discover_ai_providers() for automatic provider discovery"
    )]
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
            .map_err(|e| PrimalError::NetworkError(format!("No HTTP provider found: {e}")))?;

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

        universal_patterns::extract_rpc_result(&rpc_response).map_err(|rpc_err| {
            PrimalError::NetworkError(format!("HTTP delegation error: {rpc_err}"))
        })
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

        // Track request start time for latency calculation
        let start_time = std::time::Instant::now();

        // Delegate HTTP to discovered provider (TRUE PRIMAL!)
        let response_json = self
            .delegate_http(
                "POST",
                &ai_providers::openai_chat_completions_url(),
                headers,
                serde_json::to_value(&openai_request)?,
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
                PrimalError::ParsingError(format!("Failed to parse body JSON: {e}"))
            })?,
            other => other, // Already parsed (for future compatibility)
        };

        // Parse OpenAI response
        let openai_response: OpenAiResponse = serde_json::from_value(http_response)?;

        // Calculate cost based on token usage (approximate pricing per 1K tokens)
        // Cost tiers are configurable; defaults provide reasonable estimates.
        // NOTE(config): Cost tables could be loaded from squirrel.toml / env at startup
        let model = &openai_response.model;
        let (prompt_cost_per_1k, completion_cost_per_1k) = Self::estimate_cost_per_1k_tokens(model);

        let cost_usd = Some(
            (f64::from(openai_response.usage.prompt_tokens) / 1000.0).mul_add(
                prompt_cost_per_1k,
                f64::from(openai_response.usage.completion_tokens) / 1000.0
                    * completion_cost_per_1k,
            ),
        );

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
            cost_usd,
            latency_ms,
        })
    }

    /// Estimate cost per 1K tokens for a given model identifier.
    ///
    /// Uses model name pattern matching with reasonable defaults.
    /// These are approximate and should eventually be loaded from
    /// configuration at startup.
    fn estimate_cost_per_1k_tokens(model: &str) -> (f64, f64) {
        // Match on model family (not exact hardcoded model name)
        let model_lower = model.to_lowercase();
        if model_lower.contains("gpt-4") {
            (0.01, 0.03)
        } else if model_lower.contains("gpt-3") {
            (0.0005, 0.0015)
        } else {
            // Conservative default for unknown models
            (0.01, 0.03)
        }
    }
}

#[async_trait]
impl AiProviderAdapter for OpenAiAdapter {
    fn provider_id(&self) -> &'static str {
        "openai"
    }

    fn provider_name(&self) -> &'static str {
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
        // FUTURE: [Feature] Implement DALL-E image generation
        // Tracking: Planned for v0.2.0 - image generation support
        // For now, return unsupported
        Err(PrimalError::OperationNotSupported(
            "OpenAI image generation not yet implemented (FUTURE: DALL-E)".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openai_adapter_creation() {
        let result = temp_env::with_var("OPENAI_API_KEY", None::<&str>, OpenAiAdapter::new);
        assert!(result.is_err());

        let adapter = temp_env::with_var("OPENAI_API_KEY", Some("test-key"), OpenAiAdapter::new)
            .expect("should succeed");
        assert_eq!(adapter.provider_id(), "openai");
        assert_eq!(adapter.provider_name(), "OpenAI (GPT)");
        assert!(!adapter.is_local());
        assert!(adapter.supports_text_generation());
        assert!(adapter.supports_image_generation());
    }

    #[test]
    fn test_cost_estimation_gpt4() {
        let (prompt, completion) = OpenAiAdapter::estimate_cost_per_1k_tokens("gpt-4");
        assert!((prompt - 0.01).abs() < f64::EPSILON);
        assert!((completion - 0.03).abs() < f64::EPSILON);
    }

    #[test]
    fn test_cost_estimation_gpt4_variants() {
        // Case insensitive matching
        let (prompt, completion) = OpenAiAdapter::estimate_cost_per_1k_tokens("GPT-4-turbo");
        assert!((prompt - 0.01).abs() < f64::EPSILON);
        assert!((completion - 0.03).abs() < f64::EPSILON);

        let (prompt, completion) = OpenAiAdapter::estimate_cost_per_1k_tokens("gpt-4o");
        assert!((prompt - 0.01).abs() < f64::EPSILON);
        assert!((completion - 0.03).abs() < f64::EPSILON);
    }

    #[test]
    fn test_cost_estimation_gpt3() {
        let (prompt, completion) = OpenAiAdapter::estimate_cost_per_1k_tokens("gpt-3.5-turbo");
        assert!((prompt - 0.0005).abs() < f64::EPSILON);
        assert!((completion - 0.0015).abs() < f64::EPSILON);
    }

    #[test]
    fn test_cost_estimation_unknown_model() {
        let (prompt, completion) = OpenAiAdapter::estimate_cost_per_1k_tokens("unknown-model-xyz");
        // Should use conservative default
        assert!((prompt - 0.01).abs() < f64::EPSILON);
        assert!((completion - 0.03).abs() < f64::EPSILON);
    }

    #[test]
    fn test_adapter_quality_tier() {
        let adapter = temp_env::with_var("OPENAI_API_KEY", Some("test-key-qt"), || {
            OpenAiAdapter::new()
        })
        .expect("should succeed");
        assert_eq!(adapter.quality_tier(), QualityTier::Premium);
        assert_eq!(adapter.avg_latency_ms(), 1500);
        assert!(adapter.cost_per_unit().is_some());
    }

    #[test]
    fn test_default_model() {
        let adapter = temp_env::with_var("OPENAI_API_KEY", Some("test-key-dm"), || {
            OpenAiAdapter::new()
        })
        .expect("should succeed");
        assert_eq!(adapter.default_model, "gpt-4");
    }
}
