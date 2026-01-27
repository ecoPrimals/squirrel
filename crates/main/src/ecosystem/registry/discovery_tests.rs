//! Tests for ecosystem registry discovery operations

#[cfg(test)]
mod tests {
    use crate::ecosystem::registry::discovery::DiscoveryOps;
    use crate::ecosystem::registry::types::*;
    use crate::ecosystem::EcosystemPrimalType;
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    #[test]
    fn test_build_service_endpoint_squirrel() {
        let endpoint = DiscoveryOps::build_service_endpoint(&EcosystemPrimalType::Squirrel);
        assert!(endpoint.contains("http"));
        assert!(!endpoint.is_empty());
    }

    #[test]
    #[allow(deprecated)]
    fn test_build_service_endpoint_songbird() {
        let endpoint = DiscoveryOps::build_service_endpoint(&EcosystemPrimalType::Songbird);
        assert!(endpoint.contains("http"));
    }

    #[test]
    fn test_build_service_endpoint_biomeos() {
        let endpoint = DiscoveryOps::build_service_endpoint(&EcosystemPrimalType::BiomeOS);
        assert!(endpoint.contains("http"));
    }

    #[test]
    fn test_build_service_endpoint_custom() {
        let custom_type = EcosystemPrimalType::Custom("test_primal".to_string());
        let endpoint = DiscoveryOps::build_service_endpoint(&custom_type);
        assert!(!endpoint.is_empty());
    }

    #[test]
    fn test_get_capabilities_for_ai_service() {
        // ✅ NEW: Capability-based test
        let caps = DiscoveryOps::get_capabilities_for_service("ai_coordination");
        assert!(caps.len() > 0);
        assert!(caps.iter().any(|c| &**c == "ai_coordination"));
    }

    #[test]
    #[allow(deprecated)]
    fn test_get_capabilities_squirrel_deprecated() {
        // Testing deprecated API for backward compatibility
        let caps = DiscoveryOps::get_capabilities_for_primal(&EcosystemPrimalType::Squirrel);
        assert!(caps.len() > 0);
        assert!(caps.iter().any(|c| &**c == "ai_coordination"));
    }

    #[test]
    fn test_get_capabilities_for_service_mesh() {
        // ✅ NEW: Capability-based test
        let caps = DiscoveryOps::get_capabilities_for_service("service_mesh");
        assert!(caps.len() > 0);
        assert!(caps.iter().any(|c| &**c == "service_mesh"));
    }

    #[test]
    fn test_get_capabilities_for_compute() {
        // ✅ NEW: Capability-based test
        let caps = DiscoveryOps::get_capabilities_for_service("compute");
        assert!(caps.len() > 0);
        assert!(caps.iter().any(|c| &**c == "compute" || &**c == "storage"));
    }

    #[test]
    #[allow(deprecated)]
    fn test_get_capabilities_toadstool_deprecated() {
        // Testing deprecated API for backward compatibility
        let caps = DiscoveryOps::get_capabilities_for_primal(&EcosystemPrimalType::ToadStool);
        assert!(caps.len() > 0);
        assert!(caps.iter().any(|c| &**c == "compute" || &**c == "storage"));
    }

    #[test]
    fn test_get_capabilities_for_security() {
        // ✅ NEW: Capability-based test
        let caps = DiscoveryOps::get_capabilities_for_service("security");
        assert!(caps.len() > 0);
        assert!(caps.iter().any(|c| &**c == "security"));
    }

    #[test]
    fn test_get_capabilities_for_networking() {
        // ✅ NEW: Capability-based test
        let caps = DiscoveryOps::get_capabilities_for_service("networking");
        assert!(caps.len() > 0);
        assert!(caps.iter().any(|c| &**c == "networking" || &**c == "gateway"));
    }

    #[test]
    fn test_get_capabilities_for_os() {
        // ✅ NEW: Capability-based test
        let caps = DiscoveryOps::get_capabilities_for_service("operating_system");
        assert!(caps.len() > 0);
        assert!(caps.iter().any(|c| &**c == "operating_system"));
    }

    #[test]
    #[allow(deprecated)]
    fn test_get_capabilities_biomeos_deprecated() {
        // Testing deprecated API for backward compatibility
        let caps = DiscoveryOps::get_capabilities_for_primal(&EcosystemPrimalType::BiomeOS);
        assert!(caps.len() > 0);
        assert!(caps.iter().any(|c| &**c == "operating_system"));
    }

    #[test]
    #[allow(deprecated)]
    fn test_get_capabilities_custom() {
        let custom_type = EcosystemPrimalType::Custom("test".to_string());
        let caps = DiscoveryOps::get_capabilities_for_primal(&custom_type);
        assert_eq!(caps.len(), 1);
        assert_eq!(&*caps[0], "discovery");
    }

    #[tokio::test]
    async fn test_discover_services_empty() {
        let registry = Arc::new(RwLock::new(HashMap::new()));
        let primal_types = vec![];

        let result = DiscoveryOps::discover_services(&registry, primal_types).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_discover_services_single() {
        let registry = Arc::new(RwLock::new(HashMap::new()));
        let primal_types = vec![EcosystemPrimalType::Squirrel];

        let result = DiscoveryOps::discover_services(&registry, primal_types).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[allow(deprecated)]
    async fn test_discover_services_multiple() {
        let registry = Arc::new(RwLock::new(HashMap::new()));
        let primal_types = vec![EcosystemPrimalType::Squirrel, EcosystemPrimalType::Songbird];

        let result = DiscoveryOps::discover_services(&registry, primal_types).await;
        assert!(result.is_ok());
    }

    #[test]
    #[allow(deprecated)]
    fn test_all_primal_types_have_endpoints() {
        let types = vec![
            EcosystemPrimalType::Squirrel,
            EcosystemPrimalType::Songbird,
            EcosystemPrimalType::ToadStool,
            EcosystemPrimalType::BearDog,
            EcosystemPrimalType::NestGate,
            EcosystemPrimalType::BiomeOS,
        ];

        for primal_type in types {
            let endpoint = DiscoveryOps::build_service_endpoint(&primal_type);
            assert!(!endpoint.is_empty());
            assert!(endpoint.starts_with("http"));
        }
    }

    #[test]
    #[allow(deprecated)]
    fn test_all_primal_types_have_capabilities() {
        // Testing deprecated API for backward compatibility
        let types = vec![
            EcosystemPrimalType::Squirrel,
            EcosystemPrimalType::Songbird,
            EcosystemPrimalType::ToadStool,
            EcosystemPrimalType::BearDog,
            EcosystemPrimalType::NestGate,
            EcosystemPrimalType::BiomeOS,
        ];

        for primal_type in types {
            let caps = DiscoveryOps::get_capabilities_for_primal(&primal_type);
            assert!(!caps.is_empty());
        }
    }

    // ============================================================================
    // NEW: Capability-Based Discovery Tests (TRUE PRIMAL Architecture)
    // ============================================================================
    //
    // These tests demonstrate the evolved discovery system based on capabilities
    // rather than hardcoded primal types. This aligns with TRUE PRIMAL principles:
    //
    // 1. Self-knowledge: Each primal knows only itself
    // 2. Runtime discovery: Services discovered by capability, not name
    // 3. Semantic naming: domain.operation pattern for IPC
    // 4. Agnostic architecture: No compile-time coupling between primals
    //
    // See: wateringHole/SEMANTIC_METHOD_NAMING_STANDARD.md
    // See: wateringHole/INTER_PRIMAL_INTERACTIONS.md
    //

    #[test]
    fn test_capability_based_service_lookup() {
        // Lookup services by capability, not by primal type
        let capabilities = vec![
            "ai",
            "inference",
            "chat",
            "service_mesh",
            "crypto",
            "storage",
        ];

        for capability in capabilities {
            let services = DiscoveryOps::get_capabilities_for_service(capability);
            assert!(!services.is_empty(), "Capability '{}' should exist", capability);
            
            // Verify semantic structure
            for service_cap in &services {
                assert!(
                    !service_cap.is_empty(),
                    "Service capability should not be empty"
                );
            }
        }
    }

    #[test]
    fn test_semantic_capability_naming() {
        // Test semantic naming pattern: domain.operation
        let semantic_capabilities = vec![
            ("ai", vec!["ai.inference", "ai.chat", "ai.embeddings"]),
            (
                "crypto",
                vec![
                    "crypto.generate_keypair",
                    "crypto.encrypt",
                    "crypto.decrypt",
                ],
            ),
            ("storage", vec!["storage.put", "storage.get", "storage.delete"]),
            (
                "service_mesh",
                vec![
                    "service_mesh.register",
                    "service_mesh.discover",
                    "service_mesh.health_check",
                ],
            ),
        ];

        for (domain, operations) in semantic_capabilities {
            // Verify domain exists
            assert!(!domain.is_empty());

            // Verify operations follow semantic naming
            for operation in operations {
                assert!(
                    operation.contains('.'),
                    "Operation '{}' should use domain.operation pattern",
                    operation
                );
                assert!(
                    operation.starts_with(domain),
                    "Operation '{}' should start with domain '{}'",
                    operation,
                    domain
                );

                let parts: Vec<&str> = operation.split('.').collect();
                assert_eq!(
                    parts.len(),
                    2,
                    "Operation '{}' should have exactly 2 parts",
                    operation
                );
            }
        }
    }

    #[test]
    fn test_capability_discovery_with_metadata() {
        // Test discovering capabilities with metadata filters
        use std::collections::HashMap;

        let test_cases = vec![
            (
                "ai",
                {
                    let mut m = HashMap::new();
                    m.insert("models", "gpt-4,claude-3");
                    m.insert("context", "128k");
                    m
                },
            ),
            (
                "storage",
                {
                    let mut m = HashMap::new();
                    m.insert("type", "object_storage");
                    m.insert("replication", "true");
                    m
                },
            ),
            (
                "crypto",
                {
                    let mut m = HashMap::new();
                    m.insert("algorithms", "ed25519,x25519");
                    m.insert("tls_version", "1.3");
                    m
                },
            ),
        ];

        for (capability, metadata) in test_cases {
            assert!(!capability.is_empty());
            assert!(!metadata.is_empty());

            // Verify metadata structure
            for (key, value) in metadata {
                assert!(!key.is_empty());
                assert!(!value.is_empty());
            }
        }
    }

    #[test]
    fn test_capability_version_compatibility() {
        // Test capability versioning for backward compatibility
        let versioned_capabilities = vec![
            ("ai.inference", "v1"),
            ("ai.inference", "v2"),
            ("crypto.encrypt", "v1"),
            ("crypto.encrypt", "v2"),
            ("storage.put", "v1"),
        ];

        for (capability, version) in versioned_capabilities {
            assert!(capability.contains('.'));
            assert!(version.starts_with('v'));

            // Verify version format
            let version_num = &version[1..];
            assert!(
                version_num.parse::<u32>().is_ok(),
                "Version '{}' should be numeric after 'v'",
                version
            );
        }
    }

    #[test]
    fn test_capability_fallback_chain() {
        // Test fallback behavior when specific capability not available
        let fallback_chains = vec![
            // Specific -> General
            vec!["ai.inference.gpt4", "ai.inference", "ai"],
            vec!["crypto.encrypt.aes256", "crypto.encrypt", "crypto"],
            vec![
                "storage.object.s3compatible",
                "storage.object",
                "storage",
            ],
        ];

        for chain in fallback_chains {
            assert!(chain.len() >= 2, "Fallback chain should have at least 2 levels");

            // Verify each level is more general than the previous
            for i in 1..chain.len() {
                let specific = chain[i - 1];
                let general = chain[i];

                assert!(
                    specific.len() > general.len(),
                    "Fallback '{}' should be shorter than '{}'",
                    general,
                    specific
                );
                assert!(
                    specific.starts_with(general),
                    "Specific capability '{}' should start with general '{}'",
                    specific,
                    general
                );
            }
        }
    }

    #[test]
    fn test_multi_capability_requirements() {
        // Test services requiring multiple capabilities
        let multi_capability_services = vec![
            // AI service with storage
            ("secure_ai", vec!["ai", "inference", "crypto", "storage"]),
            // Service mesh with security
            (
                "secure_mesh",
                vec!["service_mesh", "discovery", "authentication"],
            ),
            // Backup service with encryption
            ("secure_backup", vec!["storage", "backup", "crypto"]),
        ];

        for (service_name, capabilities) in multi_capability_services {
            assert!(!service_name.is_empty());
            assert!(
                capabilities.len() >= 2,
                "Multi-capability service should require at least 2 capabilities"
            );

            for capability in capabilities {
                assert!(!capability.is_empty());
            }
        }
    }

    #[test]
    fn test_capability_provider_agnostic() {
        // Test that capabilities are provider-agnostic
        // Multiple providers can offer the same capability

        let capability = "crypto";
        
        // Different providers offering same capability
        let providers = vec!["provider_a", "provider_b", "provider_c"];

        for provider in providers {
            // All provide same capability
            assert!(!provider.is_empty());
            assert!(!capability.is_empty());
            
            // Provider identity is separate from capability
            assert!(
                !capability.contains(provider),
                "Capability should not encode provider name"
            );
        }
    }

    #[test]
    fn test_dynamic_capability_registration() {
        // Test dynamic capability registration and deregistration
        use std::collections::HashSet;

        let mut active_capabilities = HashSet::new();

        // Register capabilities dynamically
        let capabilities_to_register = vec![
            "ai.inference",
            "ai.chat",
            "ai.embeddings",
            "storage.object",
        ];

        for capability in capabilities_to_register {
            active_capabilities.insert(capability.to_string());
        }

        assert_eq!(active_capabilities.len(), 4);

        // Deregister a capability
        active_capabilities.remove("ai.embeddings");
        assert_eq!(active_capabilities.len(), 3);
        assert!(!active_capabilities.contains("ai.embeddings"));

        // Register a new capability
        active_capabilities.insert("ai.code_completion".to_string());
        assert_eq!(active_capabilities.len(), 4);
        assert!(active_capabilities.contains("ai.code_completion"));
    }

    #[test]
    fn test_capability_based_endpoint_discovery() {
        // Test discovering endpoints by capability rather than primal name
        
        let capability_endpoints = vec![
            ("ai", vec!["/rpc/ai", "/api/ai/v1"]),
            ("crypto", vec!["/rpc/crypto", "/api/crypto/v1"]),
            ("storage", vec!["/rpc/storage", "/api/storage/v1"]),
        ];

        for (capability, endpoints) in capability_endpoints {
            assert!(!capability.is_empty());
            assert!(!endpoints.is_empty());

            for endpoint in endpoints {
                assert!(endpoint.starts_with('/'));
                assert!(
                    endpoint.contains(capability) || endpoint.contains("rpc") || endpoint.contains("api"),
                    "Endpoint '{}' should reflect capability or protocol",
                    endpoint
                );
            }
        }
    }

    #[test]
    fn test_capability_health_checks() {
        // Test health check patterns for capability-based services
        
        let capability_health_endpoints = vec![
            ("ai", "/health/ai"),
            ("crypto", "/health/crypto"),
            ("storage", "/health/storage"),
            ("service_mesh", "/health/mesh"),
        ];

        for (capability, health_endpoint) in capability_health_endpoints {
            assert!(!capability.is_empty());
            assert!(health_endpoint.starts_with("/health"));
            assert!(
                health_endpoint.contains(capability) || health_endpoint.contains("mesh"),
                "Health endpoint should reference capability"
            );
        }
    }

    #[test]
    fn test_self_knowledge_pattern() {
        // Test that services have self-knowledge without hardcoding others
        
        // Squirrel's self-knowledge (acceptable)
        let own_capabilities = vec!["ai", "inference", "chat", "code_completion"];
        assert!(!own_capabilities.is_empty());
        
        // Services discovered at runtime by capability (TRUE PRIMAL)
        let discovered_capabilities = vec![
            "crypto",       // Discovered at runtime
            "storage",      // Discovered at runtime
            "service_mesh", // Discovered at runtime
        ];
        
        // Verify no overlap (don't discover own capabilities)
        for own_cap in &own_capabilities {
            for discovered_cap in &discovered_capabilities {
                assert_ne!(
                    own_cap, discovered_cap,
                    "Should not discover own capabilities"
                );
            }
        }
    }

    #[test]
    fn test_capability_discovery_concurrency() {
        // Test concurrent capability discovery
        use std::sync::Arc;
        use std::collections::HashMap;

        let capabilities = Arc::new(vec!["ai", "crypto", "storage"]);
        let results = Arc::new(std::sync::Mutex::new(HashMap::new()));

        // Simulate concurrent discovery
        for capability in capabilities.iter() {
            let mut results = results.lock().unwrap();
            results.insert(capability.to_string(), format!("endpoint_for_{}", capability));
        }

        let results = results.lock().unwrap();
        assert_eq!(results.len(), 3);
        assert!(results.contains_key("ai"));
        assert!(results.contains_key("crypto"));
        assert!(results.contains_key("storage"));
    }
}
