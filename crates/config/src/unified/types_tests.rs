// SPDX-License-Identifier: AGPL-3.0-only
// ORC-Notice: Configuration type mechanics licensed under ORC
// Copyright (C) 2026 ecoPrimals Contributors

//! Tests for unified configuration types.

use std::collections::HashMap;

use super::types::*;

#[test]
fn test_squirrel_unified_config_default() {
    let config = SquirrelUnifiedConfig::default();
    assert!(!config.system.instance_id.is_empty());
    assert_eq!(config.system.environment, "development");
    assert!(!config.system.log_level.is_empty());
    assert!(config.network.http_port > 0);
    assert!(config.network.websocket_port > 0);
    assert!(config.security.enabled);
    assert_eq!(config.mcp.version, "1.0");
    assert!(config.service_mesh.enabled);
    assert!(config.monitoring.enabled);
    assert!(config.features.enable_plugins);
}

#[test]
fn test_network_config_default_values() {
    let config = SquirrelUnifiedConfig::default();
    assert!(!config.network.bind_address.is_empty());
    assert_eq!(config.network.max_connections, 100);
    assert!(!config.network.enable_tls);
}

#[test]
fn test_load_balancing_strategy_default() {
    let strategy = LoadBalancingStrategy::default();
    assert!(matches!(strategy, LoadBalancingStrategy::RoundRobin));
}

#[test]
fn test_load_balancing_strategy_serde() {
    let strategies = vec![
        LoadBalancingStrategy::RoundRobin,
        LoadBalancingStrategy::Random,
        LoadBalancingStrategy::LeastConnections,
        LoadBalancingStrategy::HealthBased,
    ];
    for strategy in strategies {
        let json = serde_json::to_string(&strategy).unwrap();
        let decoded: LoadBalancingStrategy = serde_json::from_str(&json).unwrap();
        assert!(std::mem::discriminant(&strategy) == std::mem::discriminant(&decoded));
    }
}

#[test]
fn test_database_backend_default() {
    let backend = DatabaseBackend::default();
    assert!(matches!(backend, DatabaseBackend::SQLite));
}

#[test]
fn test_service_registry_type_in_memory_serde() {
    let registry = ServiceRegistryType::InMemory;
    let json = serde_json::to_string(&registry).unwrap();
    assert!(json.contains("in_memory"));
    let decoded: ServiceRegistryType = serde_json::from_str(&json).unwrap();
    assert!(matches!(decoded, ServiceRegistryType::InMemory));
}

#[test]
fn test_service_registry_type_file_serde() {
    let registry = ServiceRegistryType::File {
        path: "/tmp/registry.json".to_string(),
    };
    let json = serde_json::to_string(&registry).unwrap();
    let decoded: ServiceRegistryType = serde_json::from_str(&json).unwrap();
    if let ServiceRegistryType::File { path } = decoded {
        assert_eq!(path, "/tmp/registry.json");
    } else {
        panic!("Expected File variant");
    }
}

#[test]
fn test_provider_config_serde() {
    let provider = ProviderConfig {
        endpoint: "https://api.openai.com".to_string(),
        api_key: Some("sk-xxx".to_string()),
        enabled: true,
        settings: HashMap::new(),
    };
    let json = serde_json::to_string(&provider).unwrap();
    let decoded: ProviderConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.endpoint, provider.endpoint);
    assert_eq!(decoded.api_key, provider.api_key);
    assert!(decoded.enabled);
}

#[test]
fn test_circuit_breaker_config_default() {
    let cb = CircuitBreakerConfig::default();
    assert!(cb.enabled);
    assert!(cb.failure_threshold > 0);
    assert!(cb.success_threshold > 0);
    assert!(cb.timeout_secs > 0);
}

#[test]
fn test_config_validate_security_disabled() {
    let mut config = SquirrelUnifiedConfig::default();
    config.security.enabled = false;
    config.security.require_authentication = false;
    // Pin all values that validation checks — env vars from parallel tests can pollute defaults
    config.network.http_port = 8080;
    config.network.websocket_port = 8081;
    config.monitoring.prometheus_port = 9090;
    config.timeouts.health_check_timeout_secs = 5;
    config.timeouts.connection_timeout_secs = 30;
    config.timeouts.request_timeout_secs = 60;
    config.timeouts.session_timeout_secs = 3600;
    assert!(
        config.validate().is_ok(),
        "validation errors: {:?}",
        config.validate().unwrap_err()
    );
}

#[test]
fn test_config_validate_security_enabled_no_auth() {
    let mut config = SquirrelUnifiedConfig::default();
    config.security.enabled = true;
    config.security.require_authentication = false;
    config.security.jwt_secret = None;
    config.security.api_keys = vec![];
    // Pin all values that validation checks — env vars from parallel tests can pollute defaults
    config.network.http_port = 8080;
    config.network.websocket_port = 8081;
    config.monitoring.prometheus_port = 9090;
    config.timeouts.health_check_timeout_secs = 5;
    config.timeouts.connection_timeout_secs = 30;
    config.timeouts.request_timeout_secs = 60;
    config.timeouts.session_timeout_secs = 3600;
    assert!(
        config.validate().is_ok(),
        "validation errors: {:?}",
        config.validate().unwrap_err()
    );
}

#[test]
fn test_config_serde_roundtrip_minimal() {
    let config = SquirrelUnifiedConfig::default();
    let json = serde_json::to_string(&config).unwrap();
    let decoded: SquirrelUnifiedConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(config.system.environment, decoded.system.environment);
    assert_eq!(config.network.http_port, decoded.network.http_port);
}

#[test]
fn test_feature_flags_default() {
    let config = SquirrelUnifiedConfig::default();
    assert!(!config.features.experimental);
    assert!(config.features.enable_plugins);
    assert!(!config.features.enable_federation);
    assert!(config.features.enable_advanced_routing);
}

#[test]
fn test_database_config_default() {
    let db = DatabaseConfig::default();
    assert!(!db.connection_string.is_empty());
    assert!(db.max_connections > 0);
    assert!(db.enable_pooling);
}
