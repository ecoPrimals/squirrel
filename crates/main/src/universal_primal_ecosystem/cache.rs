// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Capability discovery cache: keys, eviction, and cached lookups.

use std::collections::HashMap;
use std::time::Instant;

use tracing::debug;

use crate::universal::UniversalResult;

use super::UniversalPrimalEcosystem;
use super::types::{CachedCapabilityMatch, CapabilityMatch, CapabilityRequest};

impl UniversalPrimalEcosystem {
    /// Find services by capability with intelligent caching
    pub async fn find_services_by_capability(
        &self,
        request: &CapabilityRequest,
    ) -> UniversalResult<Vec<CapabilityMatch>> {
        if !self.cache_config.enable_caching {
            return self.find_services_by_capability_uncached(request).await;
        }

        // Generate cache key from request
        let cache_key = self.generate_cache_key(request);

        // Check cache first
        {
            let mut cache = self.discovery_cache.write().await;
            if let Some(cached_entry) = cache.get_mut(&cache_key) {
                if cached_entry.is_valid() {
                    cached_entry.accessed();
                    debug!("Cache hit for capability discovery: {}", cache_key);
                    return Ok(cached_entry.matches.clone());
                }
                // Remove expired entry
                cache.remove(&cache_key);
                debug!("Cache expired for capability discovery: {}", cache_key);
            }
        }

        debug!("Cache miss for capability discovery: {}", cache_key);

        // Perform actual discovery
        let start_time = Instant::now();
        let matches = self.find_services_by_capability_uncached(request).await?;
        let discovery_time = start_time.elapsed();

        debug!(
            "Capability discovery completed in {:?} for: {}",
            discovery_time, cache_key
        );

        // Cache the results
        if !matches.is_empty() {
            let cached_entry = CachedCapabilityMatch {
                matches: matches.clone(),
                cached_at: Instant::now(),
                ttl_seconds: self.cache_config.capability_discovery_ttl,
                access_count: 1,
            };

            let mut cache = self.discovery_cache.write().await;

            // Implement cache eviction if at max capacity
            if cache.len() >= self.cache_config.max_cache_entries {
                self.evict_oldest_cache_entries(&mut cache).await;
            }

            cache.insert(cache_key, cached_entry);
        }

        Ok(matches)
    }

    /// Generate cache key from capability request
    pub(crate) fn generate_cache_key(&self, request: &CapabilityRequest) -> String {
        let mut key_parts = vec![];

        // Include required capabilities
        let mut required = request.required_capabilities.clone();
        required.sort();
        key_parts.push(format!("req:{}", required.join(",")));

        // Include optional capabilities
        let mut optional = request.optional_capabilities.clone();
        optional.sort();
        if !optional.is_empty() {
            key_parts.push(format!("opt:{}", optional.join(",")));
        }

        // Include context for context-aware caching
        key_parts.push(format!(
            "ctx:{}:{}",
            request.context.user_id, request.context.security_level
        ));

        key_parts.join("|")
    }

    /// Evict oldest cache entries to make room for new ones
    async fn evict_oldest_cache_entries(&self, cache: &mut HashMap<String, CachedCapabilityMatch>) {
        let evict_count = cache.len() / 10; // Evict 10% of entries

        // Find oldest entries by creation time
        let mut entries: Vec<_> = cache
            .iter()
            .map(|(k, v)| (k.clone(), v.cached_at))
            .collect();
        entries.sort_by_key(|(_, time)| *time);

        // Remove oldest entries
        for (key, _) in entries.into_iter().take(evict_count) {
            cache.remove(&key);
        }

        debug!("Evicted {} old cache entries", evict_count);
    }
}
