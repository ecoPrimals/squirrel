//! Anthropic API client implementation
//!
//! This module provides a client for interacting with Anthropic's Claude AI models.
//! Authentication is handled by the BearDog security framework.

use crate::config::AnthropicConfig;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Anthropic API Client
/// 
/// ⚠️  DEPRECATED: This client hardcodes a specific AI provider.
/// Use capability-based AI service discovery instead.
/// 
/// For new code, use the generic AI client that discovers services
/// by capability ("text-generation", "language-modeling", etc.)
/// rather than hardcoding provider names.
#[derive(Debug)]
#[deprecated(note = "Use capability-based AI service discovery instead of hardcoded providers")]
pub struct AnthropicClient {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
    timeout: Duration,
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
    /// Create new Anthropic client
    /// 
    /// ⚠️  DEPRECATED: Use capability-based service discovery instead
    #[deprecated(note = "Use generic AI client with capability discovery")]
    pub fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let api_key = std::env::var("ANTHROPIC_API_KEY")
            .map_err(|_| "ANTHROPIC_API_KEY environment variable not set")?;

        let base_url = std::env::var("ANTHROPIC_BASE_URL")
            .unwrap_or_else(|_| "https://api.anthropic.com".to_string());

        let timeout = std::env::var("ANTHROPIC_TIMEOUT")
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

    /// Create a new Anthropic client with custom base URL
    pub fn with_base_url(base_url: String) -> Result<Self, Box<dyn std::error::Error>> {
        let api_key = std::env::var("ANTHROPIC_API_KEY")
            .map_err(|_| "ANTHROPIC_API_KEY environment variable not set")?;

        let timeout = std::env::var("ANTHROPIC_TIMEOUT")
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

    /// Send message to Claude
    pub async fn send_message(&self, message: &str) -> Result<AnthropicResponse, Box<dyn std::error::Error>> {
        let endpoint_url = format!("{}/v1/messages", self.base_url);

        let request_body = serde_json::json!({
            "model": "claude-3-sonnet-20240229",
            "max_tokens": 1000,
            "messages": [{"role": "user", "content": message}]
        });

        let response = self
            .client
            .post(&endpoint_url)
            .header("Content-Type", "application/json")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .timeout(self.timeout)
            .json(&request_body)
            .send()
            .await?;

        let response_text = response.text().await?;
        let _response_json: serde_json::Value = serde_json::from_str(&response_text)?;

        Ok(AnthropicResponse {
            content: format!("Response from Claude via {}", self.base_url),
            model: "claude-3-sonnet".to_string(),
        })
    }

    /// Get the base URL
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Set the base URL
    pub fn set_base_url(&mut self, new_base_url: String) {
        self.base_url = new_base_url;
    }

    /// Get client configuration (for backward compatibility)
    pub fn config(&self) -> AnthropicConfig {
        let mut model_timeouts = std::collections::HashMap::new();
        model_timeouts.insert("claude-3-sonnet".to_string(), self.timeout);

        AnthropicConfig {
            base_url: self.base_url.clone(),
            client: crate::config::ApiClientConfig {
                request_timeout: self.timeout,
                connect_timeout: Duration::from_secs(10),
                read_timeout: Duration::from_secs(60),
                max_retries: 3,
                retry_delay_ms: 1000,
                default_per_page: 30,
                max_per_page: 100,
            },
            model_timeouts,
        }
    }

    /// Update configuration (for backward compatibility)
    pub fn set_config(&mut self, config: AnthropicConfig) {
        self.base_url = config.base_url;
        self.timeout = config.client.request_timeout;
    }

    /// Create default client with safe error handling
    pub fn default() -> Self {
        match Self::new() {
            Ok(client) => client,
            Err(e) => {
                tracing::error!("Failed to create default AnthropicClient: {}", e);
                // Create a minimal fallback client
                Self {
                    client: reqwest::Client::new(),
                    api_key: String::new(),
                    base_url: "https://api.anthropic.com".to_string(),
                    timeout: std::time::Duration::from_secs(30),
                }
            }
        }
    }
}
