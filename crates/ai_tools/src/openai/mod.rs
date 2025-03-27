//! OpenAI API client implementation
//!
//! This module provides an implementation of the AI client interface for OpenAI's API.

use async_trait::async_trait;
use futures::stream::StreamExt;
use reqwest::{Client, StatusCode};
use secrecy::{Secret, SecretString};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use crate::common::{
    AIClient, ChatMessage, ChatRequest, ChatResponse, ChatResponseChunk, ChatResponseStream,
    MessageRole, Tool, ToolCall, UsageInfo,
};
use crate::{Error, Result};

mod models;
mod types;

pub use models::*;
pub use types::*;

/// OpenAI API client
pub struct OpenAIClient {
    /// The HTTP client for making requests
    client: Client,
    /// The API key for authentication
    api_key: SecretString,
    /// The API base URL
    api_base: String,
    /// The API organization ID (optional)
    organization: Option<String>,
    /// The default model to use
    default_model_name: String,
}

impl OpenAIClient {
    /// Create a new OpenAI client with the given API key
    pub fn new(api_key: impl Into<String>) -> Self {
        Self::with_config(api_key, OpenAIConfig::default())
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
            api_base: config.api_base,
            organization: config.organization,
            default_model_name: config.default_model.to_string(),
        }
    }
    
    /// Convert a provider-agnostic chat request to an OpenAI-specific request
    fn prepare_request(&self, request: ChatRequest) -> OpenAIChatRequest {
        let model = request.model.unwrap_or_else(|| self.default_model_name.clone());
        
        let mut openai_request = OpenAIChatRequest {
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
        
        if let Some(org) = &self.organization {
            headers.insert(
                "OpenAI-Organization",
                reqwest::header::HeaderValue::from_str(org).expect("Invalid organization ID"),
            );
        }
        
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
}

#[async_trait]
impl AIClient for OpenAIClient {
    fn provider_name(&self) -> &str {
        "openai"
    }
    
    fn default_model(&self) -> &str {
        &self.default_model_name
    }
    
    async fn list_models(&self) -> Result<Vec<String>> {
        let response = self.client
            .get(format!("{}/models", self.api_base))
            .headers(self.build_headers())
            .send()
            .await
            .map_err(Error::Http)?;
            
        let status = response.status();
        let body = response.text().await.map_err(Error::Http)?;
        
        if !status.is_success() {
            return Err(self.handle_error_response(status, &body));
        }
        
        let models_response: OpenAIModelsResponse = serde_json::from_str(&body)
            .map_err(|e| Error::Serialization(e))?;
            
        Ok(models_response.data.into_iter().map(|m| m.id).collect())
    }
    
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
        let openai_request = self.prepare_request(request);
        
        // Don't allow streaming for non-streaming requests
        let mut request_json = serde_json::to_value(openai_request).map_err(Error::Serialization)?;
        if let Some(stream) = request_json.get_mut("stream") {
            *stream = serde_json::Value::Bool(false);
        }
        
        let response = self.client
            .post(format!("{}/chat/completions", self.api_base))
            .headers(self.build_headers())
            .json(&request_json)
            .send()
            .await
            .map_err(Error::Http)?;
            
        let status = response.status();
        let body = response.text().await.map_err(Error::Http)?;
        
        if !status.is_success() {
            return Err(self.handle_error_response(status, &body));
        }
        
        let openai_response: OpenAIChatResponse = serde_json::from_str(&body)
            .map_err(|e| Error::Serialization(e))?;
            
        if openai_response.choices.is_empty() {
            return Err(Error::InvalidResponse("No choices returned".to_string()));
        }
        
        let message = openai_response.choices[0].message.clone();
        let usage = openai_response.usage.map(|u| UsageInfo {
            prompt_tokens: u.prompt_tokens,
            completion_tokens: u.completion_tokens,
            total_tokens: u.total_tokens,
            estimated_cost_usd: None, // We could calculate this based on the model
        });
        
        let extra = HashMap::new(); // For future OpenAI-specific fields
        
        Ok(ChatResponse {
            message,
            model: openai_response.model,
            usage,
            extra,
        })
    }
    
    async fn chat_stream(&self, request: ChatRequest) -> Result<ChatResponseStream> {
        let mut openai_request = self.prepare_request(request);
        openai_request.stream = true;
        
        let response = self.client
            .post(format!("{}/chat/completions", self.api_base))
            .headers(self.build_headers())
            .json(&openai_request)
            .send()
            .await
            .map_err(Error::Http)?;
            
        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.map_err(Error::Http)?;
            return Err(self.handle_error_response(status, &body));
        }
        
        let stream = response.bytes_stream();
        
        // Parse the streaming response
        let stream = stream.map(|chunk_result| {
            let chunk = chunk_result.map_err(|e| Error::Streaming(e.to_string()))?;
            
            // The stream consists of "data: " prefixed JSON objects separated by double newlines
            let chunk_str = String::from_utf8_lossy(&chunk);
            
            // Parse each data line
            let lines = chunk_str.split("\n\n");
            let mut results = Vec::new();
            
            for line in lines {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                
                if !line.starts_with("data: ") {
                    continue;
                }
                
                let data = &line["data: ".len()..];
                if data == "[DONE]" {
                    // Stream is done
                    continue;
                }
                
                match serde_json::from_str::<OpenAIChatStreamResponse>(data) {
                    Ok(response) => {
                        if let Some(choice) = response.choices.first() {
                            let delta = &choice.delta;
                            
                            let role = delta.role.map(|r| match r {
                                OpenAIMessageRole::Assistant => MessageRole::Assistant,
                                OpenAIMessageRole::User => MessageRole::User,
                                OpenAIMessageRole::System => MessageRole::System,
                                OpenAIMessageRole::Tool => MessageRole::Tool,
                            });
                            
                            let tool_call = if let Some(tool_calls) = &delta.tool_calls {
                                if let Some(tc) = tool_calls.first() {
                                    Some(ToolCall {
                                        id: tc.id.clone(),
                                        tool_type: tc.r#type.clone().into(),
                                        function: tc.function.clone(),
                                    })
                                } else {
                                    None
                                }
                            } else {
                                None
                            };
                            
                            results.push(Ok(ChatResponseChunk {
                                role,
                                content: delta.content.clone(),
                                is_final: choice.finish_reason.is_some(),
                                tool_call,
                            }));
                        }
                    },
                    Err(e) => {
                        results.push(Err(Error::Serialization(e)));
                    }
                }
            }
            
            // If we parsed multiple chunks, return them as a stream of results
            if results.len() > 1 {
                futures::stream::iter(results).left_stream()
            } else if let Some(result) = results.into_iter().next() {
                futures::stream::once(async { result }).right_stream()
            } else {
                // No valid data found
                futures::stream::empty().right_stream()
            }
        })
        .flatten();
        
        Ok(ChatResponseStream {
            inner: Box::new(stream),
        })
    }
} 