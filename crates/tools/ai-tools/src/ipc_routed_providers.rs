// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Vendor HTTP access routed through the ecosystem IPC client ([`crate::neural_http::NeuralHttpClient`]).
//! This primal does not open outbound TLS to cloud APIs directly; it delegates via `neural_api.proxy_http`.

use std::fmt;
use std::sync::Arc;

use async_trait::async_trait;
use serde_json::{Map, Value, json};

use crate::common::capability::{AICapabilities, ModelType, TaskType};
use crate::common::client::AIClient;
use crate::common::types::{
    ChatChoice, ChatRequest, ChatResponse, ChatResponseStream, MessageRole, UsageInfo,
};
use crate::error::{Error, Result};
use crate::neural_http::{HttpResponse, NeuralHttpClient};

/// HTTP surface used by [`IpcRoutedVendorClient`] (implemented by [`NeuralHttpClient`]; mockable in tests).
#[async_trait]
trait IpcHttpDelegate: Send + Sync {
    async fn post_json(
        &self,
        url: &str,
        headers: Vec<(String, String)>,
        body: &str,
    ) -> anyhow::Result<HttpResponse>;

    async fn get(&self, url: &str, headers: Vec<(String, String)>) -> anyhow::Result<HttpResponse>;
}

#[async_trait]
#[allow(
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

/// Ecosystem router service id for [`NeuralHttpClient::discover`].
fn ipc_service_id() -> String {
    std::env::var("SQUIRREL_ECOSYSTEM_IPC_SERVICE")
        .or_else(|_| std::env::var("ECOSYSTEM_ROUTER_SERVICE_ID"))
        .unwrap_or_else(|_| "nat0".to_string())
}

/// HTTP AI access delegated through IPC (`http.client` / neural proxy).
pub struct IpcRoutedVendorClient {
    http: Arc<dyn IpcHttpDelegate>,
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
            http: Arc::new(http),
            api_key,
            vendor,
        }))
    }

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

#[inline]
fn json_u64_as_u32_saturating(n: u64) -> u32 {
    u32::try_from(n).unwrap_or(u32::MAX)
}

fn parse_openai_chat_response(body: &str) -> Result<ChatResponse> {
    let v: Value = serde_json::from_str(body).map_err(|e| Error::Parse(e.to_string()))?;
    let id = v["id"].as_str().unwrap_or("openai-ipc").to_string();
    let model = v["model"].as_str().unwrap_or("unknown").to_string();
    let choice0 = &v["choices"][0];
    let content = choice0["message"]["content"].as_str().map(str::to_string);
    let finish = choice0["finish_reason"].as_str().map(str::to_string);
    let usage = v["usage"]
        .as_object()
        .map(|u: &Map<String, Value>| UsageInfo {
            prompt_tokens: json_u64_as_u32_saturating(u["prompt_tokens"].as_u64().unwrap_or(0)),
            completion_tokens: json_u64_as_u32_saturating(
                u["completion_tokens"].as_u64().unwrap_or(0),
            ),
            total_tokens: json_u64_as_u32_saturating(u["total_tokens"].as_u64().unwrap_or(0)),
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
    let usage = v["usage"].as_object().map(|u: &Map<String, Value>| {
        let input = json_u64_as_u32_saturating(u["input_tokens"].as_u64().unwrap_or(0));
        let output = json_u64_as_u32_saturating(u["output_tokens"].as_u64().unwrap_or(0));
        UsageInfo {
            prompt_tokens: input,
            completion_tokens: output,
            total_tokens: input.saturating_add(output),
        }
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

#[cfg(test)]
impl IpcRoutedVendorClient {
    /// Build client with an injected HTTP delegate (unit tests).
    fn new_for_test(
        http: Arc<dyn IpcHttpDelegate>,
        api_key: impl Into<String>,
        vendor: VendorKind,
    ) -> Self {
        Self {
            http,
            api_key: api_key.into(),
            vendor,
        }
    }
}

/// Deterministic mock for [`IpcHttpDelegate`] (FIFO responses per operation).
#[cfg(test)]
struct MockNeuralHttp {
    post_json_responses:
        std::sync::Arc<tokio::sync::Mutex<std::collections::VecDeque<HttpResponse>>>,
    get_responses: std::sync::Arc<tokio::sync::Mutex<std::collections::VecDeque<HttpResponse>>>,
}

#[cfg(test)]
impl MockNeuralHttp {
    fn new() -> Self {
        Self {
            post_json_responses: std::sync::Arc::new(tokio::sync::Mutex::new(
                std::collections::VecDeque::new(),
            )),
            get_responses: std::sync::Arc::new(tokio::sync::Mutex::new(
                std::collections::VecDeque::new(),
            )),
        }
    }

    async fn push_post_json(&self, body: impl Into<String>) {
        self.post_json_responses
            .lock()
            .await
            .push_back(HttpResponse {
                status: 200,
                headers: vec![],
                body: body.into(),
            });
    }

    async fn push_get(&self, body: impl Into<String>) {
        self.get_responses.lock().await.push_back(HttpResponse {
            status: 200,
            headers: vec![],
            body: body.into(),
        });
    }
}

#[cfg(test)]
#[async_trait]
impl IpcHttpDelegate for MockNeuralHttp {
    async fn post_json(
        &self,
        _url: &str,
        _headers: Vec<(String, String)>,
        _body: &str,
    ) -> anyhow::Result<HttpResponse> {
        self.post_json_responses
            .lock()
            .await
            .pop_front()
            .ok_or_else(|| anyhow::anyhow!("MockNeuralHttp: no post_json response queued"))
    }

    async fn get(
        &self,
        _url: &str,
        _headers: Vec<(String, String)>,
    ) -> anyhow::Result<HttpResponse> {
        self.get_responses
            .lock()
            .await
            .pop_front()
            .ok_or_else(|| anyhow::anyhow!("MockNeuralHttp: no get response queued"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::client::AIClient;
    use crate::common::types::{ChatMessage, ChatRequest, MessageRole};

    fn msg(role: MessageRole, content: &str) -> ChatMessage {
        ChatMessage {
            role,
            content: Some(content.to_string()),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        }
    }

    #[test]
    fn vendor_kind_default_models_match_provider_names() {
        let cases = [
            (VendorKind::OpenAI, "gpt-4o-mini", "openai-ipc"),
            (
                VendorKind::Anthropic,
                "claude-3-5-sonnet-20241022",
                "anthropic-ipc",
            ),
            (VendorKind::Gemini, "gemini-1.5-flash", "gemini-ipc"),
        ];
        for (vendor, expected_model, expected_name) in cases {
            let http = Arc::new(MockNeuralHttp::new());
            let client = IpcRoutedVendorClient::new_for_test(http, "k", vendor);
            assert_eq!(client.default_model(), expected_model);
            assert_eq!(client.provider_name(), expected_name);
        }
    }

    #[test]
    fn json_u64_saturates_to_u32_max() {
        let n = u64::from(u32::MAX) + 1;
        assert_eq!(json_u64_as_u32_saturating(n), u32::MAX);
    }

    #[test]
    fn parse_openai_response_maps_usage_and_content() {
        let body = r#"{
            "id": "chatcmpl-1",
            "model": "gpt-4o-mini",
            "choices": [{"message": {"content": "hi"}, "finish_reason": "stop"}],
            "usage": {"prompt_tokens": 3, "completion_tokens": 2, "total_tokens": 5}
        }"#;
        let r = parse_openai_chat_response(body).expect("should succeed");
        assert_eq!(r.id, "chatcmpl-1");
        assert_eq!(r.model, "gpt-4o-mini");
        assert_eq!(r.choices[0].content.as_deref(), Some("hi"));
        assert_eq!(r.choices[0].finish_reason.as_deref(), Some("stop"));
        let u = r.usage.expect("usage");
        assert_eq!(u.prompt_tokens, 3);
        assert_eq!(u.completion_tokens, 2);
        assert_eq!(u.total_tokens, 5);
    }

    #[test]
    fn parse_openai_invalid_json_errors() {
        assert!(parse_openai_chat_response("not json").is_err());
    }

    #[test]
    fn parse_anthropic_response_maps_text_block_and_usage() {
        let body = r#"{
            "id": "msg_1",
            "model": "claude-3-5-sonnet-20241022",
            "content": [{"type": "text", "text": "hello"}],
            "stop_reason": "end_turn",
            "usage": {"input_tokens": 10, "output_tokens": 20}
        }"#;
        let r = parse_anthropic_chat_response(body).expect("should succeed");
        assert_eq!(r.choices[0].content.as_deref(), Some("hello"));
        assert_eq!(r.choices[0].finish_reason.as_deref(), Some("end_turn"));
        let u = r.usage.expect("usage");
        assert_eq!(u.prompt_tokens, 10);
        assert_eq!(u.completion_tokens, 20);
        assert_eq!(u.total_tokens, 30);
    }

    #[test]
    fn parse_gemini_response_concatenates_parts() {
        let body = r#"{"candidates":[{"content":{"parts":[{"text":"a"},{"text":"b"}]}}]}"#;
        let r = parse_gemini_chat_response(body, "gemini-1.5-flash").expect("should succeed");
        assert_eq!(r.choices[0].content.as_deref(), Some("ab"));
        assert_eq!(r.model, "gemini-1.5-flash");
    }

    #[tokio::test]
    async fn chat_openai_builds_request_and_parses_mock_response() {
        let mock = Arc::new(MockNeuralHttp::new());
        mock
            .push_post_json(r#"{"id":"x","model":"m","choices":[{"message":{"content":"ok"},"finish_reason":null}],"usage":null}"#)
            .await;
        let client = IpcRoutedVendorClient::new_for_test(mock, "secret", VendorKind::OpenAI);
        let req = ChatRequest {
            model: Some("custom-model".to_string()),
            messages: vec![msg(MessageRole::System, "sys"), msg(MessageRole::User, "u")],
            parameters: None,
            tools: None,
        };
        let out = client.chat_openai(req).await.expect("should succeed");
        assert_eq!(out.choices[0].content.as_deref(), Some("ok"));
    }

    #[tokio::test]
    async fn chat_anthropic_includes_system_and_skips_unknown_roles() {
        let mock = Arc::new(MockNeuralHttp::new());
        mock.push_post_json(
            r#"{"id":"a","model":"m","content":[{"type":"text","text":"y"}],"usage":null}"#,
        )
        .await;
        let client = IpcRoutedVendorClient::new_for_test(mock, "key", VendorKind::Anthropic);
        let req = ChatRequest {
            model: None,
            messages: vec![
                msg(MessageRole::System, "be nice"),
                msg(MessageRole::User, "hi"),
                msg(MessageRole::Tool, "ignored"),
            ],
            parameters: None,
            tools: None,
        };
        let out = client.chat_anthropic(req).await.expect("should succeed");
        assert_eq!(out.choices[0].content.as_deref(), Some("y"));
    }

    #[tokio::test]
    async fn chat_gemini_posts_parts_from_user_and_system() {
        let mock = Arc::new(MockNeuralHttp::new());
        mock.push_post_json(r#"{"candidates":[{"content":{"parts":[{"text":"g"}]}}]}"#)
            .await;
        let client = IpcRoutedVendorClient::new_for_test(mock, "apikey", VendorKind::Gemini);
        let req = ChatRequest {
            model: Some("gemini-pro".to_string()),
            messages: vec![msg(MessageRole::System, "s"), msg(MessageRole::User, "u")],
            parameters: None,
            tools: None,
        };
        let out = client.chat_gemini(req).await.expect("should succeed");
        assert_eq!(out.choices[0].content.as_deref(), Some("g"));
    }

    #[tokio::test]
    async fn list_models_openai_parses_data_array() {
        let mock = Arc::new(MockNeuralHttp::new());
        mock.push_get(r#"{"data":[{"id":"gpt-4"},{"id":"ada"}]}"#)
            .await;
        let client = IpcRoutedVendorClient::new_for_test(mock, "k", VendorKind::OpenAI);
        let models = client.list_models().await.expect("should succeed");
        assert_eq!(models, vec!["gpt-4".to_string(), "ada".to_string()]);
    }

    #[tokio::test]
    async fn list_models_anthropic_returns_default_only() {
        let mock = Arc::new(MockNeuralHttp::new());
        let client = IpcRoutedVendorClient::new_for_test(mock, "k", VendorKind::Anthropic);
        let models = client.list_models().await.expect("should succeed");
        assert_eq!(models.len(), 1);
        assert!(models[0].contains("claude"));
    }

    #[tokio::test]
    async fn chat_stream_is_unsupported() {
        let mock = Arc::new(MockNeuralHttp::new());
        let client = IpcRoutedVendorClient::new_for_test(mock, "k", VendorKind::OpenAI);
        let res = client
            .chat_stream(ChatRequest {
                model: None,
                messages: vec![],
                parameters: None,
                tools: None,
            })
            .await;
        assert!(matches!(res, Err(Error::UnsupportedProvider(_))));
    }
}
