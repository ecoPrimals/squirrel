// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors
#![allow(deprecated)]

//! Capability resolver - Core discovery engine
//!
//! This module implements the central capability resolution engine that
//! replaces all hardcoded service references with dynamic discovery.

use crate::discovery::mechanisms::{
    DnssdDiscovery, MdnsDiscovery, RegistryDiscovery, RegistryType,
};
use crate::discovery::types::{
    CapabilityRequest, DiscoveredService, DiscoveryError, DiscoveryResult,
};
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Discovery mechanism priority and metadata
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiscoveryMethod {
    /// Discover via environment variables.
    EnvironmentVariable,
    /// Discover via the service registry.
    ServiceRegistry,
    /// Discover via mDNS on the local network.
    MDns,
    /// Discover via DNS-SD (DNS Service Discovery).
    DnsSd,
    /// Discover via P2P multicast.
    P2PMulticast,
}

/// Capability resolver - discovers providers by capability
#[derive(Debug, Clone)]
pub struct CapabilityResolver {
    /// mDNS discovery client (local network)
    mdns: Arc<MdnsDiscovery>,

    /// DNS-SD discovery client (network-wide)
    dnssd: Arc<DnssdDiscovery>,

    /// Service registry client (optional, if configured)
    registry: Option<Arc<RegistryDiscovery>>,
}

impl CapabilityResolver {
    /// Create new capability resolver with default discovery mechanisms
    #[must_use]
    pub fn new() -> Self {
        Self {
            mdns: Arc::new(MdnsDiscovery::default()),
            dnssd: Arc::new(DnssdDiscovery::default()),
            registry: None,
        }
    }

    /// Create new capability resolver with service registry
    pub fn with_registry(registry_type: RegistryType, endpoint: String) -> Self {
        Self {
            mdns: Arc::new(MdnsDiscovery::default()),
            dnssd: Arc::new(DnssdDiscovery::default()),
            registry: Some(Arc::new(RegistryDiscovery::new(registry_type, endpoint))),
        }
    }

    /// Discover provider for a capability
    ///
    /// # Errors
    ///
    /// Returns error if capability cannot be discovered
    pub async fn discover_provider(
        &self,
        request: CapabilityRequest,
    ) -> DiscoveryResult<DiscoveredService> {
        debug!(
            "Discovering provider for capability: {}",
            request.capability
        );

        // Multi-stage discovery with priority:
        // 1. Try environment variables (fastest, highest priority)
        if let Ok(service) = self.discover_from_env(&request.capability).await {
            info!("✅ Found via environment variable (priority 100)");
            return Ok(service);
        }

        // 2. Try service registry (if configured)
        if let Some(registry) = &self.registry {
            debug!("Trying service registry discovery...");
            if let Ok(services) = registry.discover_by_capability(&request.capability).await
                && let Some(service) = services.into_iter().next()
            {
                info!("✅ Found via service registry (priority 60)");
                return Ok(service);
            }
        }

        // 3. Try mDNS for local network
        debug!("Trying mDNS discovery...");
        if let Ok(services) = self.mdns.discover_by_capability(&request.capability).await
            && let Some(service) = services.into_iter().next()
        {
            info!("✅ Found via mDNS (priority 80)");
            return Ok(service);
        }

        // 4. Try DNS-SD for network-wide discovery
        debug!("Trying DNS-SD discovery...");
        if let Ok(services) = self.dnssd.discover_by_capability(&request.capability).await
            && let Some(service) = services.into_iter().next()
        {
            info!("✅ Found via DNS-SD (priority 70)");
            return Ok(service);
        }

        // 5. P2P multicast (future implementation)
        // Would be lowest priority (40) but highly resilient

        warn!(
            "❌ No provider found for capability: {} (tried all discovery mechanisms)",
            request.capability
        );
        Err(DiscoveryError::CapabilityNotFound {
            capability: request.capability.clone(),
        })
    }

    /// Discover service from environment variables
    async fn discover_from_env(&self, capability: &str) -> DiscoveryResult<DiscoveredService> {
        let env_key = format!("{}_ENDPOINT", capability.to_uppercase().replace('.', "_"));

        if let Ok(endpoint) = std::env::var(&env_key) {
            info!(
                "✅ Discovered '{}' from {}: {}",
                capability, env_key, endpoint
            );
            return Ok(DiscoveredService {
                name: format!("{capability}-provider"),
                endpoint,
                capabilities: vec![capability.to_string()],
                metadata: std::collections::HashMap::new(),
                discovered_at: std::time::SystemTime::now(),
                discovery_method: "environment_variable".to_string(),
                healthy: Some(true),
                priority: 100, // Highest priority for explicit env vars
            });
        }

        Err(DiscoveryError::CapabilityNotFound {
            capability: capability.to_string(),
        })
    }
}

impl Default for CapabilityResolver {
    fn default() -> Self {
        Self::new()
    }
}
