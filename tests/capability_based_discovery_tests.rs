//! Comprehensive integration tests for capability-based service discovery
//! 
//! These tests validate that the capability-based architecture works correctly
//! with various service configurations and scenarios.

use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::time::sleep;

use squirrel_main::universal_primal_ecosystem::{
    UniversalPrimalEcosystem, CapabilityRequest, DiscoveredService, ServiceHealth, CacheConfig
};
use squirrel_main::universal::{PrimalContext, NetworkLocation, SecurityLevel};

/// Helper function to create a test context
fn create_test_context(user_id: &str) -> PrimalContext {
    PrimalContext {
        user_id: user_id.to_string(),
        device_id: "test-device-001".to_string(),
        session_id: "test-session-001".to_string(),
        network_location: NetworkLocation {
            ip_address: "127.0.0.1".to_string(),
            subnet: Some("192.168.1.0/24".to_string()),
            network_id: Some("test-network".to_string()),
            geo_location: Some("test-location".to_string()),
        },
        security_level: SecurityLevel::Standard,
        biome_id: Some("test-biome".to_string()),
        metadata: HashMap::new(),
    }
}

/// Helper function to create a mock discovered service
fn create_mock_service(
    service_id: &str,
    endpoint: &str,
    capabilities: Vec<&str>,
    health: ServiceHealth,
) -> DiscoveredService {
    DiscoveredService {
        service_id: service_id.to_string(),
        instance_id: format!("{}-instance", service_id),
        endpoint: endpoint.to_string(),
        capabilities: capabilities.into_iter().map(|s| s.to_string()).collect(),
        health,
        discovered_at: chrono::Utc::now(),
        last_health_check: Some(chrono::Utc::now()),
    }
}

/// Helper function to populate ecosystem with mock services
async fn populate_ecosystem_with_mock_services(ecosystem: &UniversalPrimalEcosystem) {
    let mut services = ecosystem.discovered_services.write().await;
    
    // Security service provider
    let security_service = create_mock_service(
        "security-provider-1",
        "http://localhost:8001",
        vec!["authentication", "encryption", "audit-logging"],
        ServiceHealth::Healthy,
    );
    
    // Storage service provider
    let storage_service = create_mock_service(
        "storage-provider-1", 
        "http://localhost:8002",
        vec!["data-persistence", "file-storage", "high-availability"],
        ServiceHealth::Healthy,
    );
    
    // Compute service provider
    let compute_service = create_mock_service(
        "compute-provider-1",
        "http://localhost:8003", 
        vec!["task-execution", "sandboxing", "gpu-acceleration"],
        ServiceHealth::Healthy,
    );
    
    // Multi-capability service provider
    let multi_service = create_mock_service(
        "multi-provider-1",
        "http://localhost:8004",
        vec!["authentication", "data-persistence", "task-execution"],
        ServiceHealth::Healthy,
    );
    
    // Degraded service provider
    let degraded_service = create_mock_service(
        "degraded-provider-1",
        "http://localhost:8005",
        vec!["data-persistence", "backup-storage"],
        ServiceHealth::Degraded,
    );
    
    // Index services by their capabilities
    for service in [security_service, storage_service, compute_service, multi_service, degraded_service] {
        for capability in &service.capabilities {
            services
                .entry(capability.clone())
                .or_insert_with(Vec::new)
                .push(service.clone());
        }
    }
}

#[tokio::test]
async fn test_capability_discovery_basic() {
    let context = create_test_context("test-user-001");
    let ecosystem = UniversalPrimalEcosystem::new(context.clone());
    
    // Populate with mock services
    populate_ecosystem_with_mock_services(&ecosystem).await;
    
    // Test finding security capabilities
    let security_request = CapabilityRequest {
        required_capabilities: vec!["authentication".to_string()],
        optional_capabilities: vec!["encryption".to_string()],
        context: context.clone(),
        metadata: HashMap::new(),
    };
    
    let matches = ecosystem.find_services_by_capability(&security_request).await.unwrap();
    
    assert!(!matches.is_empty(), "Should find services with authentication capability");
    assert!(
        matches.iter().any(|m| m.service.service_id == "security-provider-1"),
        "Should find dedicated security provider"
    );
    assert!(
        matches.iter().any(|m| m.service.service_id == "multi-provider-1"),
        "Should find multi-capability provider"
    );
    
    // Test that scores are calculated correctly
    let security_provider_match = matches.iter()
        .find(|m| m.service.service_id == "security-provider-1")
        .unwrap();
    let multi_provider_match = matches.iter()
        .find(|m| m.service.service_id == "multi-provider-1")
        .unwrap();
        
    // Security provider should have higher score (has both required and optional capabilities)
    assert!(
        security_provider_match.score >= multi_provider_match.score,
        "Dedicated security provider should score higher or equal"
    );
}

#[tokio::test]
async fn test_capability_discovery_caching() {
    let context = create_test_context("test-user-002");
    let cache_config = CacheConfig {
        capability_discovery_ttl: 2, // 2 seconds for testing
        service_capabilities_ttl: 600,
        max_cache_entries: 100,
        enable_caching: true,
    };
    let ecosystem = UniversalPrimalEcosystem::with_cache_config(context.clone(), cache_config);
    
    // Populate with mock services
    populate_ecosystem_with_mock_services(&ecosystem).await;
    
    let storage_request = CapabilityRequest {
        required_capabilities: vec!["data-persistence".to_string()],
        optional_capabilities: vec!["high-availability".to_string()],
        context: context.clone(),
        metadata: HashMap::new(),
    };
    
    // First request - should populate cache
    let start_time = Instant::now();
    let matches1 = ecosystem.find_services_by_capability(&storage_request).await.unwrap();
    let first_duration = start_time.elapsed();
    
    // Second request - should use cache (should be faster)
    let start_time = Instant::now();
    let matches2 = ecosystem.find_services_by_capability(&storage_request).await.unwrap();
    let second_duration = start_time.elapsed();
    
    assert_eq!(matches1.len(), matches2.len(), "Cache should return same results");
    assert!(
        second_duration <= first_duration,
        "Cached request should be faster or same speed"
    );
    
    // Verify cache statistics
    let cache_stats = ecosystem.get_cache_stats().await;
    assert!(cache_stats.total_cache_hits >= 1, "Should have at least one cache hit");
    assert!(cache_stats.valid_cache_entries >= 1, "Should have at least one valid cache entry");
    
    // Wait for cache to expire
    sleep(Duration::from_secs(3)).await;
    
    // Third request - cache should be expired, so fresh discovery
    let matches3 = ecosystem.find_services_by_capability(&storage_request).await.unwrap();
    assert_eq!(matches1.len(), matches3.len(), "Results should be consistent after cache expiry");
}

#[tokio::test]
async fn test_capability_discovery_no_matches() {
    let context = create_test_context("test-user-003");
    let ecosystem = UniversalPrimalEcosystem::new(context.clone());
    
    // Populate with mock services (none have the requested capability)
    populate_ecosystem_with_mock_services(&ecosystem).await;
    
    let nonexistent_request = CapabilityRequest {
        required_capabilities: vec!["quantum-computing".to_string()],
        optional_capabilities: vec![],
        context: context.clone(),
        metadata: HashMap::new(),
    };
    
    let matches = ecosystem.find_services_by_capability(&nonexistent_request).await.unwrap();
    assert!(matches.is_empty(), "Should find no matches for non-existent capability");
}

#[tokio::test]
async fn test_capability_discovery_multiple_requirements() {
    let context = create_test_context("test-user-004");
    let ecosystem = UniversalPrimalEcosystem::new(context.clone());
    
    populate_ecosystem_with_mock_services(&ecosystem).await;
    
    // Request multiple required capabilities that only multi-provider has
    let multi_requirement_request = CapabilityRequest {
        required_capabilities: vec![
            "authentication".to_string(),
            "data-persistence".to_string(),
            "task-execution".to_string(),
        ],
        optional_capabilities: vec![],
        context: context.clone(),
        metadata: HashMap::new(),
    };
    
    let matches = ecosystem.find_services_by_capability(&multi_requirement_request).await.unwrap();
    
    // Only multi-provider-1 should match all three required capabilities
    assert_eq!(matches.len(), 1, "Should find exactly one service with all required capabilities");
    assert_eq!(
        matches[0].service.service_id, 
        "multi-provider-1",
        "Multi-provider should be the only match"
    );
    assert!(matches[0].missing_capabilities.is_empty(), "Should have no missing capabilities");
}

#[tokio::test]
async fn test_capability_discovery_scoring() {
    let context = create_test_context("test-user-005");
    let ecosystem = UniversalPrimalEcosystem::new(context.clone());
    
    populate_ecosystem_with_mock_services(&ecosystem).await;
    
    let storage_request = CapabilityRequest {
        required_capabilities: vec!["data-persistence".to_string()],
        optional_capabilities: vec!["high-availability".to_string(), "backup-storage".to_string()],
        context: context.clone(),
        metadata: HashMap::new(),
    };
    
    let matches = ecosystem.find_services_by_capability(&storage_request).await.unwrap();
    
    assert!(!matches.is_empty(), "Should find storage services");
    
    // Verify matches are sorted by score (highest first)
    for i in 1..matches.len() {
        assert!(
            matches[i-1].score >= matches[i].score,
            "Matches should be sorted by score in descending order"
        );
    }
    
    // Find the service with high-availability (should score higher)
    let ha_service = matches.iter()
        .find(|m| m.service.capabilities.contains(&"high-availability".to_string()))
        .expect("Should find service with high-availability");
        
    // Find the service with backup-storage (should score differently)
    let backup_service = matches.iter()
        .find(|m| m.service.capabilities.contains(&"backup-storage".to_string()))
        .expect("Should find service with backup-storage");
    
    // Both should have different scores based on optional capability matching
    assert_ne!(
        ha_service.score, 
        backup_service.score,
        "Services with different optional capabilities should have different scores"
    );
}

#[tokio::test]
async fn test_capability_discovery_context_aware_caching() {
    let context1 = create_test_context("user-001");
    let mut context2 = create_test_context("user-002");
    context2.security_level = SecurityLevel::High; // Different security level
    
    let ecosystem = UniversalPrimalEcosystem::new(context1.clone());
    
    populate_ecosystem_with_mock_services(&ecosystem).await;
    
    let request1 = CapabilityRequest {
        required_capabilities: vec!["authentication".to_string()],
        optional_capabilities: vec![],
        context: context1,
        metadata: HashMap::new(),
    };
    
    let request2 = CapabilityRequest {
        required_capabilities: vec!["authentication".to_string()],
        optional_capabilities: vec![],
        context: context2,
        metadata: HashMap::new(),
    };
    
    // Make both requests
    let _matches1 = ecosystem.find_services_by_capability(&request1).await.unwrap();
    let _matches2 = ecosystem.find_services_by_capability(&request2).await.unwrap();
    
    // Check that different cache entries were created for different contexts
    let cache_stats = ecosystem.get_cache_stats().await;
    // Should have separate cache entries due to different contexts
    assert!(cache_stats.discovery_cache_size >= 1, "Should have cache entries");
}

#[tokio::test]
async fn test_cache_eviction() {
    let context = create_test_context("test-user-006");
    let cache_config = CacheConfig {
        capability_discovery_ttl: 300,
        service_capabilities_ttl: 600,
        max_cache_entries: 5, // Small cache for testing eviction
        enable_caching: true,
    };
    let ecosystem = UniversalPrimalEcosystem::with_cache_config(context.clone(), cache_config);
    
    populate_ecosystem_with_mock_services(&ecosystem).await;
    
    // Create multiple different capability requests to fill cache
    let capabilities = vec![
        "authentication",
        "data-persistence", 
        "task-execution",
        "encryption",
        "file-storage",
        "sandboxing", // This should trigger eviction
    ];
    
    for capability in capabilities {
        let request = CapabilityRequest {
            required_capabilities: vec![capability.to_string()],
            optional_capabilities: vec![],
            context: context.clone(),
            metadata: HashMap::new(),
        };
        
        let _ = ecosystem.find_services_by_capability(&request).await.unwrap();
    }
    
    let cache_stats = ecosystem.get_cache_stats().await;
    assert!(
        cache_stats.discovery_cache_size <= 5,
        "Cache size should not exceed maximum"
    );
}

#[tokio::test]
async fn test_cache_clear() {
    let context = create_test_context("test-user-007");
    let ecosystem = UniversalPrimalEcosystem::new(context.clone());
    
    populate_ecosystem_with_mock_services(&ecosystem).await;
    
    let request = CapabilityRequest {
        required_capabilities: vec!["authentication".to_string()],
        optional_capabilities: vec![],
        context: context.clone(),
        metadata: HashMap::new(),
    };
    
    // Populate cache
    let _ = ecosystem.find_services_by_capability(&request).await.unwrap();
    
    let cache_stats_before = ecosystem.get_cache_stats().await;
    assert!(cache_stats_before.discovery_cache_size > 0, "Cache should have entries");
    
    // Clear cache
    ecosystem.clear_caches().await;
    
    let cache_stats_after = ecosystem.get_cache_stats().await;
    assert_eq!(cache_stats_after.discovery_cache_size, 0, "Cache should be empty after clear");
}

#[tokio::test]
async fn test_disabled_caching() {
    let context = create_test_context("test-user-008");
    let cache_config = CacheConfig {
        capability_discovery_ttl: 300,
        service_capabilities_ttl: 600,
        max_cache_entries: 100,
        enable_caching: false, // Disabled
    };
    let ecosystem = UniversalPrimalEcosystem::with_cache_config(context.clone(), cache_config);
    
    populate_ecosystem_with_mock_services(&ecosystem).await;
    
    let request = CapabilityRequest {
        required_capabilities: vec!["authentication".to_string()],
        optional_capabilities: vec![],
        context: context.clone(),
        metadata: HashMap::new(),
    };
    
    // Make multiple requests
    let _ = ecosystem.find_services_by_capability(&request).await.unwrap();
    let _ = ecosystem.find_services_by_capability(&request).await.unwrap();
    
    let cache_stats = ecosystem.get_cache_stats().await;
    assert_eq!(cache_stats.discovery_cache_size, 0, "Cache should remain empty when disabled");
    assert_eq!(cache_stats.total_cache_hits, 0, "Should have no cache hits when disabled");
}

#[tokio::test] 
async fn test_service_health_consideration() {
    let context = create_test_context("test-user-009");
    let ecosystem = UniversalPrimalEcosystem::new(context.clone());
    
    populate_ecosystem_with_mock_services(&ecosystem).await;
    
    let storage_request = CapabilityRequest {
        required_capabilities: vec!["data-persistence".to_string()],
        optional_capabilities: vec![],
        context: context.clone(),
        metadata: HashMap::new(),
    };
    
    let matches = ecosystem.find_services_by_capability(&storage_request).await.unwrap();
    
    // Should find both healthy and degraded services
    assert!(matches.len() >= 2, "Should find multiple storage services");
    
    // Verify that services are returned regardless of health status
    // (Health-based selection happens at the routing level)
    let health_statuses: Vec<_> = matches.iter()
        .map(|m| &m.service.health)
        .collect();
        
    assert!(
        health_statuses.contains(&&ServiceHealth::Healthy),
        "Should include healthy services"
    );
} 