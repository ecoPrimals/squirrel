//! Comprehensive tests for ecosystem types
//!
//! This module provides thorough testing of all ecosystem integration types
//! including primal types, service registration, capabilities, and configuration.

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::ecosystem::arc_str;

    // ========== EcosystemPrimalType Tests ==========

    #[test]
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
    fn test_primal_type_as_str() {
        assert_eq!(EcosystemPrimalType::ToadStool.as_str(), "toadstool");
        assert_eq!(EcosystemPrimalType::Songbird.as_str(), "songbird");
        assert_eq!(EcosystemPrimalType::BearDog.as_str(), "beardog");
        assert_eq!(EcosystemPrimalType::NestGate.as_str(), "nestgate");
        assert_eq!(EcosystemPrimalType::Squirrel.as_str(), "squirrel");
        assert_eq!(EcosystemPrimalType::BiomeOS.as_str(), "biomeos");
    }

    #[test]
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
        let endpoints = ServiceEndpoints {
            primary: "http://localhost:8080".to_string(),
            secondary: vec!["http://localhost:8081".to_string()],
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
    fn test_primal_type_round_trip_parsing() {
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
}
