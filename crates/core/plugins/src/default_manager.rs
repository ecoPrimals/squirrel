//! Default plugin manager implementation
//!
//! This module provides the default implementation of the plugin manager.

use crate::dependency_resolver::DependencyResolver;
use crate::discovery::DefaultPluginDiscovery;
use crate::metrics::{PluginManagerMetrics, PluginManagerStatus};
use crate::state::{MemoryStateManager, PluginStateManager};
use crate::types::PluginStatus;
use crate::Plugin;
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Default plugin manager implementation
pub struct DefaultPluginManager {
    /// Registered plugins
    plugins: Arc<RwLock<HashMap<Uuid, Arc<dyn Plugin>>>>,
    /// Plugin statuses
    statuses: RwLock<HashMap<Uuid, PluginStatus>>,
    /// Plugin name to ID mapping
    name_to_id: RwLock<HashMap<String, Uuid>>,
    /// Dependency resolver
    dependency_resolver: Arc<RwLock<DependencyResolver>>,
    /// State manager
    state_manager: Arc<dyn PluginStateManager>,
    /// Discovery service
    discovery: Arc<DefaultPluginDiscovery>,
    /// Performance metrics
    metrics: Arc<RwLock<PluginManagerMetrics>>,
}

impl DefaultPluginManager {
    /// Create a new default plugin manager
    pub fn new() -> Self {
        Self {
            plugins: Arc::new(RwLock::new(HashMap::new())),
            statuses: RwLock::new(HashMap::new()),
            name_to_id: RwLock::new(HashMap::new()),
            dependency_resolver: Arc::new(RwLock::new(DependencyResolver::new())),
            state_manager: Arc::new(MemoryStateManager::new()) as Arc<dyn PluginStateManager>,
            discovery: Arc::new(DefaultPluginDiscovery::new()),
            metrics: Arc::new(RwLock::new(PluginManagerMetrics::new())),
        }
    }

    /// Get plugin manager status
    pub async fn get_status(&self) -> PluginManagerStatus {
        let plugins = self.plugins.read().await;
        let statuses = self.statuses.read().await;

        let total = plugins.len();
        let active = statuses
            .values()
            .filter(|s| **s == PluginStatus::Running)
            .count();
        let failed = statuses
            .values()
            .filter(|s| **s == PluginStatus::Failed)
            .count();

        PluginManagerStatus::new(total, active, failed)
    }

    /// Get performance metrics
    pub async fn get_metrics(&self) -> PluginManagerMetrics {
        let metrics = self.metrics.read().await;
        PluginManagerMetrics {
            load_time_ms: metrics.load_time_ms,
            memory_usage_kb: metrics.memory_usage_kb,
        }
    }
}

impl fmt::Debug for DefaultPluginManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DefaultPluginManager")
            .field("plugins", &"<plugin map>")
            .field("name_to_id", &"<name mapping>")
            .field("dependency_resolver", &"<dependency resolver>")
            .field("state_manager", &"<state manager>")
            .field("discovery", &"<discovery service>")
            .field("metrics", &"<metrics>")
            .finish()
    }
}

impl Default for DefaultPluginManager {
    fn default() -> Self {
        Self::new()
    }
}

// Implementation of PluginRegistry and PluginManagerTrait would go here
// (This is placeholder - the actual implementation would be extracted from manager.rs)
