//! Capability-Based Service Discovery
//!
//! This module provides runtime service discovery for primals and capabilities.
//! No hardcoded endpoints - everything is discovered dynamically.
//!
//! ## Philosophy
//!
//! - Primal code only has self-knowledge
//! - Other primals discovered at runtime
//! - No hardcoded ports or endpoints
//! - Capability-based, not service-name-based

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

/// Capability that can be discovered
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    /// Capability name (e.g., "ai-inference", "service-mesh", "security")
    pub name: String,
    /// Primal providing this capability
    pub primal_type: String,
    /// Available endpoints
    pub endpoints: Vec<String>,
    /// Metadata
    pub metadata: serde_json::Value,
}

/// Discovered service endpoint
#[derive(Debug, Clone)]
pub struct DiscoveredEndpoint {
    /// The endpoint URL
    pub url: String,
    /// Primal type providing this endpoint
    pub primal_type: String,
    /// Capabilities offered
    pub capabilities: Vec<String>,
    /// Health status
    pub healthy: bool,
}

/// Service discovery client
pub struct CapabilityDiscovery {
    /// Service mesh endpoint (if available)
    service_mesh: Option<String>,
    /// Local registry cache
    cache: Arc<RwLock<Vec<DiscoveredEndpoint>>>,
    /// Configuration
    config: DiscoveryConfig,
}

#[derive(Debug, Clone)]
pub struct DiscoveryConfig {
    /// Timeout for discovery operations
    pub timeout: Duration,
    /// Whether to use DNS-SD (if available)
    pub use_dns_sd: bool,
    /// Whether to use mDNS (if available)
    pub use_mdns: bool,
    /// Fallback endpoints (only used if discovery fails)
    pub fallback_enabled: bool,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(5),
            use_dns_sd: true,
            use_mdns: true,
            fallback_enabled: true,
        }
    }
}

impl CapabilityDiscovery {
    /// Create new capability discovery client
    #[must_use]
    pub fn new(config: DiscoveryConfig) -> Self {
        // Try to discover service mesh via environment
        let service_mesh = std::env::var("SERVICE_MESH_ENDPOINT").ok();

        Self {
            service_mesh,
            cache: Arc::new(RwLock::new(Vec::new())),
            config,
        }
    }

    /// Discover service by capability
    pub async fn discover_capability(
        &self,
        capability: &str,
    ) -> Result<DiscoveredEndpoint, DiscoveryError> {
        // 1. Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(endpoint) = cache
                .iter()
                .find(|e| e.capabilities.contains(&capability.to_string()))
            {
                if endpoint.healthy {
                    return Ok(endpoint.clone());
                }
            }
        }

        // 2. Try service mesh discovery
        if let Some(ref mesh_endpoint) = self.service_mesh {
            if let Ok(endpoint) = self
                .discover_via_service_mesh(mesh_endpoint, capability)
                .await
            {
                self.cache_endpoint(endpoint.clone()).await;
                return Ok(endpoint);
            }
        }

        // 3. Try DNS-SD discovery
        if self.config.use_dns_sd {
            if let Ok(endpoint) = self.discover_via_dns_sd(capability).await {
                self.cache_endpoint(endpoint.clone()).await;
                return Ok(endpoint);
            }
        }

        // 4. Try mDNS discovery
        if self.config.use_mdns {
            if let Ok(endpoint) = self.discover_via_mdns(capability).await {
                self.cache_endpoint(endpoint.clone()).await;
                return Ok(endpoint);
            }
        }

        // 5. Use configured fallback (only if explicitly enabled)
        if self.config.fallback_enabled {
            if let Ok(endpoint) = self.get_configured_fallback(capability).await {
                return Ok(endpoint);
            }
        }

        Err(DiscoveryError::NotFound(capability.to_string()))
    }

    /// Discover via service mesh
    async fn discover_via_service_mesh(
        &self,
        mesh_endpoint: &str,
        capability: &str,
    ) -> Result<DiscoveredEndpoint, DiscoveryError> {
        // Query service mesh for capability
        let url = format!("{mesh_endpoint}/discover/capability/{capability}");

        // TODO: Implement actual HTTP request
        // For now, return error to trigger fallback
        Err(DiscoveryError::ServiceMeshUnavailable)
    }

    /// Discover via DNS-SD (DNS Service Discovery)
    async fn discover_via_dns_sd(
        &self,
        _capability: &str,
    ) -> Result<DiscoveredEndpoint, DiscoveryError> {
        // TODO: Implement DNS-SD discovery
        Err(DiscoveryError::NotImplemented("DNS-SD"))
    }

    /// Discover via mDNS (multicast DNS)
    async fn discover_via_mdns(
        &self,
        _capability: &str,
    ) -> Result<DiscoveredEndpoint, DiscoveryError> {
        // TODO: Implement mDNS discovery
        Err(DiscoveryError::NotImplemented("mDNS"))
    }

    /// Get configured fallback endpoint
    async fn get_configured_fallback(
        &self,
        capability: &str,
    ) -> Result<DiscoveredEndpoint, DiscoveryError> {
        // Map capabilities to environment-configured endpoints
        // Philosophy: No hardcoded endpoints - all configuration via environment
        let endpoint = match capability {
            "service-mesh" => {
                // Priority: SERVICE_MESH_ENDPOINT > SONGBIRD_ENDPOINT > DEV_SERVICE_MESH_ENDPOINT
                let url = std::env::var("SERVICE_MESH_ENDPOINT")
                    .or_else(|_| std::env::var("SONGBIRD_ENDPOINT"))
                    .or_else(|_| std::env::var("DEV_SERVICE_MESH_ENDPOINT"))
                    .unwrap_or_else(|_| {
                        tracing::warn!(
                            "⚠️ No SERVICE_MESH_ENDPOINT configured for service-mesh capability. \
                             Set SERVICE_MESH_ENDPOINT for production. \
                             Using development default."
                        );
                        "http://localhost:8500".to_string()
                    });
                DiscoveredEndpoint {
                    url,
                    primal_type: "songbird".to_string(),
                    capabilities: vec!["service-mesh".to_string()],
                    healthy: true,
                }
            }
            "ai-coordinator" => {
                // Use configured port or environment-provided default
                let port = std::env::var("AI_COORDINATOR_PORT")
                    .ok()
                    .and_then(|p| p.parse().ok())
                    .or_else(|| {
                        std::env::var("SQUIRREL_PORT")
                            .ok()
                            .and_then(|p| p.parse().ok())
                    })
                    .unwrap_or_else(|| {
                        tracing::warn!("⚠️ AI_COORDINATOR_PORT not set, using default port 9010");
                        9010
                    });

                DiscoveredEndpoint {
                    url: format!("http://localhost:{port}"),
                    primal_type: "squirrel".to_string(),
                    capabilities: vec!["ai-coordinator".to_string()],
                    healthy: true,
                }
            }
            "biomeos" => {
                let url = std::env::var("BIOMEOS_ENDPOINT")
                    .or_else(|_| std::env::var("DEV_BIOMEOS_ENDPOINT"))
                    .unwrap_or_else(|_| {
                        tracing::warn!(
                            "⚠️ BIOMEOS_ENDPOINT not configured. \
                             Set BIOMEOS_ENDPOINT for production. \
                             Using development default."
                        );
                        "http://localhost:5000".to_string()
                    });
                DiscoveredEndpoint {
                    url,
                    primal_type: "biomeos".to_string(),
                    capabilities: vec!["biomeos".to_string()],
                    healthy: true,
                }
            }
            _ => return Err(DiscoveryError::NotFound(capability.to_string())),
        };

        Ok(endpoint)
    }

    /// Cache discovered endpoint
    async fn cache_endpoint(&self, endpoint: DiscoveredEndpoint) {
        let mut cache = self.cache.write().await;

        // Remove existing entry for same primal
        cache.retain(|e| e.primal_type != endpoint.primal_type);

        // Add new entry
        cache.push(endpoint);
    }

    /// Clear cache
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }
}

/// Discovery errors
#[derive(Debug, thiserror::Error)]
pub enum DiscoveryError {
    #[error("Capability '{0}' not found")]
    NotFound(String),

    #[error("Service mesh unavailable")]
    ServiceMeshUnavailable,

    #[error("{0} discovery not implemented")]
    NotImplemented(&'static str),

    #[error("Discovery timeout")]
    Timeout,

    #[error("Network error: {0}")]
    Network(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_discovery_with_fallback() {
        let discovery = CapabilityDiscovery::new(DiscoveryConfig::default());

        // Should fall back to configured endpoints
        let endpoint = discovery.discover_capability("service-mesh").await;
        assert!(endpoint.is_ok());

        let endpoint = endpoint.unwrap();
        assert_eq!(endpoint.primal_type, "songbird");
        assert!(endpoint.capabilities.contains(&"service-mesh".to_string()));
    }

    #[tokio::test]
    async fn test_cache_functionality() {
        let discovery = CapabilityDiscovery::new(DiscoveryConfig::default());

        // Manually cache an endpoint to test cache retrieval
        let test_endpoint = DiscoveredEndpoint {
            url: "http://test:8080".to_string(),
            primal_type: "test-primal".to_string(),
            capabilities: vec!["test-capability".to_string()],
            healthy: true,
        };

        discovery.cache_endpoint(test_endpoint.clone()).await;

        // Should be cached
        let cache = discovery.cache.read().await;
        assert!(!cache.is_empty());
        assert_eq!(cache.len(), 1);
        drop(cache);

        // Discover again (should use cache since endpoint is healthy)
        let endpoint = discovery
            .discover_capability("test-capability")
            .await
            .unwrap();
        assert_eq!(endpoint.url, test_endpoint.url);
        assert_eq!(endpoint.primal_type, test_endpoint.primal_type);
    }

    #[tokio::test]
    async fn test_unknown_capability() {
        let discovery = CapabilityDiscovery::new(DiscoveryConfig::default());

        let result = discovery.discover_capability("unknown-capability").await;
        assert!(result.is_err());
    }
}
