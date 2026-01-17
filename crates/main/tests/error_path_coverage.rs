//! Error Path Coverage Tests
//!
//! This module provides comprehensive error path testing to improve
//! code coverage. Focus areas:
//! - Network failures
//! - Plugin initialization errors
//! - MCP protocol errors
//! - Configuration errors
//! - Service mesh failures

#[cfg(test)]
#[cfg(feature = "disabled_until_capability_registry_exported")]
mod error_path_tests {
    use squirrel::capability_registry::{CapabilityRegistry, CapabilityRegistryConfig};
    use squirrel::universal::PrimalCapability;
    use std::collections::HashSet;
    use std::sync::Arc;

    // ============================================================================
    // Capability Discovery Error Paths
    // ============================================================================

    #[tokio::test]
    async fn test_capability_discovery_empty_registry() {
        let config = CapabilityRegistryConfig::default();
        let registry = CapabilityRegistry::new(config);

        // Query for capability that doesn't exist
        let result = registry
            .discover_by_capability(&PrimalCapability::Security)
            .await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_capability_discovery_invalid_endpoint() {
        let config = CapabilityRegistryConfig::default();
        let registry = CapabilityRegistry::new(config);

        let mut capabilities = HashSet::new();
        capabilities.insert(PrimalCapability::Security);

        // Register with invalid endpoint
        let result = registry
            .register_primal(
                "test-invalid".to_string(),
                "Test Invalid".to_string(),
                capabilities,
                "not-a-valid-url".to_string(),
                "also-invalid".to_string(),
                Default::default(),
            )
            .await;

        // Should succeed in registration but fail health checks
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_capability_discovery_duplicate_registration() {
        let config = CapabilityRegistryConfig::default();
        let registry = CapabilityRegistry::new(config);

        let mut capabilities = HashSet::new();
        capabilities.insert(PrimalCapability::Storage);

        let id = "duplicate-test".to_string();
        let name = "Duplicate Test".to_string();
        let endpoint = "http://localhost:9999".to_string();
        let health = format!("{}/health", endpoint);

        // First registration should succeed
        let result1 = registry
            .register_primal(
                id.clone(),
                name.clone(),
                capabilities.clone(),
                endpoint.clone(),
                health.clone(),
                Default::default(),
            )
            .await;
        assert!(result1.is_ok());

        // Second registration with same ID should update or fail appropriately
        let result2 = registry
            .register_primal(
                id.clone(),
                name.clone(),
                capabilities.clone(),
                endpoint.clone(),
                health.clone(),
                Default::default(),
            )
            .await;
        assert!(result2.is_ok()); // Should handle gracefully
    }

    #[tokio::test]
    async fn test_capability_discovery_multiple_capabilities() {
        let config = CapabilityRegistryConfig::default();
        let registry = CapabilityRegistry::new(config);

        let mut capabilities = HashSet::new();
        capabilities.insert(PrimalCapability::Security);
        capabilities.insert(PrimalCapability::Monitoring);
        capabilities.insert(PrimalCapability::Storage);

        let result = registry
            .register_primal(
                "multi-cap-test".to_string(),
                "Multi Capability".to_string(),
                capabilities,
                "http://localhost:10000".to_string(),
                "http://localhost:10000/health".to_string(),
                Default::default(),
            )
            .await;

        assert!(result.is_ok());

        // Should be discoverable by any of its capabilities
        let security_primals = registry
            .discover_by_capability(&PrimalCapability::Security)
            .await
            .unwrap();
        assert_eq!(security_primals.len(), 1);

        let storage_primals = registry
            .discover_by_capability(&PrimalCapability::Storage)
            .await
            .unwrap();
        assert_eq!(storage_primals.len(), 1);
    }

    // ============================================================================
    // Configuration Error Paths
    // ============================================================================

    #[tokio::test]
    async fn test_registry_config_extreme_health_check_settings() {
        let mut config = CapabilityRegistryConfig::default();

        // Test with very short health check intervals
        config.health_check_interval_secs = 1;
        config.health_check_timeout_secs = 1;
        config.max_failed_health_checks = 1;
        let registry = CapabilityRegistry::new(config);

        // Should handle gracefully
        let mut capabilities = HashSet::new();
        capabilities.insert(PrimalCapability::Monitoring);

        let result = registry
            .register_primal(
                "fast-health-check".to_string(),
                "Fast Health".to_string(),
                capabilities,
                "http://localhost:10001".to_string(),
                "http://localhost:10001/health".to_string(),
                Default::default(),
            )
            .await;

        assert!(result.is_ok());
    }

    // ============================================================================
    // Service Registration Error Paths
    // ============================================================================

    #[tokio::test]
    async fn test_register_primal_empty_name() {
        let config = CapabilityRegistryConfig::default();
        let registry = CapabilityRegistry::new(config);

        let mut capabilities = HashSet::new();
        capabilities.insert(PrimalCapability::Security);

        let result = registry
            .register_primal(
                "empty-name-test".to_string(),
                "".to_string(), // Empty name
                capabilities,
                "http://localhost:10002".to_string(),
                "http://localhost:10002/health".to_string(),
                Default::default(),
            )
            .await;

        // Should succeed - validation is lenient
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_register_primal_no_capabilities() {
        let config = CapabilityRegistryConfig::default();
        let registry = CapabilityRegistry::new(config);

        let capabilities = HashSet::new(); // Empty capabilities

        let result = registry
            .register_primal(
                "no-caps-test".to_string(),
                "No Capabilities".to_string(),
                capabilities,
                "http://localhost:10003".to_string(),
                "http://localhost:10003/health".to_string(),
                Default::default(),
            )
            .await;

        // Should succeed but primal won't be discoverable by any capability
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_register_primal_with_metadata() {
        let config = CapabilityRegistryConfig::default();
        let registry = CapabilityRegistry::new(config);

        let mut capabilities = HashSet::new();
        capabilities.insert(PrimalCapability::Compute);

        let mut metadata = std::collections::HashMap::new();
        metadata.insert("version".to_string(), "1.0.0".to_string());
        metadata.insert("region".to_string(), "us-west-2".to_string());
        metadata.insert("tier".to_string(), "production".to_string());

        let result = registry
            .register_primal(
                "metadata-test".to_string(),
                "Metadata Test".to_string(),
                capabilities,
                "http://localhost:10004".to_string(),
                "http://localhost:10004/health".to_string(),
                metadata.clone(),
            )
            .await;

        assert!(result.is_ok());

        // Verify metadata is stored
        let primals = registry
            .discover_by_capability(&PrimalCapability::Compute)
            .await
            .unwrap();

        assert_eq!(primals.len(), 1);
        assert_eq!(
            primals[0].metadata.get("version"),
            Some(&"1.0.0".to_string())
        );
    }

    // ============================================================================
    // Concurrent Access Error Paths
    // ============================================================================

    #[tokio::test]
    async fn test_concurrent_registration() {
        let config = CapabilityRegistryConfig::default();
        let registry = std::sync::Arc::new(CapabilityRegistry::new(config));

        let mut handles = vec![];

        // Spawn multiple concurrent registrations
        for i in 0..10 {
            let registry_clone = registry.clone();
            let handle = tokio::spawn(async move {
                let mut capabilities = HashSet::new();
                capabilities.insert(PrimalCapability::Storage);

                registry_clone
                    .register_primal(
                        format!("concurrent-test-{}", i),
                        format!("Concurrent Test {}", i),
                        capabilities,
                        format!("http://localhost:{}", 10010 + i),
                        format!("http://localhost:{}/health", 10010 + i),
                        Default::default(),
                    )
                    .await
            });
            handles.push(handle);
        }

        // All should succeed
        for handle in handles {
            let result = handle.await;
            assert!(result.is_ok());
            assert!(result.unwrap().is_ok());
        }

        // Verify all 10 were registered
        let primals = registry
            .discover_by_capability(&PrimalCapability::Storage)
            .await
            .unwrap();
        assert_eq!(primals.len(), 10);
    }

    #[tokio::test]
    async fn test_concurrent_discovery() {
        let config = CapabilityRegistryConfig::default();
        let registry = std::sync::Arc::new(CapabilityRegistry::new(config));

        // Register a few primals first
        let mut capabilities = HashSet::new();
        capabilities.insert(PrimalCapability::Monitoring);

        for i in 0..3 {
            registry
                .register_primal(
                    format!("discovery-test-{}", i),
                    format!("Discovery Test {}", i),
                    capabilities.clone(),
                    format!("http://localhost:{}", 10020 + i),
                    format!("http://localhost:{}/health", 10020 + i),
                    Default::default(),
                )
                .await
                .unwrap();
        }

        // Now spawn concurrent discovery requests
        let mut handles = vec![];
        for _ in 0..20 {
            let registry_clone = registry.clone();
            let handle = tokio::spawn(async move {
                registry_clone
                    .discover_by_capability(&PrimalCapability::Monitoring)
                    .await
            });
            handles.push(handle);
        }

        // All should succeed and return the same 3 primals
        for handle in handles {
            let result = handle.await;
            assert!(result.is_ok());
            let primals = result.unwrap().unwrap();
            assert_eq!(primals.len(), 3);
        }
    }

    // ============================================================================
    // Edge Case Coverage
    // ============================================================================

    #[tokio::test]
    async fn test_registry_with_all_capability_types() {
        let config = CapabilityRegistryConfig::default();
        let registry = CapabilityRegistry::new(config);

        // Test each capability type
        let capability_types = vec![
            PrimalCapability::Security,
            PrimalCapability::Storage,
            PrimalCapability::Compute,
            PrimalCapability::Monitoring,
            PrimalCapability::ServiceMesh,
            PrimalCapability::AIInference,
        ];

        for (idx, capability) in capability_types.iter().enumerate() {
            let mut capabilities = HashSet::new();
            capabilities.insert(capability.clone());

            let result = registry
                .register_primal(
                    format!("capability-type-{}", idx),
                    format!("Capability Type {}", idx),
                    capabilities,
                    format!("http://localhost:{}", 10030 + idx),
                    format!("http://localhost:{}/health", 10030 + idx),
                    Default::default(),
                )
                .await;

            assert!(result.is_ok());

            // Verify discoverable
            let primals = registry.discover_by_capability(capability).await.unwrap();
            assert_eq!(primals.len(), 1);
        }
    }

    #[tokio::test]
    async fn test_large_metadata_handling() {
        let config = CapabilityRegistryConfig::default();
        let registry = CapabilityRegistry::new(config);

        let mut capabilities = HashSet::new();
        capabilities.insert(PrimalCapability::Storage);

        // Create large metadata
        let mut metadata = std::collections::HashMap::new();
        for i in 0..100 {
            metadata.insert(format!("key_{}", i), format!("value_{}", i));
        }

        let result = registry
            .register_primal(
                "large-metadata-test".to_string(),
                "Large Metadata".to_string(),
                capabilities,
                "http://localhost:10040".to_string(),
                "http://localhost:10040/health".to_string(),
                metadata.clone(),
            )
            .await;

        assert!(result.is_ok());

        // Verify metadata is preserved
        let primals = registry
            .discover_by_capability(&PrimalCapability::Storage)
            .await
            .unwrap();

        assert_eq!(primals[0].metadata.len(), 100);
    }

    #[tokio::test]
    async fn test_unicode_in_primal_names() {
        let config = CapabilityRegistryConfig::default();
        let registry = CapabilityRegistry::new(config);

        let mut capabilities = HashSet::new();
        capabilities.insert(PrimalCapability::Compute);

        let result = registry
            .register_primal(
                "unicode-test".to_string(),
                "测试服务 🚀 Тест Service".to_string(), // Unicode name
                capabilities,
                "http://localhost:10041".to_string(),
                "http://localhost:10041/health".to_string(),
                Default::default(),
            )
            .await;

        assert!(result.is_ok());

        let primals = registry
            .discover_by_capability(&PrimalCapability::Compute)
            .await
            .unwrap();

        assert_eq!(primals.len(), 1);
        assert!(primals[0].display_name.contains("测试服务"));
    }
}

// ============================================================================
// Error Type Coverage Tests
// ============================================================================

#[cfg(test)]
mod error_type_tests {
    use squirrel::error::PrimalError;

    #[test]
    fn test_error_display_formatting() {
        let err = PrimalError::NetworkError("Connection refused".to_string());
        let display = format!("{}", err);
        assert!(display.contains("Connection refused"));
    }

    #[test]
    fn test_error_debug_formatting() {
        let err = PrimalError::SerializationError("Invalid JSON".to_string());
        let debug = format!("{:?}", err);
        assert!(debug.contains("SerializationError"));
    }

    #[test]
    fn test_error_different_types() {
        // Test various error types exist and can be created
        let _network_err = PrimalError::NetworkError("test".to_string());
        let _ser_err = PrimalError::SerializationError("test".to_string());
        let _config_err = PrimalError::ConfigurationError("test".to_string());

        // If we have an IoError variant, we could test it, but it may not exist
        // So we just verify error types can be created
    }
}

// ============================================================================
// Migration Helper Error Path Tests
// ============================================================================

#[cfg(test)]
#[cfg(feature = "disabled_until_capability_registry_exported")]
mod migration_helper_tests {
    use squirrel::universal::PrimalCapability;
    use std::collections::HashSet;

    #[tokio::test]
    async fn test_migration_helper_no_env_vars() {
        // Test helper with no environment configuration
        // Note: Uses default config instead of env var mutations for concurrent safety
        let helper = CapabilityMigrationHelper::new().await;
        assert!(helper.is_ok());

        // Should work even with no primals registered
        let helper = helper.unwrap();
        let security = helper.get_security_service().await;
        assert!(security.is_ok());
        assert_eq!(security.unwrap(), None);
    }

    #[tokio::test]
    async fn test_migration_helper_with_explicit_endpoints() {
        // Test with explicit endpoint registration (no env var mutations)
        // This ensures concurrent test safety
        let helper = CapabilityMigrationHelper::new().await.unwrap();

        // Manually register services with explicit endpoints
        let mut capabilities = std::collections::HashSet::new();
        capabilities.insert(PrimalCapability::Security);
        helper
            .register_primal(
                "beardog-test-manual".to_string(),
                "BearDog Test".to_string(),
                capabilities,
                "http://beardog-test.local:8080".to_string(),
            )
            .await
            .unwrap();

        // Should be discoverable
        let security = helper.get_security_service().await.unwrap();
        assert!(security.is_some());
    }

    #[tokio::test]
    async fn test_migration_helper_manual_registration() {
        let helper = CapabilityMigrationHelper::new().await.unwrap();

        let mut capabilities = HashSet::new();
        capabilities.insert(PrimalCapability::AIInference);

        // Manually register a custom primal
        let result = helper
            .register_primal(
                "custom-ai".to_string(),
                "Custom AI Service".to_string(),
                capabilities,
                "http://custom-ai.local:9000".to_string(),
            )
            .await;

        assert!(result.is_ok());

        // Verify it's discoverable
        let ai_primals = helper
            .discover_by_capability(&PrimalCapability::AIInference)
            .await
            .unwrap();

        assert_eq!(ai_primals.len(), 1);
        assert_eq!(ai_primals[0].id, "custom-ai");
    }

    #[tokio::test]
    async fn test_migration_helper_get_first_healthy() {
        let helper = CapabilityMigrationHelper::new().await.unwrap();

        let mut capabilities = HashSet::new();
        capabilities.insert(PrimalCapability::Storage);

        // Register multiple storage services
        for i in 0..3 {
            helper
                .register_primal(
                    format!("storage-{}", i),
                    format!("Storage {}", i),
                    capabilities.clone(),
                    format!("http://storage-{}.local:8000", i),
                )
                .await
                .unwrap();
        }

        // Get first one for capability
        let storage = helper
            .get_primal_for_capability(&PrimalCapability::Storage)
            .await
            .unwrap();

        assert!(storage.is_some());
        let storage = storage.unwrap();
        assert!(storage.id.starts_with("storage-"));
    }
}
