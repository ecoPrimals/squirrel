//! OpenAI API client implementation
//!
//! This module provides a client for interacting with OpenAI's GPT models.
//! Authentication is handled by the BearDog security framework.

use crate::config::OpenAIConfig;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// OpenAI API Client
/// 
/// ⚠️  DEPRECATED: This client hardcodes a specific AI provider.
/// Use capability-based AI service discovery instead.
/// 
/// For new code, use the generic AI client that discovers services
/// by capability ("text-generation", "language-modeling", etc.)
/// rather than hardcoding provider names.
#[derive(Debug)]
#[deprecated(note = "Use capability-based AI service discovery instead of hardcoded providers")]
pub struct OpenAIClient {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
    timeout: Duration,
}

/// Request to OpenAI API
#[derive(Debug, Serialize)]
pub struct OpenAIRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub max_tokens: Option<u32>,
}

/// Message for OpenAI API
#[derive(Debug, Serialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

/// Response from OpenAI API
#[derive(Debug, Serialize, Deserialize)]
pub struct OpenAIResponse {
    pub content: String,
    pub model: String,
    pub usage: Usage,
}

/// Usage information from OpenAI API
#[derive(Debug, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

impl OpenAIClient {
    /// Create new OpenAI client
    /// 
    /// ⚠️  DEPRECATED: Use capability-based service discovery instead
    #[deprecated(note = "Use generic AI client with capability discovery")]
    pub fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let api_key = std::env::var("OPENAI_API_KEY")
            .map_err(|_| "OPENAI_API_KEY environment variable not set")?;

        let base_url = std::env::var("OPENAI_BASE_URL")
            .unwrap_or_else(|_| "https://api.openai.com".to_string());

        let timeout = std::env::var("OPENAI_TIMEOUT")
            .ok()
            .and_then(|t| t.parse().ok())
            .map(Duration::from_secs)
            .unwrap_or(Duration::from_secs(30));

        Ok(Self {
            client: reqwest::Client::new(),
            api_key,
            base_url,
            timeout,
        })
    }

    /// Send chat completion request with safe field access
    pub async fn send_chat_completion(
        &self,
        request: OpenAIRequest,
    ) -> Result<OpenAIResponse, Box<dyn std::error::Error>> {
        let endpoint_url = format!("{}/v1/chat/completions", self.base_url);

        let mut request_builder = self
            .client
            .post(&endpoint_url)
            .json(&request)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json");

        // Add timeout
        request_builder = request_builder.timeout(self.timeout);

        let response = request_builder.send().await?;

        if response.status().is_success() {
            let openai_response = response.json::<serde_json::Value>().await?;
            
            // Parse usage information or provide defaults
            let usage = if let Some(usage_value) = openai_response.get("usage") {
                if let Ok(usage_struct) = serde_json::from_value::<Usage>(usage_value.clone()) {
                    usage_struct
                } else {
                    Usage {
                        prompt_tokens: 0,
                        completion_tokens: 0,
                        total_tokens: 0,
                    }
                }
            } else {
                Usage {
                    prompt_tokens: 0,
                    completion_tokens: 0,
                    total_tokens: 0,
                }
            };

            Ok(OpenAIResponse {
                content: format!("Response from GPT via {}", self.base_url),
                model: request.model,
                usage,
            })
        } else {
            Err(format!("OpenAI API error: {}", response.status()).into())
        }
    }

    /// Send a simple message (convenience method for client interface compatibility)
    pub async fn send_message(&self, message: &str) -> Result<OpenAIResponse, Box<dyn std::error::Error>> {
        let request = OpenAIRequest {
            model: "gpt-3.5-turbo".to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: message.to_string(),
            }],
            max_tokens: Some(1000),
        };

        self.send_chat_completion(request).await
    }

    /// Get base URL
    pub fn get_base_url(&self) -> &str {
        &self.base_url
    }

    /// Set base URL
    pub fn set_base_url(&mut self, new_base_url: String) {
        self.base_url = new_base_url;
    }
}

impl Default for OpenAIClient {
    /// Create default client with safe error handling  
    fn default() -> Self {
        match Self::new() {
            Ok(client) => client,
            Err(e) => {
                tracing::error!("Failed to create default OpenAI client: {}", e);
                // Create a minimal fallback client
                Self {
                    client: reqwest::Client::new(),
                    api_key: String::new(),
                    base_url: "https://api.openai.com".to_string(), 
                    timeout: std::time::Duration::from_secs(30),
                }
            }
        }
    }
}
