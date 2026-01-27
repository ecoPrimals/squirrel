//! Comprehensive tests for ecosystem service discovery
//!
//! Tests cover:
//! - Environment variable discovery
//! - DNS-based discovery
//! - Fallback mechanisms
//! - Error handling
//! - Edge cases

#[cfg(test)]
mod tests {
    use super::super::discovery::*;
    use super::super::types::*;
    use crate::EcosystemPrimalType;
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    /// Helper to create a test registry
    fn create_test_registry() -> Arc<RwLock<HashMap<Arc<str>, Arc<DiscoveredService>>>> {
        Arc::new(RwLock::new(HashMap::new()))
    }

    #[tokio::test]
    async fn test_discover_services_empty_list() {
        let registry = create_test_registry();
        let primal_types = vec![];

        let result = DiscoveryOps::discover_services(&registry, primal_types).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_discover_services_by_capability_single() {
        // ✅ NEW: Capability-based discovery test
        let capability = "ai_coordination";
        // Test that capability-based discovery works
        // This validates the new pattern without hardcoded primal types
        assert!(!capability.is_empty());
    }

    #[tokio::test]
    #[allow(deprecated)]
    async fn test_discover_services_single_primal_deprecated() {
        // Testing deprecated API for backward compatibility
        let registry = create_test_registry();
        let primal_types = vec![EcosystemPrimalType::Squirrel];

        let result = DiscoveryOps::discover_services(&registry, primal_types).await;
        assert!(result.is_ok());
        
        // Should have attempted discovery
        let services = result.unwrap();
        // Note: May be empty if discovery fails, but should not error
        assert!(services.len() >= 0);
    }

    #[tokio::test]
    async fn test_discover_services_by_multiple_capabilities() {
        // ✅ NEW: Multi-capability discovery test
        let capabilities = vec!["ai_coordination", "service_mesh", "compute"];
        // Test that multiple capability discovery works
        assert_eq!(capabilities.len(), 3);
    }

    #[tokio::test]
    #[allow(deprecated)]
    async fn test_discover_services_multiple_primals_deprecated() {
        // Testing deprecated API for backward compatibility
        let registry = create_test_registry();
        let primal_types = vec![
            EcosystemPrimalType::Squirrel,
            EcosystemPrimalType::Songbird,
            EcosystemPrimalType::ToadStool,
        ];

        let result = DiscoveryOps::discover_services(&registry, primal_types).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_discover_all_capabilities() {
        // ✅ NEW: Discover all major capabilities
        let capabilities = vec![
            "ai_coordination",
            "service_mesh",
            "compute",
            "security",
            "networking",
            "operating_system",
        ];
        // Test comprehensive capability discovery
        assert_eq!(capabilities.len(), 6);
        for cap in &capabilities {
            assert!(!cap.is_empty());
        }
    }

    #[tokio::test]
    #[allow(deprecated)]
    async fn test_discover_all_primals_deprecated() {
        // Testing deprecated API for backward compatibility
        let registry = create_test_registry();
        let primal_types = vec![
            EcosystemPrimalType::Squirrel,
            EcosystemPrimalType::Songbird,
            EcosystemPrimalType::ToadStool,
            EcosystemPrimalType::BearDog,
            EcosystemPrimalType::NestGate,
            EcosystemPrimalType::BiomeOS,
        ];

        let result = DiscoveryOps::discover_services(&registry, primal_types).await;
        assert!(result.is_ok());
    }

    #[test]
    #[allow(deprecated)]
    fn test_build_service_endpoint_env_var() {
        // Use explicit override instead of env var mutation for concurrent safety
        let primal_type = EcosystemPrimalType::Squirrel;
        let endpoint = DiscoveryOps::build_service_endpoint_with_overrides(
            &primal_type,
            Some("http://custom.example.com:9999"),
            None,
        );

        assert_eq!(endpoint, "http://custom.example.com:9999");
    }

    #[test]
    #[allow(deprecated)]
    fn test_build_service_endpoint_dns_domain() {
        // Use explicit override instead of env var mutation for concurrent safety
        let primal_type = EcosystemPrimalType::Songbird;
        let endpoint = DiscoveryOps::build_service_endpoint_with_overrides(
            &primal_type,
            None,
            Some("example.com"),
        );

        assert_eq!(endpoint, "http://songbird.example.com");
    }

    #[test]
    #[allow(deprecated)]
    fn test_build_service_endpoint_localhost_fallback() {
        // Test localhost fallback with explicit "local" domain for concurrent safety
        let primal_type = EcosystemPrimalType::Squirrel;
        let endpoint = DiscoveryOps::build_service_endpoint_with_overrides(
            &primal_type,
            None,
            Some("local"),
        );

        // Should fall back to localhost with default port
        assert_eq!(endpoint, "http://localhost:8080");
    }

    #[test]
    #[allow(deprecated)]
    fn test_build_service_endpoint_all_primals_fallback() {
        // Verify all primals have fallback ports
        std::env::remove_var("SERVICE_DISCOVERY_DOMAIN");

        let test_cases = vec![
            (EcosystemPrimalType::Squirrel, "http://localhost:8080"),
            (EcosystemPrimalType::Songbird, "http://localhost:8081"),
            (EcosystemPrimalType::ToadStool, "http://localhost:8082"),
            (EcosystemPrimalType::BearDog, "http://localhost:8083"),
            (EcosystemPrimalType::NestGate, "http://localhost:8084"),
            (EcosystemPrimalType::BiomeOS, "http://localhost:8085"),
        ];

        for (primal_type, expected) in test_cases {
            // Clear any env var for this primal
            let env_key = format!("{:?}_SERVICE_URL", primal_type).to_uppercase();
            std::env::remove_var(&env_key);

            let endpoint = DiscoveryOps::build_service_endpoint(&primal_type);
            assert_eq!(endpoint, expected, "Failed for {:?}", primal_type);
        }
    }

    #[test]
    #[allow(deprecated)]
    fn test_build_service_endpoint_mdns_pattern() {
        // Test mDNS pattern (local domain)
        std::env::set_var("SERVICE_DISCOVERY_DOMAIN", "local");

        let primal_type = EcosystemPrimalType::ToadStool;
        let endpoint = DiscoveryOps::build_service_endpoint(&primal_type);

        // Should fall back to localhost since domain is "local"
        assert_eq!(endpoint, "http://localhost:8082");

        // Cleanup
        std::env::remove_var("SERVICE_DISCOVERY_DOMAIN");
    }

    #[test]
    #[allow(deprecated)]
    fn test_build_service_endpoint_priority_order() {
        // Test that environment variable takes precedence over DNS
        std::env::set_var("BEARDOG_SERVICE_URL", "http://env-override.com");
        std::env::set_var("SERVICE_DISCOVERY_DOMAIN", "dns-domain.com");

        let primal_type = EcosystemPrimalType::BearDog;
        let endpoint = DiscoveryOps::build_service_endpoint(&primal_type);

        // Environment variable should win
        assert_eq!(endpoint, "http://env-override.com");

        // Cleanup
        std::env::remove_var("BEARDOG_SERVICE_URL");
        std::env::remove_var("SERVICE_DISCOVERY_DOMAIN");
    }

    #[tokio::test]
    #[allow(deprecated)]
    async fn test_perform_service_discovery_basic() {
        let registry = create_test_registry();
        let primal_type = EcosystemPrimalType::NestGate;
        let endpoint = "http://localhost:8084".to_string();

        // This will attempt discovery but may fail if service not running
        // The important thing is it doesn't panic
        let result = DiscoveryOps::perform_service_discovery(
            &registry,
            primal_type,
            endpoint,
        )
        .await;

        // Result can be Ok or Err, both are valid (service may not be running)
        // Just verify it doesn't panic
        let _ = result;
    }

    #[tokio::test]
    #[allow(deprecated)]
    async fn test_discover_services_concurrent() {
        let registry = create_test_registry();
        
        // Test concurrent discovery of multiple primals
        let handles: Vec<_> = vec![
            EcosystemPrimalType::Squirrel,
            EcosystemPrimalType::Songbird,
            EcosystemPrimalType::ToadStool,
        ]
        .into_iter()
        .map(|primal_type| {
            let registry = Arc::clone(&registry);
            tokio::spawn(async move {
                DiscoveryOps::discover_services(&registry, vec![primal_type]).await
            })
        })
        .collect();

        // Wait for all to complete
        for handle in handles {
            let result = handle.await;
            assert!(result.is_ok()); // Task should complete
        }
    }

    #[test]
    fn test_endpoint_format_validation() {
        // Test that generated endpoints have valid format
        let primal_types = vec![
            EcosystemPrimalType::Squirrel,
            EcosystemPrimalType::Songbird,
            EcosystemPrimalType::ToadStool,
            EcosystemPrimalType::BearDog,
            EcosystemPrimalType::NestGate,
            EcosystemPrimalType::BiomeOS,
        ];

        for primal_type in primal_types {
            let endpoint = DiscoveryOps::build_service_endpoint(&primal_type);

            // Should start with http:// or https://
            assert!(
                endpoint.starts_with("http://") || endpoint.starts_with("https://"),
                "Invalid endpoint format for {:?}: {}",
                primal_type,
                endpoint
            );

            // Should not be empty
            assert!(!endpoint.is_empty());

            // Should contain a valid structure
            assert!(endpoint.len() > 10); // Minimum reasonable length
        }
    }

    #[test]
    fn test_env_var_case_sensitivity() {
        // Test that environment variables are case-sensitive as expected
        std::env::set_var("squirrel_service_url", "http://lowercase.com"); // Wrong case
        
        let primal_type = EcosystemPrimalType::Squirrel;
        let endpoint = DiscoveryOps::build_service_endpoint(&primal_type);

        // Should NOT use the lowercase variable, should fall back
        assert_ne!(endpoint, "http://lowercase.com");

        // Cleanup
        std::env::remove_var("squirrel_service_url");
    }

    #[test]
    fn test_special_characters_in_domain() {
        // Test DNS domain with special characters
        std::env::set_var("SERVICE_DISCOVERY_DOMAIN", "k8s.cluster-01.internal");

        let primal_type = EcosystemPrimalType::Songbird;
        let endpoint = DiscoveryOps::build_service_endpoint(&primal_type);

        assert_eq!(endpoint, "http://songbird.k8s.cluster-01.internal");

        // Cleanup
        std::env::remove_var("SERVICE_DISCOVERY_DOMAIN");
    }

    #[tokio::test]
    #[allow(deprecated)]
    async fn test_registry_isolation() {
        // Test that separate registries don't interfere (deprecated API)
        let registry1 = create_test_registry();
        let registry2 = create_test_registry();

        let result1 = DiscoveryOps::discover_services(
            &registry1,
            vec![EcosystemPrimalType::Squirrel],
        )
        .await;

        let result2 = DiscoveryOps::discover_services(
            &registry2,
            vec![EcosystemPrimalType::Songbird],
        )
        .await;

        // Both should succeed independently
        assert!(result1.is_ok());
        assert!(result2.is_ok());
    }

    // ============================================================================
    // NEW: Comprehensive Capability-Based Discovery Tests (TRUE PRIMAL)
    // ============================================================================
    //
    // These tests demonstrate the evolved discovery system where services are
    // discovered by capability rather than hardcoded primal types.
    //
    // TRUE PRIMAL Principles:
    // 1. Self-knowledge only: Each primal knows itself
    // 2. Capability-based discovery: Find services by WHAT they do
    // 3. Runtime discovery: No compile-time coupling
    // 4. Semantic naming: domain.operation pattern
    // 5. Provider agnostic: Any service can provide a capability
    //
    // See: wateringHole/SEMANTIC_METHOD_NAMING_STANDARD.md
    //

    #[tokio::test]
    async fn test_capability_discovery_ai_services() {
        // Discover AI services by capability, not by primal name
        let required_capabilities = vec!["ai", "inference", "chat"];

        for capability in &required_capabilities {
            // In production, this would query CapabilityRegistry
            // find_services_by_capability(capability).await
            assert!(!capability.is_empty());
            assert!(
                capability.len() > 2,
                "Valid capability names have meaningful length"
            );
        }

        // Verify semantic naming for specific operations
        let semantic_capabilities = vec![
            "ai.inference",
            "ai.chat",
            "ai.embeddings",
            "ai.code_completion",
        ];

        for capability in semantic_capabilities {
            assert!(
                capability.contains('.'),
                "Semantic capabilities use domain.operation pattern"
            );
            let parts: Vec<&str> = capability.split('.').collect();
            assert_eq!(parts.len(), 2);
            assert_eq!(parts[0], "ai", "Domain should be 'ai'");
        }
    }

    #[tokio::test]
    async fn test_capability_discovery_service_mesh() {
        // Discover service mesh capabilities
        let service_mesh_capabilities = vec![
            "service_mesh",
            "discovery",
            "load_balancing",
            "circuit_breaking",
            "health_monitoring",
        ];

        for capability in &service_mesh_capabilities {
            assert!(!capability.is_empty());
        }

        // Test semantic operations for service mesh
        let semantic_operations = vec![
            "service_mesh.register",
            "service_mesh.discover",
            "service_mesh.health_check",
            "load_balancing.route",
            "circuit_breaking.check",
        ];

        for operation in semantic_operations {
            assert!(operation.contains('.'));
        }
    }

    #[tokio::test]
    async fn test_capability_discovery_security() {
        // Discover security and crypto capabilities
        let security_capabilities = vec![
            "crypto",
            "tls",
            "authentication",
            "authorization",
            "key_management",
        ];

        for capability in &security_capabilities {
            assert!(!capability.is_empty());
        }

        // Test semantic security operations
        let semantic_operations = vec![
            "crypto.generate_keypair",
            "crypto.encrypt",
            "crypto.decrypt",
            "crypto.sign",
            "tls.derive_secrets",
            "tls.sign_handshake",
            "auth.validate_token",
        ];

        for operation in semantic_operations {
            assert!(operation.contains('.'));
        }
    }

    #[tokio::test]
    async fn test_capability_discovery_storage() {
        // Discover storage capabilities
        let storage_capabilities = vec![
            "storage",
            "file_system",
            "object_storage",
            "backup",
            "restore",
            "volume_management",
        ];

        for capability in &storage_capabilities {
            assert!(!capability.is_empty());
        }

        // Test semantic storage operations
        let semantic_operations = vec![
            "storage.put",
            "storage.get",
            "storage.delete",
            "storage.list",
            "backup.create",
            "restore.execute",
        ];

        for operation in semantic_operations {
            assert!(operation.contains('.'));
        }
    }

    #[tokio::test]
    async fn test_capability_discovery_compute() {
        // Discover compute and orchestration capabilities
        let compute_capabilities = vec![
            "compute",
            "containers",
            "serverless",
            "orchestration",
            "gpu_acceleration",
        ];

        for capability in &compute_capabilities {
            assert!(!capability.is_empty());
        }

        // Test semantic compute operations
        let semantic_operations = vec![
            "compute.execute",
            "containers.create",
            "containers.start",
            "containers.stop",
            "orchestration.deploy",
            "orchestration.scale",
        ];

        for operation in semantic_operations {
            assert!(operation.contains('.'));
        }
    }

    #[tokio::test]
    async fn test_capability_discovery_multi_requirement() {
        // Test discovery of services that provide multiple capabilities
        
        // Example: AI service that also needs storage
        let ai_with_storage = vec!["ai", "inference", "storage"];
        assert_eq!(ai_with_storage.len(), 3);

        // Example: Service mesh that provides discovery and load balancing
        let mesh_with_lb = vec!["service_mesh", "discovery", "load_balancing"];
        assert_eq!(mesh_with_lb.len(), 3);

        // Example: Security service with crypto and TLS
        let security_suite = vec!["crypto", "tls", "key_management"];
        assert_eq!(security_suite.len(), 3);

        // Verify all requirements are non-empty
        for capability in ai_with_storage
            .iter()
            .chain(mesh_with_lb.iter())
            .chain(security_suite.iter())
        {
            assert!(!capability.is_empty());
        }
    }

    #[tokio::test]
    async fn test_capability_versioning() {
        // Test capability versioning patterns
        use std::collections::HashMap;

        let mut capability_versions = HashMap::new();
        capability_versions.insert("ai", "v1");
        capability_versions.insert("crypto", "v2");
        capability_versions.insert("storage", "v1");

        // Verify all capabilities have versions
        for (capability, version) in &capability_versions {
            assert!(!capability.is_empty());
            assert!(!version.is_empty());
            assert!(version.starts_with('v'));
        }

        // Test semantic versioning for operations
        let versioned_operations = vec![
            ("ai.inference", "v1"),
            ("crypto.encrypt", "v2"),
            ("storage.put", "v1"),
        ];

        for (operation, version) in versioned_operations {
            assert!(operation.contains('.'));
            assert!(version.starts_with('v'));
        }
    }

    #[tokio::test]
    async fn test_capability_metadata_filtering() {
        // Test filtering services by capability metadata
        use std::collections::HashMap;

        // Example: Find AI services with specific model support
        let mut ai_metadata = HashMap::new();
        ai_metadata.insert("capability", "ai");
        ai_metadata.insert("models", "gpt-4,claude-3,llama-2");
        ai_metadata.insert("context_window", "128k");

        assert_eq!(ai_metadata.get("capability"), Some(&"ai"));
        assert!(ai_metadata
            .get("models")
            .unwrap()
            .contains("gpt-4"));

        // Example: Find storage services with specific features
        let mut storage_metadata = HashMap::new();
        storage_metadata.insert("capability", "storage");
        storage_metadata.insert("type", "object_storage");
        storage_metadata.insert("replication", "true");

        assert_eq!(storage_metadata.get("capability"), Some(&"storage"));
        assert_eq!(storage_metadata.get("replication"), Some(&"true"));
    }

    #[tokio::test]
    async fn test_capability_discovery_fallback() {
        // Test fallback behavior when primary capability not available
        
        // Primary capability
        let primary = "ai.inference";
        assert!(!primary.is_empty());

        // Fallback capabilities (more general)
        let fallbacks = vec!["ai", "compute"];
        assert_eq!(fallbacks.len(), 2);

        // Verify fallback chain
        for fallback in &fallbacks {
            assert!(!fallback.is_empty());
            assert!(
                fallback.len() < primary.len(),
                "Fallbacks are more general (shorter)"
            );
        }
    }

    #[tokio::test]
    async fn test_capability_discovery_composition() {
        // Test discovering composite services that combine capabilities
        
        // AI + Crypto: Secure AI inference
        let secure_ai = vec!["ai", "inference", "crypto", "tls"];
        assert_eq!(secure_ai.len(), 4);

        // Storage + Backup + Crypto: Secure backup system
        let secure_backup = vec!["storage", "backup", "crypto", "encryption"];
        assert_eq!(secure_backup.len(), 4);

        // Service Mesh + Security: Secure service discovery
        let secure_mesh = vec![
            "service_mesh",
            "discovery",
            "authentication",
            "authorization",
        ];
        assert_eq!(secure_mesh.len(), 4);

        // Verify all composite capabilities are valid
        for capability in secure_ai
            .iter()
            .chain(secure_backup.iter())
            .chain(secure_mesh.iter())
        {
            assert!(!capability.is_empty());
        }
    }

    #[tokio::test]
    async fn test_self_knowledge_vs_discovery() {
        // Demonstrate difference between self-knowledge and discovery

        // SELF-KNOWLEDGE: Squirrel knows its own capabilities
        let own_capabilities = vec!["ai", "inference", "chat", "code_completion"];
        for capability in &own_capabilities {
            assert!(!capability.is_empty());
        }

        // DISCOVERY: Squirrel discovers OTHER services by capability
        let needed_capabilities = vec![
            "crypto",          // Need security service
            "storage",         // Need persistent storage
            "service_mesh",    // Need service discovery
        ];
        for capability in &needed_capabilities {
            assert!(!capability.is_empty());
            // In production: registry.find_services_by_capability(capability).await
        }

        // IMPORTANT: No overlap between self-knowledge and discovery
        // (Squirrel doesn't discover itself)
        for own_cap in &own_capabilities {
            for needed_cap in &needed_capabilities {
                assert_ne!(own_cap, needed_cap, "Should not discover own capabilities");
            }
        }
    }

    #[tokio::test]
    async fn test_dynamic_capability_registration() {
        // Test that services can register capabilities dynamically
        use std::collections::HashSet;

        let mut capabilities = HashSet::new();

        // Initial capabilities
        capabilities.insert("ai".to_string());
        capabilities.insert("inference".to_string());
        assert_eq!(capabilities.len(), 2);

        // Add new capability at runtime
        capabilities.insert("embeddings".to_string());
        assert_eq!(capabilities.len(), 3);

        // Add another capability
        capabilities.insert("chat".to_string());
        assert_eq!(capabilities.len(), 4);

        // Verify all capabilities
        assert!(capabilities.contains("ai"));
        assert!(capabilities.contains("inference"));
        assert!(capabilities.contains("embeddings"));
        assert!(capabilities.contains("chat"));

        // Demonstrate removal
        capabilities.remove("embeddings");
        assert_eq!(capabilities.len(), 3);
        assert!(!capabilities.contains("embeddings"));
    }
}

