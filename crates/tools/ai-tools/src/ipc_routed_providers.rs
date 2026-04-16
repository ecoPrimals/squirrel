// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Vendor HTTP access routed through the ecosystem IPC client ([`crate::neural_http::NeuralHttpClient`]).
//! This primal does not open outbound TLS to cloud APIs directly; it delegates via `neural_api.proxy_http`.

use std::fmt;
use std::sync::Arc;

use serde_json::{Value, json};

use crate::AiClientImpl;
use crate::common::capability::{AICapabilities, ModelType, TaskType};
use crate::common::client::AIClient;
use crate::common::types::{ChatRequest, ChatResponse, ChatResponseStream, MessageRole};
use crate::error::{Error, Result};
use crate::neural_http::{HttpResponse, NeuralHttpClient};

#[path = "ipc_routed_parsers.rs"]
mod ipc_routed_parsers;

use ipc_routed_parsers::{
    parse_anthropic_chat_response, parse_gemini_chat_response, parse_openai_chat_response,
};

/// HTTP surface used by [`IpcRoutedVendorClient`] (implemented by [`NeuralHttpClient`]; mockable in tests).
pub trait IpcHttpDelegate: Send + Sync {
    fn post_json(
        &self,
        url: &str,
        headers: Vec<(String, String)>,
        body: &str,
    ) -> impl std::future::Future<Output = anyhow::Result<HttpResponse>> + Send;

    fn get(
        &self,
        url: &str,
        headers: Vec<(String, String)>,
    ) -> impl std::future::Future<Output = anyhow::Result<HttpResponse>> + Send;
}

#[expect(
    clippy::use_self,
    reason = "Call inherent NeuralHttpClient::{post_json,get}; Self would recurse with trait methods"
)]
impl IpcHttpDelegate for NeuralHttpClient {
    async fn post_json(
        &self,
        url: &str,
        headers: Vec<(String, String)>,
        body: &str,
    ) -> anyhow::Result<HttpResponse> {
        // Call inherent methods (not the trait method being defined).
        NeuralHttpClient::post_json(self, url, headers, body).await
    }

    async fn get(&self, url: &str, headers: Vec<(String, String)>) -> anyhow::Result<HttpResponse> {
        NeuralHttpClient::get(self, url, headers).await
    }
}

/// Which vendor API shape to use when building requests (all traffic still goes through IPC).
#[derive(Debug, Clone, Copy)]
pub enum VendorKind {
    OpenAI,
    Anthropic,
    Gemini,
}

impl VendorKind {
    /// Base URL for this vendor's API, resolved from environment or well-known default.
    ///
    /// Environment overrides (capability-based, no hardcoded assumptions):
    /// - `OPENAI_API_BASE` / `OPENAI_BASE_URL`
    /// - `ANTHROPIC_API_BASE` / `ANTHROPIC_BASE_URL`
    /// - `GEMINI_API_BASE` / `GEMINI_BASE_URL`
    fn base_url(self) -> String {
        match self {
            Self::OpenAI => std::env::var("OPENAI_API_BASE")
                .or_else(|_| std::env::var("OPENAI_BASE_URL"))
                .unwrap_or_else(|_| "https://api.openai.com".to_string()),
            Self::Anthropic => std::env::var("ANTHROPIC_API_BASE")
                .or_else(|_| std::env::var("ANTHROPIC_BASE_URL"))
                .unwrap_or_else(|_| "https://api.anthropic.com".to_string()),
            Self::Gemini => std::env::var("GEMINI_API_BASE")
                .or_else(|_| std::env::var("GEMINI_BASE_URL"))
                .unwrap_or_else(|_| "https://generativelanguage.googleapis.com".to_string()),
        }
    }

    /// Chat completions endpoint for this vendor.
    fn chat_endpoint(self) -> String {
        match self {
            Self::OpenAI => format!("{}/v1/chat/completions", self.base_url()),
            Self::Anthropic => format!("{}/v1/messages", self.base_url()),
            Self::Gemini => self.base_url(),
        }
    }

    /// Models listing endpoint for this vendor.
    fn models_endpoint(self) -> String {
        format!("{}/v1/models", self.base_url())
    }
}

/// Ecosystem router service id for [`NeuralHttpClient::discover`].
fn ipc_service_id() -> String {
    std::env::var("SQUIRREL_ECOSYSTEM_IPC_SERVICE")
        .or_else(|_| std::env::var("ECOSYSTEM_ROUTER_SERVICE_ID"))
        .unwrap_or_else(|_| "nat0".to_string())
}

/// HTTP AI access delegated through IPC (`http.client` / neural proxy).
pub struct IpcRoutedVendorClient<D: IpcHttpDelegate = NeuralHttpClient> {
    http: Arc<D>,
    api_key: String,
    vendor: VendorKind,
}

impl<D: IpcHttpDelegate> fmt::Debug for IpcRoutedVendorClient<D> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IpcRoutedVendorClient")
            .field("vendor", &self.vendor)
            .finish_non_exhaustive()
    }
}

impl IpcRoutedVendorClient<NeuralHttpClient> {
    /// Discover ecosystem IPC and construct a vendor client.
    pub fn try_new(api_key: impl Into<String>, vendor: VendorKind) -> Result<Arc<AiClientImpl>> {
        let api_key = api_key.into();
        let sid = ipc_service_id();
        tracing::info!(
            target: "squirrel_ai_tools::ipc_routed",
            service_id = %sid,
            vendor = ?vendor,
            "Discovering ecosystem IPC socket for delegated vendor HTTP (capability routing)"
        );
        let http = NeuralHttpClient::discover(&sid).map_err(|e| {
            Error::UnsupportedProvider(format!(
                "IPC discovery failed for ecosystem HTTP delegation (service_id={sid}): {e}. \
                 Set SQUIRREL_ECOSYSTEM_IPC_SERVICE or ensure the ecosystem router socket exists."
            ))
        })?;
        Ok(Arc::new(AiClientImpl::IpcRouted(Box::new(Self {
            http: Arc::new(http),
            api_key,
            vendor,
        }))))
    }
}

impl<D: IpcHttpDelegate> IpcRoutedVendorClient<D> {
    const fn vendor_default_model(&self) -> &'static str {
        match self.vendor {
            VendorKind::OpenAI => "gpt-4o-mini",
            VendorKind::Anthropic => "claude-3-5-sonnet-20241022",
            VendorKind::Gemini => "gemini-1.5-flash",
        }
    }

    async fn chat_openai(&self, request: ChatRequest) -> Result<ChatResponse> {
        let model = request
            .model
            .clone()
            .unwrap_or_else(|| self.vendor_default_model().to_string());
        let mut messages = Vec::new();
        for m in &request.messages {
            let role = match m.role {
                MessageRole::System => "system",
                MessageRole::User => "user",
                MessageRole::Assistant => "assistant",
                MessageRole::Tool => "tool",
                MessageRole::Function => "function",
            };
            let mut o = json!({ "role": role });
            if let Some(ref c) = m.content {
                o["content"] = json!(c);
            }
            messages.push(o);
        }
        let body = json!({
            "model": model,
            "messages": messages,
        });
        let body_str = serde_json::to_string(&body).map_err(|e| Error::Provider(e.to_string()))?;
        let resp = self
            .http
            .post_json(
                &self.vendor.chat_endpoint(),
                vec![(
                    "Authorization".to_string(),
                    format!("Bearer {}", self.api_key),
                )],
                &body_str,
            )
            .await
            .map_err(|e| Error::Provider(e.to_string()))?;
        parse_openai_chat_response(&resp.body)
    }

    async fn chat_anthropic(&self, request: ChatRequest) -> Result<ChatResponse> {
        let model = request
            .model
            .clone()
            .unwrap_or_else(|| self.vendor_default_model().to_string());
        let mut system: Option<String> = None;
        let mut messages = Vec::new();
        for m in &request.messages {
            match m.role {
                MessageRole::System => {
                    system.clone_from(&m.content);
                }
                MessageRole::User => messages.push(json!({
                    "role": "user",
                    "content": m.content.as_deref().unwrap_or("")
                })),
                MessageRole::Assistant => messages.push(json!({
                    "role": "assistant",
                    "content": m.content.as_deref().unwrap_or("")
                })),
                _ => {}
            }
        }
        let mut body = json!({
            "model": model,
            "max_tokens": 4096u32,
            "messages": messages,
        });
        if let Some(s) = system {
            body["system"] = json!(s);
        }
        let body_str = serde_json::to_string(&body).map_err(|e| Error::Provider(e.to_string()))?;
        let resp = self
            .http
            .post_json(
                &self.vendor.chat_endpoint(),
                vec![
                    ("x-api-key".to_string(), self.api_key.clone()),
                    ("anthropic-version".to_string(), "2023-06-01".to_string()),
                ],
                &body_str,
            )
            .await
            .map_err(|e| Error::Provider(e.to_string()))?;
        parse_anthropic_chat_response(&resp.body)
    }

    async fn chat_gemini(&self, request: ChatRequest) -> Result<ChatResponse> {
        let model = request
            .model
            .clone()
            .unwrap_or_else(|| self.vendor_default_model().to_string());
        let mut parts = Vec::new();
        for m in &request.messages {
            if (m.role == MessageRole::User || m.role == MessageRole::System)
                && let Some(ref c) = m.content
            {
                parts.push(json!({"text": c}));
            }
        }
        let body = json!({ "contents": [{ "parts": parts }] });
        let body_str = serde_json::to_string(&body).map_err(|e| Error::Provider(e.to_string()))?;
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent?key={}",
            self.api_key
        );
        let resp = self
            .http
            .post_json(&url, vec![], &body_str)
            .await
            .map_err(|e| Error::Provider(e.to_string()))?;
        parse_gemini_chat_response(&resp.body, &model)
    }
}

impl<D: IpcHttpDelegate + 'static> AIClient for IpcRoutedVendorClient<D> {
    fn provider_name(&self) -> &str {
        match self.vendor {
            VendorKind::OpenAI => "openai-ipc",
            VendorKind::Anthropic => "anthropic-ipc",
            VendorKind::Gemini => "gemini-ipc",
        }
    }

    async fn get_capabilities(&self, _model: &str) -> Result<AICapabilities> {
        let mut c = AICapabilities::default();
        c.supported_model_types.insert(ModelType::ChatModel);
        c.supported_task_types.insert(TaskType::ChatCompletion);
        c.supported_task_types.insert(TaskType::TextGeneration);
        c.max_context_size = 128_000;
        Ok(c)
    }

    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
        match self.vendor {
            VendorKind::OpenAI => self.chat_openai(request).await,
            VendorKind::Anthropic => self.chat_anthropic(request).await,
            VendorKind::Gemini => self.chat_gemini(request).await,
        }
    }

    async fn chat_stream(&self, _request: ChatRequest) -> Result<ChatResponseStream> {
        Err(Error::UnsupportedProvider(
            "Streaming chat via IPC-delegated HTTP is not enabled in this build; use chat() or discover ai.chat.stream over IPC."
                .to_string(),
        ))
    }

    async fn list_models(&self) -> Result<Vec<String>> {
        match self.vendor {
            VendorKind::OpenAI => {
                let resp = self
                    .http
                    .get(
                        &self.vendor.models_endpoint(),
                        vec![(
                            "Authorization".to_string(),
                            format!("Bearer {}", self.api_key),
                        )],
                    )
                    .await
                    .map_err(|e| Error::Provider(e.to_string()))?;
                let v: Value =
                    serde_json::from_str(&resp.body).map_err(|e| Error::Parse(e.to_string()))?;
                let mut out = Vec::new();
                if let Some(data) = v["data"].as_array() {
                    for m in data {
                        if let Some(id) = m["id"].as_str() {
                            out.push(id.to_string());
                        }
                    }
                }
                Ok(out)
            }
            VendorKind::Anthropic | VendorKind::Gemini => {
                Ok(vec![self.vendor_default_model().to_string()])
            }
        }
    }

    async fn is_available(&self) -> bool {
        true
    }

    fn default_model(&self) -> &str {
        self.vendor_default_model()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
impl<D: IpcHttpDelegate> IpcRoutedVendorClient<D> {
    /// Build client with an injected HTTP delegate (unit tests).
    fn new_for_test(http: Arc<D>, api_key: impl Into<String>, vendor: VendorKind) -> Self {
        Self {
            http,
            api_key: api_key.into(),
            vendor,
        }
    }
}

#[cfg(test)]
#[path = "ipc_routed_providers_mocks.rs"]
mod ipc_routed_providers_mocks;

#[cfg(test)]
#[path = "ipc_routed_providers_tests.rs"]
mod ipc_routed_providers_tests;
