//! Anthropic API client implementation
//!
//! **DEPRECATED**: This module uses reqwest for direct HTTP calls, which pulls in the `ring` C dependency.
//! 
//! **Migration Path**: Use `capability_ai::AiClient` instead, which delegates HTTP to Songbird via Unix sockets.
//! 
//! See: `docs/CAPABILITY_AI_MIGRATION_GUIDE.md` for migration instructions.
//!
//! This module will be removed in a future release once all usages are migrated.

#![deprecated(
    since = "1.4.1",
    note = "Use capability_ai::AiClient instead. See docs/CAPABILITY_AI_MIGRATION_GUIDE.md"
)]

use std::any::Any;
use std::time::Duration;

use async_trait::async_trait;
use bytes::Bytes;
use futures::StreamExt;
use reqwest::Client;
use secrecy::{ExposeSecret, Secret, SecretString};
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

use crate::common::{
    capability::{
        AICapabilities, CostTier, ModelRegistry, ModelType, RoutingPreferences, TaskType,
    },
    AIClient, ChatChoice, ChatChoiceChunk, ChatMessage, ChatRequest, ChatResponse,
    ChatResponseChunk, ChatResponseStream, MessageRole, UsageInfo,
};
use crate::{error::Error, Result};

/// Anthropic API request structures
#[derive(Debug, Clone, Serialize)]
struct AnthropicRequest {
    model: String,
    messages: Vec<AnthropicMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_k: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_sequences: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

/// Anthropic API response structures
#[derive(Debug, Clone, Deserialize)]
struct AnthropicResponse {
    #[serde(rename = "type")]
    #[allow(dead_code)]
    response_type: String,
    id: String,
    model: String,
    #[allow(dead_code)]
    role: String,
    content: Vec<AnthropicContent>,
    stop_reason: Option<String>,
    #[allow(dead_code)]
    stop_sequence: Option<String>,
    usage: AnthropicUsage,
}

#[derive(Debug, Clone, Deserialize)]
struct AnthropicContent {
    #[serde(rename = "type")]
    content_type: String,
    text: String,
}

#[derive(Debug, Clone, Deserialize)]
struct AnthropicUsage {
    input_tokens: u32,
    output_tokens: u32,
}

/// Anthropic client configuration
#[derive(Debug, Clone)]
pub struct AnthropicConfig {
    /// Default model to use
    pub default_model: String,
    /// API base URL
    pub api_base: String,
    /// Rate limit in requests per minute
    pub rate_limit: u32,
    /// Organization ID (if any)
    pub organization: Option<String>,
    /// Timeout in seconds
    pub timeout_seconds: u64,
}

impl Default for AnthropicConfig {
    fn default() -> Self {
        Self {
            default_model: "claude-3-opus-20240229".to_string(),
            api_base: "https://api.anthropic.com/v1".to_string(),
            rate_limit: 40,
            organization: None,
            timeout_seconds: 60,
        }
    }
}

/// Anthropic API client
#[derive(Debug)]
pub struct AnthropicClient {
    /// The API key for authentication
    api_key: SecretString,
    /// Configuration
    config: AnthropicConfig,
    /// HTTP client
    client: Client,
}

impl AnthropicClient {
    /// Create a new Anthropic client with the given API key
    pub fn new(api_key: impl Into<String>) -> Self {
        Self::with_config(api_key, AnthropicConfig::default())
    }

    /// Create a new Anthropic client with a custom configuration
    pub fn with_config(api_key: impl Into<String>, config: AnthropicConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            api_key: Secret::new(api_key.into()),
            config,
            client,
        }
    }

    /// Create a new Anthropic client with a custom endpoint
    pub fn with_endpoint(api_key: impl Into<String>, endpoint: String) -> Self {
        let config = AnthropicConfig {
            api_base: endpoint,
            ..Default::default()
        };
        Self::with_config(api_key, config)
    }

    /// Convert ChatRequest to AnthropicRequest
    fn convert_chat_request(&self, request: &ChatRequest) -> AnthropicRequest {
        let mut messages = Vec::new();
        let mut system_message = None;

        for message in &request.messages {
            if let Some(content) = &message.content {
                match message.role {
                    MessageRole::System => {
                        system_message = Some(content.clone());
                    }
                    MessageRole::User => {
                        messages.push(AnthropicMessage {
                            role: "user".to_string(),
                            content: content.clone(),
                        });
                    }
                    MessageRole::Assistant => {
                        messages.push(AnthropicMessage {
                            role: "assistant".to_string(),
                            content: content.clone(),
                        });
                    }
                    MessageRole::Tool => {
                        // Skip tool messages for now
                        continue;
                    }
                    MessageRole::Function => {
                        // Function messages are deprecated, treat as tool
                        // Skip for now
                        continue;
                    }
                }
            }
        }

        let model = request.model.as_ref().unwrap_or(&self.config.default_model);

        AnthropicRequest {
            model: model.clone(),
            messages,
            max_tokens: request.parameters.as_ref().and_then(|p| p.max_tokens),
            temperature: request.parameters.as_ref().and_then(|p| p.temperature),
            top_p: request.parameters.as_ref().and_then(|p| p.top_p),
            top_k: request
                .parameters
                .as_ref()
                .and_then(|p| p.top_k.map(|k| k as u32)),
            stop_sequences: request.parameters.as_ref().and_then(|p| p.stop.clone()),
            system: system_message,
            stream: None,
        }
    }

    /// Convert AnthropicResponse to ChatResponse
    fn convert_anthropic_response(&self, response: AnthropicResponse) -> ChatResponse {
        let content = response
            .content
            .into_iter()
            .map(|c| c.text)
            .collect::<Vec<_>>()
            .join("\n");

        let choice = ChatChoice {
            index: 0,
            role: MessageRole::Assistant,
            content: if content.is_empty() {
                None
            } else {
                Some(content)
            },
            finish_reason: response.stop_reason,
            tool_calls: None,
        };

        let usage = UsageInfo {
            prompt_tokens: response.usage.input_tokens,
            completion_tokens: response.usage.output_tokens,
            total_tokens: response.usage.input_tokens + response.usage.output_tokens,
        };

        ChatResponse {
            id: response.id,
            choices: vec![choice],
            model: response.model,
            usage: Some(usage),
        }
    }

    /// Get capabilities for this Anthropic client based on model
    fn get_model_capabilities(&self, model_id: &str) -> AICapabilities {
        // Try to get capabilities from the model registry
        let registry = ModelRegistry::global();
        if let Some(capabilities) = registry.get_model_capabilities("anthropic", model_id) {
            return capabilities;
        }

        // Fall back to default capabilities for the model if not in registry
        let mut capabilities = AICapabilities {
            supports_streaming: true,
            ..Default::default()
        };

        // Common capabilities for all Anthropic models
        capabilities.supports_streaming = true;

        // Pattern match Claude model versions
        if model_id.contains("claude-3-opus")
            || model_id.contains("claude-3-sonnet")
            || model_id.contains("claude-3-haiku")
        {
            capabilities.add_model_type(ModelType::LargeLanguageModel);
            capabilities.add_task_type(TaskType::TextGeneration);
            capabilities.add_task_type(TaskType::ImageUnderstanding);
            capabilities.with_function_calling(true);
            capabilities.with_tool_use(true);
            capabilities.with_max_context_size(200_000);
        } else if model_id.contains("claude-2") || model_id.contains("claude-1") {
            capabilities.add_model_type(ModelType::LargeLanguageModel);
            capabilities.add_task_type(TaskType::TextGeneration);
            capabilities.with_function_calling(false);
            capabilities.with_tool_use(false);
            capabilities.with_max_context_size(100_000);
        } else {
            // Default for unknown models
            capabilities.add_model_type(ModelType::LargeLanguageModel);
            capabilities.add_task_type(TaskType::TextGeneration);
            capabilities.with_max_context_size(100_000);
            capabilities.with_function_calling(false);
            capabilities.with_tool_use(false);
        }

        capabilities
    }
}

#[async_trait]
impl AIClient for AnthropicClient {
    fn provider_name(&self) -> &str {
        "anthropic"
    }

    fn default_model(&self) -> &str {
        &self.config.default_model
    }

    async fn list_models(&self) -> Result<Vec<String>> {
        // Try to get models from the registry first
        let registry = ModelRegistry::global();
        let models = registry.get_provider_models("anthropic");

        if !models.is_empty() {
            return Ok(models);
        }

        // Anthropic doesn't have a models endpoint, so we return a hard-coded list
        Ok(vec![
            "claude-3-opus-20240229".to_string(),
            "claude-3-sonnet-20240229".to_string(),
            "claude-3-haiku-20240307".to_string(),
            "claude-2.1".to_string(),
        ])
    }

    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
        let anthropic_request = self.convert_chat_request(&request);

        let url = format!("{}/messages", self.config.api_base);

        let mut req_builder = self
            .client
            .post(&url)
            .header("x-api-key", self.api_key.expose_secret())
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json");

        if let Some(org) = &self.config.organization {
            req_builder = req_builder.header("anthropic-organization", org);
        }

        let response = req_builder
            .json(&anthropic_request)
            .send()
            .await
            .map_err(|e| Error::Network(format!("Failed to send request: {e}")))?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(Error::Provider(format!(
                "Anthropic API error: {error_text}"
            )));
        }

        let anthropic_response: AnthropicResponse = response
            .json()
            .await
            .map_err(|e| Error::Parse(format!("Failed to parse response: {e}")))?;

        Ok(self.convert_anthropic_response(anthropic_response))
    }

    async fn chat_stream(&self, request: ChatRequest) -> Result<ChatResponseStream> {
        let mut anthropic_request = self.convert_chat_request(&request);
        anthropic_request.stream = Some(true);

        let url = format!("{}/messages", self.config.api_base);

        let mut req_builder = self
            .client
            .post(&url)
            .header("x-api-key", self.api_key.expose_secret())
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .header("accept", "text/event-stream");

        if let Some(org) = &self.config.organization {
            req_builder = req_builder.header("anthropic-organization", org);
        }

        let response = req_builder
            .json(&anthropic_request)
            .send()
            .await
            .map_err(|e| Error::Network(format!("Failed to send request: {e}")))?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(Error::Provider(format!(
                "Anthropic API error: {error_text}"
            )));
        }

        let stream =
            response
                .bytes_stream()
                .map(move |chunk: std::result::Result<Bytes, _>| {
                    match chunk {
                        Ok(bytes) => {
                            // Parse SSE format
                            let chunk_str = String::from_utf8_lossy(&bytes);
                            if let Some(json_start) = chunk_str.find("data: ") {
                                let json_str = &chunk_str[json_start + 6..];
                                match serde_json::from_str::<AnthropicResponse>(json_str.trim()) {
                                    Ok(anthropic_response) => {
                                        // Enhanced content processing with type validation and filtering
                                        let processed_content = anthropic_response
                                            .content
                                            .into_iter()
                                            .filter_map(|content| {
                                                // Validate and process content based on type
                                                match content.content_type.as_str() {
                                                    "text" => {
                                                        debug!("📝 Processing text content: {} chars", content.text.len());
                                                        Some(content.text)
                                                    }
                                                    "image" => {
                                                        debug!("🖼️ Skipping image content (not supported in text response)");
                                                        None // Skip image content in text responses
                                                    }
                                                    "tool_use" => {
                                                        debug!("🔧 Processing tool use content");
                                                        // For tool use, we might want to format differently
                                                        Some(format!("[TOOL_USE] {}", content.text))
                                                    }
                                                    unknown_type => {
                                                        warn!("⚠️ Unknown Anthropic content type: '{}', including as text", unknown_type);
                                                        Some(format!("[{}] {}", unknown_type.to_uppercase(), content.text))
                                                    }
                                                }
                                            })
                                            .collect::<Vec<_>>();

                                        let text_content = if processed_content.is_empty() {
                                            warn!("⚠️ No valid text content found in Anthropic response");
                                            String::new()
                                        } else {
                                            processed_content.join("\n")
                                        };

                                        debug!("✅ Successfully processed {} content blocks into response", processed_content.len());

                                        let choices = vec![ChatChoiceChunk {
                                            index: 0,
                                            delta: ChatMessage {
                                                role: MessageRole::Assistant,
                                                content: Some(text_content),
                                                name: None,
                                                tool_calls: None,
                                                tool_call_id: None,
                                            },
                                            finish_reason: anthropic_response.stop_reason,
                                        }];

                                        Ok(ChatResponseChunk {
                                            id: anthropic_response.id,
                                            choices,
                                            model: anthropic_response.model,
                                        })
                                    }
                                    Err(e) => Err(Error::Parse(format!(
                                        "Failed to parse streaming chunk: {e}"
                                    ))),
                                }
                            } else {
                                Err(Error::Parse("Invalid SSE format".to_string()))
                            }
                        }
                        Err(e) => Err(Error::Network(format!("Stream error: {e}"))),
                    }
                });

        Ok(Box::pin(stream))
    }

    async fn get_capabilities(&self, model: &str) -> crate::Result<AICapabilities> {
        Ok(self.get_model_capabilities(model))
    }

    async fn is_available(&self) -> bool {
        // Test availability by making a simple API call
        let test_request = ChatRequest::new()
            .add_user("Hi")
            .with_model("claude-3-haiku-20240307");

        (self.chat(test_request).await).is_ok()
    }

    fn capabilities(&self) -> AICapabilities {
        self.get_model_capabilities(&self.config.default_model)
    }

    fn routing_preferences(&self) -> RoutingPreferences {
        // Try to get model capabilities from the registry
        let registry = ModelRegistry::global();
        if let Some(capabilities) =
            registry.get_model_capabilities("anthropic", &self.config.default_model)
        {
            // Extract cost tier from registry data
            let cost_tier = if capabilities.cost_metrics.is_free {
                CostTier::Free
            } else if let Some(input_cost) = capabilities.cost_metrics.cost_per_1k_input_tokens {
                if input_cost > 0.01 {
                    CostTier::High
                } else if input_cost > 0.002 {
                    CostTier::Medium
                } else {
                    CostTier::Low
                }
            } else {
                CostTier::Medium
            };

            return RoutingPreferences {
                priority: 75,
                allows_forwarding: true,
                handles_sensitive_data: true,
                geo_constraints: None,
                cost_tier,
                prefers_local: false,
                cost_sensitivity: 0.7,
                performance_priority: 0.8,
            };
        }

        // Determine cost tier based on model name if not in registry
        let cost_tier = if self.config.default_model.contains("claude-3-opus") {
            CostTier::High
        } else if self.config.default_model.contains("claude-3-sonnet") {
            CostTier::Medium
        } else if self.config.default_model.contains("claude-3-haiku") {
            CostTier::Low
        } else {
            CostTier::Medium
        };

        RoutingPreferences {
            priority: 70,
            allows_forwarding: true,
            handles_sensitive_data: true,
            geo_constraints: None,
            cost_tier,
            prefers_local: false,
            cost_sensitivity: 0.7,
            performance_priority: 0.8,
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
