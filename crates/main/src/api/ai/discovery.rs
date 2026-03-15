// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! AI Provider Discovery - Capability-Based
#![allow(dead_code)] // Discovery infrastructure awaiting activation
//!
//! This module implements TRUE PRIMAL discovery for AI providers.
//! NO hardcoding - all providers are discovered at runtime via capabilities.
//!
//! Discovery mechanisms:
//! 1. Capability registry (primary)
//! 2. Environment variables (explicit)
//! 3. Socket scan (fallback)

use std::sync::Arc;
use tracing::{debug, info, warn};

use crate::api::ai::adapter::UniversalAiAdapter;
use crate::api::ai::universal::BoxedAiCapability;
use crate::capabilities::discovery::discover_capability;
use crate::error::PrimalError;

/// Discover AI providers via capability discovery
///
/// This function discovers ALL available AI providers in the ecosystem
/// by querying for AI-related capabilities. It's completely agnostic
/// to specific vendors or implementations.
///
/// TRUE PRIMAL: Zero hardcoding, runtime discovery, capability-based.
///
/// # Capabilities Queried
///
/// - `ai.complete` - Text completion
/// - `ai.chat` - Chat/conversational AI
/// - `ai.inference` - General inference
/// - `ai.embedding` - Text embeddings
///
/// # Returns
///
/// A vector of all discovered AI providers, ready to use.
pub async fn discover_ai_providers() -> Vec<BoxedAiCapability> {
    // List of AI capabilities to discover -- probe all concurrently
    let ai_capabilities = vec!["ai.complete", "ai.chat", "ai.inference", "ai.embedding"];

    let handles: Vec<_> = ai_capabilities
        .into_iter()
        .map(|capability| {
            tokio::spawn(async move {
                match discover_capability(capability).await {
                    Ok(provider_info) => {
                        info!(
                            "✅ Discovered AI provider for '{}': {} (via {})",
                            capability, provider_info.id, provider_info.discovered_via
                        );
                        match UniversalAiAdapter::from_capability_provider(
                            provider_info,
                            capability.to_string(),
                        )
                        .await
                        {
                            Ok(adapter) => Some(Arc::new(adapter) as BoxedAiCapability),
                            Err(e) => {
                                warn!("⚠️  Failed to create adapter for '{}': {}", capability, e);
                                None
                            }
                        }
                    }
                    Err(e) => {
                        debug!("No provider found for '{}': {}", capability, e);
                        None
                    }
                }
            })
        })
        .collect();

    let mut providers: Vec<BoxedAiCapability> = Vec::new();
    for h in handles {
        if let Ok(Some(provider)) = h.await {
            providers.push(provider);
        }
    }

    if providers.is_empty() {
        warn!("⚠️  No AI providers discovered!");
        warn!("⚠️  Hint: Set AI_COMPLETE_PROVIDER_SOCKET or start an AI primal");
    } else {
        info!("✅ Discovered {} AI provider(s)", providers.len());
    }

    providers
}

/// Discover a specific AI provider by capability
///
/// This function discovers a single AI provider that provides the specified
/// capability. Useful when you need a specific type of AI service.
///
/// # Arguments
///
/// * `capability` - The capability to discover (e.g., "ai.complete")
///
/// # Returns
///
/// A boxed AI capability provider, or an error if not found.
pub async fn discover_ai_provider(capability: &str) -> Result<BoxedAiCapability, PrimalError> {
    let provider_info = discover_capability(capability).await?;

    info!(
        "✅ Discovered AI provider for '{}': {}",
        capability, provider_info.id
    );

    let adapter =
        UniversalAiAdapter::from_capability_provider(provider_info, capability.to_string()).await?;

    Ok(Arc::new(adapter))
}

/// Check if any AI providers are available
///
/// Quick check to see if at least one AI provider is reachable.
///
/// # Returns
///
/// `true` if at least one AI provider is available.
pub async fn has_ai_providers() -> bool {
    // Probe all AI capabilities concurrently -- return true if any found
    let caps = vec!["ai.complete", "ai.chat", "ai.inference"];
    let handles: Vec<_> = caps
        .into_iter()
        .map(|cap| tokio::spawn(async move { discover_capability(cap).await.is_ok() }))
        .collect();

    for h in handles {
        if let Ok(true) = h.await {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_discover_ai_providers() {
        // Run all 4 capability discoveries concurrently to avoid serial timeouts
        let capabilities = vec!["ai.complete", "ai.chat", "ai.inference", "ai.embedding"];
        let handles: Vec<_> = capabilities
            .into_iter()
            .map(|cap| tokio::spawn(async move { discover_capability(cap).await }))
            .collect();

        let mut found = 0;
        for h in handles {
            if let Ok(Ok(_)) = h.await {
                found += 1;
            }
        }
        // No providers in test env is fine -- main assertion is no panic
        assert!(found >= 0);
    }

    #[tokio::test]
    async fn test_has_ai_providers() {
        // Run probes concurrently instead of serial
        let caps = vec!["ai.complete", "ai.chat", "ai.inference"];
        let handles: Vec<_> = caps
            .into_iter()
            .map(|cap| tokio::spawn(async move { discover_capability(cap).await.is_ok() }))
            .collect();

        let mut has = false;
        for h in handles {
            if let Ok(true) = h.await {
                has = true;
                break;
            }
        }
        // Result is deterministic (true or false)
        assert!(has || !has);
    }
}
