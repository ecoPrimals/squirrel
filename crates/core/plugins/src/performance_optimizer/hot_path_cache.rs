// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Hot path cache for frequently used plugin operations.

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;
use tracing::info;

use super::config::HotPathCacheConfig;
use super::types::{CacheStatistics, CachedCapabilityQuery, CachedPluginLookup};

/// Hot path cache for frequently used operations
#[derive(Debug)]
pub struct HotPathCache {
    /// Cached plugin lookups
    plugin_lookups: Arc<RwLock<HashMap<String, CachedPluginLookup>>>,

    /// Cached capability queries
    capability_queries: Arc<RwLock<HashMap<String, CachedCapabilityQuery>>>,

    /// Cached execution results
    execution_results: Arc<RwLock<HashMap<String, super::types::CachedExecutionResult>>>,

    /// Cache statistics
    stats: Arc<RwLock<CacheStatistics>>,

    /// Configuration for cache behavior
    config: HotPathCacheConfig,
}

impl HotPathCache {
    pub(super) fn new(config: HotPathCacheConfig) -> Self {
        Self {
            plugin_lookups: Arc::new(RwLock::new(HashMap::new())),
            capability_queries: Arc::new(RwLock::new(HashMap::new())),
            execution_results: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(CacheStatistics::default())),
            config,
        }
    }

    pub(super) async fn get_plugin_lookup(&self, cache_key: &str) -> Option<CachedPluginLookup> {
        let lookups = self.plugin_lookups.read().await;
        lookups.get(cache_key).cloned()
    }

    pub(super) async fn cache_plugin_lookup(
        &self,
        cache_key: String,
        cached_lookup: CachedPluginLookup,
    ) {
        let mut lookups = self.plugin_lookups.write().await;
        lookups.insert(cache_key, cached_lookup);
    }

    pub(super) async fn get_capability_query(
        &self,
        cache_key: &str,
    ) -> Option<CachedCapabilityQuery> {
        let queries = self.capability_queries.read().await;
        queries.get(cache_key).cloned()
    }

    pub(super) async fn cache_capability_query(
        &self,
        cache_key: String,
        cached_query: CachedCapabilityQuery,
    ) {
        let mut queries = self.capability_queries.write().await;
        queries.insert(cache_key, cached_query);
    }

    pub(super) async fn get_statistics(&self) -> CacheStatistics {
        self.stats.read().await.clone()
    }

    pub(super) async fn start_cache_warming(&self) {
        info!("Starting hot path cache warming");
        // Implementation would pre-populate cache with frequently used items
    }
}
