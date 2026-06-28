// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Canonical `inference.*` domain handlers per SEMANTIC_METHOD_NAMING_STANDARD v2.0 §7.
//!
//! Methods: `inference.complete`, `inference.embed`, `inference.models`,
//! `inference.register_provider`, `inference.unregister_provider`.
//!
//! These bridge the ecoPrimal `inference.*` wire standard to Squirrel's
//! internal `AiRouter`. Consumers call `inference.complete` and don't
//! care whether the backend is Ollama, neuralSpring, or a remote API.
//! Springs call `inference.register_provider` to register themselves as
//! inference backends (neuralSpring, healthSpring, ludoSpring, etc.),
//! and `inference.unregister_provider` on graceful shutdown.

use super::jsonrpc_server::{JsonRpcError, JsonRpcServer, error_codes};
use crate::api::ai::adapters::RemoteProviderConfig;
use serde_json::Value;
use std::time::Instant;
use tracing::{info, warn};

fn parse_provider_capabilities(caps: &Value) -> ParsedCapabilities {
    let supported_tasks: Vec<String> = if let Some(arr) = caps.as_array() {
        arr.iter()
            .filter_map(Value::as_str)
            .map(String::from)
            .collect()
    } else {
        caps.get("supported_tasks")
            .and_then(Value::as_array)
            .map(|arr| {
                arr.iter()
                    .filter_map(Value::as_str)
                    .map(String::from)
                    .collect()
            })
            .unwrap_or_default()
    };

    let models: Vec<String> = caps
        .get("models")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(Value::as_str)
                .map(String::from)
                .collect()
        })
        .unwrap_or_default();

    ParsedCapabilities {
        supported_tasks,
        models,
        supports_streaming: caps
            .get("supports_streaming")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        max_context_size: caps
            .get("max_context_size")
            .and_then(Value::as_u64)
            .unwrap_or(4096) as usize,
        quality_tier: caps
            .get("quality_tier")
            .and_then(Value::as_str)
            .map(String::from),
        cost_per_unit: caps.get("cost_per_unit").and_then(Value::as_f64),
    }
}

struct ParsedCapabilities {
    supported_tasks: Vec<String>,
    models: Vec<String>,
    supports_streaming: bool,
    max_context_size: usize,
    quality_tier: Option<String>,
    cost_per_unit: Option<f64>,
}

impl JsonRpcServer {
    /// Handle `inference.complete` — text/chat completion via the AI router.
    ///
    /// Accepts the ecoPrimal `CompleteRequest` wire format and translates
    /// to the internal `TextGenerationRequest` used by `AiRouter`.
    pub(crate) async fn handle_inference_complete(
        &self,
        params: Option<Value>,
    ) -> Result<Value, JsonRpcError> {
        let params = params.ok_or_else(|| JsonRpcError {
            code: error_codes::INVALID_PARAMS,
            message: "inference.complete requires params".into(),
            data: None,
        })?;

        let prompt = params
            .get("prompt")
            .and_then(Value::as_str)
            .map(String::from);

        let messages_prompt = params.get("messages").and_then(|msgs| {
            msgs.as_array().map(|arr| {
                arr.iter()
                    .filter_map(|m| {
                        let role = m.get("role").and_then(Value::as_str).unwrap_or("user");
                        let content = m.get("content").and_then(Value::as_str)?;
                        Some(format!("{role}: {content}"))
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            })
        });

        let effective_prompt = prompt.or(messages_prompt).ok_or_else(|| JsonRpcError {
            code: error_codes::INVALID_PARAMS,
            message: "inference.complete requires 'prompt' or 'messages'".into(),
            data: None,
        })?;

        let model = params
            .get("model")
            .and_then(Value::as_str)
            .map(String::from);
        let temperature = params
            .get("temperature")
            .and_then(Value::as_f64)
            .unwrap_or(0.7) as f32;
        let max_tokens = params
            .get("max_tokens")
            .and_then(Value::as_u64)
            .unwrap_or(1024) as u32;

        let router = self.ai_router.as_ref().ok_or_else(|| JsonRpcError {
            code: error_codes::INTERNAL_ERROR,
            message: "No inference providers configured".into(),
            data: None,
        })?;

        let ai_request = crate::api::ai::types::TextGenerationRequest {
            prompt: effective_prompt,
            system: None,
            max_tokens,
            temperature,
            model,
            constraints: vec![],
            params: std::collections::HashMap::new(),
        };

        info!("inference.complete — routing through AiRouter");
        let start = Instant::now();

        match router.generate_text(ai_request, None).await {
            Ok(ai_response) => {
                let mut resp = serde_json::json!({
                    "text": ai_response.text,
                    "model": ai_response.model,
                    "provider": ai_response.provider_id,
                });
                if let Some(usage) = ai_response.usage {
                    resp["usage"] = serde_json::json!({
                        "prompt_tokens": usage.prompt_tokens,
                        "completion_tokens": usage.completion_tokens,
                        "total_tokens": usage.total_tokens,
                    });
                }
                resp["latency_ms"] =
                    Value::Number(serde_json::Number::from(start.elapsed().as_millis() as u64));
                Ok(resp)
            }
            Err(e) => Err(JsonRpcError {
                code: error_codes::INTERNAL_ERROR,
                message: format!("inference.complete failed: {e}"),
                data: None,
            }),
        }
    }

    /// Handle `inference.embed` — embedding generation.
    ///
    /// Routes to the first registered provider that declared embedding
    /// support in `supported_tasks`. Forwards the raw params over JSON-RPC
    /// to the remote spring.
    pub(crate) async fn handle_inference_embed(
        &self,
        params: Option<Value>,
    ) -> Result<Value, JsonRpcError> {
        let params = params.ok_or_else(|| JsonRpcError {
            code: error_codes::INVALID_PARAMS,
            message: "inference.embed requires params (at minimum 'text' or 'input')".into(),
            data: None,
        })?;

        let router = self.ai_router.as_ref().ok_or_else(|| JsonRpcError {
            code: error_codes::INTERNAL_ERROR,
            message: "No inference providers configured".into(),
            data: None,
        })?;

        let provider = router.find_embedding_provider().await.ok_or_else(|| {
            JsonRpcError {
                code: error_codes::METHOD_NOT_FOUND,
                message: "inference.embed: no embedding provider registered — register a provider with supported_tasks containing 'embedding'".into(),
                data: None,
            }
        })?;

        match provider.as_ref() {
            crate::api::ai::adapters::AiProvider::RemoteInference(adapter) => {
                let result =
                    adapter
                        .generate_embedding(&params)
                        .await
                        .map_err(|e| JsonRpcError {
                            code: error_codes::INTERNAL_ERROR,
                            message: format!("inference.embed failed: {e}"),
                            data: None,
                        })?;
                Ok(result)
            }
            crate::api::ai::adapters::AiProvider::Universal(_) => Err(JsonRpcError {
                code: error_codes::METHOD_NOT_FOUND,
                message: "inference.embed: selected provider does not support remote embedding"
                    .into(),
                data: None,
            }),
            #[cfg(test)]
            _ => Err(JsonRpcError {
                code: error_codes::METHOD_NOT_FOUND,
                message: "inference.embed: selected provider does not support remote embedding"
                    .into(),
                data: None,
            }),
        }
    }

    /// Handle `inference.models` — list available models/providers.
    ///
    /// Returns provider-level info enriched with model names declared during
    /// `inference.register_provider` and accurate embedding support flags.
    pub(crate) async fn handle_inference_models(
        &self,
        _params: Option<Value>,
    ) -> Result<Value, JsonRpcError> {
        let router = self.ai_router.as_ref().ok_or_else(|| JsonRpcError {
            code: error_codes::INTERNAL_ERROR,
            message: "No inference providers configured".into(),
            data: None,
        })?;

        let detailed = router.list_providers_detailed().await;
        let models: Vec<Value> = detailed
            .iter()
            .map(|(p, model_names, supports_embedding)| {
                let supports_completion = p
                    .capabilities
                    .iter()
                    .any(|c| c.contains("text") || c.contains("generation"));
                serde_json::json!({
                    "id": p.provider_id,
                    "name": p.provider_name,
                    "supports_completion": supports_completion,
                    "supports_embedding": supports_embedding,
                    "available_models": model_names,
                })
            })
            .collect();

        Ok(serde_json::json!({ "models": models }))
    }

    /// Handle `inference.register_provider` — register a spring as an inference backend.
    ///
    /// Wire format (JSON-RPC params):
    /// ```json
    /// {
    ///   "provider_id": "neuralSpring-node-abc123",
    ///   "socket": "/run/user/1000/biomeos/neuralSpring.sock",
    ///   "endpoint": "http://localhost:11434",
    ///   "capabilities": {
    ///     "supported_tasks": ["text_generation", "embedding", "chat"],
    ///     "models": ["llama3", "mistral-7b"],
    ///     "supports_streaming": true,
    ///     "max_context_size": 8192
    ///   }
    /// }
    /// ```
    ///
    /// Either `socket` (UDS) or `endpoint` (HTTP URL) must be provided.
    /// When both are given, `socket` takes priority for JSON-RPC springs;
    /// `endpoint` is used for HTTP-native providers (e.g. Ollama).
    ///
    /// Registers the caller's capabilities with `AiRouter` so Squirrel can
    /// route `inference.complete` / `inference.embed` to this provider.
    pub(crate) async fn handle_inference_register_provider(
        &self,
        params: Option<Value>,
    ) -> Result<Value, JsonRpcError> {
        let params = params.ok_or_else(|| JsonRpcError {
            code: error_codes::INVALID_PARAMS,
            message: "inference.register_provider requires params".into(),
            data: None,
        })?;

        let provider_id = params
            .get("provider_id")
            .and_then(Value::as_str)
            .ok_or_else(|| JsonRpcError {
                code: error_codes::INVALID_PARAMS,
                message: "inference.register_provider requires 'provider_id' (string)".into(),
                data: None,
            })?;

        let trimmed_id = provider_id.trim();
        if trimmed_id.is_empty() || trimmed_id.len() > 256 {
            return Err(JsonRpcError {
                code: error_codes::INVALID_PARAMS,
                message: "provider_id must be 1–256 non-whitespace characters".into(),
                data: None,
            });
        }

        let raw_socket = params
            .get("socket")
            .and_then(Value::as_str)
            .map(String::from);

        let raw_endpoint = params
            .get("endpoint")
            .and_then(Value::as_str)
            .map(String::from);

        // Auto-promote HTTP URLs from `socket` to `endpoint`.
        // Providers (e.g. neuralSpring for Ollama) may pass
        // "socket": "http://localhost:11434" — treating that as a UDS path
        // creates a broken provider. Detect the scheme and route correctly.
        let is_http = |s: &str| s.starts_with("http://") || s.starts_with("https://");

        let (socket, endpoint) = match (&raw_socket, &raw_endpoint) {
            (Some(s), None) if is_http(s) => {
                info!(
                    provider = trimmed_id,
                    url = s.as_str(),
                    "Auto-promoting HTTP URL from 'socket' to 'endpoint'"
                );
                (None, Some(s.clone()))
            }
            _ => (raw_socket, raw_endpoint),
        };

        if socket.is_none() && endpoint.is_none() {
            warn!(
                provider = trimmed_id,
                "inference.register_provider — no socket or endpoint; provider will be unavailable for routing"
            );
        }

        let caps = params.get("capabilities").cloned().unwrap_or(Value::Null);

        let router = self.ai_router.as_ref().ok_or_else(|| JsonRpcError {
            code: error_codes::INTERNAL_ERROR,
            message: "AI router not initialized — cannot register providers".into(),
            data: None,
        })?;

        let parsed = parse_provider_capabilities(&caps);

        let config = RemoteProviderConfig {
            provider_id: trimmed_id.to_string(),
            socket_path: socket.clone(),
            endpoint: endpoint.clone(),
            models: parsed.models,
            supported_tasks: parsed.supported_tasks,
            supports_streaming: parsed.supports_streaming,
            max_context_size: parsed.max_context_size,
            quality_tier: parsed.quality_tier,
            cost_per_unit: parsed.cost_per_unit,
        };

        router.register_remote_provider(config.clone()).await;

        let transport = if endpoint.is_some() { "HTTP" } else { "UDS" };
        info!(
            provider = trimmed_id,
            socket = ?socket,
            endpoint = ?endpoint,
            transport = transport,
            model_count = config.models.len(),
            task_count = config.supported_tasks.len(),
            "inference.register_provider — registered"
        );

        Ok(serde_json::json!({
            "registered": true,
            "provider_id": trimmed_id,
            "models": config.models,
            "supported_tasks": config.supported_tasks,
        }))
    }

    /// Handle `inference.unregister_provider` — remove a spring from the provider list.
    ///
    /// Called by springs during graceful shutdown. Params: `{ "provider_id": "..." }`.
    pub(crate) async fn handle_inference_unregister_provider(
        &self,
        params: Option<Value>,
    ) -> Result<Value, JsonRpcError> {
        let params = params.ok_or_else(|| JsonRpcError {
            code: error_codes::INVALID_PARAMS,
            message: "inference.unregister_provider requires params".into(),
            data: None,
        })?;

        let provider_id = params
            .get("provider_id")
            .and_then(Value::as_str)
            .ok_or_else(|| JsonRpcError {
                code: error_codes::INVALID_PARAMS,
                message: "inference.unregister_provider requires 'provider_id' (string)".into(),
                data: None,
            })?;

        let router = self.ai_router.as_ref().ok_or_else(|| JsonRpcError {
            code: error_codes::INTERNAL_ERROR,
            message: "AI router not initialized".into(),
            data: None,
        })?;

        let removed = router.unregister_remote_provider(provider_id).await;

        Ok(serde_json::json!({
            "unregistered": removed,
            "provider_id": provider_id,
        }))
    }
}
