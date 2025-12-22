//! Mock client implementation for testing
//!
//! This module provides a mock AI client that can be used for testing
//! without making actual API calls to external services.

use async_trait::async_trait;
use futures::stream;
use std::collections::HashMap;

use crate::common::capability::{AICapabilities, ModelType, TaskType};
use crate::common::client::AIClient;
use crate::common::types::{
    ChatChoice, ChatChoiceChunk, ChatMessage, ChatRequest, ChatResponse, ChatResponseChunk,
    ChatResponseStream, MessageRole, UsageInfo,
};

/// Mock AI client for testing
#[derive(Debug)]
pub struct MockAIClient {
    /// Mock responses to return
    responses: HashMap<String, String>,
    /// Whether the client should simulate being available
    available: bool,
    /// Mock latency in milliseconds
    latency_ms: u64,
}

impl MockAIClient {
    /// Create a new mock client
    pub fn new() -> Self {
        let mut responses = HashMap::new();
        responses.insert(
            "default".to_string(),
            "This is a mock response from the AI client.".to_string(),
        );
        responses.insert(
            "error".to_string(),
            "This is an error response.".to_string(),
        );
        responses.insert(
            "hello".to_string(),
            "Hello! How can I help you today?".to_string(),
        );
        responses.insert("code".to_string(), "Here's some example code:\n\n```rust\nfn main() {\n    println!(\"Hello, World!\");\n}\n```".to_string());

        Self {
            responses,
            available: true,
            latency_ms: 100,
        }
    }

    /// Create a mock client that simulates being unavailable
    pub fn unavailable() -> Self {
        let mut client = Self::new();
        client.available = false;
        client
    }

    /// Set a custom response for a specific input
    pub fn with_response(mut self, input: &str, response: &str) -> Self {
        self.responses
            .insert(input.to_lowercase(), response.to_string());
        self
    }

    /// Set the mock latency
    pub fn with_latency(mut self, latency_ms: u64) -> Self {
        self.latency_ms = latency_ms;
        self
    }

    /// Set availability status
    pub fn with_availability(mut self, available: bool) -> Self {
        self.available = available;
        self
    }

    /// Get a mock response based on the input
    fn get_mock_response(&self, input: &str) -> String {
        let key = input.to_lowercase();

        // Look for specific responses
        if let Some(response) = self.responses.get(&key) {
            return response.clone();
        }

        // Check for partial matches
        if key.contains("hello") || key.contains("hi") {
            return self.responses.get("hello").cloned().unwrap_or_default();
        }

        if key.contains("code") || key.contains("programming") {
            return self.responses.get("code").cloned().unwrap_or_default();
        }

        if key.contains("error") || key.contains("fail") {
            return self.responses.get("error").cloned().unwrap_or_default();
        }

        // Default response
        self.responses.get("default").cloned().unwrap_or_default()
    }

    /// Simulate network latency
    async fn simulate_latency(&self) {
        if self.latency_ms > 0 {
            tokio::time::sleep(tokio::time::Duration::from_millis(self.latency_ms)).await;
        }
    }
}

impl Default for MockAIClient {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AIClient for MockAIClient {
    async fn chat(&self, request: ChatRequest) -> crate::Result<ChatResponse> {
        self.simulate_latency().await;

        if !self.available {
            return Err(universal_error::tools::AIToolsError::Network(
                "Mock client is unavailable".to_string(),
            )
            .into());
        }

        // Extract the last user message as input
        let input = request
            .messages
            .iter()
            .filter(|msg| msg.role == MessageRole::User)
            .next_back()
            .and_then(|msg| msg.content.as_ref())
            .map_or("default", |v| v);

        let response_content = self.get_mock_response(input);
        let model = request.model.as_deref().unwrap_or("mock-model");

        let choices = vec![ChatChoice {
            index: 0,
            role: MessageRole::Assistant,
            content: Some(response_content.clone()),
            finish_reason: Some("stop".to_string()),
            tool_calls: None,
        }];

        // Mock usage information
        let usage = Some(UsageInfo {
            prompt_tokens: input.len() as u32 / 4, // Rough estimate
            completion_tokens: response_content.len() as u32 / 4, // Rough estimate
            total_tokens: (input.len() + response_content.len()) as u32 / 4,
        });

        Ok(ChatResponse {
            id: format!("mock-{}", uuid::Uuid::new_v4()),
            model: model.to_string(),
            choices,
            usage,
        })
    }

    async fn list_models(&self) -> crate::Result<Vec<String>> {
        self.simulate_latency().await;

        if !self.available {
            return Err(universal_error::tools::AIToolsError::Network(
                "Mock client is unavailable".to_string(),
            )
            .into());
        }

        Ok(vec![
            "mock-model".to_string(),
            "mock-gpt-4".to_string(),
            "mock-claude-3".to_string(),
            "mock-llama-2".to_string(),
        ])
    }

    async fn is_available(&self) -> bool {
        self.simulate_latency().await;
        self.available
    }

    fn provider_name(&self) -> &str {
        "mock"
    }

    fn default_model(&self) -> &str {
        "mock-model"
    }

    async fn get_capabilities(&self, model: &str) -> crate::Result<AICapabilities> {
        self.simulate_latency().await;

        if !self.available {
            return Err(universal_error::tools::AIToolsError::Network(
                "Mock client is unavailable".to_string(),
            )
            .into());
        }

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
            "mock-gpt-4" => {
                capabilities.max_context_size = 128000;
                capabilities.supports_function_calling = true;
                capabilities.supports_images = true;
                capabilities.add_task_type(TaskType::ImageAnalysis);
                capabilities.add_task_type(TaskType::FunctionCalling);
            }
            "mock-claude-3" => {
                capabilities.max_context_size = 200000;
                capabilities.supports_function_calling = true;
                capabilities.supports_images = true;
                capabilities.add_task_type(TaskType::ImageAnalysis);
                capabilities.add_task_type(TaskType::FunctionCalling);
            }
            "mock-llama-2" => {
                capabilities.max_context_size = 4096;
                capabilities.supports_function_calling = false;
                capabilities.supports_images = false;
            }
            _ => {
                capabilities.max_context_size = 16384;
                capabilities.supports_function_calling = true;
                capabilities.supports_images = false;
            }
        }

        capabilities.supports_streaming = true;
        Ok(capabilities)
    }

    async fn chat_stream(&self, request: ChatRequest) -> crate::Result<ChatResponseStream> {
        self.simulate_latency().await;

        if !self.available {
            return Err(universal_error::tools::AIToolsError::Network(
                "Mock client is unavailable".to_string(),
            )
            .into());
        }

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
        capabilities.supports_images = false;
        capabilities
    }

    fn priority(&self) -> u32 {
        50 // Low priority for mock client
    }

    fn estimate_cost(&self, _task: &crate::common::capability::AITask) -> f64 {
        // Mock client is free
        0.0
    }

    async fn health_score(&self) -> f64 {
        self.simulate_latency().await;
        if self.available {
            1.0
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::types::ChatRequest;

    #[test]
    fn test_mock_client_creation() {
        let client = MockAIClient::new();
        assert_eq!(client.provider_name(), "mock");
        assert_eq!(client.default_model(), "mock-model");
        assert!(client.available);
    }

    #[test]
    fn test_mock_client_unavailable() {
        let client = MockAIClient::unavailable();
        assert!(!client.available);
    }

    #[test]
    fn test_mock_client_with_response() {
        let client = MockAIClient::new().with_response("test", "custom response");

        let response = client.get_mock_response("test");
        assert_eq!(response, "custom response");
    }

    #[test]
    fn test_mock_client_with_latency() {
        let client = MockAIClient::new().with_latency(500);

        assert_eq!(client.latency_ms, 500);
    }

    #[tokio::test]
    async fn test_mock_client_chat() {
        let client = MockAIClient::new().with_response("hello", "Hi there!");

        let request = ChatRequest::new()
            .add_user("hello")
            .with_model("mock-model");

        let response = client.chat(request).await.unwrap();
        assert_eq!(response.choices.len(), 1);
        assert_eq!(response.choices[0].content, Some("Hi there!".to_string()));
        assert_eq!(response.model, "mock-model");
        assert!(response.usage.is_some());
    }

    #[tokio::test]
    async fn test_mock_client_list_models() {
        let client = MockAIClient::new();
        let models = client.list_models().await.unwrap();

        assert!(!models.is_empty());
        assert!(models.contains(&"mock-model".to_string()));
        assert!(models.contains(&"mock-gpt-4".to_string()));
    }

    #[tokio::test]
    async fn test_mock_client_unavailable_chat() {
        let client = MockAIClient::unavailable();
        let request = ChatRequest::new().add_user("hello");

        let result = client.chat(request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_mock_client_capabilities() {
        let client = MockAIClient::new();
        let capabilities = client.get_capabilities("mock-gpt-4").await.unwrap();

        assert!(capabilities.supports_model_type(&ModelType::LargeLanguageModel));
        assert!(capabilities.supports_task(&TaskType::TextGeneration));
        assert!(capabilities.supports_streaming);
        assert!(capabilities.supports_function_calling);
        assert!(capabilities.supports_images);
        assert_eq!(capabilities.max_context_size, 128000);
    }

    #[tokio::test]
    async fn test_mock_client_health_score() {
        let client = MockAIClient::new();
        let score = client.health_score().await;
        assert_eq!(score, 1.0);

        let unavailable_client = MockAIClient::unavailable();
        let score = unavailable_client.health_score().await;
        assert_eq!(score, 0.0);
    }

    #[test]
    fn test_mock_response_matching() {
        let client = MockAIClient::new();

        // Test exact match
        let response = client.get_mock_response("hello");
        assert!(response.contains("Hello"));

        // Test partial match
        let response = client.get_mock_response("Hi there!");
        assert!(response.contains("Hello"));

        // Test code match
        let response = client.get_mock_response("Show me some code");
        assert!(response.contains("```rust"));

        // Test default response
        let response = client.get_mock_response("random input");
        assert!(response.contains("mock response"));
    }
}
