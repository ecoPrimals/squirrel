// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Test-only mock adapters for [`super::AiProvider`] enum variants.

use super::{AiProviderAdapter, QualityTier};
use crate::api::ai::types::{
    GeneratedImage, ImageGenerationRequest, ImageGenerationResponse, TextGenerationRequest,
    TextGenerationResponse,
};
use crate::error::PrimalError;

/// Minimal adapter: text generation only (for routing tests).
pub struct MockTextAdapter {
    pub id: &'static str,
    pub name: &'static str,
}

impl AiProviderAdapter for MockTextAdapter {
    fn provider_id(&self) -> &str {
        self.id
    }

    fn provider_name(&self) -> &str {
        self.name
    }

    fn is_local(&self) -> bool {
        true
    }

    fn cost_per_unit(&self) -> Option<f64> {
        Some(0.0)
    }

    fn avg_latency_ms(&self) -> u64 {
        10
    }

    fn quality_tier(&self) -> QualityTier {
        QualityTier::Standard
    }

    fn supports_text_generation(&self) -> bool {
        true
    }

    fn supports_image_generation(&self) -> bool {
        false
    }

    async fn generate_text(
        &self,
        request: TextGenerationRequest,
    ) -> Result<TextGenerationResponse, PrimalError> {
        Ok(TextGenerationResponse {
            text: format!("echo:{}", request.prompt),
            provider_id: self.id.to_string(),
            model: request.model.unwrap_or_else(|| "mock".to_string()),
            usage: None,
            cost_usd: None,
            latency_ms: 1,
        })
    }

    async fn generate_image(
        &self,
        _request: ImageGenerationRequest,
    ) -> Result<ImageGenerationResponse, PrimalError> {
        Err(PrimalError::OperationFailed("mock: no image".to_string()))
    }
}

/// Image-only adapter (no text generation).
pub struct MockImageOnlyAdapter {
    pub id: &'static str,
}

impl AiProviderAdapter for MockImageOnlyAdapter {
    fn provider_id(&self) -> &str {
        self.id
    }

    fn provider_name(&self) -> &'static str {
        "ImageOnly"
    }

    fn is_local(&self) -> bool {
        true
    }

    fn cost_per_unit(&self) -> Option<f64> {
        Some(0.01)
    }

    fn avg_latency_ms(&self) -> u64 {
        20
    }

    fn quality_tier(&self) -> QualityTier {
        QualityTier::Standard
    }

    fn supports_text_generation(&self) -> bool {
        false
    }

    fn supports_image_generation(&self) -> bool {
        true
    }

    async fn generate_text(
        &self,
        _request: TextGenerationRequest,
    ) -> Result<TextGenerationResponse, PrimalError> {
        Err(PrimalError::OperationFailed("no text".to_string()))
    }

    async fn generate_image(
        &self,
        _request: ImageGenerationRequest,
    ) -> Result<ImageGenerationResponse, PrimalError> {
        Ok(ImageGenerationResponse {
            images: vec![GeneratedImage {
                url: None,
                data: None,
                mime_type: "image/png".to_string(),
                width: 1,
                height: 1,
            }],
            provider_id: self.id.to_string(),
            cost_usd: None,
            latency_ms: 1,
        })
    }
}

/// Cheaper image provider that always fails generation — triggers fallback path.
pub struct MockFailingImageAdapter {
    pub id: &'static str,
}

impl AiProviderAdapter for MockFailingImageAdapter {
    fn provider_id(&self) -> &str {
        self.id
    }

    fn provider_name(&self) -> &'static str {
        "FailImg"
    }

    fn is_local(&self) -> bool {
        true
    }

    fn cost_per_unit(&self) -> Option<f64> {
        Some(0.0)
    }

    fn avg_latency_ms(&self) -> u64 {
        5
    }

    fn quality_tier(&self) -> QualityTier {
        QualityTier::Standard
    }

    fn supports_text_generation(&self) -> bool {
        false
    }

    fn supports_image_generation(&self) -> bool {
        true
    }

    async fn generate_text(
        &self,
        _request: TextGenerationRequest,
    ) -> Result<TextGenerationResponse, PrimalError> {
        Err(PrimalError::OperationFailed("no text".to_string()))
    }

    async fn generate_image(
        &self,
        _request: ImageGenerationRequest,
    ) -> Result<ImageGenerationResponse, PrimalError> {
        Err(PrimalError::OperationFailed(
            "primary image provider failed".to_string(),
        ))
    }
}

/// Higher-cost fallback image provider that succeeds.
pub struct MockFallbackImageAdapter {
    pub id: &'static str,
}

impl AiProviderAdapter for MockFallbackImageAdapter {
    fn provider_id(&self) -> &str {
        self.id
    }

    fn provider_name(&self) -> &'static str {
        "OkImg"
    }

    fn is_local(&self) -> bool {
        true
    }

    fn cost_per_unit(&self) -> Option<f64> {
        Some(0.05)
    }

    fn avg_latency_ms(&self) -> u64 {
        30
    }

    fn quality_tier(&self) -> QualityTier {
        QualityTier::Standard
    }

    fn supports_text_generation(&self) -> bool {
        false
    }

    fn supports_image_generation(&self) -> bool {
        true
    }

    async fn generate_text(
        &self,
        _request: TextGenerationRequest,
    ) -> Result<TextGenerationResponse, PrimalError> {
        Err(PrimalError::OperationFailed("no text".to_string()))
    }

    async fn generate_image(
        &self,
        _request: ImageGenerationRequest,
    ) -> Result<ImageGenerationResponse, PrimalError> {
        Ok(ImageGenerationResponse {
            images: vec![GeneratedImage {
                url: None,
                data: None,
                mime_type: "image/png".to_string(),
                width: 2,
                height: 2,
            }],
            provider_id: self.id.to_string(),
            cost_usd: None,
            latency_ms: 2,
        })
    }
}

/// Configurable mock for `constraint_router` unit tests.
pub struct ConstraintRouterMockAdapter {
    pub id: &'static str,
    pub is_local: bool,
    pub cost: Option<f64>,
    pub latency: u64,
    pub quality: QualityTier,
    pub text: bool,
    pub image: bool,
}

impl AiProviderAdapter for ConstraintRouterMockAdapter {
    fn provider_id(&self) -> &str {
        self.id
    }

    fn provider_name(&self) -> &str {
        self.id
    }

    fn is_local(&self) -> bool {
        self.is_local
    }

    fn cost_per_unit(&self) -> Option<f64> {
        self.cost
    }

    fn avg_latency_ms(&self) -> u64 {
        self.latency
    }

    fn quality_tier(&self) -> QualityTier {
        self.quality
    }

    fn supports_text_generation(&self) -> bool {
        self.text
    }

    fn supports_image_generation(&self) -> bool {
        self.image
    }

    async fn generate_text(
        &self,
        _request: TextGenerationRequest,
    ) -> Result<TextGenerationResponse, PrimalError> {
        unreachable!("constraint_router tests do not call generate")
    }

    async fn generate_image(
        &self,
        _request: ImageGenerationRequest,
    ) -> Result<ImageGenerationResponse, PrimalError> {
        unreachable!("constraint_router tests do not call generate")
    }
}

/// Minimal text adapter for JSON-RPC + router integration tests (`jsonrpc_server_unit_tests`).
pub struct JsonRpcMockTextAdapter;

impl AiProviderAdapter for JsonRpcMockTextAdapter {
    fn provider_id(&self) -> &'static str {
        "jsonrpc-mock-text"
    }

    fn provider_name(&self) -> &'static str {
        "JsonRpcMock"
    }

    fn is_local(&self) -> bool {
        true
    }

    fn cost_per_unit(&self) -> Option<f64> {
        Some(0.0)
    }

    fn avg_latency_ms(&self) -> u64 {
        5
    }

    fn quality_tier(&self) -> QualityTier {
        QualityTier::Standard
    }

    fn supports_text_generation(&self) -> bool {
        true
    }

    fn supports_image_generation(&self) -> bool {
        false
    }

    async fn generate_text(
        &self,
        request: TextGenerationRequest,
    ) -> Result<TextGenerationResponse, PrimalError> {
        Ok(TextGenerationResponse {
            text: format!("echo:{}", request.prompt),
            provider_id: self.provider_id().to_string(),
            model: request.model.unwrap_or_else(|| "mock".to_string()),
            usage: None,
            cost_usd: None,
            latency_ms: 1,
        })
    }

    async fn generate_image(
        &self,
        _request: ImageGenerationRequest,
    ) -> Result<ImageGenerationResponse, PrimalError> {
        Err(PrimalError::OperationFailed("no image".to_string()))
    }
}

/// Configurable adapter for JSON-RPC AI router tests (`jsonrpc_ai_router_tests`).
pub struct TestAiAdapter {
    pub id: &'static str,
    pub name: &'static str,
    pub supports_text: bool,
    pub supports_image: bool,
    pub cost: Option<f64>,
    pub text_handler: TextBehavior,
}

/// Controls [`TestAiAdapter`] text generation in tests.
pub enum TextBehavior {
    Echo,
    Fail(&'static str),
    Unreachable,
}

impl TestAiAdapter {
    pub fn text_only(id: &'static str, name: &'static str, behavior: TextBehavior) -> Self {
        Self {
            id,
            name,
            supports_text: true,
            supports_image: false,
            cost: None,
            text_handler: behavior,
        }
    }

    pub fn with_cost(mut self, cost: f64) -> Self {
        self.cost = Some(cost);
        self
    }
}

impl AiProviderAdapter for TestAiAdapter {
    fn provider_id(&self) -> &str {
        self.id
    }

    fn provider_name(&self) -> &str {
        self.name
    }

    fn is_local(&self) -> bool {
        true
    }

    fn cost_per_unit(&self) -> Option<f64> {
        self.cost
    }

    fn avg_latency_ms(&self) -> u64 {
        0
    }

    fn quality_tier(&self) -> QualityTier {
        QualityTier::Standard
    }

    fn supports_text_generation(&self) -> bool {
        self.supports_text
    }

    fn supports_image_generation(&self) -> bool {
        self.supports_image
    }

    async fn generate_text(
        &self,
        request: TextGenerationRequest,
    ) -> Result<TextGenerationResponse, PrimalError> {
        match &self.text_handler {
            TextBehavior::Echo => Ok(TextGenerationResponse {
                text: format!("reply:{}", request.prompt),
                provider_id: self.id.to_string(),
                model: "mock-model".to_string(),
                usage: None,
                cost_usd: None,
                latency_ms: 1,
            }),
            TextBehavior::Fail(msg) => Err(PrimalError::OperationFailed((*msg).to_string())),
            TextBehavior::Unreachable => unreachable!("text generation not expected"),
        }
    }

    async fn generate_image(
        &self,
        _request: ImageGenerationRequest,
    ) -> Result<ImageGenerationResponse, PrimalError> {
        Err(PrimalError::OperationFailed("no image".to_string()))
    }
}
