// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors
#![allow(deprecated)]

//! Primal Self-Knowledge - Each primal knows ONLY itself
//!
//! Following Songbird's sovereignty principle: primals discover their OWN identity
//! at runtime, with ZERO hardcoded knowledge of other primals.

use crate::discovery::types::{DiscoveredService, DiscoveryError, DiscoveryResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Identity of this primal (self-knowledge)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalIdentity {
    /// Name of this primal (discovered from environment or generated)
    pub name: String,

    /// Primal type (e.g., "squirrel", "songbird", "beardog")
    pub primal_type: String,

    /// Capabilities this primal provides
    pub capabilities: Vec<String>,

    /// Version
    pub version: String,

    /// Instance ID (unique per process)
    pub instance_id: String,

    /// Host information
    pub host: String,

    /// Port (if serving)
    pub port: Option<u16>,
}

/// Primal Self-Knowledge - Core of zero-hardcoding architecture
///
/// Each primal uses this to:
/// 1. Discover its own identity
/// 2. Announce its capabilities
/// 3. Discover other primals at runtime (NO hardcoding!)
pub struct PrimalSelfKnowledge {
    /// This primal's identity
    identity: PrimalIdentity,

    /// Discovered other primals (capability → service)
    discovered: Arc<RwLock<HashMap<String, DiscoveredService>>>,
}

impl PrimalSelfKnowledge {
    /// Discover this primal's identity (zero hardcoding!)
    ///
    /// Identity is discovered from:
    /// 1. Environment variables (PRIMAL_NAME, PRIMAL_CAPABILITIES, etc.)
    /// 2. Process info (hostname, PID)
    /// 3. Runtime introspection
    ///
    /// # Errors
    ///
    /// Returns error if identity cannot be determined
    pub fn discover_self() -> DiscoveryResult<Self> {
        info!("🔍 Discovering primal self-knowledge...");

        // Discover primal type (from ENV or default)
        // ZERO-COPY: Use static str instead of allocating "squirrel" every time
        let primal_type = env::var("PRIMAL_TYPE").unwrap_or_else(|_| "squirrel".into());

        // Discover name (from ENV or default)
        let name = env::var("PRIMAL_NAME").unwrap_or_else(|_| {
            // Default: Use binary name
            env::current_exe()
                .ok()
                .and_then(|p| p.file_stem().map(|s| s.to_string_lossy().to_string()))
                .unwrap_or_else(|| primal_type.clone())
        });

        // Discover capabilities (from ENV)
        //
        // Capability namespace follows the {domain}.{operation} convention
        // used by the discovery system (probe_socket, discover_capability, etc.)
        // This ensures Squirrel is discoverable by other primals scanning sockets.
        let capabilities = env::var("PRIMAL_CAPABILITIES")
            .ok()
            .map(|s| s.split(',').map(|c| c.trim().to_string()).collect())
            .unwrap_or_else(|| {
                // Default Squirrel capabilities
                // Uses standard {domain}.{operation} namespace for discovery alignment
                vec![
                    "ai.query".into(),
                    "ai.complete".into(),
                    "ai.chat".into(),
                    "ai.inference".into(),
                    "ai.list_providers".into(),
                    "tool.execute".into(),
                    "system.health".into(),
                    "capability.discover".into(),
                ]
            });

        // Version
        let version = env!("CARGO_PKG_VERSION").to_string();

        // Instance ID (unique per process)
        let instance_id = format!("{}-{}", name, std::process::id());

        // Host
        let host = hostname::get()
            .ok()
            .and_then(|h| h.to_str().map(String::from))
            .unwrap_or_else(|| "localhost".to_string());

        // Port (if serving)
        let port = env::var("PRIMAL_PORT").ok().and_then(|p| p.parse().ok());

        let identity = PrimalIdentity {
            name: name.clone(),
            primal_type,
            capabilities: capabilities.clone(),
            version,
            instance_id,
            host,
            port,
        };

        debug!("✓ Discovered self: {}", identity.name);
        debug!("  Capabilities: {:?}", identity.capabilities);
        debug!("  Instance ID: {}", identity.instance_id);

        Ok(Self {
            identity,
            discovered: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Get this primal's identity
    #[must_use]
    pub fn identity(&self) -> &PrimalIdentity {
        &self.identity
    }

    /// Discover another primal by capability (runtime discovery!)
    ///
    /// This is the key method that replaces hardcoded primal knowledge.
    ///
    /// # Errors
    ///
    /// Returns error if capability cannot be discovered
    pub async fn discover_primal(&self, capability: &str) -> DiscoveryResult<DiscoveredService> {
        debug!("🔍 Discovering primal for capability: {}", capability);

        // Check cache first
        if let Some(service) = self.discovered.read().await.get(capability) {
            debug!("✓ Found cached provider for '{}'", capability);
            return Ok(service.clone());
        }

        // Stage 1: Environment variable (highest priority)
        if let Ok(endpoint) = env::var(format!("{}_ENDPOINT", capability.to_uppercase())) {
            info!(
                "✓ Discovered '{}' via environment variable: {}",
                capability, endpoint
            );

            let service = DiscoveredService {
                name: format!("{capability}-provider"),
                endpoint,
                capabilities: vec![capability.to_string()],
                metadata: HashMap::new(),
                discovered_at: std::time::SystemTime::now(),
                discovery_method: "environment_variable".to_string(),
                healthy: None,
                priority: 100, // ENV vars have highest priority
            };

            // Cache it
            self.discovered
                .write()
                .await
                .insert(capability.to_string(), service.clone());

            return Ok(service);
        }

        // Stage 2: Try mDNS (local network discovery)
        debug!("Stage 2: Trying mDNS discovery for '{}'", capability);
        let mdns = crate::discovery::mechanisms::MdnsDiscovery::default();
        if let Ok(services) = mdns.discover_by_capability(capability).await {
            if let Some(service) = services.into_iter().next() {
                info!("✅ Found '{}' via mDNS: {}", capability, service.endpoint);
                self.discovered
                    .write()
                    .await
                    .insert(capability.to_string(), service.clone());
                return Ok(service);
            }
        }

        // Stage 3: Try DNS-SD (network-wide discovery)
        debug!("Stage 3: Trying DNS-SD discovery for '{}'", capability);
        let dnssd = crate::discovery::mechanisms::DnssdDiscovery::default();
        if let Ok(services) = dnssd.discover_by_capability(capability).await {
            if let Some(service) = services.into_iter().next() {
                info!("✅ Found '{}' via DNS-SD: {}", capability, service.endpoint);
                self.discovered
                    .write()
                    .await
                    .insert(capability.to_string(), service.clone());
                return Ok(service);
            }
        }

        // Stage 4: Try service registry (if configured)
        if let Ok(registry_endpoint) = std::env::var("SERVICE_REGISTRY_ENDPOINT") {
            debug!(
                "Stage 4: Trying service registry discovery for '{}'",
                capability
            );
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
                    info!(
                        "✅ Found '{}' via service registry: {}",
                        capability, service.endpoint
                    );
                    self.discovered
                        .write()
                        .await
                        .insert(capability.to_string(), service.clone());
                    return Ok(service);
                }
            }
        }

        // Stage 5: P2P multicast (future implementation)
        // Would provide mesh networking and peer discovery
        // Priority: 40 (lowest, but highly resilient)

        Err(DiscoveryError::CapabilityNotFound {
            capability: capability.to_string(),
        })
    }

    /// Get all discovered primals
    pub async fn discovered(&self) -> HashMap<String, DiscoveredService> {
        self.discovered.read().await.clone()
    }

    /// Clear discovery cache (force re-discovery)
    pub async fn clear_cache(&self) {
        self.discovered.write().await.clear();
        debug!("🗑️  Discovery cache cleared");
    }

    /// Announce this primal's capabilities to the network
    ///
    /// Uses multiple announcement mechanisms for maximum discoverability:
    /// 1. mDNS for local network discovery
    /// 2. DNS-SD for DNS-based discovery
    /// 3. Service registry (if configured)
    ///
    /// # Arguments
    ///
    /// * `port` - Service port (defaults to 9200 if not specified)
    pub async fn announce_capabilities(&self) -> DiscoveryResult<()> {
        self.announce_capabilities_with_port(9200).await
    }

    /// Announce capabilities with specific port
    pub async fn announce_capabilities_with_port(&self, port: u16) -> DiscoveryResult<()> {
        info!(
            "📢 Announcing capabilities: {:?} on port {}",
            self.identity.capabilities, port
        );

        let service_name = format!(
            "{}-{}",
            self.identity.primal_type, self.identity.instance_id
        );
        let capabilities = self.identity.capabilities.clone();
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("version".to_string(), self.identity.version.clone());
        metadata.insert("primal_type".to_string(), self.identity.primal_type.clone());
        metadata.insert("instance_id".to_string(), self.identity.instance_id.clone());

        // 1. Announce via mDNS (local network, zero-config)
        let mdns = crate::discovery::mechanisms::MdnsDiscovery::default();
        if let Err(e) = mdns
            .announce_service(&service_name, port, capabilities.clone(), metadata.clone())
            .await
        {
            warn!("Failed to announce via mDNS: {}", e);
        } else {
            debug!("✅ Announced via mDNS");
        }

        // 2. Announce via DNS-SD (network-wide)
        let dnssd = crate::discovery::mechanisms::DnssdDiscovery::default();
        let hostname = format!("{service_name}.local");
        if let Err(e) = dnssd
            .register_service(
                &service_name,
                &hostname,
                port,
                capabilities.clone(),
                metadata.clone(),
            )
            .await
        {
            warn!("Failed to register in DNS-SD: {}", e);
        } else {
            debug!("✅ Registered in DNS-SD");
        }

        // 3. Announce via service registry (if configured via env var)
        if let Ok(registry_endpoint) = std::env::var("SERVICE_REGISTRY_ENDPOINT") {
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

            // Get local IP address (simplified - use first non-loopback)
            let address =
                std::env::var("SERVICE_ADDRESS").unwrap_or_else(|_| "127.0.0.1".to_string());

            let health_endpoint = Some("/health".to_string());

            if let Err(e) = registry
                .register_service(
                    &service_name,
                    &self.identity.primal_type,
                    &address,
                    port,
                    capabilities,
                    health_endpoint,
                    metadata,
                )
                .await
            {
                warn!("Failed to register in service registry: {}", e);
            } else {
                debug!("✅ Registered in service registry");
            }
        }

        info!("✅ Capability announcement complete");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discover_self() {
        let self_knowledge = PrimalSelfKnowledge::discover_self().unwrap();
        let identity = self_knowledge.identity();

        // Should have discovered a name
        assert!(!identity.name.is_empty());

        // Should have some capabilities
        assert!(!identity.capabilities.is_empty());

        // Should have an instance ID
        assert!(!identity.instance_id.is_empty());
    }

    #[tokio::test]
    async fn test_discover_primal_from_env() {
        // Set up environment
        env::set_var("AI_ENDPOINT", "http://discovered.example.com:9200");

        let self_knowledge = PrimalSelfKnowledge::discover_self().unwrap();

        // Discover AI capability
        let service = self_knowledge.discover_primal("ai").await.unwrap();

        assert_eq!(service.endpoint, "http://discovered.example.com:9200");
        assert!(service.has_capability("ai"));
        assert_eq!(service.priority, 100); // ENV vars have max priority

        // Clean up
        env::remove_var("AI_ENDPOINT");
    }

    #[tokio::test]
    async fn test_capability_not_found() {
        let self_knowledge = PrimalSelfKnowledge::discover_self().unwrap();

        // Try to discover non-existent capability
        let result = self_knowledge.discover_primal("nonexistent").await;

        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(DiscoveryError::CapabilityNotFound { .. })
        ));
    }

    #[tokio::test]
    async fn test_discovery_cache() {
        env::set_var("STORAGE_ENDPOINT", "http://cache-test.example.com:8500");

        let self_knowledge = PrimalSelfKnowledge::discover_self().unwrap();

        // First discovery
        let service1 = self_knowledge.discover_primal("storage").await.unwrap();

        // Change the environment (simulating external change)
        env::set_var("STORAGE_ENDPOINT", "http://changed.example.com:8500");

        // Second discovery should return cached value
        let service2 = self_knowledge.discover_primal("storage").await.unwrap();
        assert_eq!(service1.endpoint, service2.endpoint); // Cached!

        // Clear cache and discover again
        self_knowledge.clear_cache().await;
        let service3 = self_knowledge.discover_primal("storage").await.unwrap();
        assert_eq!(service3.endpoint, "http://changed.example.com:8500"); // Updated!

        // Clean up
        env::remove_var("STORAGE_ENDPOINT");
    }
}
