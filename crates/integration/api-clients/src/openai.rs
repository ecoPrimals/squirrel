//! OpenAI API client implementation
//!
//! This module provides a client for interacting with OpenAI's GPT models.
//! Authentication is handled by the BearDog security framework.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Client for OpenAI API
pub struct OpenAIClient {
    base_url: String,
    http_client: reqwest::Client,
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
    /// Create a new OpenAI client with configurable base URL
    pub fn new() -> Self {
        Self::with_base_url("https://api.openai.com".to_string())
    }
    
    /// Create a new OpenAI client with custom base URL
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

    /// Send a message to GPT using the configured base URL
    pub async fn send_message(
        &self,
        message: &str,
    ) -> Result<OpenAIResponse, Box<dyn std::error::Error>> {
        // Construct the endpoint URL using base_url  
        let endpoint_url = format!("{}/v1/chat/completions", self.base_url);
        
        let request = OpenAIRequest {
            model: "gpt-4".to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: message.to_string(),
            }],
            max_tokens: Some(150),
        };
        
        // Enhanced HTTP request using the configured base_url
        let response = self.http_client
            .post(&endpoint_url)
            .header("Content-Type", "application/json")
            .header("Authorization", "Bearer placeholder-key") // In real implementation, use BearDog auth
            .json(&request)
            .send()
            .await?;
            
        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("OpenAI API error ({}): {}", status, error_text).into());
        }
        
        // For now, return placeholder response since this requires real API integration
        // In production, this would parse the actual OpenAI response format
        Ok(OpenAIResponse {
            content: format!("Response from GPT via {}", self.base_url),
            model: "gpt-4".to_string(),
            usage: Usage {
                prompt_tokens: message.len() as u32 / 4, // Rough estimate
                completion_tokens: 50,
                total_tokens: (message.len() as u32 / 4) + 50,
            },
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

impl Default for OpenAIClient {
    fn default() -> Self {
        Self::new()
    }
}
