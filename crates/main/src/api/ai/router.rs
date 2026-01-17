//! AI request router with intelligent provider selection and fallback
//!
//! Routes AI requests to the best available provider with retry logic.
//!
//! ## Capability-Based Discovery (TRUE PRIMAL)
//!
//! The router supports capability-based discovery via service mesh:
//!
//! ```rust,ignore
//! // With service mesh (capability-based - TRUE PRIMAL)
//! let router = AiRouter::new_with_discovery(service_mesh_client).await?;
//!
//! // Without service mesh (fallback to dev adapters or env config)
//! let router = AiRouter::new().await?;
//! ```

// Always available (production + dev)
use super::adapters::{AiProviderAdapter, ProviderMetadata, UniversalAiAdapter};

// HTTP adapters (dev mode only)
#[cfg(feature = "dev-direct-http")]
use super::adapters::{HuggingFaceAdapter, OllamaAdapter, OpenAIAdapter};
use super::constraint_router::select_provider_with_constraints;
use super::selector::{ProviderInfo, ProviderSelector, QualityTier};
use super::types::{
    ActionRequirements, ImageGenerationRequest, ImageGenerationResponse, TextGenerationRequest,
    TextGenerationResponse,
};
use crate::error::PrimalError;
use std::collections::HashMap;
use std::path::PathBuf;
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
    /// Create a new AI router with capability-based discovery (TRUE PRIMAL!)
    ///
    /// This method uses service mesh to discover AI providers via capability-based discovery.
    /// It will discover ANY primal offering AI capabilities and also load external
    /// vendors from configuration.
    ///
    /// # Arguments
    ///
    /// * `_service_mesh_client` - Service mesh client for capability discovery (placeholder)
    ///
    /// # Returns
    ///
    /// New AiRouter with discovered providers
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let router = AiRouter::new_with_discovery(service_mesh).await?;
    /// ```
    pub async fn new_with_discovery(
        _service_mesh_client: Option<Arc<dyn std::any::Any + Send + Sync>>,
    ) -> Result<Self, PrimalError> {
        info!("🔍 Initializing AI router with capability-based discovery...");

        let mut providers: Vec<Arc<dyn AiProviderAdapter>> = Vec::new();

        // TODO: Implement actual service mesh capability discovery
        // For now, we'll use a placeholder that loads from environment
        // This will be replaced with actual capability registry integration:
        //
        // let text_gen_providers = songbird.discover_by_capability("ai:text-generation").await?;
        // let image_gen_providers = songbird.discover_by_capability("ai:image-generation").await?;
        //
        // for discovery in text_gen_providers {
        //     let adapter = UniversalAiAdapter::from_discovery(...);
        //     providers.push(Arc::new(adapter));
        // }

        // For now, check environment for discovered provider sockets
        if let Ok(socket_paths) = std::env::var("AI_PROVIDER_SOCKETS") {
            info!("📡 Discovering AI providers from environment...");
            for socket_path in socket_paths.split(',') {
                match Self::create_universal_adapter_from_path(socket_path.trim()).await {
                    Ok(adapter) => {
                        info!("✅ Discovered provider: {}", adapter.provider_name());
                        providers.push(Arc::new(adapter));
                    }
                    Err(e) => {
                        warn!("⚠️ Failed to connect to provider at {}: {}", socket_path, e);
                    }
                }
            }
        }

        // Fallback: Load legacy adapters in parallel (dev mode only!)
        #[cfg(feature = "dev-direct-http")]
        {
            info!("🔄 Loading legacy AI adapters (DEV MODE - HTTP enabled)...");
            let legacy_providers = Self::load_legacy_adapters_parallel().await;
            providers.extend(legacy_providers);
        }

        #[cfg(not(feature = "dev-direct-http"))]
        {
            info!("✅ Production mode: Using UniversalAiAdapter ONLY (Unix sockets)");
        }

        if providers.is_empty() {
            warn!("⚠️  No AI providers available. Configure AI_PROVIDER_SOCKETS for capability discovery");
            warn!("⚠️  Or enable dev-direct-http feature and set API keys for development");
        } else {
            info!(
                "✅ AI router initialized with {} provider(s) (capability-based + legacy)",
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

    /// Create UniversalAiAdapter from socket path (helper for discovery)
    async fn create_universal_adapter_from_path(
        socket_path: &str,
    ) -> Result<UniversalAiAdapter, PrimalError> {
        // Parse socket path to extract metadata
        // Format: /path/to/socket.sock or /path/to/primal-capability.sock
        let path = PathBuf::from(socket_path);
        let file_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");

        // Create basic metadata from socket path
        let metadata = ProviderMetadata {
            primal_id: file_name.to_string(),
            name: format!("Universal AI ({})", file_name),
            is_local: Some(true), // Unix socket implies local
            quality: Some("standard".to_string()),
            cost: Some(0.0), // Local providers are free
            max_tokens: Some(4096),
            additional: HashMap::new(),
        };

        let adapter = UniversalAiAdapter::from_discovery(
            "ai:text-generation", // Default capability
            path,
            metadata,
        );

        // Verify adapter is available
        if !adapter.is_available().await {
            return Err(PrimalError::OperationFailed(format!(
                "Provider at {} is not available",
                socket_path
            )));
        }

        Ok(adapter)
    }

    /// Load legacy adapters in parallel (concurrent initialization!)
    /// **v1.1.0**: Only available with `dev-direct-http` feature
    #[cfg(feature = "dev-direct-http")]
    async fn load_legacy_adapters_parallel() -> Vec<Arc<dyn AiProviderAdapter>> {
        // Execute all initializations in parallel using tokio::join!
        let (openai_result, ollama_result, huggingface_result) = tokio::join!(
            async {
                match OpenAIAdapter::new() {
                    Ok(adapter) => {
                        info!("✅ OpenAI adapter initialized (text + image generation)");
                        Some(Arc::new(adapter) as Arc<dyn AiProviderAdapter>)
                    }
                    Err(_) => {
                        info!("⚠️  OpenAI adapter not available (OPENAI_API_KEY not set)");
                        None
                    }
                }
            },
            async {
                let adapter = OllamaAdapter::new();
                if adapter.is_available().await {
                    info!("✅ Ollama adapter initialized (local AI)");
                    Some(Arc::new(adapter) as Arc<dyn AiProviderAdapter>)
                } else {
                    info!("⚠️  Ollama not available (install: https://ollama.ai)");
                    None
                }
            },
            async {
                let adapter = HuggingFaceAdapter::new();
                if adapter.is_available().await {
                    info!("✅ HuggingFace adapter initialized");
                    Some(Arc::new(adapter) as Arc<dyn AiProviderAdapter>)
                } else {
                    info!("⚠️  HuggingFace adapter not available (HUGGINGFACE_API_KEY not set)");
                    None
                }
            }
        );

        // Collect successful initializations
        vec![openai_result, ollama_result, huggingface_result]
            .into_iter()
            .flatten()
            .collect()
    }

    /// Create a new AI router (legacy - uses hardcoded adapters)
    ///
    /// **Note**: This is the legacy initialization method. For TRUE PRIMAL compliance,
    /// use `new_with_discovery()` instead for capability-based provider discovery.
    ///
    /// **v1.1.0**: This method requires the `dev-direct-http` feature.
    /// Production builds should use `new_with_discovery()` which uses UniversalAiAdapter only.
    #[cfg(feature = "dev-direct-http")]
    pub async fn new() -> Result<Self, PrimalError> {
        info!("⚠️  Using legacy AI router initialization (hardcoded adapters)");
        info!("⚠️  Consider using new_with_discovery() for capability-based discovery");

        // Use parallel loading for legacy adapters
        let providers = Self::load_legacy_adapters_parallel().await;

        if providers.is_empty() {
            warn!("⚠️  No AI providers available. Enable dev-direct-http feature and configure API keys for development");
        } else {
            info!(
                "✅ AI router initialized with {} provider(s) (legacy mode)",
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
                "No providers available for text generation. Configure AI_PROVIDER_SOCKETS or enable dev-direct-http feature"
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
            // Only include providers that support text generation
            if !provider.supports_text_generation() {
                continue;
            }

            let is_available = true; // Assume available if registered

            // Map adapter QualityTier to selector QualityTier
            use super::adapters::QualityTier as AdapterQT;
            use super::selector::QualityTier as SelectorQT;
            let quality_tier = match provider.quality_tier() {
                AdapterQT::Basic => SelectorQT::Low,
                AdapterQT::Fast => SelectorQT::Low, // Fast models sacrifice quality for speed
                AdapterQT::Standard => SelectorQT::Medium,
                AdapterQT::High => SelectorQT::High,
                AdapterQT::Premium => SelectorQT::Premium,
            };

            let info = ProviderInfo {
                provider_id: provider.provider_id().to_string(),
                provider_name: provider.provider_name().to_string(),
                capabilities: vec!["text.generation".to_string()],
                quality_tier,
                cost_per_unit: provider.cost_per_unit(),
                avg_latency_ms: provider.avg_latency_ms(),
                reliability: 0.99, // Default for now
                is_local: provider.is_local(),
                is_available,
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
