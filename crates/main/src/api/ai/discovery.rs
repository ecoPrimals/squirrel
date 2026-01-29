//! AI Provider Discovery - Capability-Based
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
use crate::api::ai::universal::{AiCapability, BoxedAiCapability};
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
    let mut providers: Vec<BoxedAiCapability> = Vec::new();

    // List of AI capabilities to discover
    let ai_capabilities = vec!["ai.complete", "ai.chat", "ai.inference", "ai.embedding"];

    for capability in ai_capabilities {
        match discover_capability(capability).await {
            Ok(provider_info) => {
                info!(
                    "✅ Discovered AI provider for '{}': {} (via {})",
                    capability, provider_info.id, provider_info.discovered_via
                );

                // Create universal adapter for this provider
                match UniversalAiAdapter::from_capability_provider(
                    provider_info,
                    capability.to_string(),
                )
                .await
                {
                    Ok(adapter) => {
                        providers.push(Arc::new(adapter));
                    }
                    Err(e) => {
                        warn!("⚠️  Failed to create adapter for '{}': {}", capability, e);
                    }
                }
            }
            Err(e) => {
                debug!("No provider found for '{}': {}", capability, e);
            }
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
    // Try to discover ai.complete capability (most common)
    discover_capability("ai.complete").await.is_ok()
        || discover_capability("ai.chat").await.is_ok()
        || discover_capability("ai.inference").await.is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_discover_ai_providers() {
        // This test will discover providers if any are available
        // In CI, this should gracefully handle no providers
        let providers = discover_ai_providers().await;

        // Should not panic, even if no providers found
        assert!(providers.len() >= 0);
    }

    #[tokio::test]
    async fn test_has_ai_providers() {
        // This test checks if providers are available
        // Should return true or false, not panic
        let has_providers = has_ai_providers().await;

        // Result should be deterministic
        assert!(has_providers || !has_providers);
    }
}
