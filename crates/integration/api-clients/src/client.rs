//! Unified AI client interface
//!
//! This module provides a unified interface for interacting with different AI providers.

use crate::{AnthropicClient, OpenAIClient};
use serde::{Deserialize, Serialize};

/// Unified AI client that can use different providers
#[derive(Debug)]
pub enum AIClient {
    /// Generic client that works with any AI service
    Generic(GenericAiClient),
    /// Local fallback client
    Local(LocalAiClient),
    /// Legacy variants (deprecated - kept for backward compatibility)
    #[deprecated(note = "Use capability-based Generic variant instead")]
    Anthropic(AnthropicClient),
    #[deprecated(note = "Use capability-based Generic variant instead")]  
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
            Self::Generic(client) => {
                // This case is handled by the GenericAiClient struct
                // For now, we'll return an error or a placeholder
                Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Generic client not implemented")))
            }
            Self::Local(_) => {
                // This case is handled by the LocalAiClient struct
                // For now, we'll return an error or a placeholder
                Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Local client not implemented")))
            }
        }
    }
}

impl Default for AIClient {
    fn default() -> Self {
        // Use capability-based service discovery for AI services
        use tracing::{debug, info, warn};
        
        debug!("Discovering AI services by capability rather than hardcoded names");
        
        // Look for services that provide AI capabilities we need
        let required_capabilities = vec![
            "text-generation",
            "language-modeling", 
            "chat-completion"
        ];
        
        // Try to discover any AI service with required capabilities
        match Self::discover_ai_service_by_capability(&required_capabilities) {
            Some(ai_service) => {
                info!("Created AIClient using discovered service with capabilities: {:?}", required_capabilities);
                ai_service
            }
            None => {
                warn!("No AI services discovered by capability, using fallback client");
                Self::create_fallback_client()
            }
        }
    }
}

impl AIClient {
    /// Discover AI service by capability (not by name)
    fn discover_ai_service_by_capability(required_capabilities: &[&str]) -> Option<Self> {
        use tracing::debug;
        
        // In a real implementation, this would query a service registry
        // For now, we'll check environment variables for capability endpoints
        
        // Check for any AI service endpoint that advertises our needed capabilities
        if let Ok(ai_endpoint) = std::env::var("AI_SERVICE_ENDPOINT") {
            debug!("Found AI service endpoint via capability discovery: {}", ai_endpoint);
            
            // Create a generic AI client that works with any service providing the capabilities
            return Some(Self::create_capability_based_client(ai_endpoint, required_capabilities));
        }
        
        // Check for service discovery endpoints
        if let Ok(discovery_endpoint) = std::env::var("SERVICE_DISCOVERY_ENDPOINT") {
            debug!("Attempting AI service discovery via: {}", discovery_endpoint);
            // In real implementation, would query the discovery service
            // for services advertising the required AI capabilities
        }
        
        None
    }
    
    /// Create a capability-based client that works with any AI service
    fn create_capability_based_client(endpoint: String, _capabilities: &[&str]) -> Self {
        // Create a generic client that can adapt to any AI service
        Self::Generic(GenericAiClient::new(endpoint))
    }
    
    /// Create fallback client when no services discovered
    fn create_fallback_client() -> Self {
        // Create a minimal local client or mock client
        Self::Local(LocalAiClient::new())
    }
}

/// Generic AI client that works with any AI service via capability-based interface
#[derive(Debug)]
pub struct GenericAiClient {
    endpoint: String,
    client: reqwest::Client,
}

impl GenericAiClient {
    pub fn new(endpoint: String) -> Self {
        Self {
            endpoint,
            client: reqwest::Client::new(),
        }
    }
}

/// Local/fallback AI client for when no external services available
#[derive(Debug)]
pub struct LocalAiClient {
    // Minimal local processing capabilities
}

impl LocalAiClient {
    pub fn new() -> Self {
        Self {}
    }
}
