// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Remote inference provider adapter.
//!
//! Forwards `inference.complete` calls to a registered remote provider (e.g.
//! neuralSpring, Ollama) via JSON-RPC over Unix domain sockets or HTTP.
//! Registered at runtime via `inference.register_provider`.
//!
//! ## Transport selection
//!
//! - **UDS (default)**: `socket` param → JSON-RPC over Unix socket.
//! - **HTTP**: `endpoint` param with `http://` or `https://` scheme →
//!   Ollama-compatible REST API (`/api/generate`, `/api/embeddings`).
//!   Used when providers expose an HTTP API instead of (or alongside) UDS.

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
    /// HTTP endpoint URL (e.g. `http://localhost:11434` for Ollama).
    /// When set, adapter uses HTTP REST instead of UDS JSON-RPC.
    pub endpoint: Option<String>,
    pub models: Vec<String>,
    pub supported_tasks: Vec<String>,
    pub supports_streaming: bool,
    pub max_context_size: usize,
    pub quality_tier: Option<String>,
    pub cost_per_unit: Option<f64>,
}

/// Default read timeout for UDS inference calls (120s — LLM responses can be slow).
const INFERENCE_UDS_READ_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(120);

/// Adapter that routes inference to a remote provider over UDS JSON-RPC or HTTP REST.
pub struct RemoteInferenceAdapter {
    config: RemoteProviderConfig,
}

impl RemoteInferenceAdapter {
    #[must_use]
    pub const fn new(config: RemoteProviderConfig) -> Self {
        Self { config }
    }

    /// Model names declared at registration time.
    pub fn model_names(&self) -> &[String] {
        &self.config.models
    }

    /// Whether this provider declared embedding support in `supported_tasks`.
    pub fn supports_embedding(&self) -> bool {
        self.config
            .supported_tasks
            .iter()
            .any(|t| t == "embedding" || t == "inference.embed" || t == "text_embedding")
    }

    /// Whether this provider was registered with an HTTP endpoint.
    fn is_http(&self) -> bool {
        self.config
            .endpoint
            .as_ref()
            .is_some_and(|e| e.starts_with("http://") || e.starts_with("https://"))
    }

    /// Forward an `inference.embed` call to the remote provider.
    pub async fn generate_embedding(
        &self,
        params: &serde_json::Value,
    ) -> Result<serde_json::Value, PrimalError> {
        if self.is_http() {
            return self.http_embed(params).await;
        }

        let Some(socket) = &self.config.socket_path else {
            return Err(PrimalError::Configuration(
                "Remote inference provider has no socket path or endpoint".into(),
            ));
        };

        debug!(
            provider = self.config.provider_id.as_str(),
            socket = socket.as_str(),
            "Forwarding inference.embed to remote spring (UDS)"
        );

        let rpc_request = json!({
            "jsonrpc": "2.0",
            "method": "inference.embed",
            "params": params,
            "id": 1,
        });

        let socket_path = std::path::Path::new(socket);
        let parsed = crate::capabilities::lifecycle::send_jsonrpc_with_timeout(
            socket_path,
            &rpc_request,
            INFERENCE_UDS_READ_TIMEOUT,
        )
        .await
        .map_err(|e| {
            warn!(
                provider = self.config.provider_id.as_str(),
                error = %e,
                "Remote embedding call failed"
            );
            PrimalError::Internal(format!("Remote embedding call to {socket} failed: {e}"))
        })?;

        parsed.get("result").cloned().ok_or_else(|| {
            let err_msg = parsed
                .get("error")
                .and_then(|e| e.get("message"))
                .and_then(|m| m.as_str())
                .unwrap_or("unknown error");
            PrimalError::Internal(format!("Remote embedding error: {err_msg}"))
        })
    }

    /// HTTP embedding via Ollama `/api/embeddings`.
    async fn http_embed(
        &self,
        params: &serde_json::Value,
    ) -> Result<serde_json::Value, PrimalError> {
        let endpoint = self.config.endpoint.as_deref().unwrap_or_default();
        let model = params
            .get("model")
            .and_then(|v| v.as_str())
            .or_else(|| self.config.models.first().map(String::as_str))
            .unwrap_or("default");
        let input = params
            .get("input")
            .or_else(|| params.get("text"))
            .cloned()
            .unwrap_or_else(|| json!(""));

        let body = json!({ "model": model, "prompt": input });
        let url = format!("{endpoint}/api/embeddings");

        let response = send_http_json(&url, &body).await.map_err(|e| {
            PrimalError::Internal(format!("HTTP embedding to {endpoint} failed: {e}"))
        })?;

        Ok(json!({ "embedding": response.get("embedding") }))
    }

    /// HTTP text generation via Ollama `/api/generate`.
    async fn http_generate(
        &self,
        request: &TextGenerationRequest,
    ) -> Result<TextGenerationResponse, PrimalError> {
        let endpoint = self.config.endpoint.as_deref().unwrap_or_default();
        let model = request
            .model
            .as_deref()
            .or_else(|| self.config.models.first().map(String::as_str))
            .unwrap_or("default");

        let body = json!({
            "model": model,
            "prompt": request.prompt,
            "stream": false,
            "options": {
                "temperature": request.temperature,
                "num_predict": request.max_tokens,
            }
        });
        let url = format!("{endpoint}/api/generate");

        debug!(
            provider = self.config.provider_id.as_str(),
            url = url.as_str(),
            model = model,
            "Forwarding inference.complete to HTTP provider"
        );

        let response = send_http_json(&url, &body).await.map_err(|e| {
            PrimalError::Internal(format!("HTTP inference to {endpoint} failed: {e}"))
        })?;

        let text = response
            .get("response")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let resp_model = response
            .get("model")
            .and_then(|v| v.as_str())
            .unwrap_or(model)
            .to_string();

        Ok(TextGenerationResponse {
            text,
            model: resp_model,
            provider_id: self.config.provider_id.clone(),
            usage: None,
            cost_usd: None,
            latency_ms: 0,
        })
    }

    /// Check HTTP provider health via transport connect.
    async fn http_is_available(&self) -> bool {
        let Some(endpoint) = &self.config.endpoint else {
            return false;
        };
        let Some(transport_ep) = parse_http_transport_endpoint(endpoint) else {
            return false;
        };
        crate::transport::connect_transport_with_timeout(
            &transport_ep,
            std::time::Duration::from_secs(2),
        )
        .await
        .is_ok()
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
        self.config.socket_path.is_some() || self.is_http()
    }

    fn cost_per_unit(&self) -> Option<f64> {
        self.config.cost_per_unit
    }

    fn avg_latency_ms(&self) -> u64 {
        if self.is_http() { 100 } else { 50 }
    }

    fn quality_tier(&self) -> QualityTier {
        match self.config.quality_tier.as_deref() {
            Some("basic") => QualityTier::Basic,
            Some("fast") => QualityTier::Fast,
            Some("high") => QualityTier::High,
            Some("premium") => QualityTier::Premium,
            _ => QualityTier::Standard,
        }
    }

    fn supports_text_generation(&self) -> bool {
        self.config.supported_tasks.is_empty()
            || self
                .config
                .supported_tasks
                .iter()
                .any(|t| t == "text_generation" || t == "chat" || t == "inference.complete")
    }

    fn supports_image_generation(&self) -> bool {
        self.config
            .supported_tasks
            .iter()
            .any(|t| t == "image_generation" || t == "inference.image")
    }

    async fn is_available(&self) -> bool {
        if self.is_http() {
            return self.http_is_available().await;
        }
        let Some(socket) = &self.config.socket_path else {
            return false;
        };
        std::path::Path::new(socket).exists()
    }

    async fn generate_text(
        &self,
        request: TextGenerationRequest,
    ) -> Result<TextGenerationResponse, PrimalError> {
        if self.is_http() {
            return self.http_generate(&request).await;
        }

        let Some(socket) = &self.config.socket_path else {
            return Err(PrimalError::Configuration(
                "Remote inference provider has no socket path or endpoint".into(),
            ));
        };

        debug!(
            provider = self.config.provider_id.as_str(),
            socket = socket.as_str(),
            "Forwarding inference.complete to remote spring (UDS)"
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
        let parsed = crate::capabilities::lifecycle::send_jsonrpc_with_timeout(
            socket_path,
            &rpc_request,
            INFERENCE_UDS_READ_TIMEOUT,
        )
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

// ---------------------------------------------------------------------------
// HTTP transport helpers (lightweight, no reqwest — pure tokio TCP + HTTP/1.1)
// ---------------------------------------------------------------------------

/// Send a JSON POST request over raw TCP and parse the JSON response.
async fn send_http_json(
    url: &str,
    body: &serde_json::Value,
) -> Result<serde_json::Value, PrimalError> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let transport_ep = parse_http_transport_endpoint(url)
        .ok_or_else(|| PrimalError::Configuration(format!("Invalid HTTP endpoint URL: {url}")))?;
    let path = parse_http_path(url);
    let host = match &transport_ep {
        crate::transport::TransportEndpoint::Tcp { host, .. } => host.as_str(),
        _ => universal_constants::network::DEFAULT_LOCALHOST,
    };

    let payload = serde_json::to_string(body)
        .map_err(|e| PrimalError::Internal(format!("JSON serialize: {e}")))?;

    let request_str = format!(
        "POST {path} HTTP/1.1\r\nHost: {host}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{payload}",
        payload.len()
    );

    let mut stream = crate::transport::connect_transport_with_timeout(
        &transport_ep,
        std::time::Duration::from_secs(5),
    )
    .await
    .map_err(|e| PrimalError::Internal(format!("HTTP connect to {transport_ep}: {e}")))?;

    stream
        .write_all(request_str.as_bytes())
        .await
        .map_err(|e| PrimalError::Internal(format!("HTTP write: {e}")))?;

    let mut response_buf = Vec::with_capacity(8192);
    tokio::time::timeout(
        std::time::Duration::from_secs(120),
        stream.read_to_end(&mut response_buf),
    )
    .await
    .map_err(|_| PrimalError::Internal("HTTP read timeout".into()))?
    .map_err(|e| PrimalError::Internal(format!("HTTP read: {e}")))?;

    let response_str = String::from_utf8_lossy(&response_buf);
    let body_start = response_str.find("\r\n\r\n").map_or(0, |i| i + 4);
    let body_str = &response_str[body_start..];

    serde_json::from_str(body_str).map_err(|e| {
        PrimalError::Internal(format!(
            "HTTP JSON parse: {e} — body: {}",
            &body_str[..body_str.len().min(200)]
        ))
    })
}

/// Parse an HTTP URL into a `TransportEndpoint::Tcp` for connect.
fn parse_http_transport_endpoint(url: &str) -> Option<crate::transport::TransportEndpoint> {
    let without_scheme = url
        .strip_prefix("http://")
        .or_else(|| url.strip_prefix("https://"))?;
    let host_port = without_scheme.split('/').next()?;
    if let Some((host, port_str)) = host_port.rsplit_once(':') {
        let port = port_str.parse::<u16>().ok()?;
        Some(crate::transport::TransportEndpoint::tcp(host, port))
    } else {
        Some(crate::transport::TransportEndpoint::tcp(host_port, 80))
    }
}

/// Extract the path component from an HTTP URL.
fn parse_http_path(url: &str) -> &str {
    let without_scheme = url
        .strip_prefix("http://")
        .or_else(|| url.strip_prefix("https://"))
        .unwrap_or(url);
    without_scheme
        .find('/')
        .map_or("/", |i| &without_scheme[i..])
}
