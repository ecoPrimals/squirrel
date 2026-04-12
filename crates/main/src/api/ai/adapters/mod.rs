// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! AI provider adapters
//!
//! Adapters for different AI providers
//!
//! All adapters use capability-based HTTP delegation (TRUE PRIMAL pattern).
//! HTTP requests are delegated to whoever provides "http.request" capability.
//! NO hardcoded primal names (Songbird, etc.) - discovered at runtime!
//!
//! ## Migration Note (v0.3.0)
//!
//! The vendor-specific adapters (Anthropic, OpenAI) are **deprecated** and
//! feature-gated behind `deprecated-adapters`. Use `UniversalAiAdapter` instead.
//!
//! To enable deprecated adapters (temporary):
//! ```toml
//! squirrel = { features = ["deprecated-adapters"] }
//! ```

// Universal adapter (always available - Unix socket providers)
mod universal;

/// Remote inference adapter — forwards to springs registered via `inference.register_provider`.
pub mod remote_inference;

// Re-exports (always available)
pub use remote_inference::{RemoteInferenceAdapter, RemoteProviderConfig};
pub use universal::{ProviderMetadata, UniversalAiAdapter};

// DEPRECATED: HTTP-delegating adapters (v0.3.0 removal planned)
// Enable with `deprecated-adapters` feature
#[cfg(feature = "deprecated-adapters")]
pub mod anthropic;
#[cfg(feature = "deprecated-adapters")]
pub mod openai;

#[cfg(feature = "deprecated-adapters")]
pub use anthropic::AnthropicAdapter;
#[cfg(feature = "deprecated-adapters")]
pub use openai::OpenAiAdapter;

#[cfg(test)]
pub mod test_mocks;

use super::types::{
    ImageGenerationRequest, ImageGenerationResponse, TextGenerationRequest, TextGenerationResponse,
};
use crate::error::PrimalError;

/// Quality tier for AI models
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum QualityTier {
    /// Basic quality models
    Basic,
    /// Fast models (optimized for speed)
    Fast,
    /// Standard quality models
    Standard,
    /// High quality models
    High,
    /// Premium quality models
    Premium,
}

/// Trait for AI provider adapters
pub trait AiProviderAdapter: Send + Sync {
    /// Get provider identifier
    fn provider_id(&self) -> &str;

    /// Get provider display name
    fn provider_name(&self) -> &str;

    /// Check if this is a local provider (privacy)
    fn is_local(&self) -> bool;

    /// Get cost per unit (None if variable)
    fn cost_per_unit(&self) -> Option<f64>;

    /// Get average latency in milliseconds
    fn avg_latency_ms(&self) -> u64;

    /// Get quality tier
    fn quality_tier(&self) -> QualityTier;

    /// Check if supports text generation
    fn supports_text_generation(&self) -> bool;

    /// Check if supports image generation
    fn supports_image_generation(&self) -> bool;

    /// Check if provider is currently available
    async fn is_available(&self) -> bool {
        true // Default: assume available
    }

    /// Generate text
    async fn generate_text(
        &self,
        request: TextGenerationRequest,
    ) -> Result<TextGenerationResponse, PrimalError>;

    /// Generate image
    async fn generate_image(
        &self,
        request: ImageGenerationRequest,
    ) -> Result<ImageGenerationResponse, PrimalError>;
}

/// Runtime-selected AI provider adapter (enum dispatch).
pub enum AiProvider {
    /// Capability-based Unix socket adapter.
    Universal(UniversalAiAdapter),
    /// Remote spring registered via `inference.register_provider`.
    RemoteInference(RemoteInferenceAdapter),
    #[cfg(feature = "deprecated-adapters")]
    OpenAi(OpenAiAdapter),
    #[cfg(feature = "deprecated-adapters")]
    Anthropic(AnthropicAdapter),
    #[cfg(test)]
    MockText(test_mocks::MockTextAdapter),
    #[cfg(test)]
    MockImageOnly(test_mocks::MockImageOnlyAdapter),
    #[cfg(test)]
    MockFailingImage(test_mocks::MockFailingImageAdapter),
    #[cfg(test)]
    MockFallbackImage(test_mocks::MockFallbackImageAdapter),
    #[cfg(test)]
    ConstraintRouter(test_mocks::ConstraintRouterMockAdapter),
    #[cfg(test)]
    JsonRpcMockText(test_mocks::JsonRpcMockTextAdapter),
    #[cfg(test)]
    JsonRpcTestAi(test_mocks::TestAiAdapter),
}

impl AiProviderAdapter for AiProvider {
    fn provider_id(&self) -> &str {
        match self {
            Self::Universal(a) => a.provider_id(),
            Self::RemoteInference(a) => a.provider_id(),
            #[cfg(feature = "deprecated-adapters")]
            Self::OpenAi(a) => a.provider_id(),
            #[cfg(feature = "deprecated-adapters")]
            Self::Anthropic(a) => a.provider_id(),
            #[cfg(test)]
            Self::MockText(a) => a.provider_id(),
            #[cfg(test)]
            Self::MockImageOnly(a) => a.provider_id(),
            #[cfg(test)]
            Self::MockFailingImage(a) => a.provider_id(),
            #[cfg(test)]
            Self::MockFallbackImage(a) => a.provider_id(),
            #[cfg(test)]
            Self::ConstraintRouter(a) => a.provider_id(),
            #[cfg(test)]
            Self::JsonRpcMockText(a) => a.provider_id(),
            #[cfg(test)]
            Self::JsonRpcTestAi(a) => a.provider_id(),
        }
    }

    fn provider_name(&self) -> &str {
        match self {
            Self::Universal(a) => a.provider_name(),
            Self::RemoteInference(a) => a.provider_name(),
            #[cfg(feature = "deprecated-adapters")]
            Self::OpenAi(a) => a.provider_name(),
            #[cfg(feature = "deprecated-adapters")]
            Self::Anthropic(a) => a.provider_name(),
            #[cfg(test)]
            Self::MockText(a) => a.provider_name(),
            #[cfg(test)]
            Self::MockImageOnly(a) => a.provider_name(),
            #[cfg(test)]
            Self::MockFailingImage(a) => a.provider_name(),
            #[cfg(test)]
            Self::MockFallbackImage(a) => a.provider_name(),
            #[cfg(test)]
            Self::ConstraintRouter(a) => a.provider_name(),
            #[cfg(test)]
            Self::JsonRpcMockText(a) => a.provider_name(),
            #[cfg(test)]
            Self::JsonRpcTestAi(a) => a.provider_name(),
        }
    }

    fn is_local(&self) -> bool {
        match self {
            Self::Universal(a) => a.is_local(),
            Self::RemoteInference(a) => a.is_local(),
            #[cfg(feature = "deprecated-adapters")]
            Self::OpenAi(a) => a.is_local(),
            #[cfg(feature = "deprecated-adapters")]
            Self::Anthropic(a) => a.is_local(),
            #[cfg(test)]
            Self::MockText(a) => a.is_local(),
            #[cfg(test)]
            Self::MockImageOnly(a) => a.is_local(),
            #[cfg(test)]
            Self::MockFailingImage(a) => a.is_local(),
            #[cfg(test)]
            Self::MockFallbackImage(a) => a.is_local(),
            #[cfg(test)]
            Self::ConstraintRouter(a) => a.is_local(),
            #[cfg(test)]
            Self::JsonRpcMockText(a) => a.is_local(),
            #[cfg(test)]
            Self::JsonRpcTestAi(a) => a.is_local(),
        }
    }

    fn cost_per_unit(&self) -> Option<f64> {
        match self {
            Self::Universal(a) => a.cost_per_unit(),
            Self::RemoteInference(a) => a.cost_per_unit(),
            #[cfg(feature = "deprecated-adapters")]
            Self::OpenAi(a) => a.cost_per_unit(),
            #[cfg(feature = "deprecated-adapters")]
            Self::Anthropic(a) => a.cost_per_unit(),
            #[cfg(test)]
            Self::MockText(a) => a.cost_per_unit(),
            #[cfg(test)]
            Self::MockImageOnly(a) => a.cost_per_unit(),
            #[cfg(test)]
            Self::MockFailingImage(a) => a.cost_per_unit(),
            #[cfg(test)]
            Self::MockFallbackImage(a) => a.cost_per_unit(),
            #[cfg(test)]
            Self::ConstraintRouter(a) => a.cost_per_unit(),
            #[cfg(test)]
            Self::JsonRpcMockText(a) => a.cost_per_unit(),
            #[cfg(test)]
            Self::JsonRpcTestAi(a) => a.cost_per_unit(),
        }
    }

    fn avg_latency_ms(&self) -> u64 {
        match self {
            Self::Universal(a) => a.avg_latency_ms(),
            Self::RemoteInference(a) => a.avg_latency_ms(),
            #[cfg(feature = "deprecated-adapters")]
            Self::OpenAi(a) => a.avg_latency_ms(),
            #[cfg(feature = "deprecated-adapters")]
            Self::Anthropic(a) => a.avg_latency_ms(),
            #[cfg(test)]
            Self::MockText(a) => a.avg_latency_ms(),
            #[cfg(test)]
            Self::MockImageOnly(a) => a.avg_latency_ms(),
            #[cfg(test)]
            Self::MockFailingImage(a) => a.avg_latency_ms(),
            #[cfg(test)]
            Self::MockFallbackImage(a) => a.avg_latency_ms(),
            #[cfg(test)]
            Self::ConstraintRouter(a) => a.avg_latency_ms(),
            #[cfg(test)]
            Self::JsonRpcMockText(a) => a.avg_latency_ms(),
            #[cfg(test)]
            Self::JsonRpcTestAi(a) => a.avg_latency_ms(),
        }
    }

    fn quality_tier(&self) -> QualityTier {
        match self {
            Self::Universal(a) => a.quality_tier(),
            Self::RemoteInference(a) => a.quality_tier(),
            #[cfg(feature = "deprecated-adapters")]
            Self::OpenAi(a) => a.quality_tier(),
            #[cfg(feature = "deprecated-adapters")]
            Self::Anthropic(a) => a.quality_tier(),
            #[cfg(test)]
            Self::MockText(a) => a.quality_tier(),
            #[cfg(test)]
            Self::MockImageOnly(a) => a.quality_tier(),
            #[cfg(test)]
            Self::MockFailingImage(a) => a.quality_tier(),
            #[cfg(test)]
            Self::MockFallbackImage(a) => a.quality_tier(),
            #[cfg(test)]
            Self::ConstraintRouter(a) => a.quality_tier(),
            #[cfg(test)]
            Self::JsonRpcMockText(a) => a.quality_tier(),
            #[cfg(test)]
            Self::JsonRpcTestAi(a) => a.quality_tier(),
        }
    }

    fn supports_text_generation(&self) -> bool {
        match self {
            Self::Universal(a) => a.supports_text_generation(),
            Self::RemoteInference(a) => a.supports_text_generation(),
            #[cfg(feature = "deprecated-adapters")]
            Self::OpenAi(a) => a.supports_text_generation(),
            #[cfg(feature = "deprecated-adapters")]
            Self::Anthropic(a) => a.supports_text_generation(),
            #[cfg(test)]
            Self::MockText(a) => a.supports_text_generation(),
            #[cfg(test)]
            Self::MockImageOnly(a) => a.supports_text_generation(),
            #[cfg(test)]
            Self::MockFailingImage(a) => a.supports_text_generation(),
            #[cfg(test)]
            Self::MockFallbackImage(a) => a.supports_text_generation(),
            #[cfg(test)]
            Self::ConstraintRouter(a) => a.supports_text_generation(),
            #[cfg(test)]
            Self::JsonRpcMockText(a) => a.supports_text_generation(),
            #[cfg(test)]
            Self::JsonRpcTestAi(a) => a.supports_text_generation(),
        }
    }

    fn supports_image_generation(&self) -> bool {
        match self {
            Self::Universal(a) => a.supports_image_generation(),
            Self::RemoteInference(a) => a.supports_image_generation(),
            #[cfg(feature = "deprecated-adapters")]
            Self::OpenAi(a) => a.supports_image_generation(),
            #[cfg(feature = "deprecated-adapters")]
            Self::Anthropic(a) => a.supports_image_generation(),
            #[cfg(test)]
            Self::MockText(a) => a.supports_image_generation(),
            #[cfg(test)]
            Self::MockImageOnly(a) => a.supports_image_generation(),
            #[cfg(test)]
            Self::MockFailingImage(a) => a.supports_image_generation(),
            #[cfg(test)]
            Self::MockFallbackImage(a) => a.supports_image_generation(),
            #[cfg(test)]
            Self::ConstraintRouter(a) => a.supports_image_generation(),
            #[cfg(test)]
            Self::JsonRpcMockText(a) => a.supports_image_generation(),
            #[cfg(test)]
            Self::JsonRpcTestAi(a) => a.supports_image_generation(),
        }
    }

    async fn is_available(&self) -> bool {
        match self {
            Self::Universal(a) => a.is_available().await,
            Self::RemoteInference(a) => a.is_available().await,
            #[cfg(feature = "deprecated-adapters")]
            Self::OpenAi(a) => a.is_available().await,
            #[cfg(feature = "deprecated-adapters")]
            Self::Anthropic(a) => a.is_available().await,
            #[cfg(test)]
            Self::MockText(a) => a.is_available().await,
            #[cfg(test)]
            Self::MockImageOnly(a) => a.is_available().await,
            #[cfg(test)]
            Self::MockFailingImage(a) => a.is_available().await,
            #[cfg(test)]
            Self::MockFallbackImage(a) => a.is_available().await,
            #[cfg(test)]
            Self::ConstraintRouter(a) => a.is_available().await,
            #[cfg(test)]
            Self::JsonRpcMockText(a) => a.is_available().await,
            #[cfg(test)]
            Self::JsonRpcTestAi(a) => a.is_available().await,
        }
    }

    async fn generate_text(
        &self,
        request: TextGenerationRequest,
    ) -> Result<TextGenerationResponse, PrimalError> {
        match self {
            Self::Universal(a) => a.generate_text(request).await,
            Self::RemoteInference(a) => a.generate_text(request).await,
            #[cfg(feature = "deprecated-adapters")]
            Self::OpenAi(a) => a.generate_text(request).await,
            #[cfg(feature = "deprecated-adapters")]
            Self::Anthropic(a) => a.generate_text(request).await,
            #[cfg(test)]
            Self::MockText(a) => a.generate_text(request).await,
            #[cfg(test)]
            Self::MockImageOnly(a) => a.generate_text(request).await,
            #[cfg(test)]
            Self::MockFailingImage(a) => a.generate_text(request).await,
            #[cfg(test)]
            Self::MockFallbackImage(a) => a.generate_text(request).await,
            #[cfg(test)]
            Self::ConstraintRouter(a) => a.generate_text(request).await,
            #[cfg(test)]
            Self::JsonRpcMockText(a) => a.generate_text(request).await,
            #[cfg(test)]
            Self::JsonRpcTestAi(a) => a.generate_text(request).await,
        }
    }

    async fn generate_image(
        &self,
        request: ImageGenerationRequest,
    ) -> Result<ImageGenerationResponse, PrimalError> {
        match self {
            Self::Universal(a) => a.generate_image(request).await,
            Self::RemoteInference(a) => a.generate_image(request).await,
            #[cfg(feature = "deprecated-adapters")]
            Self::OpenAi(a) => a.generate_image(request).await,
            #[cfg(feature = "deprecated-adapters")]
            Self::Anthropic(a) => a.generate_image(request).await,
            #[cfg(test)]
            Self::MockText(a) => a.generate_image(request).await,
            #[cfg(test)]
            Self::MockImageOnly(a) => a.generate_image(request).await,
            #[cfg(test)]
            Self::MockFailingImage(a) => a.generate_image(request).await,
            #[cfg(test)]
            Self::MockFallbackImage(a) => a.generate_image(request).await,
            #[cfg(test)]
            Self::ConstraintRouter(a) => a.generate_image(request).await,
            #[cfg(test)]
            Self::JsonRpcMockText(a) => a.generate_image(request).await,
            #[cfg(test)]
            Self::JsonRpcTestAi(a) => a.generate_image(request).await,
        }
    }
}
