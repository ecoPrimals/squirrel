// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

use std::sync::Arc;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tokio_stream::StreamExt;

use super::{
    ProviderPlugin, ProviderConfig, ProviderType, ModelInfo, ModelCapability,
    HealthStatus, HealthLevel, CostEstimate, CostBreakdown, ProviderStats
};
use crate::common::{
    ChatRequest, ChatResponse, ChatResponseStream, ChatMessage, MessageRole,
    UsageInfo, ChatResponseChunk
};
use crate::common::capability::AICapabilities;
use crate::Result;

/// OpenRouter provider implementation
pub struct OpenRouterProvider {
    client: Client,
    api_key: String,
    base_url: String,
    app_name: String,
    site_url: String,
    models: Vec<ModelInfo>,
    stats: Arc<Mutex<ProviderStats>>,
}

/// OpenRouter API request format
#[derive(Debug, Serialize)]
struct OpenRouterRequest {
    model: String,
    messages: Vec<OpenRouterMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    frequency_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    presence_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}

#[derive(Debug, Serialize)]
struct OpenRouterMessage {
    role: String,
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
}

/// OpenRouter API response format
#[derive(Debug, Deserialize)]
struct OpenRouterResponse {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<OpenRouterChoice>,
    usage: OpenRouterUsage,
}

#[derive(Debug, Deserialize)]
struct OpenRouterChoice {
    index: u32,
    message: OpenRouterMessage,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenRouterUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

/// OpenRouter streaming response chunk
#[derive(Debug, Deserialize)]
struct OpenRouterStreamChunk {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<OpenRouterStreamChoice>,
}

#[derive(Debug, Deserialize)]
struct OpenRouterStreamChoice {
    index: u32,
    delta: OpenRouterDelta,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenRouterDelta {
    role: Option<String>,
    content: Option<String>,
}

/// OpenRouter models API response
#[derive(Debug, Deserialize)]
struct OpenRouterModelsResponse {
    data: Vec<OpenRouterModel>,
}

#[derive(Debug, Deserialize)]
struct OpenRouterModel {
    id: String,
    name: String,
    description: Option<String>,
    context_length: Option<u32>,
    pricing: Option<OpenRouterPricing>,
    top_provider: Option<OpenRouterTopProvider>,
}

#[derive(Debug, Deserialize)]
struct OpenRouterPricing {
    prompt: String,
    completion: String,
}

#[derive(Debug, Deserialize)]
struct OpenRouterTopProvider {
    max_completion_tokens: Option<u32>,
}

impl OpenRouterProvider {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            api_key: String::new(),
            base_url: "https://openrouter.ai/api/v1".to_string(),
            app_name: "Squirrel AI Tools".to_string(),
            site_url: "https://github.com/squirrel-ai/squirrel".to_string(),
            models: Vec::new(),
            stats: Arc::new(Mutex::new(ProviderStats {
                total_requests: 0,
                successful_requests: 0,
                failed_requests: 0,
                average_response_time_ms: 0.0,
                total_tokens_processed: 0,
                total_cost_usd: 0.0,
                uptime_percentage: 100.0,
                last_error: None,
                last_error_time: None,
            })),
        }
    }

    fn convert_messages(&self, messages: &[ChatMessage]) -> Vec<OpenRouterMessage> {
        messages.iter().map(|msg| {
            OpenRouterMessage {
                role: match msg.role {
                    MessageRole::System => "system".to_string(),
                    MessageRole::User => "user".to_string(),
                    MessageRole::Assistant => "assistant".to_string(),
                    MessageRole::Tool => "tool".to_string(),
                },
                content: msg.content.clone().unwrap_or_default(),
                name: msg.name.clone(),
            }
        }).collect()
    }

    async fn fetch_models(&mut self) -> Result<()> {
        let response = self.client
            .get(&format!("{}/models", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("HTTP-Referer", &self.site_url)
            .header("X-Title", &self.app_name)
            .send()
            .await?;

        if response.status().is_success() {
            let models_response: OpenRouterModelsResponse = response.json().await?;
            self.models = models_response.data.into_iter().map(|model| {
                let mut capabilities = vec![
                    ModelCapability::TextGeneration,
                    ModelCapability::ChatCompletion,
                ];

                // Add capabilities based on model name/type
                if model.id.contains("gpt-4") || model.id.contains("claude") {
                    capabilities.push(ModelCapability::Reasoning);
                    capabilities.push(ModelCapability::FunctionCalling);
                }
                
                if model.id.contains("code") || model.id.contains("codex") {
                    capabilities.push(ModelCapability::CodeGeneration);
                    capabilities.push(ModelCapability::Coding);
                }

                ModelInfo {
                    id: model.id.clone(),
                    name: model.name,
                    description: model.description,
                    context_window: model.context_length.map(|c| c as usize),
                    max_output_tokens: model.top_provider
                        .and_then(|p| p.max_completion_tokens)
                        .map(|t| t as usize),
                    input_cost_per_1k_tokens: model.pricing.as_ref()
                        .and_then(|p| p.prompt.parse::<f64>().ok()),
                    output_cost_per_1k_tokens: model.pricing.as_ref()
                        .and_then(|p| p.completion.parse::<f64>().ok()),
                    capabilities,
                    tags: self.extract_tags(&model.id),
                }
            }).collect();
        }

        Ok(())
    }

    fn extract_tags(&self, model_id: &str) -> Vec<String> {
        let mut tags = Vec::new();
        
        if model_id.contains("gpt") {
            tags.push("openai".to_string());
        }
        if model_id.contains("claude") {
            tags.push("anthropic".to_string());
        }
        if model_id.contains("llama") {
            tags.push("meta".to_string());
        }
        if model_id.contains("mistral") {
            tags.push("mistralai".to_string());
        }
        if model_id.contains("gemini") {
            tags.push("google".to_string());
        }
        
        // Add capability tags
        if model_id.contains("code") {
            tags.push("coding".to_string());
        }
        if model_id.contains("instruct") {
            tags.push("instruction-following".to_string());
        }
        if model_id.contains("chat") {
            tags.push("conversational".to_string());
        }
        
        tags
    }

    async fn update_stats(&self, success: bool, response_time_ms: u64, tokens: u64, cost: f64) {
        let mut stats = self.stats.lock().await;
        stats.total_requests += 1;
        
        if success {
            stats.successful_requests += 1;
        } else {
            stats.failed_requests += 1;
        }
        
        // Update average response time
        let total_time = stats.average_response_time_ms * (stats.total_requests - 1) as f64;
        stats.average_response_time_ms = (total_time + response_time_ms as f64) / stats.total_requests as f64;
        
        stats.total_tokens_processed += tokens;
        stats.total_cost_usd += cost;
        
        // Update uptime percentage
        stats.uptime_percentage = (stats.successful_requests as f64 / stats.total_requests as f64) * 100.0;
    }

    async fn estimate_cost(&self, request: &ChatRequest) -> Result<f64> {
        // Get model pricing from OpenRouter API
        let model_id = request.model.as_ref().unwrap_or(&self.base_url);
        
        // Try to get cached model info first
        if let Some(model_info) = self.models.iter()
            .find(|m| m.id == *model_id)
            .cloned() {
            let prompt_tokens = self.estimate_tokens(&request.messages);
            let completion_tokens = request.parameters.as_ref()
                .and_then(|p| p.max_tokens)
                .unwrap_or(150) as u32;
            
            let input_cost = model_info.input_cost_per_1k_tokens.unwrap_or(0.0) * (prompt_tokens as f64 / 1000.0);
            let output_cost = model_info.output_cost_per_1k_tokens.unwrap_or(0.0) * (completion_tokens as f64 / 1000.0);
            
            Ok(input_cost + output_cost)
        } else {
            // Fallback to default pricing if model info not available
            let prompt_tokens = self.estimate_tokens(&request.messages);
            let completion_tokens = request.parameters.as_ref()
                .and_then(|p| p.max_tokens)
                .unwrap_or(150) as u32;
            
            // Use average pricing: $0.002 per 1K input tokens, $0.006 per 1K output tokens
            let input_cost = 0.002 * (prompt_tokens as f64 / 1000.0);
            let output_cost = 0.006 * (completion_tokens as f64 / 1000.0);
            
            Ok(input_cost + output_cost)
        }
    }
    
    /// Estimate token count for messages
    fn estimate_tokens(&self, messages: &[ChatMessage]) -> u32 {
        // Simple token estimation: ~4 characters per token
        let total_chars: usize = messages.iter()
            .map(|msg| msg.content.as_ref().map(|c| c.len()).unwrap_or(0))
            .sum();
        
        (total_chars / 4) as u32
    }
}

#[async_trait]
impl ProviderPlugin for OpenRouterProvider {
    fn name(&self) -> &str {
        "OpenRouter"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn provider_type(&self) -> ProviderType {
        ProviderType::Proxy
    }

    fn supported_models(&self) -> Vec<ModelInfo> {
        self.models.clone()
    }

    async fn initialize(&mut self, config: ProviderConfig) -> Result<()> {
        // Extract configuration
        self.api_key = config.settings.get("api_key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::Error::Configuration(
                "OpenRouter API key is required".to_string()
            ))?
            .to_string();

        if let Some(base_url) = config.settings.get("base_url").and_then(|v| v.as_str()) {
            self.base_url = base_url.to_string();
        }

        if let Some(app_name) = config.settings.get("app_name").and_then(|v| v.as_str()) {
            self.app_name = app_name.to_string();
        }

        if let Some(site_url) = config.settings.get("site_url").and_then(|v| v.as_str()) {
            self.site_url = site_url.to_string();
        }

        // Fetch available models
        self.fetch_models().await?;

        Ok(())
    }

    async fn health_check(&self) -> Result<HealthStatus> {
        let start_time = std::time::Instant::now();
        
        let response = self.client
            .get(&format!("{}/models", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("HTTP-Referer", &self.site_url)
            .header("X-Title", &self.app_name)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await;

        let response_time = start_time.elapsed().as_millis() as u64;

        match response {
            Ok(resp) if resp.status().is_success() => {
                let stats = self.stats.lock().await;
                Ok(HealthStatus {
                    status: HealthLevel::Healthy,
                    message: Some("OpenRouter API is accessible".to_string()),
                    last_checked: chrono::Utc::now(),
                    response_time_ms: Some(response_time),
                    error_rate: Some(stats.failed_requests as f64 / stats.total_requests.max(1) as f64),
                })
            }
            Ok(resp) => {
                let stats = self.stats.lock().await;
                Ok(HealthStatus {
                    status: HealthLevel::Degraded,
                    message: Some(format!("OpenRouter API returned status: {}", resp.status())),
                    last_checked: chrono::Utc::now(),
                    response_time_ms: Some(response_time),
                    error_rate: Some(stats.failed_requests as f64 / stats.total_requests.max(1) as f64),
                })
            }
            Err(e) => {
                let stats = self.stats.lock().await;
                Ok(HealthStatus {
                    status: HealthLevel::Unhealthy,
                    message: Some(format!("OpenRouter API error: {}", e)),
                    last_checked: chrono::Utc::now(),
                    response_time_ms: None,
                    error_rate: Some(stats.failed_requests as f64 / stats.total_requests.max(1) as f64),
                })
            }
        }
    }

    async fn get_capabilities(&self, model: &str) -> Result<AICapabilities> {
        let model_info = self.models.iter()
            .find(|m| m.id == model)
            .ok_or_else(|| crate::error::Error::Configuration(
                format!("Model {} not found", model)
            ))?;

        let mut capabilities = AICapabilities::new();
        
        // Set basic capabilities
        capabilities.max_context_size = model_info.context_window.unwrap_or(4096);
        capabilities.supports_streaming = true;
        capabilities.supports_function_calling = model_info.capabilities.contains(&ModelCapability::FunctionCalling);
        capabilities.supports_tool_use = model_info.capabilities.contains(&ModelCapability::ToolUse);
        capabilities.supports_images = model_info.capabilities.contains(&ModelCapability::ImageUnderstanding);

        Ok(capabilities)
    }

    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
        let start_time = std::time::Instant::now();
        
        let openrouter_request = OpenRouterRequest {
            model: request.model.unwrap_or_else(|| "openai/gpt-3.5-turbo".to_string()),
            messages: self.convert_messages(&request.messages),
            temperature: request.parameters.as_ref().and_then(|p| p.temperature),
            max_tokens: request.parameters.as_ref().and_then(|p| p.max_tokens),
            top_p: request.parameters.as_ref().and_then(|p| p.top_p),
            frequency_penalty: request.parameters.as_ref().and_then(|p| p.frequency_penalty),
            presence_penalty: request.parameters.as_ref().and_then(|p| p.presence_penalty),
            stop: request.parameters.as_ref().and_then(|p| p.stop.clone()),
            stream: Some(false),
        };

        let response = self.client
            .post(&format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("HTTP-Referer", &self.site_url)
            .header("X-Title", &self.app_name)
            .header("Content-Type", "application/json")
            .json(&openrouter_request)
            .send()
            .await?;

        let response_time = start_time.elapsed().as_millis() as u64;

        if response.status().is_success() {
            let openrouter_response: OpenRouterResponse = response.json().await?;
            
            let chat_response = ChatResponse {
                id: openrouter_response.id,
                model: Some(openrouter_response.model),
                choices: openrouter_response.choices.into_iter().map(|choice| {
                    crate::common::ChatChoice {
                        index: choice.index,
                        message: ChatMessage {
                            role: match choice.message.role.as_str() {
                                "system" => MessageRole::System,
                                "user" => MessageRole::User,
                                "assistant" => MessageRole::Assistant,
                                "tool" => MessageRole::Tool,
                                _ => MessageRole::Assistant,
                            },
                            content: Some(choice.message.content),
                            name: choice.message.name,
                            tool_calls: None,
                            tool_call_id: None,
                        },
                        finish_reason: choice.finish_reason,
                    }
                }).collect(),
                usage: Some(UsageInfo {
                    prompt_tokens: openrouter_response.usage.prompt_tokens,
                    completion_tokens: openrouter_response.usage.completion_tokens,
                    total_tokens: openrouter_response.usage.total_tokens,
                }),
                created: Some(openrouter_response.created),
            };

            // Calculate actual cost based on usage
            let actual_cost = if let Some(usage) = &chat_response.usage {
                // Use the existing estimate_cost method with actual usage data
                let prompt_tokens = usage.prompt_tokens as f64;
                let completion_tokens = usage.completion_tokens as f64;
                
                // Calculate cost based on model pricing
                let input_cost = self.models.iter()
                    .find(|m| m.id == request.model.unwrap_or_else(|| "openai/gpt-3.5-turbo".to_string()))
                    .and_then(|m| m.input_cost_per_1k_tokens)
                    .unwrap_or(0.002) * (prompt_tokens / 1000.0);
                let output_cost = self.models.iter()
                    .find(|m| m.id == request.model.unwrap_or_else(|| "openai/gpt-3.5-turbo".to_string()))
                    .and_then(|m| m.output_cost_per_1k_tokens)
                    .unwrap_or(0.006) * (completion_tokens / 1000.0);
                
                input_cost + output_cost
            } else {
                0.0
            };

            // Update stats
            self.update_stats(
                true,
                response_time,
                openrouter_response.usage.total_tokens as u64,
                actual_cost
            ).await;

            Ok(chat_response)
        } else {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            
            // Update stats
            self.update_stats(false, response_time, 0, 0.0).await;
            {
                let mut stats = self.stats.lock().await;
                stats.last_error = Some(error_text.clone());
                stats.last_error_time = Some(chrono::Utc::now());
            }
            
            Err(crate::error::Error::Provider(error_text))
        }
    }

    async fn chat_stream(&self, request: ChatRequest) -> Result<ChatResponseStream> {
        let openrouter_request = OpenRouterRequest {
            model: request.model.unwrap_or_else(|| "openai/gpt-3.5-turbo".to_string()),
            messages: self.convert_messages(&request.messages),
            temperature: request.parameters.as_ref().and_then(|p| p.temperature),
            max_tokens: request.parameters.as_ref().and_then(|p| p.max_tokens),
            top_p: request.parameters.as_ref().and_then(|p| p.top_p),
            frequency_penalty: request.parameters.as_ref().and_then(|p| p.frequency_penalty),
            presence_penalty: request.parameters.as_ref().and_then(|p| p.presence_penalty),
            stop: request.parameters.as_ref().and_then(|p| p.stop.clone()),
            stream: Some(true),
        };

        let response = self.client
            .post(&format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("HTTP-Referer", &self.site_url)
            .header("X-Title", &self.app_name)
            .header("Content-Type", "application/json")
            .json(&openrouter_request)
            .send()
            .await?;

        if response.status().is_success() {
            let stream = response.bytes_stream()
                .map(|chunk| {
                    match chunk {
                        Ok(bytes) => {
                            let text = String::from_utf8_lossy(&bytes);
                            // Parse SSE format
                            for line in text.lines() {
                                if line.starts_with("data: ") {
                                    let data = &line[6..];
                                    if data == "[DONE]" {
                                        return Ok(ChatResponseChunk {
                                            id: "done".to_string(),
                                            choices: vec![],
                                            model: None,
                                            created: None,
                                        });
                                    }
                                    
                                    if let Ok(chunk) = serde_json::from_str::<OpenRouterStreamChunk>(data) {
                                        return Ok(ChatResponseChunk {
                                            id: chunk.id,
                                            choices: chunk.choices.into_iter().map(|choice| {
                                                crate::common::ChatChoiceChunk {
                                                    index: choice.index,
                                                    delta: crate::common::ChatDelta {
                                                        role: choice.delta.role.map(|r| match r.as_str() {
                                                            "system" => MessageRole::System,
                                                            "user" => MessageRole::User,
                                                            "assistant" => MessageRole::Assistant,
                                                            "tool" => MessageRole::Tool,
                                                            _ => MessageRole::Assistant,
                                                        }),
                                                        content: choice.delta.content,
                                                        name: None,
                                                        tool_calls: None,
                                                    },
                                                    finish_reason: choice.finish_reason,
                                                }
                                            }).collect(),
                                            model: Some(chunk.model),
                                            created: Some(chunk.created),
                                        });
                                    }
                                }
                            }
                            
                            // Return empty chunk if parsing fails
                            Ok(ChatResponseChunk {
                                id: "parse_error".to_string(),
                                choices: vec![],
                                model: None,
                                created: None,
                            })
                        }
                        Err(e) => Err(crate::error::Error::Provider(e.to_string())),
                    }
                });

            Ok(ChatResponseStream {
                inner: Box::pin(stream),
            })
        } else {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(crate::error::Error::Provider(error_text))
        }
    }

    async fn list_models(&self) -> Result<Vec<ModelInfo>> {
        Ok(self.models.clone())
    }

    async fn get_stats(&self) -> Result<ProviderStats> {
        let stats = self.stats.lock().await;
        Ok(stats.clone())
    }

    async fn shutdown(&mut self) -> Result<()> {
        // OpenRouter doesn't require special shutdown
        Ok(())
    }
}

/// Factory function for creating OpenRouter provider
pub fn create_provider(config: ProviderConfig) -> Result<Box<dyn ProviderPlugin>> {
    Ok(Box::new(OpenRouterProvider::new()))
} 