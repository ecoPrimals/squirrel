//! Ollama client implementation
//!
//! This module provides a production-ready Ollama client that implements
//! the AIClient trait for seamless integration with the AI tools system.

use async_trait::async_trait;
use futures::stream;
use reqwest::Client;
use serde_json::json;

use crate::common::capability::{AICapabilities, ModelType, SecurityRequirements, TaskType};
use crate::common::client::AIClient;
use crate::common::types::{
    ChatChoice, ChatChoiceChunk, ChatMessage, ChatRequest, ChatResponse, ChatResponseChunk,
    ChatResponseStream, MessageRole, UsageInfo,
};

/// Production Ollama client
#[derive(Debug)]
pub struct OllamaClient {
    /// Ollama server endpoint
    endpoint: String,
    /// HTTP client for making requests
    client: Client,
}

impl OllamaClient {
    /// Create a new Ollama client
    pub fn new(endpoint: String) -> Self {
        Self {
            endpoint,
            client: Client::new(),
        }
    }

    /// Set custom HTTP client
    pub fn with_client(mut self, client: Client) -> Self {
        self.client = client;
        self
    }

    /// Get the endpoint
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    /// Convert internal ChatRequest to Ollama API format
    fn convert_request(&self, request: &ChatRequest) -> serde_json::Value {
        let model = request.model.as_deref().unwrap_or("llama2");

        // Convert messages to a single prompt for Ollama
        let prompt = request
            .messages
            .iter()
            .map(|msg| {
                let role = match msg.role {
                    MessageRole::System => "System",
                    MessageRole::User => "User",
                    MessageRole::Assistant => "Assistant",
                    MessageRole::Tool => "Tool",
                    MessageRole::Function => "Function",
                };

                if let Some(content) = &msg.content {
                    format!("{}: {}", role, content)
                } else {
                    format!("{}: [no content]", role)
                }
            })
            .collect::<Vec<_>>()
            .join("\n");

        let mut payload = json!({
            "model": model,
            "prompt": prompt,
            "stream": false,
        });

        // Add parameters if specified
        if let Some(parameters) = &request.parameters {
            let mut options = json!({});

            if let Some(temperature) = parameters.temperature {
                options["temperature"] = json!(temperature);
            }
            if let Some(top_p) = parameters.top_p {
                options["top_p"] = json!(top_p);
            }
            if let Some(max_tokens) = parameters.max_tokens {
                options["num_predict"] = json!(max_tokens);
            }
            if let Some(stop) = &parameters.stop {
                options["stop"] = json!(stop);
            }

            if !options.as_object().unwrap().is_empty() {
                payload["options"] = options;
            }
        }

        payload
    }

    /// Parse Ollama API response
    fn parse_response(&self, response_json: serde_json::Value) -> crate::Result<ChatResponse> {
        let response_text = response_json["response"]
            .as_str()
            .unwrap_or_default()
            .to_string();

        let model = response_json["model"]
            .as_str()
            .unwrap_or("llama2")
            .to_string();

        let choices = vec![ChatChoice {
            index: 0,
            role: MessageRole::Assistant,
            content: Some(response_text),
            finish_reason: if response_json["done"].as_bool().unwrap_or(false) {
                Some("stop".to_string())
            } else {
                None
            },
            tool_calls: None,
        }];

        // Ollama doesn't provide detailed token usage, so we estimate
        let usage = if let Some(eval_count) = response_json["eval_count"].as_u64() {
            Some(UsageInfo {
                prompt_tokens: response_json["prompt_eval_count"].as_u64().unwrap_or(0) as u32,
                completion_tokens: eval_count as u32,
                total_tokens: (response_json["prompt_eval_count"].as_u64().unwrap_or(0)
                    + eval_count) as u32,
            })
        } else {
            None
        };

        Ok(ChatResponse {
            id: format!("ollama-{}", uuid::Uuid::new_v4()),
            model,
            choices,
            usage,
        })
    }
}

#[async_trait]
impl AIClient for OllamaClient {
    async fn chat(&self, request: ChatRequest) -> crate::Result<ChatResponse> {
        let url = format!("{}/api/generate", self.endpoint);
        let payload = self.convert_request(&request);

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| crate::error::AIError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(crate::error::AIError::ApiError(format!(
                "Ollama API error: {} - {}",
                response.status(),
                response.text().await.unwrap_or_default()
            )));
        }

        let response_json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| crate::error::AIError::ParseError(e.to_string()))?;

        self.parse_response(response_json)
    }

    async fn list_models(&self) -> crate::Result<Vec<String>> {
        let url = format!("{}/api/tags", self.endpoint);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| crate::error::AIError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(crate::error::AIError::ApiError(format!(
                "Ollama API error: {}",
                response.status()
            )));
        }

        let response_json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| crate::error::AIError::ParseError(e.to_string()))?;

        let models = response_json["models"]
            .as_array()
            .ok_or_else(|| crate::error::AIError::ParseError("No models data".to_string()))?
            .iter()
            .filter_map(|model| model["name"].as_str().map(|s| s.to_string()))
            .collect();

        Ok(models)
    }

    async fn is_available(&self) -> bool {
        let url = format!("{}/api/version", self.endpoint);

        self.client
            .get(&url)
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }

    fn provider_name(&self) -> &str {
        "ollama"
    }

    fn default_model(&self) -> &str {
        "llama2"
    }

    async fn get_capabilities(&self, model: &str) -> crate::Result<AICapabilities> {
        let mut capabilities = AICapabilities::default();
        capabilities.add_model_type(ModelType::LargeLanguageModel);
        capabilities.add_task_type(TaskType::TextGeneration);
        capabilities.add_task_type(TaskType::ChatCompletion);
        capabilities.add_task_type(TaskType::CodeGeneration);

        // Model-specific capabilities
        match model {
            m if m.starts_with("codellama") => {
                capabilities.max_context_size = 16384;
                capabilities.add_task_type(TaskType::CodeGeneration);
            }
            m if m.starts_with("llama2") => {
                capabilities.max_context_size = 4096;
            }
            m if m.starts_with("mistral") => {
                capabilities.max_context_size = 8192;
            }
            m if m.starts_with("phi") => {
                capabilities.max_context_size = 2048;
            }
            _ => {
                capabilities.max_context_size = 4096;
            }
        }

        capabilities.supports_streaming = true;
        capabilities.supports_function_calling = false; // Ollama doesn't support function calling
        capabilities.supports_images = false; // Most Ollama models don't support images

        Ok(capabilities)
    }

    async fn chat_stream(&self, request: ChatRequest) -> crate::Result<ChatResponseStream> {
        // For now, return a simple stream that yields the regular chat response
        // In a production implementation, this would use Ollama's streaming API
        let response = self.chat(request).await?;
        let chunk = ChatResponseChunk {
            id: response.id,
            model: response.model,
            choices: response
                .choices
                .into_iter()
                .map(|choice| ChatChoiceChunk {
                    index: choice.index,
                    delta: ChatMessage {
                        role: choice.role,
                        content: choice.content,
                        name: None,
                        tool_calls: choice.tool_calls,
                        tool_call_id: None,
                    },
                    finish_reason: choice.finish_reason,
                })
                .collect(),
        };

        let stream = stream::once(async move { Ok(chunk) });
        Ok(Box::pin(stream))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn capabilities(&self) -> AICapabilities {
        let mut capabilities = AICapabilities::default();
        capabilities.add_model_type(ModelType::LargeLanguageModel);
        capabilities.add_task_type(TaskType::TextGeneration);
        capabilities.add_task_type(TaskType::ChatCompletion);
        capabilities.max_context_size = 4096;
        capabilities.supports_streaming = true;
        capabilities.supports_function_calling = false;
        capabilities.supports_images = false;
        capabilities
    }

    fn priority(&self) -> u32 {
        110 // Lower priority for Ollama (local models)
    }

    fn estimate_cost(&self, _task: &crate::common::capability::AITask) -> f64 {
        // Ollama is free to use (local models), so cost is always 0
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::types::ChatRequest;

    #[test]
    fn test_ollama_client_creation() {
        let client = OllamaClient::new("http://localhost:11434".to_string());
        assert_eq!(client.endpoint(), "http://localhost:11434");
        assert_eq!(client.provider_name(), "ollama");
        assert_eq!(client.default_model(), "llama2");
    }

    #[test]
    fn test_convert_request() {
        let client = OllamaClient::new("http://localhost:11434".to_string());
        let request = ChatRequest::new()
            .add_system("You are a helpful assistant")
            .add_user("Hello")
            .with_model("llama2");

        let payload = client.convert_request(&request);
        assert_eq!(payload["model"], "llama2");
        assert!(payload["prompt"]
            .as_str()
            .unwrap()
            .contains("System: You are a helpful assistant"));
        assert!(payload["prompt"].as_str().unwrap().contains("User: Hello"));
    }

    #[tokio::test]
    async fn test_capabilities() {
        let client = OllamaClient::new("http://localhost:11434".to_string());
        let capabilities = client.get_capabilities("llama2").await.unwrap();

        assert!(capabilities.supports_model_type(&ModelType::LargeLanguageModel));
        assert!(capabilities.supports_task(&TaskType::TextGeneration));
        assert!(capabilities.supports_streaming);
        assert!(!capabilities.supports_function_calling);
        assert!(!capabilities.supports_images);
        assert_eq!(capabilities.max_context_size, 4096);
    }

    #[test]
    fn test_cost_estimation() {
        let client = OllamaClient::new("http://localhost:11434".to_string());
        let task = crate::common::capability::AITask {
            task_type: TaskType::TextGeneration,
            required_model_type: Some(ModelType::LargeLanguageModel),
            min_context_size: None,
            requires_streaming: false,
            requires_function_calling: false,
            requires_tool_use: false,
            security_requirements: SecurityRequirements::default(),
            complexity_score: Some(50),
            priority: 50,
        };

        let cost = client.estimate_cost(&task);
        assert_eq!(cost, 0.0); // Ollama is free
    }
}
