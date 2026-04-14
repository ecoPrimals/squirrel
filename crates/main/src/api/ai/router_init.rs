// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Provider initialization helpers for [`super::router::AiRouter`].
//!
//! Extracted from `router.rs` for module size management.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::warn;

use super::adapters::{AiProvider, AiProviderAdapter, ProviderMetadata, UniversalAiAdapter};
use super::http_provider_config::HttpAiProviderConfig;
use super::router::AiRouter;
use crate::error::PrimalError;

#[cfg(feature = "deprecated-adapters")]
use super::adapters::{AnthropicAdapter, OpenAiAdapter};

impl AiRouter {
    /// Initialize an HTTP AI provider from config.
    ///
    /// **Note**: Vendor-specific adapters (Anthropic, OpenAI) are deprecated
    /// and gated behind the `deprecated-adapters` feature.
    pub(crate) async fn init_http_provider(
        config: &HttpAiProviderConfig,
    ) -> Result<Option<Arc<AiProvider>>, PrimalError> {
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
            let adapter_result: Result<Arc<AiProvider>, PrimalError> =
                match config.provider_id.as_str() {
                    "anthropic" =>
                    {
                        #[expect(
                            deprecated,
                            reason = "backward compat: AnthropicAdapter until v0.3.0 removal"
                        )]
                        match AnthropicAdapter::new() {
                            Ok(adapter) => Ok(Arc::new(AiProvider::Anthropic(adapter))),
                            Err(e) => Err(e),
                        }
                    }
                    "openai" =>
                    {
                        #[expect(
                            deprecated,
                            reason = "backward compat: OpenAiAdapter until v0.3.0 removal"
                        )]
                        match OpenAiAdapter::new() {
                            Ok(adapter) => Ok(Arc::new(AiProvider::OpenAi(adapter))),
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
                    match tokio::time::timeout(
                        std::time::Duration::from_secs(5),
                        adapter.is_available(),
                    )
                    .await
                    {
                        Ok(true) => Ok(Some(adapter)),
                        Ok(false) => Ok(None),
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

    /// Create `UniversalAiAdapter` from a socket path (helper for discovery).
    pub(crate) async fn create_universal_adapter_from_path(
        socket_path: &str,
    ) -> Result<UniversalAiAdapter, PrimalError> {
        let path = PathBuf::from(socket_path);
        let file_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");

        let metadata = ProviderMetadata {
            primal_id: file_name.to_string(),
            name: format!("Universal AI ({file_name})"),
            is_local: Some(true),
            quality: Some("standard".to_string()),
            cost: Some(0.0),
            max_tokens: Some(4096),
            additional: HashMap::new(),
        };

        let adapter = UniversalAiAdapter::from_discovery("ai:text-generation", path, metadata);

        if !adapter.is_available().await {
            return Err(PrimalError::OperationFailed(format!(
                "Provider at {socket_path} is not available"
            )));
        }

        Ok(adapter)
    }
}
