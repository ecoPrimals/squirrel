//! MCP AI Tools Adapter
//!
//! This module provides an adapter for the MCP AI tools.

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use squirrel_ai_tools::common::capability::AICapabilities;
use squirrel_ai_tools::common::{ChatRequest, ChatResponse, ChatResponseChunk, ChatResponseStream};
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
        Self {
            providers: HashMap::new(),
            timeout_ms: 30000,
            streaming: true,
            default_ollama_endpoint: "http://localhost:11434".to_string(),
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
                tracing::debug!("Processing OpenAI chat request");
                // Create OpenAI provider instance
                let provider_config = squirrel_ai_tools::config::ProviderConfig {
                    provider_type: "openai".to_string(),
                    api_key: provider.config.get("api_key").cloned(),
                    base_url: provider.config.get("endpoint").cloned(),
                    default_model: provider.config.get("model").cloned(),
                    settings: HashMap::new(),
                };

                let openai_provider =
                    squirrel_ai_tools::common::providers::OpenAIProvider::new(provider_config)
                        .context("Failed to create OpenAI provider")?;

                use squirrel_ai_tools::common::AIProvider;
                let response = openai_provider
                    .process_chat(&request)
                    .await
                    .context("Failed to process OpenAI chat request")?;

                Ok(response)
            }
            "anthropic" => {
                tracing::debug!("Processing Anthropic chat request");
                // Create Anthropic provider instance
                let provider_config = squirrel_ai_tools::config::ProviderConfig {
                    provider_type: "anthropic".to_string(),
                    api_key: provider.config.get("api_key").cloned(),
                    base_url: provider.config.get("endpoint").cloned(),
                    default_model: provider.config.get("model").cloned(),
                    settings: HashMap::new(),
                };

                let anthropic_provider =
                    squirrel_ai_tools::common::providers::AnthropicProvider::new(provider_config)
                        .context("Failed to create Anthropic provider")?;

                use squirrel_ai_tools::common::AIProvider;
                let response = anthropic_provider
                    .process_chat(&request)
                    .await
                    .context("Failed to process Anthropic chat request")?;

                Ok(response)
            }
            "ollama" => {
                tracing::debug!("Processing Ollama chat request");
                // Create Ollama provider instance
                let provider_config = squirrel_ai_tools::config::ProviderConfig {
                    provider_type: "ollama".to_string(),
                    api_key: provider.config.get("api_key").cloned(),
                    base_url: provider
                        .config
                        .get("endpoint")
                        .cloned()
                        .or_else(|| Some(self.config.default_ollama_endpoint.clone())),
                    default_model: provider.config.get("model").cloned(),
                    settings: HashMap::new(),
                };

                let ollama_provider =
                    squirrel_ai_tools::common::providers::OllamaProvider::new(provider_config)
                        .context("Failed to create Ollama provider")?;

                use squirrel_ai_tools::common::AIProvider;
                let response = ollama_provider
                    .process_chat(&request)
                    .await
                    .context("Failed to process Ollama chat request")?;

                Ok(response)
            }
            _ => {
                tracing::error!("Unknown provider: {}", provider_id);
                Err(anyhow::anyhow!("Unknown provider: {}", provider_id))
            }
        }
    }

    /// Send a streaming chat request
    pub async fn send_streaming_chat_request(
        &self,
        provider_id: &str,
        request: ChatRequest,
    ) -> Result<ChatResponseStream> {
        // Get provider
        let provider = self
            .provider_registry
            .get_provider(provider_id)
            .context(format!("Provider not found: {}", provider_id))?;

        // Route request to appropriate provider implementation
        match provider.id.as_str() {
            "openai" => {
                tracing::debug!("Processing OpenAI streaming chat request");
                // Create OpenAI provider instance
                let provider_config = squirrel_ai_tools::config::ProviderConfig {
                    provider_type: "openai".to_string(),
                    api_key: provider.config.get("api_key").cloned(),
                    base_url: provider.config.get("endpoint").cloned(),
                    default_model: provider.config.get("model").cloned(),
                    settings: HashMap::new(),
                };

                let openai_provider =
                    squirrel_ai_tools::common::providers::OpenAIProvider::new(provider_config)
                        .context("Failed to create OpenAI provider")?;

                // For now, fall back to non-streaming (streaming can be implemented later)
                use squirrel_ai_tools::common::AIProvider;
                let response = openai_provider
                    .process_chat(&request)
                    .await
                    .context("Failed to process OpenAI streaming chat request")?;

                // Convert regular response to streaming
                let stream = futures::stream::once(async move {
                    Ok(ChatResponseChunk {
                        id: response.id.clone(),
                        model: response.model.clone(),
                        choices: response
                            .choices
                            .into_iter()
                            .map(|choice| squirrel_ai_tools::common::ChatChoiceChunk {
                                index: choice.index,
                                delta: squirrel_ai_tools::common::ChatMessage {
                                    role: choice.role,
                                    content: choice.content,
                                    name: None,
                                    tool_calls: choice.tool_calls,
                                    tool_call_id: None,
                                },
                                finish_reason: choice.finish_reason,
                            })
                            .collect(),
                    })
                });

                Ok(Box::pin(stream))
            }
            "anthropic" => {
                tracing::debug!("Processing Anthropic streaming chat request");
                // Create Anthropic provider instance
                let provider_config = squirrel_ai_tools::config::ProviderConfig {
                    provider_type: "anthropic".to_string(),
                    api_key: provider.config.get("api_key").cloned(),
                    base_url: provider.config.get("endpoint").cloned(),
                    default_model: provider.config.get("model").cloned(),
                    settings: HashMap::new(),
                };

                let anthropic_provider =
                    squirrel_ai_tools::common::providers::AnthropicProvider::new(provider_config)
                        .context("Failed to create Anthropic provider")?;

                // For now, fall back to non-streaming (streaming can be implemented later)
                use squirrel_ai_tools::common::AIProvider;
                let response = anthropic_provider
                    .process_chat(&request)
                    .await
                    .context("Failed to process Anthropic streaming chat request")?;

                // Convert regular response to streaming
                let stream = futures::stream::once(async move {
                    Ok(ChatResponseChunk {
                        id: response.id.clone(),
                        model: response.model.clone(),
                        choices: response
                            .choices
                            .into_iter()
                            .map(|choice| squirrel_ai_tools::common::ChatChoiceChunk {
                                index: choice.index,
                                delta: squirrel_ai_tools::common::ChatMessage {
                                    role: choice.role,
                                    content: choice.content,
                                    name: None,
                                    tool_calls: choice.tool_calls,
                                    tool_call_id: None,
                                },
                                finish_reason: choice.finish_reason,
                            })
                            .collect(),
                    })
                });

                Ok(Box::pin(stream))
            }
            "ollama" => {
                tracing::debug!("Processing Ollama streaming chat request");
                // Create Ollama provider instance
                let provider_config = squirrel_ai_tools::config::ProviderConfig {
                    provider_type: "ollama".to_string(),
                    api_key: provider.config.get("api_key").cloned(),
                    base_url: provider
                        .config
                        .get("endpoint")
                        .cloned()
                        .or_else(|| Some(self.config.default_ollama_endpoint.clone())),
                    default_model: provider.config.get("model").cloned(),
                    settings: HashMap::new(),
                };

                let ollama_provider =
                    squirrel_ai_tools::common::providers::OllamaProvider::new(provider_config)
                        .context("Failed to create Ollama provider")?;

                // For now, fall back to non-streaming (streaming can be implemented later)
                use squirrel_ai_tools::common::AIProvider;
                let response = ollama_provider
                    .process_chat(&request)
                    .await
                    .context("Failed to process Ollama streaming chat request")?;

                // Convert regular response to streaming
                let stream = futures::stream::once(async move {
                    Ok(ChatResponseChunk {
                        id: response.id.clone(),
                        model: response.model.clone(),
                        choices: response
                            .choices
                            .into_iter()
                            .map(|choice| squirrel_ai_tools::common::ChatChoiceChunk {
                                index: choice.index,
                                delta: squirrel_ai_tools::common::ChatMessage {
                                    role: choice.role,
                                    content: choice.content,
                                    name: None,
                                    tool_calls: choice.tool_calls,
                                    tool_call_id: None,
                                },
                                finish_reason: choice.finish_reason,
                            })
                            .collect(),
                    })
                });

                Ok(Box::pin(stream))
            }
            _ => {
                tracing::error!("Unknown streaming provider: {}", provider_id);
                Err(anyhow::anyhow!(
                    "Unknown streaming provider: {}",
                    provider_id
                ))
            }
        }
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
