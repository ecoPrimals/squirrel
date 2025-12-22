//! OpenAI client implementation
//!
//! This module provides a production-ready OpenAI client that implements
//! the AIClient trait for seamless integration with the AI tools system.

use async_trait::async_trait;
use futures::stream;
use reqwest::Client;
use serde_json::json;
use universal_error::tools::AIToolsError;

use crate::common::capability::{AICapabilities, ModelType, TaskType};
use crate::common::client::AIClient;
use crate::common::types::{
    ChatChoice, ChatChoiceChunk, ChatMessage, ChatRequest, ChatResponse, ChatResponseChunk,
    ChatResponseStream, MessageRole, UsageInfo,
};

/// Production OpenAI client
#[derive(Debug)]
pub struct OpenAIClient {
    /// API key for authentication
    api_key: String,
    /// Base URL for the API
    base_url: String,
    /// HTTP client for making requests
    client: Client,
}

impl OpenAIClient {
    /// Create a new OpenAI client
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            base_url: "https://api.openai.com/v1".to_string(),
            client: Client::new(),
        }
    }

    /// Create a new OpenAI client with custom base URL
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

    /// Create authorization header
    fn auth_header(&self) -> String {
        format!("Bearer {}", self.api_key)
    }

    /// Convert internal ChatRequest to OpenAI API format
    fn convert_request(&self, request: &ChatRequest) -> serde_json::Value {
        let mut payload = json!({
            "model": request.model.as_deref().unwrap_or("gpt-3.5-turbo"),
            "messages": request.messages.iter().map(|msg| {
                let mut message = json!({
                    "role": match msg.role {
                        MessageRole::System => "system",
                        MessageRole::User => "user",
                        MessageRole::Assistant => "assistant",
                        MessageRole::Tool => "tool",
                        MessageRole::Function => "function",
                    },
                });

                if let Some(content) = &msg.content {
                    message["content"] = json!(content);
                }

                if let Some(name) = &msg.name {
                    message["name"] = json!(name);
                }

                if let Some(tool_calls) = &msg.tool_calls {
                    message["tool_calls"] = json!(tool_calls);
                }

                if let Some(tool_call_id) = &msg.tool_call_id {
                    message["tool_call_id"] = json!(tool_call_id);
                }

                message
            }).collect::<Vec<_>>(),
        });

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
            if let Some(frequency_penalty) = parameters.frequency_penalty {
                payload["frequency_penalty"] = json!(frequency_penalty);
            }
            if let Some(presence_penalty) = parameters.presence_penalty {
                payload["presence_penalty"] = json!(presence_penalty);
            }
            if let Some(stop) = &parameters.stop {
                payload["stop"] = json!(stop);
            }
        }

        // Add tools if specified
        if let Some(tools) = &request.tools {
            payload["tools"] = json!(tools
                .iter()
                .map(|tool| {
                    json!({
                        "type": "function",
                        "function": {
                            "name": tool.name,
                            "description": tool.description,
                            "parameters": tool.parameters
                        }
                    })
                })
                .collect::<Vec<_>>());
        }

        payload
    }

    /// Parse OpenAI API response
    fn parse_response(&self, response_json: serde_json::Value) -> crate::Result<ChatResponse> {
        let id = response_json["id"]
            .as_str()
            .unwrap_or("unknown")
            .to_string();

        let model = response_json["model"]
            .as_str()
            .unwrap_or("unknown")
            .to_string();

        let choices = response_json["choices"]
            .as_array()
            .ok_or_else(|| AIToolsError::InvalidResponse(
                "OpenAI response missing 'choices' array. Response may be malformed or request invalid.".to_string()
            ))?
            .iter()
            .enumerate()
            .map(|(index, choice)| {
                let role = match choice["message"]["role"].as_str() {
                    Some("assistant") => MessageRole::Assistant,
                    Some("user") => MessageRole::User,
                    Some("system") => MessageRole::System,
                    Some("tool") => MessageRole::Tool,
                    Some("function") => MessageRole::Function,
                    _ => MessageRole::Assistant,
                };

                let content = choice["message"]["content"].as_str().map(|s| s.to_string());

                let finish_reason = choice["finish_reason"].as_str().map(|s| s.to_string());

                let tool_calls = choice["message"]["tool_calls"].as_array().map(|calls| {
                    calls
                        .iter()
                        .filter_map(|call| {
                            Some(crate::common::types::ToolCall {
                                id: call["id"].as_str()?.to_string(),
                                name: call["function"]["name"].as_str()?.to_string(),
                                arguments: call["function"]["arguments"].clone(),
                            })
                        })
                        .collect()
                });

                ChatChoice {
                    index,
                    role,
                    content,
                    finish_reason,
                    tool_calls,
                }
            })
            .collect();

        let usage = response_json["usage"].as_object().map(|usage| UsageInfo {
            prompt_tokens: usage["prompt_tokens"].as_u64().unwrap_or(0) as u32,
            completion_tokens: usage["completion_tokens"].as_u64().unwrap_or(0) as u32,
            total_tokens: usage["total_tokens"].as_u64().unwrap_or(0) as u32,
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
impl AIClient for OpenAIClient {
    async fn chat(&self, request: ChatRequest) -> crate::Result<ChatResponse> {
        let url = format!("{}/chat/completions", self.base_url);
        let payload = self.convert_request(&request);

        let response = self
            .client
            .post(&url)
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| {
                AIToolsError::Network(format!(
                    "Failed to reach OpenAI API: {}. Check network connectivity and endpoint.",
                    e
                ))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(AIToolsError::Api(format!(
                "OpenAI API error (status {}): {}. Verify API key at platform.openai.com.",
                status, error_text
            ))
            .into());
        }

        let response_json: serde_json::Value = response.json().await.map_err(|e| {
            AIToolsError::Parse(format!(
                "Failed to parse OpenAI response: {}. Response may be malformed.",
                e
            ))
        })?;

        self.parse_response(response_json)
    }

    async fn list_models(&self) -> crate::Result<Vec<String>> {
        let url = format!("{}/models", self.base_url);

        let response = self
            .client
            .get(&url)
            .header("Authorization", self.auth_header())
            .send()
            .await
            .map_err(|e| {
                AIToolsError::Network(format!(
                    "Failed to fetch OpenAI models: {}. Check network connectivity.",
                    e
                ))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            return Err(AIToolsError::Api(format!(
                "OpenAI API error fetching models (status {}). Verify API key at platform.openai.com.",
                status
            )).into());
        }

        let response_json: serde_json::Value = response.json().await.map_err(|e| {
            AIToolsError::Parse(format!(
                "Failed to parse OpenAI models response: {}. Response may be malformed.",
                e
            ))
        })?;

        let models = response_json["data"]
            .as_array()
            .ok_or_else(|| {
                AIToolsError::InvalidResponse(
                    "OpenAI models response missing 'data' array. API may have changed format."
                        .to_string(),
                )
            })?
            .iter()
            .filter_map(|model| model["id"].as_str().map(|s| s.to_string()))
            .collect();

        Ok(models)
    }

    async fn is_available(&self) -> bool {
        self.list_models().await.is_ok()
    }

    fn provider_name(&self) -> &str {
        "openai"
    }

    fn default_model(&self) -> &str {
        "gpt-3.5-turbo"
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
            "gpt-4" | "gpt-4-turbo" => {
                capabilities.max_context_size = 128000;
                capabilities.supports_function_calling = true;
            }
            "gpt-4-vision-preview" => {
                capabilities.max_context_size = 128000;
                capabilities.supports_function_calling = true;
                capabilities.supports_images = true;
                capabilities.add_task_type(TaskType::ImageAnalysis);
            }
            "gpt-3.5-turbo" => {
                capabilities.max_context_size = 16384;
                capabilities.supports_function_calling = true;
            }
            _ => {
                capabilities.max_context_size = 16384;
                capabilities.supports_function_calling = true;
            }
        }

        capabilities.supports_streaming = true;
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
        capabilities.max_context_size = 16384;
        capabilities.supports_streaming = true;
        capabilities.supports_function_calling = true;
        capabilities
    }

    fn priority(&self) -> u32 {
        150 // Higher priority for OpenAI
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::types::ChatRequest;

    #[test]
    fn test_openai_client_creation() {
        let client = OpenAIClient::new("test-key".to_string());
        assert_eq!(client.api_key(), "test-key");
        assert_eq!(client.base_url(), "https://api.openai.com/v1");
        assert_eq!(client.provider_name(), "openai");
        assert_eq!(client.default_model(), "gpt-3.5-turbo");
    }

    #[test]
    fn test_openai_client_with_base_url() {
        let client = OpenAIClient::with_base_url(
            "test-key".to_string(),
            "https://custom.openai.com/v1".to_string(),
        );
        assert_eq!(client.base_url(), "https://custom.openai.com/v1");
    }

    #[test]
    fn test_convert_request() {
        let client = OpenAIClient::new("test-key".to_string());
        let request = ChatRequest::new()
            .add_system("You are a helpful assistant")
            .add_user("Hello")
            .with_model("gpt-4");

        let payload = client.convert_request(&request);
        assert_eq!(payload["model"], "gpt-4");
        assert_eq!(payload["messages"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_auth_header() {
        let client = OpenAIClient::new("test-key".to_string());
        assert_eq!(client.auth_header(), "Bearer test-key");
    }

    #[tokio::test]
    async fn test_capabilities() {
        let client = OpenAIClient::new("test-key".to_string());
        let capabilities = client.get_capabilities("gpt-4").await.unwrap();

        assert!(capabilities.supports_model_type(&ModelType::LargeLanguageModel));
        assert!(capabilities.supports_task(&TaskType::TextGeneration));
        assert!(capabilities.supports_streaming);
        assert!(capabilities.supports_function_calling);
        assert_eq!(capabilities.max_context_size, 128000);
    }
}
