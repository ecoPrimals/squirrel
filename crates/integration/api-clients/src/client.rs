//! Unified AI client interface
//!
//! This module provides a unified interface for interacting with different AI providers.

use crate::{AnthropicClient, OpenAIClient};
use serde::{Deserialize, Serialize};

/// Unified AI client that can use different providers
pub enum AIClient {
    Anthropic(AnthropicClient),
    OpenAI(OpenAIClient),
}

/// Unified response from AI providers
#[derive(Debug, Serialize, Deserialize)]
pub struct AIResponse {
    pub content: String,
    pub model: String,
    pub provider: String,
}

impl AIClient {
    /// Create a new Anthropic client
    pub fn anthropic() -> Self {
        Self::Anthropic(AnthropicClient::new().expect("Failed to create AnthropicClient"))
    }

    /// Create a new OpenAI client
    pub fn openai() -> Self {
        Self::OpenAI(OpenAIClient::new().expect("Failed to create OpenAIClient"))
    }

    /// Send a message using the configured provider
    pub async fn send_message(
        &self,
        message: &str,
    ) -> Result<AIResponse, Box<dyn std::error::Error>> {
        match self {
            Self::Anthropic(client) => {
                let response = client.send_message(message).await?;
                Ok(AIResponse {
                    content: response.content,
                    model: response.model,
                    provider: "anthropic".to_string(),
                })
            }
            Self::OpenAI(client) => {
                let response = client.send_message(message).await?;
                Ok(AIResponse {
                    content: response.content,
                    model: response.model,
                    provider: "openai".to_string(),
                })
            }
        }
    }
}

impl Default for AIClient {
    fn default() -> Self {
        Self::anthropic()
    }
}
