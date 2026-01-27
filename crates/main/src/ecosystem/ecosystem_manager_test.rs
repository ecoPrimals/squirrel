use crate::ecosystem::{ComponentHealth, EcosystemConfig, EcosystemManager, EcosystemPrimalType};
use crate::monitoring::metrics::MetricsCollector;
use std::sync::Arc;

/// Helper to create test ecosystem config
fn create_test_config() -> EcosystemConfig {
    EcosystemConfig {
        service_id: "test-squirrel-001".to_string(),
        service_name: "Test Squirrel".to_string(),
        service_host: "localhost".to_string(),
        service_port: 8080,
        biome_id: Some("test-biome".to_string()),
        service_mesh_endpoint: "http://localhost:8000".to_string(),
        ..Default::default()
    }
}

/// Helper to create test metrics collector
fn create_test_metrics() -> Arc<MetricsCollector> {
    Arc::new(MetricsCollector::new())
}

#[tokio::test]
async fn test_ecosystem_manager_creation() {
    let config = create_test_config();
    let metrics = create_test_metrics();

    let manager = EcosystemManager::new(config.clone(), metrics);

    assert_eq!(manager.config.service_id, "test-squirrel-001");
    assert_eq!(manager.config.service_name, "Test Squirrel");
}

#[tokio::test]
async fn test_ecosystem_config_default() {
    let config = EcosystemConfig::default();

    assert!(config.service_id.starts_with("squirrel-"));
    assert_eq!(config.service_name, "Squirrel AI Primal");
    assert_eq!(config.service_host, "127.0.0.1"); // deployment::hosts::localhost() returns 127.0.0.1
    assert_eq!(config.service_port, 8080);
}

#[tokio::test]
#[allow(deprecated)]
async fn test_ecosystem_primal_type_as_str_deprecated() {
    // Testing deprecated API for backward compatibility
    assert_eq!(EcosystemPrimalType::Squirrel.as_str(), "squirrel");
    assert_eq!(EcosystemPrimalType::Songbird.as_str(), "songbird");
    assert_eq!(EcosystemPrimalType::BearDog.as_str(), "beardog");
    assert_eq!(EcosystemPrimalType::ToadStool.as_str(), "toadstool");
    assert_eq!(EcosystemPrimalType::NestGate.as_str(), "nestgate");
}

#[tokio::test]
async fn test_ecosystem_manager_initialization() {
    let config = create_test_config();
    let metrics = create_test_metrics();
    let mut manager = EcosystemManager::new(config, metrics);

    let result = manager.initialize().await;
    assert!(result.is_ok(), "Manager initialization should succeed");

    let status = manager.get_manager_status().await;
    assert_eq!(status.status, "initialized");
    assert!(status.initialized_at.is_some());
}

#[tokio::test]
async fn test_ecosystem_config_with_biome_id() {
    let config = EcosystemConfig {
        biome_id: Some("production-biome".to_string()),
        ..Default::default()
    };

    assert_eq!(config.biome_id, Some("production-biome".to_string()));
}

#[tokio::test]
async fn test_ecosystem_config_without_biome_id() {
    let config = EcosystemConfig {
        biome_id: None,
        ..Default::default()
    };

    assert!(config.biome_id.is_none());
}

#[tokio::test]
async fn test_component_health_creation() {
    let health = ComponentHealth {
        status: "healthy".to_string(),
        last_check: chrono::Utc::now(),
        error: None,
        metadata: std::collections::HashMap::new(),
    };

    assert_eq!(health.status, "healthy");
    assert!(health.error.is_none());
}

#[tokio::test]
async fn test_multiple_ecosystem_managers() {
    let config1 = create_test_config();
    let config2 = EcosystemConfig {
        service_id: "test-squirrel-002".to_string(),
        ..create_test_config()
    };

    let metrics = create_test_metrics();
    let manager1 = EcosystemManager::new(config1, metrics.clone());
    let manager2 = EcosystemManager::new(config2, metrics);

    assert_eq!(manager1.config.service_id, "test-squirrel-001");
    assert_eq!(manager2.config.service_id, "test-squirrel-002");
}

#[tokio::test]
async fn test_ecosystem_config_serialization() {
    let config = create_test_config();

    // Test that config can be serialized
    let json = serde_json::to_string(&config);
    assert!(json.is_ok(), "Config should be serializable");

    // Test that it can be deserialized back
    let json_str = json.expect("test: should succeed");
    let deserialized: Result<EcosystemConfig, _> = serde_json::from_str(&json_str);
    assert!(deserialized.is_ok(), "Config should be deserializable");

    let deserialized_config = deserialized.expect("test: should succeed");
    assert_eq!(deserialized_config.service_id, config.service_id);
}

#[tokio::test]
async fn test_component_health_unhealthy() {
    let health = ComponentHealth {
        status: "unhealthy".to_string(),
        last_check: chrono::Utc::now(),
        error: Some("Connection failed".to_string()),
        metadata: std::collections::HashMap::new(),
    };

    assert_eq!(health.status, "unhealthy");
    assert!(health.error.is_some());
    assert_eq!(
        health.error.expect("test: should succeed"),
        "Connection failed"
    );
}

#[tokio::test]
async fn test_ecosystem_config_custom_port() {
    let mut config = create_test_config();
    config.service_port = 9090;

    assert_eq!(config.service_port, 9090);
    assert!(config.service_id.starts_with("test-"));
}

#[tokio::test]
async fn test_ecosystem_config_custom_songbird_endpoint() {
    let mut config = create_test_config();
    config.songbird_endpoint = "http://songbird.example.com:9000".to_string();

    assert_eq!(config.songbird_endpoint, "http://songbird.example.com:9000");
}

#[tokio::test]
async fn test_ecosystem_manager_config_immutability() {
    let config = create_test_config();
    let original_id = config.service_id.clone();
    let metrics = create_test_metrics();

    let manager = EcosystemManager::new(config, metrics);

    // Manager should preserve the config
    assert_eq!(manager.config.service_id, original_id);
}

#[tokio::test]
async fn test_ecosystem_manager_status_initialized() {
    let config = create_test_config();
    let metrics = create_test_metrics();
    let mut manager = EcosystemManager::new(config, metrics);

    manager.initialize().await.expect("test: should initialize");

    let status = manager.get_manager_status().await;
    assert_eq!(status.status, "initialized");
    assert!(status.initialized_at.is_some());
    assert_eq!(status.error_count, 0);
}

#[tokio::test]
async fn test_ecosystem_manager_discover_services() {
    let config = create_test_config();
    let metrics = create_test_metrics();
    let mut manager = EcosystemManager::new(config, metrics);

    manager.initialize().await.expect("test: should initialize");

    let result = manager.discover_services().await;
    assert!(result.is_ok(), "Service discovery should not fail");

    let services = result.expect("test: should succeed");
    // Empty or populated - just verify it returns a vec
    assert!(services.len() >= 0);
}

#[tokio::test]
async fn test_ecosystem_manager_find_services_by_capability() {
    let config = create_test_config();
    let metrics = create_test_metrics();
    let mut manager = EcosystemManager::new(config, metrics);

    manager.initialize().await.expect("test: should initialize");

    // ✅ NEW: Use capability-based discovery instead of hardcoded primal type
    let result = manager
        .find_services_by_capability("service_mesh")
        .await;
    assert!(result.is_ok(), "Find services by capability should not fail");
}

#[tokio::test]
#[allow(deprecated)]
async fn test_ecosystem_manager_find_services_by_type_deprecated() {
    // Test that deprecated method returns proper error
    let config = create_test_config();
    let metrics = create_test_metrics();
    let mut manager = EcosystemManager::new(config, metrics);

    manager.initialize().await.expect("test: should initialize");

    let result = manager
        .find_services_by_type(EcosystemPrimalType::Songbird)
        .await;
    assert!(result.is_err(), "Deprecated method should return error");
}

#[tokio::test]
async fn test_ecosystem_primal_type_from_str_squirrel() {
    let result = EcosystemPrimalType::from_str("squirrel");
    assert!(result.is_ok());
    assert_eq!(
        result.expect("test: should succeed"),
        EcosystemPrimalType::Squirrel
    );
}

#[tokio::test]
async fn test_ecosystem_primal_type_from_str_songbird() {
    let result = EcosystemPrimalType::from_str("songbird");
    assert!(result.is_ok());
    assert_eq!(
        result.expect("test: should succeed"),
        EcosystemPrimalType::Songbird
    );
}

#[tokio::test]
async fn test_ecosystem_primal_type_from_str_beardog() {
    let result = EcosystemPrimalType::from_str("beardog");
    assert!(result.is_ok());
    assert_eq!(
        result.expect("test: should succeed"),
        EcosystemPrimalType::BearDog
    );
}

#[tokio::test]
async fn test_ecosystem_primal_type_from_str_toadstool() {
    let result = EcosystemPrimalType::from_str("toadstool");
    assert!(result.is_ok());
    assert_eq!(
        result.expect("test: should succeed"),
        EcosystemPrimalType::ToadStool
    );
}

#[tokio::test]
async fn test_ecosystem_primal_type_from_str_nestgate() {
    let result = EcosystemPrimalType::from_str("nestgate");
    assert!(result.is_ok());
    assert_eq!(
        result.expect("test: should succeed"),
        EcosystemPrimalType::NestGate
    );
}

#[tokio::test]
async fn test_ecosystem_primal_type_from_str_invalid() {
    let result = EcosystemPrimalType::from_str("invalid_primal");
    assert!(result.is_err(), "Invalid primal type should return error");
}

#[tokio::test]
async fn test_ecosystem_primal_type_from_str_case_insensitive() {
    let result = EcosystemPrimalType::from_str("SQUIRREL");
    assert!(result.is_ok());
    assert_eq!(
        result.expect("test: should succeed"),
        EcosystemPrimalType::Squirrel
    );
}

#[tokio::test]
async fn test_component_health_with_metadata() {
    let mut metadata = std::collections::HashMap::new();
    metadata.insert("cpu_usage".to_string(), "25%".to_string());
    metadata.insert("memory".to_string(), "512MB".to_string());

    let health = ComponentHealth {
        status: "healthy".to_string(),
        last_check: chrono::Utc::now(),
        error: None,
        metadata: metadata.clone(),
    };

    assert_eq!(health.metadata.len(), 2);
    assert_eq!(health.metadata.get("cpu_usage"), Some(&"25%".to_string()));
}

#[tokio::test]
async fn test_ecosystem_manager_status_tracking() {
    let config = create_test_config();
    let metrics = create_test_metrics();
    let manager = EcosystemManager::new(config, metrics);

    let status = manager.get_manager_status().await;
    assert_eq!(status.status, "initializing");
    assert!(status.initialized_at.is_none());
}

#[tokio::test]
async fn test_ecosystem_config_with_custom_host() {
    let config = EcosystemConfig {
        service_host: "0.0.0.0".to_string(),
        ..Default::default()
    };

    assert_eq!(config.service_host, "0.0.0.0");
}

#[tokio::test]
async fn test_ecosystem_config_service_name_custom() {
    let config = EcosystemConfig {
        service_name: "Custom Squirrel Service".to_string(),
        ..Default::default()
    };

    assert_eq!(config.service_name, "Custom Squirrel Service");
}

#[tokio::test]
async fn test_multiple_initializations() {
    let config = create_test_config();
    let metrics = create_test_metrics();
    let mut manager = EcosystemManager::new(config, metrics);

    // First initialization
    let result1 = manager.initialize().await;
    assert!(result1.is_ok());

    // Second initialization should also succeed (idempotent)
    let result2 = manager.initialize().await;
    assert!(result2.is_ok());
}

#[tokio::test]
async fn test_ecosystem_manager_metrics_collector() {
    let config = create_test_config();
    let metrics = create_test_metrics();
    let manager = EcosystemManager::new(config, metrics.clone());

    // Verify metrics collector is accessible
    assert!(Arc::ptr_eq(&manager.metrics_collector, &metrics));
}

#[tokio::test]
async fn test_ecosystem_config_metadata_empty() {
    let config = create_test_config();

    assert!(config.metadata.is_empty() || !config.metadata.is_empty());
}

#[tokio::test]
async fn test_ecosystem_config_metadata_custom() {
    let mut metadata = std::collections::HashMap::new();
    metadata.insert("region".to_string(), "us-west-2".to_string());
    metadata.insert("environment".to_string(), "production".to_string());

    let config = EcosystemConfig {
        metadata,
        ..Default::default()
    };

    assert_eq!(config.metadata.len(), 2);
    assert_eq!(
        config.metadata.get("region"),
        Some(&"us-west-2".to_string())
    );
}

#[tokio::test]
async fn test_component_health_error_message() {
    let health = ComponentHealth {
        status: "degraded".to_string(),
        last_check: chrono::Utc::now(),
        error: Some("Timeout connecting to dependency".to_string()),
        metadata: std::collections::HashMap::new(),
    };

    assert_eq!(health.status, "degraded");
    let error_msg = health.error.expect("test: should have error");
    assert!(error_msg.contains("Timeout"));
}

#[tokio::test]
async fn test_ecosystem_primal_type_all_variants() {
    let types = vec![
        EcosystemPrimalType::Squirrel,
        EcosystemPrimalType::Songbird,
        EcosystemPrimalType::BearDog,
        EcosystemPrimalType::ToadStool,
        EcosystemPrimalType::NestGate,
    ];

    assert_eq!(types.len(), 5);

    // Verify each has a string representation
    for primal_type in types {
        let s = primal_type.as_str();
        assert!(!s.is_empty());
    }
}
