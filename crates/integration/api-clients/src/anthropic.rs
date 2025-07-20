//! Anthropic API client implementation
//!
//! This module provides a client for interacting with Anthropic's Claude AI models.
//! Authentication is handled by the BearDog security framework.

use crate::config::{AnthropicConfig, ANTHROPIC_BASE_URL};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Client for Anthropic API
pub struct AnthropicClient {
    config: AnthropicConfig,
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
    /// Create a new Anthropic client with default configuration
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Self::with_config(AnthropicConfig::from_env())
    }

    /// Create a new Anthropic client with custom base URL
    pub fn with_base_url(base_url: String) -> Result<Self, Box<dyn std::error::Error>> {
        let mut config = AnthropicConfig::from_env();
        config.base_url = base_url;
        Self::with_config(config)
    }

    /// Create a new Anthropic client with custom configuration
    pub fn with_config(config: AnthropicConfig) -> Result<Self, Box<dyn std::error::Error>> {
        config
            .client
            .validate()
            .map_err(|e| format!("Invalid configuration: {}", e))?;

        let http_client = reqwest::Client::builder()
            .timeout(config.client.request_timeout)
            .connect_timeout(config.client.connect_timeout)
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        Ok(Self {
            config,
            http_client,
        })
    }

    /// Send a message to Claude using the configured base URL
    pub async fn send_message(
        &self,
        message: &str,
    ) -> Result<AnthropicResponse, Box<dyn std::error::Error>> {
        // Construct the endpoint URL using base_url
        let endpoint_url = format!("{}/v1/messages", self.config.base_url);

        let request = AnthropicRequest {
            message: message.to_string(),
            model: "claude-3".to_string(),
        };

        // Enhanced HTTP request using the configured base_url
        let response = self
            .http_client
            .post(&endpoint_url)
            .header("Content-Type", "application/json")
            .header("x-api-key", "placeholder-key") // In real implementation, use BearDog auth
            .json(&request)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("Anthropic API error ({}): {}", status, error_text).into());
        }

        // For now, return placeholder response since this requires real API integration
        // In production, this would: let api_response: AnthropicResponse = response.json().await?;
        Ok(AnthropicResponse {
            content: format!("Response from Claude via {}", self.config.base_url),
            model: "claude-3".to_string(),
        })
    }

    /// Get the configured base URL
    pub fn get_base_url(&self) -> &str {
        &self.config.base_url
    }

    /// Update the base URL for this client
    pub fn set_base_url(&mut self, new_base_url: String) {
        self.config.base_url = new_base_url;
    }

    /// Get the current configuration
    pub fn get_config(&self) -> &AnthropicConfig {
        &self.config
    }

    /// Update the client configuration
    pub fn update_config(&mut self, config: AnthropicConfig) -> Result<(), String> {
        config.client.validate()?;
        self.config = config;
        Ok(())
    }
}

impl Default for AnthropicClient {
    fn default() -> Self {
        Self::new().expect("Failed to create default AnthropicClient")
    }
}
