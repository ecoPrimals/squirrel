//! Ollama provider for local AI models
//!
//! This module provides integration with Ollama for running local language models.

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::{debug, info};
use uuid;

use crate::common::capability::{AICapabilities, ModelType, TaskType};
use crate::common::{
    ChatChoice, ChatChoiceChunk, ChatMessage, ChatRequest, ChatResponse, ChatResponseChunk,
    ChatResponseStream, MessageRole, UsageInfo,
};
use crate::error::{Error, Result};
use crate::local::config::OllamaConfig;
use crate::local::{LocalModelInfo, LocalModelProvider};

/// Ollama local model provider
#[derive(Debug)]
pub struct OllamaProvider {
    /// Configuration
    config: OllamaConfig,

    /// HTTP client for API requests
    client: Client,

    /// Cache of model information
    model_cache: RwLock<HashMap<String, LocalModelInfo>>,

    /// Loaded models tracking
    loaded_models: RwLock<HashMap<String, Instant>>,
}

#[derive(Debug, Deserialize)]
struct OllamaModel {
    name: String,
    model: String,
    modified_at: String,
    size: u64,
    digest: String,
    details: OllamaModelDetails,
}

#[derive(Debug, Deserialize)]
struct OllamaModelDetails {
    parent_model: Option<String>,
    format: String,
    family: String,
    families: Option<Vec<String>>,
    parameter_size: String,
    quantization_level: String,
}

#[derive(Debug, Serialize)]
struct OllamaChatRequest {
    model: String,
    messages: Vec<OllamaMessage>,
    stream: bool,
    options: Option<OllamaOptions>,
}

#[derive(Debug, Serialize)]
struct OllamaMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct OllamaOptions {
    temperature: Option<f32>,
    top_p: Option<f32>,
    top_k: Option<i32>,
    num_predict: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct OllamaChatResponse {
    model: String,
    created_at: String,
    message: OllamaResponseMessage,
    done: bool,
    total_duration: Option<u64>,
    load_duration: Option<u64>,
    prompt_eval_count: Option<u32>,
    prompt_eval_duration: Option<u64>,
    eval_count: Option<u32>,
    eval_duration: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct OllamaResponseMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct OllamaModelsResponse {
    models: Vec<OllamaModel>,
}

impl OllamaProvider {
    /// Create a new Ollama provider
    pub async fn new(config: OllamaConfig) -> Result<Self> {
        let client = Client::new();

        let provider = Self {
            config,
            client,
            model_cache: RwLock::new(HashMap::new()),
            loaded_models: RwLock::new(HashMap::new()),
        };

        // Test connection
        provider.test_connection().await?;

        // Initial model discovery
        if provider.config.auto_discover_models {
            provider.refresh_model_cache().await?;
        }

        Ok(provider)
    }

    /// Test connection to Ollama server
    async fn test_connection(&self) -> Result<()> {
        let url = format!("{}/api/tags", self.config.base_url);

        let response = self
            .client
            .get(&url)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| Error::Http(e.to_string()))?;

        if response.status().is_success() {
            info!(
                "Successfully connected to Ollama server at {}",
                self.config.base_url
            );
            Ok(())
        } else {
            Err(Error::Network(format!(
                "Failed to connect to Ollama server: {}",
                response.status()
            )))
        }
    }

    /// Refresh the model cache by querying Ollama
    async fn refresh_model_cache(&self) -> Result<()> {
        let url = format!("{}/api/tags", self.config.base_url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| Error::Network(format!("Failed to fetch models from Ollama: {e}")))?;
        if !response.status().is_success() {
            return Err(Error::Network(format!(
                "Failed to fetch models from Ollama: {}",
                response.status()
            )));
        }

        let models_response: OllamaModelsResponse = response
            .json()
            .await
            .map_err(|e| Error::Http(e.to_string()))?;

        let mut cache = self.model_cache.write().await;
        cache.clear();

        for ollama_model in models_response.models {
            let model_info = self.convert_ollama_model_to_info(ollama_model);
            cache.insert(model_info.id.clone(), model_info);
        }

        info!("Refreshed Ollama model cache with {} models", cache.len());
        Ok(())
    }

    /// Convert Ollama model to our LocalModelInfo format
    fn convert_ollama_model_to_info(&self, ollama_model: OllamaModel) -> LocalModelInfo {
        let mut capabilities = AICapabilities::default();
        capabilities.add_model_type(ModelType::LargeLanguageModel);
        capabilities.add_task_type(TaskType::TextGeneration);
        capabilities.with_streaming(true);
        capabilities.with_function_calling(false); // Most Ollama models don't support function calling
        capabilities.with_tool_use(false);

        // Estimate context size based on model family
        let context_size = if ollama_model.details.family.contains("llama") {
            if ollama_model.name.contains("70b") {
                8192
            } else {
                4096
            }
        } else if ollama_model.details.family.contains("mistral") {
            8192
        } else {
            4096 // Conservative default
        };
        capabilities.with_max_context_size(context_size);

        // Estimate resource requirements based on parameter size
        let (min_memory_mb, min_gpu_memory_mb) =
            self.estimate_resource_requirements(&ollama_model.details.parameter_size);

        LocalModelInfo {
            id: ollama_model.name.clone(),
            name: ollama_model.model,
            path: ollama_model.name,
            implementation: "ollama".to_string(),
            capabilities,
            resource_requirements: crate::local::ResourceRequirements {
                min_memory_mb,
                min_gpu_memory_mb,
                requires_gpu: min_gpu_memory_mb.is_some(),
                min_cpu_cores: Some(4),
                estimated_load_time_ms: 5000, // 5 seconds estimate
            },
            is_loaded: false,
            load_time_ms: None,
        }
    }

    /// Convert our message format to Ollama format
    fn convert_messages_to_ollama(&self, messages: Vec<ChatMessage>) -> Vec<OllamaMessage> {
        messages
            .into_iter()
            .map(|msg| {
                let role = match msg.role {
                    MessageRole::System => "system",
                    MessageRole::User => "user",
                    MessageRole::Assistant => "assistant",
                    MessageRole::Tool => "user", // Ollama doesn't have tool role, map to user
                    MessageRole::Function => "user", // Function messages are deprecated, map to user
                };

                OllamaMessage {
                    role: role.to_string(),
                    content: msg.content.unwrap_or_default(),
                }
            })
            .collect()
    }

    /// Estimate resource requirements based on parameter size
    fn estimate_resource_requirements(&self, parameter_size: &str) -> (u64, Option<u64>) {
        // Parse parameter size (e.g., "7B", "13B", "70B")
        let size_str = parameter_size.to_uppercase();

        if size_str.contains("70B") {
            (32000, Some(80000)) // 32GB RAM, 80GB GPU memory
        } else if size_str.contains("13B") {
            (16000, Some(16000)) // 16GB RAM, 16GB GPU memory
        } else if size_str.contains("7B") || size_str.contains("8B") {
            (8000, Some(8000)) // 8GB RAM, 8GB GPU memory
        } else if size_str.contains("3B") {
            (4000, Some(4000)) // 4GB RAM, 4GB GPU memory
        } else {
            (8000, Some(8000)) // Default to 8GB
        }
    }

    /// Create Ollama request options from chat request
    fn create_ollama_options(&self, request: &ChatRequest) -> Option<OllamaOptions> {
        // Check if we have parameters to work with
        request.parameters.as_ref().map(|params| OllamaOptions {
            temperature: params.temperature,
            top_p: params.top_p,
            top_k: None, // Not supported in ChatRequest
            num_predict: params.max_tokens.map(|t| t as i32),
        })
    }
}

#[async_trait]
impl LocalModelProvider for OllamaProvider {
    fn provider_name(&self) -> &str {
        "ollama"
    }

    async fn list_models(&self) -> Result<Vec<LocalModelInfo>> {
        // Refresh cache if it's empty or if auto-discovery is enabled
        if self.model_cache.read().await.is_empty() || self.config.auto_discover_models {
            self.refresh_model_cache().await?;
        }

        let cache = self.model_cache.read().await;
        Ok(cache.values().cloned().collect())
    }

    async fn load_model(&self, model_id: &str) -> Result<()> {
        debug!("Loading Ollama model: {}", model_id);

        // For Ollama, we don't need to explicitly load models
        // They are loaded on first use, but we can send a simple request to warm them up
        let url = format!("{}/api/generate", self.config.base_url);

        let request = serde_json::json!({
            "model": model_id,
            "prompt": "",
            "stream": false
        });

        let start_time = Instant::now();
        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| Error::Http(e.to_string()))?;

        if response.status().is_success() {
            let load_time = start_time.elapsed();
            let mut loaded = self.loaded_models.write().await;
            loaded.insert(model_id.to_string(), Instant::now());

            // Update model info with load time
            let mut cache = self.model_cache.write().await;
            if let Some(model_info) = cache.get_mut(model_id) {
                model_info.is_loaded = true;
                model_info.load_time_ms = Some(load_time.as_millis() as u64);
            }

            info!("Loaded Ollama model {} in {:?}", model_id, load_time);
            Ok(())
        } else {
            Err(Error::Model(format!(
                "Failed to load model {}: {}",
                model_id,
                response.status()
            )))
        }
    }

    async fn unload_model(&self, model_id: &str) -> Result<()> {
        debug!("Unloading Ollama model: {}", model_id);

        // For Ollama, we can't explicitly unload models, but we can track that it's "unloaded"
        let mut loaded = self.loaded_models.write().await;
        loaded.remove(model_id);

        let mut cache = self.model_cache.write().await;
        if let Some(model_info) = cache.get_mut(model_id) {
            model_info.is_loaded = false;
        }

        info!("Marked Ollama model {} as unloaded", model_id);
        Ok(())
    }

    async fn is_model_loaded(&self, model_id: &str) -> Result<bool> {
        let loaded = self.loaded_models.read().await;
        Ok(loaded.contains_key(model_id))
    }

    async fn chat(&self, model_id: &str, request: ChatRequest) -> Result<ChatResponse> {
        debug!("Sending chat request to Ollama model: {}", model_id);

        let url = format!("{}/api/chat", self.config.base_url);

        let options = self.create_ollama_options(&request);
        let messages = self.convert_messages_to_ollama(request.messages);

        let ollama_request = OllamaChatRequest {
            model: model_id.to_string(),
            messages,
            stream: false,
            options,
        };

        let response = self
            .client
            .post(&url)
            .json(&ollama_request)
            .send()
            .await
            .map_err(|e| Error::Http(e.to_string()))?;

        if !response.status().is_success() {
            return Err(Error::Model(format!(
                "Ollama API error: {}",
                response.status()
            )));
        }

        let ollama_response: OllamaChatResponse = response
            .json()
            .await
            .map_err(|e| Error::Http(e.to_string()))?;

        Ok(ChatResponse {
            id: format!("ollama-{}", uuid::Uuid::new_v4()),
            model: model_id.to_string(),
            choices: vec![ChatChoice {
                index: 0,
                role: MessageRole::Assistant,
                content: Some(ollama_response.message.content),
                finish_reason: Some("stop".to_string()),
                tool_calls: None,
            }],
            usage: Some(UsageInfo {
                prompt_tokens: ollama_response.prompt_eval_count.unwrap_or(0),
                completion_tokens: ollama_response.eval_count.unwrap_or(0),
                total_tokens: ollama_response.prompt_eval_count.unwrap_or(0)
                    + ollama_response.eval_count.unwrap_or(0),
            }),
        })
    }

    async fn chat_stream(
        &self,
        model_id: &str,
        request: ChatRequest,
    ) -> Result<ChatResponseStream> {
        debug!(
            "Sending streaming chat request to Ollama model: {}",
            model_id
        );

        let url = format!("{}/api/chat", self.config.base_url);

        let options = self.create_ollama_options(&request);
        let messages = self.convert_messages_to_ollama(request.messages);

        let ollama_request = OllamaChatRequest {
            model: model_id.to_string(),
            messages,
            stream: true,
            options,
        };

        let response = self
            .client
            .post(&url)
            .json(&ollama_request)
            .send()
            .await
            .map_err(|e| Error::Http(e.to_string()))?;

        if !response.status().is_success() {
            return Err(Error::Model(format!(
                "Ollama API error: {}",
                response.status()
            )));
        }

        // Parse streaming response from Ollama
        use futures::stream;

        let model_id_owned = model_id.to_string();

        // Read the full response text for now (simplified implementation)
        let response_text = response
            .text()
            .await
            .map_err(|e| Error::Http(format!("Failed to read response: {e}")))?;

        // Parse the complete response
        let ollama_response: serde_json::Value = serde_json::from_str(&response_text)
            .map_err(|e| Error::Model(format!("JSON parse error: {e}")))?;

        // Extract content and create response chunk
        let content = if let Some(message) = ollama_response.get("message") {
            message
                .get("content")
                .and_then(|c| c.as_str())
                .unwrap_or("No content")
                .to_string()
        } else {
            "No response content".to_string()
        };

        let chunk = ChatResponseChunk {
            id: format!("ollama-{}", uuid::Uuid::new_v4()),
            model: model_id_owned,
            choices: vec![ChatChoiceChunk {
                index: 0,
                delta: ChatMessage {
                    role: MessageRole::Assistant,
                    content: Some(content),
                    name: None,
                    tool_calls: None,
                    tool_call_id: None,
                },
                finish_reason: Some("stop".to_string()),
            }],
        };

        let stream = stream::iter(vec![Ok(chunk)]);
        Ok(Box::pin(stream))
    }

    async fn get_model_info(&self, model_id: &str) -> Result<LocalModelInfo> {
        let cache = self.model_cache.read().await;
        cache
            .get(model_id)
            .cloned()
            .ok_or_else(|| Error::Model(format!("Model not found: {model_id}")))
    }

    fn capabilities(&self) -> AICapabilities {
        let mut capabilities = AICapabilities::default();
        capabilities.add_model_type(ModelType::LargeLanguageModel);
        capabilities.add_task_type(TaskType::TextGeneration);
        capabilities.with_streaming(true);
        capabilities.with_function_calling(false);
        capabilities.with_tool_use(false);
        capabilities.with_max_context_size(8192);
        capabilities
    }
}
