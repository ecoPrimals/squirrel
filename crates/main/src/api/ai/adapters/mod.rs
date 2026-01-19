//! AI provider adapters
//!
//! Adapters for different AI providers (`OpenAI`, `HuggingFace`, Ollama, etc.)
//!
//! **v1.1.0**: HTTP-based adapters (OpenAI, HuggingFace, Ollama) are only available
//! with the `dev-direct-http` feature. Production builds use `UniversalAiAdapter` only.

// Test harness adapters DELETED! (OpenAI, Ollama, HuggingFace)
// Production uses capability_ai exclusively (Pure Rust Unix sockets)

// Universal adapter (always available - production mode)
mod universal;

// Always available
pub use universal::{ProviderMetadata, UniversalAiAdapter};

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
