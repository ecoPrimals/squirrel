// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Remote inference provider adapter.
//!
//! Forwards `inference.complete` calls to a registered remote spring (e.g.
//! neuralSpring) over Unix domain sockets via JSON-RPC.  Registered at
//! runtime via `inference.register_provider`.

use super::{AiProviderAdapter, QualityTier};
use crate::api::ai::types::{
    ImageGenerationRequest, ImageGenerationResponse, TextGenerationRequest, TextGenerationResponse,
};
use crate::error::PrimalError;
use serde_json::json;
use tracing::{debug, warn};

/// Metadata supplied by a remote spring during `inference.register_provider`.
#[derive(Debug, Clone)]
pub struct RemoteProviderConfig {
    pub provider_id: String,
    pub socket_path: Option<String>,
    pub models: Vec<String>,
    pub supports_streaming: bool,
    pub max_context_size: usize,
}

/// Adapter that routes inference to a remote spring over UDS JSON-RPC.
pub struct RemoteInferenceAdapter {
    config: RemoteProviderConfig,
}

impl RemoteInferenceAdapter {
    #[must_use]
    pub const fn new(config: RemoteProviderConfig) -> Self {
        Self { config }
    }
}

impl AiProviderAdapter for RemoteInferenceAdapter {
    fn provider_id(&self) -> &str {
        &self.config.provider_id
    }

    fn provider_name(&self) -> &str {
        &self.config.provider_id
    }

    fn is_local(&self) -> bool {
        self.config.socket_path.is_some()
    }

    fn cost_per_unit(&self) -> Option<f64> {
        Some(0.0)
    }

    fn avg_latency_ms(&self) -> u64 {
        50
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

    async fn is_available(&self) -> bool {
        let Some(socket) = &self.config.socket_path else {
            return false;
        };
        std::path::Path::new(socket).exists()
    }

    async fn generate_text(
        &self,
        request: TextGenerationRequest,
    ) -> Result<TextGenerationResponse, PrimalError> {
        let Some(socket) = &self.config.socket_path else {
            return Err(PrimalError::Configuration(
                "Remote inference provider has no socket path".into(),
            ));
        };

        debug!(
            provider = self.config.provider_id.as_str(),
            socket = socket.as_str(),
            "Forwarding inference.complete to remote spring"
        );

        let rpc_request = json!({
            "jsonrpc": "2.0",
            "method": "inference.complete",
            "params": {
                "prompt": request.prompt,
                "model": request.model,
                "temperature": request.temperature,
                "max_tokens": request.max_tokens,
            },
            "id": 1,
        });

        let socket_path = std::path::Path::new(socket);
        let parsed = crate::capabilities::lifecycle::send_jsonrpc_public(socket_path, &rpc_request)
            .await
            .map_err(|e| {
                warn!(
                    provider = self.config.provider_id.as_str(),
                    error = %e,
                    "Remote inference call failed"
                );
                PrimalError::Internal(format!("Remote inference call to {socket} failed: {e}"))
            })?;

        let result = parsed.get("result").ok_or_else(|| {
            let err_msg = parsed
                .get("error")
                .and_then(|e| e.get("message"))
                .and_then(|m| m.as_str())
                .unwrap_or("unknown error");
            PrimalError::Internal(format!("Remote inference error: {err_msg}"))
        })?;

        let text = result
            .get("text")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let model = result
            .get("model")
            .and_then(|v| v.as_str())
            .unwrap_or(&self.config.provider_id)
            .to_string();

        Ok(TextGenerationResponse {
            text,
            model,
            provider_id: self.config.provider_id.clone(),
            usage: None,
            cost_usd: None,
            latency_ms: 0,
        })
    }

    async fn generate_image(
        &self,
        _request: ImageGenerationRequest,
    ) -> Result<ImageGenerationResponse, PrimalError> {
        Err(PrimalError::Configuration(
            "Remote inference provider does not support image generation".into(),
        ))
    }
}
