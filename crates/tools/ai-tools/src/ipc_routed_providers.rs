// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Vendor HTTP access routed through the ecosystem IPC client ([`crate::neural_http::NeuralHttpClient`]).
//! This primal does not open outbound TLS to cloud APIs directly; it delegates via `neural_api.proxy_http`.

use std::fmt;
use std::sync::Arc;

use async_trait::async_trait;
use serde_json::{Value, json};

use crate::common::capability::{AICapabilities, ModelType, TaskType};
use crate::common::client::AIClient;
use crate::common::types::{
    ChatChoice, ChatMessage, ChatRequest, ChatResponse, ChatResponseStream, MessageRole, UsageInfo,
};
use crate::error::{Error, Result};
use crate::neural_http::NeuralHttpClient;

/// Which vendor API shape to use when building requests (all traffic still goes through IPC).
#[derive(Debug, Clone, Copy)]
pub enum VendorKind {
    OpenAI,
    Anthropic,
    Gemini,
}

/// Ecosystem router service id for [`NeuralHttpClient::discover`].
fn ipc_service_id() -> String {
    std::env::var("SQUIRREL_ECOSYSTEM_IPC_SERVICE")
        .or_else(|_| std::env::var("ECOSYSTEM_ROUTER_SERVICE_ID"))
        .unwrap_or_else(|_| "nat0".to_string())
}

/// HTTP AI access delegated through IPC (`http.client` / neural proxy).
pub struct IpcRoutedVendorClient {
    http: NeuralHttpClient,
    api_key: String,
    vendor: VendorKind,
}

impl fmt::Debug for IpcRoutedVendorClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IpcRoutedVendorClient")
            .field("vendor", &self.vendor)
            .finish_non_exhaustive()
    }
}

impl IpcRoutedVendorClient {
    /// Discover ecosystem IPC and construct a vendor client.
    pub fn try_new(api_key: impl Into<String>, vendor: VendorKind) -> Result<Arc<dyn AIClient>> {
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
        Ok(Arc::new(Self {
            http,
            api_key,
            vendor,
        }))
    }

    fn vendor_default_model(&self) -> &'static str {
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
                "https://api.openai.com/v1/chat/completions",
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
                    system = m.content.clone();
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
                "https://api.anthropic.com/v1/messages",
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

fn parse_openai_chat_response(body: &str) -> Result<ChatResponse> {
    let v: Value = serde_json::from_str(body).map_err(|e| Error::Parse(e.to_string()))?;
    let id = v["id"].as_str().unwrap_or("openai-ipc").to_string();
    let model = v["model"].as_str().unwrap_or("unknown").to_string();
    let choice0 = &v["choices"][0];
    let content = choice0["message"]["content"].as_str().map(str::to_string);
    let finish = choice0["finish_reason"].as_str().map(str::to_string);
    let usage = v["usage"].as_object().map(|u| UsageInfo {
        prompt_tokens: u["prompt_tokens"].as_u64().unwrap_or(0) as u32,
        completion_tokens: u["completion_tokens"].as_u64().unwrap_or(0) as u32,
        total_tokens: u["total_tokens"].as_u64().unwrap_or(0) as u32,
    });
    Ok(ChatResponse {
        id,
        model,
        choices: vec![ChatChoice {
            index: 0,
            role: MessageRole::Assistant,
            content,
            finish_reason: finish,
            tool_calls: None,
        }],
        usage,
    })
}

fn parse_anthropic_chat_response(body: &str) -> Result<ChatResponse> {
    let v: Value = serde_json::from_str(body).map_err(|e| Error::Parse(e.to_string()))?;
    let id = v["id"].as_str().unwrap_or("anthropic-ipc").to_string();
    let model = v["model"].as_str().unwrap_or("unknown").to_string();
    let mut content: Option<String> = None;
    if let Some(arr) = v["content"].as_array() {
        for block in arr {
            if block["type"].as_str() == Some("text") {
                content = block["text"].as_str().map(str::to_string);
                break;
            }
        }
    }
    let usage = v["usage"].as_object().map(|u| UsageInfo {
        prompt_tokens: u["input_tokens"].as_u64().unwrap_or(0) as u32,
        completion_tokens: u["output_tokens"].as_u64().unwrap_or(0) as u32,
        total_tokens: u["input_tokens"].as_u64().unwrap_or(0) as u32
            + u["output_tokens"].as_u64().unwrap_or(0) as u32,
    });
    Ok(ChatResponse {
        id,
        model,
        choices: vec![ChatChoice {
            index: 0,
            role: MessageRole::Assistant,
            content,
            finish_reason: v["stop_reason"].as_str().map(str::to_string),
            tool_calls: None,
        }],
        usage,
    })
}

fn parse_gemini_chat_response(body: &str, model: &str) -> Result<ChatResponse> {
    let v: Value = serde_json::from_str(body).map_err(|e| Error::Parse(e.to_string()))?;
    let mut text = String::new();
    if let Some(c) = v["candidates"][0]["content"]["parts"].as_array() {
        for p in c {
            if let Some(t) = p["text"].as_str() {
                text.push_str(t);
            }
        }
    }
    Ok(ChatResponse {
        id: "gemini-ipc".to_string(),
        model: model.to_string(),
        choices: vec![ChatChoice {
            index: 0,
            role: MessageRole::Assistant,
            content: Some(text),
            finish_reason: None,
            tool_calls: None,
        }],
        usage: None,
    })
}

#[async_trait]
impl AIClient for IpcRoutedVendorClient {
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
                        "https://api.openai.com/v1/models",
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
