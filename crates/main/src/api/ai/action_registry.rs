//! Dynamic action registry for provider-advertised capabilities
//!
//! The `ActionRegistry` is the heart of Phase 6 - it allows providers to
//! dynamically register new AI actions at runtime without code changes.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

/// Dynamic action provider registration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionProvider {
    /// Unique provider ID
    pub provider_id: String,

    /// Provider display name
    pub provider_name: String,

    /// Action this provider handles (e.g., "image.generation")
    pub action: String,

    /// Input schema (JSON Schema)
    pub input_schema: serde_json::Value,

    /// Output schema (JSON Schema)
    pub output_schema: serde_json::Value,

    /// Cost per unit (USD)
    pub cost_per_unit: Option<f64>,

    /// Average latency in milliseconds
    pub avg_latency_ms: u64,

    /// Quality tier
    pub quality: String,

    /// Reliability score (0.0 - 1.0)
    pub reliability: f64,

    /// Whether provider is local
    pub is_local: bool,

    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,

    /// Registration timestamp
    pub registered_at: chrono::DateTime<chrono::Utc>,
}

/// Registry for dynamically registered actions
pub struct ActionRegistry {
    /// Map of action -> list of providers
    actions: Arc<RwLock<HashMap<String, Vec<ActionProvider>>>>,
}

impl ActionRegistry {
    /// Create a new action registry
    #[must_use]
    pub fn new() -> Self {
        Self {
            actions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a new action provider
    pub async fn register_action(&self, provider: ActionProvider) {
        let action = provider.action.clone();
        let provider_id = provider.provider_id.clone();

        info!(
            "📝 Registering provider '{}' for action '{}'",
            provider_id, action
        );

        let mut actions = self.actions.write().await;
        let providers = actions.entry(action.clone()).or_insert_with(Vec::new);

        // Remove existing registration for this provider (if any)
        providers.retain(|p| p.provider_id != provider_id);

        // Add new registration
        providers.push(provider);

        info!(
            "✅ Registered '{}' for '{}' ({} total providers)",
            provider_id,
            action,
            providers.len()
        );
    }

    /// Find providers for a specific action
    pub async fn find_providers_for_action(&self, action: &str) -> Option<Vec<ActionProvider>> {
        let actions = self.actions.read().await;
        actions.get(action).cloned()
    }

    /// List all registered actions
    pub async fn list_all_actions(&self) -> Vec<String> {
        let actions = self.actions.read().await;
        actions.keys().cloned().collect()
    }

    /// Get all providers across all actions
    pub async fn list_all_providers(&self) -> Vec<ActionProvider> {
        let actions = self.actions.read().await;
        actions
            .values()
            .flat_map(|providers| providers.iter().cloned())
            .collect()
    }

    /// Deregister a provider
    pub async fn deregister_provider(&self, provider_id: &str) {
        let mut actions = self.actions.write().await;
        let mut deregistered_count = 0;

        for providers in actions.values_mut() {
            let before = providers.len();
            providers.retain(|p| p.provider_id != provider_id);
            deregistered_count += before - providers.len();
        }

        if deregistered_count > 0 {
            info!(
                "✅ Deregistered provider '{}' ({} actions)",
                provider_id, deregistered_count
            );
        } else {
            warn!("⚠️  Provider '{}' not found in registry", provider_id);
        }
    }

    /// Get statistics about registered actions
    pub async fn get_stats(&self) -> RegistryStats {
        let actions = self.actions.read().await;

        let total_actions = actions.len();
        let total_providers = actions.values().map(std::vec::Vec::len).sum();
        let available_actions = actions
            .iter()
            .filter(|(_, providers)| !providers.is_empty())
            .count();

        RegistryStats {
            total_actions,
            total_providers,
            available_actions,
            action_list: actions.keys().cloned().collect(),
        }
    }
}

impl Default for ActionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Registry statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryStats {
    pub total_actions: usize,
    pub total_providers: usize,
    pub available_actions: usize,
    pub action_list: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn mock_provider(id: &str, action: &str) -> ActionProvider {
        ActionProvider {
            provider_id: id.to_string(),
            provider_name: format!("Provider {}", id),
            action: action.to_string(),
            input_schema: serde_json::json!({"prompt": "string"}),
            output_schema: serde_json::json!({"result": "string"}),
            cost_per_unit: Some(0.01),
            avg_latency_ms: 1000,
            quality: "high".to_string(),
            reliability: 0.95,
            is_local: false,
            metadata: HashMap::new(),
            registered_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_register_and_find() {
        let registry = ActionRegistry::new();

        registry
            .register_action(mock_provider("p1", "test.action"))
            .await;

        let providers = registry.find_providers_for_action("test.action").await;
        assert!(providers.is_some());
        assert_eq!(providers.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_multiple_providers_same_action() {
        let registry = ActionRegistry::new();

        registry
            .register_action(mock_provider("p1", "test.action"))
            .await;
        registry
            .register_action(mock_provider("p2", "test.action"))
            .await;

        let providers = registry.find_providers_for_action("test.action").await;
        assert_eq!(providers.unwrap().len(), 2);
    }

    #[tokio::test]
    async fn test_deregister() {
        let registry = ActionRegistry::new();

        registry
            .register_action(mock_provider("p1", "test.action"))
            .await;
        registry.deregister_provider("p1").await;

        let providers = registry.find_providers_for_action("test.action").await;
        assert!(providers.is_none() || providers.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_list_all_actions() {
        let registry = ActionRegistry::new();

        registry
            .register_action(mock_provider("p1", "action1"))
            .await;
        registry
            .register_action(mock_provider("p2", "action2"))
            .await;

        let actions = registry.list_all_actions().await;
        assert_eq!(actions.len(), 2);
        assert!(actions.contains(&"action1".to_string()));
        assert!(actions.contains(&"action2".to_string()));
    }
}
