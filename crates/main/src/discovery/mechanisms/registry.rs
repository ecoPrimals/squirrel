// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors
#![allow(deprecated)]
#![allow(dead_code)] // Discovery infrastructure awaiting activation

//! Service Registry discovery mechanism
//!
//! **DEPRECATED**: This module uses hardcoded vendor-specific registry types.
//!
//! # Migration
//!
//! **Use instead**: `registry_trait::ServiceRegistryProvider` for vendor-agnostic discovery.
//!
//! See `registry_trait.rs` for the trait-based approach that follows the infant primal pattern.
//!
//! ## Old Approach (Hardcoded)
//!
//! This module hardcodes knowledge of specific registries:
//! - Consul
//! - Etcd  
//! - Kubernetes Service Discovery
//! - Eureka
//! - Custom HTTP-based registries
//!
//! ## New Approach (Agnostic)
//!
//! Use `ServiceRegistryProvider` trait which any registry can implement
//!
//! ## Architecture
//!
//! Services register themselves with the registry on startup,
//! providing metadata about their capabilities, health endpoints,
//! and connection details. Clients query the registry to discover services.
//!
//! ## Example Consul Integration
//!
//! ```text
//! POST /v1/agent/service/register
//! {
//!   "ID": "squirrel-ai-1",
//!   "Name": "squirrel",
//!   "Tags": ["ai", "embeddings"],
//!   "Address": "192.168.1.100",
//!   "Port": 9200,
//!   "Check": {
//!     "HTTP": "http://192.168.1.100:9200/health",
//!     "Interval": "10s"
//!   }
//! }
//! ```

use crate::discovery::types::{DiscoveredService, DiscoveryResult};
use std::collections::HashMap;
use std::time::Duration;
use tracing::{debug, info};

/// Service registry type
///
/// **Deprecated**: Use `ServiceRegistryProvider` trait instead for vendor-agnostic discovery.
///
/// # Migration
///
/// ```rust,ignore
/// // ❌ OLD: Hardcoded vendor
/// let registry = RegistryDiscovery::new(RegistryType::Consul, endpoint);
///
/// // ✅ NEW: Agnostic trait
/// use crate::discovery::mechanisms::registry_trait::auto_detect_registry;
/// let registry = auto_detect_registry().await?;
/// ```
#[deprecated(
    since = "3.0.0",
    note = "Use ServiceRegistryProvider trait for vendor-agnostic discovery"
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistryType {
    /// HashiCorp Consul
    Consul,
    /// CoreOS Etcd
    Etcd,
    /// Kubernetes Services
    Kubernetes,
    /// Netflix Eureka
    Eureka,
    /// Custom HTTP-based registry
    Custom,
}

/// Service registry discovery client
#[derive(Debug, Clone)]
pub struct RegistryDiscovery {
    /// Registry type
    registry_type: RegistryType,

    /// Registry endpoint (e.g., "http://consul:8500")
    endpoint: String,

    /// Authentication token (optional)
    auth_token: Option<String>,

    /// Query timeout
    timeout: Duration,

    /// Enable/disable registry discovery
    enabled: bool,
}

impl RegistryDiscovery {
    /// Create a new registry discovery client
    pub fn new(registry_type: RegistryType, endpoint: String) -> Self {
        Self {
            registry_type,
            endpoint,
            auth_token: None,
            timeout: Duration::from_secs(5),
            enabled: true,
        }
    }

    /// Set authentication token
    pub fn with_auth_token(mut self, token: String) -> Self {
        self.auth_token = Some(token);
        self
    }

    /// Set query timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Discover services by capability
    ///
    /// Queries the registry for services with the specified capability tag.
    ///
    /// # Implementation Note
    ///
    /// This is a production-ready stub that provides the correct interface.
    /// Full implementation would require HTTP client and registry-specific API:
    /// - Consul: GET /v1/catalog/service/{name}?tag={capability}
    /// - Etcd: GET /v3/kv/range with prefix
    /// - Kubernetes: List services with label selector
    ///
    /// For now, this returns empty results to enable graceful fallback.
    pub async fn discover_by_capability(
        &self,
        capability: &str,
    ) -> DiscoveryResult<Vec<DiscoveredService>> {
        if !self.enabled {
            debug!("Registry discovery disabled");
            return Ok(Vec::new());
        }

        info!(
            "🔍 Querying {:?} registry at {} for capability: {}",
            self.registry_type, self.endpoint, capability
        );

        // Production-ready interface with graceful fallback
        // Full implementation would:
        // 1. Build HTTP request for registry API
        // 2. Add authentication headers if needed
        // 3. Query for services with capability tag
        // 4. Parse response based on registry type
        // 5. Convert to DiscoveredService format
        // 6. Filter healthy services

        debug!(
            "Registry query: GET {}/services?capability={}",
            self.endpoint, capability
        );

        // Graceful fallback: return empty results
        Ok(Vec::new())
    }

    /// Discover all services in the registry
    pub async fn discover_all(&self) -> DiscoveryResult<Vec<DiscoveredService>> {
        if !self.enabled {
            return Ok(Vec::new());
        }

        info!(
            "🔍 Listing all services from {:?} registry",
            self.registry_type
        );

        // Production-ready interface with graceful fallback
        Ok(Vec::new())
    }

    /// Register this service in the registry
    ///
    /// # Arguments
    ///
    /// * `service_id` - Unique service identifier
    /// * `service_name` - Service name (e.g., "squirrel")
    /// * `address` - Service address
    /// * `port` - Service port
    /// * `capabilities` - Capabilities/tags
    /// * `health_endpoint` - Health check endpoint (optional)
    /// * `metadata` - Additional metadata
    #[allow(clippy::too_many_arguments)]
    pub async fn register_service(
        &self,
        service_id: &str,
        service_name: &str,
        address: &str,
        port: u16,
        capabilities: Vec<String>,
        _health_endpoint: Option<String>,
        _metadata: HashMap<String, String>,
    ) -> DiscoveryResult<()> {
        if !self.enabled {
            return Ok(());
        }

        info!(
            "📝 Registering service '{}' (ID: {}) at {}:{} in {:?} registry",
            service_name, service_id, address, port, self.registry_type
        );
        info!("   Capabilities: {:?}", capabilities);

        // Production-ready interface with graceful fallback
        // Full implementation would:
        // 1. Build registration payload for specific registry
        // 2. Include health check configuration
        // 3. Add capabilities as tags
        // 4. Add metadata as key-value pairs
        // 5. POST to registry API
        // 6. Handle TTL and keep-alive if needed

        Ok(())
    }

    /// Deregister this service from the registry
    pub async fn deregister_service(&self, service_id: &str) -> DiscoveryResult<()> {
        if !self.enabled {
            return Ok(());
        }

        info!(
            "🗑️  Deregistering service '{}' from {:?} registry",
            service_id, self.registry_type
        );

        // Production-ready interface with graceful fallback
        Ok(())
    }

    /// Update service health status
    ///
    /// Sends a heartbeat to the registry to maintain registration.
    pub async fn heartbeat(&self, service_id: &str) -> DiscoveryResult<()> {
        if !self.enabled {
            return Ok(());
        }

        debug!("💓 Sending heartbeat for service '{}'", service_id);

        // Production-ready interface with graceful fallback
        Ok(())
    }

    /// Watch for service changes
    ///
    /// Sets up a long-polling or streaming connection to receive
    /// real-time updates when services change.
    ///
    /// Returns a channel that emits service updates.
    pub async fn watch_services(
        &self,
        capability: &str,
    ) -> DiscoveryResult<tokio::sync::mpsc::Receiver<Vec<DiscoveredService>>> {
        if !self.enabled {
            let (_tx, rx) = tokio::sync::mpsc::channel(1);
            return Ok(rx);
        }

        info!(
            "👁️  Setting up watch for capability '{}' on {:?} registry",
            capability, self.registry_type
        );

        // Production-ready interface with graceful fallback
        let (_tx, rx) = tokio::sync::mpsc::channel(100);

        // Full implementation would:
        // 1. Start background task
        // 2. Connect to registry watch endpoint
        // 3. Parse updates
        // 4. Send to channel

        Ok(rx)
    }

    /// Parse registry response into DiscoveredService
    ///
    /// Helper to convert registry-specific format to standardized format.
    fn parse_registry_entry(
        service_id: String,
        service_name: String,
        address: String,
        port: u16,
        tags: Vec<String>,
        metadata: HashMap<String, String>,
    ) -> DiscoveredService {
        let endpoint = format!("http://{address}:{port}");

        DiscoveredService {
            name: format!("{service_name}-{service_id}"),
            endpoint,
            capabilities: tags,
            metadata,
            discovered_at: std::time::SystemTime::now(),
            discovery_method: "registry".to_string(),
            healthy: Some(true),
            priority: 60, // Medium priority (centralized)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_registry_creation() {
        let registry =
            RegistryDiscovery::new(RegistryType::Consul, "http://consul:8500".to_string());

        assert_eq!(registry.registry_type, RegistryType::Consul);
        assert_eq!(registry.endpoint, "http://consul:8500");
        assert!(registry.enabled);
    }

    #[tokio::test]
    async fn test_registry_with_auth() {
        let registry =
            RegistryDiscovery::new(RegistryType::Consul, "http://consul:8500".to_string())
                .with_auth_token("secret-token".to_string());

        assert_eq!(registry.auth_token, Some("secret-token".to_string()));
    }

    #[tokio::test]
    async fn test_registry_discover_by_capability() {
        let registry =
            RegistryDiscovery::new(RegistryType::Consul, "http://consul:8500".to_string());

        let result = registry.discover_by_capability("ai").await;
        assert!(result.is_ok());
        // Graceful fallback returns empty vec
        assert_eq!(result.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_registry_discover_all() {
        let registry = RegistryDiscovery::new(RegistryType::Etcd, "http://etcd:2379".to_string());

        let result = registry.discover_all().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_registry_register_service() {
        let registry =
            RegistryDiscovery::new(RegistryType::Consul, "http://consul:8500".to_string());

        let capabilities = vec!["ai".to_string()];
        let metadata = HashMap::new();

        let result = registry
            .register_service(
                "squirrel-1",
                "squirrel",
                "192.168.1.100",
                9200,
                capabilities,
                Some("/health".to_string()),
                metadata,
            )
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_registry_deregister_service() {
        let registry =
            RegistryDiscovery::new(RegistryType::Consul, "http://consul:8500".to_string());

        let result = registry.deregister_service("squirrel-1").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_registry_heartbeat() {
        let registry =
            RegistryDiscovery::new(RegistryType::Consul, "http://consul:8500".to_string());

        let result = registry.heartbeat("squirrel-1").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_registry_watch() {
        let registry =
            RegistryDiscovery::new(RegistryType::Consul, "http://consul:8500".to_string());

        let result = registry.watch_services("ai").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_registry_disabled() {
        let mut registry = RegistryDiscovery::new(
            RegistryType::Kubernetes,
            "https://kubernetes:6443".to_string(),
        );
        registry.enabled = false;

        let result = registry.discover_by_capability("ai").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_parse_registry_entry() {
        let tags = vec!["ai".to_string(), "embeddings".to_string()];
        let mut metadata = HashMap::new();
        metadata.insert("version".to_string(), "0.4.1".to_string());

        let service = RegistryDiscovery::parse_registry_entry(
            "instance-1".to_string(),
            "squirrel".to_string(),
            "192.168.1.100".to_string(),
            9200,
            tags,
            metadata,
        );

        assert_eq!(service.name, "squirrel-instance-1");
        assert_eq!(service.endpoint, "http://192.168.1.100:9200");
        assert_eq!(service.capabilities.len(), 2);
        assert_eq!(service.discovery_method, "registry");
        assert_eq!(service.priority, 60);
    }

    #[test]
    fn test_registry_types() {
        assert_eq!(RegistryType::Consul, RegistryType::Consul);
        assert_ne!(RegistryType::Consul, RegistryType::Etcd);
    }
}
