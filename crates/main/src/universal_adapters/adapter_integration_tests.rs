// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Integration tests for universal adapters
//!
//! Tests adapter coordination with service registry

#[cfg(test)]
mod tests {
    use super::super::compute_adapter::UniversalComputeAdapter;
    use super::super::registry::InMemoryServiceRegistry;
    use super::super::security_adapter::UniversalSecurityAdapter;
    use super::super::storage_adapter::UniversalStorageAdapter;
    use super::super::*;
    use chrono::Utc;
    use std::collections::HashMap;
    use std::sync::Arc;
    use uuid::Uuid;

    // ========================================================================
    // TEST HELPERS
    // ========================================================================

    fn create_test_registry() -> Arc<InMemoryServiceRegistry> {
        Arc::new(InMemoryServiceRegistry::new())
    }

    fn create_compute_service_registration() -> UniversalServiceRegistration {
        UniversalServiceRegistration {
            service_id: Uuid::new_v4(),
            metadata: ServiceMetadata {
                name: "test-compute".to_string(),
                category: ServiceCategory::Compute {
                    specialties: vec!["ml".to_string()],
                },
                version: "1.0.0".to_string(),
                description: "Test compute".to_string(),
                maintainer: "test@example.com".to_string(),
                protocols: vec!["http".to_string()],
            },
            capabilities: vec![ServiceCapability::Computation {
                types: vec!["batch".to_string()],
                resources: HashMap::new(),
                constraints: vec![],
            }],
            endpoints: vec![ServiceEndpoint {
                name: "api".to_string(),
                url: "http://localhost:8080".to_string(),
                protocol: "http".to_string(),
                port: Some(8080),
                path: Some("/api".to_string()),
            }],
            resources: ResourceSpec {
                cpu_cores: Some(8),
                memory_gb: Some(32),
                storage_gb: Some(500),
                network_bandwidth: Some(1000),
                custom_resources: HashMap::new(),
            },
            integration: IntegrationPreferences {
                preferred_protocols: vec!["http".to_string()],
                retry_policy: "exponential".to_string(),
                timeout_seconds: 30,
                load_balancing_weight: 10,
            },
            extensions: HashMap::new(),
            registration_timestamp: Utc::now(),
            service_version: "1.0.0".to_string(),
            instance_id: "test-1".to_string(),
            priority: 8,
        }
    }

    fn create_storage_service_registration() -> UniversalServiceRegistration {
        UniversalServiceRegistration {
            service_id: Uuid::new_v4(),
            metadata: ServiceMetadata {
                name: "test-storage".to_string(),
                category: ServiceCategory::Storage {
                    types: vec!["object".to_string()],
                },
                version: "1.0.0".to_string(),
                description: "Test storage".to_string(),
                maintainer: "test@example.com".to_string(),
                protocols: vec!["http".to_string()],
            },
            capabilities: vec![ServiceCapability::DataManagement {
                operations: vec!["store".to_string(), "retrieve".to_string()],
                consistency: "eventual".to_string(),
                durability: "high".to_string(),
            }],
            endpoints: vec![ServiceEndpoint {
                name: "api".to_string(),
                url: "http://localhost:8081".to_string(),
                protocol: "http".to_string(),
                port: Some(8081),
                path: Some("/api".to_string()),
            }],
            resources: ResourceSpec {
                cpu_cores: Some(4),
                memory_gb: Some(16),
                storage_gb: Some(1000),
                network_bandwidth: Some(1000),
                custom_resources: HashMap::new(),
            },
            integration: IntegrationPreferences {
                preferred_protocols: vec!["http".to_string()],
                retry_policy: "simple".to_string(),
                timeout_seconds: 60,
                load_balancing_weight: 5,
            },
            extensions: HashMap::new(),
            registration_timestamp: Utc::now(),
            service_version: "1.0.0".to_string(),
            instance_id: "test-2".to_string(),
            priority: 7,
        }
    }

    fn create_security_service_registration() -> UniversalServiceRegistration {
        UniversalServiceRegistration {
            service_id: Uuid::new_v4(),
            metadata: ServiceMetadata {
                name: "test-security".to_string(),
                category: ServiceCategory::Security {
                    domains: vec!["auth".to_string()],
                },
                version: "1.0.0".to_string(),
                description: "Test security".to_string(),
                maintainer: "test@example.com".to_string(),
                protocols: vec!["https".to_string()],
            },
            capabilities: vec![ServiceCapability::Security {
                functions: vec!["auth".to_string()],
                compliance: vec!["soc2".to_string()],
                trust_levels: vec!["high".to_string()],
            }],
            endpoints: vec![ServiceEndpoint {
                name: "api".to_string(),
                url: "https://localhost:8082".to_string(),
                protocol: "https".to_string(),
                port: Some(8082),
                path: Some("/api".to_string()),
            }],
            resources: ResourceSpec {
                cpu_cores: Some(2),
                memory_gb: Some(8),
                storage_gb: Some(100),
                network_bandwidth: Some(500),
                custom_resources: HashMap::new(),
            },
            integration: IntegrationPreferences {
                preferred_protocols: vec!["https".to_string()],
                retry_policy: "simple".to_string(),
                timeout_seconds: 15,
                load_balancing_weight: 10,
            },
            extensions: HashMap::new(),
            registration_timestamp: Utc::now(),
            service_version: "1.0.0".to_string(),
            instance_id: "test-3".to_string(),
            priority: 9,
        }
    }

    // ========================================================================
    // COMPUTE ADAPTER TESTS
    // ========================================================================

    #[test]
    fn test_compute_adapter_creation() {
        let registry = create_test_registry();
        let _adapter = UniversalComputeAdapter::new(registry);
        // Creation should succeed
    }

    #[tokio::test]
    async fn test_compute_adapter_with_registered_service() {
        let registry = create_test_registry();
        let service = create_compute_service_registration();

        registry.register_service(service).await.unwrap();

        let _adapter = UniversalComputeAdapter::new(registry.clone());

        // Verify service is registered
        let services = registry.list_all_services().await.unwrap();
        assert_eq!(services.len(), 1);
    }

    // ========================================================================
    // STORAGE ADAPTER TESTS
    // ========================================================================

    #[test]
    fn test_storage_adapter_creation() {
        let registry = create_test_registry();
        let _adapter = UniversalStorageAdapter::new(registry);
        // Creation should succeed
    }

    #[tokio::test]
    async fn test_storage_adapter_with_registered_service() {
        let registry = create_test_registry();
        let service = create_storage_service_registration();

        registry.register_service(service).await.unwrap();

        let _adapter = UniversalStorageAdapter::new(registry.clone());

        // Verify service is registered
        let services = registry.list_all_services().await.unwrap();
        assert_eq!(services.len(), 1);
    }

    // ========================================================================
    // SECURITY ADAPTER TESTS
    // ========================================================================

    #[test]
    fn test_security_adapter_creation() {
        let registry = create_test_registry();
        let _adapter = UniversalSecurityAdapter::new(registry);
        // Creation should succeed
    }

    #[tokio::test]
    async fn test_security_adapter_with_registered_service() {
        let registry = create_test_registry();
        let service = create_security_service_registration();

        registry.register_service(service).await.unwrap();

        let _adapter = UniversalSecurityAdapter::new(registry.clone());

        // Verify service is registered
        let services = registry.list_all_services().await.unwrap();
        assert_eq!(services.len(), 1);
    }

    // ========================================================================
    // REGISTRY INTERACTION TESTS
    // ========================================================================

    #[tokio::test]
    async fn test_registry_with_multiple_services() {
        let registry = create_test_registry();

        // Register all types of services
        registry
            .register_service(create_compute_service_registration())
            .await
            .unwrap();
        registry
            .register_service(create_storage_service_registration())
            .await
            .unwrap();
        registry
            .register_service(create_security_service_registration())
            .await
            .unwrap();

        let services = registry.list_all_services().await.unwrap();
        assert_eq!(services.len(), 3);
    }

    #[tokio::test]
    async fn test_capability_based_discovery() {
        let registry = create_test_registry();

        // Register compute service
        registry
            .register_service(create_compute_service_registration())
            .await
            .unwrap();

        // Discover by computation capability
        let compute_capability = ServiceCapability::Computation {
            types: vec!["batch".to_string()],
            resources: HashMap::new(),
            constraints: vec![],
        };

        let services = registry
            .discover_by_capability(compute_capability)
            .await
            .unwrap();
        assert!(!services.is_empty());
    }

    #[tokio::test]
    async fn test_category_based_discovery() {
        let registry = create_test_registry();

        // Register storage service
        registry
            .register_service(create_storage_service_registration())
            .await
            .unwrap();

        // Discover by category
        let services = registry.discover_by_category("Storage").await.unwrap();
        assert!(!services.is_empty());
    }

    #[tokio::test]
    async fn test_service_health_update() {
        let registry = create_test_registry();
        let service = create_compute_service_registration();
        let service_id = service.service_id.to_string();

        registry.register_service(service).await.unwrap();

        // Update health
        let health = ServiceHealth {
            healthy: true,
            message: Some("Operational".to_string()),
            metrics: HashMap::new(),
        };

        let result = registry.update_service_health(&service_id, health).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_service_deregistration() {
        let registry = create_test_registry();
        let service = create_compute_service_registration();
        let service_id = service.service_id.to_string();

        registry.register_service(service).await.unwrap();

        // Verify registered
        let services = registry.list_all_services().await.unwrap();
        assert_eq!(services.len(), 1);

        // Deregister
        registry.deregister_service(&service_id).await.unwrap();

        // Verify deregistered
        let services = registry.list_all_services().await.unwrap();
        assert_eq!(services.len(), 0);
    }

    // ========================================================================
    // SERVICE ENDPOINT TESTS
    // ========================================================================

    #[test]
    fn test_service_endpoint_creation() {
        let endpoint = ServiceEndpoint {
            name: "main".to_string(),
            url: "http://localhost:8080".to_string(),
            protocol: "http".to_string(),
            port: Some(8080),
            path: Some("/api".to_string()),
        };

        assert_eq!(endpoint.name, "main");
        assert_eq!(endpoint.url, "http://localhost:8080");
        assert_eq!(endpoint.port, Some(8080));
    }

    #[test]
    fn test_service_endpoint_https() {
        let endpoint = ServiceEndpoint {
            name: "secure".to_string(),
            url: "https://api.example.com".to_string(),
            protocol: "https".to_string(),
            port: Some(443),
            path: Some("/v1/api".to_string()),
        };

        assert!(endpoint.url.starts_with("https://"));
        assert_eq!(endpoint.protocol, "https");
        assert_eq!(endpoint.port, Some(443));
    }

    // ========================================================================
    // RESOURCE SPEC TESTS
    // ========================================================================

    #[test]
    fn test_resource_spec_full() {
        let resources = ResourceSpec {
            cpu_cores: Some(16),
            memory_gb: Some(64),
            storage_gb: Some(1000),
            network_bandwidth: Some(10000),
            custom_resources: HashMap::new(),
        };

        assert_eq!(resources.cpu_cores, Some(16));
        assert_eq!(resources.memory_gb, Some(64));
    }

    #[test]
    fn test_resource_spec_with_custom() {
        let mut custom = HashMap::new();
        custom.insert("gpu_count".to_string(), serde_json::json!(4));

        let resources = ResourceSpec {
            cpu_cores: Some(8),
            memory_gb: Some(32),
            storage_gb: Some(500),
            network_bandwidth: Some(1000),
            custom_resources: custom,
        };

        assert_eq!(resources.custom_resources.len(), 1);
    }

    // ========================================================================
    // SERVICE CAPABILITY TESTS
    // ========================================================================

    #[test]
    fn test_service_capability_computation() {
        let cap = ServiceCapability::Computation {
            types: vec!["batch".to_string()],
            resources: HashMap::new(),
            constraints: vec![],
        };

        assert!(matches!(cap, ServiceCapability::Computation { .. }));
    }

    #[test]
    fn test_service_capability_data_management() {
        let cap = ServiceCapability::DataManagement {
            operations: vec!["store".to_string()],
            consistency: "eventual".to_string(),
            durability: "high".to_string(),
        };

        assert!(matches!(cap, ServiceCapability::DataManagement { .. }));
    }

    #[test]
    fn test_service_capability_security() {
        let cap = ServiceCapability::Security {
            functions: vec!["auth".to_string()],
            compliance: vec!["soc2".to_string()],
            trust_levels: vec!["high".to_string()],
        };

        assert!(matches!(cap, ServiceCapability::Security { .. }));
    }

    #[test]
    fn test_service_capability_coordination() {
        let cap = ServiceCapability::Coordination {
            patterns: vec!["orchestration".to_string()],
            consistency: "strong".to_string(),
            fault_tolerance: "high".to_string(),
        };

        assert!(matches!(cap, ServiceCapability::Coordination { .. }));
    }

    // ========================================================================
    // SERVICE CATEGORY TESTS
    // ========================================================================

    #[test]
    fn test_service_category_compute() {
        let category = ServiceCategory::Compute {
            specialties: vec!["ml".to_string(), "batch".to_string()],
        };

        assert!(matches!(category, ServiceCategory::Compute { .. }));
    }

    #[test]
    fn test_service_category_storage() {
        let category = ServiceCategory::Storage {
            types: vec!["object".to_string()],
        };

        assert!(matches!(category, ServiceCategory::Storage { .. }));
    }

    #[test]
    fn test_service_category_security() {
        let category = ServiceCategory::Security {
            domains: vec!["auth".to_string()],
        };

        assert!(matches!(category, ServiceCategory::Security { .. }));
    }

    // ========================================================================
    // INTEGRATION PREFERENCES TESTS
    // ========================================================================

    #[test]
    fn test_integration_preferences_creation() {
        let prefs = IntegrationPreferences {
            preferred_protocols: vec!["tarpc".to_string(), "http".to_string()],
            retry_policy: "exponential_backoff".to_string(),
            timeout_seconds: 45,
            load_balancing_weight: 7,
        };

        assert_eq!(prefs.preferred_protocols.len(), 2);
        assert_eq!(prefs.timeout_seconds, 45);
    }

    // ========================================================================
    // COMPLETE REGISTRATION WORKFLOW TESTS
    // ========================================================================

    #[tokio::test]
    async fn test_complete_service_registration_workflow() {
        let registry = create_test_registry();

        // 1. Register service
        let service = create_compute_service_registration();
        let service_id = service.service_id.to_string();
        registry.register_service(service).await.unwrap();

        // 2. Verify registration
        let services = registry.list_all_services().await.unwrap();
        assert_eq!(services.len(), 1);

        // 3. Update health
        let health = ServiceHealth {
            healthy: true,
            message: Some("Running".to_string()),
            metrics: HashMap::new(),
        };
        registry
            .update_service_health(&service_id, health)
            .await
            .unwrap();

        // 4. Deregister
        registry.deregister_service(&service_id).await.unwrap();

        // 5. Verify deregistration
        let services = registry.list_all_services().await.unwrap();
        assert_eq!(services.len(), 0);
    }

    #[tokio::test]
    async fn test_concurrent_service_registration() {
        let registry = create_test_registry();

        // Register multiple services concurrently
        let handles: Vec<_> = (0..5)
            .map(|_| {
                let reg = registry.clone();
                tokio::spawn(async move {
                    let service = create_compute_service_registration();
                    reg.register_service(service).await
                })
            })
            .collect();

        for handle in handles {
            assert!(handle.await.unwrap().is_ok());
        }

        let services = registry.list_all_services().await.unwrap();
        assert_eq!(services.len(), 5);
    }

    #[tokio::test]
    async fn test_service_priority_handling() {
        let registry = create_test_registry();

        // Register services with different priorities
        let mut service1 = create_compute_service_registration();
        service1.priority = 5;

        let mut service2 = create_compute_service_registration();
        service2.priority = 10;

        registry.register_service(service1).await.unwrap();
        registry.register_service(service2).await.unwrap();

        let services = registry.list_all_services().await.unwrap();
        assert_eq!(services.len(), 2);
    }

    // ========================================================================
    // MULTI-ADAPTER INTEGRATION TESTS
    // ========================================================================

    #[tokio::test]
    async fn test_multiple_adapters_same_registry() {
        let registry = create_test_registry();

        // Create all adapter types
        let _compute = UniversalComputeAdapter::new(registry.clone());
        let _storage = UniversalStorageAdapter::new(registry.clone());
        let _security = UniversalSecurityAdapter::new(registry.clone());

        // Register services
        registry
            .register_service(create_compute_service_registration())
            .await
            .unwrap();
        registry
            .register_service(create_storage_service_registration())
            .await
            .unwrap();
        registry
            .register_service(create_security_service_registration())
            .await
            .unwrap();

        // All adapters share the same registry
        let services = registry.list_all_services().await.unwrap();
        assert_eq!(services.len(), 3);
    }

    #[tokio::test]
    async fn test_adapter_service_discovery_isolation() {
        let registry = create_test_registry();

        // Register different service types
        registry
            .register_service(create_compute_service_registration())
            .await
            .unwrap();
        registry
            .register_service(create_storage_service_registration())
            .await
            .unwrap();

        // Discover compute services only
        let compute_services = registry.discover_by_category("Compute").await.unwrap();
        assert_eq!(compute_services.len(), 1);

        // Discover storage services only
        let storage_services = registry.discover_by_category("Storage").await.unwrap();
        assert_eq!(storage_services.len(), 1);
    }

    // ========================================================================
    // SERVICE METADATA TESTS
    // ========================================================================

    #[test]
    fn test_service_metadata_creation() {
        let metadata = ServiceMetadata {
            name: "test-service".to_string(),
            category: ServiceCategory::Compute {
                specialties: vec!["ml".to_string()],
            },
            version: "1.0.0".to_string(),
            description: "Test".to_string(),
            maintainer: "test@example.com".to_string(),
            protocols: vec!["http".to_string()],
        };

        assert_eq!(metadata.name, "test-service");
        assert_eq!(metadata.version, "1.0.0");
    }

    #[tokio::test]
    async fn test_service_registration_with_extensions() {
        let registry = create_test_registry();

        let mut service = create_compute_service_registration();
        service
            .extensions
            .insert("custom_field".to_string(), serde_json::json!("value"));
        service
            .extensions
            .insert("priority_boost".to_string(), serde_json::json!(true));

        registry.register_service(service).await.unwrap();

        let services = registry.list_all_services().await.unwrap();
        assert_eq!(services.len(), 1);
        assert_eq!(services[0].metadata.len(), 2);
    }

    #[tokio::test]
    async fn test_service_health_status() {
        let registry = create_test_registry();
        let service = create_compute_service_registration();
        let service_id = service.service_id.to_string();

        registry.register_service(service).await.unwrap();

        // Test healthy status
        let health = ServiceHealth {
            healthy: true,
            message: None,
            metrics: HashMap::new(),
        };
        registry
            .update_service_health(&service_id, health)
            .await
            .unwrap();

        // Test unhealthy status
        let unhealthy = ServiceHealth {
            healthy: false,
            message: Some("Service degraded".to_string()),
            metrics: HashMap::new(),
        };
        registry
            .update_service_health(&service_id, unhealthy)
            .await
            .unwrap();
    }

    // ========================================================================
    // ADAPTER CREATION PATTERN TESTS
    // ========================================================================

    #[tokio::test]
    async fn test_adapter_creation_pattern_compute() {
        let registry = create_test_registry();

        // Register service first
        registry
            .register_service(create_compute_service_registration())
            .await
            .unwrap();

        // Then create adapter
        let _adapter = UniversalComputeAdapter::new(registry.clone());

        // Pattern works correctly
        let services = registry.list_all_services().await.unwrap();
        assert_eq!(services.len(), 1);
    }

    #[tokio::test]
    async fn test_adapter_creation_pattern_storage() {
        let registry = create_test_registry();

        // Register service first
        registry
            .register_service(create_storage_service_registration())
            .await
            .unwrap();

        // Then create adapter
        let _adapter = UniversalStorageAdapter::new(registry.clone());

        // Pattern works correctly
        let services = registry.list_all_services().await.unwrap();
        assert_eq!(services.len(), 1);
    }

    #[tokio::test]
    async fn test_adapter_creation_pattern_security() {
        let registry = create_test_registry();

        // Register service first
        registry
            .register_service(create_security_service_registration())
            .await
            .unwrap();

        // Then create adapter
        let _adapter = UniversalSecurityAdapter::new(registry.clone());

        // Pattern works correctly
        let services = registry.list_all_services().await.unwrap();
        assert_eq!(services.len(), 1);
    }
}
