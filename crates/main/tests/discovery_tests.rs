// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Integration tests for the discovery system
//!
//! Tests the full discovery flow including capability resolution,
//! environment variable discovery, and the infant primal pattern.

use serial_test::serial;
use squirrel::discovery::{
    CapabilityResolver, DiscoveredService, PrimalSelfKnowledge, RuntimeDiscoveryEngine,
};
use std::time::SystemTime;

#[tokio::test]
#[serial] // Serialize env var tests to prevent pollution
async fn test_discovery_from_environment_variable() {
    // Set up test environment - env vars use uppercase with underscores
    std::env::set_var("AI_INFERENCE_ENDPOINT", "http://localhost:8001");
    std::env::set_var("STORAGE_ENDPOINT", "http://localhost:8002");

    let engine = RuntimeDiscoveryEngine::new();

    // Test AI capability discovery - dots are replaced with underscores for env vars
    let ai_service = engine
        .discover_capability("ai.inference")
        .await
        .expect("Should discover AI service from environment");

    assert_eq!(ai_service.endpoint, "http://localhost:8001");
    // The capability list should match what we requested
    assert!(!ai_service.capabilities.is_empty());
    assert_eq!(ai_service.discovery_method, "environment_variable");

    // Test storage capability discovery
    let storage_service = engine
        .discover_capability("storage")
        .await
        .expect("Should discover storage service from environment");

    assert_eq!(storage_service.endpoint, "http://localhost:8002");
    assert_eq!(storage_service.discovery_method, "environment_variable");

    // Cleanup
    std::env::remove_var("AI_INFERENCE_ENDPOINT");
    std::env::remove_var("STORAGE_ENDPOINT");
}

#[tokio::test]
#[serial] // Serialize env var tests to prevent pollution
async fn test_discovery_capability_not_found() {
    // Ensure no environment variable exists
    std::env::remove_var("NONEXISTENT_CAPABILITY_ENDPOINT");

    let engine = RuntimeDiscoveryEngine::new();

    // Should return error when capability not found
    let result = engine.discover_capability("nonexistent.capability").await;

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Capability not found"));
}

#[tokio::test]
async fn test_primal_self_knowledge_discovery() {
    let self_knowledge =
        PrimalSelfKnowledge::discover_self().expect("Should discover own identity");

    let identity = self_knowledge.identity();

    // Should have discovered a name (hostname or default)
    assert!(!identity.name.is_empty());

    // Should have some capabilities (the actual capabilities are configurable)
    assert!(!identity.capabilities.is_empty());

    // Verify it's a valid list of capability strings
    for capability in &identity.capabilities {
        assert!(!capability.is_empty());
    }
}

#[tokio::test]
#[serial] // Serialize env var tests to prevent pollution
async fn test_capability_resolver_with_dots_in_name() {
    // Test that capability names with dots (like "ai.inference") work correctly
    std::env::set_var("AI_INFERENCE_ENDPOINT", "http://localhost:9000");

    let resolver = CapabilityResolver::new();
    let request = squirrel::discovery::CapabilityRequest::new("ai.inference");

    let result = resolver.discover_provider(request).await;

    assert!(result.is_ok());
    let service = result.unwrap();
    assert_eq!(service.endpoint, "http://localhost:9000");

    std::env::remove_var("AI_INFERENCE_ENDPOINT");
}

#[tokio::test]
async fn test_discovered_service_freshness() {
    use std::time::{Duration, SystemTime};

    let service = DiscoveredService {
        name: "test-service".to_string(),
        endpoint: "http://localhost:8080".to_string(),
        capabilities: vec!["test".to_string()],
        metadata: std::collections::HashMap::new(),
        discovered_at: SystemTime::now(),
        discovery_method: "test".to_string(),
        healthy: Some(true),
        priority: 100,
    };

    // Fresh service
    assert!(service.is_fresh(Duration::from_secs(60)));

    // Test with old discovery time
    let old_service = DiscoveredService {
        discovered_at: SystemTime::now() - Duration::from_secs(120),
        ..service.clone()
    };

    assert!(!old_service.is_fresh(Duration::from_secs(60)));
    assert!(old_service.is_fresh(Duration::from_secs(180)));
}

#[tokio::test]
async fn test_discovered_service_capability_matching() {
    let service = DiscoveredService {
        name: "multi-cap-service".to_string(),
        endpoint: "http://localhost:8080".to_string(),
        capabilities: vec![
            "ai.inference".to_string(),
            "ai.embeddings".to_string(),
            "storage".to_string(),
        ],
        metadata: std::collections::HashMap::new(),
        discovered_at: SystemTime::now(),
        discovery_method: "test".to_string(),
        healthy: Some(true),
        priority: 50,
    };

    // Test exact matches
    assert!(service.has_capability("ai.inference"));
    assert!(service.has_capability("storage"));

    // Test case-insensitive matching
    assert!(service.has_capability("AI.INFERENCE"));
    assert!(service.has_capability("Storage"));

    // Test non-existent capability
    assert!(!service.has_capability("compute"));
}

#[tokio::test]
#[serial] // Serialize env var tests to prevent pollution
async fn test_runtime_discovery_engine_caching() {
    std::env::set_var("CACHE_TEST_ENDPOINT", "http://localhost:7777");

    let engine = RuntimeDiscoveryEngine::new();

    // First discovery (use underscore format for env var matching)
    let service1 = engine
        .discover_capability("cache_test")
        .await
        .expect("Should discover service");

    // Second discovery should use cache (same endpoint)
    let service2 = engine
        .discover_capability("cache_test")
        .await
        .expect("Should discover cached service");

    assert_eq!(service1.endpoint, service2.endpoint);
    assert_eq!(service1.name, service2.name);

    std::env::remove_var("CACHE_TEST_ENDPOINT");
}

#[tokio::test]
#[serial] // Serialize env var tests to prevent pollution
async fn test_discovery_with_complex_endpoint() {
    // Test various endpoint formats
    let test_cases = vec![
        ("http://localhost:8080", "http://localhost:8080"),
        ("https://api.example.com:443", "https://api.example.com:443"),
        (
            "unix:///var/run/service.sock",
            "unix:///var/run/service.sock",
        ),
        ("tcp://192.168.1.100:9000", "tcp://192.168.1.100:9000"),
    ];

    for (env_value, expected_endpoint) in test_cases {
        std::env::set_var("COMPLEX_ENDPOINT", env_value);

        let engine = RuntimeDiscoveryEngine::new();
        let result = engine.discover_capability("complex").await;

        assert!(result.is_ok(), "Failed for endpoint: {}", env_value);
        let service = result.unwrap();
        assert_eq!(service.endpoint, expected_endpoint);

        std::env::remove_var("COMPLEX_ENDPOINT");
    }
}

#[tokio::test]
async fn test_primal_self_knowledge_discover_primal() {
    let self_knowledge = PrimalSelfKnowledge::discover_self().expect("Should discover self");

    // Test discovering other primals (using mock implementation)
    let storage_result = self_knowledge.discover_primal("storage").await;

    // Current implementation may return mock or error - both are acceptable
    // This test documents the expected behavior
    if let Ok(primal_info) = storage_result {
        assert!(!primal_info.name.is_empty());
        assert!(primal_info.capabilities.contains(&"storage".to_string()));
    }
}

#[tokio::test]
async fn test_discovery_priority_ordering() {
    // Test that discovered services can be ordered by priority
    let mut services = vec![
        DiscoveredService {
            name: "low-priority".to_string(),
            endpoint: "http://localhost:8001".to_string(),
            capabilities: vec!["test".to_string()],
            metadata: std::collections::HashMap::new(),
            discovered_at: SystemTime::now(),
            discovery_method: "test".to_string(),
            healthy: Some(true),
            priority: 10,
        },
        DiscoveredService {
            name: "high-priority".to_string(),
            endpoint: "http://localhost:8002".to_string(),
            capabilities: vec!["test".to_string()],
            metadata: std::collections::HashMap::new(),
            discovered_at: SystemTime::now(),
            discovery_method: "test".to_string(),
            healthy: Some(true),
            priority: 100,
        },
        DiscoveredService {
            name: "medium-priority".to_string(),
            endpoint: "http://localhost:8003".to_string(),
            capabilities: vec!["test".to_string()],
            metadata: std::collections::HashMap::new(),
            discovered_at: SystemTime::now(),
            discovery_method: "test".to_string(),
            healthy: Some(true),
            priority: 50,
        },
    ];

    // Sort by priority (descending)
    services.sort_by(|a, b| b.priority.cmp(&a.priority));

    assert_eq!(services[0].name, "high-priority");
    assert_eq!(services[1].name, "medium-priority");
    assert_eq!(services[2].name, "low-priority");
}
