//! OpenAI API client implementation
//!
//! This module provides an implementation of the AI client interface for OpenAI's API.

use futures::{
    stream::{self},
    StreamExt, TryStreamExt,
};
use reqwest::{Client, Response, StatusCode};

use crate::{
    common::{
        AIClient, ChatRequest, ChatResponse, ChatResponseChunk,
        ChatResponseStream, ToolCall, UsageInfo,
        tool::{FunctionCall, ToolType},
    },
    error::Error,
    Result,
};

use secrecy::{Secret, ExposeSecret};
use std::time::Duration;

use self::{
    models::DEFAULT_MODEL,
    types::{
        OpenAIChatRequest, OpenAIChatResponse, OpenAIChatStreamResponse,
        OpenAIMessageRole, OpenAIToolCall,
    },
};

pub mod models;
pub mod types;

pub use models::*;
pub use types::*;

/// OpenAI API client
#[derive(Debug, Clone)]
pub struct OpenAIClient {
    /// API key
    api_key: Secret<String>,
    /// HTTP client
    client: Client,
    /// API base URL
    base_url: String,
}

impl OpenAIClient {
    /// Create a new OpenAI client
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: Secret::new(api_key.into()),
            client: Client::new(),
            base_url: "https://api.openai.com/v1".to_string(),
        }
    }
    
    /// Create a new OpenAI client with a custom configuration
    pub fn with_config(api_key: impl Into<String>, config: OpenAIConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()
            .expect("Failed to create HTTP client");
            
        Self {
            client,
            api_key: Secret::new(api_key.into()),
            base_url: config.api_base,
        }
    }
    
    /// Convert a provider-agnostic chat request to an OpenAI-specific request
    fn prepare_request(&self, request: ChatRequest) -> OpenAIChatRequest {
        let model = request.model.unwrap_or_else(|| DEFAULT_MODEL.to_string());
        
        let openai_request = OpenAIChatRequest {
            model,
            messages: request.messages,
            temperature: request.parameters.as_ref().and_then(|p| p.temperature),
            top_p: request.parameters.as_ref().and_then(|p| p.top_p),
            max_tokens: request.parameters.as_ref().and_then(|p| p.max_tokens),
            stream: request.parameters.as_ref().and_then(|p| p.stream).unwrap_or(false),
            tools: request.tools,
            tool_choice: None,
            frequency_penalty: request.parameters.as_ref().and_then(|p| p.frequency_penalty),
            presence_penalty: request.parameters.as_ref().and_then(|p| p.presence_penalty),
            response_format: request.parameters.as_ref().and_then(|p| p.response_format.clone().map(|f| match f {
                crate::common::ResponseFormat::Json => OpenAIResponseFormat {
                    type_field: "json_object".to_string(),
                },
                _ => OpenAIResponseFormat {
                    type_field: "text".to_string(),
                },
            })),
            user: None,
        };
        
        openai_request
    }
    
    /// Build request headers
    fn build_headers(&self) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            reqwest::header::HeaderValue::from_static("application/json"),
        );
        headers.insert(
            reqwest::header::AUTHORIZATION,
            reqwest::header::HeaderValue::from_str(&format!("Bearer {}", self.api_key.expose_secret()))
                .expect("Invalid API key"),
        );
        
        headers
    }
    
    /// Handle an error response from the API
    fn handle_error_response(&self, status: StatusCode, body: &str) -> Error {
        // Try to parse the error response
        if let Ok(error_resp) = serde_json::from_str::<OpenAIErrorResponse>(body) {
            Error::ApiError {
                status: status.as_u16(),
                message: error_resp.error.message,
            }
        } else {
            Error::ApiError {
                status: status.as_u16(),
                message: format!("OpenAI API error: {}", body),
            }
        }
    }

    /// Send a request to the OpenAI API
    async fn send_request(&self, request: &OpenAIChatRequest) -> Result<Response> {
        let url = format!("{}/chat/completions", self.base_url);
        let response = self.client
            .post(&url)
            .headers(self.build_headers())
            .json(request)
            .send()
            .await
            .map_err(|e| Error::Streaming(e.to_string()))?;

        if !response.status().is_success() {
            let error = response.json::<serde_json::Value>().await
                .map_err(|e| Error::Streaming(e.to_string()))?;
            return Err(Error::Streaming(error.to_string()));
        }

        Ok(response)
    }

    async fn handle_tool_call(&self, call: OpenAIToolCall) -> Result<ToolCall> {
        let function_name = call.function.name.clone();
        Ok(ToolCall {
            id: call.id,
            tool_type: ToolType::Function,
            function: FunctionCall {
                name: function_name,
                arguments: call.function.arguments,
            },
        })
    }
}

#[async_trait::async_trait]
impl AIClient for OpenAIClient {
    fn provider_name(&self) -> &str {
        "openai"
    }
    
    fn default_model(&self) -> &str {
        DEFAULT_MODEL
    }
    
    async fn list_models(&self) -> Result<Vec<String>> {
        Ok(vec![DEFAULT_MODEL.to_string()])
    }
    
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
        let openai_request = OpenAIChatRequest {
            model: request.model.unwrap_or_else(|| DEFAULT_MODEL.to_string()),
            messages: request.messages,
            temperature: request.parameters.as_ref().and_then(|p| p.temperature),
            top_p: request.parameters.as_ref().and_then(|p| p.top_p),
            max_tokens: request.parameters.as_ref().and_then(|p| p.max_tokens),
            stream: false,
            tools: request.tools,
            tool_choice: None,
            frequency_penalty: request.parameters.as_ref().and_then(|p| p.frequency_penalty),
            presence_penalty: request.parameters.as_ref().and_then(|p| p.presence_penalty),
            response_format: None,
            user: None,
        };

        let response = self.send_request(&openai_request).await?;
        let openai_response: OpenAIChatResponse = response.json().await?;

        let usage = openai_response.usage.map(|u| UsageInfo {
            prompt_tokens: u.prompt_tokens,
            completion_tokens: u.completion_tokens,
            total_tokens: u.total_tokens,
            estimated_cost_usd: None, // TODO: Calculate cost based on model
        });

        Ok(ChatResponse {
            choices: openai_response.choices.into_iter().map(|c| c.message).collect(),
            usage,
        })
    }

    async fn chat_stream(&self, request: ChatRequest) -> Result<ChatResponseStream> {
        let mut openai_request = self.prepare_request(request);
        // Ensure streaming is enabled regardless of what's in the parameters
        openai_request.stream = true;
        
        let response = self.send_request(&openai_request).await?;
        let lines = response.bytes_stream()
            .map_err(|e| Error::Streaming(e.to_string()))
            .try_filter_map(|bytes| async move {
                let chunk = String::from_utf8_lossy(&bytes);
                let mut results = Vec::new();
                for line in chunk.lines() {
                    if line.is_empty() || line.starts_with("data: [DONE]") {
                        continue;
                    }
                    if let Some(data) = line.strip_prefix("data: ") {
                        if let Ok(response) = serde_json::from_str::<OpenAIChatStreamResponse>(data) {
                            let delta = response.choices[0].delta.clone();
                            results.push(Ok(ChatResponseChunk {
                                role: delta.role.map(|r| match r {
                                    OpenAIMessageRole::System => "system".to_string(),
                                    OpenAIMessageRole::User => "user".to_string(),
                                    OpenAIMessageRole::Assistant => "assistant".to_string(),
                                    OpenAIMessageRole::Tool => "tool".to_string(),
                                }),
                                content: delta.content,
                                tool_calls: delta.tool_calls.map(|calls| {
                                    calls.into_iter().map(|call| {
                                        let function_name = call.function.name.clone();
                                        ToolCall {
                                            id: call.id,
                                            tool_type: ToolType::Function,
                                            function: FunctionCall {
                                                name: function_name,
                                                arguments: call.function.arguments,
                                            },
                                        }
                                    }).collect()
                                }),
                            }));
                        }
                    }
                }
                Ok(Some(stream::iter(results)))
            })
            .try_flatten();

        Ok(ChatResponseStream { inner: Box::new(Box::pin(lines)) })
    }
} 