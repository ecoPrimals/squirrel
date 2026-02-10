// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

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

// Re-exports (always available)
pub use universal::{ProviderMetadata, UniversalAiAdapter};

// DEPRECATED: HTTP-delegating adapters (v0.3.0 removal planned)
// Enable with `deprecated-adapters` feature
#[cfg(feature = "deprecated-adapters")]
#[allow(unexpected_cfgs)] // Feature defined in Cargo.toml
pub mod anthropic;
#[cfg(feature = "deprecated-adapters")]
#[allow(unexpected_cfgs)] // Feature defined in Cargo.toml
pub mod openai;

#[cfg(feature = "deprecated-adapters")]
#[allow(unexpected_cfgs)] // Feature defined in Cargo.toml
pub use anthropic::AnthropicAdapter;
#[cfg(feature = "deprecated-adapters")]
#[allow(unexpected_cfgs)] // Feature defined in Cargo.toml
pub use openai::OpenAiAdapter;

use super::types::{
    ImageGenerationRequest, ImageGenerationResponse, TextGenerationRequest, TextGenerationResponse,
};
use crate::error::PrimalError;
use async_trait::async_trait;

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
#[async_trait]
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
