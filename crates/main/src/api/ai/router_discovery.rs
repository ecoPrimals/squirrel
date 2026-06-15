// SPDX-License-Identifier: AGPL-3.0-or-later
// ORC-Notice: AI coordination mechanics licensed under ORC
// Copyright (C) 2026 ecoPrimals Contributors

//! Provider discovery pipeline for [`super::router::AiRouter`].
//!
//! Extracted from `router.rs` for module size management. This module
//! implements the capability-based provider discovery that runs during
//! router initialization.

use std::path::Path;
use std::sync::Arc;
use tracing::{debug, info, warn};

use super::adapters::{AiProvider, AiProviderAdapter};
use super::http_provider_config::get_enabled_http_providers;
use super::router::{AiRouter, compute_capability_unix_socket};
use crate::error::PrimalError;

/// Discover all available AI providers using capability-based resolution.
///
/// Steps:
/// 1. HTTP providers from `AI_HTTP_PROVIDERS` / API keys
/// 2. Local AI inference (`LOCAL_AI_ENDPOINT` / Ollama chain)
/// 3. Inference endpoints from `INFERENCE_ENDPOINT` / `AI_INFERENCE_ENDPOINT`
/// 4. Unix socket providers from `AI_PROVIDER_SOCKETS`
/// 5. Compute capability socket scan (tiered resolution)
pub async fn discover_providers() -> Vec<Arc<AiProvider>> {
    let initialization_result =
        tokio::time::timeout(std::time::Duration::from_secs(10), discover_all_providers()).await;

    match initialization_result {
        Ok(Ok(providers)) => providers,
        Ok(Err(e)) => {
            tracing::error!("AI provider initialization failed: {}", e);
            Vec::new()
        }
        Err(_) => {
            tracing::error!("AI provider initialization timed out (>10s)");
            Vec::new()
        }
    }
}

async fn discover_all_providers() -> Result<Vec<Arc<AiProvider>>, PrimalError> {
    let mut providers: Vec<Arc<AiProvider>> = Vec::new();

    discover_http_providers(&mut providers).await;
    discover_local_ai(&mut providers).await;
    discover_inference_endpoints(&mut providers).await;
    discover_socket_providers(&mut providers).await;
    discover_compute_sockets(&mut providers).await;

    Ok(providers)
}

async fn discover_http_providers(providers: &mut Vec<Arc<AiProvider>>) {
    info!("Discovering HTTP-based AI providers from configuration...");
    let enabled = get_enabled_http_providers();

    if enabled.is_empty() {
        info!("No HTTP providers enabled. Set AI_HTTP_PROVIDERS or API keys to enable.");
        return;
    }

    info!(
        "Enabled HTTP providers: {}",
        enabled
            .iter()
            .map(|p| p.provider_id.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    );

    for provider_config in enabled {
        match AiRouter::init_http_provider(&provider_config).await {
            Ok(Some(adapter)) => {
                info!(
                    "{} adapter available (HTTP via capability discovery)",
                    provider_config.provider_name
                );
                providers.push(adapter);
            }
            Ok(None) => {
                debug!(
                    "{} adapter not available (check {} + HTTP provider)",
                    provider_config.provider_name, provider_config.api_key_env
                );
            }
            Err(e) => {
                warn!(
                    "{} adapter initialization failed: {}",
                    provider_config.provider_name, e
                );
            }
        }
    }
}

async fn discover_local_ai(providers: &mut Vec<Arc<AiProvider>>) {
    if let Some(endpoint) = AiRouter::resolve_local_ai_endpoint() {
        info!("LOCAL_AI_ENDPOINT resolved: {}", endpoint);
        match tokio::time::timeout(
            std::time::Duration::from_secs(3),
            AiRouter::probe_local_ai_endpoint(&endpoint),
        )
        .await
        {
            Ok(Ok(adapter)) => {
                info!("Local AI inference server discovered at {}", endpoint);
                providers.push(adapter);
            }
            Ok(Err(e)) => {
                debug!("Local AI endpoint {} not reachable: {}", endpoint, e);
            }
            Err(_) => {
                debug!("Local AI endpoint {} probe timed out (>3s)", endpoint);
            }
        }
    } else if providers.is_empty() {
        let default_endpoint = universal_constants::deployment::endpoints::ollama();
        debug!("Probing default Ollama endpoint: {}", default_endpoint);
        match tokio::time::timeout(
            std::time::Duration::from_secs(2),
            AiRouter::probe_local_ai_endpoint(&default_endpoint),
        )
        .await
        {
            Ok(Ok(adapter)) => {
                info!("Default Ollama instance discovered at {}", default_endpoint);
                providers.push(adapter);
            }
            Ok(Err(_)) => {
                debug!("No Ollama at default endpoint {}", default_endpoint);
            }
            Err(_) => {
                debug!("Default Ollama probe timed out (>2s)");
            }
        }
    }
}

async fn discover_inference_endpoints(providers: &mut Vec<Arc<AiProvider>>) {
    for env_key in [
        universal_constants::env_vars::ai::INFERENCE_ENDPOINT,
        universal_constants::env_vars::ai::AI_INFERENCE_ENDPOINT,
    ] {
        if let Ok(endpoint) = std::env::var(env_key) {
            info!("Inference endpoint discovered via {env_key}: {endpoint}");
            let config = super::adapters::RemoteProviderConfig {
                provider_id: format!("env-{}", env_key.to_lowercase().replace('_', "-")),
                socket_path: if endpoint.starts_with("unix://") || endpoint.starts_with('/') {
                    Some(
                        endpoint
                            .strip_prefix("unix://")
                            .unwrap_or(&endpoint)
                            .to_string(),
                    )
                } else {
                    None
                },
                endpoint: if endpoint.starts_with("http://") || endpoint.starts_with("https://") {
                    Some(endpoint.clone())
                } else {
                    None
                },
                models: Vec::new(),
                supported_tasks: vec![
                    "text_generation".to_string(),
                    "inference.complete".to_string(),
                ],
                supports_streaming: false,
                max_context_size: 4096,
                quality_tier: Some("standard".to_string()),
                cost_per_unit: None,
            };
            let adapter = super::adapters::RemoteInferenceAdapter::new(config);
            if adapter.is_available().await {
                info!("Inference provider registered from {env_key}: {endpoint}");
                providers.push(Arc::new(AiProvider::RemoteInference(adapter)));
            } else {
                debug!("Inference endpoint from {env_key} not reachable: {endpoint}");
            }
            break;
        }
    }
}

async fn discover_socket_providers(providers: &mut Vec<Arc<AiProvider>>) {
    if let Ok(socket_paths) = std::env::var(universal_constants::env_vars::ai::PROVIDER_SOCKETS) {
        info!("Using AI_PROVIDER_SOCKETS hint: {}", socket_paths);
        for socket_path in socket_paths.split(',') {
            let socket_path = socket_path.trim();
            match tokio::time::timeout(
                std::time::Duration::from_secs(2),
                AiRouter::create_universal_adapter_from_path(socket_path),
            )
            .await
            {
                Ok(Ok(adapter)) => {
                    info!("Connected to provider: {}", socket_path);
                    providers.push(Arc::new(AiProvider::Universal(adapter)));
                }
                Ok(Err(e)) => {
                    warn!("Failed to connect to {}: {}", socket_path, e);
                }
                Err(_) => {
                    warn!("Timeout connecting to {} (>2s)", socket_path);
                }
            }
        }
    }
}

async fn discover_compute_sockets(providers: &mut Vec<Arc<AiProvider>>) {
    if !providers.is_empty() {
        return;
    }

    info!("Scanning for AI compute provider Unix sockets...");
    let compute_socket = compute_capability_unix_socket();
    let compute_candidates = [compute_socket];

    for socket_path in &compute_candidates {
        let path: &Path = socket_path.as_path();
        if !path.exists() {
            continue;
        }
        let socket_path = path.to_string_lossy();
        debug!("Probing potential AI provider: {}", socket_path);
        match tokio::time::timeout(
            std::time::Duration::from_secs(2),
            AiRouter::create_universal_adapter_from_path(socket_path.as_ref()),
        )
        .await
        {
            Ok(Ok(adapter)) => {
                info!("Discovered AI compute provider at {}", socket_path.as_ref());
                providers.push(Arc::new(AiProvider::Universal(adapter)));
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

/// Log a summary of discovered providers or print help instructions.
pub fn log_discovery_summary(count: usize) {
    if count == 0 {
        warn!("No AI providers available!");
        warn!("For local AI (Ollama/llama.cpp/vLLM):");
        warn!(
            "  - Set LOCAL_AI_ENDPOINT={} (or your configured Ollama endpoint)",
            universal_constants::deployment::endpoints::ollama()
        );
        warn!("  - Or start Ollama (auto-discovered at default port)");
        warn!("For external AI APIs:");
        warn!("  - Set ANTHROPIC_API_KEY or OPENAI_API_KEY");
        warn!("For Unix socket providers:");
        warn!("  - Set AI_PROVIDER_SOCKETS=/tmp/provider.sock");
    } else {
        info!(
            "AI router initialized with {} provider(s) via capability discovery",
            count
        );
    }
}
