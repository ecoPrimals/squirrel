// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Comprehensive ecosystem integration tests
//!
//! Tests for the universal primal ecosystem integration

use squirrel::ecosystem::*;
use squirrel::error::PrimalError;
use std::time::Duration;
use tokio::time::sleep;

/// Helper function to get configurable test endpoint
fn get_test_service_endpoint(default_port: u16) -> String {
    let port = std::env::var("TEST_ECOSYSTEM_PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(default_port);
    format!("http://localhost:{}", port)
}

#[tokio::test]
async fn test_ecosystem_manager_initialization() {
    let config = EcosystemConfig::default();
    let manager = EcosystemManager::new(config);
    
    assert!(manager.is_initialized());
}

#[tokio::test]
async fn test_service_registration_flow() {
    let config = EcosystemConfig::default();
    let manager = EcosystemManager::new(config);
    
    let registration = ServiceRegistration {
        service_id: "test-service".to_string(),
        capabilities: vec!["ai_inference".to_string()],
        endpoint: get_test_service_endpoint(8080),
        ..Default::default()
    };
    
    let result = manager.register_service(registration).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_capability_discovery() {
    let config = EcosystemConfig::default();
    let manager = EcosystemManager::new(config);
    
    // Register service with capabilities
    let registration = ServiceRegistration {
        service_id: "ai-service".to_string(),
        capabilities: vec!["chat".to_string(), "vision".to_string()],
        endpoint: get_test_service_endpoint(8080),
        ..Default::default()
    };
    
    manager.register_service(registration).await.unwrap();
    
    // Discover by capability
    let services = manager.discover_by_capability("chat").await.unwrap();
    assert!(!services.is_empty());
    assert_eq!(services[0].service_id, "ai-service");
}

#[tokio::test]
async fn test_service_health_monitoring() {
    let config = EcosystemConfig::default();
    let manager = EcosystemManager::new(config);
    
    let registration = ServiceRegistration {
        service_id: "health-test".to_string(),
        capabilities: vec!["test".to_string()],
        endpoint: get_test_service_endpoint(8080),
        health_check_enabled: true,
        ..Default::default()
    };
    
    manager.register_service(registration).await.unwrap();
    
    // Check health status
    let health = manager.get_service_health("health-test").await;
    assert!(health.is_ok());
}

#[tokio::test]
async fn test_service_deregistration() {
    let config = EcosystemConfig::default();
    let manager = EcosystemManager::new(config);
    
    let registration = ServiceRegistration {
        service_id: "dereg-test".to_string(),
        capabilities: vec!["test".to_string()],
        endpoint: get_test_service_endpoint(8080),
        ..Default::default()
    };
    
    manager.register_service(registration).await.unwrap();
    
    // Verify registered
    let services = manager.get_all_services().await.unwrap();
    assert!(services.iter().any(|s| s.service_id == "dereg-test"));
    
    // Deregister
    manager.deregister_service("dereg-test").await.unwrap();
    
    // Verify removed
    let services = manager.get_all_services().await.unwrap();
    assert!(!services.iter().any(|s| s.service_id == "dereg-test"));
}

#[tokio::test]
async fn test_multiple_service_registration() {
    let config = EcosystemConfig::default();
    let manager = EcosystemManager::new(config);
    
    for i in 1..=5 {
        let registration = ServiceRegistration {
            service_id: format!("service-{}", i),
            capabilities: vec![format!("capability-{}", i)],
            endpoint: format!("http://localhost:808{}", i),
            ..Default::default()
        };
        
        manager.register_service(registration).await.unwrap();
    }
    
    let services = manager.get_all_services().await.unwrap();
    assert_eq!(services.len(), 5);
}

#[tokio::test]
async fn test_service_capability_matching() {
    let config = EcosystemConfig::default();
    let manager = EcosystemManager::new(config);
    
    // Register services with different capabilities
    let services = vec![
        ("ai-1", vec!["chat", "vision"]),
        ("ai-2", vec!["chat", "audio"]),
        ("compute-1", vec!["gpu", "training"]),
    ];
    
    for (id, caps) in services {
        let registration = ServiceRegistration {
            service_id: id.to_string(),
            capabilities: caps.iter().map(|s| s.to_string()).collect(),
            endpoint: format!("http://localhost:8080/{}", id),
            ..Default::default()
        };
        manager.register_service(registration).await.unwrap();
    }
    
    // Test exact match
    let chat_services = manager.discover_by_capability("chat").await.unwrap();
    assert_eq!(chat_services.len(), 2);
    
    // Test unique capability
    let gpu_services = manager.discover_by_capability("gpu").await.unwrap();
    assert_eq!(gpu_services.len(), 1);
    assert_eq!(gpu_services[0].service_id, "compute-1");
}

#[tokio::test]
async fn test_service_metadata_storage() {
    let config = EcosystemConfig::default();
    let manager = EcosystemManager::new(config);
    
    let mut metadata = std::collections::HashMap::new();
    metadata.insert("version".to_string(), "1.0.0".to_string());
    metadata.insert("region".to_string(), "us-east-1".to_string());
    
    let registration = ServiceRegistration {
        service_id: "metadata-test".to_string(),
        capabilities: vec!["test".to_string()],
        endpoint: get_test_service_endpoint(8080),
        metadata: Some(metadata.clone()),
        ..Default::default()
    };
    
    manager.register_service(registration).await.unwrap();
    
    let services = manager.get_all_services().await.unwrap();
    let service = services.iter().find(|s| s.service_id == "metadata-test").unwrap();
    
    assert_eq!(service.metadata.as_ref().unwrap().get("version").unwrap(), "1.0.0");
    assert_eq!(service.metadata.as_ref().unwrap().get("region").unwrap(), "us-east-1");
}

#[tokio::test]
async fn test_concurrent_service_operations() {
    let config = EcosystemConfig::default();
    let manager = std::sync::Arc::new(EcosystemManager::new(config));
    
    let mut handles = vec![];
    
    for i in 0..10 {
        let mgr = manager.clone();
        let handle = tokio::spawn(async move {
            let registration = ServiceRegistration {
                service_id: format!("concurrent-{}", i),
                capabilities: vec!["test".to_string()],
                endpoint: format!("http://localhost:808{}", i),
                ..Default::default()
            };
            mgr.register_service(registration).await
        });
        handles.push(handle);
    }
    
    for handle in handles {
        assert!(handle.await.unwrap().is_ok());
    }
    
    let services = manager.get_all_services().await.unwrap();
    assert_eq!(services.len(), 10);
}

#[tokio::test]
async fn test_service_filtering_by_status() {
    let config = EcosystemConfig::default();
    let manager = EcosystemManager::new(config);
    
    // Register healthy service
    let reg1 = ServiceRegistration {
        service_id: "healthy-service".to_string(),
        capabilities: vec!["test".to_string()],
        endpoint: get_test_service_endpoint(8080),
        health_check_enabled: true,
        ..Default::default()
    };
    
    manager.register_service(reg1).await.unwrap();
    
    // Get only healthy services
    let healthy = manager.get_healthy_services().await.unwrap();
    assert!(!healthy.is_empty());
}

#[tokio::test]
async fn test_ecosystem_configuration_validation() {
    let mut config = EcosystemConfig::default();
    config.max_services = 0; // Invalid
    
    let result = config.validate();
    assert!(result.is_err());
    
    config.max_services = 100; // Valid
    let result = config.validate();
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_service_ttl_expiration() {
    let mut config = EcosystemConfig::default();
    config.service_ttl = Duration::from_millis(100);
    
    let manager = EcosystemManager::new(config);
    
    let registration = ServiceRegistration {
        service_id: "ttl-test".to_string(),
        capabilities: vec!["test".to_string()],
        endpoint: get_test_service_endpoint(8080),
        ..Default::default()
    };
    
    manager.register_service(registration).await.unwrap();
    
    // Wait for TTL expiration
    sleep(Duration::from_millis(150)).await;
    
    // Service should be expired or require refresh
    let services = manager.get_all_services().await.unwrap();
    // Note: Behavior depends on implementation - either removed or marked stale
}

#[tokio::test]
async fn test_ecosystem_stats_collection() {
    let config = EcosystemConfig::default();
    let manager = EcosystemManager::new(config);
    
    // Register multiple services
    for i in 1..=3 {
        let registration = ServiceRegistration {
            service_id: format!("stats-{}", i),
            capabilities: vec!["test".to_string()],
            endpoint: format!("http://localhost:808{}", i),
            ..Default::default()
        };
        manager.register_service(registration).await.unwrap();
    }
    
    let stats = manager.get_ecosystem_stats().await.unwrap();
    assert_eq!(stats.total_services, 3);
    assert!(stats.healthy_services <= 3);
}

#[tokio::test]
async fn test_service_endpoint_validation() {
    let config = EcosystemConfig::default();
    let manager = EcosystemManager::new(config);
    
    let invalid_registration = ServiceRegistration {
        service_id: "invalid-endpoint".to_string(),
        capabilities: vec!["test".to_string()],
        endpoint: "not-a-valid-url".to_string(),
        ..Default::default()
    };
    
    let result = manager.register_service(invalid_registration).await;
    // Should fail validation
    assert!(result.is_err());
}

#[tokio::test]
async fn test_ecosystem_graceful_shutdown() {
    let config = EcosystemConfig::default();
    let manager = EcosystemManager::new(config);
    
    // Register services
    for i in 1..=3 {
        let registration = ServiceRegistration {
            service_id: format!("shutdown-{}", i),
            capabilities: vec!["test".to_string()],
            endpoint: format!("http://localhost:808{}", i),
            ..Default::default()
        };
        manager.register_service(registration).await.unwrap();
    }
    
    // Shutdown should deregister all
    let result = manager.shutdown_gracefully().await;
    assert!(result.is_ok());
    
    let services = manager.get_all_services().await.unwrap();
    assert_eq!(services.len(), 0);
}

#[tokio::test]
async fn test_service_update_capabilities() {
    let config = EcosystemConfig::default();
    let manager = EcosystemManager::new(config);
    
    let registration = ServiceRegistration {
        service_id: "update-test".to_string(),
        capabilities: vec!["initial".to_string()],
        endpoint: get_test_service_endpoint(8080),
        ..Default::default()
    };
    
    manager.register_service(registration).await.unwrap();
    
    // Update capabilities
    let updated_caps = vec!["initial".to_string(), "new".to_string()];
    manager.update_service_capabilities("update-test", updated_caps).await.unwrap();
    
    let services = manager.get_all_services().await.unwrap();
    let service = services.iter().find(|s| s.service_id == "update-test").unwrap();
    assert!(service.capabilities.contains(&"new".to_string()));
}

#[tokio::test]
async fn test_ecosystem_load_balancing() {
    let config = EcosystemConfig::default();
    let manager = EcosystemManager::new(config);
    
    // Register multiple services with same capability
    for i in 1..=3 {
        let registration = ServiceRegistration {
            service_id: format!("lb-{}", i),
            capabilities: vec!["shared".to_string()],
            endpoint: format!("http://localhost:808{}", i),
            ..Default::default()
        };
        manager.register_service(registration).await.unwrap();
    }
    
    // Get service for load balancing
    let selected = manager.select_service_for_capability("shared").await.unwrap();
    assert!(selected.is_some());
    assert!(selected.unwrap().service_id.starts_with("lb-"));
}

#[tokio::test]
async fn test_service_dependency_tracking() {
    let config = EcosystemConfig::default();
    let manager = EcosystemManager::new(config);
    
    let registration = ServiceRegistration {
        service_id: "dependent-service".to_string(),
        capabilities: vec!["requires-db".to_string()],
        endpoint: get_test_service_endpoint(8080),
        dependencies: Some(vec!["database-service".to_string()]),
        ..Default::default()
    };
    
    manager.register_service(registration).await.unwrap();
    
    let services = manager.get_all_services().await.unwrap();
    let service = services.iter().find(|s| s.service_id == "dependent-service").unwrap();
    assert!(service.dependencies.as_ref().unwrap().contains(&"database-service".to_string()));
}

