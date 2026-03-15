// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab
#![allow(deprecated)]

//! Runtime discovery engine
//!
//! Implements the multi-stage discovery process for finding services at runtime.

use crate::discovery::types::{DiscoveredService, DiscoveryError, DiscoveryResult};
use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, info};

/// Runtime service discovery engine
///
/// Discovers services by capability at runtime with zero hardcoded knowledge.
pub struct RuntimeDiscoveryEngine {
    /// Cache of discovered services
    cache: Arc<DashMap<String, DiscoveredService>>,

    /// Cache TTL
    cache_ttl: Duration,
}

impl RuntimeDiscoveryEngine {
    /// Create new discovery engine
    #[must_use]
    pub fn new() -> Self {
        Self {
            cache: Arc::new(DashMap::new()),
            cache_ttl: Duration::from_secs(300), // 5 minutes
        }
    }

    /// Discover service by capability (NO hardcoding)
    ///
    /// # Discovery Strategy
    ///
    /// 1. Check environment variable first (e.g., `COMPUTE_ENDPOINT`)
    /// 2. Try mDNS/DNS-SD for local network discovery
    /// 3. Query central registry if configured
    /// 4. Listen for capability announcements
    ///
    /// # Errors
    ///
    /// Returns error if capability cannot be discovered
    pub async fn discover_capability(
        &self,
        capability: &str,
    ) -> DiscoveryResult<DiscoveredService> {
        debug!("🔍 Discovering capability: {}", capability);

        // Check cache first
        if let Some(service_entry) = self.cache.get(capability) {
            let service = service_entry.value();
            if service.is_fresh(self.cache_ttl) {
                debug!("✓ Found cached service for '{}'", capability);
                return Ok(service.clone());
            }
        }

        // Stage 1: Environment variable
        // Convert capability name to env var format: "ai.inference" -> "AI_INFERENCE_ENDPOINT"
        let env_key = format!("{}_ENDPOINT", capability.to_uppercase().replace('.', "_"));
        if let Ok(endpoint) = std::env::var(&env_key) {
            info!(
                "✓ Discovered '{}' via environment variable {}: {}",
                capability, env_key, endpoint
            );

            let service = DiscoveredService {
                name: format!("{capability}-provider"),
                endpoint,
                capabilities: vec![capability.into()], // ZERO-COPY: Avoid allocating capability string
                metadata: HashMap::new(),
                discovered_at: std::time::SystemTime::now(),
                discovery_method: "environment_variable".into(), // ZERO-COPY: Static string
                healthy: None,
                priority: 100,
            };

            // Cache it
            self.cache.insert(capability.into(), service.clone());

            return Ok(service);
        }

        // Stage 2: Try mDNS (local network)
        debug!("Stage 2: Trying mDNS for '{}'", capability);
        let mdns = crate::discovery::mechanisms::MdnsDiscovery::default();
        if let Ok(services) = mdns.discover_by_capability(capability).await {
            if let Some(service) = services.into_iter().next() {
                info!("✅ Found via mDNS: {}", service.endpoint);
                return Ok(service);
            }
        }

        // Stage 3: Try DNS-SD (network-wide)
        debug!("Stage 3: Trying DNS-SD for '{}'", capability);
        let dnssd = crate::discovery::mechanisms::DnssdDiscovery::default();
        if let Ok(services) = dnssd.discover_by_capability(capability).await {
            if let Some(service) = services.into_iter().next() {
                info!("✅ Found via DNS-SD: {}", service.endpoint);
                return Ok(service);
            }
        }

        // Stage 4: Try service registry (if configured)
        if let Ok(registry_endpoint) = std::env::var("SERVICE_REGISTRY_ENDPOINT") {
            debug!("Stage 4: Trying service registry for '{}'", capability);
            let registry_type =
                std::env::var("SERVICE_REGISTRY_TYPE").unwrap_or_else(|_| "consul".to_string());

            let reg_type = match registry_type.to_lowercase().as_str() {
                "consul" => crate::discovery::mechanisms::RegistryType::Consul,
                "etcd" => crate::discovery::mechanisms::RegistryType::Etcd,
                "kubernetes" | "k8s" => crate::discovery::mechanisms::RegistryType::Kubernetes,
                "eureka" => crate::discovery::mechanisms::RegistryType::Eureka,
                _ => crate::discovery::mechanisms::RegistryType::Custom,
            };

            let registry =
                crate::discovery::mechanisms::RegistryDiscovery::new(reg_type, registry_endpoint);
            if let Ok(services) = registry.discover_by_capability(capability).await {
                if let Some(service) = services.into_iter().next() {
                    info!("✅ Found via service registry: {}", service.endpoint);
                    return Ok(service);
                }
            }
        }

        // Stage 5: P2P multicast (future)
        // Mesh networking for peer discovery

        Err(DiscoveryError::CapabilityNotFound {
            capability: capability.to_string(),
        })
    }

    /// Clear discovery cache
    pub async fn clear_cache(&self) {
        self.cache.clear();
        debug!("🗑️  Discovery cache cleared");
    }
}

impl Default for RuntimeDiscoveryEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_discover_from_env() {
        std::env::set_var("COMPUTE_ENDPOINT", "http://localhost:8500");

        let engine = RuntimeDiscoveryEngine::new();
        let service = engine.discover_capability("compute").await.unwrap();

        assert_eq!(service.endpoint, "http://localhost:8500");
        assert!(service.has_capability("compute"));

        std::env::remove_var("COMPUTE_ENDPOINT");
    }

    #[tokio::test]
    async fn test_capability_not_found() {
        let engine = RuntimeDiscoveryEngine::new();
        let result = engine.discover_capability("nonexistent").await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_cache() {
        std::env::set_var("CACHE_TEST_ENDPOINT", "http://original:8080");

        let engine = RuntimeDiscoveryEngine::new();

        // First discovery
        let service1 = engine.discover_capability("cache_test").await.unwrap();

        // Change environment
        std::env::set_var("CACHE_TEST_ENDPOINT", "http://changed:8080");

        // Should return cached value
        let service2 = engine.discover_capability("cache_test").await.unwrap();
        assert_eq!(service1.endpoint, service2.endpoint); // Cached!

        // Clear cache
        engine.clear_cache().await;

        // Should discover new value
        let service3 = engine.discover_capability("cache_test").await.unwrap();
        assert_eq!(service3.endpoint, "http://changed:8080");

        std::env::remove_var("CACHE_TEST_ENDPOINT");
    }
}
