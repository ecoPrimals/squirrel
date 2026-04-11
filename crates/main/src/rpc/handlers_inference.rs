// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Canonical `inference.*` domain handlers per SEMANTIC_METHOD_NAMING_STANDARD v2.0 §7.
//!
//! Methods: `inference.complete`, `inference.embed`, `inference.models`,
//! `inference.register_provider`.
//!
//! These bridge the ecoPrimal `inference.*` wire standard to Squirrel's
//! internal `AiRouter`. Consumers call `inference.complete` and don't
//! care whether the backend is Ollama, neuralSpring, or a remote API.
//! Springs call `inference.register_provider` to register themselves as
//! inference backends (neuralSpring, healthSpring, ludoSpring, etc.).

use super::jsonrpc_server::{JsonRpcError, JsonRpcServer, error_codes};
use serde_json::Value;
use std::time::Instant;
use tracing::info;

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
    /// Embedding is not yet wired through `AiRouter` (the router only
    /// supports text and image generation). Returns method-not-found
    /// until an embedding provider is registered.
    pub(crate) async fn handle_inference_embed(
        &self,
        _params: Option<Value>,
    ) -> Result<Value, JsonRpcError> {
        Err(JsonRpcError {
            code: error_codes::METHOD_NOT_FOUND,
            message: "inference.embed: no embedding provider registered yet".into(),
            data: None,
        })
    }

    /// Handle `inference.models` — list available models.
    ///
    /// Bridges to `AiRouter::list_providers()` and collects available
    /// model metadata.
    pub(crate) async fn handle_inference_models(
        &self,
        _params: Option<Value>,
    ) -> Result<Value, JsonRpcError> {
        let router = self.ai_router.as_ref().ok_or_else(|| JsonRpcError {
            code: error_codes::INTERNAL_ERROR,
            message: "No inference providers configured".into(),
            data: None,
        })?;

        let providers = router.list_providers().await;
        let models: Vec<Value> = providers
            .iter()
            .map(|p| {
                let supports_completion = p
                    .capabilities
                    .iter()
                    .any(|c| c.contains("text") || c.contains("generation"));
                serde_json::json!({
                    "id": p.provider_id,
                    "name": p.provider_name,
                    "supports_completion": supports_completion,
                    "supports_embedding": false,
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
    ///   "capabilities": {
    ///     "supported_tasks": ["text_generation", "embedding", "chat"],
    ///     "models": ["llama3", "mistral-7b"],
    ///     "supports_streaming": true,
    ///     "max_context_size": 8192
    ///   }
    /// }
    /// ```
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

        let socket = params
            .get("socket")
            .and_then(Value::as_str)
            .map(String::from);

        let caps = params.get("capabilities").cloned().unwrap_or(Value::Null);

        let router = self.ai_router.as_ref().ok_or_else(|| JsonRpcError {
            code: error_codes::INTERNAL_ERROR,
            message: "AI router not initialized — cannot register providers".into(),
            data: None,
        })?;

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

        let supports_streaming = caps
            .get("supports_streaming")
            .and_then(Value::as_bool)
            .unwrap_or(false);

        let max_context_size = caps
            .get("max_context_size")
            .and_then(Value::as_u64)
            .unwrap_or(4096) as usize;

        use crate::api::ai::adapters::RemoteProviderConfig;

        let config = RemoteProviderConfig {
            provider_id: provider_id.to_string(),
            socket_path: socket.clone(),
            models: models.clone(),
            supports_streaming,
            max_context_size,
        };

        router.register_remote_provider(config).await;

        info!(
            provider = provider_id,
            socket = ?socket,
            model_count = models.len(),
            "inference.register_provider — registered remote inference backend"
        );

        Ok(serde_json::json!({
            "registered": true,
            "provider_id": provider_id,
            "models": models,
        }))
    }
}
