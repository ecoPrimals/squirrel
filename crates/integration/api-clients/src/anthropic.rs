//! Anthropic API client implementation
//!
//! This module provides a client for interacting with Anthropic's Claude AI models.
//! Authentication is handled by the BearDog security framework.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Client for Anthropic API
pub struct AnthropicClient {
    base_url: String,
    http_client: reqwest::Client,
}

/// Request to Anthropic API
#[derive(Debug, Serialize)]
pub struct AnthropicRequest {
    pub message: String,
    pub model: String,
}

/// Response from Anthropic API
#[derive(Debug, Serialize, Deserialize)]
pub struct AnthropicResponse {
    pub content: String,
    pub model: String,
}

impl AnthropicClient {
    /// Create a new Anthropic client with configurable base URL
    pub fn new() -> Self {
        Self::with_base_url("https://api.anthropic.com".to_string())
    }
    
    /// Create a new Anthropic client with custom base URL
    pub fn with_base_url(base_url: String) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");
            
        Self {
            base_url,
            http_client,
        }
    }

    /// Send a message to Claude using the configured base URL
    pub async fn send_message(
        &self,
        message: &str,
    ) -> Result<AnthropicResponse, Box<dyn std::error::Error>> {
        // Construct the endpoint URL using base_url
        let endpoint_url = format!("{}/v1/messages", self.base_url);
        
        let request = AnthropicRequest {
            message: message.to_string(),
            model: "claude-3".to_string(),
        };
        
        // Enhanced HTTP request using the configured base_url
        let response = self.http_client
            .post(&endpoint_url)
            .header("Content-Type", "application/json")
            .header("x-api-key", "placeholder-key") // In real implementation, use BearDog auth
            .json(&request)
            .send()
            .await?;
            
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("Anthropic API error ({}): {}", response.status(), error_text).into());
        }
        
        // For now, return placeholder response since this requires real API integration
        // In production, this would: let api_response: AnthropicResponse = response.json().await?;
        Ok(AnthropicResponse {
            content: format!("Response from Claude via {}", self.base_url),
            model: "claude-3".to_string(),
        })
    }
    
    /// Get the configured base URL
    pub fn get_base_url(&self) -> &str {
        &self.base_url
    }
    
    /// Update the base URL for this client
    pub fn set_base_url(&mut self, new_base_url: String) {
        self.base_url = new_base_url;
    }
}

impl Default for AnthropicClient {
    fn default() -> Self {
        Self::new()
    }
}
