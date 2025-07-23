//! OpenAI API client implementation
//!
//! This module provides an implementation of the AI client interface for OpenAI's API.

use std::sync::Arc;
use std::time::Duration;

use reqwest::{Client, Response, StatusCode};
use secrecy::{ExposeSecret, Secret};
use tracing::{debug, warn};

use crate::{
    common::{
        capability::{
            AICapabilities, CostTier, ModelRegistry, ModelType, RoutingPreferences, TaskType,
        },
        AIClient, ChatChoice, ChatChoiceChunk, ChatMessage, ChatRequest, ChatResponse,
        ChatResponseChunk, ChatResponseStream, MessageRole, RateLimiter, UsageInfo,
    },
    error::Error,
    Result,
};

pub mod models;
pub mod types;

// Re-exports
pub use models::{OpenAIModel, DEFAULT_MODEL};
pub use types::{
    OpenAIChatRequest, OpenAIChatResponse, OpenAIChatStreamResponse, OpenAIErrorResponse,
    OpenAIMessage, OpenAIMessageRole, OpenAIResponseFormat, OpenAIToolCall,
};

/// OpenAI client configuration
#[derive(Debug, Clone)]
pub struct OpenAIConfig {
    /// Default model to use
    pub default_model: String,
    /// API base URL
    pub api_base: String,
    /// Rate limit in requests per minute
    pub rate_limit: u32,
    /// Whether to retry on rate limits
    pub retry_on_rate_limit: bool,
    /// Maximum number of retries
    pub max_retries: u32,
    /// Retry delay in milliseconds
    pub retry_delay_ms: u64,
    /// Organization ID
    pub organization: Option<String>,
    /// Timeout in seconds
    pub timeout_seconds: u64,
}

impl Default for OpenAIConfig {
    fn default() -> Self {
        Self {
            default_model: DEFAULT_MODEL.to_string(),
            api_base: "https://api.openai.com/v1".to_string(),
            rate_limit: 60,
            retry_on_rate_limit: true,
            max_retries: 3,
            retry_delay_ms: 2000,
            organization: None,
            timeout_seconds: 60,
        }
    }
}

/// OpenAI API client
#[derive(Clone, Debug)]
pub struct OpenAIClient {
    /// API key
    api_key: Secret<String>,
    /// HTTP client
    client: Client,
    /// API base URL
    base_url: String,
    /// Rate limiter
    pub rate_limiter: Arc<RateLimiter>,
    /// Configuration
    pub config: OpenAIConfig,
}

impl OpenAIClient {
    /// Create a new OpenAI client with a custom configuration
    pub fn with_config(api_key: impl Into<String>, config: OpenAIConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()
            .map_err(|e| Error::Configuration(format!("Failed to create HTTP client: {e}")))?;

        // Create rate limiter from config
        let rate_limiter = Arc::new(RateLimiter::default_with_rate(config.rate_limit, "openai"));

        Ok(Self {
            client,
            api_key: Secret::new(api_key.into()),
            base_url: config.api_base.clone(),
            rate_limiter,
            config,
        })
    }

    /// Create a new OpenAI client with default configuration
    pub fn new(api_key: impl Into<String>) -> Result<Self> {
        Self::with_config(api_key, OpenAIConfig::default())
    }

    /// Convert a provider-agnostic chat request to an OpenAI-specific request
    fn prepare_request(&self, request: ChatRequest) -> OpenAIChatRequest {
        let model = request
            .model
            .unwrap_or_else(|| self.config.default_model.clone());

        let openai_request = OpenAIChatRequest {
            model,
            messages: request.messages,
            temperature: request.parameters.as_ref().and_then(|p| p.temperature),
            top_p: request.parameters.as_ref().and_then(|p| p.top_p),
            max_tokens: request.parameters.as_ref().and_then(|p| p.max_tokens),
            stream: request
                .parameters
                .as_ref()
                .and_then(|p| p.stream)
                .unwrap_or(false),
            tools: request.tools.map(|tools| {
                tools
                    .into_iter()
                    .map(|t| crate::common::tool::Tool {
                        tool_type: crate::common::tool::ToolType::Function,
                        function: t.function.map(|f| crate::common::tool::FunctionDefinition {
                            name: f.name,
                            description: f.description,
                            parameters: serde_json::from_value(f.parameters).unwrap_or_else(|_| {
                                crate::common::tool::ParameterSchema {
                                    schema_type: "object".to_string(),
                                    properties: None,
                                    required: None,
                                    items: None,
                                }
                            }),
                        }),
                    })
                    .collect()
            }),
            tool_choice: None,
            frequency_penalty: request
                .parameters
                .as_ref()
                .and_then(|p| p.frequency_penalty),
            presence_penalty: request.parameters.as_ref().and_then(|p| p.presence_penalty),
            response_format: None,
            user: None,
        };

        openai_request
    }

    /// Build request headers
    fn build_headers(&self) -> Result<reqwest::header::HeaderMap> {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            reqwest::header::HeaderValue::from_static("application/json"),
        );

        let auth_header = reqwest::header::HeaderValue::from_str(&format!(
            "Bearer {}",
            self.api_key.expose_secret()
        ))
        .map_err(|e| Error::Configuration(format!("Invalid API key format: {e}")))?;
        headers.insert(reqwest::header::AUTHORIZATION, auth_header);

        // Add organization header if specified
        if let Some(org) = &self.config.organization {
            let org_header = reqwest::header::HeaderValue::from_str(org).map_err(|e| {
                Error::Configuration(format!("Invalid organization ID format: {e}"))
            })?;
            headers.insert("OpenAI-Organization", org_header);
        }

        Ok(headers)
    }

    /// Send a request to the OpenAI API with rate limiting
    async fn send_request(&self, request: &OpenAIChatRequest) -> Result<Response> {
        // Use the rate limiter
        self.rate_limiter
            .execute(async {
                let url = format!("{}/chat/completions", self.base_url);
                let response = self
                    .client
                    .post(&url)
                    .headers(self.build_headers()?)
                    .json(request)
                    .send()
                    .await
                    .map_err(|e| Error::Streaming(e.to_string()))?;

                if !response.status().is_success() {
                    let status = response.status();
                    // Handle potential rate limit errors from OpenAI
                    if status == StatusCode::TOO_MANY_REQUESTS {
                        return Err(Error::RateLimit("OpenAI rate limit exceeded".to_string()));
                    }

                    let error = response
                        .json::<serde_json::Value>()
                        .await
                        .map_err(|e| Error::Streaming(e.to_string()))?;
                    return Err(Error::Streaming(error.to_string()));
                }

                Ok(response)
            })
            .await
    }

    /// Get capabilities for this OpenAI client based on model
    fn get_model_capabilities(&self, model: &str) -> AICapabilities {
        // Try to get capabilities from the model registry
        let registry = crate::common::capability::ModelRegistry::global();
        if let Some(capabilities) = registry.get_model_capabilities("openai", model) {
            return capabilities;
        }

        // Fall back to default capabilities for the model if not in registry
        let mut capabilities = AICapabilities {
            supports_streaming: true,
            ..Default::default()
        };

        // Common capabilities for all models
        capabilities.supports_streaming = true;

        // Add model specific capabilities based on name patterns
        if model.starts_with("gpt-4") {
            capabilities.add_model_type(ModelType::LargeLanguageModel);
            capabilities.add_task_type(TaskType::TextGeneration);
            capabilities.with_function_calling(true);
            capabilities.with_tool_use(true);

            // Set context window based on model variant
            if model.contains("128k") {
                capabilities.with_max_context_size(128_000);
            } else if model.contains("32k") {
                capabilities.with_max_context_size(32_768);
            } else {
                capabilities.with_max_context_size(8_192);
            }
        } else if model.starts_with("gpt-3.5") {
            capabilities.add_model_type(ModelType::LargeLanguageModel);
            capabilities.add_task_type(TaskType::TextGeneration);
            capabilities.with_function_calling(true);
            capabilities.with_tool_use(true);

            // Set context window based on model variant
            if model.contains("16k") {
                capabilities.with_max_context_size(16_384);
            } else {
                capabilities.with_max_context_size(4_096);
            }
        } else if model.starts_with("text-embedding") {
            capabilities.add_model_type(ModelType::Embedding);
            capabilities.add_task_type(TaskType::TextEmbedding);
            capabilities.with_function_calling(false);
            capabilities.with_tool_use(false);
            capabilities.supports_streaming = false;
        }

        capabilities
    }
}

#[async_trait::async_trait]
impl AIClient for OpenAIClient {
    fn provider_name(&self) -> &str {
        "openai"
    }

    fn default_model(&self) -> &str {
        &self.config.default_model
    }

    async fn list_models(&self) -> Result<Vec<String>> {
        // Try to get models from the registry first
        let registry = ModelRegistry::global();
        let models = registry.get_provider_models("openai");

        if !models.is_empty() {
            return Ok(models);
        }

        // Fall back to a default list if registry is empty
        Ok(vec![
            "gpt-4".to_string(),
            "gpt-4-32k".to_string(),
            "gpt-3.5-turbo".to_string(),
            "gpt-3.5-turbo-16k".to_string(),
        ])
    }

    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
        let openai_request = self.prepare_request(request);

        let response = self.send_request(&openai_request).await?;
        let openai_response: OpenAIChatResponse = response.json().await?;

        let usage = openai_response.usage.map(|u| UsageInfo {
            prompt_tokens: u.prompt_tokens,
            completion_tokens: u.completion_tokens,
            total_tokens: u.total_tokens,
        });

        Ok(ChatResponse {
            id: openai_response.id,
            model: openai_response.model,
            choices: {
                let mut processed_choices = Vec::new();
                for c in openai_response.choices {
                    // Enhanced tool calls processing with validation and logging for AI integration
                    let validated_tool_calls = if let Some(tool_calls) = &c.message.tool_calls {
                        let mut validated_calls = Vec::new();
                        for tool_call in tool_calls {
                            // Validate and log tool call for AI coordination
                            if !tool_call.id.is_empty() && !tool_call.name.is_empty() {
                                debug!(
                                    "🔧 Validated tool call: {} -> {} with args: {:?}",
                                    tool_call.id, tool_call.name, tool_call.arguments
                                );
                                validated_calls.push(tool_call.clone());
                            } else {
                                warn!(
                                    "⚠️ Invalid tool call detected - missing id or name: {:?}",
                                    tool_call
                                );
                                // Skip invalid tool calls to prevent AI coordination issues
                            }
                        }
                        if !validated_calls.is_empty() {
                            debug!(
                                "✅ Processed {} valid tool calls for AI integration",
                                validated_calls.len()
                            );
                            Some(validated_calls)
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    processed_choices.push(ChatChoice {
                        index: c.index as usize,
                        role: MessageRole::Assistant,
                        content: c.message.content,
                        finish_reason: c.finish_reason,
                        tool_calls: validated_tool_calls,
                    });
                }
                processed_choices
            },
            usage,
        })
    }

    async fn chat_stream(&self, request: ChatRequest) -> Result<ChatResponseStream> {
        // For now, just convert the regular response to a stream
        use futures::stream;
        let response = self.chat(request).await?;
        let chunk = ChatResponseChunk {
            id: response.id,
            model: response.model,
            choices: vec![ChatChoiceChunk {
                index: 0,
                delta: ChatMessage {
                    role: MessageRole::Assistant,
                    content: response.choices[0].content.clone(),
                    name: None,
                    tool_calls: None,
                    tool_call_id: None,
                },
                finish_reason: response.choices[0].finish_reason.clone(),
            }],
        };
        let stream = stream::once(async move { Ok(chunk) });
        Ok(Box::pin(stream))
    }

    async fn get_capabilities(&self, model: &str) -> Result<AICapabilities> {
        Ok(self.get_model_capabilities(model))
    }

    async fn is_available(&self) -> bool {
        // Try a simple API call to check if the service is available
        let test_request = OpenAIChatRequest {
            model: self.config.default_model.clone(),
            messages: vec![ChatMessage {
                role: MessageRole::User,
                content: Some("test".to_string()),
                name: None,
                tool_calls: None,
                tool_call_id: None,
            }],
            max_tokens: Some(1),
            temperature: Some(0.0),
            top_p: None,
            stream: false,
            presence_penalty: None,
            frequency_penalty: None,
            user: None,
            response_format: None,
            tools: None,
            tool_choice: None,
        };

        match self.send_request(&test_request).await {
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    /// Get capabilities of this AI client
    fn capabilities(&self) -> AICapabilities {
        self.get_model_capabilities(&self.config.default_model)
    }

    /// Get routing preferences for this client
    fn routing_preferences(&self) -> RoutingPreferences {
        // Try to get model capabilities from the registry
        let registry = ModelRegistry::global();
        if let Some(model_capabilities) =
            registry.get_model_capabilities("openai", &self.config.default_model)
        {
            let cost_tier = {
                let metrics = &model_capabilities.cost_metrics;
                if metrics.is_free {
                    CostTier::Free
                } else if let Some(input_cost) = metrics.cost_per_1k_input_tokens {
                    if input_cost > 0.02 {
                        CostTier::High
                    } else if input_cost > 0.005 {
                        CostTier::Medium
                    } else {
                        CostTier::Low
                    }
                } else {
                    CostTier::Medium
                }
            };

            return RoutingPreferences {
                priority: 70,
                allows_forwarding: true,
                handles_sensitive_data: false,
                geo_constraints: None,
                cost_tier,
                prefers_local: false,
                cost_sensitivity: 0.7,
                performance_priority: 0.8,
            };
        }

        // Determine cost tier based on model name if not in registry
        let cost_tier = if self.config.default_model.starts_with("gpt-4") {
            CostTier::High
        } else if self.config.default_model.starts_with("gpt-3.5") {
            CostTier::Medium
        } else if self.config.default_model.contains("text-embedding") {
            CostTier::Low
        } else {
            CostTier::Medium
        };

        RoutingPreferences {
            priority: 60,
            allows_forwarding: true,
            handles_sensitive_data: false,
            geo_constraints: None,
            cost_tier,
            prefers_local: false,
            cost_sensitivity: 0.7,
            performance_priority: 0.8,
        }
    }
}
