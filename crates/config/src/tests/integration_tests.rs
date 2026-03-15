// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Integration tests for centralized configuration management
//!
//! These tests verify that the configuration system works correctly
//! across different components and handles environment overrides properly.

use std::collections::HashMap;
use std::env;
use std::time::Duration;

use crate::core::{Config, NetworkConfig, manager::DefaultConfigManager, types::*};
use crate::core::ai::AIConfig;
use crate::core::security::SecurityConfig;
use crate::core::ecosystem::EcosystemConfig;

#[cfg(test)]
mod config_integration_tests {
    use super::*;

    #[test]
    fn test_default_config_creation() {
        let config = Config::default();

        // Verify network defaults
        assert_eq!(config.network.host, "127.0.0.1");
        assert_eq!(config.network.port, 8080);
        assert_eq!(config.network.cors_origins, vec!["http://localhost:3000"]);

        // Verify AI config has required fields
        assert!(!config.ai.providers.is_empty() || config.ai.default_provider.is_empty());
        assert!(config.ai.max_retries > 0);
        assert!(config.ai.timeout.as_secs() > 0);

        // Verify security config
        assert!(!config.security.jwt_secret_key_id.is_empty());
        assert!(config.security.session_timeout.as_secs() > 0);
        assert!(config.security.max_failed_attempts > 0);

        // Verify ecosystem config
        assert!(matches!(config.ecosystem.mode, crate::core::ecosystem::EcosystemMode::Sovereign));
    }

    #[test]
    fn test_environment_variable_overrides() {
        // Set environment variables
        unsafe { env::set_var("SQUIRREL_HOST", "0.0.0.0") };
        unsafe { env::set_var("SQUIRREL_PORT", "9090") };
        unsafe { env::set_var("SQUIRREL_CORS_ORIGINS", "http://example.com,https://app.example.com") };

        let mut config = Config::default();
        
        // Apply environment overrides
        if let Ok(host) = env::var("SQUIRREL_HOST") {
            config.network.host = host;
        }
        if let Ok(port) = env::var("SQUIRREL_PORT") {
            if let Ok(port_num) = port.parse::<u16>() {
                config.network.port = port_num;
            }
        }
        if let Ok(origins) = env::var("SQUIRREL_CORS_ORIGINS") {
            config.network.cors_origins = origins.split(',').map(|s| s.to_string()).collect();
        }

        // Verify overrides were applied
        assert_eq!(config.network.host, "0.0.0.0");
        assert_eq!(config.network.port, 9090);
        assert_eq!(config.network.cors_origins, vec!["http://example.com", "https://app.example.com"]);

        // Clean up
        unsafe { env::remove_var("SQUIRREL_HOST") };
        unsafe { env::remove_var("SQUIRREL_PORT") };
        unsafe { env::remove_var("SQUIRREL_CORS_ORIGINS") };
    }

    #[test]
    fn test_config_validation() {
        let manager = DefaultConfigManager::new();

        // Test valid config
        let valid_config = Config::default();
        assert!(manager.validate_config(&valid_config).is_ok());

        // Test invalid network config
        let mut invalid_config = Config::default();
        invalid_config.network.port = 0;
        assert!(manager.validate_config(&invalid_config).is_err());

        // Test invalid host
        invalid_config.network.port = 8080;
        invalid_config.network.host = "".to_string();
        assert!(manager.validate_config(&invalid_config).is_err());
    }

    #[test]
    fn test_network_config_environment_integration() {
        // Test different network configurations
        let test_cases = vec![
            ("127.0.0.1", 8080, vec!["http://localhost:3000"]),
            ("0.0.0.0", 3000, vec!["https://example.com"]),
            ("192.168.1.100", 8443, vec!["http://localhost:8080", "https://app.local"]),
        ];

        for (host, port, origins) in test_cases {
            let mut config = Config::default();
            config.network.host = host.to_string();
            config.network.port = port;
            config.network.cors_origins = origins.iter().map(|s| s.to_string()).collect();

            // Verify configuration is valid
            let manager = DefaultConfigManager::new();
            assert!(manager.validate_config(&config).is_ok());

            // Verify values are set correctly
            assert_eq!(config.network.host, host);
            assert_eq!(config.network.port, port);
            assert_eq!(config.network.cors_origins.len(), origins.len());
        }
    }

    #[test]
    fn test_ai_config_validation() {
        let manager = DefaultConfigManager::new();

        // Test valid AI config
        let mut config = Config::default();
        config.ai = AIConfig {
            providers: vec![crate::core::ai::AIProvider {
                name: "test_provider".to_string(),
                provider_type: crate::core::ai::AIProviderType::OpenAI,
                endpoint: "https://api.openai.com/v1".to_string(),
                api_key: "test_key".to_string(),
                model: "gpt-4".to_string(),
                max_tokens: 4000,
                temperature: 0.7,
                priority: 1,
                enabled: true,
                rate_limit: crate::core::ai::RateLimit {
                    requests_per_minute: 60,
                    tokens_per_minute: 100000,
                },
            }],
            default_provider: "test_provider".to_string(),
            max_retries: 3,
            timeout: Duration::from_secs(30),
            fallback_enabled: true,
            health_check_interval: Duration::from_secs(60),
        };

        assert!(manager.validate_config(&config).is_ok());

        // Test invalid AI config - no providers
        config.ai.providers.clear();
        assert!(manager.validate_config(&config).is_err());

        // Test invalid AI config - zero retries
        config.ai.providers.push(crate::core::ai::AIProvider {
            name: "test".to_string(),
            provider_type: crate::core::ai::AIProviderType::OpenAI,
            endpoint: "test".to_string(),
            api_key: "test".to_string(),
            model: "test".to_string(),
            max_tokens: 1000,
            temperature: 0.5,
            priority: 1,
            enabled: true,
            rate_limit: crate::core::ai::RateLimit {
                requests_per_minute: 60,
                tokens_per_minute: 10000,
            },
        });
        config.ai.max_retries = 0;
        assert!(manager.validate_config(&config).is_err());
    }

    #[test]
    fn test_security_config_validation() {
        let manager = DefaultConfigManager::new();

        // Test valid security config
        let mut config = Config::default();
        config.security = SecurityConfig {
            backend: crate::core::security::SecurityBackend::Internal,
            jwt_secret_key_id: "test_key_id".to_string(),
            jwt_expiration: Duration::from_secs(3600),
            encryption_algorithm: "AES-256-GCM".to_string(),
            hsm_provider: "internal".to_string(),
            authentication_required: true,
            session_timeout: Duration::from_secs(1800),
            max_failed_attempts: 5,
            lockout_duration: Duration::from_secs(300),
        };

        assert!(manager.validate_config(&config).is_ok());

        // Test invalid security config - empty JWT key
        config.security.jwt_secret_key_id = "".to_string();
        assert!(manager.validate_config(&config).is_err());

        // Test invalid security config - short session timeout
        config.security.jwt_secret_key_id = "valid_key".to_string();
        config.security.session_timeout = Duration::from_secs(30);
        assert!(manager.validate_config(&config).is_err());
    }

    #[test]
    fn test_ecosystem_config_validation() {
        let manager = DefaultConfigManager::new();

        // Test valid ecosystem config
        let mut config = Config::default();
        config.ecosystem = EcosystemConfig {
            enabled: true,
            mode: crate::core::ecosystem::EcosystemMode::Coordinated,
            discovery: crate::core::ecosystem::DiscoveryConfig {
                songbird_endpoint: Some("http://songbird.local:8080".to_string()),
                auto_discovery: true,
                probe_interval: Duration::from_secs(30),
                direct_endpoints: HashMap::new(),
                health_check_timeout: Duration::from_secs(10),
            },
            coordination: crate::core::ecosystem::CoordinationConfig {
                nestgate: None,
                beardog: None,
                toadstool: None,
                fallback_strategies: HashMap::new(),
            },
            biome_manifest: crate::core::ecosystem::BiomeManifestConfig {
                enabled: true,
                auto_generate: true,
                include_capabilities: true,
                include_dependencies: true,
            },
        };

        assert!(manager.validate_config(&config).is_ok());

        // Test invalid ecosystem config - zero health check timeout
        config.ecosystem.discovery.health_check_timeout = Duration::from_secs(0);
        assert!(manager.validate_config(&config).is_err());
    }

    #[test]
    fn test_config_serialization_deserialization() {
        let original_config = Config::default();

        // Test JSON serialization
        let json_str = serde_json::to_string(&original_config).unwrap();
        let deserialized_config: Config = serde_json::from_str(&json_str).unwrap();

        // Verify critical fields match
        assert_eq!(original_config.network.host, deserialized_config.network.host);
        assert_eq!(original_config.network.port, deserialized_config.network.port);
        assert_eq!(original_config.ai.max_retries, deserialized_config.ai.max_retries);
        assert_eq!(original_config.security.jwt_secret_key_id, deserialized_config.security.jwt_secret_key_id);
    }

    #[test]
    fn test_config_manager_required_sections() {
        let manager = DefaultConfigManager::new();
        let config = Config::default();

        let missing_sections = manager.check_required_config_sections(&config);

        // Should not have missing required sections for default config
        assert!(missing_sections.is_empty() || missing_sections.iter().all(|s| {
            // Allow missing optional configurations
            !s.contains("network.host") && !s.contains("ai.providers")
        }));
    }

    #[test]
    fn test_partial_config_updates() {
        let mut config = Config::default();
        let original_port = config.network.port;

        // Test partial network config update
        config.network.host = "192.168.1.100".to_string();
        config.network.cors_origins.push("https://newapp.com".to_string());

        // Verify other fields remain unchanged
        assert_eq!(config.network.port, original_port);
        assert_eq!(config.network.host, "192.168.1.100");
        assert!(config.network.cors_origins.contains(&"https://newapp.com".to_string()));

        // Validate the updated config
        let manager = DefaultConfigManager::new();
        assert!(manager.validate_config(&config).is_ok());
    }
}

#[cfg(test)]
mod config_performance_tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_config_creation_performance() {
        let start = Instant::now();
        let _configs: Vec<Config> = (0..1000).map(|_| Config::default()).collect();
        let elapsed = start.elapsed();

        println!("Created 1000 default configs in {:?}", elapsed);
        assert!(elapsed.as_millis() < 1000, "Config creation too slow: {:?}", elapsed);
    }

    #[test]
    fn test_config_validation_performance() {
        let manager = DefaultConfigManager::new();
        let config = Config::default();

        let start = Instant::now();
        for _ in 0..100 {
            assert!(manager.validate_config(&config).is_ok());
        }
        let elapsed = start.elapsed();

        println!("Validated config 100 times in {:?}", elapsed);
        assert!(elapsed.as_millis() < 500, "Config validation too slow: {:?}", elapsed);
    }

    #[test]
    fn test_config_serialization_performance() {
        let config = Config::default();

        let start = Instant::now();
        for _ in 0..100 {
            let _json = serde_json::to_string(&config).unwrap();
        }
        let elapsed = start.elapsed();

        println!("Serialized config 100 times in {:?}", elapsed);
        assert!(elapsed.as_millis() < 500, "Config serialization too slow: {:?}", elapsed);
    }
}

#[cfg(test)]
mod config_edge_case_tests {
    use super::*;

    #[test]
    fn test_extreme_values() {
        let manager = DefaultConfigManager::new();

        // Test maximum port value
        let mut config = Config::default();
        config.network.port = 65535;
        assert!(manager.validate_config(&config).is_ok());

        // Test minimum valid port
        config.network.port = 1;
        assert!(manager.validate_config(&config).is_ok());

        // Test very long CORS origins list
        config.network.cors_origins = (0..100)
            .map(|i| format!("https://app{}.example.com", i))
            .collect();
        assert!(manager.validate_config(&config).is_ok());
    }

    #[test]
    fn test_unicode_and_special_characters() {
        let mut config = Config::default();

        // Test Unicode in host (should be valid for some use cases)
        config.network.host = "localност".to_string(); // Cyrillic characters
        
        // Test special characters in CORS origins
        config.network.cors_origins = vec![
            "https://app-test.example.com".to_string(),
            "http://localhost:3000/app#fragment".to_string(),
            "https://api.example.com/v1?param=value".to_string(),
        ];

        let manager = DefaultConfigManager::new();
        let result = manager.validate_config(&config);
        
        // Configuration validation might reject Unicode hosts, but should handle CORS origins
        if result.is_err() {
            config.network.host = "localhost".to_string();
            assert!(manager.validate_config(&config).is_ok());
        }
    }

    #[test]
    fn test_empty_and_null_values() {
        let manager = DefaultConfigManager::new();
        let mut config = Config::default();

        // Test empty CORS origins
        config.network.cors_origins = vec![];
        assert!(manager.validate_config(&config).is_ok());

        // Test empty AI providers list (should fail)
        config.ai.providers = vec![];
        assert!(manager.validate_config(&config).is_err());
    }
} 