//! Anthropic client implementation
//!
//! This module provides a production-ready Anthropic client that implements
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

/// Production Anthropic client
#[derive(Debug)]
pub struct AnthropicClient {
    /// API key for authentication
    api_key: String,
    /// Base URL for the API
    base_url: String,
    /// HTTP client for making requests
    client: Client,
}

impl AnthropicClient {
    /// Create a new Anthropic client
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            base_url: "https://api.anthropic.com/v1".to_string(),
            client: Client::new(),
        }
    }

    /// Create a new Anthropic client with custom base URL
    pub fn with_base_url(api_key: String, base_url: String) -> Self {
        Self {
            api_key,
            base_url,
            client: Client::new(),
        }
    }

    /// Set custom HTTP client
    pub fn with_client(mut self, client: Client) -> Self {
        self.client = client;
        self
    }

    /// Get the API key (for testing purposes)
    pub fn api_key(&self) -> &str {
        &self.api_key
    }

    /// Get the base URL
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Convert internal ChatRequest to Anthropic API format
    fn convert_request(&self, request: &ChatRequest) -> serde_json::Value {
        let mut payload = json!({
            "model": request.model.as_deref().unwrap_or("claude-3-sonnet-20240229"),
            "messages": request.messages.iter().filter_map(|msg| {
                // Skip system messages - they're handled separately in Anthropic API
                if msg.role == MessageRole::System {
                    return None;
                }

                let mut message = json!({
                    "role": match msg.role {
                        MessageRole::User => "user",
                        MessageRole::Assistant => "assistant",
                        MessageRole::Tool => "tool",
                        MessageRole::Function => "user", // Anthropic doesn't have function role, map to user
                        MessageRole::System => "user", // Fallback, shouldn't happen
                    },
                });

                if let Some(content) = &msg.content {
                    message["content"] = json!(content);
                }

                Some(message)
            }).collect::<Vec<_>>(),
            "max_tokens": 1000,
        });

        // Handle system messages separately
        let system_messages: Vec<String> = request
            .messages
            .iter()
            .filter_map(|msg| {
                if msg.role == MessageRole::System {
                    msg.content.clone()
                } else {
                    None
                }
            })
            .collect();

        if !system_messages.is_empty() {
            payload["system"] = json!(system_messages.join("\n"));
        }

        // Add parameters if specified
        if let Some(parameters) = &request.parameters {
            if let Some(temperature) = parameters.temperature {
                payload["temperature"] = json!(temperature);
            }
            if let Some(max_tokens) = parameters.max_tokens {
                payload["max_tokens"] = json!(max_tokens);
            }
            if let Some(top_p) = parameters.top_p {
                payload["top_p"] = json!(top_p);
            }
            if let Some(stop) = &parameters.stop {
                payload["stop_sequences"] = json!(stop);
            }
        }

        payload
    }

    /// Parse Anthropic API response
    fn parse_response(&self, response_json: serde_json::Value) -> crate::Result<ChatResponse> {
        let id = response_json["id"]
            .as_str()
            .unwrap_or("unknown")
            .to_string();

        let model = response_json["model"]
            .as_str()
            .unwrap_or("claude-3-sonnet-20240229")
            .to_string();

        // Anthropic returns a single content field, not choices
        let content = response_json["content"]
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|item| item["text"].as_str())
            .unwrap_or_default()
            .to_string();

        let choices = vec![ChatChoice {
            index: 0,
            role: MessageRole::Assistant,
            content: Some(content),
            finish_reason: response_json["stop_reason"].as_str().map(|s| s.to_string()),
            tool_calls: None,
        }];

        let usage = response_json["usage"].as_object().map(|usage| UsageInfo {
            prompt_tokens: usage["input_tokens"].as_u64().unwrap_or(0) as u32,
            completion_tokens: usage["output_tokens"].as_u64().unwrap_or(0) as u32,
            total_tokens: (usage["input_tokens"].as_u64().unwrap_or(0)
                + usage["output_tokens"].as_u64().unwrap_or(0)) as u32,
        });

        Ok(ChatResponse {
            id,
            model,
            choices,
            usage,
        })
    }
}

#[async_trait]
impl AIClient for AnthropicClient {
    async fn chat(&self, request: ChatRequest) -> crate::Result<ChatResponse> {
        let url = format!("{}/messages", self.base_url);
        let payload = self.convert_request(&request);

        let response = self
            .client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("Content-Type", "application/json")
            .header("anthropic-version", "2023-06-01")
            .json(&payload)
            .send()
            .await
            .map_err(|e| crate::error::AIError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(crate::error::AIError::ApiError(format!(
                "Anthropic API error: {} - {}",
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
        // Anthropic doesn't have a public models endpoint, so we return known models
        Ok(vec![
            "claude-3-5-sonnet-20241022".to_string(),
            "claude-3-sonnet-20240229".to_string(),
            "claude-3-haiku-20240307".to_string(),
            "claude-3-opus-20240229".to_string(),
            "claude-2.1".to_string(),
            "claude-2.0".to_string(),
            "claude-instant-1.2".to_string(),
        ])
    }

    async fn is_available(&self) -> bool {
        // Simple health check - try to make a minimal request
        let request = ChatRequest::new()
            .add_user("Hi")
            .with_model("claude-3-haiku-20240307");

        self.chat(request).await.is_ok()
    }

    fn provider_name(&self) -> &str {
        "anthropic"
    }

    fn default_model(&self) -> &str {
        "claude-3-sonnet-20240229"
    }

    async fn get_capabilities(&self, model: &str) -> crate::Result<AICapabilities> {
        let mut capabilities = AICapabilities::default();
        capabilities.add_model_type(ModelType::LargeLanguageModel);
        capabilities.add_task_type(TaskType::TextGeneration);
        capabilities.add_task_type(TaskType::ChatCompletion);
        capabilities.add_task_type(TaskType::CodeGeneration);
        capabilities.add_task_type(TaskType::Translation);
        capabilities.add_task_type(TaskType::Summarization);
        capabilities.add_task_type(TaskType::QuestionAnswering);

        // Model-specific capabilities
        match model {
            "claude-3-opus-20240229" => {
                capabilities.max_context_size = 200000;
                capabilities.supports_function_calling = true;
            }
            "claude-3-sonnet-20240229" | "claude-3-5-sonnet-20241022" => {
                capabilities.max_context_size = 200000;
                capabilities.supports_function_calling = true;
            }
            "claude-3-haiku-20240307" => {
                capabilities.max_context_size = 200000;
                capabilities.supports_function_calling = true;
            }
            "claude-2.1" | "claude-2.0" => {
                capabilities.max_context_size = 100000;
                capabilities.supports_function_calling = false;
            }
            "claude-instant-1.2" => {
                capabilities.max_context_size = 100000;
                capabilities.supports_function_calling = false;
            }
            _ => {
                capabilities.max_context_size = 100000;
                capabilities.supports_function_calling = false;
            }
        }

        capabilities.supports_streaming = true;
        capabilities.supports_images = model.contains("claude-3");

        if capabilities.supports_images {
            capabilities.add_task_type(TaskType::ImageAnalysis);
        }

        Ok(capabilities)
    }

    async fn chat_stream(&self, request: ChatRequest) -> crate::Result<ChatResponseStream> {
        // For now, return a simple stream that yields the regular chat response
        // In a production implementation, this would use Server-Sent Events (SSE)
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
        capabilities.add_task_type(TaskType::CodeGeneration);
        capabilities.max_context_size = 200000;
        capabilities.supports_streaming = true;
        capabilities.supports_function_calling = true;
        capabilities.supports_images = true;
        capabilities
    }

    fn priority(&self) -> u32 {
        140 // High priority for Anthropic
    }

    fn estimate_cost(&self, task: &crate::common::capability::AITask) -> f64 {
        // Anthropic-specific cost estimation
        let base_cost = match task.task_type {
            TaskType::TextGeneration => 0.015,
            TaskType::CodeGeneration => 0.025,
            TaskType::Translation => 0.02,
            TaskType::Summarization => 0.012,
            TaskType::QuestionAnswering => 0.018,
            TaskType::ChatCompletion => 0.015,
            TaskType::ImageAnalysis => 0.2,
            _ => 0.015,
        };

        let token_multiplier = if let Some(min_tokens) = task.min_context_size {
            (min_tokens as f64) / 1000.0
        } else {
            1.0
        };

        base_cost * token_multiplier
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::types::ChatRequest;

    #[test]
    fn test_anthropic_client_creation() {
        let client = AnthropicClient::new("test-key".to_string());
        assert_eq!(client.api_key(), "test-key");
        assert_eq!(client.base_url(), "https://api.anthropic.com/v1");
        assert_eq!(client.provider_name(), "anthropic");
        assert_eq!(client.default_model(), "claude-3-sonnet-20240229");
    }

    #[test]
    fn test_anthropic_client_with_base_url() {
        let client = AnthropicClient::with_base_url(
            "test-key".to_string(),
            "https://custom.anthropic.com/v1".to_string(),
        );
        assert_eq!(client.base_url(), "https://custom.anthropic.com/v1");
    }

    #[test]
    fn test_convert_request() {
        let client = AnthropicClient::new("test-key".to_string());
        let request = ChatRequest::new()
            .add_system("You are a helpful assistant")
            .add_user("Hello")
            .with_model("claude-3-sonnet-20240229");

        let payload = client.convert_request(&request);
        assert_eq!(payload["model"], "claude-3-sonnet-20240229");
        assert_eq!(payload["messages"].as_array().unwrap().len(), 1); // Only user message
        assert_eq!(payload["system"], "You are a helpful assistant");
    }

    #[tokio::test]
    async fn test_list_models() {
        let client = AnthropicClient::new("test-key".to_string());
        let models = client.list_models().await.unwrap();

        assert!(!models.is_empty());
        assert!(models.contains(&"claude-3-sonnet-20240229".to_string()));
        assert!(models.contains(&"claude-3-haiku-20240307".to_string()));
    }

    #[tokio::test]
    async fn test_capabilities() {
        let client = AnthropicClient::new("test-key".to_string());
        let capabilities = client
            .get_capabilities("claude-3-sonnet-20240229")
            .await
            .unwrap();

        assert!(capabilities.supports_model_type(&ModelType::LargeLanguageModel));
        assert!(capabilities.supports_task(&TaskType::TextGeneration));
        assert!(capabilities.supports_streaming);
        assert!(capabilities.supports_function_calling);
        assert!(capabilities.supports_images);
        assert_eq!(capabilities.max_context_size, 200000);
    }

    #[test]
    fn test_cost_estimation() {
        let client = AnthropicClient::new("test-key".to_string());
        let task = crate::common::capability::AITask {
            task_type: TaskType::TextGeneration,
            required_model_type: Some(ModelType::LargeLanguageModel),
            min_context_size: Some(1000),
            requires_streaming: false,
            requires_function_calling: false,
            requires_tool_use: false,
            security_requirements: SecurityRequirements::default(),
            complexity_score: Some(50),
            priority: 100,
        };

        let cost = client.estimate_cost(&task);
        assert!(cost > 0.0);
    }
}
