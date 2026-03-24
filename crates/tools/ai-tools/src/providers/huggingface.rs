// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

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

/// Hugging Face provider implementation
pub struct HuggingFaceProvider {
    client: Client,
    api_key: String,
    base_url: String,
    default_model: String,
    models: Vec<ModelInfo>,
    stats: Arc<Mutex<ProviderStats>>,
    use_inference_api: bool,
}

/// Hugging Face Inference API request format
#[derive(Debug, Serialize)]
struct HuggingFaceRequest {
    inputs: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    parameters: Option<HuggingFaceParameters>,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<HuggingFaceOptions>,
}

#[derive(Debug, Serialize)]
struct HuggingFaceParameters {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_new_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_k: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    repetition_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    do_sample: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    return_full_text: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
struct HuggingFaceOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    wait_for_model: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    use_cache: Option<bool>,
}

/// Hugging Face Inference API response format
#[derive(Debug, Deserialize)]
struct HuggingFaceResponse {
    #[serde(default)]
    generated_text: Option<String>,
    #[serde(default)]
    error: Option<String>,
    #[serde(default)]
    warnings: Option<Vec<String>>,
}

/// Hugging Face Chat Completions API request (for compatible models)
#[derive(Debug, Serialize)]
struct HuggingFaceChatRequest {
    model: String,
    messages: Vec<HuggingFaceChatMessage>,
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
struct HuggingFaceChatMessage {
    role: String,
    content: String,
}

/// Hugging Face Chat Completions API response
#[derive(Debug, Deserialize)]
struct HuggingFaceChatResponse {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<HuggingFaceChatChoice>,
    usage: Option<HuggingFaceUsage>,
}

#[derive(Debug, Deserialize)]
struct HuggingFaceChatChoice {
    index: u32,
    message: HuggingFaceChatMessage,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct HuggingFaceUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

/// Hugging Face streaming response chunk
#[derive(Debug, Deserialize)]
struct HuggingFaceStreamChunk {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<HuggingFaceStreamChoice>,
}

#[derive(Debug, Deserialize)]
struct HuggingFaceStreamChoice {
    index: u32,
    delta: HuggingFaceDelta,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct HuggingFaceDelta {
    role: Option<String>,
    content: Option<String>,
}

/// Model information from Hugging Face Hub
#[derive(Debug, Deserialize)]
struct HuggingFaceModelInfo {
    id: String,
    #[serde(default)]
    pipeline_tag: Option<String>,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    downloads: Option<u64>,
    #[serde(default)]
    likes: Option<u32>,
    #[serde(default)]
    library_name: Option<String>,
    #[serde(default)]
    model_index: Option<serde_json::Value>,
}

impl HuggingFaceProvider {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            api_key: String::new(),
            base_url: "https://api-inference.huggingface.co".to_string(),
            default_model: "microsoft/DialoGPT-medium".to_string(),
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
            use_inference_api: true,
        }
    }

    fn convert_messages_to_prompt(&self, messages: &[ChatMessage]) -> String {
        let mut prompt = String::new();
        
        for message in messages {
            let content = message.content.as_ref().unwrap_or(&String::new());
            match message.role {
                MessageRole::System => {
                    prompt.push_str(&format!("System: {}\n", content));
                }
                MessageRole::User => {
                    prompt.push_str(&format!("User: {}\n", content));
                }
                MessageRole::Assistant => {
                    prompt.push_str(&format!("Assistant: {}\n", content));
                }
                MessageRole::Tool => {
                    prompt.push_str(&format!("Tool: {}\n", content));
                }
            }
        }
        
        // Add assistant prompt to continue the conversation
        prompt.push_str("Assistant:");
        
        prompt
    }

    fn convert_messages_to_chat(&self, messages: &[ChatMessage]) -> Vec<HuggingFaceChatMessage> {
        messages.iter().map(|msg| {
            HuggingFaceChatMessage {
                role: match msg.role {
                    MessageRole::System => "system".to_string(),
                    MessageRole::User => "user".to_string(),
                    MessageRole::Assistant => "assistant".to_string(),
                    MessageRole::Tool => "tool".to_string(),
                },
                content: msg.content.clone().unwrap_or_default(),
            }
        }).collect()
    }

    async fn populate_popular_models(&mut self) {
        // Popular Hugging Face models for different tasks
        self.models = vec![
            ModelInfo {
                id: "microsoft/DialoGPT-medium".to_string(),
                name: "DialoGPT Medium".to_string(),
                description: Some("Conversational AI model trained on Reddit data".to_string()),
                context_window: Some(1024),
                max_output_tokens: Some(512),
                input_cost_per_1k_tokens: Some(0.0), // Free tier
                output_cost_per_1k_tokens: Some(0.0),
                capabilities: vec![
                    ModelCapability::TextGeneration,
                    ModelCapability::ChatCompletion,
                ],
                tags: vec!["conversational".to_string(), "microsoft".to_string()],
            },
            ModelInfo {
                id: "meta-llama/Llama-2-7b-chat-hf".to_string(),
                name: "Llama 2 7B Chat".to_string(),
                description: Some("Meta's Llama 2 model fine-tuned for chat".to_string()),
                context_window: Some(4096),
                max_output_tokens: Some(2048),
                input_cost_per_1k_tokens: Some(0.0),
                output_cost_per_1k_tokens: Some(0.0),
                capabilities: vec![
                    ModelCapability::TextGeneration,
                    ModelCapability::ChatCompletion,
                    ModelCapability::Reasoning,
                ],
                tags: vec!["llama".to_string(), "meta".to_string(), "chat".to_string()],
            },
            ModelInfo {
                id: "mistralai/Mistral-7B-Instruct-v0.1".to_string(),
                name: "Mistral 7B Instruct".to_string(),
                description: Some("Mistral's instruction-following model".to_string()),
                context_window: Some(8192),
                max_output_tokens: Some(4096),
                input_cost_per_1k_tokens: Some(0.0),
                output_cost_per_1k_tokens: Some(0.0),
                capabilities: vec![
                    ModelCapability::TextGeneration,
                    ModelCapability::ChatCompletion,
                    ModelCapability::Reasoning,
                    ModelCapability::Coding,
                ],
                tags: vec!["mistral".to_string(), "instruct".to_string()],
            },
            ModelInfo {
                id: "codellama/CodeLlama-7b-Python-hf".to_string(),
                name: "Code Llama 7B Python".to_string(),
                description: Some("Code Llama specialized for Python".to_string()),
                context_window: Some(4096),
                max_output_tokens: Some(2048),
                input_cost_per_1k_tokens: Some(0.0),
                output_cost_per_1k_tokens: Some(0.0),
                capabilities: vec![
                    ModelCapability::CodeGeneration,
                    ModelCapability::Coding,
                    ModelCapability::TextGeneration,
                ],
                tags: vec!["code".to_string(), "python".to_string(), "llama".to_string()],
            },
            ModelInfo {
                id: "HuggingFaceH4/zephyr-7b-beta".to_string(),
                name: "Zephyr 7B Beta".to_string(),
                description: Some("Zephyr model fine-tuned for helpfulness".to_string()),
                context_window: Some(4096),
                max_output_tokens: Some(2048),
                input_cost_per_1k_tokens: Some(0.0),
                output_cost_per_1k_tokens: Some(0.0),
                capabilities: vec![
                    ModelCapability::TextGeneration,
                    ModelCapability::ChatCompletion,
                    ModelCapability::Reasoning,
                ],
                tags: vec!["zephyr".to_string(), "helpful".to_string()],
            },
            ModelInfo {
                id: "stabilityai/stablelm-tuned-alpha-7b".to_string(),
                name: "StableLM Tuned Alpha 7B".to_string(),
                description: Some("Stability AI's language model".to_string()),
                context_window: Some(4096),
                max_output_tokens: Some(2048),
                input_cost_per_1k_tokens: Some(0.0),
                output_cost_per_1k_tokens: Some(0.0),
                capabilities: vec![
                    ModelCapability::TextGeneration,
                    ModelCapability::ChatCompletion,
                ],
                tags: vec!["stability".to_string(), "stablelm".to_string()],
            },
        ];
    }

    async fn update_stats(&self, success: bool, response_time_ms: u64, tokens: u64) {
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
        
        // Update uptime percentage
        stats.uptime_percentage = (stats.successful_requests as f64 / stats.total_requests as f64) * 100.0;
    }

    fn supports_chat_completions(&self, model: &str) -> bool {
        // Models that support OpenAI-compatible chat completions
        model.contains("chat") || 
        model.contains("instruct") || 
        model.contains("zephyr") ||
        model.contains("vicuna") ||
        model.contains("alpaca")
    }

    fn estimate_tokens(&self, text: &str) -> u32 {
        // Rough estimation: 1 token ≈ 4 characters
        (text.len() / 4) as u32
    }
}

#[async_trait]
impl ProviderPlugin for HuggingFaceProvider {
    fn name(&self) -> &str {
        "Hugging Face"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn provider_type(&self) -> ProviderType {
        ProviderType::Cloud
    }

    fn supported_models(&self) -> Vec<ModelInfo> {
        self.models.clone()
    }

    async fn initialize(&mut self, config: ProviderConfig) -> Result<()> {
        // Extract configuration
        self.api_key = config.settings.get("api_key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::Error::Configuration(
                "Hugging Face API key is required".to_string()
            ))?
            .to_string();

        if let Some(base_url) = config.settings.get("base_url").and_then(|v| v.as_str()) {
            self.base_url = base_url.to_string();
        }

        if let Some(default_model) = config.settings.get("default_model").and_then(|v| v.as_str()) {
            self.default_model = default_model.to_string();
        }

        if let Some(use_inference_api) = config.settings.get("use_inference_api").and_then(|v| v.as_bool()) {
            self.use_inference_api = use_inference_api;
        }

        // Populate model list
        self.populate_popular_models().await;

        Ok(())
    }

    async fn health_check(&self) -> Result<HealthStatus> {
        let start_time = std::time::Instant::now();
        
        // Test with a simple request to the default model
        let test_request = HuggingFaceRequest {
            inputs: "Hello".to_string(),
            parameters: Some(HuggingFaceParameters {
                max_new_tokens: Some(1),
                do_sample: Some(false),
                return_full_text: Some(false),
                temperature: None,
                top_p: None,
                top_k: None,
                repetition_penalty: None,
                stop: None,
            }),
            options: Some(HuggingFaceOptions {
                wait_for_model: Some(true),
                use_cache: Some(false),
            }),
        };

        let response = self.client
            .post(&format!("{}/models/{}", self.base_url, self.default_model))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&test_request)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await;

        let response_time = start_time.elapsed().as_millis() as u64;

        match response {
            Ok(resp) if resp.status().is_success() => {
                Ok(HealthStatus {
                    status: HealthLevel::Healthy,
                    message: Some("Hugging Face API is accessible".to_string()),
                    last_checked: chrono::Utc::now(),
                    response_time_ms: Some(response_time),
                    error_rate: Some(self.stats.failed_requests as f64 / self.stats.total_requests.max(1) as f64),
                })
            }
            Ok(resp) => {
                let status_code = resp.status();
                let error_text = resp.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                
                let health_level = if status_code.is_server_error() {
                    HealthLevel::Unhealthy
                } else {
                    HealthLevel::Degraded
                };

                Ok(HealthStatus {
                    status: health_level,
                    message: Some(format!("Hugging Face API returned status: {} - {}", status_code, error_text)),
                    last_checked: chrono::Utc::now(),
                    response_time_ms: Some(response_time),
                    error_rate: Some(self.stats.failed_requests as f64 / self.stats.total_requests.max(1) as f64),
                })
            }
            Err(e) => {
                Ok(HealthStatus {
                    status: HealthLevel::Unhealthy,
                    message: Some(format!("Hugging Face API error: {}", e)),
                    last_checked: chrono::Utc::now(),
                    response_time_ms: None,
                    error_rate: Some(self.stats.failed_requests as f64 / self.stats.total_requests.max(1) as f64),
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
        
        capabilities.max_context_size = model_info.context_window.unwrap_or(2048);
        capabilities.supports_streaming = self.supports_chat_completions(model);
        capabilities.supports_function_calling = false; // Most HF models don't support this
        capabilities.supports_tool_use = false;
        capabilities.supports_images = false; // Text-only for now

        Ok(capabilities)
    }

    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
        let start_time = std::time::Instant::now();
        let model = request.model.as_ref().unwrap_or(&self.default_model);

        if self.supports_chat_completions(model) {
            // Use chat completions API for compatible models
            let hf_request = HuggingFaceChatRequest {
                model: model.clone(),
                messages: self.convert_messages_to_chat(&request.messages),
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
                .header("Content-Type", "application/json")
                .json(&hf_request)
                .send()
                .await?;

            let response_time = start_time.elapsed().as_millis() as u64;

            if response.status().is_success() {
                let hf_response: HuggingFaceChatResponse = response.json().await?;
                
                let chat_response = ChatResponse {
                    id: hf_response.id,
                    model: Some(hf_response.model),
                    choices: hf_response.choices.into_iter().map(|choice| {
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
                                name: None,
                                tool_calls: None,
                                tool_call_id: None,
                            },
                            finish_reason: choice.finish_reason,
                        }
                    }).collect(),
                    usage: hf_response.usage.map(|usage| UsageInfo {
                        prompt_tokens: usage.prompt_tokens,
                        completion_tokens: usage.completion_tokens,
                        total_tokens: usage.total_tokens,
                    }),
                    created: Some(hf_response.created),
                };

                // Update stats
                let tokens = hf_response.usage.map(|u| u.total_tokens as u64).unwrap_or(0);
                self.update_stats(true, response_time, tokens).await;

                Ok(chat_response)
            } else {
                let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                
                self.update_stats(false, response_time, 0).await;
                {
                    let mut stats = self.stats.lock().await;
                    stats.last_error = Some(error_text.clone());
                    stats.last_error_time = Some(chrono::Utc::now());
                }
                
                Err(crate::error::Error::Provider(error_text))
            }
        } else {
            // Use inference API for other models
            let prompt = self.convert_messages_to_prompt(&request.messages);
            
            let hf_request = HuggingFaceRequest {
                inputs: prompt.clone(),
                parameters: Some(HuggingFaceParameters {
                    temperature: request.parameters.as_ref().and_then(|p| p.temperature),
                    max_new_tokens: request.parameters.as_ref().and_then(|p| p.max_tokens),
                    top_p: request.parameters.as_ref().and_then(|p| p.top_p),
                    top_k: None,
                    repetition_penalty: None,
                    do_sample: Some(true),
                    return_full_text: Some(false),
                    stop: request.parameters.as_ref().and_then(|p| p.stop.clone()),
                }),
                options: Some(HuggingFaceOptions {
                    wait_for_model: Some(true),
                    use_cache: Some(true),
                }),
            };

            let response = self.client
                .post(&format!("{}/models/{}", self.base_url, model))
                .header("Authorization", format!("Bearer {}", self.api_key))
                .header("Content-Type", "application/json")
                .json(&hf_request)
                .send()
                .await?;

            let response_time = start_time.elapsed().as_millis() as u64;

            if response.status().is_success() {
                let hf_responses: Vec<HuggingFaceResponse> = response.json().await?;
                
                if let Some(hf_response) = hf_responses.first() {
                    if let Some(error) = &hf_response.error {
                        self.update_stats(false, response_time, 0).await;
                        {
                            let mut stats = self.stats.lock().await;
                            stats.last_error = Some(error.clone());
                            stats.last_error_time = Some(chrono::Utc::now());
                        }
                        
                        return Err(crate::error::Error::Provider(error.clone()));
                    }
                    
                    let generated_text = hf_response.generated_text.clone().unwrap_or_default();
                    let prompt_tokens = self.estimate_tokens(&prompt);
                    let completion_tokens = self.estimate_tokens(&generated_text);
                    
                    let chat_response = ChatResponse {
                        id: uuid::Uuid::new_v4().to_string(),
                        model: Some(model.clone()),
                        choices: vec![crate::common::ChatChoice {
                            index: 0,
                            message: ChatMessage {
                                role: MessageRole::Assistant,
                                content: Some(generated_text),
                                name: None,
                                tool_calls: None,
                                tool_call_id: None,
                            },
                            finish_reason: Some("stop".to_string()),
                        }],
                        usage: Some(UsageInfo {
                            prompt_tokens,
                            completion_tokens,
                            total_tokens: prompt_tokens + completion_tokens,
                        }),
                        created: Some(chrono::Utc::now().timestamp() as u64),
                    };

                    // Update stats
                    self.update_stats(true, response_time, (prompt_tokens + completion_tokens) as u64).await;

                    Ok(chat_response)
                } else {
                    self.update_stats(false, response_time, 0).await;
                    
                    Err(crate::error::Error::Provider("Empty response from Hugging Face".to_string()))
                }
            } else {
                let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                
                self.update_stats(false, response_time, 0).await;
                {
                    let mut stats = self.stats.lock().await;
                    stats.last_error = Some(error_text.clone());
                    stats.last_error_time = Some(chrono::Utc::now());
                }
                
                Err(crate::error::Error::Provider(error_text))
            }
        }
    }

    async fn chat_stream(&self, request: ChatRequest) -> Result<ChatResponseStream> {
        let model = request.model.as_ref().unwrap_or(&self.default_model);

        if !self.supports_chat_completions(model) {
            return Err(crate::error::Error::Provider(
                "Streaming not supported for this model type".to_string()
            ));
        }

        let hf_request = HuggingFaceChatRequest {
            model: model.clone(),
            messages: self.convert_messages_to_chat(&request.messages),
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
            .header("Content-Type", "application/json")
            .json(&hf_request)
            .send()
            .await?;

        if response.status().is_success() {
            let stream = response.bytes_stream()
                .map(|chunk| {
                    match chunk {
                        Ok(bytes) => {
                            let text = String::from_utf8_lossy(&bytes);
                            
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
                                    
                                    if let Ok(chunk) = serde_json::from_str::<HuggingFaceStreamChunk>(data) {
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

    async fn estimate_cost(&self, request: &ChatRequest) -> Result<CostEstimate> {
        // Hugging Face Inference API is free for public models
        // Pro/Enterprise tiers have different pricing
        let prompt_text = request.messages.iter()
            .map(|m| m.content.as_ref().unwrap_or(&String::new()))
            .collect::<Vec<_>>()
            .join(" ");
        
        let input_tokens = self.estimate_tokens(&prompt_text);
        let output_tokens = request.parameters.as_ref()
            .and_then(|p| p.max_tokens)
            .unwrap_or(100);

        Ok(CostEstimate {
            estimated_input_tokens: input_tokens,
            estimated_output_tokens: output_tokens,
            estimated_cost_usd: 0.0, // Free tier
            currency: "USD".to_string(),
            breakdown: Some(CostBreakdown {
                input_cost: 0.0,
                output_cost: 0.0,
                fixed_cost: None,
                additional_fees: None,
            }),
        })
    }

    async fn get_stats(&self) -> Result<ProviderStats> {
        let stats = self.stats.lock().await;
        Ok(stats.clone())
    }

    async fn shutdown(&mut self) -> Result<()> {
        // Hugging Face doesn't require special shutdown
        Ok(())
    }
}

/// Factory function for creating Hugging Face provider
pub fn create_provider(config: ProviderConfig) -> Result<Box<dyn ProviderPlugin>> {
    Ok(Box::new(HuggingFaceProvider::new()))
} 