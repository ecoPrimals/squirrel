//! AI request router with intelligent provider selection and fallback
//!
//! Routes AI requests to the best available provider with retry logic.

use super::adapters::{AiProviderAdapter, HuggingFaceAdapter, OllamaAdapter, OpenAIAdapter};
use super::constraint_router::select_provider_with_constraints;
use super::selector::{ProviderInfo, ProviderSelector, QualityTier};
use super::types::{
    ActionRequirements, ImageGenerationRequest, ImageGenerationResponse, TextGenerationRequest,
    TextGenerationResponse,
};
use crate::error::PrimalError;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// AI request router
pub struct AiRouter {
    /// Available provider adapters
    providers: Arc<RwLock<Vec<Arc<dyn AiProviderAdapter>>>>,

    /// Provider selector
    selector: Arc<ProviderSelector>,

    /// Enable retry with fallback providers
    enable_retry: bool,

    /// Maximum retry attempts
    max_retries: usize,
}

impl AiRouter {
    /// Create a new AI router
    pub async fn new() -> Result<Self, PrimalError> {
        let mut providers: Vec<Arc<dyn AiProviderAdapter>> = Vec::new();

        // Try to initialize OpenAI if available
        if let Ok(openai) = OpenAIAdapter::new() {
            info!("✅ OpenAI adapter initialized (text + image generation)");
            providers.push(Arc::new(openai));
        } else {
            info!("⚠️  OpenAI adapter not available (OPENAI_API_KEY not set)");
        }

        // Try to initialize Ollama if available (local AI)
        let ollama = OllamaAdapter::new();
        if ollama.is_available().await {
            info!("✅ Ollama adapter initialized (local AI)");
            providers.push(Arc::new(ollama));
        } else {
            info!("⚠️  Ollama not available (install: https://ollama.ai)");
        }

        // Try to initialize HuggingFace if available
        let huggingface = HuggingFaceAdapter::new();
        if huggingface.is_available().await {
            info!("✅ HuggingFace adapter initialized");
            providers.push(Arc::new(huggingface));
        } else {
            info!("⚠️  HuggingFace adapter not available (HUGGINGFACE_API_KEY not set)");
        }

        if providers.is_empty() {
            warn!("⚠️  No AI providers available. Set OPENAI_API_KEY, HUGGINGFACE_API_KEY, or install Ollama");
        } else {
            info!(
                "✅ AI router initialized with {} provider(s)",
                providers.len()
            );
        }

        Ok(Self {
            providers: Arc::new(RwLock::new(providers)),
            selector: Arc::new(ProviderSelector::new()),
            enable_retry: true,
            max_retries: 2,
        })
    }

    /// Get count of available providers
    pub async fn provider_count(&self) -> usize {
        self.providers.read().await.len()
    }

    /// Route image generation request
    pub async fn generate_image(
        &self,
        request: ImageGenerationRequest,
        requirements: Option<ActionRequirements>,
    ) -> Result<ImageGenerationResponse, PrimalError> {
        info!("🎨 Routing image generation request: '{}'", request.prompt);

        let provider_infos = self.get_image_generation_providers().await?;

        if provider_infos.is_empty() {
            return Err(PrimalError::OperationFailed(
                "No providers available for image generation. Set OPENAI_API_KEY or HUGGINGFACE_API_KEY".to_string()
            ));
        }

        // Select best provider
        let selected = self
            .selector
            .select_best(&provider_infos, requirements.as_ref())
            .map_err(|e| PrimalError::OperationFailed(format!("Provider selection failed: {e}")))?;

        info!(
            "🎯 Selected provider: {} ({})",
            selected.provider_name, selected.provider_id
        );

        // Try primary provider
        let providers = self.providers.read().await;
        let provider = providers
            .iter()
            .find(|p| p.provider_id() == selected.provider_id)
            .ok_or_else(|| {
                PrimalError::OperationFailed("Selected provider not found".to_string())
            })?;

        match provider.generate_image(request.clone()).await {
            Ok(response) => {
                info!(
                    "✅ Image generation successful via {}",
                    selected.provider_name
                );
                Ok(response)
            }
            Err(e) => {
                error!(
                    "❌ Image generation failed with {}: {}",
                    selected.provider_name, e
                );

                // Try fallback if enabled
                if self.enable_retry && self.max_retries > 0 {
                    return self
                        .retry_image_generation(
                            &request,
                            &selected.provider_id,
                            &provider_infos,
                            requirements.as_ref(),
                        )
                        .await;
                }

                Err(e)
            }
        }
    }

    /// Retry image generation with fallback providers
    async fn retry_image_generation(
        &self,
        request: &ImageGenerationRequest,
        failed_provider_id: &str,
        available_providers: &[ProviderInfo],
        requirements: Option<&ActionRequirements>,
    ) -> Result<ImageGenerationResponse, PrimalError> {
        warn!("🔄 Attempting fallback providers...");

        // Filter out failed provider
        let fallback_providers: Vec<_> = available_providers
            .iter()
            .filter(|p| p.provider_id != failed_provider_id)
            .cloned()
            .collect();

        if fallback_providers.is_empty() {
            return Err(PrimalError::OperationFailed(
                "No fallback providers available".to_string(),
            ));
        }

        // Try fallback provider
        let fallback = self
            .selector
            .select_best(&fallback_providers, requirements)
            .map_err(|e| {
                PrimalError::OperationFailed(format!("Fallback provider selection failed: {e}"))
            })?;

        info!(
            "🔄 Trying fallback provider: {} ({})",
            fallback.provider_name, fallback.provider_id
        );

        let providers = self.providers.read().await;
        let provider = providers
            .iter()
            .find(|p| p.provider_id() == fallback.provider_id)
            .ok_or_else(|| {
                PrimalError::OperationFailed("Fallback provider not found".to_string())
            })?;

        provider.generate_image(request.clone()).await
    }

    /// Route text generation request
    pub async fn generate_text(
        &self,
        request: TextGenerationRequest,
        requirements: Option<ActionRequirements>,
    ) -> Result<TextGenerationResponse, PrimalError> {
        info!(
            "💬 Routing text generation request ({} tokens max)",
            request.max_tokens
        );

        let providers = self.providers.read().await;

        if providers.is_empty() {
            return Err(PrimalError::OperationFailed(
                "No providers available for text generation. Set OPENAI_API_KEY or install Ollama"
                    .to_string(),
            ));
        }

        // Filter to text-capable providers
        let text_providers: Vec<_> = providers
            .iter()
            .filter(|p| p.supports_text_generation())
            .cloned()
            .collect();

        if text_providers.is_empty() {
            return Err(PrimalError::OperationFailed(
                "No text generation providers available".to_string(),
            ));
        }

        // Use constraint-based selection if constraints provided
        let selected = if request.constraints.is_empty() {
            // Fallback to old selector logic
            let provider_infos = self.get_text_generation_providers().await?;
            let selected_info = self
                .selector
                .select_best(&provider_infos, requirements.as_ref())
                .map_err(|e| {
                    PrimalError::OperationFailed(format!("Provider selection failed: {e}"))
                })?;

            text_providers
                .iter()
                .find(|p| p.provider_id() == selected_info.provider_id)
                .cloned()
        } else {
            info!(
                "🎯 Using constraint-based routing with {} constraint(s)",
                request.constraints.len()
            );
            select_provider_with_constraints(&text_providers, &request.constraints, "text")
        };

        let provider = selected.ok_or_else(|| {
            PrimalError::OperationFailed("No suitable provider found".to_string())
        })?;

        info!(
            "🎯 Selected provider: {} ({})",
            provider.provider_name(),
            provider.provider_id()
        );

        provider.generate_text(request.clone()).await
    }

    /// Get available providers for image generation
    async fn get_image_generation_providers(&self) -> Result<Vec<ProviderInfo>, PrimalError> {
        let providers = self.providers.read().await;
        let mut provider_infos = Vec::new();

        for provider in providers.iter() {
            let is_available = true; // Assume available if registered

            let info = match provider.provider_id() {
                "openai" => ProviderInfo {
                    provider_id: "openai".to_string(),
                    provider_name: "OpenAI DALL-E".to_string(),
                    capabilities: vec!["image.generation".to_string()],
                    quality_tier: QualityTier::High,
                    cost_per_unit: Some(0.02),
                    avg_latency_ms: 12000,
                    reliability: 0.98,
                    is_local: false,
                    is_available,
                },
                "huggingface" => ProviderInfo {
                    provider_id: "huggingface".to_string(),
                    provider_name: "HuggingFace Stable Diffusion".to_string(),
                    capabilities: vec!["image.generation".to_string()],
                    quality_tier: QualityTier::Medium,
                    cost_per_unit: Some(0.0),
                    avg_latency_ms: 20000,
                    reliability: 0.85,
                    is_local: false,
                    is_available,
                },
                _ => continue,
            };

            provider_infos.push(info);
        }

        debug!("Found {} image generation providers", provider_infos.len());
        Ok(provider_infos)
    }

    /// Get available providers for text generation
    async fn get_text_generation_providers(&self) -> Result<Vec<ProviderInfo>, PrimalError> {
        let providers = self.providers.read().await;
        let mut provider_infos = Vec::new();

        for provider in providers.iter() {
            let is_available = true; // Assume available if registered

            let info = match provider.provider_id() {
                "openai" => ProviderInfo {
                    provider_id: "openai".to_string(),
                    provider_name: "OpenAI GPT".to_string(),
                    capabilities: vec!["text.generation".to_string()],
                    quality_tier: QualityTier::High,
                    cost_per_unit: Some(0.002),
                    avg_latency_ms: 2000,
                    reliability: 0.99,
                    is_local: false,
                    is_available,
                },
                _ => continue,
            };

            provider_infos.push(info);
        }

        debug!("Found {} text generation providers", provider_infos.len());
        Ok(provider_infos)
    }

    /// List all available providers and their capabilities
    pub async fn list_providers(&self) -> Vec<ProviderInfo> {
        let mut all_providers = Vec::new();

        if let Ok(image_providers) = self.get_image_generation_providers().await {
            all_providers.extend(image_providers);
        }

        if let Ok(text_providers) = self.get_text_generation_providers().await {
            // Avoid duplicates
            for provider in text_providers {
                if !all_providers
                    .iter()
                    .any(|p: &ProviderInfo| p.provider_id == provider.provider_id)
                {
                    all_providers.push(provider);
                }
            }
        }

        all_providers
    }
}

impl Default for AiRouter {
    fn default() -> Self {
        // For default, create empty router - use new() for initialization
        Self {
            providers: Arc::new(RwLock::new(Vec::new())),
            selector: Arc::new(ProviderSelector::new()),
            enable_retry: true,
            max_retries: 2,
        }
    }
}
