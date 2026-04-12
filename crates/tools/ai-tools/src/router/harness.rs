// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Configurable [`RouterHarnessClient`] for router unit tests (optimization + dispatch).

use crate::Result;
use crate::common::RoutingPreferences;
use crate::common::capability::{AICapabilities, TaskType};
use crate::common::client::AIClient;
use crate::common::types::{
    ChatChoice, ChatRequest, ChatResponse, ChatResponseStream, MessageRole,
};
use crate::error::Error;

/// Test double with configurable capabilities and routing metadata (replaces ad-hoc `MockClient` / `TestClient`).
///
/// Only used via [`crate::AiClientImpl::RouterHarness`]; not intended for application code.
#[derive(Debug, Clone)]
pub struct RouterHarnessClient {
    pub(crate) name: String,
    pub(crate) caps: AICapabilities,
    pub(crate) prefs: RoutingPreferences,
    pub(crate) default_model: String,
    pub(crate) chat_ok: bool,
}

impl RouterHarnessClient {
    pub(crate) fn new(name: &str) -> Self {
        let mut caps = AICapabilities::new();
        caps.add_task_type(TaskType::TextGeneration);
        caps.add_model_type(crate::common::capability::ModelType::LargeLanguageModel);
        caps.max_context_size = 8192;
        caps.supports_streaming = true;
        caps.supports_function_calling = true;
        caps.supports_tool_use = true;
        caps.performance_metrics.avg_latency_ms = Some(100);
        Self {
            name: name.to_string(),
            caps,
            prefs: RoutingPreferences::default(),
            default_model: "mock-model".to_string(),
            chat_ok: true,
        }
    }

    pub(crate) fn with_prefs(mut self, prefs: RoutingPreferences) -> Self {
        self.prefs = prefs;
        self
    }

    pub(crate) fn with_caps(mut self, caps: AICapabilities) -> Self {
        self.caps = caps;
        self
    }

    pub(crate) fn with_default_model(mut self, m: impl Into<String>) -> Self {
        self.default_model = m.into();
        self
    }

    pub(crate) fn with_chat_ok(mut self, ok: bool) -> Self {
        self.chat_ok = ok;
        self
    }
}

impl AIClient for RouterHarnessClient {
    fn provider_name(&self) -> &str {
        &self.name
    }

    async fn get_capabilities(&self, _model: &str) -> Result<AICapabilities> {
        Ok(self.caps.clone())
    }

    fn capabilities(&self) -> AICapabilities {
        self.caps.clone()
    }

    fn routing_preferences(&self) -> RoutingPreferences {
        self.prefs.clone()
    }

    async fn is_available(&self) -> bool {
        true
    }

    fn default_model(&self) -> &str {
        &self.default_model
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    async fn list_models(&self) -> Result<Vec<String>> {
        Ok(vec![format!("{}-model", self.name)])
    }

    async fn chat(&self, _request: ChatRequest) -> Result<ChatResponse> {
        if !self.chat_ok {
            return Err(Error::Configuration("chat failed".to_string()));
        }
        Ok(ChatResponse {
            id: "mock-response".to_string(),
            model: self.default_model.clone(),
            choices: vec![ChatChoice {
                index: 0,
                role: MessageRole::Assistant,
                content: Some("ok".to_string()),
                finish_reason: Some("stop".to_string()),
                tool_calls: None,
            }],
            usage: None,
        })
    }

    async fn chat_stream(&self, request: ChatRequest) -> Result<ChatResponseStream> {
        self.chat(request).await?;
        Err(Error::Configuration(
            "stream unsupported in RouterHarnessClient".to_string(),
        ))
    }
}
