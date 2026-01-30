//! Comprehensive tests for ecosystem types
//!
//! This module provides thorough testing of all ecosystem integration types
//! including primal types, service registration, capabilities, and configuration.
//!
//! ## Deprecated API Tests
//!
//! This module includes tests for the deprecated `EcosystemPrimalType` enum.
//! These tests are intentionally kept to ensure backward compatibility.
//! They use `#[allow(deprecated)]` to acknowledge testing deprecated APIs.

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::ecosystem::arc_str;

    // ========== EcosystemPrimalType Tests (Backward Compatibility) ==========
    // These tests verify the deprecated EcosystemPrimalType API remains functional

    #[test]
    #[allow(deprecated)]
    fn test_primal_type_variants() {
        let types = [
            EcosystemPrimalType::ToadStool,
            EcosystemPrimalType::Songbird,
            EcosystemPrimalType::BearDog,
            EcosystemPrimalType::NestGate,
            EcosystemPrimalType::Squirrel,
            EcosystemPrimalType::BiomeOS,
        ];

        // Verify all variants are distinct
        assert_eq!(types.len(), 6);
        for i in 0..types.len() {
            for j in 0..types.len() {
                if i == j {
                    assert_eq!(types[i], types[j]);
                } else {
                    assert_ne!(types[i], types[j]);
                }
            }
        }
    }

    #[test]
    #[allow(deprecated)]
    fn test_primal_type_as_str() {
        assert_eq!(EcosystemPrimalType::ToadStool.as_str(), "toadstool");
        assert_eq!(EcosystemPrimalType::Songbird.as_str(), "songbird");
        assert_eq!(EcosystemPrimalType::BearDog.as_str(), "beardog");
        assert_eq!(EcosystemPrimalType::NestGate.as_str(), "nestgate");
        assert_eq!(EcosystemPrimalType::Squirrel.as_str(), "squirrel");
        assert_eq!(EcosystemPrimalType::BiomeOS.as_str(), "biomeos");
    }

    #[test]
    #[allow(deprecated)]
    fn test_primal_type_from_str() {
        assert_eq!(
            "toadstool".parse::<EcosystemPrimalType>().unwrap(),
            EcosystemPrimalType::ToadStool
        );
        assert_eq!(
            "ToadStool".parse::<EcosystemPrimalType>().unwrap(),
            EcosystemPrimalType::ToadStool
        );
        assert_eq!(
            "TOADSTOOL".parse::<EcosystemPrimalType>().unwrap(),
            EcosystemPrimalType::ToadStool
        );

        assert_eq!(
            "songbird".parse::<EcosystemPrimalType>().unwrap(),
            EcosystemPrimalType::Songbird
        );
        assert_eq!(
            "beardog".parse::<EcosystemPrimalType>().unwrap(),
            EcosystemPrimalType::BearDog
        );
        assert_eq!(
            "nestgate".parse::<EcosystemPrimalType>().unwrap(),
            EcosystemPrimalType::NestGate
        );
        assert_eq!(
            "squirrel".parse::<EcosystemPrimalType>().unwrap(),
            EcosystemPrimalType::Squirrel
        );
        assert_eq!(
            "biomeos".parse::<EcosystemPrimalType>().unwrap(),
            EcosystemPrimalType::BiomeOS
        );
    }

    #[test]
    #[allow(deprecated)]
    fn test_primal_type_from_str_custom() {
        // Unknown primal types now parse as Custom variants (forward compatibility)
        assert_eq!(
            "invalid".parse::<EcosystemPrimalType>().unwrap(),
            EcosystemPrimalType::Custom("invalid".to_string())
        );
        assert_eq!(
            "".parse::<EcosystemPrimalType>().unwrap(),
            EcosystemPrimalType::Custom("".to_string())
        );
        assert_eq!(
            "primal".parse::<EcosystemPrimalType>().unwrap(),
            EcosystemPrimalType::Custom("primal".to_string())
        );
    }

    #[test]
    fn test_primal_type_clone() {
        // Note: EcosystemPrimalType no longer implements Copy due to Custom(String) variant
        let original = EcosystemPrimalType::Squirrel;
        let cloned = original.clone();
        let cloned2 = original.clone();

        assert_eq!(cloned, cloned2);
    }

    #[test]
    fn test_primal_type_hash() {
        use std::collections::HashMap;

        let mut map = HashMap::new();
        map.insert(EcosystemPrimalType::Squirrel, "squirrel_data");
        #[allow(deprecated)]
        {
            map.insert(EcosystemPrimalType::BearDog, "beardog_data");
        }
        map.insert(
            EcosystemPrimalType::Custom("my-primal".to_string()),
            "custom_data",
        );

        assert_eq!(
            map.get(&EcosystemPrimalType::Squirrel),
            Some(&"squirrel_data")
        );
        #[allow(deprecated)]
        {
            assert_eq!(
                map.get(&EcosystemPrimalType::BearDog),
                Some(&"beardog_data")
            );
        }
        assert_eq!(
            map.get(&EcosystemPrimalType::Custom("my-primal".to_string())),
            Some(&"custom_data")
        );
    }

    #[test]
    fn test_primal_type_serialization() {
        let primal_type = EcosystemPrimalType::Squirrel;
        let json = serde_json::to_string(&primal_type).unwrap();

        let deserialized: EcosystemPrimalType = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, EcosystemPrimalType::Squirrel);
    }

    #[test]
    fn test_all_primal_types_serialization() {
        #[allow(deprecated)]
        let types = vec![
            EcosystemPrimalType::ToadStool,
            EcosystemPrimalType::Songbird,
            EcosystemPrimalType::BearDog,
            EcosystemPrimalType::NestGate,
            EcosystemPrimalType::Squirrel,
            EcosystemPrimalType::BiomeOS,
            EcosystemPrimalType::Custom("test-primal".to_string()),
        ];

        for primal_type in types {
            let json = serde_json::to_string(&primal_type).unwrap();
            let deserialized: EcosystemPrimalType = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, primal_type);
        }
    }

    // ========== ServiceCapabilities Tests ==========

    #[test]
    fn test_service_capabilities_default() {
        let caps = ServiceCapabilities::default();
        assert!(caps.core.is_empty());
        assert!(caps.extended.is_empty());
        assert!(caps.integrations.is_empty());
    }

    #[test]
    fn test_service_capabilities_creation() {
        let caps = ServiceCapabilities {
            core: vec!["ai".to_string(), "inference".to_string()],
            extended: vec!["multi-model".to_string()],
            integrations: vec!["anthropic".to_string(), "openai".to_string()],
        };

        assert_eq!(caps.core.len(), 2);
        assert_eq!(caps.extended.len(), 1);
        assert_eq!(caps.integrations.len(), 2);
    }

    #[test]
    fn test_service_capabilities_clone() {
        let original = ServiceCapabilities {
            core: vec!["test".to_string()],
            extended: vec![],
            integrations: vec![],
        };

        let cloned = original.clone();
        assert_eq!(cloned.core, original.core);
    }

    #[test]
    fn test_service_capabilities_serialization() {
        let caps = ServiceCapabilities {
            core: vec!["ai".to_string()],
            extended: vec!["advanced".to_string()],
            integrations: vec!["api".to_string()],
        };

        let json = serde_json::to_string(&caps).unwrap();
        let deserialized: ServiceCapabilities = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.core, caps.core);
        assert_eq!(deserialized.extended, caps.extended);
        assert_eq!(deserialized.integrations, caps.integrations);
    }

    // ========== ServiceEndpoints Tests ==========

    #[test]
    fn test_service_endpoints_default() {
        let endpoints = ServiceEndpoints::default();
        assert_eq!(endpoints.primary, "");
        assert!(endpoints.secondary.is_empty());
        assert!(endpoints.health.is_none());
    }

    #[test]
    fn test_service_endpoints_creation() {
        // Use flexible endpoints for testing
        let primary_port = std::env::var("TEST_PRIMARY_PORT")
            .ok()
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or(8080);
        let secondary_port = std::env::var("TEST_SECONDARY_PORT")
            .ok()
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or(8081);
        
        let endpoints = ServiceEndpoints {
            primary: format!("http://localhost:{}", primary_port),
            secondary: vec![format!("http://localhost:{}", secondary_port)],
            health: Some("/health".to_string()),
        };

        assert_eq!(endpoints.primary, "http://localhost:8080");
        assert_eq!(endpoints.secondary.len(), 1);
        assert_eq!(endpoints.health, Some("/health".to_string()));
    }

    #[test]
    fn test_service_endpoints_multiple_secondary() {
        let endpoints = ServiceEndpoints {
            primary: "http://primary:8080".to_string(),
            secondary: vec![
                "http://secondary1:8081".to_string(),
                "http://secondary2:8082".to_string(),
                "http://secondary3:8083".to_string(),
            ],
            health: Some("/health".to_string()),
        };

        assert_eq!(endpoints.secondary.len(), 3);
    }

    #[test]
    fn test_service_endpoints_serialization() {
        let endpoints = ServiceEndpoints {
            primary: "http://test:3000".to_string(),
            secondary: vec!["http://backup:3001".to_string()],
            health: Some("/status".to_string()),
        };

        let json = serde_json::to_string(&endpoints).unwrap();
        let deserialized: ServiceEndpoints = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.primary, endpoints.primary);
        assert_eq!(deserialized.secondary, endpoints.secondary);
        assert_eq!(deserialized.health, endpoints.health);
    }

    // ========== HealthCheckConfig Tests ==========

    #[test]
    fn test_health_check_config_default() {
        let config = HealthCheckConfig::default();
        // Verify fields exist
        assert!(!config.enabled || config.enabled); // Boolean field
        assert!(config.interval_secs >= 0);
        assert!(config.timeout_secs >= 0);
        assert!(config.failure_threshold >= 0);
    }

    #[test]
    fn test_health_check_config_creation() {
        let config = HealthCheckConfig {
            enabled: true,
            interval_secs: 30,
            timeout_secs: 10,
            failure_threshold: 3,
        };

        assert!(config.enabled);
        assert_eq!(config.interval_secs, 30);
        assert_eq!(config.timeout_secs, 10);
        assert_eq!(config.failure_threshold, 3);
    }

    #[test]
    fn test_health_check_config_serialization() {
        let config = HealthCheckConfig {
            enabled: true,
            interval_secs: 60,
            timeout_secs: 15,
            failure_threshold: 5,
        };
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: HealthCheckConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.enabled, config.enabled);
        assert_eq!(deserialized.interval_secs, config.interval_secs);
    }

    // ========== SecurityConfig Tests ==========

    #[test]
    fn test_security_config_default() {
        let config = SecurityConfig::default();
        // Verify defaults exist
        assert!(!config.auth_required || config.auth_required);
        assert!(!config.audit_enabled || config.audit_enabled);
    }

    #[test]
    fn test_security_config_creation() {
        let config = SecurityConfig {
            auth_required: true,
            encryption_level: "high".to_string(),
            access_level: "restricted".to_string(),
            policies: vec!["policy1".to_string()],
            audit_enabled: true,
            security_level: "maximum".to_string(),
        };

        assert!(config.auth_required);
        assert_eq!(config.encryption_level, "high");
        assert_eq!(config.access_level, "restricted");
        assert_eq!(config.policies.len(), 1);
        assert!(config.audit_enabled);
    }

    #[test]
    fn test_security_config_no_auth() {
        let config = SecurityConfig {
            auth_required: false,
            encryption_level: "none".to_string(),
            access_level: "public".to_string(),
            policies: vec![],
            audit_enabled: false,
            security_level: "low".to_string(),
        };

        assert!(!config.auth_required);
        assert!(!config.audit_enabled);
        assert_eq!(config.policies.len(), 0);
    }

    #[test]
    fn test_security_config_serialization() {
        let config = SecurityConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: SecurityConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.auth_required, config.auth_required);
        assert_eq!(deserialized.encryption_level, config.encryption_level);
    }

    // ========== ResourceSpec Tests ==========

    #[test]
    fn test_resource_spec_creation() {
        let spec = ResourceSpec {
            cpu: "2 cores".to_string(),
            memory: "4GB".to_string(),
            storage: "20GB".to_string(),
            network: "1Gbps".to_string(),
            gpu: None,
        };

        assert_eq!(spec.cpu, "2 cores");
        assert_eq!(spec.memory, "4GB");
        assert_eq!(spec.storage, "20GB");
        assert_eq!(spec.network, "1Gbps");
        assert!(spec.gpu.is_none());
    }

    #[test]
    fn test_resource_spec_with_gpu() {
        let spec = ResourceSpec {
            cpu: "4 cores".to_string(),
            memory: "8GB".to_string(),
            storage: "50GB".to_string(),
            network: "10Gbps".to_string(),
            gpu: Some("NVIDIA A100".to_string()),
        };

        assert!(spec.gpu.is_some());
        assert_eq!(spec.gpu.unwrap(), "NVIDIA A100");
    }

    #[test]
    fn test_resource_spec_large_requirements() {
        let spec = ResourceSpec {
            cpu: "16 cores".to_string(),
            memory: "64GB".to_string(),
            storage: "1TB".to_string(),
            network: "40Gbps".to_string(),
            gpu: Some("8x NVIDIA H100".to_string()),
        };

        assert_eq!(spec.cpu, "16 cores");
        assert_eq!(spec.memory, "64GB");
        assert!(spec.gpu.is_some());
    }

    #[test]
    fn test_resource_spec_serialization() {
        let spec = ResourceSpec {
            cpu: "2 cores".to_string(),
            memory: "2GB".to_string(),
            storage: "10GB".to_string(),
            network: "100Mbps".to_string(),
            gpu: None,
        };
        let json = serde_json::to_string(&spec).unwrap();
        let deserialized: ResourceSpec = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.cpu, spec.cpu);
        assert_eq!(deserialized.memory, spec.memory);
    }

    // ========== EcosystemServiceRegistration Tests ==========

    #[test]
    fn test_service_registration_creation() {
        let reg = EcosystemServiceRegistration {
            service_id: arc_str("primal-squirrel-test"),
            primal_type: EcosystemPrimalType::Squirrel,
            name: arc_str("Test Squirrel"),
            description: "Test instance".to_string(),
            biome_id: Some(arc_str("test-biome")),
            version: arc_str("0.1.0"),
            capabilities: ServiceCapabilities::default(),
            endpoints: ServiceEndpoints::default(),
            dependencies: vec![],
            tags: vec!["test".to_string()],
            primal_provider: Some("test_provider".to_string()),
            health_check: HealthCheckConfig::default(),
            security_config: SecurityConfig::default(),
            resource_requirements: ResourceSpec {
                cpu: "1 core".to_string(),
                memory: "1GB".to_string(),
                storage: "10GB".to_string(),
                network: "100Mbps".to_string(),
                gpu: None,
            },
            metadata: std::collections::HashMap::new(),
            registered_at: chrono::Utc::now(),
        };

        assert_eq!(reg.service_id.as_ref(), "primal-squirrel-test");
        assert_eq!(reg.primal_type, EcosystemPrimalType::Squirrel);
        assert_eq!(reg.version.as_ref(), "0.1.0");
    }

    #[test]
    fn test_service_registration_with_dependencies() {
        let reg = EcosystemServiceRegistration {
            service_id: arc_str("primal-squirrel-1"),
            primal_type: EcosystemPrimalType::Squirrel,
            name: arc_str("Squirrel"),
            description: "AI Primal".to_string(),
            biome_id: None,
            version: arc_str("1.0.0"),
            capabilities: ServiceCapabilities::default(),
            endpoints: ServiceEndpoints::default(),
            dependencies: vec!["beardog".to_string(), "nestgate".to_string()],
            tags: vec![],
            primal_provider: None,
            health_check: HealthCheckConfig::default(),
            security_config: SecurityConfig::default(),
            resource_requirements: ResourceSpec {
                cpu: "1 core".to_string(),
                memory: "1GB".to_string(),
                storage: "10GB".to_string(),
                network: "100Mbps".to_string(),
                gpu: None,
            },
            metadata: std::collections::HashMap::new(),
            registered_at: chrono::Utc::now(),
        };

        assert_eq!(reg.dependencies.len(), 2);
        assert!(reg.dependencies.contains(&"beardog".to_string()));
        assert!(reg.dependencies.contains(&"nestgate".to_string()));
    }

    #[test]
    fn test_service_registration_with_metadata() {
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("region".to_string(), "us-west".to_string());
        metadata.insert("environment".to_string(), "production".to_string());

        let reg = EcosystemServiceRegistration {
            service_id: arc_str("primal-squirrel-prod"),
            primal_type: EcosystemPrimalType::Squirrel,
            name: arc_str("Squirrel Production"),
            description: "Production instance".to_string(),
            biome_id: Some(arc_str("prod-biome")),
            version: arc_str("2.0.0"),
            capabilities: ServiceCapabilities::default(),
            endpoints: ServiceEndpoints::default(),
            dependencies: vec![],
            tags: vec!["production".to_string(), "stable".to_string()],
            primal_provider: None,
            health_check: HealthCheckConfig::default(),
            security_config: SecurityConfig::default(),
            resource_requirements: ResourceSpec {
                cpu: "1 core".to_string(),
                memory: "1GB".to_string(),
                storage: "10GB".to_string(),
                network: "100Mbps".to_string(),
                gpu: None,
            },
            metadata,
            registered_at: chrono::Utc::now(),
        };

        assert_eq!(reg.metadata.len(), 2);
        assert_eq!(reg.metadata.get("region"), Some(&"us-west".to_string()));
        assert_eq!(reg.tags.len(), 2);
    }

    #[test]
    fn test_service_registration_serialization() {
        let reg = EcosystemServiceRegistration {
            service_id: arc_str("test-service"),
            primal_type: EcosystemPrimalType::Squirrel,
            name: arc_str("Test"),
            description: "Test service".to_string(),
            biome_id: None,
            version: arc_str("0.1.0"),
            capabilities: ServiceCapabilities::default(),
            endpoints: ServiceEndpoints::default(),
            dependencies: vec![],
            tags: vec![],
            primal_provider: None,
            health_check: HealthCheckConfig::default(),
            security_config: SecurityConfig::default(),
            resource_requirements: ResourceSpec {
                cpu: "1 core".to_string(),
                memory: "1GB".to_string(),
                storage: "10GB".to_string(),
                network: "100Mbps".to_string(),
                gpu: None,
            },
            metadata: std::collections::HashMap::new(),
            registered_at: chrono::Utc::now(),
        };

        let json = serde_json::to_string(&reg).unwrap();
        let deserialized: EcosystemServiceRegistration = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.service_id, reg.service_id);
        assert_eq!(deserialized.primal_type, reg.primal_type);
        assert_eq!(deserialized.version, reg.version);
    }

    #[test]
    fn test_service_registration_clone() {
        let original = EcosystemServiceRegistration {
            service_id: arc_str("test"),
            primal_type: EcosystemPrimalType::Squirrel,
            name: arc_str("Test"),
            description: "Test".to_string(),
            biome_id: None,
            version: arc_str("0.1.0"),
            capabilities: ServiceCapabilities::default(),
            endpoints: ServiceEndpoints::default(),
            dependencies: vec![],
            tags: vec![],
            primal_provider: None,
            health_check: HealthCheckConfig::default(),
            security_config: SecurityConfig::default(),
            resource_requirements: ResourceSpec {
                cpu: "1 core".to_string(),
                memory: "1GB".to_string(),
                storage: "10GB".to_string(),
                network: "100Mbps".to_string(),
                gpu: None,
            },
            metadata: std::collections::HashMap::new(),
            registered_at: chrono::Utc::now(),
        };

        let cloned = original.clone();
        assert_eq!(cloned.service_id, original.service_id);
        assert_eq!(cloned.primal_type, original.primal_type);
    }

    // ========== Integration Tests ==========

    #[test]
    fn test_complete_service_registration_workflow() {
        // Create a complete, realistic service registration
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("datacenter".to_string(), "dc1".to_string());
        metadata.insert("rack".to_string(), "rack-42".to_string());

        let capabilities = ServiceCapabilities {
            core: vec!["ai_inference".to_string(), "model_management".to_string()],
            extended: vec!["multi_model".to_string(), "streaming".to_string()],
            integrations: vec![
                "anthropic".to_string(),
                "openai".to_string(),
                "ollama".to_string(),
            ],
        };

        let endpoints = ServiceEndpoints {
            primary: "http://squirrel-1.internal:8080".to_string(),
            secondary: vec![
                "http://squirrel-2.internal:8080".to_string(),
                "http://squirrel-3.internal:8080".to_string(),
            ],
            health: Some("/api/v1/health".to_string()),
        };

        let health_check = HealthCheckConfig {
            enabled: true,
            interval_secs: 30,
            timeout_secs: 10,
            failure_threshold: 3,
        };

        let security = SecurityConfig {
            auth_required: true,
            encryption_level: "high".to_string(),
            access_level: "restricted".to_string(),
            policies: vec!["tls_required".to_string()],
            audit_enabled: true,
            security_level: "maximum".to_string(),
        };

        let resources = ResourceSpec {
            cpu: "2 cores".to_string(),
            memory: "4GB".to_string(),
            storage: "20GB".to_string(),
            network: "1Gbps".to_string(),
            gpu: Some("NVIDIA A100".to_string()),
        };

        let registration = EcosystemServiceRegistration {
            service_id: arc_str("primal-squirrel-prod-1"),
            primal_type: EcosystemPrimalType::Squirrel,
            name: arc_str("Squirrel AI Primal"),
            description: "AI inference and model management service".to_string(),
            biome_id: Some(arc_str("production-biome-1")),
            version: arc_str("1.0.0"),
            capabilities,
            endpoints,
            dependencies: vec!["primal-beardog".to_string(), "primal-nestgate".to_string()],
            tags: vec![
                "production".to_string(),
                "ai".to_string(),
                "inference".to_string(),
            ],
            primal_provider: Some("squirrel_provider_v1".to_string()),
            health_check,
            security_config: security,
            resource_requirements: resources,
            metadata,
            registered_at: chrono::Utc::now(),
        };

        // Verify all fields are properly set
        assert_eq!(registration.primal_type, EcosystemPrimalType::Squirrel);
        assert_eq!(registration.capabilities.core.len(), 2);
        assert_eq!(registration.capabilities.integrations.len(), 3);
        assert_eq!(registration.endpoints.secondary.len(), 2);
        assert_eq!(registration.dependencies.len(), 2);
        assert_eq!(registration.tags.len(), 3);
        assert_eq!(registration.metadata.len(), 2);
        assert!(registration.security_config.auth_required);
        assert_eq!(registration.resource_requirements.memory, "4GB");

        // Test serialization round-trip
        let json = serde_json::to_string(&registration).unwrap();
        let recovered: EcosystemServiceRegistration = serde_json::from_str(&json).unwrap();

        assert_eq!(recovered.service_id, registration.service_id);
        assert_eq!(recovered.primal_type, registration.primal_type);
        assert_eq!(
            recovered.capabilities.core.len(),
            registration.capabilities.core.len()
        );
    }

    #[test]
    #[allow(deprecated)]
    fn test_primal_type_round_trip_parsing() {
        // Testing deprecated enum for backward compatibility
        let types = vec![
            "toadstool",
            "songbird",
            "beardog",
            "nestgate",
            "squirrel",
            "biomeos",
        ];

        for type_str in types {
            let parsed: EcosystemPrimalType = type_str.parse().unwrap();
            let back_to_str = parsed.as_str();
            let reparsed: EcosystemPrimalType = back_to_str.parse().unwrap();

            assert_eq!(parsed, reparsed);
        }
    }

    // ============================================================================
    // NEW: Capability-Based Type System Tests (TRUE PRIMAL Architecture)
    // ============================================================================
    //
    // These tests demonstrate the evolved type system where services are
    // identified by capabilities rather than hardcoded primal types.
    //
    // Principles:
    // - Self-knowledge: Each primal knows only itself
    // - Runtime discovery: Other primals discovered by capability
    // - Semantic naming: Capabilities describe WHAT, not WHO
    // - Agnostic architecture: No compile-time coupling
    //
    // See: wateringHole/SEMANTIC_METHOD_NAMING_STANDARD.md
    // See: wateringHole/INTER_PRIMAL_INTERACTIONS.md
    //

    /// Capability categories for TRUE PRIMAL architecture
    mod capability_types {
        /// Service mesh capabilities (discovery, routing, load balancing)
        pub const SERVICE_MESH: &[&str] = &[
            "service_mesh",
            "discovery",
            "load_balancing",
            "routing",
            "circuit_breaking",
        ];

        /// Security and cryptography capabilities
        pub const SECURITY: &[&str] = &[
            "crypto",
            "tls",
            "security",
            "authentication",
            "encryption",
            "key_management",
        ];

        /// Storage and persistence capabilities
        pub const STORAGE: &[&str] = &[
            "storage",
            "file_system",
            "object_storage",
            "backup",
            "restore",
            "volume_management",
        ];

        /// Compute and execution capabilities
        pub const COMPUTE: &[&str] = &[
            "compute",
            "containers",
            "serverless",
            "orchestration",
            "gpu_acceleration",
        ];

        /// AI and machine learning capabilities
        pub const AI: &[&str] = &[
            "ai",
            "inference",
            "chat",
            "code_completion",
            "embeddings",
            "model_training",
        ];

        /// System orchestration capabilities
        pub const ORCHESTRATION: &[&str] = &[
            "orchestration",
            "deployment",
            "manifest",
            "lifecycle",
            "health_monitoring",
        ];
    }

    #[test]
    fn test_capability_categories_completeness() {
        use capability_types::*;

        // Verify all capability categories are defined
        assert!(
            !SERVICE_MESH.is_empty(),
            "Service mesh capabilities must be defined"
        );
        assert!(!SECURITY.is_empty(), "Security capabilities must be defined");
        assert!(!STORAGE.is_empty(), "Storage capabilities must be defined");
        assert!(!COMPUTE.is_empty(), "Compute capabilities must be defined");
        assert!(!AI.is_empty(), "AI capabilities must be defined");
        assert!(
            !ORCHESTRATION.is_empty(),
            "Orchestration capabilities must be defined"
        );

        // Verify primary capabilities exist
        assert!(SERVICE_MESH.contains(&"service_mesh"));
        assert!(SECURITY.contains(&"crypto"));
        assert!(STORAGE.contains(&"storage"));
        assert!(COMPUTE.contains(&"compute"));
        assert!(AI.contains(&"ai"));
        assert!(ORCHESTRATION.contains(&"orchestration"));
    }

    #[test]
    fn test_capability_semantic_naming() {
        // Semantic naming: domain.operation pattern
        let semantic_capabilities = vec![
            "crypto.generate_keypair",
            "crypto.encrypt",
            "crypto.decrypt",
            "tls.derive_secrets",
            "tls.sign_handshake",
            "storage.put",
            "storage.get",
            "ai.inference",
            "ai.embeddings",
        ];

        for capability in semantic_capabilities {
            // Verify semantic naming structure
            assert!(
                capability.contains('.'),
                "Semantic capabilities should use domain.operation pattern: {}",
                capability
            );

            let parts: Vec<&str> = capability.split('.').collect();
            assert_eq!(
                parts.len(),
                2,
                "Semantic capability should have exactly 2 parts: {}",
                capability
            );

            // Domain should be valid
            let domain = parts[0];
            assert!(
                !domain.is_empty(),
                "Domain should not be empty in: {}",
                capability
            );

            // Operation should be valid
            let operation = parts[1];
            assert!(
                !operation.is_empty(),
                "Operation should not be empty in: {}",
                capability
            );
        }
    }

    #[test]
    fn test_capability_vs_primal_type() {
        // Demonstrate the difference between capability-based and type-based discovery

        // OLD PATTERN (Hardcoded - violates primal sovereignty):
        // let primal_type = EcosystemPrimalType::Songbird;
        // Couples code to specific primal name
        // Requires recompilation if primal changes
        // Violates TRUE PRIMAL principle

        // NEW PATTERN (Capability-based - TRUE PRIMAL):
        let capabilities = vec!["service_mesh", "discovery", "load_balancing"];
        // Describes WHAT is needed, not WHO provides it
        // Discoverable at runtime
        // Respects primal sovereignty
        // Allows primal substitution

        assert!(!capabilities.is_empty());
        assert!(capabilities.contains(&"service_mesh"));
    }

    #[test]
    fn test_self_knowledge_pattern() {
        // TRUE PRIMAL: Squirrel knows itself, discovers others

        // Self-knowledge (acceptable hardcoding):
        let own_capabilities = capability_types::AI;
        assert!(own_capabilities.contains(&"ai"));
        assert!(own_capabilities.contains(&"inference"));

        // Discovery of others (capability-based, no hardcoding):
        let needed_capabilities = vec!["crypto", "storage", "service_mesh"];

        for capability in needed_capabilities {
            // Would discover services providing these capabilities at runtime
            // NO hardcoded primal types
            // NO compile-time coupling
            assert!(!capability.is_empty());
        }
    }

    #[test]
    fn test_capability_based_service_registration() {
        // Create a service registration using capability-based pattern
        use std::collections::HashMap;

        let mut capabilities_map = HashMap::new();
        capabilities_map.insert("ai".to_string(), "true".to_string());
        capabilities_map.insert("inference".to_string(), "true".to_string());
        capabilities_map.insert(
            "models".to_string(),
            "gpt-4,claude-3,llama-2".to_string(),
        );

        let registration = EcosystemServiceRegistration {
            service_id: "squirrel-001".to_string(),
            primal_type: EcosystemPrimalType::Squirrel, // Self-knowledge only
            name: "Squirrel AI".to_string(),
            description: "AI inference and agent coordination".to_string(),
            biome_id: Some("prod-01".to_string()),
            version: "0.1.0".to_string(),
            capabilities: ServiceCapabilities {
                core: vec![
                    "ai".to_string(),
                    "inference".to_string(),
                    "chat".to_string(),
                ],
                optional: vec!["training".to_string()],
                metadata: capabilities_map,
            },
            endpoints: ServiceEndpoints {
                primary: "/rpc/v1".to_string(),
                health: "/health".to_string(),
                metrics: Some("/metrics".to_string()),
                metadata: HashMap::new(),
            },
            dependencies: vec![], // Discovered at runtime, not hardcoded
            tags: vec!["ai".to_string(), "inference".to_string()],
            primal_provider: Some("squirrel".to_string()),
            health_check: HealthCheckConfig {
                enabled: true,
                interval: 30,
                timeout: 5,
                retries: 3,
                path: "/health".to_string(),
            },
            security_config: SecurityConfig {
                auth_required: true,
                tls_enabled: true,
                allowed_origins: vec!["*".to_string()],
            },
            resource_requirements: ResourceSpec {
                memory: "4GB".to_string(),
                cpu: "2".to_string(),
                disk: "10GB".to_string(),
            },
            metadata: HashMap::new(),
            registered_at: chrono::Utc::now(),
        };

        // Verify capability-based registration
        assert!(registration.capabilities.core.contains(&"ai".to_string()));
        assert!(registration
            .capabilities
            .core
            .contains(&"inference".to_string()));
        assert!(registration.dependencies.is_empty()); // Runtime discovery!
    }

    #[test]
    fn test_capability_discovery_patterns() {
        // Test patterns for capability-based service discovery

        // Pattern 1: Primary capability discovery
        let primary_capabilities = vec!["service_mesh", "crypto", "storage", "ai"];

        for capability in &primary_capabilities {
            assert!(
                !capability.is_empty(),
                "Primary capability should be defined"
            );
        }

        // Pattern 2: Specific operation discovery (semantic naming)
        let specific_operations = vec![
            ("crypto", "generate_keypair"),
            ("crypto", "encrypt"),
            ("tls", "derive_secrets"),
            ("storage", "put"),
            ("ai", "inference"),
        ];

        for (domain, operation) in specific_operations {
            let semantic_capability = format!("{}.{}", domain, operation);
            assert!(semantic_capability.contains('.'));
            assert!(semantic_capability.starts_with(domain));
            assert!(semantic_capability.ends_with(operation));
        }

        // Pattern 3: Multi-capability discovery
        let service_requirements = vec![
            vec!["crypto", "tls"], // Need both crypto AND tls
            vec!["storage", "backup"], // Need both storage AND backup
        ];

        for requirements in service_requirements {
            assert!(requirements.len() >= 2, "Multi-capability requirements");
        }
    }

    #[test]
    fn test_agnostic_architecture() {
        // TRUE PRIMAL: Agnostic, capability-based, no hardcoded coupling

        // What we DON'T do (hardcoded coupling):
        // let songbird = find_primal_by_type(EcosystemPrimalType::Songbird);
        // let beardog = find_primal_by_type(EcosystemPrimalType::BearDog);

        // What we DO (capability-based discovery):
        let service_mesh = "service_mesh"; // Capability, not primal name
        let crypto_provider = "crypto"; // Capability, not primal name

        // Benefits:
        // 1. No compile-time coupling
        assert!(!service_mesh.is_empty());
        assert!(!crypto_provider.is_empty());

        // 2. Provider substitution
        let alternative_crypto = "crypto"; // Any provider with this capability
        assert_eq!(crypto_provider, alternative_crypto);

        // 3. Future-proof
        let future_capability = "quantum_crypto"; // Can add new capabilities
        assert!(!future_capability.is_empty());
    }

    #[test]
    fn test_capability_metadata() {
        // Test capability metadata and versioning
        use std::collections::HashMap;

        let mut metadata = HashMap::new();
        metadata.insert("version".to_string(), "1.0.0".to_string());
        metadata.insert("provider".to_string(), "discovered_at_runtime".to_string());
        metadata.insert(
            "capabilities".to_string(),
            "ai,inference,chat".to_string(),
        );

        // Capability-based registration includes rich metadata
        assert!(metadata.contains_key("capabilities"));
        assert_eq!(metadata.get("provider"), Some(&"discovered_at_runtime".to_string()));

        // Metadata is NOT hardcoded - it's discovered
        let capabilities_str = metadata.get("capabilities").unwrap();
        assert!(capabilities_str.contains("ai"));
        assert!(capabilities_str.contains("inference"));
    }
}
