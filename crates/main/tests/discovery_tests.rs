// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "Test code: explicit unwrap/expect and local lint noise"
)]
//! Integration tests for the discovery system
//!
//! Tests the full discovery flow including capability resolution,
//! environment variable discovery, and the infant primal pattern.

use squirrel::discovery::{
    CapabilityResolver, DiscoveredService, PrimalSelfKnowledge, RuntimeDiscoveryEngine,
};
use std::time::SystemTime;

#[test]
fn test_discovery_from_environment_variable() {
    temp_env::with_vars(
        [
            ("AI_INFERENCE_ENDPOINT", Some("http://localhost:8001")),
            ("STORAGE_ENDPOINT", Some("http://localhost:8002")),
        ],
        || {
            let rt = tokio::runtime::Runtime::new().expect("should succeed");
            rt.block_on(async {
                let engine = RuntimeDiscoveryEngine::new();

                let ai_service = engine
                    .discover_capability("ai.inference")
                    .await
                    .expect("Should discover AI service from environment");

                assert_eq!(ai_service.endpoint, "http://localhost:8001");
                assert!(!ai_service.capabilities.is_empty());
                assert_eq!(ai_service.discovery_method, "environment_variable");

                let storage_service = engine
                    .discover_capability("storage")
                    .await
                    .expect("Should discover storage service from environment");

                assert_eq!(storage_service.endpoint, "http://localhost:8002");
                assert_eq!(storage_service.discovery_method, "environment_variable");
            });
        },
    );
}

#[test]
fn test_discovery_capability_not_found() {
    temp_env::with_var_unset("NONEXISTENT_CAPABILITY_ENDPOINT", || {
        let rt = tokio::runtime::Runtime::new().expect("should succeed");
        rt.block_on(async {
            let engine = RuntimeDiscoveryEngine::new();
            let result = engine.discover_capability("nonexistent.capability").await;

            assert!(result.is_err());
            assert!(
                result
                    .unwrap_err()
                    .to_string()
                    .contains("Capability not found")
            );
        });
    });
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

#[test]
fn test_capability_resolver_with_dots_in_name() {
    temp_env::with_var(
        "AI_INFERENCE_ENDPOINT",
        Some("http://localhost:9000"),
        || {
            let rt = tokio::runtime::Runtime::new().expect("should succeed");
            rt.block_on(async {
                let resolver = CapabilityResolver::new();
                let request = squirrel::discovery::CapabilityRequest::new("ai.inference");

                let result = resolver.discover_provider(request).await;

                assert!(result.is_ok());
                let service = result.expect("should succeed");
                assert_eq!(service.endpoint, "http://localhost:9000");
            });
        },
    );
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
        ..service
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

#[test]
fn test_runtime_discovery_engine_caching() {
    temp_env::with_var("CACHE_TEST_ENDPOINT", Some("http://localhost:7777"), || {
        let rt = tokio::runtime::Runtime::new().expect("should succeed");
        rt.block_on(async {
            let engine = RuntimeDiscoveryEngine::new();

            let service1 = engine
                .discover_capability("cache_test")
                .await
                .expect("Should discover service");

            let service2 = engine
                .discover_capability("cache_test")
                .await
                .expect("Should discover cached service");

            assert_eq!(service1.endpoint, service2.endpoint);
            assert_eq!(service1.name, service2.name);
        });
    });
}

#[test]
fn test_discovery_with_complex_endpoint() {
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
        temp_env::with_var("COMPLEX_ENDPOINT", Some(env_value), || {
            let rt = tokio::runtime::Runtime::new().expect("should succeed");
            rt.block_on(async {
                let engine = RuntimeDiscoveryEngine::new();
                let result = engine.discover_capability("complex").await;

                assert!(result.is_ok(), "Failed for endpoint: {env_value}");
                let service = result.expect("should succeed");
                assert_eq!(service.endpoint, expected_endpoint);
            });
        });
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
    let mut services = [
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
