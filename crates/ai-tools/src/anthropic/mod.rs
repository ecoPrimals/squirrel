//! Anthropic API client implementation
//!
//! This module provides an implementation of the AI client interface for Anthropic's API.

use async_trait::async_trait;
use secrecy::{Secret, SecretString};

use crate::common::{
    AIClient, ChatRequest, ChatResponse, ChatResponseStream
};
use crate::{Error, Result};

/// Anthropic API client
pub struct AnthropicClient {
    /// The API key for authentication
    api_key: SecretString,
    /// The default model to use
    default_model_name: String,
}

impl AnthropicClient {
    /// Create a new Anthropic client with the given API key
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: Secret::new(api_key.into()),
            default_model_name: "claude-3-opus-20240229".to_string(),
        }
    }
}

#[async_trait]
impl AIClient for AnthropicClient {
    fn provider_name(&self) -> &str {
        "anthropic"
    }
    
    fn default_model(&self) -> &str {
        &self.default_model_name
    }
    
    async fn list_models(&self) -> Result<Vec<String>> {
        // Anthropic doesn't have a models endpoint, so we return a hard-coded list
        Ok(vec![
            "claude-3-opus-20240229".to_string(), 
            "claude-3-sonnet-20240229".to_string(),
            "claude-3-haiku-20240307".to_string(),
            "claude-2.1".to_string(),
            "claude-2.0".to_string(),
            "claude-instant-1.2".to_string(),
        ])
    }
    
    async fn chat(&self, _request: ChatRequest) -> Result<ChatResponse> {
        // This is a placeholder implementation
        Err(Error::UnsupportedFeature("Anthropic implementation not yet available".to_string()))
    }
    
    async fn chat_stream(&self, _request: ChatRequest) -> Result<ChatResponseStream> {
        // This is a placeholder implementation
        Err(Error::UnsupportedFeature("Anthropic implementation not yet available".to_string()))
    }
} 