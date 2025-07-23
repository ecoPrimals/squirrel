//! Google Gemini API client implementation
//!
//! This module provides an implementation of the AI client interface for Google's Gemini API.

use std::any::Any;
use std::time::Duration;

use async_trait::async_trait;
use reqwest::{Client, Response};
use secrecy::{ExposeSecret, Secret, SecretString};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use futures::StreamExt;
use bytes::Bytes;
use tracing::{debug, error, info, warn};

use crate::common::{
    capability::{
        AICapabilities, CostTier, ModelRegistry, ModelType, RoutingPreferences, TaskType,
    },
    AIClient, ChatChoice, ChatChoiceChunk, ChatMessage, ChatRequest, ChatResponse,
    ChatResponseChunk, ChatResponseStream, MessageRole, UsageInfo,
};
use crate::{error::Error, Result};

/// Gemini API request structures
#[derive(Debug, Clone, Serialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    generation_config: Option<GeminiGenerationConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    safety_settings: Option<Vec<GeminiSafetySetting>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GeminiContent {
    parts: Vec<GeminiPart>,
    #[serde(skip_serializing_if = "Option::is_none")]
    role: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GeminiPart {
    text: String,
}

#[derive(Debug, Clone, Serialize)]
struct GeminiGenerationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_k: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_output_tokens: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_sequences: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize)]
struct GeminiSafetySetting {
    category: String,
    threshold: String,
}

/// Gemini API response structures
#[derive(Debug, Clone, Deserialize)]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
    #[serde(rename = "usageMetadata")]
    usage_metadata: Option<GeminiUsageMetadata>,
}

#[derive(Debug, Clone, Deserialize)]
struct GeminiCandidate {
    content: GeminiContent,
    #[serde(rename = "finishReason")]
    finish_reason: Option<String>,
    index: Option<i32>,
    #[serde(rename = "safetyRatings")]
    safety_ratings: Option<Vec<GeminiSafetyRating>>,
}

#[derive(Debug, Clone, Deserialize)]
struct GeminiSafetyRating {
    category: String,
    probability: String,
}

#[derive(Debug, Clone, Deserialize)]
struct GeminiUsageMetadata {
    #[serde(rename = "promptTokenCount")]
    prompt_token_count: Option<i32>,
    #[serde(rename = "candidatesTokenCount")]
    candidates_token_count: Option<i32>,
    #[serde(rename = "totalTokenCount")]
    total_token_count: Option<i32>,
}

/// Gemini client configuration
#[derive(Debug, Clone)]
pub struct GeminiConfig {
    /// Default model to use
    pub default_model: String,
    /// API base URL
    pub api_base: String,
    /// Rate limit in requests per minute
    pub rate_limit: u32,
    /// Timeout in seconds
    pub timeout_seconds: u64,
}

impl Default for GeminiConfig {
    fn default() -> Self {
        Self {
            default_model: "gemini-pro".to_string(),
            api_base: "https://generativelanguage.googleapis.com/v1beta".to_string(),
            rate_limit: 60,
            timeout_seconds: 60,
        }
    }
}

/// Gemini API client
#[derive(Debug)]
pub struct GeminiClient {
    /// The API key for authentication
    api_key: SecretString,
    /// Configuration
    config: GeminiConfig,
    /// HTTP client
    client: Client,
}

impl GeminiClient {
    /// Create a new Gemini client with the given API key
    pub fn new(api_key: impl Into<String>) -> Self {
        Self::with_config(api_key, GeminiConfig::default())
    }

    /// Create a new Gemini client with a custom configuration
    pub fn with_config(api_key: impl Into<String>, config: GeminiConfig) -> Self {
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

    /// Create a new Gemini client with a custom endpoint
    pub fn with_endpoint(api_key: impl Into<String>, endpoint: String) -> Self {
        let config = GeminiConfig {
            api_base: endpoint,
            ..Default::default()
        };
        Self::with_config(api_key, config)
    }

    /// Convert ChatRequest to GeminiRequest
    fn convert_chat_request(&self, request: &ChatRequest) -> GeminiRequest {
        let mut contents = Vec::new();

        for message in &request.messages {
            if let Some(content) = &message.content {
                let role = match message.role {
                    MessageRole::User => Some("user".to_string()),
                    MessageRole::Assistant => Some("model".to_string()),
                    MessageRole::System => {
                        // System messages are merged into the first user message in Gemini
                        Some("user".to_string())
                    }
                    MessageRole::Tool => continue, // Skip tool messages for now
                    MessageRole::Function => continue, // Skip function messages for now
                };

                contents.push(GeminiContent {
                    parts: vec![GeminiPart {
                        text: content.clone(),
                    }],
                    role,
                });
            }
        }

        // Extract generation config from parameters
        let generation_config = request
            .parameters
            .as_ref()
            .map(|params| GeminiGenerationConfig {
                temperature: params.temperature,
                top_p: params.top_p,
                top_k: params.top_k.map(|k| k as i32),
                max_output_tokens: params.max_tokens.map(|t| t as i32),
                stop_sequences: params.stop.clone(),
            });

        GeminiRequest {
            contents,
            generation_config,
            safety_settings: Some(vec![
                GeminiSafetySetting {
                    category: "HARM_CATEGORY_HARASSMENT".to_string(),
                    threshold: "BLOCK_MEDIUM_AND_ABOVE".to_string(),
                },
                GeminiSafetySetting {
                    category: "HARM_CATEGORY_HATE_SPEECH".to_string(),
                    threshold: "BLOCK_MEDIUM_AND_ABOVE".to_string(),
                },
                GeminiSafetySetting {
                    category: "HARM_CATEGORY_SEXUALLY_EXPLICIT".to_string(),
                    threshold: "BLOCK_MEDIUM_AND_ABOVE".to_string(),
                },
                GeminiSafetySetting {
                    category: "HARM_CATEGORY_DANGEROUS_CONTENT".to_string(),
                    threshold: "BLOCK_MEDIUM_AND_ABOVE".to_string(),
                },
            ]),
        }
    }

    /// Convert GeminiResponse to ChatResponse
    fn convert_gemini_response(&self, response: GeminiResponse, model: &str) -> ChatResponse {
        let mut choices = Vec::new();

        for (enum_index, candidate) in response.candidates.into_iter().enumerate() {
            // Validate candidate index consistency for safety
            let candidate_index = candidate.index.unwrap_or(enum_index as i32);
            if candidate_index != enum_index as i32 {
                warn!("🔍 Candidate index mismatch: expected {}, got {} - potential response ordering issue", 
                      enum_index, candidate_index);
            }

            // Enhanced AI safety validation using safety ratings
            let mut safety_passed = true;
            let mut safety_warnings = Vec::new();

            if let Some(safety_ratings) = &candidate.safety_ratings {
                debug!(
                    "🛡️ Evaluating {} safety ratings for candidate {}",
                    safety_ratings.len(),
                    candidate_index
                );

                for safety_rating in safety_ratings {
                    match safety_rating.probability.as_str() {
                        "HIGH" => {
                            let warning =
                                format!("HIGH risk detected for {}", safety_rating.category);
                            warn!("⚠️ AI Safety Alert: {}", warning);
                            safety_warnings.push(warning);
                            safety_passed = false;
                        }
                        "MEDIUM" => {
                            let warning =
                                format!("MEDIUM risk detected for {}", safety_rating.category);
                            info!("🔶 AI Safety Notice: {}", warning);
                            safety_warnings.push(warning);
                        }
                        "LOW" | "NEGLIGIBLE" => {
                            debug!(
                                "✅ AI Safety OK: {} - {}",
                                safety_rating.category, safety_rating.probability
                            );
                        }
                        unknown_level => {
                            warn!(
                                "❓ Unknown AI safety probability level: '{}' for category '{}'",
                                unknown_level, safety_rating.category
                            );
                        }
                    }
                }

                if safety_passed {
                    debug!(
                        "🛡️ All AI safety checks passed for candidate {}",
                        candidate_index
                    );
                } else {
                    error!(
                        "🚨 AI Safety validation failed for candidate {} - {} issues detected",
                        candidate_index,
                        safety_warnings.len()
                    );
                }
            } else {
                debug!("ℹ️ No safety ratings provided for candidate {} - proceeding with standard processing", candidate_index);
            }

            let content = candidate
                .content
                .parts
                .into_iter()
                .map(|part| part.text)
                .collect::<Vec<_>>()
                .join("\n");

            // Enhanced content with safety context if warnings exist
            let final_content = if !safety_warnings.is_empty() {
                let safety_context = format!("[SAFETY_WARNINGS: {}] ", safety_warnings.join(", "));
                Some(format!("{safety_context}{content}"))
            } else if content.is_empty() {
                None
            } else {
                Some(content)
            };

            choices.push(ChatChoice {
                index: candidate_index as usize,
                role: MessageRole::Assistant,
                content: final_content,
                finish_reason: candidate.finish_reason,
                tool_calls: None,
            });
        }

        let usage = response.usage_metadata.map(|usage| UsageInfo {
            prompt_tokens: usage.prompt_token_count.unwrap_or(0) as u32,
            completion_tokens: usage.candidates_token_count.unwrap_or(0) as u32,
            total_tokens: usage.total_token_count.unwrap_or(0) as u32,
        });

        ChatResponse {
            id: format!("gemini-{}", uuid::Uuid::new_v4()),
            choices,
            model: model.to_string(),
            usage,
        }
    }

    /// Get capabilities for this Gemini client based on model
    fn get_model_capabilities(&self, model_id: &str) -> AICapabilities {
        // Try to get capabilities from the model registry
        let registry = ModelRegistry::global();
        if let Some(capabilities) = registry.get_model_capabilities("gemini", model_id) {
            return capabilities;
        }

        // Fall back to default capabilities for the model if not in registry
        let mut capabilities = AICapabilities {
            supports_streaming: true,
            ..Default::default()
        };

        // Common capabilities for all Gemini models
        capabilities.supports_streaming = true;

        if model_id.contains("gemini-pro") {
            capabilities.add_model_type(ModelType::LargeLanguageModel);
            capabilities.add_task_type(TaskType::TextGeneration);
            capabilities.with_function_calling(true);
            capabilities.with_tool_use(false);
            capabilities.with_max_context_size(32_768);
        } else if model_id.contains("gemini-pro-vision") {
            capabilities.add_model_type(ModelType::LargeLanguageModel);
            capabilities.add_model_type(ModelType::MultiModal);
            capabilities.add_task_type(TaskType::TextGeneration);
            capabilities.add_task_type(TaskType::ImageUnderstanding);
            capabilities.with_function_calling(true);
            capabilities.with_tool_use(false);
            capabilities.with_max_context_size(16_384);
        } else if model_id.contains("gemini-ultra") {
            capabilities.add_model_type(ModelType::LargeLanguageModel);
            capabilities.add_model_type(ModelType::MultiModal);
            capabilities.add_task_type(TaskType::TextGeneration);
            capabilities.add_task_type(TaskType::ImageUnderstanding);
            capabilities.with_function_calling(true);
            capabilities.with_tool_use(true);
            capabilities.with_max_context_size(32_768);
        } else {
            // Default for unknown models
            capabilities.add_model_type(ModelType::LargeLanguageModel);
            capabilities.add_task_type(TaskType::TextGeneration);
            capabilities.with_max_context_size(8_192);
            capabilities.with_function_calling(false);
            capabilities.with_tool_use(false);
        }

        capabilities
    }
}

#[async_trait]
impl AIClient for GeminiClient {
    fn provider_name(&self) -> &str {
        "gemini"
    }

    fn default_model(&self) -> &str {
        &self.config.default_model
    }

    async fn list_models(&self) -> Result<Vec<String>> {
        // Try to get models from the registry first
        let registry = ModelRegistry::global();
        let models = registry.get_provider_models("gemini");

        if !models.is_empty() {
            return Ok(models);
        }

        // Fall back to a default list of known Gemini models
        Ok(vec![
            "gemini-pro".to_string(),
            "gemini-pro-vision".to_string(),
            "gemini-1.5-pro".to_string(),
            "gemini-1.5-flash".to_string(),
        ])
    }

    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
        let model = request.model.as_ref().unwrap_or(&self.config.default_model);
        let gemini_request = self.convert_chat_request(&request);

        let url = format!(
            "{}/models/{}:generateContent?key={}",
            self.config.api_base,
            model,
            self.api_key.expose_secret()
        );

        let response = self
            .client
            .post(&url)
            .json(&gemini_request)
            .send()
            .await
            .map_err(|e| Error::Network(format!("Failed to send request: {e}")))?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(Error::Provider(format!("Gemini API error: {error_text}")));
        }

        let gemini_response: GeminiResponse = response
            .json()
            .await
            .map_err(|e| Error::Parse(format!("Failed to parse response: {e}")))?;

        Ok(self.convert_gemini_response(gemini_response, model))
    }

    async fn chat_stream(&self, request: ChatRequest) -> Result<ChatResponseStream> {
        let model = request
            .model
            .clone()
            .unwrap_or_else(|| self.config.default_model.clone());
        let gemini_request = self.convert_chat_request(&request);

        let url = format!(
            "{}/models/{}:streamGenerateContent?key={}",
            self.config.api_base,
            model,
            self.api_key.expose_secret()
        );

        let response = self
            .client
            .post(&url)
            .json(&gemini_request)
            .send()
            .await
            .map_err(|e| Error::Network(format!("Failed to send request: {e}")))?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(Error::Provider(format!("Gemini API error: {error_text}")));
        }

        let stream =
            response
                .bytes_stream()
                .map(move |chunk: std::result::Result<Bytes, _>| {
                    match chunk {
                        Ok(bytes) => {
                            // Parse each chunk as a streaming response
                            match serde_json::from_slice::<GeminiResponse>(&bytes) {
                                Ok(gemini_response) => {
                                    // Inline the response conversion since we can't borrow self
                                    let mut choices = Vec::new();

                                    for (index, candidate) in
                                        gemini_response.candidates.into_iter().enumerate()
                                    {
                                        let content = candidate
                                            .content
                                            .parts
                                            .into_iter()
                                            .map(|part| part.text)
                                            .collect::<Vec<_>>()
                                            .join("\n");

                                        choices.push(ChatChoiceChunk {
                                            index,
                                            delta: ChatMessage {
                                                role: MessageRole::Assistant,
                                                content: if content.is_empty() {
                                                    None
                                                } else {
                                                    Some(content)
                                                },
                                                name: None,
                                                tool_calls: None,
                                                tool_call_id: None,
                                            },
                                            finish_reason: candidate.finish_reason,
                                        });
                                    }

                                    Ok(ChatResponseChunk {
                                        id: format!("gemini-{}", uuid::Uuid::new_v4()),
                                        choices,
                                        model: model.clone(),
                                    })
                                }
                                Err(e) => Err(Error::Parse(format!(
                                    "Failed to parse streaming chunk: {e}"
                                ))),
                            }
                        }
                        Err(e) => Err(Error::Network(format!("Stream error: {e}"))),
                    }
                });

        Ok(Box::pin(stream))
    }

    async fn get_capabilities(&self, model: &str) -> Result<AICapabilities> {
        Ok(self.get_model_capabilities(model))
    }

    async fn is_available(&self) -> bool {
        // Test availability by making a simple API call
        let test_request = ChatRequest::new().add_user("Test").with_model("gemini-pro");

        (self.chat(test_request).await).is_ok()
    }

    fn capabilities(&self) -> AICapabilities {
        self.get_model_capabilities(&self.config.default_model)
    }

    fn routing_preferences(&self) -> RoutingPreferences {
        // Try to get model capabilities from the registry
        let registry = ModelRegistry::global();
        if let Some(capabilities) =
            registry.get_model_capabilities("gemini", &self.config.default_model)
        {
            // Extract cost tier from registry data
            let cost_tier = {
                let metrics = &capabilities.cost_metrics;
                if metrics.is_free {
                    CostTier::Free
                } else if let Some(input_cost) = metrics.cost_per_1k_input_tokens {
                    if input_cost > 0.01 {
                        CostTier::High
                    } else if input_cost > 0.002 {
                        CostTier::Medium
                    } else {
                        CostTier::Low
                    }
                } else {
                    CostTier::Medium
                }
            };

            return RoutingPreferences {
                priority: 65,
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
        let cost_tier = if self.config.default_model.contains("gemini-ultra") {
            CostTier::High
        } else {
            CostTier::Medium
        };

        RoutingPreferences {
            priority: 65,
            allows_forwarding: true,
            handles_sensitive_data: false,
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
