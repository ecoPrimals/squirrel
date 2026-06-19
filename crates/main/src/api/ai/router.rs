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

use super::adapters::{AiProvider, AiProviderAdapter};
use super::constraint_router::select_provider_with_constraints;
use super::dignity::{DignityCheckRequest, DignityEnforcementLevel, DignityGuard};
use super::selector::{ProviderInfo, ProviderSelector};
use super::types::{
    ActionRequirements, ImageGenerationRequest, ImageGenerationResponse, TextGenerationRequest,
    TextGenerationResponse,
};
use crate::error::PrimalError;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use universal_constants::network::resolve_capability_unix_socket;

/// AI request router
pub struct AiRouter {
    /// Available provider adapters
    providers: Arc<RwLock<Vec<Arc<AiProvider>>>>,

    /// Provider selector
    selector: Arc<ProviderSelector>,

    /// Enable retry with fallback providers
    enable_retry: bool,

    /// Maximum retry attempts
    max_retries: usize,
}

/// Run dignity check (wateringHole/sovereignty guard). Behavior depends on
/// [`super::dignity::DIGNITY_ENFORCEMENT_ENV`] (`warn` | `enforce` | `audit`).
fn run_dignity_check(
    prompt: &str,
    model: Option<&str>,
    context: Option<&str>,
) -> Result<(), PrimalError> {
    let guard = DignityGuard::with_enforcement(DignityEnforcementLevel::from_env());
    let request = DignityCheckRequest {
        prompt,
        model,
        context,
    };
    guard.guard(&request).map_err(|violation| {
        PrimalError::SecurityError(format!(
            "Dignity check failed (wateringHole/sovereignty guard): {}",
            violation.result.explanation
        ))
    })
}

/// Unix socket path for the local compute capability (`COMPUTE_SOCKET` → tiered resolution).
#[inline]
pub fn compute_capability_unix_socket() -> PathBuf {
    resolve_capability_unix_socket("COMPUTE_SOCKET", "compute")
}

impl AiRouter {
    /// Create a new AI router with capability-based discovery (TRUE PRIMAL!)
    ///
    /// Discovers AI providers via:
    /// 1. HTTP providers from `AI_HTTP_PROVIDERS` / API keys
    /// 2. `LOCAL_AI_ENDPOINT` / `OLLAMA_ENDPOINT` / `OLLAMA_URL` (local inference)
    /// 3. Default Ollama probe at `localhost:11434` (implicit discovery)
    /// 4. `AI_PROVIDER_SOCKETS` hint (Unix socket providers)
    /// 5. `COMPUTE_SOCKET` / tiered Unix resolution for local compute capability sockets
    ///
    /// # Arguments
    ///
    /// * `_service_mesh_client` - Reserved for future capability registry integration.
    ///   Currently unused; discovery uses env/config/sockets. Pass `None`.
    pub async fn new_with_discovery(
        _service_mesh_client: Option<Arc<dyn std::any::Any + Send + Sync>>,
    ) -> Result<Self, PrimalError> {
        info!("Initializing AI router with capability-based discovery...");

        let providers = super::router_discovery::discover_providers().await;
        super::router_discovery::log_discovery_summary(providers.len());

        Ok(Self {
            providers: Arc::new(RwLock::new(providers)),
            selector: Arc::new(ProviderSelector::new()),
            enable_retry: true,
            max_retries: 2,
        })
    }

    /// Resolve the local AI inference endpoint from environment variables.
    ///
    /// Uses the same multi-tier resolution as `AIProviderConfig::from_env()`:
    /// `LOCAL_AI_ENDPOINT` → `OLLAMA_ENDPOINT` → `OLLAMA_URL`.
    /// Returns `None` if no env var is explicitly set (does NOT fall back to
    /// a default — the socket scan in step 3 handles implicit discovery).
    pub(crate) fn resolve_local_ai_endpoint() -> Option<String> {
        std::env::var(universal_constants::env_vars::ai::local::ENDPOINT)
            .or_else(|_| std::env::var(universal_constants::env_vars::ai::ollama::ENDPOINT))
            .or_else(|_| std::env::var(universal_constants::env_vars::ai::ollama::URL))
            .ok()
    }

    /// Probe a local AI endpoint and wrap it as a provider if reachable.
    ///
    /// If the endpoint is a Unix socket path (starts with `/` and ends with
    /// `.sock`), delegates to `create_universal_adapter_from_path`.
    /// If it is an HTTP URL, performs a TCP probe to verify the server is
    /// listening and prefers a compute-primal Unix socket (`COMPUTE_SOCKET` /
    /// tiered resolution) when present for local inference.
    pub(crate) async fn probe_local_ai_endpoint(
        endpoint: &str,
    ) -> Result<Arc<AiProvider>, PrimalError> {
        // Socket path — delegate directly
        if endpoint.starts_with('/') {
            let adapter = Self::create_universal_adapter_from_path(endpoint).await?;
            return Ok(Arc::new(AiProvider::Universal(adapter)));
        }

        // HTTP URL — extract host:port and probe
        let url = url::Url::parse(endpoint).map_err(|e| {
            PrimalError::Configuration(format!("Invalid LOCAL_AI_ENDPOINT URL: {e}"))
        })?;
        let host = url
            .host_str()
            .unwrap_or(universal_constants::network::DEFAULT_LOCALHOST);
        let port = url
            .port()
            .unwrap_or_else(universal_constants::deployment::ports::ollama);
        let endpoint = crate::transport::TransportEndpoint::tcp(host, port);

        crate::transport::connect_transport(&endpoint)
            .await
            .map_err(|e| PrimalError::OperationFailed(format!("Cannot reach {endpoint}: {e}")))?;

        // Server is reachable. Prefer a compute-primal Unix socket if present.
        let socket_candidates = [compute_capability_unix_socket()];

        for socket_path in &socket_candidates {
            if !socket_path.exists() {
                continue;
            }
            let socket_path_str = socket_path.to_string_lossy();
            if let Ok(adapter) =
                Self::create_universal_adapter_from_path(socket_path_str.as_ref()).await
            {
                info!(
                    "✅ Wired LOCAL_AI_ENDPOINT {} via compute primal Unix socket {}",
                    endpoint,
                    socket_path_str.as_ref()
                );
                return Ok(Arc::new(AiProvider::Universal(adapter)));
            }
        }

        // No compute primal socket wrapping the HTTP endpoint.
        // UniversalAiAdapter only works with Unix sockets; registering an
        // HTTP URL as a socket_path would cause every request to fail with
        // "No such file or directory".  Skip and let the HTTP-based adapters
        // handle the endpoint via the service mesh instead.
        Err(PrimalError::OperationFailed(format!(
            "Local AI at {endpoint} is reachable but no compute primal Unix socket wraps it. \
             Use the OpenAI adapter (OPENAI_API_KEY + OPENAI_BASE_URL) to \
             route through the service mesh http.request capability instead."
        )))
    }

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

        run_dignity_check(&request.prompt, None, None)?;

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

        run_dignity_check(&request.prompt, request.model.as_deref(), None)?;

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

    /// Map adapter quality tier to selector quality tier.
    const fn map_quality_tier(tier: super::adapters::QualityTier) -> super::selector::QualityTier {
        use super::adapters::QualityTier as AdapterQT;
        use super::selector::QualityTier as SelectorQT;
        match tier {
            AdapterQT::Basic | AdapterQT::Fast => SelectorQT::Low,
            AdapterQT::Standard => SelectorQT::Medium,
            AdapterQT::High => SelectorQT::High,
            AdapterQT::Premium => SelectorQT::Premium,
        }
    }

    /// Build a `ProviderInfo` from a provider and a capability label.
    async fn provider_to_info(
        provider: &AiProvider,
        capability: &str,
        default_reliability: f64,
    ) -> ProviderInfo {
        ProviderInfo {
            provider_id: provider.provider_id().to_string(),
            provider_name: provider.provider_name().to_string(),
            capabilities: vec![capability.to_string()],
            quality_tier: Self::map_quality_tier(provider.quality_tier()),
            cost_per_unit: provider.cost_per_unit(),
            avg_latency_ms: provider.avg_latency_ms(),
            reliability: default_reliability,
            is_local: provider.is_local(),
            is_available: provider.is_available().await,
        }
    }

    /// Get providers matching a capability predicate.
    async fn providers_for_capability(
        &self,
        predicate: impl Fn(&AiProvider) -> bool + Send + Sync,
        capability: &str,
        default_reliability: f64,
    ) -> Result<Vec<ProviderInfo>, PrimalError> {
        let matching: Vec<_> = {
            let providers = self.providers.read().await;
            providers.iter().filter(|p| predicate(p)).cloned().collect()
        };
        let mut infos = Vec::with_capacity(matching.len());
        for provider in &matching {
            infos.push(Self::provider_to_info(provider, capability, default_reliability).await);
        }
        debug!("Found {} {capability} providers", infos.len());
        Ok(infos)
    }

    /// Get available providers for image generation
    pub(crate) async fn get_image_generation_providers(
        &self,
    ) -> Result<Vec<ProviderInfo>, PrimalError> {
        self.providers_for_capability(
            AiProvider::supports_image_generation,
            "image.generation",
            0.95,
        )
        .await
    }

    /// Get available providers for text generation
    pub(crate) async fn get_text_generation_providers(
        &self,
    ) -> Result<Vec<ProviderInfo>, PrimalError> {
        self.providers_for_capability(
            AiProvider::supports_text_generation,
            "text.generation",
            0.99,
        )
        .await
    }

    /// List all available providers and their capabilities
    pub async fn list_providers(&self) -> Vec<ProviderInfo> {
        let mut all = Vec::new();
        if let Ok(img) = self.get_image_generation_providers().await {
            all.extend(img);
        }
        if let Ok(txt) = self.get_text_generation_providers().await {
            for p in txt {
                if !all
                    .iter()
                    .any(|existing: &ProviderInfo| existing.provider_id == p.provider_id)
                {
                    all.push(p);
                }
            }
        }
        all
    }

    /// List providers with enriched model-level detail (model names, embedding support).
    pub async fn list_providers_detailed(&self) -> Vec<(ProviderInfo, Vec<String>, bool)> {
        let infos = self.list_providers().await;
        let providers = self.providers.read().await;
        infos
            .into_iter()
            .map(|info| {
                let (models, embeds) = providers
                    .iter()
                    .find(|p| p.provider_id() == info.provider_id)
                    .map(|p| (p.available_model_names(), p.supports_embedding()))
                    .unwrap_or_default();
                (info, models, embeds)
            })
            .collect()
    }

    /// Find the first available provider that supports embedding.
    pub async fn find_embedding_provider(&self) -> Option<Arc<AiProvider>> {
        let providers = self.providers.read().await;
        providers.iter().find(|p| p.supports_embedding()).cloned()
    }

    /// Register a remote spring as an inference provider.
    ///
    /// Called by `inference.register_provider` JSON-RPC handler. Creates a
    /// `RemoteInferenceAdapter` and adds it to the live provider list.
    pub async fn register_remote_provider(&self, config: super::adapters::RemoteProviderConfig) {
        let id = config.provider_id.clone();
        let adapter = super::adapters::RemoteInferenceAdapter::new(config);
        let new_provider = Arc::new(AiProvider::RemoteInference(adapter));

        let mut providers = self.providers.write().await;
        if let Some(pos) = providers.iter().position(|p| p.provider_id() == id) {
            providers[pos] = new_provider;
            info!(provider = %id, total = providers.len(), "Re-registered inference provider (upsert)");
        } else {
            providers.push(new_provider);
            info!(provider = %id, total = providers.len(), "Registered new inference provider");
        }
    }

    /// Remove a remote provider by ID. Returns `true` if it was found and removed.
    pub async fn unregister_remote_provider(&self, provider_id: &str) -> bool {
        let mut providers = self.providers.write().await;
        let before = providers.len();
        providers.retain(|p| p.provider_id() != provider_id);
        let removed = providers.len() < before;
        if removed {
            info!(provider = %provider_id, total = providers.len(), "Unregistered inference provider");
        }
        removed
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
    pub(crate) fn from_adapters_for_test(providers: Vec<Arc<AiProvider>>) -> Self {
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
