// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! HTTP-based AI provider configuration (vendor-agnostic)
//!
//! This module provides a registry of HTTP-based AI providers that can be
//! discovered at runtime through environment variables, eliminating hardcoded
//! vendor dependencies.
//!
//! ## TRUE PRIMAL Compliance
//!
//! - ✅ Zero compile-time coupling to specific vendors
//! - ✅ Runtime configuration via environment
//! - ✅ HTTP delegation via capability discovery
//!
//! ## Usage
//!
//! ```bash
//! # Configure which HTTP providers to use
//! export AI_HTTP_PROVIDERS="anthropic,openai"
//! export ANTHROPIC_API_KEY="sk-..."
//! export OPENAI_API_KEY="sk-..."
//! export HTTP_REQUEST_PROVIDER_SOCKET="/run/user/1000/biomeos/songbird.sock"
//!
//! # Squirrel discovers and initializes providers at runtime
//! ./squirrel server
//! ```
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use universal_constants::ai_providers;

/// Configuration for an HTTP-based AI provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpAiProviderConfig {
    /// Provider identifier (e.g., "anthropic", "openai")
    pub provider_id: String,

    /// Display name
    pub provider_name: String,

    /// API base URL
    pub api_base: String,

    /// Environment variable name for API key
    pub api_key_env: String,

    /// Supported models
    pub models: Vec<String>,

    /// Additional headers required for this provider
    pub required_headers: HashMap<String, String>,

    /// API version (for providers that require it)
    pub api_version: Option<String>,
}

/// Get all known HTTP AI provider configurations
///
/// This is data, not code. Adding new providers requires zero code changes.
pub fn get_http_provider_configs() -> Vec<HttpAiProviderConfig> {
    vec![
        // Anthropic (Claude)
        HttpAiProviderConfig {
            provider_id: "anthropic".to_string(),
            provider_name: "Anthropic".to_string(),
            api_base: ai_providers::anthropic_base_url(),
            api_key_env: "ANTHROPIC_API_KEY".to_string(),
            models: vec![
                "claude-3-opus-20240229".to_string(),
                "claude-3-sonnet-20240229".to_string(),
                "claude-3-haiku-20240307".to_string(),
            ],
            required_headers: std::iter::once((
                "anthropic-version".to_string(),
                "2023-06-01".to_string(),
            ))
            .collect(),
            api_version: Some("2023-06-01".to_string()),
        },
        // OpenAI (GPT)
        HttpAiProviderConfig {
            provider_id: "openai".to_string(),
            provider_name: "OpenAI".to_string(),
            api_base: ai_providers::openai_base_url(),
            api_key_env: "OPENAI_API_KEY".to_string(),
            models: vec![
                "gpt-4".to_string(),
                "gpt-4-turbo-preview".to_string(),
                "gpt-3.5-turbo".to_string(),
            ],
            required_headers: HashMap::new(),
            api_version: None,
        },
        // 🎯 EXTENSIBILITY: Add more providers here (data-driven!)
        // No code changes needed - just add configuration.
        //
        // Example for Google Gemini:
        // HttpAiProviderConfig {
        //     provider_id: "gemini".to_string(),
        //     provider_name: "Google Gemini".to_string(),
        //     api_base: "https://generativelanguage.googleapis.com/v1".to_string(),
        //     api_key_env: "GEMINI_API_KEY".to_string(),
        //     models: vec!["gemini-pro".to_string()],
        //     required_headers: HashMap::new(),
        //     api_version: None,
        // },
    ]
}

/// Find a provider configuration by ID
pub fn find_provider_config(provider_id: &str) -> Option<HttpAiProviderConfig> {
    get_http_provider_configs()
        .into_iter()
        .find(|c| c.provider_id == provider_id)
}

/// Parse the AI_HTTP_PROVIDERS environment variable and return enabled providers
///
/// Format: Comma-separated list of provider IDs
/// Example: "anthropic,openai" or "anthropic" or "openai,gemini"
pub fn get_enabled_http_providers() -> Vec<HttpAiProviderConfig> {
    match std::env::var("AI_HTTP_PROVIDERS") {
        Ok(providers_str) => {
            let all_configs = get_http_provider_configs();
            providers_str
                .split(',')
                .map(str::trim)
                .filter_map(|provider_id| {
                    all_configs
                        .iter()
                        .find(|c| c.provider_id == provider_id)
                        .cloned()
                })
                .collect()
        }
        Err(_) => {
            // Default: Try all configured providers that have API keys set
            get_http_provider_configs()
                .into_iter()
                .filter(|config| std::env::var(&config.api_key_env).is_ok())
                .collect()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_http_provider_configs() {
        let configs = get_http_provider_configs();
        assert!(!configs.is_empty());

        // Should have at least Anthropic and OpenAI
        assert!(configs.iter().any(|c| c.provider_id == "anthropic"));
        assert!(configs.iter().any(|c| c.provider_id == "openai"));
    }

    #[test]
    fn test_find_provider_config() {
        let anthropic = find_provider_config("anthropic");
        assert!(anthropic.is_some());
        assert_eq!(
            anthropic.expect("should succeed").provider_name,
            "Anthropic"
        );

        let openai = find_provider_config("openai");
        assert!(openai.is_some());
        assert_eq!(openai.expect("should succeed").provider_name, "OpenAI");

        let unknown = find_provider_config("unknown");
        assert!(unknown.is_none());
    }

    #[test]
    fn test_get_enabled_http_providers_no_env() {
        // Without AI_HTTP_PROVIDERS, should return providers with API keys set
        // (This test depends on environment, so just check it doesn't panic)
        let enabled = get_enabled_http_providers();
        // Could be empty if no API keys are set
        assert!(enabled.len() <= get_http_provider_configs().len());
    }
}
