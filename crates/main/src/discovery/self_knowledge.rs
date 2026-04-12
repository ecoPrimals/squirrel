// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors
#![expect(deprecated, reason = "Backward compatibility during migration")]

//! Primal Self-Knowledge - Each primal knows ONLY itself
//!
//! Sovereignty principle: primals discover their own identity at runtime,
//! with zero hardcoded knowledge of other primals.

use crate::discovery::types::{DiscoveredService, DiscoveryError, DiscoveryResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

fn serialize_arc_str_vec<S>(v: &[Arc<str>], s: S) -> std::result::Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    use serde::Serialize;
    v.iter()
        .map(AsRef::as_ref)
        .collect::<Vec<&str>>()
        .serialize(s)
}

fn deserialize_arc_str_vec<'de, D>(d: D) -> std::result::Result<Vec<Arc<str>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let v: Vec<String> = Vec::deserialize(d)?;
    Ok(v.into_iter().map(Arc::from).collect())
}

/// Store a discovered service and return an owned copy without cloning on insert.
fn insert_discovered_and_return(
    cache: &mut HashMap<String, DiscoveredService>,
    capability: &str,
    service: DiscoveredService,
) -> DiscoveryResult<DiscoveredService> {
    cache.insert(capability.to_string(), service);
    cache.get(capability).cloned().ok_or_else(|| {
        DiscoveryError::ParseError("discovery cache missing entry after insert".into())
    })
}

/// Identity of this primal (self-knowledge)
/// Uses `Arc<str>` for capabilities to avoid cloning on hot paths.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalIdentity {
    /// Name of this primal (discovered from environment or generated)
    pub name: String,

    /// Primal type (e.g., "squirrel", "songbird", "beardog")
    pub primal_type: String,

    /// Capabilities this primal provides (Arc for O(1) clone)
    #[serde(
        serialize_with = "serialize_arc_str_vec",
        deserialize_with = "deserialize_arc_str_vec"
    )]
    pub capabilities: Vec<Arc<str>>,

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
        let primal_type =
            env::var("PRIMAL_TYPE").unwrap_or_else(|_| crate::niche::PRIMAL_ID.into());

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
        // Default Squirrel capabilities — static to avoid allocation
        const DEFAULT_CAPABILITIES: &[&str] = &[
            "ai.query",
            "ai.complete",
            "ai.chat",
            "ai.inference",
            "ai.list_providers",
            "tool.execute",
            "system.health",
            "capability.discover",
        ];

        let capabilities = env::var("PRIMAL_CAPABILITIES").ok().map_or_else(
            || {
                DEFAULT_CAPABILITIES
                    .iter()
                    .map(|s| Arc::from(*s))
                    .collect::<Vec<Arc<str>>>()
            },
            |s| s.split(',').map(|c| Arc::from(c.trim())).collect(),
        );

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
            name,
            primal_type,
            capabilities,
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
    pub const fn identity(&self) -> &PrimalIdentity {
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

            let mut cache = self.discovered.write().await;
            return insert_discovered_and_return(&mut cache, capability, service);
        }

        // Stage 2: Try socket registry (biomeOS - primary primal discovery)
        debug!("Stage 2: Trying socket registry for '{}'", capability);
        if let Ok(services) =
            crate::discovery::mechanisms::discover_from_socket_registry(capability)
            && let Some(service) = services.into_iter().next()
        {
            info!(
                "✅ Found '{}' via socket registry: {}",
                capability, service.endpoint
            );
            let mut cache = self.discovered.write().await;
            return insert_discovered_and_return(&mut cache, capability, service);
        }

        // Stage 3: Try mDNS (local network; falls back to socket registry)
        debug!("Stage 3: Trying mDNS discovery for '{}'", capability);
        let mdns = crate::discovery::mechanisms::MdnsDiscovery::default();
        if let Ok(services) = mdns.discover_by_capability(capability).await
            && let Some(service) = services.into_iter().next()
        {
            info!("✅ Found '{}' via mDNS: {}", capability, service.endpoint);
            let mut cache = self.discovered.write().await;
            return insert_discovered_and_return(&mut cache, capability, service);
        }

        // Stage 4: Try DNS-SD (network-wide; falls back to socket registry)
        debug!("Stage 4: Trying DNS-SD discovery for '{}'", capability);
        let dnssd = crate::discovery::mechanisms::DnssdDiscovery::default();
        if let Ok(services) = dnssd.discover_by_capability(capability).await
            && let Some(service) = services.into_iter().next()
        {
            info!("✅ Found '{}' via DNS-SD: {}", capability, service.endpoint);
            let mut cache = self.discovered.write().await;
            return insert_discovered_and_return(&mut cache, capability, service);
        }

        // Stage 5: Try external service registry (if configured)
        if let Ok(registry_endpoint) = std::env::var("SERVICE_REGISTRY_ENDPOINT") {
            debug!(
                "Stage 5: Trying service registry discovery for '{}'",
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
            if let Ok(services) = registry.discover_by_capability(capability).await
                && let Some(service) = services.into_iter().next()
            {
                info!(
                    "✅ Found '{}' via service registry: {}",
                    capability, service.endpoint
                );
                let mut cache = self.discovered.write().await;
                return insert_discovered_and_return(&mut cache, capability, service);
            }
        }

        // Stage 6: P2P multicast (future implementation)
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
        // Convert Arc<str> to Vec<String> once for discovery APIs
        let capabilities: Vec<String> = self
            .identity
            .capabilities
            .iter()
            .map(ToString::to_string)
            .collect();
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
            let address = std::env::var("SERVICE_ADDRESS")
                .unwrap_or_else(|_| universal_constants::network::LOCALHOST_IPV4.to_string());

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
#[expect(
    clippy::expect_used,
    reason = "Invariant or startup failure: expect after validation"
)]
mod tests {
    use super::*;
    use crate::niche;

    #[test]
    fn test_discover_self() {
        let self_knowledge = PrimalSelfKnowledge::discover_self().expect("discover_self in test");
        let identity = self_knowledge.identity();

        // Should have discovered a name
        assert!(!identity.name.is_empty());

        // Should have some capabilities
        assert!(!identity.capabilities.is_empty());

        // Should have an instance ID
        assert!(!identity.instance_id.is_empty());
    }

    #[test]
    fn test_discover_primal_from_env() {
        temp_env::with_var(
            "AI_ENDPOINT",
            Some("http://discovered.example.com:9200"),
            || {
                let rt = tokio::runtime::Runtime::new().expect("tokio runtime for test");
                rt.block_on(async {
                    let self_knowledge =
                        PrimalSelfKnowledge::discover_self().expect("discover_self in test");

                    let service = self_knowledge
                        .discover_primal("ai")
                        .await
                        .expect("discover_primal ai");

                    assert_eq!(service.endpoint, "http://discovered.example.com:9200");
                    assert!(service.has_capability("ai"));
                    assert_eq!(service.priority, 100);
                });
            },
        );
    }

    #[tokio::test]
    async fn test_capability_not_found() {
        let self_knowledge = PrimalSelfKnowledge::discover_self().expect("discover_self in test");

        // Try to discover non-existent capability
        let result = self_knowledge.discover_primal("nonexistent").await;

        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(DiscoveryError::CapabilityNotFound { .. })
        ));
    }

    #[test]
    fn test_discovery_cache() {
        temp_env::with_var(
            "STORAGE_ENDPOINT",
            Some("http://cache-test.example.com:8500"),
            || {
                let rt = tokio::runtime::Runtime::new().expect("tokio runtime for test");
                let (self_knowledge, service1) = rt.block_on(async {
                    let self_knowledge =
                        PrimalSelfKnowledge::discover_self().expect("discover_self in test");
                    let service1 = self_knowledge
                        .discover_primal("storage")
                        .await
                        .expect("discover_primal storage");
                    (self_knowledge, service1)
                });

                temp_env::with_var(
                    "STORAGE_ENDPOINT",
                    Some("http://changed.example.com:8500"),
                    || {
                        rt.block_on(async {
                            let service2 = self_knowledge
                                .discover_primal("storage")
                                .await
                                .expect("discover_primal storage");
                            assert_eq!(service1.endpoint, service2.endpoint);

                            self_knowledge.clear_cache().await;
                            let service3 = self_knowledge
                                .discover_primal("storage")
                                .await
                                .expect("discover_primal storage");
                            assert_eq!(service3.endpoint, "http://changed.example.com:8500");
                        });
                    },
                );
            },
        );
    }

    #[test]
    fn test_primal_identity_construction() {
        temp_env::with_vars(
            [
                ("PRIMAL_NAME", Some("test-squirrel")),
                ("PRIMAL_TYPE", Some("squirrel")),
                ("PRIMAL_CAPABILITIES", Some("ai.query,system.health")),
                ("PRIMAL_PORT", Some("9200")),
            ],
            || {
                let self_knowledge =
                    PrimalSelfKnowledge::discover_self().expect("discover_self in test");
                let identity = self_knowledge.identity();

                assert_eq!(identity.name, "test-squirrel");
                assert_eq!(identity.primal_type, "squirrel");
                assert_eq!(identity.capabilities.len(), 2);
                assert!(
                    identity
                        .capabilities
                        .iter()
                        .any(|c| c.as_ref() == "ai.query")
                );
                assert!(
                    identity
                        .capabilities
                        .iter()
                        .any(|c| c.as_ref() == "system.health")
                );
                assert_eq!(identity.port, Some(9200));
                assert!(!identity.instance_id.is_empty());
                assert!(identity.instance_id.contains("test-squirrel"));
            },
        );
    }

    #[test]
    fn test_primal_identity_default_capabilities() {
        temp_env::with_var("PRIMAL_CAPABILITIES", None::<&str>, || {
            let self_knowledge =
                PrimalSelfKnowledge::discover_self().expect("discover_self in test");
            let identity = self_knowledge.identity();

            assert!(!identity.capabilities.is_empty());
            assert!(
                identity
                    .capabilities
                    .iter()
                    .any(|c| c.as_ref() == "ai.query")
            );
            assert!(
                identity
                    .capabilities
                    .iter()
                    .any(|c| c.as_ref() == "system.health")
            );
        });
    }

    #[test]
    fn test_niche_cost_estimates() {
        let costs = niche::cost_estimates_json();
        let map = costs
            .as_object()
            .expect("cost_estimates_json returns object");
        assert!(map.contains_key("ai.query"));
        assert!(map.contains_key("lifecycle.register"));
        let ai_query = map
            .get("ai.query")
            .and_then(|v| v.as_object())
            .expect("ai.query cost entry should be object");
        assert!(ai_query.contains_key("latency_ms"));
        assert!(ai_query.contains_key("gpu_beneficial"));
    }

    #[test]
    fn test_niche_operation_dependencies() {
        let deps = niche::operation_dependencies();
        let map = deps
            .as_object()
            .expect("operation_dependencies returns object");
        assert!(map.contains_key("ai.query"));
        assert!(
            map.get("ai.query")
                .and_then(|v| v.as_array())
                .expect("ai.query dependency list")
                .contains(&serde_json::json!("prompt"))
        );
        assert!(map.contains_key("tool.execute"));
        assert!(
            map.get("tool.execute")
                .and_then(|v| v.as_array())
                .expect("tool.execute dependency list")
                .contains(&serde_json::json!("tool"))
        );
    }

    #[test]
    fn test_niche_consumed_capabilities() {
        assert!(niche::CONSUMED_CAPABILITIES.contains(&"discovery.register"));
        assert!(niche::CONSUMED_CAPABILITIES.contains(&"crypto.sign"));
        assert!(niche::CONSUMED_CAPABILITIES.iter().all(|c| c.contains('.')));
    }

    #[tokio::test]
    async fn test_discovered_empty_initially() {
        let self_knowledge = PrimalSelfKnowledge::discover_self().expect("discover_self in test");
        let discovered = self_knowledge.discovered().await;
        assert!(discovered.is_empty());
    }

    #[tokio::test]
    async fn test_announce_capabilities_succeeds() {
        let self_knowledge = PrimalSelfKnowledge::discover_self().expect("discover_self in test");
        let result = self_knowledge.announce_capabilities().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_announce_capabilities_with_port() {
        let self_knowledge = PrimalSelfKnowledge::discover_self().expect("discover_self in test");
        let result = self_knowledge.announce_capabilities_with_port(9999).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_primal_identity_serialization_roundtrip() {
        temp_env::with_vars(
            [
                ("PRIMAL_NAME", Some("serial-test")),
                ("PRIMAL_TYPE", Some("squirrel")),
                ("PRIMAL_CAPABILITIES", Some("ai.query,system.health")),
            ],
            || {
                let self_knowledge =
                    PrimalSelfKnowledge::discover_self().expect("discover_self in test");
                let identity = self_knowledge.identity();
                let json = serde_json::to_string(identity).expect("identity serializes");
                let deserialized: PrimalIdentity =
                    serde_json::from_str(&json).expect("identity deserializes");
                assert_eq!(identity.name, deserialized.name);
                assert_eq!(identity.primal_type, deserialized.primal_type);
                assert_eq!(identity.capabilities.len(), deserialized.capabilities.len());
                assert_eq!(identity.version, deserialized.version);
            },
        );
    }

    #[test]
    fn test_primal_identity_host_fallback() {
        temp_env::with_var("PRIMAL_NAME", Some("host-test"), || {
            let self_knowledge =
                PrimalSelfKnowledge::discover_self().expect("discover_self in test");
            let identity = self_knowledge.identity();
            assert!(!identity.host.is_empty());
            assert!(identity.host == "localhost" || hostname::get().is_ok());
        });
    }

    #[test]
    fn test_primal_identity_instance_id_format() {
        temp_env::with_var("PRIMAL_NAME", Some("instance-test"), || {
            let self_knowledge =
                PrimalSelfKnowledge::discover_self().expect("discover_self in test");
            let identity = self_knowledge.identity();
            assert!(identity.instance_id.contains("instance-test"));
            assert!(
                identity
                    .instance_id
                    .contains(&std::process::id().to_string())
            );
        });
    }
}
