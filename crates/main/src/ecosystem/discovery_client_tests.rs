//! Comprehensive tests for service discovery client

use super::*;
use std::collections::HashMap;

#[cfg(test)]
mod tests {
    use super::*;

    /// Test basic service discovery creation
    #[tokio::test]
    async fn test_discovery_creation() {
        let discovery = EcosystemServiceDiscovery::new();
        assert!(discovery.cache.read().await.is_empty());
    }

    /// Test explicit endpoint configuration (no env vars)
    #[tokio::test]
    async fn test_explicit_endpoint_discovery() {
        // Use config injection instead of env vars for concurrent safety
        let mut endpoint_overrides = HashMap::new();
        endpoint_overrides.insert(
            capabilities::COORDINATION.to_string(),
            "http://test-coord-unique:9000".to_string(),
        );

        let config = ServiceDiscoveryConfig {
            environment: Some("test".to_string()),
            dns_domain: None,
            endpoint_overrides,
            dev_fallback: None,
        };

        let discovery = EcosystemServiceDiscovery::new_with_config(config);
        let result = discovery
            .discover_by_capability(capabilities::COORDINATION)
            .await;

        assert!(result.is_ok());
        let service = result.unwrap();
        assert_eq!(service.endpoint, "http://test-coord-unique:9000");
        assert!(service
            .capabilities
            .contains(&capabilities::COORDINATION.to_string()));
    }

    /// Test cache functionality with explicit config
    #[tokio::test]
    async fn test_cache_behavior() {
        // Use config injection instead of env vars
        let mut endpoint_overrides = HashMap::new();
        endpoint_overrides.insert(
            capabilities::STORAGE.to_string(),
            "http://test-storage:9001".to_string(),
        );

        let config = ServiceDiscoveryConfig {
            environment: Some("test".to_string()),
            dns_domain: None,
            endpoint_overrides,
            dev_fallback: None,
        };

        let discovery = EcosystemServiceDiscovery::new_with_config(config);

        // First discovery
        let result1 = discovery
            .discover_by_capability(capabilities::STORAGE)
            .await;
        assert!(result1.is_ok());

        // Cache should now contain the service
        let cache = discovery.cache.read().await;
        assert!(cache.contains_key(capabilities::STORAGE));
    }

    /// Test cache refresh with explicit config
    #[tokio::test]
    async fn test_cache_refresh() {
        let mut endpoint_overrides = HashMap::new();
        endpoint_overrides.insert(
            capabilities::WORKLOAD.to_string(),
            "http://test-workload:9002".to_string(),
        );

        let config = ServiceDiscoveryConfig {
            environment: Some("test".to_string()),
            dns_domain: None,
            endpoint_overrides,
            dev_fallback: None,
        };

        let discovery = EcosystemServiceDiscovery::new_with_config(config);

        // Populate cache
        let _ = discovery
            .discover_by_capability(capabilities::WORKLOAD)
            .await;

        // Verify cache has data
        {
            let cache = discovery.cache.read().await;
            assert!(!cache.is_empty());
        }

        // Refresh should clear cache
        discovery.refresh().await.unwrap();

        // Verify cache is empty
        let cache = discovery.cache.read().await;
        assert!(cache.is_empty());
    }

    /// Test is_capability_available with explicit config
    #[tokio::test]
    async fn test_capability_availability() {
        let mut endpoint_overrides = HashMap::new();
        endpoint_overrides.insert(
            capabilities::SECURITY.to_string(),
            "http://test-security:9003".to_string(),
        );

        let config = ServiceDiscoveryConfig {
            environment: Some("test".to_string()),
            dns_domain: None,
            endpoint_overrides,
            dev_fallback: None,
        };

        let discovery = EcosystemServiceDiscovery::new_with_config(config);

        let available = discovery
            .is_capability_available(capabilities::SECURITY)
            .await;
        assert!(available);

        // Non-existent capability (in production without fallback)
        let prod_config = ServiceDiscoveryConfig {
            environment: Some("production".to_string()),
            dns_domain: None,
            endpoint_overrides: HashMap::new(),
            dev_fallback: None,
        };
        let prod_discovery = EcosystemServiceDiscovery::new_with_config(prod_config);
        let not_available = prod_discovery
            .is_capability_available("nonexistent_capability")
            .await;
        assert!(!not_available);
    }

    /// Test discover_all_by_capability with explicit config
    #[tokio::test]
    async fn test_discover_all() {
        let mut endpoint_overrides = HashMap::new();
        endpoint_overrides.insert(
            capabilities::AI_INTELLIGENCE.to_string(),
            "http://test-ai:9004".to_string(),
        );

        let config = ServiceDiscoveryConfig {
            environment: Some("test".to_string()),
            dns_domain: None,
            endpoint_overrides,
            dev_fallback: None,
        };

        let discovery = EcosystemServiceDiscovery::new_with_config(config);

        let services = discovery
            .discover_all_by_capability(capabilities::AI_INTELLIGENCE)
            .await;

        assert!(services.is_ok());
        let services = services.unwrap();
        assert!(!services.is_empty());
        assert_eq!(services[0].endpoint, "http://test-ai:9004");
    }

    /// Test service health status
    #[test]
    fn test_service_health_enum() {
        assert_eq!(ServiceHealth::Healthy, ServiceHealth::Healthy);
        assert_ne!(ServiceHealth::Healthy, ServiceHealth::Degraded);
        assert_ne!(ServiceHealth::Degraded, ServiceHealth::Unavailable);
        assert_ne!(ServiceHealth::Unavailable, ServiceHealth::Unknown);
    }

    /// Test capability constants
    #[test]
    fn test_capability_constants() {
        assert_eq!(capabilities::COORDINATION, "coordination");
        assert_eq!(capabilities::SECURITY, "security");
        assert_eq!(capabilities::STORAGE, "storage");
        assert_eq!(capabilities::WORKLOAD, "workload");
        assert_eq!(capabilities::AI_INTELLIGENCE, "ai_intelligence");
        assert_eq!(capabilities::MCP_PROTOCOL, "mcp_protocol");
        assert_eq!(capabilities::CONTEXT_MANAGEMENT, "context_management");
    }

    /// Test DiscoveredServiceInfo structure
    #[test]
    fn test_discovered_service_info() {
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("version".to_string(), "1.0.0".to_string());

        let service = DiscoveredServiceInfo {
            service_id: "test-service".to_string(),
            endpoint: "http://localhost:8080".to_string(),
            capabilities: vec!["test_capability".to_string()],
            health: ServiceHealth::Healthy,
            metadata,
        };

        assert_eq!(service.service_id, "test-service");
        assert_eq!(service.endpoint, "http://localhost:8080");
        assert_eq!(service.health, ServiceHealth::Healthy);
        assert_eq!(service.metadata.get("version").unwrap(), "1.0.0");
    }

    /// Test fallback behavior in development (concurrent-safe config)
    #[tokio::test]
    async fn test_development_fallback() {
        // Use explicit development config instead of env vars
        let config = ServiceDiscoveryConfig {
            environment: Some("development".to_string()),
            dns_domain: None,
            endpoint_overrides: HashMap::new(),
            dev_fallback: None,
        };

        let discovery = EcosystemServiceDiscovery::new_with_config(config);

        // Should fall back to localhost for known capabilities
        let result = discovery
            .discover_by_capability(capabilities::COORDINATION)
            .await;

        assert!(result.is_ok());
        let service = result.unwrap();
        assert!(service.endpoint.contains("localhost") || service.endpoint.contains("http"));
    }

    /// Test no fallback in production (concurrent-safe config)
    #[tokio::test]
    async fn test_production_no_fallback() {
        // Use explicit production config instead of env vars
        let config = ServiceDiscoveryConfig {
            environment: Some("production".to_string()),
            dns_domain: None,
            endpoint_overrides: HashMap::new(),
            dev_fallback: None,
        };

        let discovery = EcosystemServiceDiscovery::new_with_config(config);

        // Should fail without proper discovery in production
        let result = discovery.discover_by_capability("unknown_capability").await;

        assert!(result.is_err());
    }

    /// Test DNS discovery domain configuration (concurrent-safe config)
    #[tokio::test]
    async fn test_dns_discovery_domain() {
        // Use explicit DNS config instead of env vars
        let mut endpoint_overrides = HashMap::new();
        endpoint_overrides.insert(
            capabilities::MCP_PROTOCOL.to_string(),
            "http://mcp.test.local".to_string(),
        );

        let config = ServiceDiscoveryConfig {
            environment: Some("test".to_string()),
            dns_domain: Some("test.local".to_string()),
            endpoint_overrides,
            dev_fallback: None,
        };

        let discovery = EcosystemServiceDiscovery::new_with_config(config);

        let result = discovery
            .discover_by_capability(capabilities::MCP_PROTOCOL)
            .await;

        assert!(result.is_ok());
        let service = result.unwrap();
        assert_eq!(service.endpoint, "http://mcp.test.local");
    }

    /// Test get_service_by_id (concurrent-safe config)
    #[tokio::test]
    async fn test_get_service_by_id() {
        // Use explicit config instead of env vars
        let mut endpoint_overrides = HashMap::new();
        endpoint_overrides.insert(
            capabilities::CONTEXT_MANAGEMENT.to_string(),
            "http://test-context:9005".to_string(),
        );

        let config = ServiceDiscoveryConfig {
            environment: Some("test".to_string()),
            dns_domain: None,
            endpoint_overrides,
            dev_fallback: None,
        };

        let discovery = EcosystemServiceDiscovery::new_with_config(config);

        // First discover the service
        let service = discovery
            .discover_by_capability(capabilities::CONTEXT_MANAGEMENT)
            .await
            .unwrap();

        let service_id = service.service_id.clone();

        // Now retrieve by ID
        let result = discovery.get_service_by_id(&service_id).await;

        assert!(result.is_ok());
        let retrieved = result.unwrap();
        assert_eq!(retrieved.service_id, service_id);
        assert_eq!(retrieved.endpoint, "http://test-context:9005");
    }

    /// Test get_service_by_id with non-existent ID
    #[tokio::test]
    async fn test_get_service_by_id_not_found() {
        let discovery = EcosystemServiceDiscovery::new();

        let result = discovery.get_service_by_id("nonexistent-id").await;

        assert!(result.is_err());
    }

    /// Test multiple discoveries don't duplicate cache (concurrent-safe config)
    #[tokio::test]
    async fn test_multiple_discoveries_cache() {
        // Use explicit config instead of env vars
        let mut endpoint_overrides = HashMap::new();
        endpoint_overrides.insert(
            capabilities::COORDINATION.to_string(),
            "http://test-coord:9006".to_string(),
        );

        let config = ServiceDiscoveryConfig {
            environment: Some("test".to_string()),
            dns_domain: None,
            endpoint_overrides,
            dev_fallback: None,
        };

        let discovery = EcosystemServiceDiscovery::new_with_config(config);

        // Discover multiple times
        let _ = discovery
            .discover_by_capability(capabilities::COORDINATION)
            .await;
        let _ = discovery
            .discover_by_capability(capabilities::COORDINATION)
            .await;
        let _ = discovery
            .discover_by_capability(capabilities::COORDINATION)
            .await;

        // Cache should only have one entry
        let cache = discovery.cache.read().await;
        assert_eq!(cache.len(), 1);
    }

    /// Test default implementation
    #[test]
    fn test_default_implementation() {
        let discovery = EcosystemServiceDiscovery::default();
        // Should create successfully
        drop(discovery);
    }

    /// Test ServiceHealth copy and equality
    #[test]
    fn test_service_health_traits() {
        let health1 = ServiceHealth::Healthy;
        let health2 = health1; // Test Copy
        assert_eq!(health1, health2); // Test Eq
    }

    /// Test error message for failed discovery (concurrent-safe config)
    #[tokio::test]
    async fn test_discovery_error_message() {
        // Use explicit production config instead of env vars
        let config = ServiceDiscoveryConfig {
            environment: Some("production".to_string()),
            dns_domain: None,
            endpoint_overrides: HashMap::new(),
            dev_fallback: None,
        };

        let discovery = EcosystemServiceDiscovery::new_with_config(config);

        let result = discovery.discover_by_capability("invalid_capability").await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("No service found"));
    }
}
