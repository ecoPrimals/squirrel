// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Bridge Adapter - Connect Universal AI to Legacy Router
#![allow(dead_code)] // Migration bridge awaiting activation
//!
//! This module provides a bridge between the new universal AI interface
//! (AiCapability) and the legacy router interface (AiProviderAdapter).
//!
//! This allows gradual migration without breaking existing code.

use async_trait::async_trait;
use std::sync::Arc;

use super::adapters::{AiProviderAdapter, QualityTier};
use super::types::{
    ImageGenerationRequest, ImageGenerationResponse, TextGenerationRequest, TextGenerationResponse,
};
use super::universal::{AiCapability, ChatMessage, MessageRole, UniversalAiRequest};
use crate::error::PrimalError;

/// Bridge adapter that wraps AiCapability to implement AiProviderAdapter
///
/// This allows the new universal adapters to work with the existing router
/// without modifications.
pub struct BridgeAdapter {
    /// The underlying universal AI capability
    capability: Arc<dyn AiCapability>,
}

impl BridgeAdapter {
    /// Create a new bridge adapter
    pub fn new(capability: Arc<dyn AiCapability>) -> Self {
        Self { capability }
    }

    /// Convert legacy request to universal request
    fn convert_text_request(&self, request: &TextGenerationRequest) -> UniversalAiRequest {
        // Check if this is a chat-style request (has system message or multi-turn)
        let has_system = request.system.is_some();

        if has_system {
            // Use messages format
            let mut messages = Vec::new();

            if let Some(system) = &request.system {
                messages.push(ChatMessage {
                    role: MessageRole::System,
                    content: system.clone(),
                    name: None,
                });
            }

            messages.push(ChatMessage {
                role: MessageRole::User,
                content: request.prompt.clone(),
                name: None,
            });

            UniversalAiRequest {
                prompt: None,
                messages: Some(messages),
                max_tokens: Some(request.max_tokens),
                temperature: Some(request.temperature),
                top_p: None,
                model: request.model.clone(),
                stop: None,
                stream: false,
                metadata: std::collections::HashMap::new(),
            }
        } else {
            // Use simple prompt format
            UniversalAiRequest {
                prompt: Some(request.prompt.clone()),
                messages: None,
                max_tokens: Some(request.max_tokens),
                temperature: Some(request.temperature),
                top_p: None,
                model: request.model.clone(),
                stop: None,
                stream: false,
                metadata: std::collections::HashMap::new(),
            }
        }
    }
}

#[async_trait]
impl AiProviderAdapter for BridgeAdapter {
    fn provider_id(&self) -> &str {
        self.capability.provider_id()
    }

    fn provider_name(&self) -> &str {
        self.capability.provider_id()
    }

    fn is_local(&self) -> bool {
        // Check if provider is local based on metadata
        let metadata = self.capability.metadata();
        metadata.provider_type == super::universal::ProviderType::Local
    }

    fn cost_per_unit(&self) -> Option<f64> {
        // Estimate based on cost tier
        let metadata = self.capability.metadata();
        metadata.cost_tier.as_ref().map(|tier| match tier {
            super::universal::CostTier::Free => 0.0,
            super::universal::CostTier::Low => 0.0001,
            super::universal::CostTier::Medium => 0.001,
            super::universal::CostTier::High => 0.01,
        })
    }

    fn avg_latency_ms(&self) -> u64 {
        let metadata = self.capability.metadata();
        metadata.avg_latency_ms.unwrap_or(1000) // Default 1s
    }

    fn quality_tier(&self) -> QualityTier {
        // Map cost tier to quality tier (higher cost usually means higher quality)
        let metadata = self.capability.metadata();
        match metadata.cost_tier {
            Some(super::universal::CostTier::Free) => QualityTier::Basic,
            Some(super::universal::CostTier::Low) => QualityTier::Standard,
            Some(super::universal::CostTier::Medium) => QualityTier::High,
            Some(super::universal::CostTier::High) => QualityTier::Premium,
            None => QualityTier::Standard,
        }
    }

    fn supports_text_generation(&self) -> bool {
        let caps = self.capability.capabilities();
        caps.iter()
            .any(|c| c == "ai.complete" || c == "ai.chat" || c == "ai.inference")
    }

    fn supports_image_generation(&self) -> bool {
        let caps = self.capability.capabilities();
        caps.iter().any(|c| c == "ai.image.generation")
    }

    async fn generate_text(
        &self,
        request: TextGenerationRequest,
    ) -> Result<TextGenerationResponse, PrimalError> {
        let universal_request = self.convert_text_request(&request);
        let response = self.capability.complete(universal_request).await?;

        Ok(TextGenerationResponse {
            text: response.text,
            provider_id: response.provider_id.to_string(),
            model: response.model.to_string(),
            usage: response.usage.map(|u| crate::api::ai::types::TokenUsage {
                prompt_tokens: u.prompt_tokens,
                completion_tokens: u.completion_tokens,
                total_tokens: u.total_tokens,
            }),
            cost_usd: response.cost_usd,
            latency_ms: response.latency_ms.unwrap_or(0),
        })
    }

    async fn generate_image(
        &self,
        _request: ImageGenerationRequest,
    ) -> Result<ImageGenerationResponse, PrimalError> {
        Err(PrimalError::NotSupported(
            "Image generation not yet supported via universal interface".to_string(),
        ))
    }

    async fn is_available(&self) -> bool {
        self.capability.is_available().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::ai::universal::{
        ProviderMetadata, ProviderType, TokenUsage, UniversalAiResponse,
    };
    use std::collections::HashMap;

    // Mock AI capability for testing
    struct MockAiCapability {
        provider_id: String,
        available: bool,
    }

    #[async_trait]
    impl AiCapability for MockAiCapability {
        async fn complete(
            &self,
            request: UniversalAiRequest,
        ) -> Result<UniversalAiResponse, PrimalError> {
            Ok(UniversalAiResponse {
                text: format!("Mock response to: {:?}", request.prompt.unwrap_or_default()),
                provider_id: std::sync::Arc::from(self.provider_id.as_str()),
                model: std::sync::Arc::from("mock-model"),
                usage: Some(TokenUsage {
                    prompt_tokens: 10,
                    completion_tokens: 20,
                    total_tokens: 30,
                }),
                stop_reason: Some("finished".to_string()),
                latency_ms: Some(100),
                cost_usd: Some(0.001),
                metadata: HashMap::new(),
            })
        }

        async fn is_available(&self) -> bool {
            self.available
        }

        fn capabilities(&self) -> Vec<String> {
            vec!["ai.complete".to_string(), "ai.chat".to_string()]
        }

        fn metadata(&self) -> ProviderMetadata {
            ProviderMetadata {
                name: self.provider_id.clone(),
                provider_type: ProviderType::Local,
                models: vec!["mock-model".to_string()],
                capabilities: vec!["ai.complete".to_string()],
                avg_latency_ms: Some(100),
                cost_tier: Some(super::super::universal::CostTier::Free),
                extra: HashMap::new(),
            }
        }

        fn provider_id(&self) -> &str {
            &self.provider_id
        }
    }

    #[tokio::test]
    async fn test_bridge_adapter_text_generation() {
        let mock = Arc::new(MockAiCapability {
            provider_id: "test-provider".to_string(),
            available: true,
        });

        let bridge = BridgeAdapter::new(mock);

        let request = TextGenerationRequest {
            prompt: "Hello, world!".to_string(),
            system: None,
            max_tokens: 100,
            temperature: 0.7,
            model: None,
            constraints: Vec::new(),
            params: HashMap::new(),
        };

        let response = bridge.generate_text(request).await.unwrap();
        assert!(response.text.contains("Mock response"));
        assert_eq!(response.provider_id, "test-provider");
    }

    #[tokio::test]
    async fn test_bridge_adapter_is_available() {
        let mock = Arc::new(MockAiCapability {
            provider_id: "test-provider".to_string(),
            available: true,
        });

        let bridge = BridgeAdapter::new(mock);
        assert!(bridge.is_available().await);
    }

    #[test]
    fn test_bridge_adapter_supports() {
        let mock = Arc::new(MockAiCapability {
            provider_id: "test-provider".to_string(),
            available: true,
        });

        let bridge = BridgeAdapter::new(mock);

        assert_eq!(bridge.provider_id(), "test-provider");
        assert!(bridge.is_local());
        assert!(bridge.supports_text_generation());
    }
}
