// SPDX-License-Identifier: AGPL-3.0-or-later
// ORC-Notice: AI coordination mechanics licensed under ORC
// Copyright (C) 2026 ecoPrimals Contributors

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

// Deprecated adapters (feature-gated, v0.3.0 removal planned)
#[cfg(feature = "deprecated-adapters")]
use super::adapters::{AnthropicAdapter, OpenAiAdapter};
use super::http_provider_config::{HttpAiProviderConfig, get_enabled_http_providers};

use super::constraint_router::select_provider_with_constraints;
use super::dignity::{DignityCheckRequest, DignityEvaluator};
use super::selector::{ProviderInfo, ProviderSelector};
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

/// Run dignity check (wateringHole/sovereignty_guardian). Non-blocking: logs warning on violation.
fn run_dignity_check(prompt: &str, model: Option<&str>, context: Option<&str>) {
    let evaluator = DignityEvaluator;
    let request = DignityCheckRequest {
        prompt,
        model,
        context,
    };
    let result = evaluator.evaluate_request(&request);
    if !result.passed {
        warn!(
            "Dignity check failed (wateringHole/sovereignty_guardian): {}",
            result.explanation
        );
    }
}

impl AiRouter {
    /// Create a new AI router with capability-based discovery (TRUE PRIMAL!)
    ///
    /// Discovers AI providers via: (1) HTTP providers from `AI_HTTP_PROVIDERS` / API keys,
    /// (2) `AI_PROVIDER_SOCKETS` hint, (3) biomeOS socket scan for local compute primals.
    ///
    /// # Arguments
    ///
    /// * `_service_mesh_client` - Reserved for future capability registry integration.
    ///   Currently unused; discovery uses env/config/sockets. Pass `None`.
    ///
    /// # Returns
    ///
    /// New AiRouter with discovered providers
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let router = AiRouter::new_with_discovery(None).await?;
    /// ```
    #[expect(clippy::too_many_lines, reason = "Router dispatch; refactor planned")]
    pub async fn new_with_discovery(
        _service_mesh_client: Option<Arc<dyn std::any::Any + Send + Sync>>,
    ) -> Result<Self, PrimalError> {
        info!("🔍 Initializing AI router with capability-based discovery...");

        let mut providers: Vec<Arc<dyn AiProviderAdapter>> = Vec::new();

        // Overall timeout to prevent hangs during provider initialization (10s max)
        let initialization_result = tokio::time::timeout(
            std::time::Duration::from_secs(10),
            async {
                let mut local_providers: Vec<Arc<dyn AiProviderAdapter>> = Vec::new();

                // 1. ✅ VENDOR-AGNOSTIC: Discover HTTP providers from configuration
                // TRUE PRIMAL: Zero compile-time coupling to specific vendors
                // Operators control which providers via AI_HTTP_PROVIDERS env var
                info!("🔍 Discovering HTTP-based AI providers from configuration...");

                let enabled_http_providers = get_enabled_http_providers();

                if enabled_http_providers.is_empty() {
                    info!("ℹ️  No HTTP providers enabled. Set AI_HTTP_PROVIDERS or API keys to enable.");
                } else {
                    info!("📋 Enabled HTTP providers: {}",
                        enabled_http_providers.iter()
                            .map(|p| p.provider_id.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    );

                    for provider_config in enabled_http_providers {
                        // Try to initialize adapter for this provider
                        match Self::init_http_provider(&provider_config).await {
                            Ok(Some(adapter)) => {
                                info!("✅ {} adapter available (HTTP via capability discovery)",
                                    provider_config.provider_name);
                                local_providers.push(adapter);
                            }
                            Ok(None) => {
                                debug!("⚠️  {} adapter not available (check {} + HTTP provider)",
                                    provider_config.provider_name,
                                    provider_config.api_key_env);
                            }
                            Err(e) => {
                                warn!("❌ {} adapter initialization failed: {}",
                                    provider_config.provider_name, e);
                            }
                        }
                    }
                }

                // 2. Check for Unix socket providers (other primals)
                // BIOME OS RECOMMENDATION: Use AI_PROVIDER_SOCKETS hint (simple & fast)
                if let Ok(socket_paths) = std::env::var("AI_PROVIDER_SOCKETS") {
                    info!("🎯 Using AI_PROVIDER_SOCKETS hint: {}", socket_paths);
                    for socket_path in socket_paths.split(',') {
                        let socket_path = socket_path.trim();
                        // Per-socket connection timeout (2s max)
                        match tokio::time::timeout(
                            std::time::Duration::from_secs(2),
                            Self::create_universal_adapter_from_path(socket_path)
                        ).await {
                            Ok(Ok(adapter)) => {
                                info!("✅ Connected to provider: {}", socket_path);
                                local_providers.push(Arc::new(adapter));
                            }
                            Ok(Err(e)) => {
                                warn!("⚠️  Failed to connect to {}: {}", socket_path, e);
                            }
                            Err(_) => {
                                warn!("⚠️  Timeout connecting to {} (>2s)", socket_path);
                            }
                        }
                    }
                }

                // 3. biomeOS socket scan: probe for local AI inference providers
                // Any primal exposing compute.ai.* capabilities can serve
                // (capability-based — no primal name hardcoded).
                if local_providers.is_empty() {
                    info!("🔍 Scanning biomeOS sockets for AI compute providers...");
                    let uid = nix::unistd::getuid();
                    let dir = crate::primal_names::BIOMEOS_SOCKET_DIR;
                    let compute_hint = crate::primal_names::TOADSTOOL;
                    let compute_candidates = [
                        format!("/run/user/{uid}/{dir}/{compute_hint}.sock"),
                        format!("/tmp/{compute_hint}.sock"),
                    ];

                    for socket_path in &compute_candidates {
                        let path = PathBuf::from(socket_path);
                        if !path.exists() {
                            continue;
                        }
                        debug!("Probing potential AI provider: {}", socket_path);
                        match tokio::time::timeout(
                            std::time::Duration::from_secs(2),
                            Self::create_universal_adapter_from_path(socket_path),
                        )
                        .await
                        {
                            Ok(Ok(adapter)) => {
                                info!("✅ Discovered AI compute provider at {}", socket_path);
                                local_providers.push(Arc::new(adapter));
                                break;
                            }
                            Ok(Err(e)) => {
                                debug!("Socket {} not an AI provider: {}", socket_path, e);
                            }
                            Err(_) => {
                                debug!("Timeout probing {} (>2s)", socket_path);
                            }
                        }
                    }
                }

                Ok::<Vec<Arc<dyn AiProviderAdapter>>, PrimalError>(local_providers)
            }
        ).await;

        // Handle initialization timeout gracefully
        match initialization_result {
            Ok(Ok(found_providers)) => {
                providers = found_providers;
            }
            Ok(Err(e)) => {
                error!("❌ AI provider initialization failed: {}", e);
            }
            Err(_) => {
                error!("❌ AI provider initialization timed out (>10s)");
            }
        }

        // Summary
        if providers.is_empty() {
            warn!("⚠️  No AI providers available!");
            warn!("⚠️  For external AI APIs:");
            warn!("     - Set ANTHROPIC_API_KEY or OPENAI_API_KEY");
            warn!("     - Ensure HTTP provider available (http.request capability)");
            warn!("⚠️  For local AI primals:");
            warn!("     - Set AI_PROVIDER_SOCKETS=/tmp/provider.sock");
        } else {
            info!(
                "✅ AI router initialized with {} provider(s) via capability discovery",
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

    /// Initialize HTTP provider adapter based on configuration
    ///
    /// Maps provider_id to the appropriate adapter implementation.
    ///
    /// **Note**: Vendor-specific adapters (Anthropic, OpenAI) are deprecated
    /// and gated behind the `deprecated-adapters` feature.
    /// Use capability-based discovery with UniversalAiAdapter instead.
    async fn init_http_provider(
        config: &HttpAiProviderConfig,
    ) -> Result<Option<Arc<dyn AiProviderAdapter>>, PrimalError> {
        #[cfg(not(feature = "deprecated-adapters"))]
        {
            if config.provider_id == "anthropic" || config.provider_id == "openai" {
                warn!(
                    "⚠️  Provider '{}' requires deprecated-adapters feature. \
                     Use capability-based discovery instead.",
                    config.provider_id
                );
                return Ok(None);
            }
            Err(PrimalError::Configuration(format!(
                "Unknown HTTP provider: {}. Use capability-based discovery instead.",
                config.provider_id
            )))
        }

        #[cfg(feature = "deprecated-adapters")]
        {
            let adapter_result: Result<Arc<dyn AiProviderAdapter>, PrimalError> =
                match config.provider_id.as_str() {
                    "anthropic" => {
                        // Backward compatibility: deprecated-adapters feature, v0.3.0 removal planned
                        #[allow(
                            deprecated,
                            reason = "backward compat: AnthropicAdapter until v0.3.0 removal"
                        )]
                        match AnthropicAdapter::new() {
                            Ok(adapter) => Ok(Arc::new(adapter) as Arc<dyn AiProviderAdapter>),
                            Err(e) => Err(e),
                        }
                    }
                    "openai" => {
                        // Backward compatibility: deprecated-adapters feature, v0.3.0 removal planned
                        #[allow(
                            deprecated,
                            reason = "backward compat: OpenAiAdapter until v0.3.0 removal"
                        )]
                        match OpenAiAdapter::new() {
                            Ok(adapter) => Ok(Arc::new(adapter) as Arc<dyn AiProviderAdapter>),
                            Err(e) => Err(e),
                        }
                    }
                    _ => {
                        return Err(PrimalError::Configuration(format!(
                            "Unknown HTTP provider: {}. Use capability-based discovery instead.",
                            config.provider_id
                        )));
                    }
                };

            match adapter_result {
                Ok(adapter) => {
                    // Availability check with 5s timeout to prevent hangs
                    match tokio::time::timeout(
                        std::time::Duration::from_secs(5),
                        adapter.is_available(),
                    )
                    .await
                    {
                        Ok(true) => Ok(Some(adapter)),
                        Ok(false) => Ok(None), // Not available (missing API key or HTTP provider)
                        Err(_) => {
                            warn!("⏱️  {} availability check timed out", config.provider_name);
                            Ok(None)
                        }
                    }
                }
                Err(e) => Err(e),
            }
        }
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
            name: format!("Universal AI ({file_name})"),
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
                "Provider at {socket_path} is not available"
            )));
        }

        Ok(adapter)
    }

    // Legacy initialization removed - use new_with_discovery() for all builds
    // All providers now use capability discovery (TRUE PRIMAL pattern)

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

        run_dignity_check(&request.prompt, None, None);

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

        run_dignity_check(&request.prompt, request.model.as_deref(), None);

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
    pub(crate) async fn get_image_generation_providers(
        &self,
    ) -> Result<Vec<ProviderInfo>, PrimalError> {
        let providers = self.providers.read().await;
        let mut provider_infos = Vec::new();

        for provider in providers.iter() {
            // Only include providers that support image generation
            if !provider.supports_image_generation() {
                continue;
            }

            let is_available = provider.is_available().await;

            // Map adapter QualityTier to selector QualityTier
            use super::adapters::QualityTier as AdapterQT;
            use super::selector::QualityTier as SelectorQT;
            let quality_tier = match provider.quality_tier() {
                AdapterQT::Basic | AdapterQT::Fast => SelectorQT::Low,
                AdapterQT::Standard => SelectorQT::Medium,
                AdapterQT::High => SelectorQT::High,
                AdapterQT::Premium => SelectorQT::Premium,
            };

            let info = ProviderInfo {
                provider_id: provider.provider_id().to_string(),
                provider_name: provider.provider_name().to_string(),
                capabilities: vec!["image.generation".to_string()],
                quality_tier,
                cost_per_unit: provider.cost_per_unit(),
                avg_latency_ms: provider.avg_latency_ms(),
                reliability: 0.95, // Default reliability
                is_local: provider.is_local(),
                is_available,
            };

            provider_infos.push(info);
        }

        debug!("Found {} image generation providers", provider_infos.len());
        Ok(provider_infos)
    }

    /// Get available providers for text generation
    pub(crate) async fn get_text_generation_providers(
        &self,
    ) -> Result<Vec<ProviderInfo>, PrimalError> {
        let providers = self.providers.read().await;
        let mut provider_infos = Vec::new();

        for provider in providers.iter() {
            // Only include providers that support text generation
            if !provider.supports_text_generation() {
                continue;
            }

            let is_available = provider.is_available().await;

            // Map adapter QualityTier to selector QualityTier
            use super::adapters::QualityTier as AdapterQT;
            use super::selector::QualityTier as SelectorQT;
            let quality_tier = match provider.quality_tier() {
                AdapterQT::Basic | AdapterQT::Fast => SelectorQT::Low, // Fast models sacrifice quality for speed
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

#[cfg(test)]
impl AiRouter {
    /// Construct a router with explicit adapters (unit tests only).
    pub(crate) fn from_adapters_for_test(providers: Vec<Arc<dyn AiProviderAdapter>>) -> Self {
        Self {
            providers: Arc::new(RwLock::new(providers)),
            selector: Arc::new(ProviderSelector::new()),
            enable_retry: true,
            max_retries: 2,
        }
    }
}

#[cfg(test)]
#[path = "router_tests.rs"]
mod tests;
