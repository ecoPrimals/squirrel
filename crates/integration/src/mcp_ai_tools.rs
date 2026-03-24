// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! MCP AI Tools Adapter
#![allow(
    dead_code,
    reason = "MCP AI tools integration adapter awaiting activation"
)]
//!
//! This module provides an adapter for the MCP AI tools.

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use uuid;

use squirrel_ai_tools::common::capability::AICapabilities;
use squirrel_ai_tools::common::{ChatRequest, ChatResponse, ChatResponseStream};
use squirrel_ai_tools::router::MCPInterface;

/// Provider settings for AI tools
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderSettings {
    /// Provider ID
    pub id: String,
    /// Provider name
    pub name: String,
    /// Provider configuration
    pub config: HashMap<String, String>,
    /// Provider models
    pub models: Vec<String>,
}

impl ProviderSettings {
    /// Create new provider settings with default values
    pub fn default_openai() -> Self {
        Self {
            id: "openai".to_string(),
            name: "OpenAI".to_string(),
            config: HashMap::new(),
            models: vec!["gpt-3.5-turbo".to_string()],
        }
    }

    /// Add a parameter to the provider settings
    pub fn with_parameter(mut self, key: String, value: serde_json::Value) -> Self {
        self.config.insert(key, value.to_string());
        self
    }

    /// Set the models for the provider
    pub fn with_models(mut self, models: Vec<String>) -> Self {
        self.models = models;
        self
    }
}

/// Provider registry for AI tools
#[derive(Debug, Default)]
pub struct ProviderRegistry {
    /// Registered providers
    providers: HashMap<String, ProviderSettings>,
}

impl ProviderRegistry {
    /// Create a new provider registry
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    /// Register a provider
    pub fn register_provider(&mut self, id: &str, settings: ProviderSettings) -> Result<()> {
        self.providers.insert(id.to_string(), settings);
        Ok(())
    }

    /// Get a provider
    pub fn get_provider(&self, id: &str) -> Result<&ProviderSettings> {
        self.providers
            .get(id)
            .context(format!("Provider not found: {}", id))
    }

    /// List providers
    pub fn list_providers(&self) -> Vec<String> {
        self.providers.keys().cloned().collect()
    }

    /// Get provider capabilities
    pub fn get_provider_capabilities(&self, _provider_id: &str) -> Option<AICapabilities> {
        // For now, return None as we don't have actual capabilities
        None
    }
}

/// Configuration for the MCP AI tools adapter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpAiToolsConfig {
    /// Provider registry
    #[serde(skip)]
    providers: HashMap<String, ProviderSettings>,
    /// Timeout in milliseconds
    pub timeout_ms: u64,
    /// Whether to enable streaming
    pub streaming: bool,
    /// Default Ollama endpoint
    pub default_ollama_endpoint: String,
}

impl Default for McpAiToolsConfig {
    fn default() -> Self {
        // Multi-tier Ollama endpoint resolution
        // 1. OLLAMA_ENDPOINT (full endpoint)
        // 2. TOADSTOOL_ENDPOINT (ToadStool as Ollama host)
        // 3. OLLAMA_PORT or TOADSTOOL_PORT (port override)
        // 4. Default: http://localhost:11434
        let default_ollama_endpoint = std::env::var("OLLAMA_ENDPOINT")
            .or_else(|_| std::env::var("TOADSTOOL_ENDPOINT"))
            .unwrap_or_else(|_| {
                let port = std::env::var("OLLAMA_PORT")
                    .or_else(|_| std::env::var("TOADSTOOL_PORT"))
                    .ok()
                    .and_then(|p| p.parse::<u16>().ok())
                    .unwrap_or(11434); // Default Ollama port
                format!("http://localhost:{}", port)
            });

        Self {
            providers: HashMap::new(),
            timeout_ms: 30000,
            streaming: true,
            default_ollama_endpoint,
        }
    }
}

impl McpAiToolsConfig {
    /// Add a provider to the configuration
    pub fn with_provider(mut self, id: String, settings: ProviderSettings) -> Self {
        self.providers.insert(id, settings);
        self
    }

    /// Set the timeout
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }

    /// Set streaming
    pub fn with_streaming(mut self, streaming: bool) -> Self {
        self.streaming = streaming;
        self
    }

    /// Set default Ollama endpoint
    pub fn with_default_ollama_endpoint(mut self, endpoint: String) -> Self {
        self.default_ollama_endpoint = endpoint;
        self
    }
}

/// MCP AI tools adapter
pub struct McpAiToolsAdapter {
    /// MCP interface
    mcp: Arc<dyn MCPInterface>,
    /// Provider registry
    provider_registry: Arc<ProviderRegistry>,
    /// Configuration
    config: McpAiToolsConfig,
}

impl std::fmt::Debug for McpAiToolsAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("McpAiToolsAdapter")
            .field("provider_registry", &self.provider_registry)
            .field("config", &self.config)
            .finish()
    }
}

impl McpAiToolsAdapter {
    /// Create a new MCP AI tools adapter
    pub fn new(mcp: Arc<dyn MCPInterface>, config: McpAiToolsConfig) -> Result<Self> {
        // Create provider registry
        let mut provider_registry = ProviderRegistry::new();

        // Register providers from config
        for (id, settings) in &config.providers {
            provider_registry
                .register_provider(id, settings.clone())
                .context(format!("Failed to register provider: {}", id))?;
        }

        Ok(Self {
            mcp,
            provider_registry: Arc::new(provider_registry),
            config,
        })
    }

    /// Get the provider registry
    pub fn provider_registry(&self) -> Arc<ProviderRegistry> {
        self.provider_registry.clone()
    }

    /// Create a chat request
    pub fn create_chat_request(&self) -> ChatRequest {
        ChatRequest::new()
    }

    /// Send a chat request
    pub async fn send_chat_request(
        &self,
        provider_id: &str,
        request: ChatRequest,
    ) -> Result<ChatResponse> {
        // Get provider
        let provider = self
            .provider_registry
            .get_provider(provider_id)
            .context(format!("Provider not found: {}", provider_id))?;

        // Route request to appropriate provider implementation
        match provider.id.as_str() {
            "openai" => {
                tracing::debug!("Processing OpenAI chat request via capability_ai");
                // Use capability-based AI client (TRUE PRIMAL!)
                use squirrel_ai_tools::capability_ai::{AiClient, ChatMessage as CapMsg};

                let client =
                    AiClient::from_env().context("Failed to create capability AI client")?;

                // Convert messages to capability format
                let messages: Vec<CapMsg> = request
                    .messages
                    .iter()
                    .map(|m| CapMsg {
                        role: match m.role {
                            squirrel_ai_tools::common::MessageRole::System => "system".to_string(),
                            squirrel_ai_tools::common::MessageRole::User => "user".to_string(),
                            squirrel_ai_tools::common::MessageRole::Assistant => {
                                "assistant".to_string()
                            }
                            _ => "user".to_string(),
                        },
                        content: m.content.clone().unwrap_or_default(),
                    })
                    .collect();

                let model = request.model.as_deref().unwrap_or("gpt-4");
                let cap_response = client
                    .chat_completion(model, messages, None)
                    .await
                    .context("Failed to process OpenAI chat via capability_ai")?;

                // Convert back to common format
                let response = squirrel_ai_tools::common::ChatResponse {
                    choices: vec![squirrel_ai_tools::common::ChatChoice {
                        index: 0,
                        content: Some(cap_response.content),
                        role: squirrel_ai_tools::common::MessageRole::Assistant,
                        finish_reason: Some("stop".to_string()),
                        tool_calls: None,
                    }],
                    usage: cap_response
                        .usage
                        .map(|u| squirrel_ai_tools::common::UsageInfo {
                            prompt_tokens: u.prompt_tokens,
                            completion_tokens: u.completion_tokens,
                            total_tokens: u.total_tokens,
                        }),
                    model: model.to_string(),
                    id: uuid::Uuid::new_v4().to_string(),
                };

                Ok(response)
            }
            "anthropic" => {
                tracing::debug!("Processing Anthropic chat request via capability_ai");
                // Use capability-based AI client (TRUE PRIMAL!)
                use squirrel_ai_tools::capability_ai::{AiClient, ChatMessage as CapMsg};

                let client =
                    AiClient::from_env().context("Failed to create capability AI client")?;

                // Convert messages to capability format
                let messages: Vec<CapMsg> = request
                    .messages
                    .iter()
                    .map(|m| CapMsg {
                        role: match m.role {
                            squirrel_ai_tools::common::MessageRole::System => "system".to_string(),
                            squirrel_ai_tools::common::MessageRole::User => "user".to_string(),
                            squirrel_ai_tools::common::MessageRole::Assistant => {
                                "assistant".to_string()
                            }
                            _ => "user".to_string(),
                        },
                        content: m.content.clone().unwrap_or_default(),
                    })
                    .collect();

                let model = request.model.as_deref().unwrap_or("claude-3-opus");
                let cap_response = client
                    .chat_completion(model, messages, None)
                    .await
                    .context("Failed to process Anthropic chat via capability_ai")?;

                // Convert back to common format
                let response = squirrel_ai_tools::common::ChatResponse {
                    choices: vec![squirrel_ai_tools::common::ChatChoice {
                        index: 0,
                        content: Some(cap_response.content),
                        role: squirrel_ai_tools::common::MessageRole::Assistant,
                        finish_reason: Some("stop".to_string()),
                        tool_calls: None,
                    }],
                    usage: cap_response
                        .usage
                        .map(|u| squirrel_ai_tools::common::UsageInfo {
                            prompt_tokens: u.prompt_tokens,
                            completion_tokens: u.completion_tokens,
                            total_tokens: u.total_tokens,
                        }),
                    model: model.to_string(),
                    id: uuid::Uuid::new_v4().to_string(),
                };

                Ok(response)
            }
            "ollama" => {
                tracing::debug!("Processing Ollama chat request via capability_ai");
                // Use capability-based AI client (TRUE PRIMAL!)
                use squirrel_ai_tools::capability_ai::{AiClient, ChatMessage as CapMsg};

                let client =
                    AiClient::from_env().context("Failed to create capability AI client")?;

                // Convert messages to capability format
                let messages: Vec<CapMsg> = request
                    .messages
                    .iter()
                    .map(|m| CapMsg {
                        role: match m.role {
                            squirrel_ai_tools::common::MessageRole::System => "system".to_string(),
                            squirrel_ai_tools::common::MessageRole::User => "user".to_string(),
                            squirrel_ai_tools::common::MessageRole::Assistant => {
                                "assistant".to_string()
                            }
                            _ => "user".to_string(),
                        },
                        content: m.content.clone().unwrap_or_default(),
                    })
                    .collect();

                let model = request.model.as_deref().unwrap_or("llama2");
                let cap_response = client
                    .chat_completion(model, messages, None)
                    .await
                    .context("Failed to process Ollama chat via capability_ai")?;

                // Convert back to common format
                let response = squirrel_ai_tools::common::ChatResponse {
                    choices: vec![squirrel_ai_tools::common::ChatChoice {
                        index: 0,
                        content: Some(cap_response.content),
                        role: squirrel_ai_tools::common::MessageRole::Assistant,
                        finish_reason: Some("stop".to_string()),
                        tool_calls: None,
                    }],
                    usage: cap_response
                        .usage
                        .map(|u| squirrel_ai_tools::common::UsageInfo {
                            prompt_tokens: u.prompt_tokens,
                            completion_tokens: u.completion_tokens,
                            total_tokens: u.total_tokens,
                        }),
                    model: model.to_string(),
                    id: uuid::Uuid::new_v4().to_string(),
                };

                Ok(response)
            }
            _ => {
                tracing::error!("Unknown provider: {}", provider_id);
                Err(anyhow::anyhow!("Unknown provider: {}", provider_id))
            }
        }
    }

    /// Send a streaming chat request
    /// **FUTURE**: Implement streaming with `capability_ai`
    /// Tracking: Planned for v0.2.0 - streaming support
    pub async fn send_streaming_chat_request(
        &self,
        _provider_id: &str,
        _request: ChatRequest,
    ) -> Result<ChatResponseStream> {
        // Streaming implementation: delegate to capability_ai with streaming support
        // For now, return clear error guidance
        tracing::warn!(
            "Streaming chat requested for provider {} - not yet implemented",
            _provider_id
        );
        Err(anyhow::anyhow!(
            "Streaming chat responses not yet implemented in capability_ai integration. \
             Please use send_chat_request() for batch responses. \
             Streaming support is planned for future release. \
             See https://github.com/ecoPrimals/squirrel/issues for tracking."
        ))
    }

    /// Generate a response for the specified message
    pub async fn generate_response(
        &self,
        conversation_id: &str,
        model: Option<String>,
        _temperature: Option<f32>,
        _max_tokens: Option<u32>,
    ) -> Result<String, anyhow::Error> {
        // Production implementation: this method requires a proper provider integration
        tracing::warn!(
            "generate_response called for conversation {} with model {:?} - not yet implemented",
            conversation_id,
            model
        );

        // Return a clear error indicating the feature is not yet implemented
        Err(anyhow::anyhow!(
            "AI response generation not yet implemented. Please use send_chat_request with a specific provider instead."
        ))
    }

    /// List available providers
    pub fn list_providers(&self) -> Vec<String> {
        self.provider_registry.list_providers()
    }

    /// Get provider capabilities
    pub fn get_provider_capabilities(&self, provider_id: &str) -> Option<AICapabilities> {
        self.provider_registry
            .get_provider_capabilities(provider_id)
    }
}

/// Create a new MCP AI tools adapter with default configuration
pub fn create_mcp_ai_tools_adapter(mcp: Arc<dyn MCPInterface>) -> Result<McpAiToolsAdapter> {
    McpAiToolsAdapter::new(mcp, McpAiToolsConfig::default())
}

/// Create a new MCP AI tools adapter with the specified configuration
pub fn create_mcp_ai_tools_adapter_with_config(
    mcp: Arc<dyn MCPInterface>,
    config: McpAiToolsConfig,
) -> Result<McpAiToolsAdapter> {
    McpAiToolsAdapter::new(mcp, config)
}

#[cfg(test)]
mod tests {
    #![expect(
        deprecated,
        reason = "squirrel_ai_tools::AIError variants deprecated during universal_error migration"
    )]

    use super::*;
    use async_trait::async_trait;
    use squirrel_ai_tools::common::capability::AICapabilities;
    use squirrel_ai_tools::error::{Error as AiError, Result as AiResult};
    use squirrel_ai_tools::router::types::{NodeId, RemoteAIRequest, RemoteAIResponseStream};
    use std::collections::HashMap;

    struct MockMcp;

    #[async_trait]
    impl MCPInterface for MockMcp {
        async fn send_request(
            &self,
            _node_id: &NodeId,
            _request: RemoteAIRequest,
        ) -> AiResult<squirrel_ai_tools::router::types::RemoteAIResponse> {
            Err(AiError::Network("mock".to_string()))
        }

        async fn stream_request(
            &self,
            _node_id: &NodeId,
            _request: RemoteAIRequest,
        ) -> AiResult<RemoteAIResponseStream> {
            Err(AiError::Network("mock stream".to_string()))
        }

        async fn discover_capabilities(
            &self,
        ) -> AiResult<HashMap<NodeId, HashMap<String, AICapabilities>>> {
            Ok(HashMap::new())
        }
    }

    #[test]
    fn provider_settings_defaults_and_builders() {
        let p = ProviderSettings::default_openai()
            .with_parameter("k".to_string(), serde_json::json!(1))
            .with_models(vec!["m1".to_string()]);
        assert_eq!(p.id, "openai");
        assert_eq!(p.models, vec!["m1".to_string()]);
        assert!(p.config.contains_key("k"));
    }

    #[test]
    fn provider_settings_serde_roundtrip() {
        let p = ProviderSettings {
            id: "x".to_string(),
            name: "n".to_string(),
            config: std::iter::once(("a".to_string(), "b".to_string())).collect(),
            models: vec!["m".to_string()],
        };
        let j = serde_json::to_string(&p).unwrap();
        let p2: ProviderSettings = serde_json::from_str(&j).unwrap();
        assert_eq!(p2.id, p.id);
    }

    #[test]
    fn provider_registry_register_get_list_errors() {
        let mut r = ProviderRegistry::new();
        r.register_provider("a", ProviderSettings::default_openai())
            .unwrap();
        assert_eq!(r.get_provider("a").unwrap().id, "openai");
        assert!(r.list_providers().contains(&"a".to_string()));
        assert!(r.get_provider("missing").is_err());
        assert!(r.get_provider_capabilities("a").is_none());
    }

    #[test]
    fn mcp_ai_tools_config_builder_and_serde() {
        let c = McpAiToolsConfig::default()
            .with_timeout(1234)
            .with_streaming(false)
            .with_default_ollama_endpoint("http://127.0.0.1:1".to_string())
            .with_provider("p1".to_string(), ProviderSettings::default_openai());
        assert_eq!(c.timeout_ms, 1234);
        assert!(!c.streaming);
        let j = serde_json::to_string(&c).unwrap();
        let c2: McpAiToolsConfig = serde_json::from_str(&j).unwrap();
        assert_eq!(c2.timeout_ms, c.timeout_ms);
        assert_eq!(c2.default_ollama_endpoint, c.default_ollama_endpoint);
    }

    #[tokio::test]
    async fn adapter_factory_and_provider_surface() {
        let mcp = Arc::new(MockMcp);
        let adapter = create_mcp_ai_tools_adapter(mcp).unwrap();
        assert!(adapter.list_providers().is_empty());

        let mut cfg = McpAiToolsConfig::default();
        cfg = cfg.with_provider("openai".to_string(), ProviderSettings::default_openai());
        let mcp2 = Arc::new(MockMcp);
        let adapter2 = McpAiToolsAdapter::new(mcp2, cfg).unwrap();
        assert!(adapter2.list_providers().contains(&"openai".to_string()));
        assert!(adapter2.get_provider_capabilities("openai").is_none());
        let _ = adapter2.create_chat_request();
    }

    #[tokio::test]
    async fn send_chat_unknown_provider_errors() {
        let mcp = Arc::new(MockMcp);
        let mut cfg = McpAiToolsConfig::default();
        cfg = cfg.with_provider("openai".to_string(), ProviderSettings::default_openai());
        let adapter = McpAiToolsAdapter::new(mcp, cfg).unwrap();
        let req = adapter.create_chat_request();
        let err = adapter.send_chat_request("unknown", req).await.unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("Provider not found") || msg.contains("Unknown provider"),
            "unexpected error: {msg}"
        );
    }

    #[tokio::test]
    async fn streaming_and_generate_response_errors() {
        let mcp = Arc::new(MockMcp);
        let adapter = create_mcp_ai_tools_adapter(mcp).unwrap();
        let req = adapter.create_chat_request();
        let Err(e1) = adapter
            .send_streaming_chat_request("openai", req.clone())
            .await
        else {
            panic!("expected streaming to be unimplemented");
        };
        assert!(e1.to_string().contains("Streaming") || e1.to_string().contains("not yet"));

        let Err(e2) = adapter
            .generate_response("c1", Some("m".to_string()), None, None)
            .await
        else {
            panic!("expected generate_response to be unimplemented");
        };
        assert!(e2.to_string().contains("not yet implemented"));
    }

    #[tokio::test]
    async fn send_chat_registry_key_mismatches_known_provider_id() {
        let mcp = Arc::new(MockMcp);
        let mut cfg = McpAiToolsConfig::default();
        cfg = cfg.with_provider(
            "local".to_string(),
            ProviderSettings {
                id: "custom-vendor".to_string(),
                name: "Custom".to_string(),
                config: HashMap::new(),
                models: vec![],
            },
        );
        let adapter = McpAiToolsAdapter::new(mcp, cfg).unwrap();
        let req = adapter.create_chat_request();
        let err = adapter
            .send_chat_request("local", req)
            .await
            .expect_err("unknown provider branch");
        assert!(err.to_string().contains("Unknown provider"));
    }

    #[tokio::test]
    async fn create_mcp_ai_tools_adapter_with_config_registers_providers() {
        let mcp = Arc::new(MockMcp);
        let cfg = McpAiToolsConfig::default()
            .with_provider("p9".to_string(), ProviderSettings::default_openai());
        let adapter = create_mcp_ai_tools_adapter_with_config(mcp, cfg).unwrap();
        assert!(adapter.list_providers().contains(&"p9".to_string()));
    }

    #[test]
    fn adapter_debug_format_is_stable_enough_for_observability() {
        let mcp = Arc::new(MockMcp);
        let adapter = create_mcp_ai_tools_adapter(mcp).unwrap();
        let s = format!("{adapter:?}");
        assert!(s.contains("McpAiToolsAdapter"));
        assert!(s.contains("timeout_ms"));
    }

    #[test]
    fn mcp_ai_tools_config_default_has_expected_baseline() {
        let c = McpAiToolsConfig::default();
        assert_eq!(c.timeout_ms, 30_000);
        assert!(c.streaming);
        assert!(
            c.default_ollama_endpoint.contains("localhost")
                || c.default_ollama_endpoint.contains("127.0.0.1")
                || c.default_ollama_endpoint.starts_with("http"),
            "endpoint: {}",
            c.default_ollama_endpoint
        );
    }

    #[tokio::test]
    async fn streaming_error_message_mentions_batch_alternative() {
        let mcp = Arc::new(MockMcp);
        let adapter = create_mcp_ai_tools_adapter(mcp).unwrap();
        let req = adapter.create_chat_request();
        let Err(e) = adapter.send_streaming_chat_request("any", req).await else {
            panic!("expected streaming to be unimplemented");
        };
        let msg = e.to_string();
        assert!(
            msg.contains("send_chat_request") || msg.contains("batch"),
            "unexpected: {msg}"
        );
    }
}
