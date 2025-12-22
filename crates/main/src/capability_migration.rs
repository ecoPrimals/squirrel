//! Capability Migration Helper
//!
//! This module provides utilities to migrate from hardcoded primal names
//! to capability-based discovery using the CapabilityRegistry.
//!
//! # Migration Strategy
//!
//! 1. Use this module as a temporary bridge during migration
//! 2. Replace hardcoded "beardog", "songbird" etc. with capability queries
//! 3. Gradually remove this module once migration is complete
//!
//! # Example
//!
//! ```no_run
//! use squirrel::capability_migration::CapabilityMigrationHelper;
//! use squirrel::capability_registry::PrimalCapability;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let helper = CapabilityMigrationHelper::new().await?;
//!
//! // Old way (hardcoded):
//! // let beardog_url = "https://beardog.local";
//!
//! // New way (capability-based):
//! let security_primals = helper.discover_by_capability(&PrimalCapability::Security).await?;
//! if let Some(primal) = security_primals.first() {
//!     let endpoint = &primal.endpoint;
//!     // Use endpoint for security operations
//! }
//! # Ok(())
//! # }
//! ```

use std::collections::HashMap;
use std::sync::Arc;

use crate::capability_registry::{
    CapabilityRegistry, CapabilityRegistryConfig, PrimalCapability, RegisteredPrimal,
};
use crate::error::PrimalError;

/// Information about a discovered primal service
#[derive(Debug, Clone, serde::Deserialize)]
struct PrimalDiscoveryInfo {
    id: String,
    name: String,
    capabilities: std::collections::HashSet<PrimalCapability>,
    endpoint: String,
    health_endpoint: String,
    #[serde(default)]
    metadata: HashMap<String, String>,
}

/// Helper for migrating from hardcoded primal names to capability-based discovery
pub struct CapabilityMigrationHelper {
    registry: Arc<CapabilityRegistry>,
}

impl CapabilityMigrationHelper {
    /// Create a new migration helper with default registry
    pub async fn new() -> Result<Self, PrimalError> {
        let config = CapabilityRegistryConfig::default();
        let registry = Arc::new(CapabilityRegistry::new(config));

        // Auto-discover and register available primals
        let helper = Self { registry };
        helper.auto_discover_primals().await?;

        Ok(helper)
    }

    /// Create with existing registry
    pub fn with_registry(registry: Arc<CapabilityRegistry>) -> Self {
        Self { registry }
    }

    /// Auto-discover primals from environment and register them
    ///
    /// Implements multi-source discovery:
    /// 1. Environment variables (PRIMAL_*_ENDPOINT)
    /// 2. Service mesh discovery (via SONGBIRD_DISCOVERY_URL)
    /// 3. DNS-SD service discovery (if enabled)
    async fn auto_discover_primals(&self) -> Result<(), PrimalError> {
        // Strategy 1: Register from environment variables
        self.discover_from_env_vars().await?;

        // Strategy 2: Register from service mesh discovery
        self.discover_from_service_mesh().await?;

        // Strategy 3: DNS-SD discovery (optional, requires feature flag)
        #[cfg(feature = "dns-sd")]
        self.discover_from_dns_sd().await?;

        Ok(())
    }

    /// Discover primals from environment variables
    async fn discover_from_env_vars(&self) -> Result<(), PrimalError> {
        // Check for BearDog (Security)
        if let Ok(endpoint) = std::env::var("BEARDOG_ENDPOINT") {
            let mut capabilities = std::collections::HashSet::new();
            capabilities.insert(PrimalCapability::Security);
            capabilities.insert(PrimalCapability::Monitoring);

            self.registry
                .register_primal(
                    "beardog-security".to_string(),
                    "BearDog Security Service".to_string(),
                    capabilities,
                    endpoint.clone(),
                    format!("{}/health", endpoint),
                    HashMap::new(),
                )
                .await?;
        }

        // Check for Songbird (Service Mesh)
        if let Ok(endpoint) = std::env::var("SONGBIRD_ENDPOINT") {
            let mut capabilities = std::collections::HashSet::new();
            capabilities.insert(PrimalCapability::ServiceMesh);
            capabilities.insert(PrimalCapability::Monitoring);

            self.registry
                .register_primal(
                    "songbird-mesh".to_string(),
                    "Songbird Service Mesh".to_string(),
                    capabilities,
                    endpoint.clone(),
                    format!("{}/health", endpoint),
                    HashMap::new(),
                )
                .await?;
        }

        // Check for ToadStool (Storage)
        if let Ok(endpoint) = std::env::var("TOADSTOOL_ENDPOINT") {
            let mut capabilities = std::collections::HashSet::new();
            capabilities.insert(PrimalCapability::Storage);
            capabilities.insert(PrimalCapability::Monitoring);

            self.registry
                .register_primal(
                    "toadstool-storage".to_string(),
                    "ToadStool Storage Service".to_string(),
                    capabilities,
                    endpoint.clone(),
                    format!("{}/health", endpoint),
                    HashMap::new(),
                )
                .await?;
        }

        // Check for NestGate (Compute)
        if let Ok(endpoint) = std::env::var("NESTGATE_ENDPOINT") {
            let mut capabilities = std::collections::HashSet::new();
            capabilities.insert(PrimalCapability::Compute);
            capabilities.insert(PrimalCapability::Monitoring);

            self.registry
                .register_primal(
                    "nestgate-compute".to_string(),
                    "NestGate Compute Service".to_string(),
                    capabilities,
                    endpoint.clone(),
                    format!("{}/health", endpoint),
                    HashMap::new(),
                )
                .await?;
        }

        Ok(())
    }

    /// Discover primals from service mesh (Songbird)
    async fn discover_from_service_mesh(&self) -> Result<(), PrimalError> {
        // Check if service mesh discovery is available
        if let Ok(discovery_url) = std::env::var("SONGBIRD_DISCOVERY_URL") {
            // Attempt to query service mesh for registered primals
            match self.query_service_mesh(&discovery_url).await {
                Ok(primals) => {
                    for primal_info in primals {
                        self.registry
                            .register_primal(
                                primal_info.id,
                                primal_info.name,
                                primal_info.capabilities,
                                primal_info.endpoint,
                                primal_info.health_endpoint,
                                primal_info.metadata,
                            )
                            .await?;
                    }
                }
                Err(e) => {
                    // Log warning but don't fail - mesh discovery is optional
                    tracing::warn!("Service mesh discovery failed: {}", e);
                }
            }
        }

        Ok(())
    }

    /// Query service mesh for registered primals
    async fn query_service_mesh(
        &self,
        discovery_url: &str,
    ) -> Result<Vec<PrimalDiscoveryInfo>, PrimalError> {
        // Use HTTP client to query service mesh
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .map_err(|e| PrimalError::NetworkError(e.to_string()))?;

        let response = client
            .get(format!("{}/api/v1/services", discovery_url))
            .send()
            .await
            .map_err(|e| PrimalError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(PrimalError::ServiceDiscoveryError(format!(
                "Service mesh query failed: {}",
                response.status()
            )));
        }

        let services: Vec<PrimalDiscoveryInfo> = response
            .json()
            .await
            .map_err(|e| PrimalError::SerializationError(e.to_string()))?;

        Ok(services)
    }

    /// Discover primals via DNS-SD (Service Discovery)
    #[cfg(feature = "dns-sd")]
    async fn discover_from_dns_sd(&self) -> Result<(), PrimalError> {
        // DNS-SD service discovery using mDNS
        // Look for services advertised as _squirrel-primal._tcp.local
        tracing::info!("DNS-SD discovery not yet implemented");
        Ok(())
    }

    /// Discover primals by capability
    pub async fn discover_by_capability(
        &self,
        capability: &PrimalCapability,
    ) -> Result<Vec<RegisteredPrimal>, PrimalError> {
        self.registry.discover_by_capability(capability).await
    }

    /// Get the first healthy primal for a capability
    pub async fn get_primal_for_capability(
        &self,
        capability: &PrimalCapability,
    ) -> Result<Option<RegisteredPrimal>, PrimalError> {
        let primals = self.discover_by_capability(capability).await?;
        Ok(primals.into_iter().next())
    }

    /// Get registry reference
    pub fn registry(&self) -> Arc<CapabilityRegistry> {
        Arc::clone(&self.registry)
    }

    /// Migration helper: Replace hardcoded "beardog" with capability-based discovery
    pub async fn get_security_service(&self) -> Result<Option<String>, PrimalError> {
        let primal = self
            .get_primal_for_capability(&PrimalCapability::Security)
            .await?;
        Ok(primal.map(|p| p.endpoint.to_string()))
    }

    /// Migration helper: Replace hardcoded "songbird" with capability-based discovery
    pub async fn get_service_mesh(&self) -> Result<Option<String>, PrimalError> {
        let primal = self
            .get_primal_for_capability(&PrimalCapability::ServiceMesh)
            .await?;
        Ok(primal.map(|p| p.endpoint.to_string()))
    }

    /// Migration helper: Replace hardcoded "toadstool" with capability-based discovery
    pub async fn get_storage_service(&self) -> Result<Option<String>, PrimalError> {
        let primal = self
            .get_primal_for_capability(&PrimalCapability::Storage)
            .await?;
        Ok(primal.map(|p| p.endpoint.to_string()))
    }

    /// Migration helper: Replace hardcoded "nestgate" with capability-based discovery
    pub async fn get_compute_service(&self) -> Result<Option<String>, PrimalError> {
        let primal = self
            .get_primal_for_capability(&PrimalCapability::Compute)
            .await?;
        Ok(primal.map(|p| p.endpoint.to_string()))
    }

    /// Register a primal manually
    pub async fn register_primal(
        &self,
        id: String,
        display_name: String,
        capabilities: std::collections::HashSet<PrimalCapability>,
        endpoint: String,
    ) -> Result<(), PrimalError> {
        let health_endpoint = format!("{}/health", endpoint);
        self.registry
            .register_primal(
                id,
                display_name,
                capabilities,
                endpoint,
                health_endpoint,
                HashMap::new(),
            )
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_migration_helper_creation() {
        let helper = CapabilityMigrationHelper::new().await;
        assert!(helper.is_ok());
    }

    #[tokio::test]
    async fn test_capability_discovery() {
        let helper = CapabilityMigrationHelper::new()
            .await
            .expect("Failed to create migration helper");

        // Register a test security service
        let mut caps = std::collections::HashSet::new();
        caps.insert(PrimalCapability::Security);

        helper
            .register_primal(
                "test-security".to_string(),
                "Test Security".to_string(),
                caps,
                std::env::var("TEST_SECURITY_ENDPOINT")
                    .unwrap_or_else(|_| "http://localhost:9000".to_string()),
            )
            .await
            .expect("Failed to register test primal");

        // Discover by capability
        let security_services = helper
            .discover_by_capability(&PrimalCapability::Security)
            .await
            .unwrap();

        assert!(!security_services.is_empty());
        assert_eq!(security_services[0].id, "test-security");
    }

    #[tokio::test]
    async fn test_migration_helpers() {
        let helper = CapabilityMigrationHelper::new().await.unwrap();

        // Register test services
        let mut security_caps = std::collections::HashSet::new();
        security_caps.insert(PrimalCapability::Security);
        helper
            .register_primal(
                "beardog-test".to_string(),
                "BearDog".to_string(),
                security_caps,
                "http://beardog.local".to_string(),
            )
            .await
            .unwrap();

        let mut storage_caps = std::collections::HashSet::new();
        storage_caps.insert(PrimalCapability::Storage);
        helper
            .register_primal(
                "toadstool-test".to_string(),
                "ToadStool".to_string(),
                storage_caps,
                "http://toadstool.local".to_string(),
            )
            .await
            .unwrap();

        // Test migration helpers
        let security_endpoint = helper.get_security_service().await.unwrap();
        assert_eq!(security_endpoint, Some("http://beardog.local".to_string()));

        let storage_endpoint = helper.get_storage_service().await.unwrap();
        assert_eq!(storage_endpoint, Some("http://toadstool.local".to_string()));
    }
}
