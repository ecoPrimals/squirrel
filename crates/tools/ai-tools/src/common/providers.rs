//! AI provider implementations
//!
//! This module contains concrete implementations of AI providers including
//! OpenAI, Anthropic, Ollama, and other AI service providers.

use crate::ProviderConfig;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::{debug, error};
use universal_error::tools::AIToolsError;

use super::{ChatChoice, ChatRequest, ChatResponse, MessageRole, UsageInfo};

/// Trait for AI providers
#[async_trait]
pub trait AIProvider: Send + Sync + std::fmt::Debug {
    /// Process a chat request
    async fn process_chat(&self, request: &ChatRequest) -> crate::Result<ChatResponse>;

    /// Get provider name
    fn name(&self) -> &str;

    /// Check if provider is healthy
    async fn health_check(&self) -> bool;

    /// Get provider capabilities
    fn capabilities(&self) -> &[AICapability];
}

/// AI provider capabilities
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AICapability {
    /// Chat completion capability
    Chat,
    /// Text completion capability
    Completion,
    /// Text embedding capability
    Embedding,
    /// Image generation capability
    ImageGeneration,
    /// Code generation capability
    CodeGeneration,
    /// Local processing capability
    LocalProcessing,
}

/// OpenAI provider implementation
#[derive(Debug)]
pub struct OpenAIProvider {
    config: ProviderConfig,
    client: reqwest::Client,
}

impl OpenAIProvider {
    /// Create a new OpenAI provider
    pub fn new(config: ProviderConfig) -> crate::Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| AIToolsError::Network(
                format!("Failed to initialize OpenAI HTTP client: {}. Check system resources and network configuration.", e)
            ))?;

        Ok(Self { config, client })
    }

    /// Convert internal chat request to OpenAI format
    fn convert_request(&self, request: &ChatRequest) -> serde_json::Value {
        let model = request.model.as_deref().unwrap_or("gpt-3.5-turbo");

        let messages: Vec<serde_json::Value> = request
            .messages
            .iter()
            .map(|msg| {
                let role = match msg.role {
                    MessageRole::User => "user",
                    MessageRole::Assistant => "assistant",
                    MessageRole::System => "system",
                    MessageRole::Function => "function",
                    MessageRole::Tool => "tool",
                };

                serde_json::json!({
                    "role": role,
                    "content": msg.content.as_ref().unwrap_or(&String::new())
                })
            })
            .collect();

        let mut request_body = serde_json::json!({
            "model": model,
            "messages": messages
        });

        // Add parameters if provided
        if let Some(params) = &request.parameters {
            if let Some(temp) = params.temperature {
                request_body["temperature"] = serde_json::Value::from(temp);
            }
            if let Some(max_tokens) = params.max_tokens {
                request_body["max_tokens"] = serde_json::Value::from(max_tokens);
            }
            if let Some(top_p) = params.top_p {
                request_body["top_p"] = serde_json::Value::from(top_p);
            }
        }

        // Add tools if provided
        if let Some(tools) = &request.tools {
            let openai_tools: Vec<serde_json::Value> = tools
                .iter()
                .map(|tool| {
                    if let Some(function) = &tool.function {
                        serde_json::json!({
                            "type": "function",
                            "function": {
                                "name": function.name,
                                "description": function.description,
                                "parameters": function.parameters
                            }
                        })
                    } else {
                        serde_json::json!({
                            "type": "function",
                            "function": {
                                "name": "unknown",
                                "description": "Unknown tool",
                                "parameters": {}
                            }
                        })
                    }
                })
                .collect();
            request_body["tools"] = serde_json::Value::Array(openai_tools);
        }

        request_body
    }

    /// Convert OpenAI response to internal format
    fn convert_response(&self, response: serde_json::Value) -> crate::Result<ChatResponse> {
        let choices = response["choices"]
            .as_array()
            .ok_or_else(|| AIToolsError::InvalidResponse(
                "OpenAI response missing 'choices' array. API response format may have changed or request was invalid.".to_string()
            ))?;

        let converted_choices: std::result::Result<Vec<ChatChoice>, AIToolsError> = choices
            .iter()
            .map(|choice| {
                let message = &choice["message"];
                let content = message["content"].as_str().map(|s| s.to_string());
                let role = match message["role"].as_str() {
                    Some("assistant") => MessageRole::Assistant,
                    Some("user") => MessageRole::User,
                    Some("system") => MessageRole::System,
                    Some("function") => MessageRole::Function,
                    Some("tool") => MessageRole::Tool,
                    _ => MessageRole::Assistant,
                };

                Ok(ChatChoice {
                    index: choice["index"].as_u64().unwrap_or(0) as usize,
                    role,
                    content,
                    finish_reason: choice["finish_reason"].as_str().map(|s| s.to_string()),
                    tool_calls: None, // TODO: Parse tool calls
                })
            })
            .collect();

        let usage = response["usage"].as_object().map(|usage_obj| UsageInfo {
            prompt_tokens: usage_obj["prompt_tokens"].as_u64().unwrap_or(0) as u32,
            completion_tokens: usage_obj["completion_tokens"].as_u64().unwrap_or(0) as u32,
            total_tokens: usage_obj["total_tokens"].as_u64().unwrap_or(0) as u32,
        });

        Ok(ChatResponse {
            choices: converted_choices?,
            usage,
            model: response["model"].as_str().unwrap_or("unknown").to_string(),
            id: response["id"].as_str().unwrap_or("unknown").to_string(),
        })
    }
}

#[async_trait]
impl AIProvider for OpenAIProvider {
    async fn process_chat(&self, request: &ChatRequest) -> crate::Result<ChatResponse> {
        let request_body = self.convert_request(request);

        debug!("Sending request to OpenAI: {:?}", request_body);

        let response = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .header(
                "Authorization",
                format!("Bearer {}", self.config.api_key.as_deref().unwrap_or("")),
            )
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| AIToolsError::Network(
                format!("Failed to reach OpenAI API at https://api.openai.com: {}. Check network connectivity and API endpoint.", e)
            ))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!("OpenAI API error (status {}): {}", status, error_text);
            return Err(AIToolsError::Api(format!(
                "OpenAI API returned error status {}: {}. Verify API key at platform.openai.com and check request format.",
                status, error_text
            )).into());
        }

        let response_json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| AIToolsError::Parse(
                format!("Failed to parse OpenAI API response as JSON: {}. API may have returned unexpected format.", e)
            ))?;

        self.convert_response(response_json)
    }

    fn name(&self) -> &str {
        "openai"
    }

    async fn health_check(&self) -> bool {
        // Simple health check by listing models
        self.client
            .get("https://api.openai.com/v1/models")
            .header(
                "Authorization",
                format!("Bearer {}", self.config.api_key.as_deref().unwrap_or("")),
            )
            .send()
            .await
            .map(|response| response.status().is_success())
            .unwrap_or(false)
    }

    fn capabilities(&self) -> &[AICapability] {
        &[
            AICapability::Chat,
            AICapability::Completion,
            AICapability::Embedding,
            AICapability::CodeGeneration,
        ]
    }
}

/// Anthropic provider implementation
#[derive(Debug)]
pub struct AnthropicProvider {
    config: ProviderConfig,
    client: reqwest::Client,
}

impl AnthropicProvider {
    /// Create a new Anthropic provider
    pub fn new(config: ProviderConfig) -> crate::Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| AIToolsError::Network(
                format!("Failed to initialize Anthropic HTTP client: {}. Check system resources and network configuration.", e)
            ))?;

        Ok(Self { config, client })
    }

    /// Convert internal chat request to Anthropic format
    fn convert_request(&self, request: &ChatRequest) -> serde_json::Value {
        let model = request
            .model
            .as_deref()
            .unwrap_or("claude-3-sonnet-20240229");

        let messages: Vec<serde_json::Value> = request
            .messages
            .iter()
            .map(|msg| {
                let role = match msg.role {
                    MessageRole::User => "user",
                    MessageRole::Assistant => "assistant",
                    MessageRole::System => "user", // Anthropic handles system messages differently
                    MessageRole::Function => "user",
                    MessageRole::Tool => "user",
                };

                serde_json::json!({
                    "role": role,
                    "content": msg.content.as_ref().unwrap_or(&String::new())
                })
            })
            .collect();

        let mut request_body = serde_json::json!({
            "model": model,
            "messages": messages,
            "max_tokens": 1024
        });

        // Add parameters if provided
        if let Some(params) = &request.parameters {
            if let Some(temp) = params.temperature {
                request_body["temperature"] = serde_json::Value::from(temp);
            }
            if let Some(max_tokens) = params.max_tokens {
                request_body["max_tokens"] = serde_json::Value::from(max_tokens);
            }
            if let Some(top_p) = params.top_p {
                request_body["top_p"] = serde_json::Value::from(top_p);
            }
        }

        request_body
    }

    /// Convert Anthropic response to internal format
    fn convert_response(&self, response: serde_json::Value) -> crate::Result<ChatResponse> {
        let content = response["content"]
            .as_array()
            .ok_or_else(|| AIToolsError::InvalidResponse(
                "Anthropic response missing 'content' array. API response format may have changed or request was invalid.".to_string()
            ))?;

        let text_content = content
            .first()
            .and_then(|c| c["text"].as_str())
            .unwrap_or("")
            .to_string();

        let choice = ChatChoice {
            index: 0,
            role: MessageRole::Assistant,
            content: Some(text_content),
            finish_reason: response["stop_reason"].as_str().map(|s| s.to_string()),
            tool_calls: None,
        };

        let usage = response["usage"].as_object().map(|usage_obj| UsageInfo {
            prompt_tokens: usage_obj["input_tokens"].as_u64().unwrap_or(0) as u32,
            completion_tokens: usage_obj["output_tokens"].as_u64().unwrap_or(0) as u32,
            total_tokens: (usage_obj["input_tokens"].as_u64().unwrap_or(0)
                + usage_obj["output_tokens"].as_u64().unwrap_or(0))
                as u32,
        });

        Ok(ChatResponse {
            choices: vec![choice],
            usage,
            model: response["model"]
                .as_str()
                .unwrap_or("claude-3-sonnet-20240229")
                .to_string(),
            id: response["id"].as_str().unwrap_or("unknown").to_string(),
        })
    }
}

#[async_trait]
impl AIProvider for AnthropicProvider {
    async fn process_chat(&self, request: &ChatRequest) -> crate::Result<ChatResponse> {
        let request_body = self.convert_request(request);

        debug!("Sending request to Anthropic: {:?}", request_body);

        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", self.config.api_key.as_deref().unwrap_or(""))
            .header("Content-Type", "application/json")
            .header("anthropic-version", "2023-06-01")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| AIToolsError::Network(
                format!("Failed to reach Anthropic API at https://api.anthropic.com: {}. Check network connectivity and API endpoint.", e)
            ))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!("Anthropic API error (status {}): {}", status, error_text);
            return Err(AIToolsError::Api(format!(
                "Anthropic API returned error status {}: {}. Verify API key at console.anthropic.com and check request format.",
                status, error_text
            )).into());
        }

        let response_json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| AIToolsError::Parse(
                format!("Failed to parse Anthropic API response as JSON: {}. API may have returned unexpected format.", e)
            ))?;

        self.convert_response(response_json)
    }

    fn name(&self) -> &str {
        "anthropic"
    }

    async fn health_check(&self) -> bool {
        // Simple health check - attempt to get account info
        self.client
            .get("https://api.anthropic.com/v1/messages")
            .header("x-api-key", self.config.api_key.as_deref().unwrap_or(""))
            .header("anthropic-version", "2023-06-01")
            .send()
            .await
            .map(|response| response.status().as_u16() != 401) // Not unauthorized
            .unwrap_or(false)
    }

    fn capabilities(&self) -> &[AICapability] {
        &[
            AICapability::Chat,
            AICapability::Completion,
            AICapability::CodeGeneration,
        ]
    }
}

/// Ollama provider implementation
#[derive(Debug)]
pub struct OllamaProvider {
    config: ProviderConfig,
    client: reqwest::Client,
}

impl OllamaProvider {
    /// Create a new Ollama provider
    pub fn new(config: ProviderConfig) -> crate::Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(120)) // Longer timeout for local models
            .build()
            .map_err(|e| AIToolsError::Network(
                format!("Failed to initialize Ollama HTTP client: {}. Check system resources and network configuration.", e)
            ))?;

        Ok(Self { config, client })
    }

    /// Convert internal chat request to Ollama format
    fn convert_request(&self, request: &ChatRequest) -> serde_json::Value {
        let model = request.model.as_deref().unwrap_or("llama2");

        let messages: Vec<serde_json::Value> = request
            .messages
            .iter()
            .map(|msg| {
                let role = match msg.role {
                    MessageRole::User => "user",
                    MessageRole::Assistant => "assistant",
                    MessageRole::System => "system",
                    MessageRole::Function => "user",
                    MessageRole::Tool => "user",
                };

                serde_json::json!({
                    "role": role,
                    "content": msg.content.as_ref().unwrap_or(&String::new())
                })
            })
            .collect();

        let mut request_body = serde_json::json!({
            "model": model,
            "messages": messages,
            "stream": false
        });

        // Add parameters if provided
        if let Some(params) = &request.parameters {
            let mut options = serde_json::Map::new();

            if let Some(temp) = params.temperature {
                options.insert("temperature".to_string(), serde_json::Value::from(temp));
            }
            if let Some(top_p) = params.top_p {
                options.insert("top_p".to_string(), serde_json::Value::from(top_p));
            }

            if !options.is_empty() {
                request_body["options"] = serde_json::Value::Object(options);
            }
        }

        request_body
    }

    /// Convert Ollama response to internal format
    fn convert_response(&self, response: serde_json::Value) -> crate::Result<ChatResponse> {
        let content = response["message"]["content"].as_str().ok_or_else(|| {
            AIToolsError::InvalidResponse(
                "Ollama response missing 'message.content' field. Local model may have returned unexpected format.".to_string()
            )
        })?;

        let choice = ChatChoice {
            index: 0,
            role: MessageRole::Assistant,
            content: Some(content.to_string()),
            finish_reason: Some("stop".to_string()),
            tool_calls: None,
        };

        // Ollama doesn't provide detailed usage info, so we estimate
        let usage = Some(UsageInfo {
            prompt_tokens: 0,     // Not available from Ollama
            completion_tokens: 0, // Not available from Ollama
            total_tokens: 0,      // Not available from Ollama
        });

        Ok(ChatResponse {
            choices: vec![choice],
            usage,
            model: response["model"].as_str().unwrap_or("unknown").to_string(),
            id: format!("ollama-{}", chrono::Utc::now().timestamp()),
        })
    }

    /// Get the Ollama endpoint URL
    fn get_endpoint(&self) -> String {
        self.config.base_url.clone().unwrap_or_else(|| {
            std::env::var("OLLAMA_BASE_URL")
                .unwrap_or_else(|_| crate::config::DefaultEndpoints::ollama_endpoint())
        })
    }
}

#[async_trait]
impl AIProvider for OllamaProvider {
    async fn process_chat(&self, request: &ChatRequest) -> crate::Result<ChatResponse> {
        let request_body = self.convert_request(request);
        let endpoint = format!("{}/api/chat", self.get_endpoint());

        debug!("Sending request to Ollama: {:?}", request_body);

        let response = self
            .client
            .post(&endpoint)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| AIToolsError::Local(
                format!("Failed to reach Ollama at {}: {}. Ensure Ollama is running locally and endpoint is correct.", endpoint, e)
            ))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!("Ollama API error (status {}): {}", status, error_text);
            return Err(AIToolsError::Local(format!(
                "Ollama returned error status {}: {}. Check model availability and request format.",
                status, error_text
            ))
            .into());
        }

        let response_json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| AIToolsError::Parse(
                format!("Failed to parse Ollama response as JSON: {}. Local model may have returned unexpected format.", e)
            ))?;

        self.convert_response(response_json)
    }

    fn name(&self) -> &str {
        "ollama"
    }

    async fn health_check(&self) -> bool {
        let endpoint = format!("{}/api/tags", self.get_endpoint());

        self.client
            .get(&endpoint)
            .send()
            .await
            .map(|response| response.status().is_success())
            .unwrap_or(false)
    }

    fn capabilities(&self) -> &[AICapability] {
        &[
            AICapability::Chat,
            AICapability::Completion,
            AICapability::LocalProcessing,
        ]
    }
}

/// Create a provider instance based on the provider name
pub fn create_provider(name: &str, config: ProviderConfig) -> crate::Result<Box<dyn AIProvider>> {
    match name.to_lowercase().as_str() {
        "openai" => Ok(Box::new(OpenAIProvider::new(config)?)),
        "anthropic" => Ok(Box::new(AnthropicProvider::new(config)?)),
        "ollama" => Ok(Box::new(OllamaProvider::new(config)?)),
        _ => Err(AIToolsError::UnsupportedProvider(format!(
            "'{}' is not supported. Available providers: openai, anthropic, ollama",
            name
        ))
        .into()),
    }
}

/// Get all available provider names
pub fn available_providers() -> Vec<&'static str> {
    vec!["openai", "anthropic", "ollama"]
}

/// Check if a provider is supported
pub fn is_provider_supported(name: &str) -> bool {
    available_providers().contains(&name.to_lowercase().as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openai_provider_creation() {
        let config = ProviderConfig {
            provider_type: "openai".to_string(),
            api_key: Some("test-key".to_string()),
            base_url: Some("https://api.openai.com/v1".to_string()),
            default_model: Some("gpt-3.5-turbo".to_string()),
            settings: std::collections::HashMap::new(),
        };

        let provider = OpenAIProvider::new(config);
        assert!(provider.is_ok());
    }

    #[test]
    fn test_provider_capabilities() {
        let config = ProviderConfig {
            provider_type: "openai".to_string(),
            api_key: Some("test-key".to_string()),
            base_url: Some("https://api.openai.com/v1".to_string()),
            default_model: Some("gpt-3.5-turbo".to_string()),
            settings: std::collections::HashMap::new(),
        };

        let openai = OpenAIProvider::new(config.clone()).unwrap();
        let anthropic = AnthropicProvider::new(config.clone()).unwrap();
        let ollama = OllamaProvider::new(config).unwrap();

        assert!(openai.capabilities().contains(&AICapability::Chat));
        assert!(anthropic.capabilities().contains(&AICapability::Chat));
        assert!(ollama
            .capabilities()
            .contains(&AICapability::LocalProcessing));
    }

    #[test]
    fn test_create_provider() {
        let config = ProviderConfig {
            provider_type: "openai".to_string(),
            api_key: Some("test-key".to_string()),
            base_url: Some("https://api.openai.com/v1".to_string()),
            default_model: Some("gpt-3.5-turbo".to_string()),
            settings: std::collections::HashMap::new(),
        };

        let openai = create_provider("openai", config.clone());
        assert!(openai.is_ok());

        let invalid = create_provider("invalid", config);
        assert!(invalid.is_err());
    }

    #[test]
    fn test_available_providers() {
        let providers = available_providers();
        assert!(providers.contains(&"openai"));
        assert!(providers.contains(&"anthropic"));
        assert!(providers.contains(&"ollama"));
    }

    #[test]
    fn test_is_provider_supported() {
        assert!(is_provider_supported("openai"));
        assert!(is_provider_supported("OpenAI"));
        assert!(is_provider_supported("anthropic"));
        assert!(is_provider_supported("ollama"));
        assert!(!is_provider_supported("unsupported"));
    }
}
