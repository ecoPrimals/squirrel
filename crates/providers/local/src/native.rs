//! Native AI Provider Implementation
//!
//! This module provides a simplified interface to the native AI provider
//! implementation, maintaining backward compatibility while delegating
//! to the reorganized module structure.

use async_trait::async_trait;

use crate::types::{
    UniversalAIRequest, UniversalAIResponse, UniversalAIStream,
    ModelInfo, ProviderHealth, CostEstimate,
};
use crate::error::ProviderResult;

// Re-export everything from the native module
pub use crate::native::*;

/// Implementation of the UniversalAIProvider trait for NativeAIProvider
#[async_trait]
impl crate::UniversalAIProvider for NativeAIProvider {
    /// Process a single AI request
    async fn process_request(&self, request: UniversalAIRequest) -> ProviderResult<UniversalAIResponse> {
        self.queue_request(request).await
    }

    /// Stream a request (not yet implemented for native provider)
    async fn stream_request(&self, _request: UniversalAIRequest) -> ProviderResult<UniversalAIStream> {
        Err(crate::error::ProviderError::UnsupportedOperation(
            "Streaming not yet implemented for native provider".to_string()
        ))
    }

    /// Perform a health check
    async fn health_check(&self) -> ProviderResult<ProviderHealth> {
        Ok(self.get_health().await)
    }

    /// Get available models
    async fn get_models(&self) -> ProviderResult<Vec<ModelInfo>> {
        self.get_available_models().await
    }

    /// Estimate cost for a request
    async fn estimate_cost(&self, request: &UniversalAIRequest) -> ProviderResult<CostEstimate> {
        self.estimate_cost(request).await
    }

    /// Get provider name
    fn name(&self) -> &str {
        "native"
    }

    /// Get provider type
    fn provider_type(&self) -> crate::ProviderType {
        crate::ProviderType::Local
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::RequestMetadata;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_native_ai_provider_creation() {
        let config = NativeAIConfig::default();
        let provider = NativeAIProvider::new(config);
        
        assert_eq!(provider.name(), "native");
        assert_eq!(provider.provider_type(), crate::ProviderType::Local);
    }

    #[tokio::test]
    async fn test_native_ai_provider_initialization() {
        let mut config = NativeAIConfig::default();
        // Use a temporary file path for testing
        config.model_config.model_path = "/tmp/test_model".to_string();
        
        let provider = NativeAIProvider::new(config);
        
        // Create the test model file
        let _ = std::fs::write("/tmp/test_model", "test model content");
        
        let result = provider.initialize().await;
        
        // Clean up
        let _ = std::fs::remove_file("/tmp/test_model");
        
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_text_generation_request() {
        let mut config = NativeAIConfig::default();
        config.model_config.model_path = "/tmp/test_model_gen".to_string();
        
        let provider = NativeAIProvider::new(config);
        
        // Create the test model file
        let _ = std::fs::write("/tmp/test_model_gen", "test model content");
        
        let _ = provider.initialize().await;
        
        let mut request_content = HashMap::new();
        request_content.insert("prompt".to_string(), 
            serde_json::Value::String("Generate a hello world program".to_string()));
        
        let request = UniversalAIRequest {
            id: "test-123".to_string(),
            request_type: crate::types::AIRequestType::TextGeneration,
            content: request_content,
            metadata: RequestMetadata::default(),
        };

        let response = provider.process_request(request).await;
        
        // Clean up
        let _ = std::fs::remove_file("/tmp/test_model_gen");
        
        assert!(response.is_ok());
        let resp = response.unwrap();
        assert!(resp.content.contains_key("text"));
    }

    #[tokio::test]
    async fn test_model_listing() {
        let config = NativeAIConfig::default();
        let provider = NativeAIProvider::new(config);
        
        let models = provider.get_models().await.unwrap();
        assert!(!models.is_empty());
        assert_eq!(models[0].provider, "native");
    }

    #[tokio::test]
    async fn test_cost_estimation() {
        let config = NativeAIConfig::default();
        let provider = NativeAIProvider::new(config);
        
        let request = UniversalAIRequest {
            id: "test".to_string(),
            request_type: crate::types::AIRequestType::TextGeneration,
            content: HashMap::new(),
            metadata: RequestMetadata::default(),
        };

        let cost = provider.estimate_cost(&request).await.unwrap();
        assert_eq!(cost.estimated_cost, 0.0); // Native models are free
    }

    #[tokio::test]
    async fn test_health_check() {
        let config = NativeAIConfig::default();
        let provider = NativeAIProvider::new(config);
        
        let health = provider.health_check().await.unwrap();
        // Should start as Unknown since not initialized
        assert_eq!(health, ProviderHealth::Unknown);
    }
} 