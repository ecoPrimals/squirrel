// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! AI client implementations
//!
//! This module contains concrete implementations of AI clients for various providers.
//! Each client implements the `AIClient` trait for seamless integration.
//!
//! **Note**: Old vendor-specific HTTP clients have been replaced with capability-based routing.
//! Use `capability_ai::AiClient` instead for TRUE ecoBin compliance.

pub mod mock;

#[cfg(test)]
pub use mock::MockAIClient;

use crate::common::client::AIClient;
use std::sync::Arc;

/// Client factory for creating AI clients
/// **Note**: Old provider-specific methods removed. Use `capability_ai::AiClient` instead.
pub struct ClientFactory;

impl ClientFactory {
    /// Create a mock client for testing
    #[cfg(test)]
    #[must_use]
    pub fn create_mock_client() -> Arc<dyn AIClient> {
        Arc::new(MockAIClient::new())
    }
}

/// Client configuration
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// API key for authentication
    pub api_key: Option<String>,
    /// Custom endpoint URL
    pub endpoint: Option<String>,
    /// Model to use by default
    pub default_model: Option<String>,
    /// Maximum context size
    pub max_context_size: Option<usize>,
    /// Request timeout in seconds
    pub timeout_seconds: Option<u64>,
    /// Additional configuration options
    pub extra_config: std::collections::HashMap<String, String>,
}

impl ClientConfig {
    /// Create a new client configuration
    #[must_use]
    pub fn new() -> Self {
        Self {
            api_key: None,
            endpoint: None,
            default_model: None,
            max_context_size: None,
            timeout_seconds: None,
            extra_config: std::collections::HashMap::new(),
        }
    }

    /// Set the API key
    #[must_use]
    pub fn with_api_key(mut self, api_key: String) -> Self {
        self.api_key = Some(api_key);
        self
    }

    /// Set the endpoint
    #[must_use]
    pub fn with_endpoint(mut self, endpoint: String) -> Self {
        self.endpoint = Some(endpoint);
        self
    }

    /// Set the default model
    #[must_use]
    pub fn with_default_model(mut self, model: String) -> Self {
        self.default_model = Some(model);
        self
    }

    /// Set the maximum context size
    #[must_use]
    pub const fn with_max_context_size(mut self, size: usize) -> Self {
        self.max_context_size = Some(size);
        self
    }

    /// Set the request timeout
    #[must_use]
    pub const fn with_timeout(mut self, seconds: u64) -> Self {
        self.timeout_seconds = Some(seconds);
        self
    }

    /// Add extra configuration
    #[must_use]
    pub fn with_extra_config(mut self, key: String, value: String) -> Self {
        self.extra_config.insert(key, value);
        self
    }

    /// Check if the configuration is valid for a provider
    /// **Note**: Old providers removed. Use `capability_ai` instead.
    #[must_use]
    pub const fn is_valid_for_provider(&self, _provider: &str) -> bool {
        // Old provider validation removed
        // Use capability_ai::AiClient::from_env() instead
        false
    }
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Client registry for managing multiple clients
pub struct ClientRegistry {
    clients: std::collections::HashMap<String, Arc<dyn AIClient>>,
}

impl ClientRegistry {
    /// Create a new client registry
    #[must_use]
    pub fn new() -> Self {
        Self {
            clients: std::collections::HashMap::new(),
        }
    }

    /// Add a client to the registry
    pub fn add_client(&mut self, name: String, client: Arc<dyn AIClient>) {
        self.clients.insert(name, client);
    }

    /// Get a client by name
    #[must_use]
    pub fn get_client(&self, name: &str) -> Option<&Arc<dyn AIClient>> {
        self.clients.get(name)
    }

    /// Remove a client from the registry
    pub fn remove_client(&mut self, name: &str) -> Option<Arc<dyn AIClient>> {
        self.clients.remove(name)
    }

    /// List all client names
    #[must_use]
    pub fn list_clients(&self) -> Vec<String> {
        self.clients.keys().cloned().collect()
    }

    /// Get all clients
    #[must_use]
    pub fn get_all_clients(&self) -> Vec<(String, Arc<dyn AIClient>)> {
        self.clients
            .iter()
            .map(|(name, client)| (name.clone(), client.clone()))
            .collect()
    }

    /// Check if a client exists
    #[must_use]
    pub fn has_client(&self, name: &str) -> bool {
        self.clients.contains_key(name)
    }

    /// Get the number of clients
    #[must_use]
    pub fn client_count(&self) -> usize {
        self.clients.len()
    }

    /// Clear all clients
    pub fn clear(&mut self) {
        self.clients.clear();
    }
}

impl Default for ClientRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_config() {
        let config = ClientConfig::new()
            .with_api_key("test-key".to_string())
            .with_endpoint("https://api.test.com".to_string())
            .with_default_model("test-model".to_string())
            .with_max_context_size(4096)
            .with_timeout(30)
            .with_extra_config("custom".to_string(), "value".to_string());

        assert_eq!(config.api_key, Some("test-key".to_string()));
        assert_eq!(config.endpoint, Some("https://api.test.com".to_string()));
        assert_eq!(config.default_model, Some("test-model".to_string()));
        assert_eq!(config.max_context_size, Some(4096));
        assert_eq!(config.timeout_seconds, Some(30));
        assert_eq!(
            config.extra_config.get("custom"),
            Some(&"value".to_string())
        );
    }

    #[test]
    fn test_client_config_validation() {
        let config = ClientConfig::new().with_api_key("test-key".to_string());
        // Old provider validation removed - use capability_ai instead
        assert!(!config.is_valid_for_provider("any"));
    }

    #[test]
    fn test_client_registry() {
        let mut registry = ClientRegistry::new();
        let client = ClientFactory::create_mock_client();

        registry.add_client("test-client".to_string(), client.clone());

        assert!(registry.has_client("test-client"));
        assert_eq!(registry.client_count(), 1);
        assert_eq!(registry.list_clients(), vec!["test-client".to_string()]);

        let retrieved = registry.get_client("test-client");
        assert!(retrieved.is_some());

        let removed = registry.remove_client("test-client");
        assert!(removed.is_some());
        assert_eq!(registry.client_count(), 0);
    }
}
