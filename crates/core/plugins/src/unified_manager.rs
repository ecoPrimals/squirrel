// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Unified Plugin Manager — lifecycle, event bus, and security.

use crate::errors::{PluginError, Result};
use crate::plugin::Plugin;
use dashmap::DashMap;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::Duration;
use tokio::sync::{RwLock, broadcast};

/// Configuration for the unified plugin manager.
#[derive(Debug, Clone)]
pub struct UnifiedManagerConfig {
    /// Plugin discovery paths.
    pub plugin_dirs: Vec<PathBuf>,
    /// Maximum number of plugins (default 64).
    pub max_plugins: usize,
    /// Load timeout (default 30s).
    pub load_timeout: Duration,
    /// Enable security checks (default true).
    pub enable_security: bool,
}

impl Default for UnifiedManagerConfig {
    fn default() -> Self {
        Self {
            plugin_dirs: Vec::new(),
            max_plugins: 64,
            load_timeout: Duration::from_secs(30),
            enable_security: true,
        }
    }
}

/// Metrics for plugin manager operations.
#[derive(Debug, Default)]
pub struct ManagerMetrics {
    /// Number of plugins successfully loaded.
    pub plugins_loaded: AtomicUsize,
    /// Number of plugins that failed to load.
    pub plugins_failed: AtomicUsize,
    /// Total load time in milliseconds.
    pub total_load_time_ms: AtomicU64,
}

/// Event message for plugin pub/sub.
#[derive(Clone, Debug)]
pub struct EventMessage {
    /// Topic name.
    pub topic: String,
    /// Event payload.
    pub payload: Value,
}

/// Event bus for plugin-to-plugin communication.
#[derive(Debug)]
pub struct PluginEventBus {
    topics: RwLock<HashMap<String, broadcast::Sender<EventMessage>>>,
}

impl PluginEventBus {
    /// Creates a new event bus.
    #[must_use]
    pub fn new() -> Self {
        Self {
            topics: RwLock::new(HashMap::new()),
        }
    }

    /// Publishes an event to a topic. Creates the channel if it does not exist.
    pub async fn publish(&self, topic: &str, payload: Value) -> Result<()> {
        const CHANNEL_CAP: usize = 32;
        let mut topics = self.topics.write().await;
        let sender = topics
            .entry(topic.to_string())
            .or_insert_with(|| broadcast::channel(CHANNEL_CAP).0);
        let _ = sender.send(EventMessage {
            topic: topic.to_string(),
            payload,
        });
        Ok(())
    }

    /// Subscribes to a topic. Creates the channel if it does not exist.
    pub async fn subscribe(&self, topic: &str) -> Result<broadcast::Receiver<EventMessage>> {
        const CHANNEL_CAP: usize = 32;
        let mut topics = self.topics.write().await;
        let sender = topics
            .entry(topic.to_string())
            .or_insert_with(|| broadcast::channel(CHANNEL_CAP).0);
        Ok(sender.subscribe())
    }
}

impl Default for PluginEventBus {
    fn default() -> Self {
        Self::new()
    }
}

/// Security manager for plugin sandboxing and policies.
#[derive(Debug)]
pub struct PluginSecurityManager {
    allowed_capabilities: HashSet<String>,
}

impl PluginSecurityManager {
    /// Creates a new security manager with the given allowed capabilities.
    #[must_use]
    pub const fn new(allowed_capabilities: HashSet<String>) -> Self {
        Self {
            allowed_capabilities,
        }
    }

    /// Checks whether a capability is permitted.
    #[must_use]
    pub fn is_allowed(&self, capability: &str) -> bool {
        self.allowed_capabilities.contains(capability)
    }

    /// Checks permissions for a set of requested capabilities.
    pub fn check_permissions(&self, requested: &[String]) -> Result<()> {
        for cap in requested {
            if !self.is_allowed(cap) {
                return Err(PluginError::SecurityError(format!(
                    "Capability not allowed: {cap}"
                )));
            }
        }
        Ok(())
    }
}

/// Loader for plugin discovery. Holds config for future loading.
#[derive(Debug)]
pub struct UnifiedPluginLoader {
    /// Plugin directories to search.
    pub plugin_dirs: Vec<PathBuf>,
}

impl UnifiedPluginLoader {
    /// Creates a new loader with the given plugin directories.
    #[must_use]
    pub const fn new(plugin_dirs: Vec<PathBuf>) -> Self {
        Self { plugin_dirs }
    }
}

/// Internal plugin entry.
struct PluginEntry {
    plugin: Arc<dyn Plugin>,
}

impl std::fmt::Debug for PluginEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PluginEntry")
            .field("name", &self.plugin.metadata().name)
            .finish()
    }
}

/// Unified plugin manager combining lifecycle, metrics, and security.
#[derive(Debug)]
pub struct UnifiedPluginManager {
    config: UnifiedManagerConfig,
    plugins: DashMap<String, PluginEntry>,
    metrics: Arc<ManagerMetrics>,
    security: Arc<PluginSecurityManager>,
}

impl UnifiedPluginManager {
    /// Creates a new unified plugin manager.
    #[must_use]
    pub fn new(config: UnifiedManagerConfig) -> Self {
        let allowed: HashSet<String> = ["read", "write", "network"]
            .into_iter()
            .map(String::from)
            .collect();
        Self {
            config,
            plugins: DashMap::new(),
            metrics: Arc::new(ManagerMetrics::default()),
            security: Arc::new(PluginSecurityManager::new(allowed)),
        }
    }

    /// Loads a plugin. Enforces `max_plugins` and security when enabled.
    pub async fn load_plugin(&self, plugin: Arc<dyn Plugin>) -> Result<()> {
        let name = plugin.metadata().name.clone();
        if self.plugins.len() >= self.config.max_plugins {
            return Err(PluginError::ConfigurationError(format!(
                "Max plugins ({}) reached",
                self.config.max_plugins
            )));
        }
        if self.config.enable_security {
            self.security
                .check_permissions(&plugin.metadata().capabilities)?;
        }
        self.plugins.insert(
            name,
            PluginEntry {
                plugin: plugin.clone(),
            },
        );
        self.metrics.plugins_loaded.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    /// Unloads a plugin by name.
    pub async fn unload_plugin(&self, name: &str) -> Result<()> {
        self.plugins
            .remove(name)
            .map(|_| ())
            .ok_or_else(|| PluginError::PluginNotFound(name.to_string()))
    }

    /// Lists all loaded plugin names.
    #[must_use]
    pub fn list_plugins(&self) -> Vec<String> {
        self.plugins.iter().map(|r| r.key().clone()).collect()
    }

    /// Gets a plugin by name.
    pub fn get_plugin(&self, name: &str) -> Result<Arc<dyn Plugin>> {
        self.plugins
            .get(name)
            .map(|e| e.value().plugin.clone())
            .ok_or_else(|| PluginError::PluginNotFound(name.to_string()))
    }

    /// Shuts down all plugins.
    pub async fn shutdown(&self) -> Result<()> {
        for entry in &self.plugins {
            let _ = entry.value().plugin.shutdown().await;
        }
        self.plugins.clear();
        Ok(())
    }

    /// Returns the manager metrics.
    #[must_use]
    pub fn metrics(&self) -> Arc<ManagerMetrics> {
        Arc::clone(&self.metrics)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::PluginMetadata;
    use async_trait::async_trait;
    use std::any::Any;

    /// Test plugin implementation for unified manager tests.
    struct TestPlugin {
        metadata: PluginMetadata,
    }

    impl TestPlugin {
        fn with_capabilities(name: &str, capabilities: Vec<String>) -> Self {
            let mut meta = PluginMetadata::new(name, "1.0.0", "Test plugin", "Test");
            meta.capabilities = capabilities;
            Self { metadata: meta }
        }
    }

    #[async_trait]
    impl Plugin for TestPlugin {
        fn metadata(&self) -> &PluginMetadata {
            &self.metadata
        }

        async fn initialize(&self) -> anyhow::Result<()> {
            Ok(())
        }

        async fn shutdown(&self) -> anyhow::Result<()> {
            Ok(())
        }

        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    // --- UnifiedManagerConfig ---

    #[test]
    fn test_unified_manager_config_default() {
        let config = UnifiedManagerConfig::default();
        assert!(config.plugin_dirs.is_empty());
        assert_eq!(config.max_plugins, 64);
        assert_eq!(config.load_timeout, Duration::from_secs(30));
        assert!(config.enable_security);
    }

    // --- ManagerMetrics ---

    #[test]
    fn test_manager_metrics_default() {
        let metrics = ManagerMetrics::default();
        assert_eq!(metrics.plugins_loaded.load(Ordering::Relaxed), 0);
        assert_eq!(metrics.plugins_failed.load(Ordering::Relaxed), 0);
        assert_eq!(metrics.total_load_time_ms.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_manager_metrics_atomic_operations() {
        let metrics = ManagerMetrics::default();
        metrics.plugins_loaded.fetch_add(3, Ordering::Relaxed);
        metrics.plugins_failed.fetch_add(1, Ordering::Relaxed);
        metrics.total_load_time_ms.fetch_add(100, Ordering::Relaxed);

        assert_eq!(metrics.plugins_loaded.load(Ordering::Relaxed), 3);
        assert_eq!(metrics.plugins_failed.load(Ordering::Relaxed), 1);
        assert_eq!(metrics.total_load_time_ms.load(Ordering::Relaxed), 100);
    }

    // --- UnifiedPluginManager::new ---

    #[test]
    fn test_unified_plugin_manager_new() {
        let config = UnifiedManagerConfig::default();
        let manager = UnifiedPluginManager::new(config);
        assert!(manager.list_plugins().is_empty());
        assert_eq!(manager.metrics().plugins_loaded.load(Ordering::Relaxed), 0);
    }

    // --- Lifecycle: load, unload, list, get ---

    #[tokio::test]
    async fn test_load_unload_list_get_plugin() {
        let manager = UnifiedPluginManager::new(UnifiedManagerConfig::default());

        let plugin = Arc::new(TestPlugin::with_capabilities(
            "test_plugin",
            vec!["read".to_string()],
        ));

        manager.load_plugin(plugin.clone()).await.unwrap();

        assert_eq!(manager.list_plugins(), vec!["test_plugin"]);
        let retrieved = manager.get_plugin("test_plugin").unwrap();
        assert_eq!(retrieved.metadata().name, "test_plugin");

        manager.unload_plugin("test_plugin").await.unwrap();
        assert!(manager.list_plugins().is_empty());
        assert!(manager.get_plugin("test_plugin").is_err());
    }

    #[tokio::test]
    async fn test_load_multiple_plugins() {
        let manager = UnifiedPluginManager::new(UnifiedManagerConfig::default());

        let p1 = Arc::new(TestPlugin::with_capabilities(
            "p1",
            vec!["read".to_string()],
        ));
        let p2 = Arc::new(TestPlugin::with_capabilities(
            "p2",
            vec!["write".to_string()],
        ));
        let p3 = Arc::new(TestPlugin::with_capabilities(
            "p3",
            vec!["network".to_string()],
        ));

        manager.load_plugin(p1).await.unwrap();
        manager.load_plugin(p2).await.unwrap();
        manager.load_plugin(p3).await.unwrap();

        let mut list = manager.list_plugins();
        list.sort();
        assert_eq!(list, vec!["p1", "p2", "p3"]);

        assert_eq!(manager.metrics().plugins_loaded.load(Ordering::Relaxed), 3);
    }

    // --- Max plugins limit ---

    #[tokio::test]
    async fn test_max_plugins_limit_enforcement() {
        let config = UnifiedManagerConfig {
            max_plugins: 2,
            ..Default::default()
        };

        let manager = UnifiedPluginManager::new(config);

        let p1 = Arc::new(TestPlugin::with_capabilities(
            "p1",
            vec!["read".to_string()],
        ));
        let p2 = Arc::new(TestPlugin::with_capabilities(
            "p2",
            vec!["read".to_string()],
        ));
        let p3 = Arc::new(TestPlugin::with_capabilities(
            "p3",
            vec!["read".to_string()],
        ));

        manager.load_plugin(p1).await.unwrap();
        manager.load_plugin(p2).await.unwrap();

        let err = manager.load_plugin(p3).await.unwrap_err();
        assert!(matches!(err, PluginError::ConfigurationError(_)));
        assert!(err.to_string().contains("Max plugins (2) reached"));

        assert_eq!(manager.list_plugins().len(), 2);
    }

    // --- Plugin security manager ---

    #[tokio::test]
    async fn test_security_plugin_with_allowed_capability() {
        let manager = UnifiedPluginManager::new(UnifiedManagerConfig::default());

        let plugin = Arc::new(TestPlugin::with_capabilities(
            "allowed_plugin",
            vec!["read".to_string(), "write".to_string()],
        ));

        manager.load_plugin(plugin).await.unwrap();
        assert_eq!(manager.list_plugins(), vec!["allowed_plugin"]);
    }

    #[tokio::test]
    async fn test_security_plugin_with_disallowed_capability() {
        let manager = UnifiedPluginManager::new(UnifiedManagerConfig::default());

        let plugin = Arc::new(TestPlugin::with_capabilities(
            "disallowed_plugin",
            vec!["filesystem".to_string()],
        ));

        let err = manager.load_plugin(plugin).await.unwrap_err();
        assert!(matches!(err, PluginError::SecurityError(_)));
        assert!(
            err.to_string()
                .contains("Capability not allowed: filesystem")
        );
    }

    #[tokio::test]
    async fn test_security_disabled_allows_any_capability() {
        let config = UnifiedManagerConfig {
            enable_security: false,
            ..Default::default()
        };

        let manager = UnifiedPluginManager::new(config);

        let plugin = Arc::new(TestPlugin::with_capabilities(
            "unsafe_plugin",
            vec!["filesystem".to_string(), "sudo".to_string()],
        ));

        manager.load_plugin(plugin).await.unwrap();
        assert_eq!(manager.list_plugins(), vec!["unsafe_plugin"]);
    }

    #[test]
    fn test_plugin_security_manager_is_allowed() {
        let allowed: HashSet<String> = ["read", "write"].into_iter().map(String::from).collect();
        let security = PluginSecurityManager::new(allowed);

        assert!(security.is_allowed("read"));
        assert!(security.is_allowed("write"));
        assert!(!security.is_allowed("network"));
        assert!(!security.is_allowed("filesystem"));
    }

    #[test]
    fn test_plugin_security_manager_check_permissions() {
        let allowed: HashSet<String> = ["read", "write"].into_iter().map(String::from).collect();
        let security = PluginSecurityManager::new(allowed);

        assert!(security.is_allowed("read"));
        assert!(
            security
                .check_permissions(&["read".to_string(), "write".to_string()])
                .is_ok()
        );
        assert!(security.check_permissions(&["read".to_string()]).is_ok());

        let err = security
            .check_permissions(&["read".to_string(), "network".to_string()])
            .unwrap_err();
        assert!(matches!(err, PluginError::SecurityError(_)));
    }

    // --- Plugin event bus ---

    #[tokio::test]
    async fn test_event_bus_publish_subscribe() {
        let bus = PluginEventBus::new();

        let mut sub = bus.subscribe("events").await.unwrap();

        bus.publish("events", serde_json::json!({"message": "hello"}))
            .await
            .unwrap();

        let msg = sub.recv().await.unwrap();
        assert_eq!(msg.topic, "events");
        assert_eq!(msg.payload, serde_json::json!({"message": "hello"}));
    }

    #[tokio::test]
    async fn test_event_bus_multiple_subscribers() {
        let bus = PluginEventBus::new();

        let mut sub1 = bus.subscribe("topic").await.unwrap();
        let mut sub2 = bus.subscribe("topic").await.unwrap();

        bus.publish("topic", serde_json::json!({"n": 42}))
            .await
            .unwrap();

        let msg1 = sub1.recv().await.unwrap();
        let msg2 = sub2.recv().await.unwrap();

        assert_eq!(msg1.payload, serde_json::json!({"n": 42}));
        assert_eq!(msg2.payload, serde_json::json!({"n": 42}));
    }

    #[tokio::test]
    async fn test_event_bus_multiple_topics() {
        let bus = PluginEventBus::new();

        let mut sub_a = bus.subscribe("topic_a").await.unwrap();
        let mut sub_b = bus.subscribe("topic_b").await.unwrap();

        bus.publish("topic_a", serde_json::json!("a"))
            .await
            .unwrap();
        bus.publish("topic_b", serde_json::json!("b"))
            .await
            .unwrap();

        assert_eq!(sub_a.recv().await.unwrap().payload, serde_json::json!("a"));
        assert_eq!(sub_b.recv().await.unwrap().payload, serde_json::json!("b"));
    }

    #[test]
    fn test_event_bus_default() {
        let _bus = PluginEventBus::default();
    }

    // --- Shutdown ---

    #[tokio::test]
    async fn test_shutdown_clears_plugins() {
        let manager = UnifiedPluginManager::new(UnifiedManagerConfig::default());

        let p1 = Arc::new(TestPlugin::with_capabilities(
            "p1",
            vec!["read".to_string()],
        ));
        manager.load_plugin(p1).await.unwrap();

        manager.shutdown().await.unwrap();

        assert!(manager.list_plugins().is_empty());
        assert!(manager.get_plugin("p1").is_err());
    }

    #[tokio::test]
    async fn test_shutdown_idempotent() {
        let manager = UnifiedPluginManager::new(UnifiedManagerConfig::default());
        manager.shutdown().await.unwrap();
        manager.shutdown().await.unwrap();
    }

    // --- Error paths ---

    #[tokio::test]
    async fn test_unload_nonexistent_plugin() {
        let manager = UnifiedPluginManager::new(UnifiedManagerConfig::default());

        let err = manager.unload_plugin("nonexistent").await.unwrap_err();
        assert!(matches!(err, PluginError::PluginNotFound(_)));
        assert!(err.to_string().contains("nonexistent"));
    }

    #[tokio::test]
    async fn test_get_nonexistent_plugin() {
        let manager = UnifiedPluginManager::new(UnifiedManagerConfig::default());

        let Err(err) = manager.get_plugin("nonexistent") else {
            panic!("get_plugin should fail for nonexistent plugin")
        };
        assert!(matches!(err, PluginError::PluginNotFound(_)));
        assert!(err.to_string().contains("nonexistent"));
    }

    #[tokio::test]
    async fn test_load_after_max_reached() {
        let config = UnifiedManagerConfig {
            max_plugins: 1,
            ..Default::default()
        };

        let manager = UnifiedPluginManager::new(config);

        let p1 = Arc::new(TestPlugin::with_capabilities(
            "p1",
            vec!["read".to_string()],
        ));
        let p2 = Arc::new(TestPlugin::with_capabilities(
            "p2",
            vec!["read".to_string()],
        ));

        manager.load_plugin(p1).await.unwrap();
        let err = manager.load_plugin(p2).await.unwrap_err();

        assert!(matches!(err, PluginError::ConfigurationError(_)));
    }
}
