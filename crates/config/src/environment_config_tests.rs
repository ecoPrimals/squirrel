// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Comprehensive tests for environment configuration
//!
//! These tests significantly improve coverage for the config module.

#[cfg(test)]
mod tests {
    use crate::environment_config::*;
    use std::env;

    #[test]
    fn test_environment_from_env_all_variants() {
        // Test production variants
        for prod_env in &["production", "prod", "PRODUCTION", "PROD"] {
            env::set_var("SQUIRREL_ENV", prod_env);
            assert_eq!(Environment::from_env(), Environment::Production);
        }

        // Test staging variants
        for staging_env in &["staging", "stage", "STAGING", "STAGE"] {
            env::set_var("SQUIRREL_ENV", staging_env);
            assert_eq!(Environment::from_env(), Environment::Staging);
        }

        // Test testing variants
        for test_env in &["testing", "test", "TESTING", "TEST"] {
            env::set_var("SQUIRREL_ENV", test_env);
            assert_eq!(Environment::from_env(), Environment::Testing);
        }

        // Test development (default)
        for dev_env in &["development", "dev", "DEVELOPMENT", "DEV", "", "invalid"] {
            env::set_var("SQUIRREL_ENV", dev_env);
            assert_eq!(Environment::from_env(), Environment::Development);
        }

        // Test missing env var
        env::remove_var("SQUIRREL_ENV");
        assert_eq!(Environment::from_env(), Environment::Development);
    }

    #[test]
    fn test_environment_config_files() {
        assert_eq!(
            Environment::Development.config_file(),
            "config/development.toml"
        );
        assert_eq!(Environment::Testing.config_file(), "config/testing.toml");
        assert_eq!(Environment::Staging.config_file(), "config/production.toml");
        assert_eq!(
            Environment::Production.config_file(),
            "config/production.toml"
        );
    }

    #[test]
    fn test_environment_equality() {
        assert_eq!(Environment::Development, Environment::Development);
        assert_ne!(Environment::Development, Environment::Production);
        assert_ne!(Environment::Testing, Environment::Production);
        assert_eq!(Environment::Staging, Environment::Staging);
    }

    #[test]
    fn test_environment_debug() {
        let env = Environment::Development;
        let debug_str = format!("{:?}", env);
        assert!(debug_str.contains("Development"));
    }

    #[test]
    fn test_environment_clone() {
        let env1 = Environment::Production;
        let env2 = env1;
        assert_eq!(env1, env2);
    }

    #[test]
    fn test_network_config_construction() {
        let config = NetworkConfig {
            default_host: "0.0.0.0".to_string(),
            default_port: 8080,
            health_check_port: 8081,
            metrics_port: 9090,
        };

        assert_eq!(config.default_host, "0.0.0.0");
        assert_eq!(config.default_port, 8080);
        assert_eq!(config.health_check_port, 8081);
        assert_eq!(config.metrics_port, 9090);
    }

    #[test]
    fn test_database_config_construction() {
        let config = DatabaseConfig {
            host: "localhost".to_string(),
            port: 5432,
            database: "test_db".to_string(),
            max_connections: 100,
        };

        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 5432);
        assert_eq!(config.database, "test_db");
        assert_eq!(config.max_connections, 100);
    }

    #[test]
    fn test_mcp_config_construction() {
        let config = McpConfig {
            server_host: "127.0.0.1".to_string(),
            server_port: 3000,
            websocket_port: 3001,
            max_connections: 1000,
        };

        assert_eq!(config.server_host, "127.0.0.1");
        assert_eq!(config.server_port, 3000);
        assert_eq!(config.websocket_port, 3001);
        assert_eq!(config.max_connections, 1000);
    }

    #[test]
    fn test_ai_tools_config_construction() {
        let config = AIToolsConfig {
            anthropic_url: "https://api.anthropic.com".to_string(),
            openai_url: "https://api.openai.com".to_string(),
            local_server_url: "http://localhost:11434".to_string(),
        };

        assert!(config.anthropic_url.starts_with("https://"));
        assert!(config.openai_url.contains("openai"));
        assert!(config.local_server_url.contains("localhost"));
    }

    #[test]
    fn test_security_config_construction() {
        let config = SecurityConfig {
            beardog_url: "http://localhost:8443".to_string(),
            auth_timeout_secs: 300,
            session_timeout_secs: 3600,
        };

        assert_eq!(config.beardog_url, "http://localhost:8443");
        assert_eq!(config.auth_timeout_secs, 300);
        assert_eq!(config.session_timeout_secs, 3600);
    }

    #[test]
    fn test_limits_config_construction() {
        let config = LimitsConfig {
            max_request_size_bytes: 10485760,
            max_response_size_bytes: 52428800,
            request_timeout_secs: 30,
            connection_timeout_secs: 5,
        };

        assert_eq!(config.max_request_size_bytes, 10485760);
        assert_eq!(config.max_response_size_bytes, 52428800);
        assert_eq!(config.request_timeout_secs, 30);
        assert_eq!(config.connection_timeout_secs, 5);
    }

    #[test]
    fn test_feature_flags_construction() {
        let flags = FeatureFlags {
            enable_chaos_testing: true,
            enable_experimental_features: false,
            enable_telemetry: true,
            enable_metrics: true,
        };

        assert!(flags.enable_chaos_testing);
        assert!(!flags.enable_experimental_features);
        assert!(flags.enable_telemetry);
        assert!(flags.enable_metrics);
    }

    #[test]
    fn test_observability_config_construction() {
        let config = ObservabilityConfig {
            jaeger_endpoint: "http://localhost:14268".to_string(),
            prometheus_port: 9090,
            grafana_port: 3000,
        };

        assert!(
            config.jaeger_endpoint.contains("jaeger")
                || config.jaeger_endpoint.contains("localhost")
        );
        assert_eq!(config.prometheus_port, 9090);
        assert_eq!(config.grafana_port, 3000);
    }

    #[test]
    fn test_service_discovery_config_construction() {
        let config = ServiceDiscoveryConfig {
            registry_url: "http://localhost:8500".to_string(),
            consul_address: "127.0.0.1:8500".to_string(),
            etcd_endpoints: vec!["http://localhost:2379".to_string()],
        };

        assert_eq!(config.registry_url, "http://localhost:8500");
        assert_eq!(config.consul_address, "127.0.0.1:8500");
        assert_eq!(config.etcd_endpoints.len(), 1);
    }

    #[test]
    fn test_config_clone() {
        let config = NetworkConfig {
            default_host: "0.0.0.0".to_string(),
            default_port: 8080,
            health_check_port: 8081,
            metrics_port: 9090,
        };

        let cloned = config.clone();
        assert_eq!(config.default_host, cloned.default_host);
        assert_eq!(config.default_port, cloned.default_port);
    }

    #[test]
    fn test_config_debug_formatting() {
        let config = NetworkConfig {
            default_host: "test".to_string(),
            default_port: 1234,
            health_check_port: 1235,
            metrics_port: 1236,
        };

        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("NetworkConfig"));
        assert!(debug_str.contains("test"));
        assert!(debug_str.contains("1234"));
    }
}
