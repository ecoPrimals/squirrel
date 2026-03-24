// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Agnostic Service Registry Trait - Infant Primal Pattern
//!
//! **Philosophy**: Zero vendor lock-in. Any service registry can implement this trait.
//!
//! Following the infant primal pattern:
//! - No hardcoded vendor names (Consul, k8s, etc.)
//! - Runtime provider selection
//! - Graceful degradation
//!
//! # Example
//!
//! ```rust,ignore
//! // ❌ BAD: Vendor-specific
//! let consul = ConsulClient::new("http://consul:8500");
//! let services = consul.query_services().await?;
//!
//! // ✅ GOOD: Agnostic trait
//! let registry: Box<dyn ServiceRegistryProvider> = detect_registry().await?;
//! let services = registry.discover_by_capability("ai").await?;
//! ```
//!
//! # Implementations
//!
//! Any service registry can implement this trait:
//! - Consul (HashiCorp)
//! - Etcd (CoreOS)
//! - Kubernetes Services
//! - Eureka (Netflix)
//! - mDNS (local)
//! - Custom HTTP registries
//! - File-based (for testing)

use crate::discovery::types::{DiscoveredService, DiscoveryError, DiscoveryResult};
use async_trait::async_trait;
use std::collections::HashMap;

/// Agnostic Service Registry Provider
///
/// Any service registry can implement this trait to provide
/// service discovery capabilities without vendor lock-in.
///
/// # Infant Primal Pattern
///
/// Implementations should:
/// 1. Discover their own configuration from environment
/// 2. Gracefully handle missing services
/// 3. Provide health checking
/// 4. Support capability-based queries
#[async_trait]
pub trait ServiceRegistryProvider: Send + Sync {
    /// Provider name (for logging/debugging)
    ///
    /// Examples: "consul", "kubernetes", "mdns", "local"
    fn provider_name(&self) -> &str;

    /// Discover services by capability
    ///
    /// Returns all services that provide the specified capability.
    ///
    /// # Arguments
    ///
    /// * `capability` - The capability to search for (e.g., "ai", "storage", "compute")
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<DiscoveredService>)` - List of services (may be empty)
    /// * `Err(DiscoveryError)` - Only on fatal errors, not for "no services found"
    async fn discover_by_capability(
        &self,
        capability: &str,
    ) -> DiscoveryResult<Vec<DiscoveredService>>;

    /// Discover all services
    ///
    /// Returns all services in the registry, regardless of capability.
    async fn discover_all(&self) -> DiscoveryResult<Vec<DiscoveredService>>;

    /// Register this service in the registry
    ///
    /// Announces this primal's presence and capabilities.
    ///
    /// # Arguments
    ///
    /// * `service` - Service information to register
    async fn register_service(&self, service: DiscoveredService) -> DiscoveryResult<()>;

    /// Deregister this service from the registry
    ///
    /// Removes this primal from the registry (typically on shutdown).
    ///
    /// # Arguments
    ///
    /// * `service_id` - The ID of the service to deregister
    async fn deregister_service(&self, service_id: &str) -> DiscoveryResult<()>;

    /// Health check - verify registry is reachable
    ///
    /// Returns true if the registry is healthy and reachable.
    async fn health_check(&self) -> bool;

    /// Get provider metadata
    ///
    /// Returns additional information about this provider (optional).
    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert("provider".to_string(), self.provider_name().to_string());
        meta
    }
}

/// Auto-detect service registry provider
///
/// Attempts to detect and create an appropriate service registry provider
/// based on the runtime environment.
///
/// # Detection Order
///
/// 1. Environment variable `SERVICE_REGISTRY_TYPE`
/// 2. Kubernetes detection (if `/var/run/secrets/kubernetes.io` exists)
/// 3. Consul detection (if `CONSUL_HTTP_ADDR` is set)
/// 4. mDNS (local network)
/// 5. File-based (for testing)
///
/// # Example
///
/// ```rust,ignore
/// let registry = auto_detect_registry().await?;
/// println!("Using registry: {}", registry.provider_name());
/// ```
pub async fn auto_detect_registry() -> DiscoveryResult<Box<dyn ServiceRegistryProvider>> {
    use tracing::{debug, info};

    // 1. Check environment variable
    if let Ok(registry_type) = std::env::var("SERVICE_REGISTRY_TYPE") {
        info!("Service registry type specified: {}", registry_type);
        return create_registry_from_type(&registry_type).await;
    }

    // 2. Detect Kubernetes
    if std::path::Path::new("/var/run/secrets/kubernetes.io").exists() {
        debug!("Detected Kubernetes environment");
        return create_registry_from_type("kubernetes").await;
    }

    // 3. Detect Consul
    if std::env::var("CONSUL_HTTP_ADDR").is_ok() {
        debug!("Detected Consul environment");
        return create_registry_from_type("consul").await;
    }

    // 4. Fall back to mDNS
    debug!("No service registry detected, using mDNS");
    create_registry_from_type("mdns").await
}

/// Create registry provider from type string
async fn create_registry_from_type(
    registry_type: &str,
) -> DiscoveryResult<Box<dyn ServiceRegistryProvider>> {
    match registry_type.to_lowercase().as_str() {
        "kubernetes" | "k8s" => {
            // TRUE PRIMAL PRINCIPLE: No hardcoded external service integrations
            // Kubernetes would be discovered at runtime via capability advertisements
            // See: docs/true-primal-philosophy/ for design rationale
            Err(DiscoveryError::NotSupported(
                "Kubernetes registry provider not implemented (by design - use runtime discovery)"
                    .to_string(),
            ))
        }
        "consul" => {
            // TRUE PRIMAL PRINCIPLE: No hardcoded external service integrations
            Err(DiscoveryError::NotSupported(
                "Consul registry provider not implemented (by design - use runtime discovery)"
                    .to_string(),
            ))
        }
        "mdns" => {
            // TRUE PRIMAL PRINCIPLE: No hardcoded external service integrations
            Err(DiscoveryError::NotSupported(
                "mDNS registry provider not implemented (by design - use runtime discovery)"
                    .to_string(),
            ))
        }
        "file" | "local" => {
            // TRUE PRIMAL PRINCIPLE: No hardcoded external service integrations
            Err(DiscoveryError::NotSupported(
                "File registry provider not implemented (by design - use runtime discovery)"
                    .to_string(),
            ))
        }
        unknown => Err(DiscoveryError::NotSupported(format!(
            "Unknown registry type: {unknown}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock implementation for testing
    struct MockRegistryProvider {
        name: String,
        services: Vec<DiscoveredService>,
    }

    #[async_trait]
    impl ServiceRegistryProvider for MockRegistryProvider {
        fn provider_name(&self) -> &str {
            &self.name
        }

        async fn discover_by_capability(
            &self,
            _capability: &str,
        ) -> DiscoveryResult<Vec<DiscoveredService>> {
            Ok(self.services.clone())
        }

        async fn discover_all(&self) -> DiscoveryResult<Vec<DiscoveredService>> {
            Ok(self.services.clone())
        }

        async fn register_service(&self, _service: DiscoveredService) -> DiscoveryResult<()> {
            Ok(())
        }

        async fn deregister_service(&self, _service_id: &str) -> DiscoveryResult<()> {
            Ok(())
        }

        async fn health_check(&self) -> bool {
            true
        }
    }

    #[tokio::test]
    async fn test_provider_trait() {
        let provider = MockRegistryProvider {
            name: "test".to_string(),
            services: vec![],
        };

        assert_eq!(provider.provider_name(), "test");
        assert!(provider.health_check().await);
        assert_eq!(
            provider.discover_all().await.expect("should succeed").len(),
            0
        );
    }

    #[tokio::test]
    async fn test_metadata() {
        let provider = MockRegistryProvider {
            name: "test".to_string(),
            services: vec![],
        };

        let meta = provider.metadata();
        assert_eq!(meta.get("provider"), Some(&"test".to_string()));
    }
}
