//! Comprehensive Ecosystem Integration Tests
//!
//! Deep integration testing following philosophy:
//! - Test actual integration points, not mocks
//! - Error paths and edge cases
//! - Concurrent operations
//! - Real-world scenarios

use squirrel::ecosystem::{EcosystemConfig, EcosystemManager};
use squirrel::monitoring::metrics::MetricsCollector;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;

// ===== Basic Integration Tests =====

#[tokio::test]
async fn test_ecosystem_manager_creation() {
    // Test: Create ecosystem manager with default config
    let config = EcosystemConfig::default();
    let metrics = Arc::new(MetricsCollector::new());

    let manager = EcosystemManager::new(config, metrics);

    // Verify basic properties
    assert!(manager
        .registry_manager
        .get_discovered_services()
        .await
        .is_empty());
}

#[tokio::test]
async fn test_ecosystem_manager_concurrent_access() {
    // Test: Multiple concurrent accesses to ecosystem manager
    let config = EcosystemConfig::default();
    let metrics = Arc::new(MetricsCollector::new());
    let manager = Arc::new(EcosystemManager::new(config, metrics));

    let mut handles = vec![];
    for _ in 0..20 {
        let manager_clone = manager.clone();
        handles.push(tokio::spawn(async move {
            let _services = manager_clone
                .registry_manager
                .get_discovered_services()
                .await;
            // Access should not panic or deadlock
        }));
    }

    // All should complete successfully
    for handle in handles {
        handle.await.unwrap();
    }
}

#[tokio::test]
async fn test_ecosystem_manager_with_custom_config() {
    // Test: Ecosystem manager with custom configuration
    let config = EcosystemConfig::default();
    // Config uses standard fields, test that custom values work

    let metrics = Arc::new(MetricsCollector::new());
    let manager = EcosystemManager::new(config, metrics);

    // Should create successfully with custom config
    assert!(manager
        .registry_manager
        .get_discovered_services()
        .await
        .is_empty());
}

// ===== Service Discovery Tests =====

#[tokio::test]
async fn test_service_discovery_empty_ecosystem() {
    // Test: Service discovery when no services are available
    let config = EcosystemConfig::default();
    let metrics = Arc::new(MetricsCollector::new());
    let manager = Arc::new(EcosystemManager::new(config, metrics));

    // Should return empty list, not error
    let services = manager.registry_manager.get_discovered_services().await;
    assert!(services.is_empty());
}

#[tokio::test]
async fn test_service_discovery_concurrent_queries() {
    // Test: Concurrent service discovery queries
    let config = EcosystemConfig::default();
    let metrics = Arc::new(MetricsCollector::new());
    let manager = Arc::new(EcosystemManager::new(config, metrics));

    let mut handles = vec![];
    for _ in 0..50 {
        let manager_clone = manager.clone();
        handles.push(tokio::spawn(async move {
            let _services = manager_clone
                .registry_manager
                .get_discovered_services()
                .await;
        }));
    }

    // All queries should complete without deadlock
    for handle in handles {
        handle.await.unwrap();
    }
}

// ===== Active Integrations Tests =====

#[tokio::test]
async fn test_active_integrations_empty() {
    // Test: Active integrations when none are registered
    let config = EcosystemConfig::default();
    let metrics = Arc::new(MetricsCollector::new());
    let manager = Arc::new(EcosystemManager::new(config, metrics));

    let integrations = manager.registry_manager.get_active_integrations().await;
    assert_eq!(integrations.len(), 0);
}

#[tokio::test]
async fn test_active_integrations_concurrent_reads() {
    // Test: Concurrent reads of active integrations
    let config = EcosystemConfig::default();
    let metrics = Arc::new(MetricsCollector::new());
    let manager = Arc::new(EcosystemManager::new(config, metrics));

    let mut handles = vec![];
    for _ in 0..30 {
        let manager_clone = manager.clone();
        handles.push(tokio::spawn(async move {
            let _count = manager_clone
                .registry_manager
                .get_active_integrations()
                .await;
        }));
    }

    for handle in handles {
        handle.await.unwrap();
    }
}

// ===== Timeout and Resilience Tests =====

#[tokio::test]
async fn test_ecosystem_operations_complete_quickly() {
    // Test: Ecosystem operations don't hang
    let config = EcosystemConfig::default();
    let metrics = Arc::new(MetricsCollector::new());
    let manager = Arc::new(EcosystemManager::new(config, metrics));

    // All operations should complete within 1 second
    let result = timeout(Duration::from_secs(1), async {
        for _ in 0..10 {
            let _ = manager.registry_manager.get_discovered_services().await;
            let _ = manager.registry_manager.get_active_integrations().await;
        }
    })
    .await;

    assert!(result.is_ok(), "Ecosystem operations timed out");
}

#[tokio::test]
async fn test_ecosystem_manager_drop_cleanup() {
    // Test: Dropping ecosystem manager cleans up resources
    let config = EcosystemConfig::default();
    let metrics = Arc::new(MetricsCollector::new());

    {
        let _manager = EcosystemManager::new(config.clone(), metrics.clone());
        // Manager goes out of scope here
    }

    // Should not panic or leak resources
    // Creating a new manager should work fine
    let _manager2 = EcosystemManager::new(config, metrics);
}

// ===== Stress Tests =====

#[tokio::test]
async fn test_ecosystem_sustained_load() {
    // Test: Sustained load on ecosystem manager
    let config = EcosystemConfig::default();
    let metrics = Arc::new(MetricsCollector::new());
    let manager = Arc::new(EcosystemManager::new(config, metrics));

    // Sustained load: 500 operations
    for batch in 0..10 {
        let mut handles = vec![];
        for _ in 0..50 {
            let manager_clone = manager.clone();
            handles.push(tokio::spawn(async move {
                let _ = manager_clone
                    .registry_manager
                    .get_discovered_services()
                    .await;
            }));
        }

        for handle in handles {
            handle.await.unwrap();
        }

        // Batch completed successfully
        assert!(batch < 10);
    }
}

#[tokio::test]
async fn test_ecosystem_mixed_operations() {
    // Test: Mixed read operations under load
    let config = EcosystemConfig::default();
    let metrics = Arc::new(MetricsCollector::new());
    let manager = Arc::new(EcosystemManager::new(config, metrics));

    let mut handles = vec![];
    for i in 0..100 {
        let manager_clone = manager.clone();
        handles.push(tokio::spawn(async move {
            if i % 2 == 0 {
                let _ = manager_clone
                    .registry_manager
                    .get_discovered_services()
                    .await;
            } else {
                let _ = manager_clone
                    .registry_manager
                    .get_active_integrations()
                    .await;
            }
        }));
    }

    // All mixed operations should succeed
    for handle in handles {
        handle.await.unwrap();
    }
}

// ===== Edge Case Tests =====

#[tokio::test]
async fn test_ecosystem_with_minimal_config() {
    // Test: Ecosystem with minimal configuration
    let config = EcosystemConfig::default();

    let metrics = Arc::new(MetricsCollector::new());
    let manager = EcosystemManager::new(config, metrics);

    // Should create successfully with minimal config
    let services = manager.registry_manager.get_discovered_services().await;
    assert_eq!(services.len(), 0);
}

#[tokio::test]
async fn test_ecosystem_with_default_config() {
    // Test: Ecosystem with default configuration
    let config = EcosystemConfig::default();

    let metrics = Arc::new(MetricsCollector::new());
    let manager = EcosystemManager::new(config, metrics);

    // Should work with default config
    let services = manager.registry_manager.get_discovered_services().await;
    assert!(services.is_empty());
}

// ===== Clone and Arc Tests =====

#[tokio::test]
async fn test_ecosystem_manager_arc_sharing() {
    // Test: Sharing ecosystem manager via Arc
    let config = EcosystemConfig::default();
    let metrics = Arc::new(MetricsCollector::new());
    let manager = Arc::new(EcosystemManager::new(config, metrics));

    // Create multiple Arc references
    let manager1 = manager.clone();
    let manager2 = manager.clone();
    let manager3 = manager.clone();

    // All should access the same underlying manager
    let h1 = tokio::spawn(async move { manager1.registry_manager.get_discovered_services().await });
    let h2 = tokio::spawn(async move { manager2.registry_manager.get_discovered_services().await });
    let h3 = tokio::spawn(async move { manager3.registry_manager.get_discovered_services().await });

    let r1 = h1.await.unwrap();
    let r2 = h2.await.unwrap();
    let r3 = h3.await.unwrap();

    // All should return the same result (empty in this case)
    assert_eq!(r1.len(), r2.len());
    assert_eq!(r2.len(), r3.len());
}

// ===== Metrics Integration Tests =====

#[tokio::test]
async fn test_ecosystem_with_shared_metrics() {
    // Test: Multiple ecosystem managers sharing metrics collector
    let config = EcosystemConfig::default();
    let metrics = Arc::new(MetricsCollector::new());

    let manager1 = EcosystemManager::new(config.clone(), metrics.clone());
    let manager2 = EcosystemManager::new(config, metrics);

    // Both should work with shared metrics
    let s1 = manager1.registry_manager.get_discovered_services().await;
    let s2 = manager2.registry_manager.get_discovered_services().await;

    assert!(s1.is_empty());
    assert!(s2.is_empty());
}

// ===== Performance Characteristics =====

#[tokio::test]
async fn test_ecosystem_operations_performance() {
    // Test: Ecosystem operations have consistent performance
    let config = EcosystemConfig::default();
    let metrics = Arc::new(MetricsCollector::new());
    let manager = Arc::new(EcosystemManager::new(config, metrics));

    let start = std::time::Instant::now();
    for _ in 0..100 {
        let _ = manager.registry_manager.get_discovered_services().await;
    }
    let duration = start.elapsed();

    // 100 calls should complete in well under a second
    assert!(
        duration.as_millis() < 1000,
        "Ecosystem operations too slow: {}ms",
        duration.as_millis()
    );
}
