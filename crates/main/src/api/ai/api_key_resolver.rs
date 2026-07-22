// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! AI API key resolution — bearDog `secrets.*` with env-var fallback.
//!
//! When bearDog is running, API keys are retrieved via `secrets.retrieve`.
//! When bearDog is unavailable, the resolver falls back to environment
//! variables (the legacy path).  This provides zero-downtime migration:
//! existing env-var setups keep working while bearDog gradually absorbs
//! credential management.

use squirrel_mcp::security::secret_store::SecretStore;
use tracing::debug;

/// Resolve an AI provider's API key.
///
/// 1. Try `store.get(secret_name)` — bearDog's encrypted credential store.
/// 2. Fall back to `std::env::var(env_var)` — legacy env-var path.
///
/// `secret_name` is the bearDog-side key (e.g. `"openai_api_key"`).
/// `env_var` is the environment variable (e.g. `"OPENAI_API_KEY"`).
pub async fn resolve_api_key(
    store: &impl SecretStore,
    secret_name: &str,
    env_var: &str,
) -> Option<String> {
    squirrel_mcp::security::security_provider_secret_store::resolve_secret_or_env(
        store, secret_name, env_var,
    )
    .await
}

/// Convert an env var name to a bearDog secret name.
///
/// `"OPENAI_API_KEY"` → `"openai_api_key"`
#[must_use]
pub fn env_var_to_secret_name(env_var: &str) -> String {
    env_var.to_ascii_lowercase()
}

/// Check whether an API key is available for a provider (async).
///
/// Returns `true` if either the security provider or env var yields a key.
pub async fn is_api_key_available(
    store: &impl SecretStore,
    env_var: &str,
) -> bool {
    let secret_name = env_var_to_secret_name(env_var);
    resolve_api_key(store, &secret_name, env_var).await.is_some()
}

/// Filter provider configs to only those with available API keys.
///
/// This is the async replacement for the sync `std::env::var().is_ok()` filter
/// in `get_enabled_http_providers`.
pub async fn filter_providers_with_keys<T: HasApiKeyEnv>(
    store: &impl SecretStore,
    providers: Vec<T>,
) -> Vec<T> {
    let mut available = Vec::with_capacity(providers.len());
    for provider in providers {
        if is_api_key_available(store, provider.api_key_env()).await {
            debug!(
                env_var = provider.api_key_env(),
                "API key available for provider"
            );
            available.push(provider);
        }
    }
    available
}

/// Trait for config types that carry an API key env var name.
pub trait HasApiKeyEnv {
    fn api_key_env(&self) -> &str;
}

impl HasApiKeyEnv for super::http_provider_config::HttpAiProviderConfig {
    fn api_key_env(&self) -> &str {
        &self.api_key_env
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use squirrel_mcp::security::secret_store::InMemorySecretStore;

    #[test]
    fn env_var_to_secret_name_lowercases() {
        assert_eq!(env_var_to_secret_name("OPENAI_API_KEY"), "openai_api_key");
        assert_eq!(
            env_var_to_secret_name("ANTHROPIC_API_KEY"),
            "anthropic_api_key"
        );
        assert_eq!(env_var_to_secret_name("already_lower"), "already_lower");
    }

    #[tokio::test]
    async fn resolve_api_key_from_store() {
        let store = InMemorySecretStore::new();
        store
            .set("openai_api_key", b"sk-from-store".to_vec())
            .await
            .unwrap();

        let key = resolve_api_key(&store, "openai_api_key", "NONEXISTENT_ENV_XYZ").await;
        assert_eq!(key, Some("sk-from-store".to_string()));
    }

    #[test]
    fn resolve_api_key_env_fallback() {
        let store = InMemorySecretStore::new();
        temp_env::with_var("TEST_AI_KEY_RESOLVE_XYZ", Some("sk-from-env"), || {
            let rt = tokio::runtime::Runtime::new().expect("runtime");
            let key = rt.block_on(resolve_api_key(
                &store,
                "missing_key",
                "TEST_AI_KEY_RESOLVE_XYZ",
            ));
            assert_eq!(key, Some("sk-from-env".to_string()));
        });
    }

    #[tokio::test]
    async fn is_api_key_available_true_from_store() {
        let store = InMemorySecretStore::new();
        store
            .set("openai_api_key", b"sk-test".to_vec())
            .await
            .unwrap();

        assert!(is_api_key_available(&store, "OPENAI_API_KEY").await);
    }

    #[tokio::test]
    async fn is_api_key_available_false_when_missing() {
        let store = InMemorySecretStore::new();
        assert!(!is_api_key_available(&store, "TOTALLY_MISSING_KEY_XYZ_999").await);
    }

    #[tokio::test]
    async fn filter_providers_keeps_available() {
        let store = InMemorySecretStore::new();
        store
            .set("test_key_a", b"val".to_vec())
            .await
            .unwrap();

        struct FakeProvider(&'static str);
        impl HasApiKeyEnv for FakeProvider {
            fn api_key_env(&self) -> &str {
                self.0
            }
        }

        let providers = vec![
            FakeProvider("TEST_KEY_A"),
            FakeProvider("TEST_KEY_B_MISSING"),
        ];

        let filtered = filter_providers_with_keys(&store, providers).await;
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].api_key_env(), "TEST_KEY_A");
    }
}
