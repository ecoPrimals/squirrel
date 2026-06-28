// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Provider initialization helpers for [`super::router::AiRouter`].
//!
//! Vendor-specific HTTP adapters (Anthropic, OpenAI) were deleted in Wave 128.
//! All provider discovery is capability-based via `UniversalAiAdapter`.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::warn;

use super::adapters::{AiProvider, AiProviderAdapter, ProviderMetadata, UniversalAiAdapter};
use super::http_provider_config::HttpAiProviderConfig;
use super::router::AiRouter;
use crate::error::PrimalError;

impl AiRouter {
    /// Initialize an HTTP AI provider from config.
    ///
    /// Vendor-specific adapters have been removed. All provider discovery is
    /// capability-based via `UniversalAiAdapter`.
    pub(crate) async fn init_http_provider(
        config: &HttpAiProviderConfig,
    ) -> Result<Option<Arc<AiProvider>>, PrimalError> {
        if config.provider_id == "anthropic" || config.provider_id == "openai" {
            warn!(
                "Provider '{}' is no longer supported as a direct adapter. \
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
