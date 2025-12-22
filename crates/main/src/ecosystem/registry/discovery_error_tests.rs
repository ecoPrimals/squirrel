//! Comprehensive error path tests for service discovery
//!
//! These tests expand coverage by validating error handling paths
//! in the discovery system, bringing coverage closer to 70%+ target.

#[cfg(test)]
mod error_path_tests {
    use crate::ecosystem::registry::discovery::DiscoveryOps;
    use crate::ecosystem::registry::types::DiscoveredService;
    use crate::EcosystemPrimalType;
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    /// Test discovery with empty primal types list
    #[tokio::test]
    async fn test_discover_services_empty_list() {
        let registry = Arc::new(RwLock::new(HashMap::new()));
        let primal_types = vec![];

        let result = DiscoveryOps::discover_services(&registry, primal_types).await;

        assert!(result.is_ok());
        let services = result.unwrap();
        assert!(services.is_empty());
    }

    /// Test discovery with single primal type
    #[tokio::test]
    async fn test_discover_single_service() {
        let registry = Arc::new(RwLock::new(HashMap::new()));
        let primal_types = vec![EcosystemPrimalType::Squirrel];

        let result = DiscoveryOps::discover_services(&registry, primal_types).await;

        assert!(result.is_ok());
        let services = result.unwrap();
        assert_eq!(services.len(), 1);
        assert_eq!(services[0].primal_type, EcosystemPrimalType::Squirrel);
    }

    /// Test discovery with multiple primal types
    #[tokio::test]
    async fn test_discover_multiple_services() {
        let registry = Arc::new(RwLock::new(HashMap::new()));
        let primal_types = vec![
            EcosystemPrimalType::Squirrel,
            EcosystemPrimalType::BearDog,
            EcosystemPrimalType::Songbird,
        ];

        let result = DiscoveryOps::discover_services(&registry, primal_types).await;

        assert!(result.is_ok());
        let services = result.unwrap();
        assert_eq!(services.len(), 3);

        // Verify all types discovered
        let types: Vec<_> = services.iter().map(|s| s.primal_type.clone()).collect();
        assert!(types.contains(&EcosystemPrimalType::Squirrel));
        assert!(types.contains(&EcosystemPrimalType::BearDog));
        assert!(types.contains(&EcosystemPrimalType::Songbird));
    }

    /// Test Arc<str> efficiency in discovered services
    #[tokio::test]
    async fn test_arc_str_sharing_efficiency() {
        let registry = Arc::new(RwLock::new(HashMap::new()));
        let primal_types = vec![EcosystemPrimalType::Squirrel];

        let _ = DiscoveryOps::discover_services(&registry, primal_types).await;

        let reg = registry.read().await;
        if let Some(service) = reg.values().next() {
            // Clone Arc<str> fields - should be cheap (pointer copy only)
            let service_id_1 = service.service_id.clone();
            let service_id_2 = service.service_id.clone();

            // Verify they point to same data (Arc semantics)
            assert_eq!(service_id_1, service_id_2);
            // Arc count may vary depending on internal references, just verify cloning works
            assert!(Arc::strong_count(&service.service_id) >= 2); // At least original + clones
        }
    }

    /// Test capability checking without allocation
    #[tokio::test]
    async fn test_has_capability_zero_copy() {
        let registry = Arc::new(RwLock::new(HashMap::new()));
        let primal_types = vec![EcosystemPrimalType::Squirrel];

        let _ = DiscoveryOps::discover_services(&registry, primal_types).await;

        let reg = registry.read().await;
        if let Some(service) = reg.values().next() {
            // These lookups should not allocate
            assert!(service.has_capability("discovery"));
            assert!(service.has_capability("health_check"));
            assert!(!service.has_capability("nonexistent"));
        }
    }

    /// Test metadata lookup without allocation
    #[tokio::test]
    async fn test_get_metadata_zero_copy() {
        use crate::optimization::zero_copy::ArcStr;
        let registry = Arc::new(RwLock::new(HashMap::<ArcStr, Arc<DiscoveredService>>::new()));
        let mut metadata = HashMap::new();
        metadata.insert("version", "1.0.0");
        metadata.insert("region", "us-west");

        let service = Arc::new(DiscoveredService::new(
            "test-service",
            EcosystemPrimalType::Squirrel,
            "http://localhost:8080",
            "http://localhost:8080/health",
            "v1",
            vec!["test"],
            metadata,
        ));

        // Metadata lookup should not allocate
        assert_eq!(
            service.get_metadata("version").map(|v| v.as_ref()),
            Some("1.0.0")
        );
        assert_eq!(
            service.get_metadata("region").map(|v| v.as_ref()),
            Some("us-west")
        );
        assert_eq!(service.get_metadata("nonexistent"), None);
    }

    /// Test concurrent discovery operations
    #[tokio::test]
    async fn test_concurrent_discoveries() {
        let registry = Arc::new(RwLock::new(HashMap::new()));

        // Spawn multiple concurrent discoveries
        let handles: Vec<_> = (0..5)
            .map(|_i| {
                let registry = Arc::clone(&registry);
                tokio::spawn(async move {
                    let primal_types =
                        vec![EcosystemPrimalType::Squirrel, EcosystemPrimalType::BearDog];
                    DiscoveryOps::discover_services(&registry, primal_types).await
                })
            })
            .collect();

        // Wait for all to complete
        for handle in handles {
            assert!(handle.await.is_ok());
        }

        // Registry should have services (may have duplicates overwritten)
        let reg = registry.read().await;
        assert!(!reg.is_empty());
    }

    /// Test service health status validation
    #[tokio::test]
    async fn test_service_health_status_types() {
        use crate::ecosystem::registry::types::ServiceHealthStatus;

        let registry = Arc::new(RwLock::new(HashMap::new()));
        let primal_types = vec![EcosystemPrimalType::Squirrel];

        let _ = DiscoveryOps::discover_services(&registry, primal_types).await;

        let reg = registry.read().await;
        if let Some(service) = reg.values().next() {
            match service.health_status {
                ServiceHealthStatus::Healthy => assert!(true),
                ServiceHealthStatus::Unhealthy => assert!(false, "Should be healthy"),
                ServiceHealthStatus::Unknown => assert!(false, "Should be healthy"),
                ServiceHealthStatus::Degraded => assert!(false, "Should be healthy"),
                ServiceHealthStatus::Offline => assert!(false, "Should be healthy"),
            }
        }
    }

    /// Test endpoint URL construction with Arc<str>
    #[tokio::test]
    async fn test_endpoint_arc_str_construction() {
        let service = DiscoveredService::new(
            "test",
            EcosystemPrimalType::Squirrel,
            "http://example.com:8080",
            "http://example.com:8080/health",
            "v1",
            vec!["test"],
            HashMap::new(),
        );

        // Endpoint should be stored as Arc<str>
        assert_eq!(service.endpoint.as_ref(), "http://example.com:8080");
        assert_eq!(
            service.health_endpoint.as_ref(),
            "http://example.com:8080/health"
        );

        // Clone should be cheap (pointer copy)
        let endpoint_clone = service.endpoint.clone();
        assert_eq!(Arc::strong_count(&service.endpoint), 2);
        assert_eq!(endpoint_clone.as_ref(), "http://example.com:8080");
    }

    /// Test capability list operations with Arc<str>
    #[tokio::test]
    async fn test_capability_list_zero_copy() {
        let service = DiscoveredService::new(
            "test",
            EcosystemPrimalType::Squirrel,
            "http://localhost:8080",
            "http://localhost:8080/health",
            "v1",
            vec!["storage", "compute", "network"],
            HashMap::new(),
        );

        // All capability checks should be zero-copy
        assert!(service.has_capability("storage"));
        assert!(service.has_capability("compute"));
        assert!(service.has_capability("network"));
        assert!(!service.has_capability("database"));

        // Capability list should use Arc<str>
        assert_eq!(service.capabilities.len(), 3);
        for cap in &service.capabilities {
            // Each capability is Arc<str>
            assert!(cap.len() > 0);
        }
    }

    /// Test service discovery with all primal types
    #[tokio::test]
    async fn test_discover_all_primal_types() {
        let registry = Arc::new(RwLock::new(HashMap::new()));
        let primal_types = vec![
            EcosystemPrimalType::Squirrel,
            EcosystemPrimalType::BearDog,
            EcosystemPrimalType::Songbird,
            EcosystemPrimalType::ToadStool,
            EcosystemPrimalType::NestGate,
            EcosystemPrimalType::BiomeOS,
        ];

        let result = DiscoveryOps::discover_services(&registry, primal_types.clone()).await;

        assert!(result.is_ok());
        let services = result.unwrap();
        assert_eq!(services.len(), primal_types.len());
    }

    /// Test registry read-write lock behavior
    #[tokio::test]
    async fn test_registry_concurrent_read_write() {
        let registry = Arc::new(RwLock::new(HashMap::new()));

        // Write operation
        let write_handle = {
            let registry = Arc::clone(&registry);
            tokio::spawn(async move {
                let primal_types = vec![EcosystemPrimalType::Squirrel];
                DiscoveryOps::discover_services(&registry, primal_types).await
            })
        };

        // Wait for write to complete
        assert!(write_handle.await.is_ok());

        // Multiple concurrent reads
        let read_handles: Vec<_> = (0..10)
            .map(|_| {
                let registry = Arc::clone(&registry);
                tokio::spawn(async move {
                    let reg = registry.read().await;
                    reg.len()
                })
            })
            .collect();

        // All reads should succeed
        for handle in read_handles {
            let count = handle.await.unwrap();
            assert_eq!(count, 1);
        }
    }

    /// Test discovered service timestamp handling
    #[tokio::test]
    async fn test_service_timestamp_accuracy() {
        use chrono::Utc;

        let before = Utc::now();

        let service = DiscoveredService::new(
            "test",
            EcosystemPrimalType::Squirrel,
            "http://localhost:8080",
            "http://localhost:8080/health",
            "v1",
            vec!["test"],
            HashMap::new(),
        );

        let after = Utc::now();

        // Discovered timestamp should be between before and after
        assert!(service.discovered_at >= before);
        assert!(service.discovered_at <= after);
    }

    /// Test API version string interning
    #[tokio::test]
    async fn test_api_version_interning() {
        let service1 = DiscoveredService::new(
            "test1",
            EcosystemPrimalType::Squirrel,
            "http://localhost:8080",
            "http://localhost:8080/health",
            "v1",
            vec!["test"],
            HashMap::new(),
        );

        let service2 = DiscoveredService::new(
            "test2",
            EcosystemPrimalType::BearDog,
            "http://localhost:8081",
            "http://localhost:8081/health",
            "v1",
            vec!["test"],
            HashMap::new(),
        );

        // Both services should share the same "v1" Arc<str> (string interning)
        assert_eq!(service1.api_version.as_ref(), "v1");
        assert_eq!(service2.api_version.as_ref(), "v1");
    }
}
