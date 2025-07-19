//! Google Gemini AI client
//!
//! This module provides integration with Google's Gemini AI models.

use async_trait::async_trait;
use std::sync::Arc;
use std::any::Any;

use crate::{
    common::{AIClient, ChatMessage, ChatResponse, ChatResponseChunk, ChatRequest, ChatResponseStream},
    error::{Error, Result},
};

/// Configuration for Google Gemini client
#[derive(Debug, Clone)]
pub struct GeminiConfig {
    /// API key for authentication
    pub api_key: String,
    /// Base URL for the API
    pub api_base: String,
    /// Default model to use
    pub default_model: String,
    /// Request timeout in seconds
    pub timeout: u64,
}

impl Default for GeminiConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            api_base: "https://generativelanguage.googleapis.com".to_string(),
            default_model: "gemini-pro".to_string(),
            timeout: 30,
        }
    }
}

/// Google Gemini AI client
#[derive(Debug, Clone)]
pub struct GeminiClient {
    config: GeminiConfig,
    client: reqwest::Client,
}

impl GeminiClient {
    /// Create a new Gemini client
    pub fn new(api_key: impl Into<String>) -> Result<Self> {
        let config = GeminiConfig {
            api_key: api_key.into(),
            ..Default::default()
        };
        
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout))
            .build()
            .map_err(|e| Error::NetworkError(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self { config, client })
    }

    /// Create a new Gemini client with custom configuration
    pub fn with_config(config: GeminiConfig) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout))
            .build()
            .map_err(|e| Error::NetworkError(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self { config, client })
    }
}

#[async_trait]
impl AIClient for GeminiClient {
    fn provider_name(&self) -> &str {
        "google"
    }

    fn default_model(&self) -> &str {
        &self.config.default_model
    }

    async fn list_models(&self) -> Result<Vec<String>> {
        // Attempt to fetch actual model list from Google AI API
        match self.fetch_available_models().await {
            Ok(models) => {
                if models.is_empty() {
                    // Fallback to known models if API call returns empty
                    Ok(self.get_known_models())
                } else {
                    Ok(models)
                }
            }
            Err(_) => {
                // Fallback to known models if API call fails
                Ok(self.get_known_models())
            }
        }
    }

    /// Fetch available models from Google AI API
    async fn fetch_available_models(&self) -> Result<Vec<String>> {
        let url = "https://generativelanguage.googleapis.com/v1/models";
        
        let response = self.client
            .get(url)
            .header("x-goog-api-key", &self.api_key)
            .send()
            .await?;

        if response.status().is_success() {
            let json: serde_json::Value = response.json().await?;
            
            let mut models = Vec::new();
            if let Some(model_array) = json.get("models").and_then(|m| m.as_array()) {
                for model in model_array {
                    if let Some(name) = model.get("name").and_then(|n| n.as_str()) {
                        // Extract model name from full path (e.g., "models/gemini-1.5-pro" -> "gemini-1.5-pro")
                        if let Some(model_name) = name.strip_prefix("models/") {
                            models.push(model_name.to_string());
                        }
                    }
                }
            }
            
            Ok(models)
        } else {
            Err(Error::Http(format!("API request failed with status: {}", response.status())))
        }
    }

    /// Get list of known Google AI models as fallback
    fn get_known_models(&self) -> Vec<String> {
        vec![
            "gemini-1.5-pro".to_string(),
            "gemini-1.5-pro-001".to_string(),
            "gemini-1.5-pro-002".to_string(),
            "gemini-1.5-flash".to_string(),
            "gemini-1.5-flash-001".to_string(),
            "gemini-1.5-flash-002".to_string(),
            "gemini-1.0-pro".to_string(),
            "gemini-pro".to_string(),
            "text-bison-001".to_string(),
            "chat-bison-001".to_string(),
        ]
    }

    async fn chat(&self, _request: ChatRequest) -> Result<ChatResponse> {
        Err(Error::NotImplemented("Gemini chat API not yet implemented".to_string()))
    }

    async fn chat_stream(&self, _request: ChatRequest) -> Result<ChatResponseStream> {
        Err(Error::NotImplemented("Gemini streaming API not yet implemented".to_string()))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
} 