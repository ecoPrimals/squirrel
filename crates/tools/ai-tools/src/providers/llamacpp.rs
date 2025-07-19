use std::collections::HashMap;
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

/// llama.cpp provider implementation
pub struct LlamaCppProvider {
    client: Client,
    endpoint: String,
    model_name: String,
    context_size: usize,
    stats: Arc<Mutex<ProviderStats>>,
}

/// llama.cpp completion request
#[derive(Debug, Serialize)]
struct LlamaCppRequest {
    prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_k: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    n_predict: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    repeat_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    repeat_last_n: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    penalize_nl: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    presence_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    frequency_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mirostat: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mirostat_tau: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mirostat_eta: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    seed: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ignore_eos: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    logit_bias: Option<HashMap<String, f32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    n_probs: Option<u32>,
}

/// llama.cpp completion response
#[derive(Debug, Deserialize)]
struct LlamaCppResponse {
    content: String,
    #[serde(default)]
    stop: bool,
    #[serde(default)]
    generation_settings: Option<LlamaCppGenerationSettings>,
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    prompt: Option<String>,
    #[serde(default)]
    stopped_eos: Option<bool>,
    #[serde(default)]
    stopped_limit: Option<bool>,
    #[serde(default)]
    stopped_word: Option<bool>,
    #[serde(default)]
    stopping_word: Option<String>,
    #[serde(default)]
    tokens_cached: Option<u32>,
    #[serde(default)]
    tokens_evaluated: Option<u32>,
    #[serde(default)]
    tokens_predicted: Option<u32>,
    #[serde(default)]
    truncated: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct LlamaCppGenerationSettings {
    #[serde(default)]
    frequency_penalty: Option<f32>,
    #[serde(default)]
    ignore_eos: Option<bool>,
    #[serde(default)]
    logit_bias: Option<HashMap<String, f32>>,
    #[serde(default)]
    max_tokens: Option<u32>,
    #[serde(default)]
    mirostat: Option<u32>,
    #[serde(default)]
    mirostat_eta: Option<f32>,
    #[serde(default)]
    mirostat_tau: Option<f32>,
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    n_ctx: Option<u32>,
    #[serde(default)]
    n_keep: Option<u32>,
    #[serde(default)]
    n_predict: Option<i32>,
    #[serde(default)]
    n_probs: Option<u32>,
    #[serde(default)]
    penalize_nl: Option<bool>,
    #[serde(default)]
    presence_penalty: Option<f32>,
    #[serde(default)]
    repeat_last_n: Option<i32>,
    #[serde(default)]
    repeat_penalty: Option<f32>,
    #[serde(default)]
    seed: Option<i32>,
    #[serde(default)]
    stop: Option<Vec<String>>,
    #[serde(default)]
    stream: Option<bool>,
    #[serde(default)]
    temperature: Option<f32>,
    #[serde(default)]
    tfs_z: Option<f32>,
    #[serde(default)]
    top_k: Option<u32>,
    #[serde(default)]
    top_p: Option<f32>,
    #[serde(default)]
    typical_p: Option<f32>,
}

/// llama.cpp streaming response chunk
#[derive(Debug, Deserialize)]
struct LlamaCppStreamChunk {
    content: String,
    #[serde(default)]
    stop: bool,
    #[serde(default)]
    multimodal: Option<bool>,
    #[serde(default)]
    slot_id: Option<u32>,
}

/// llama.cpp health/props response
#[derive(Debug, Deserialize)]
struct LlamaCppProps {
    #[serde(default)]
    user_name: Option<String>,
    #[serde(default)]
    assistant_name: Option<String>,
    #[serde(default)]
    default_generation_settings: Option<LlamaCppGenerationSettings>,
    #[serde(default)]
    total_slots: Option<u32>,
    #[serde(default)]
    chat_template: Option<String>,
}

impl LlamaCppProvider {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            endpoint: "http://localhost:8080".to_string(),
            model_name: "unknown".to_string(),
            context_size: 4096,
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

    fn format_chat_prompt(&self, messages: &[ChatMessage]) -> Result<String> {
        let mut prompt = String::new();
        
        for message in messages {
            let content = message.content.as_ref().unwrap_or(&String::new());
            match message.role {
                MessageRole::System => {
                    prompt.push_str(&format!("### System:\n{}\n\n", content));
                }
                MessageRole::User => {
                    prompt.push_str(&format!("### User:\n{}\n\n", content));
                }
                MessageRole::Assistant => {
                    prompt.push_str(&format!("### Assistant:\n{}\n\n", content));
                }
                MessageRole::Tool => {
                    prompt.push_str(&format!("### Tool:\n{}\n\n", content));
                }
            }
        }
        
        // Add assistant prompt to continue the conversation
        prompt.push_str("### Assistant:\n");
        
        Ok(prompt)
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

    async fn fetch_props(&mut self) -> Result<()> {
        let response = self.client
            .get(&format!("{}/props", self.endpoint))
            .send()
            .await?;

        if response.status().is_success() {
            let props: LlamaCppProps = response.json().await?;
            
            // Extract context size from default generation settings
            if let Some(settings) = props.default_generation_settings {
                if let Some(n_ctx) = settings.n_ctx {
                    self.context_size = n_ctx as usize;
                }
            }
        }

        Ok(())
    }
}

#[async_trait]
impl ProviderPlugin for LlamaCppProvider {
    fn name(&self) -> &str {
        "llama.cpp"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn provider_type(&self) -> ProviderType {
        ProviderType::Local
    }

    fn supported_models(&self) -> Vec<ModelInfo> {
        vec![ModelInfo {
            id: self.model_name.clone(),
            name: format!("llama.cpp - {}", self.model_name),
            description: Some("Local model running on llama.cpp server".to_string()),
            context_window: Some(self.context_size),
            max_output_tokens: Some(self.context_size / 2), // Conservative estimate
            input_cost_per_1k_tokens: Some(0.0), // Local models are free
            output_cost_per_1k_tokens: Some(0.0),
            capabilities: vec![
                ModelCapability::TextGeneration,
                ModelCapability::TextCompletion,
                ModelCapability::ChatCompletion,
                ModelCapability::Reasoning,
            ],
            tags: vec![
                "local".to_string(),
                "llama".to_string(),
                "cpp".to_string(),
                "fast".to_string(),
            ],
        }]
    }

    async fn initialize(&mut self, config: ProviderConfig) -> Result<()> {
        // Extract configuration
        if let Some(endpoint) = config.settings.get("endpoint").and_then(|v| v.as_str()) {
            self.endpoint = endpoint.to_string();
        }

        if let Some(model_name) = config.settings.get("model_name").and_then(|v| v.as_str()) {
            self.model_name = model_name.to_string();
        }

        if let Some(context_size) = config.settings.get("context_size").and_then(|v| v.as_u64()) {
            self.context_size = context_size as usize;
        }

        // Fetch server properties
        self.fetch_props().await?;

        Ok(())
    }

    async fn health_check(&self) -> Result<HealthStatus> {
        let start_time = std::time::Instant::now();
        
        let response = self.client
            .get(&format!("{}/health", self.endpoint))
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await;

        let response_time = start_time.elapsed().as_millis() as u64;

        match response {
            Ok(resp) if resp.status().is_success() => {
                Ok(HealthStatus {
                    status: HealthLevel::Healthy,
                    message: Some("llama.cpp server is running".to_string()),
                    last_checked: chrono::Utc::now(),
                    response_time_ms: Some(response_time),
                    error_rate: Some(self.stats.failed_requests as f64 / self.stats.total_requests.max(1) as f64),
                })
            }
            Ok(resp) => {
                // Try /props endpoint as fallback
                let props_response = self.client
                    .get(&format!("{}/props", self.endpoint))
                    .timeout(std::time::Duration::from_secs(5))
                    .send()
                    .await;

                match props_response {
                    Ok(props_resp) if props_resp.status().is_success() => {
                        Ok(HealthStatus {
                            status: HealthLevel::Healthy,
                            message: Some("llama.cpp server is running (via props)".to_string()),
                            last_checked: chrono::Utc::now(),
                            response_time_ms: Some(response_time),
                            error_rate: Some(self.stats.failed_requests as f64 / self.stats.total_requests.max(1) as f64),
                        })
                    }
                    _ => {
                        Ok(HealthStatus {
                            status: HealthLevel::Degraded,
                            message: Some(format!("llama.cpp server returned status: {}", resp.status())),
                            last_checked: chrono::Utc::now(),
                            response_time_ms: Some(response_time),
                            error_rate: Some(self.stats.failed_requests as f64 / self.stats.total_requests.max(1) as f64),
                        })
                    }
                }
            }
            Err(e) => {
                Ok(HealthStatus {
                    status: HealthLevel::Unhealthy,
                    message: Some(format!("llama.cpp server error: {}", e)),
                    last_checked: chrono::Utc::now(),
                    response_time_ms: None,
                    error_rate: Some(self.stats.failed_requests as f64 / self.stats.total_requests.max(1) as f64),
                })
            }
        }
    }

    async fn get_capabilities(&self, _model: &str) -> Result<AICapabilities> {
        let mut capabilities = AICapabilities::new();
        
        capabilities.max_context_size = self.context_size;
        capabilities.supports_streaming = true;
        capabilities.supports_function_calling = false; // Most llama.cpp models don't support this
        capabilities.supports_tool_use = false;
        capabilities.supports_images = false; // Basic text-only support

        Ok(capabilities)
    }

    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
        let start_time = std::time::Instant::now();
        
        let prompt = self.format_chat_prompt(&request.messages)?;
        
        let llamacpp_request = LlamaCppRequest {
            prompt,
            temperature: request.parameters.as_ref().and_then(|p| p.temperature),
            top_p: request.parameters.as_ref().and_then(|p| p.top_p),
            top_k: None,
            max_tokens: request.parameters.as_ref().and_then(|p| p.max_tokens),
            stop: request.parameters.as_ref().and_then(|p| p.stop.clone()),
            stream: Some(false),
            n_predict: request.parameters.as_ref().and_then(|p| p.max_tokens.map(|t| t as i32)),
            repeat_penalty: None,
            repeat_last_n: None,
            penalize_nl: None,
            presence_penalty: request.parameters.as_ref().and_then(|p| p.presence_penalty),
            frequency_penalty: request.parameters.as_ref().and_then(|p| p.frequency_penalty),
            mirostat: None,
            mirostat_tau: None,
            mirostat_eta: None,
            seed: None,
            ignore_eos: None,
            logit_bias: None,
            n_probs: None,
        };

        let response = self.client
            .post(&format!("{}/completion", self.endpoint))
            .header("Content-Type", "application/json")
            .json(&llamacpp_request)
            .send()
            .await?;

        let response_time = start_time.elapsed().as_millis() as u64;

        if response.status().is_success() {
            let llamacpp_response: LlamaCppResponse = response.json().await?;
            
            let tokens_evaluated = llamacpp_response.tokens_evaluated.unwrap_or(0);
            let tokens_predicted = llamacpp_response.tokens_predicted.unwrap_or(0);
            let total_tokens = tokens_evaluated + tokens_predicted;
            
            let chat_response = ChatResponse {
                id: uuid::Uuid::new_v4().to_string(),
                model: Some(self.model_name.clone()),
                choices: vec![crate::common::ChatChoice {
                    index: 0,
                    message: ChatMessage {
                        role: MessageRole::Assistant,
                        content: Some(llamacpp_response.content),
                        name: None,
                        tool_calls: None,
                        tool_call_id: None,
                    },
                    finish_reason: if llamacpp_response.stop {
                        Some("stop".to_string())
                    } else {
                        Some("length".to_string())
                    },
                }],
                usage: Some(UsageInfo {
                    prompt_tokens: tokens_evaluated,
                    completion_tokens: tokens_predicted,
                    total_tokens,
                }),
                created: Some(chrono::Utc::now().timestamp() as u64),
            };

            // Update stats
            self.update_stats(true, response_time, total_tokens as u64).await;

            Ok(chat_response)
        } else {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            
            // Update stats
            self.update_stats(false, response_time, 0).await;
            {
                let mut stats = self.stats.lock().await;
                stats.last_error = Some(error_text.clone());
                stats.last_error_time = Some(chrono::Utc::now());
            }
            
            Err(crate::error::Error::Provider(error_text))
        }
    }

    async fn chat_stream(&self, request: ChatRequest) -> Result<ChatResponseStream> {
        let prompt = self.format_chat_prompt(&request.messages)?;
        
        let llamacpp_request = LlamaCppRequest {
            prompt,
            temperature: request.parameters.as_ref().and_then(|p| p.temperature),
            top_p: request.parameters.as_ref().and_then(|p| p.top_p),
            top_k: None,
            max_tokens: request.parameters.as_ref().and_then(|p| p.max_tokens),
            stop: request.parameters.as_ref().and_then(|p| p.stop.clone()),
            stream: Some(true),
            n_predict: request.parameters.as_ref().and_then(|p| p.max_tokens.map(|t| t as i32)),
            repeat_penalty: None,
            repeat_last_n: None,
            penalize_nl: None,
            presence_penalty: request.parameters.as_ref().and_then(|p| p.presence_penalty),
            frequency_penalty: request.parameters.as_ref().and_then(|p| p.frequency_penalty),
            mirostat: None,
            mirostat_tau: None,
            mirostat_eta: None,
            seed: None,
            ignore_eos: None,
            logit_bias: None,
            n_probs: None,
        };

        let response = self.client
            .post(&format!("{}/completion", self.endpoint))
            .header("Content-Type", "application/json")
            .json(&llamacpp_request)
            .send()
            .await?;

        if response.status().is_success() {
            let stream = response.bytes_stream()
                .map(|chunk| {
                    match chunk {
                        Ok(bytes) => {
                            let text = String::from_utf8_lossy(&bytes);
                            
                            // llama.cpp sends SSE format
                            for line in text.lines() {
                                if line.starts_with("data: ") {
                                    let data = &line[6..];
                                    
                                    if let Ok(chunk) = serde_json::from_str::<LlamaCppStreamChunk>(data) {
                                        return Ok(ChatResponseChunk {
                                            id: uuid::Uuid::new_v4().to_string(),
                                            choices: vec![crate::common::ChatChoiceChunk {
                                                index: 0,
                                                delta: crate::common::ChatDelta {
                                                    role: Some(MessageRole::Assistant),
                                                    content: Some(chunk.content),
                                                    name: None,
                                                    tool_calls: None,
                                                },
                                                finish_reason: if chunk.stop {
                                                    Some("stop".to_string())
                                                } else {
                                                    None
                                                },
                                            }],
                                            model: Some(self.model_name.clone()),
                                            created: Some(chrono::Utc::now().timestamp() as u64),
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
        Ok(self.supported_models())
    }

    async fn estimate_cost(&self, _request: &ChatRequest) -> Result<CostEstimate> {
        // Local models are free
        Ok(CostEstimate {
            estimated_input_tokens: 0,
            estimated_output_tokens: 0,
            estimated_cost_usd: 0.0,
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
        // llama.cpp doesn't require special shutdown from client side
        Ok(())
    }
}

/// Factory function for creating llama.cpp provider
pub fn create_provider(config: ProviderConfig) -> Result<Box<dyn ProviderPlugin>> {
    Ok(Box::new(LlamaCppProvider::new()))
}

impl Default for LlamaCppConfig {
    fn default() -> Self {
        Self {
            endpoint: std::env::var("LLAMACPP_ENDPOINT")
                .unwrap_or_else(|_| "http://localhost:8080".to_string()),
            model_name: std::env::var("LLAMACPP_MODEL")
                .unwrap_or_else(|_| "unknown".to_string()),
            context_size: 4096,
            temperature: 0.7,
            top_p: 0.9,
            top_k: 40,
            repeat_penalty: 1.1,
            timeout_seconds: 30,
        }
    }
} 