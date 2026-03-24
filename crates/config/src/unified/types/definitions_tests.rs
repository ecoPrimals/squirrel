// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Serde round-trip and structural tests for configuration definition types.

use super::*;
use crate::unified::timeouts::TimeoutConfig;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::path::PathBuf;

fn assert_serde_json_roundtrip<T>(value: &T)
where
    T: Serialize + DeserializeOwned + core::fmt::Debug,
{
    let json = serde_json::to_string(value).expect("should succeed");
    let back: T = serde_json::from_str(&json).expect("should succeed");
    assert_eq!(
        serde_json::to_value(value).expect("should succeed"),
        serde_json::to_value(&back).expect("should succeed"),
        "JSON round-trip mismatch for type {}",
        core::any::type_name::<T>()
    );
}

fn sample_timeouts() -> TimeoutConfig {
    TimeoutConfig {
        connection_timeout_secs: 30,
        request_timeout_secs: 60,
        health_check_timeout_secs: 5,
        operation_timeout_secs: 10,
        database_timeout_secs: 30,
        heartbeat_interval_secs: 30,
        discovery_timeout_secs: 10,
        ai_inference_timeout_secs: 120,
        plugin_load_timeout_secs: 15,
        session_timeout_secs: 3600,
        custom_timeouts: HashMap::from([("edge_op".to_string(), u64::MAX)]),
    }
}

#[expect(
    clippy::too_many_lines,
    reason = "Test code: explicit unwrap/expect and local lint noise"
)]
fn sample_squirrel_unified_config() -> SquirrelUnifiedConfig {
    let mut providers = HashMap::new();
    providers.insert(
        "openai".to_string(),
        ProviderConfig {
            endpoint: "https://api.example/v1".to_string(),
            api_key: Some("secret".to_string()),
            enabled: true,
            settings: HashMap::from([("temperature".to_string(), serde_json::json!(0.7_f64))]),
        },
    );

    SquirrelUnifiedConfig {
        system: SystemConfig {
            instance_id: "test-instance".to_string(),
            environment: "staging".to_string(),
            log_level: "info".to_string(),
            work_dir: PathBuf::from("/var/work"),
            data_dir: PathBuf::from("/var/data"),
            plugin_dir: PathBuf::from("/var/plugins"),
        },
        network: NetworkConfig {
            bind_address: "0.0.0.0".to_string(),
            http_port: 8080,
            websocket_port: 8081,
            grpc_port: 50051,
            max_connections: 500,
            enable_tls: false,
            tls_cert_path: None,
            tls_key_path: None,
        },
        security: SecurityConfig {
            enabled: true,
            require_authentication: true,
            enable_authorization: true,
            jwt_secret: None,
            token_expiration_secs: 3600,
            api_keys: vec!["k1".to_string()],
            allowed_origins: vec!["https://app.example".to_string()],
            tls_enabled: false,
            tls_cert_path: None,
            tls_key_path: None,
            ca_cert_path: None,
            mtls_enabled: false,
            encryption_default_format: "AES256GCM".to_string(),
            enable_audit: true,
            enable_encryption: true,
            enable_rbac: true,
            token_expiry_minutes: 60,
        },
        mcp: McpConfig {
            version: "1.0".to_string(),
            max_message_size: 16 * 1024 * 1024,
            buffer_size: 8192,
            enable_compression: false,
            compression_level: 6,
        },
        ai: AiProvidersConfig {
            default_endpoint: "https://ai.default".to_string(),
            providers,
            enable_local: true,
            enable_cloud: true,
            max_concurrent_requests: 10,
        },
        service_mesh: ServiceMeshConfig {
            enabled: true,
            discovery_endpoints: vec!["http://disc:9000".to_string()],
            registry_type: ServiceRegistryType::InMemory,
            max_services: 1000,
            health_check_interval_secs: 30,
            heartbeat_interval_secs: 15,
            service_expiration_secs: 90,
            enable_failover: true,
            metrics_enabled: true,
            namespace: Some("ns1".to_string()),
        },
        timeouts: sample_timeouts(),
        monitoring: MonitoringConfig {
            enabled: true,
            metrics_endpoint: "/metrics".to_string(),
            tracing_endpoint: Some("http://trace:4317".to_string()),
            enable_prometheus: true,
            prometheus_port: 9090,
        },
        database: DatabaseConfig {
            connection_string: "sqlite::memory:".to_string(),
            max_connections: 10,
            timeout_seconds: 30,
            backend: DatabaseBackend::SQLite,
            enable_pooling: true,
            pool_size: 5,
        },
        load_balancing: LoadBalancingConfig {
            strategy: LoadBalancingStrategy::RoundRobin,
            sticky_sessions: false,
            session_timeout_secs: 3600,
            circuit_breaker: CircuitBreakerConfig {
                enabled: true,
                failure_threshold: 5,
                success_threshold: 3,
                timeout_secs: 60,
                half_open_max_requests: 3,
            },
            health_based_routing: true,
            retry_failed: true,
            max_retries: 3,
        },
        features: FeatureFlags {
            experimental: false,
            enable_plugins: true,
            enable_federation: false,
            enable_advanced_routing: true,
            custom: HashMap::from([("beta_ui".to_string(), true)]),
        },
        custom: HashMap::from([(
            "primal".to_string(),
            serde_json::json!({ "mode": "strict" }),
        )]),
    }
}

#[test]
fn database_backend_default_is_sqlite() {
    assert!(matches!(
        DatabaseBackend::default(),
        DatabaseBackend::SQLite
    ));
}

#[test]
fn database_backend_serializes_roundtrip() {
    for backend in [
        DatabaseBackend::NestGate,
        DatabaseBackend::PostgreSQL,
        DatabaseBackend::SQLite,
        DatabaseBackend::Memory,
    ] {
        assert_serde_json_roundtrip(&backend);
    }
}

#[test]
fn load_balancing_strategy_default_is_round_robin() {
    assert!(matches!(
        LoadBalancingStrategy::default(),
        LoadBalancingStrategy::RoundRobin
    ));
}

#[test]
fn load_balancing_strategy_serializes_roundtrip() {
    for strategy in [
        LoadBalancingStrategy::RoundRobin,
        LoadBalancingStrategy::Random,
        LoadBalancingStrategy::LeastConnections,
        LoadBalancingStrategy::WeightedRoundRobin,
        LoadBalancingStrategy::HealthBased,
        LoadBalancingStrategy::ResponseTime,
        LoadBalancingStrategy::ConsistentHash,
    ] {
        assert_serde_json_roundtrip(&strategy);
    }
}

#[test]
fn system_config_constructed_and_serializes_roundtrip() {
    let cfg = SystemConfig {
        instance_id: "id-1".to_string(),
        environment: "dev".to_string(),
        log_level: "trace".to_string(),
        work_dir: PathBuf::from("/w"),
        data_dir: PathBuf::from("/d"),
        plugin_dir: PathBuf::from("/p"),
    };
    assert_serde_json_roundtrip(&cfg);
    let debug = format!("{cfg:?}");
    assert!(debug.contains("id-1"));
    assert_serde_json_roundtrip(&cfg);
}

#[test]
fn system_config_deserializes_minimal_json_with_serde_defaults() {
    let cfg: SystemConfig = serde_json::from_str("{}").expect("should succeed");
    assert!(!cfg.instance_id.is_empty());
    let debug = format!("{cfg:?}");
    assert!(debug.contains("instance_id"));
}

#[test]
fn network_config_serializes_roundtrip() {
    let cfg = NetworkConfig {
        bind_address: "127.0.0.1".to_string(),
        http_port: 3000,
        websocket_port: 3001,
        grpc_port: 3002,
        max_connections: 42,
        enable_tls: true,
        tls_cert_path: Some(PathBuf::from("/tmp/cert.pem")),
        tls_key_path: Some(PathBuf::from("/tmp/key.pem")),
    };
    assert_serde_json_roundtrip(&cfg);
}

#[test]
fn network_config_accepts_port_boundaries() {
    let cfg = NetworkConfig {
        bind_address: "::1".to_string(),
        http_port: u16::MAX,
        websocket_port: 0,
        grpc_port: 1,
        max_connections: u32::MAX,
        enable_tls: false,
        tls_cert_path: None,
        tls_key_path: None,
    };
    assert_serde_json_roundtrip(&cfg);
    assert_eq!(cfg.http_port, u16::MAX);
    assert_eq!(cfg.max_connections, u32::MAX);
}

#[test]
fn security_config_serializes_roundtrip() {
    let cfg = SecurityConfig {
        enabled: false,
        require_authentication: false,
        enable_authorization: false,
        jwt_secret: Some("jwt".to_string()),
        token_expiration_secs: 7200,
        api_keys: vec![],
        allowed_origins: vec![],
        tls_enabled: true,
        tls_cert_path: Some("/c".to_string()),
        tls_key_path: Some("/k".to_string()),
        ca_cert_path: Some("/ca".to_string()),
        mtls_enabled: true,
        encryption_default_format: "CHACHA20".to_string(),
        enable_audit: false,
        enable_encryption: false,
        enable_rbac: false,
        token_expiry_minutes: 120,
    };
    assert_serde_json_roundtrip(&cfg);
}

#[test]
fn security_config_numeric_fields_support_large_values() {
    let cfg = SecurityConfig {
        enabled: true,
        require_authentication: true,
        enable_authorization: true,
        jwt_secret: None,
        token_expiration_secs: u64::MAX,
        api_keys: vec![],
        allowed_origins: vec![],
        tls_enabled: false,
        tls_cert_path: None,
        tls_key_path: None,
        ca_cert_path: None,
        mtls_enabled: false,
        encryption_default_format: "AES256GCM".to_string(),
        enable_audit: true,
        enable_encryption: true,
        enable_rbac: true,
        token_expiry_minutes: u64::MAX,
    };
    assert_serde_json_roundtrip(&cfg);
}

#[test]
fn mcp_config_serializes_roundtrip() {
    let cfg = McpConfig {
        version: "2.0".to_string(),
        max_message_size: 1024,
        buffer_size: 512,
        enable_compression: true,
        compression_level: 9,
    };
    assert_serde_json_roundtrip(&cfg);
}

#[test]
fn mcp_config_message_and_buffer_size_boundaries() {
    let cfg = McpConfig {
        version: "1.0".to_string(),
        max_message_size: 0,
        buffer_size: usize::MAX,
        enable_compression: false,
        compression_level: 0,
    };
    assert_serde_json_roundtrip(&cfg);
}

#[test]
fn ai_providers_config_serializes_roundtrip() {
    let mut providers = HashMap::new();
    providers.insert(
        "local".to_string(),
        ProviderConfig {
            endpoint: "http://127.0.0.1:11434".to_string(),
            api_key: None,
            enabled: true,
            settings: HashMap::new(),
        },
    );
    let cfg = AiProvidersConfig {
        default_endpoint: String::new(),
        providers,
        enable_local: false,
        enable_cloud: false,
        max_concurrent_requests: usize::MAX,
    };
    assert_serde_json_roundtrip(&cfg);
}

#[test]
fn provider_config_preserves_float_setting_within_epsilon() {
    let mut settings = HashMap::new();
    let expected = 1.5_f64;
    settings.insert("ratio".to_string(), serde_json::json!(expected));
    let cfg = ProviderConfig {
        endpoint: "http://x".to_string(),
        api_key: None,
        enabled: true,
        settings,
    };
    assert_serde_json_roundtrip(&cfg);
    let back: ProviderConfig =
        serde_json::from_str(&serde_json::to_string(&cfg).expect("should succeed"))
            .expect("should succeed");
    let ratio = back
        .settings
        .get("ratio")
        .and_then(serde_json::Value::as_f64)
        .expect("should succeed");
    assert!((ratio - expected).abs() < f64::EPSILON);
}

#[test]
fn service_mesh_config_serializes_roundtrip() {
    let cfg = ServiceMeshConfig {
        enabled: false,
        discovery_endpoints: vec![],
        registry_type: ServiceRegistryType::InMemory,
        max_services: 0,
        health_check_interval_secs: u64::MAX,
        heartbeat_interval_secs: 1,
        service_expiration_secs: 0,
        enable_failover: false,
        metrics_enabled: false,
        namespace: None,
    };
    assert_serde_json_roundtrip(&cfg);
}

#[test]
fn service_registry_type_in_memory_serializes_roundtrip() {
    let t = ServiceRegistryType::InMemory;
    assert_serde_json_roundtrip(&t);
}

#[test]
fn service_registry_type_file_serializes_roundtrip() {
    let t = ServiceRegistryType::File {
        path: "/reg/services.json".to_string(),
    };
    assert_serde_json_roundtrip(&t);
}

#[test]
fn service_registry_type_network_serializes_roundtrip() {
    let t = ServiceRegistryType::Network {
        endpoints: vec!["consul:8500".to_string(), "etcd:2379".to_string()],
    };
    assert_serde_json_roundtrip(&t);
}

#[test]
fn service_registry_type_redis_serializes_roundtrip() {
    let t = ServiceRegistryType::Redis {
        connection_string: "redis://localhost:6379/0".to_string(),
    };
    assert_serde_json_roundtrip(&t);
}

#[test]
fn service_registry_type_database_serializes_roundtrip() {
    let t = ServiceRegistryType::Database {
        connection_string: "postgres://localhost/db".to_string(),
    };
    assert_serde_json_roundtrip(&t);
}

#[test]
fn service_registry_type_custom_serializes_roundtrip() {
    let t = ServiceRegistryType::Custom {
        config: HashMap::from([("a".to_string(), "b".to_string())]),
    };
    assert_serde_json_roundtrip(&t);
}

#[test]
fn monitoring_config_serializes_roundtrip() {
    let cfg = MonitoringConfig {
        enabled: true,
        metrics_endpoint: "/m".to_string(),
        tracing_endpoint: None,
        enable_prometheus: false,
        prometheus_port: u16::MAX,
    };
    assert_serde_json_roundtrip(&cfg);
}

#[test]
fn feature_flags_serializes_roundtrip() {
    let cfg = FeatureFlags {
        experimental: true,
        enable_plugins: false,
        enable_federation: true,
        enable_advanced_routing: false,
        custom: HashMap::from([("x".to_string(), false)]),
    };
    assert_serde_json_roundtrip(&cfg);
}

#[test]
fn database_config_serializes_roundtrip() {
    let cfg = DatabaseConfig {
        connection_string: "sqlite::memory:".to_string(),
        max_connections: u32::MAX,
        timeout_seconds: 1,
        backend: DatabaseBackend::PostgreSQL,
        enable_pooling: false,
        pool_size: 0,
    };
    assert_serde_json_roundtrip(&cfg);
}

#[test]
fn load_balancing_config_serializes_roundtrip() {
    let cfg = LoadBalancingConfig {
        strategy: LoadBalancingStrategy::LeastConnections,
        sticky_sessions: true,
        session_timeout_secs: 0,
        circuit_breaker: CircuitBreakerConfig {
            enabled: false,
            failure_threshold: u32::MAX,
            success_threshold: 0,
            timeout_secs: u64::MAX,
            half_open_max_requests: u32::MAX,
        },
        health_based_routing: false,
        retry_failed: false,
        max_retries: u32::MAX,
    };
    assert_serde_json_roundtrip(&cfg);
}

#[test]
fn circuit_breaker_config_serializes_roundtrip() {
    let cfg = CircuitBreakerConfig {
        enabled: true,
        failure_threshold: 1,
        success_threshold: 2,
        timeout_secs: 3,
        half_open_max_requests: 4,
    };
    assert_serde_json_roundtrip(&cfg);
}

#[test]
fn squirrel_unified_config_serializes_roundtrip() {
    let cfg = sample_squirrel_unified_config();
    assert_serde_json_roundtrip(&cfg);
}

#[test]
fn squirrel_unified_config_clone_matches_serde_value() {
    let cfg = sample_squirrel_unified_config();
    let expected = serde_json::to_value(&cfg).expect("should succeed");
    let cfg2 = sample_squirrel_unified_config();
    assert_eq!(
        expected,
        serde_json::to_value(&cfg2).expect("should succeed")
    );
}

#[test]
fn squirrel_unified_config_debug_is_non_empty() {
    let cfg = sample_squirrel_unified_config();
    let debug = format!("{cfg:?}");
    assert!(debug.len() > 50);
    assert!(debug.contains("SquirrelUnifiedConfig"));
}

#[test]
fn timeout_config_serializes_roundtrip() {
    let t = sample_timeouts();
    assert_serde_json_roundtrip(&t);
}
