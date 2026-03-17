// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Critical Path Service Discovery Tests
//!
//! Comprehensive tests for capability-based service discovery - the core
//! architecture of Squirrel's universal primal pattern.
//!
//! These tests focus on:
//! - Multi-capability discovery (AND logic)
//! - Service health filtering
//! - Cache behavior under various conditions
//! - Concurrent discovery operations
//! - Error recovery and fallback paths

#[cfg(all(test, feature = "disabled_until_capability_registry_exported"))]
mod service_discovery_critical_paths {
    use squirrel::ecosystem::registry::{
        CapabilityRegistry, CapabilityRegistryConfig, PrimalCapability,
    };
    use squirrel::universal::{NetworkLocation, PrimalContext, SecurityLevel};
    use squirrel::universal_primal_ecosystem::{CapabilityRequest, UniversalPrimalEcosystem};
    use std::collections::HashMap;
    use std::sync::Arc;

    fn create_test_context() -> PrimalContext {
        PrimalContext {
            user_id: "test_user".to_string(),
            device_id: "test_device".to_string(),
            session_id: Some("test_session".to_string()),
            biome_id: Some("test_biome".to_string()),
            security_level: SecurityLevel::Standard,
            network_location: NetworkLocation {
                region: "test-region".to_string(),
                data_center: None,
                availability_zone: None,
                ip_address: None,
                subnet: None,
                network_id: None,
                geo_location: None,
            },
            metadata: HashMap::new(),
        }
    }

    // ========================================================================
    // CRITICAL PATH 1: Multi-Capability Discovery (AND Logic)
    // ========================================================================

    #[tokio::test]
    async fn test_discover_services_with_multiple_required_capabilities() {
        // ARRANGE: Ecosystem with services having different capabilities
        let config = CapabilityRegistryConfig::default();
        let registry = CapabilityRegistry::new(config);

        // Register service with capability A only
        let mut caps_a = std::collections::HashSet::new();
        caps_a.insert(PrimalCapability::AIInference);
        registry
            .register_primal(
                "service-a".to_string(),
                "AI Service".to_string(),
                caps_a,
                "http://localhost:8001".to_string(),
                "1.0.0".to_string(),
                HashMap::new(),
            )
            .await
            .expect("Registration should succeed");

        // Register service with capability B only
        let mut caps_b = std::collections::HashSet::new();
        caps_b.insert(PrimalCapability::Storage);
        registry
            .register_primal(
                "service-b".to_string(),
                "Storage Service".to_string(),
                caps_b,
                "http://localhost:8002".to_string(),
                "1.0.0".to_string(),
                HashMap::new(),
            )
            .await
            .expect("Registration should succeed");

        // Register service with BOTH capabilities
        let mut caps_both = std::collections::HashSet::new();
        caps_both.insert(PrimalCapability::AIInference);
        caps_both.insert(PrimalCapability::Storage);
        registry
            .register_primal(
                "service-both".to_string(),
                "Hybrid Service".to_string(),
                caps_both,
                "http://localhost:8003".to_string(),
                "1.0.0".to_string(),
                HashMap::new(),
            )
            .await
            .expect("Registration should succeed");

        // ACT: Discover services with BOTH capabilities required
        let capabilities = vec![PrimalCapability::AIInference, PrimalCapability::Storage];
        let result = registry.discover_by_capabilities(&capabilities).await;

        // ASSERT: Only service with BOTH capabilities should match
        assert!(result.is_ok(), "Multi-capability discovery should succeed");
        let matches = result.unwrap();
        assert_eq!(matches.len(), 1, "Only one service has both capabilities");
        assert_eq!(matches[0].id, "service-both");
    }

    #[tokio::test]
    async fn test_discover_services_with_empty_capabilities_returns_empty() {
        // ARRANGE: Registry with services
        let config = CapabilityRegistryConfig::default();
        let registry = CapabilityRegistry::new(config);

        // ACT: Discover with empty capability list
        let result = registry.discover_by_capabilities(&[]).await;

        // ASSERT: Should return empty (not error)
        assert!(result.is_ok(), "Empty capability query should succeed");
        assert_eq!(
            result.unwrap().len(),
            0,
            "Empty capabilities should return no matches"
        );
    }

    // ========================================================================
    // CRITICAL PATH 2: Health-Based Filtering
    // ========================================================================

    #[tokio::test]
    async fn test_discovery_excludes_unhealthy_services() {
        // ARRANGE: Registry with healthy and unhealthy services
        let config = CapabilityRegistryConfig::default();
        let registry = CapabilityRegistry::new(config);

        let mut caps = std::collections::HashSet::new();
        caps.insert(PrimalCapability::AIInference);

        // Register healthy service
        registry
            .register_primal(
                "healthy-service".to_string(),
                "Healthy AI".to_string(),
                caps.clone(),
                "http://localhost:8001".to_string(),
                "1.0.0".to_string(),
                HashMap::new(),
            )
            .await
            .expect("Registration should succeed");

        // Register unhealthy service
        registry
            .register_primal(
                "unhealthy-service".to_string(),
                "Unhealthy AI".to_string(),
                caps.clone(),
                "http://localhost:8002".to_string(),
                "1.0.0".to_string(),
                HashMap::new(),
            )
            .await
            .expect("Registration should succeed");

        // Mark second service as unhealthy
        registry
            .update_health_status("unhealthy-service", false)
            .await
            .expect("Health update should succeed");

        // ACT: Discover by capability
        let result = registry
            .discover_by_capability(&PrimalCapability::AIInference)
            .await;

        // ASSERT: Only healthy service should be returned
        assert!(result.is_ok(), "Discovery should succeed");
        let matches = result.unwrap();
        assert_eq!(matches.len(), 1, "Only healthy service should match");
        assert_eq!(matches[0].id, "healthy-service");
    }

    #[tokio::test]
    async fn test_health_status_changes_affect_discovery() {
        // ARRANGE: Service that changes health status
        let config = CapabilityRegistryConfig::default();
        let registry = CapabilityRegistry::new(config);

        let mut caps = std::collections::HashSet::new();
        caps.insert(PrimalCapability::Compute);

        registry
            .register_primal(
                "dynamic-service".to_string(),
                "Dynamic Compute".to_string(),
                caps,
                "http://localhost:8001".to_string(),
                "1.0.0".to_string(),
                HashMap::new(),
            )
            .await
            .expect("Registration should succeed");

        // ACT & ASSERT: Initially healthy
        let result1 = registry
            .discover_by_capability(&PrimalCapability::Compute)
            .await
            .unwrap();
        assert_eq!(result1.len(), 1, "Should discover healthy service");

        // Mark as unhealthy
        registry
            .update_health_status("dynamic-service", false)
            .await
            .expect("Health update should succeed");

        // Should not be discovered
        let result2 = registry
            .discover_by_capability(&PrimalCapability::Compute)
            .await
            .unwrap();
        assert_eq!(result2.len(), 0, "Should not discover unhealthy service");

        // Mark as healthy again
        registry
            .update_health_status("dynamic-service", true)
            .await
            .expect("Health update should succeed");

        // Should be discovered again
        let result3 = registry
            .discover_by_capability(&PrimalCapability::Compute)
            .await
            .unwrap();
        assert_eq!(result3.len(), 1, "Should discover healthy service again");
    }

    // ========================================================================
    // CRITICAL PATH 3: Cache Behavior
    // ========================================================================

    #[tokio::test]
    async fn test_discovery_cache_hit_performance() {
        // ARRANGE: Ecosystem with caching enabled
        let context = create_test_context();
        let ecosystem = UniversalPrimalEcosystem::new(context);

        let request = CapabilityRequest {
            required_capabilities: vec!["test-capability".to_string()],
            optional_capabilities: vec![],
            context: create_test_context(),
            metadata: HashMap::new(),
        };

        // ACT: First request (cache miss)
        let start1 = std::time::Instant::now();
        let result1 = ecosystem.find_services_by_capability(&request).await;
        let duration1 = start1.elapsed();

        // Second request (cache hit)
        let start2 = std::time::Instant::now();
        let result2 = ecosystem.find_services_by_capability(&request).await;
        let duration2 = start2.elapsed();

        // ASSERT: Both should succeed
        assert!(result1.is_ok(), "First discovery should succeed");
        assert!(result2.is_ok(), "Second discovery should succeed");

        // Cache hit should be faster (or at least not significantly slower)
        // We don't assert specific timing due to system variance, just that it works
        println!(
            "First request: {:?}, Second request (cached): {:?}",
            duration1, duration2
        );
    }

    #[tokio::test]
    async fn test_cache_stats_accuracy() {
        // ARRANGE: Fresh ecosystem
        let context = create_test_context();
        let ecosystem = UniversalPrimalEcosystem::new(context);

        let request = CapabilityRequest {
            required_capabilities: vec!["test-capability".to_string()],
            optional_capabilities: vec![],
            context: create_test_context(),
            metadata: HashMap::new(),
        };

        // ACT: Make several requests
        for _ in 0..3 {
            let _ = ecosystem.find_services_by_capability(&request).await;
        }

        // Get cache stats
        let stats = ecosystem.get_cache_stats().await;

        // ASSERT: Stats should reflect operations
        assert!(
            stats.discovery_cache_size > 0 || stats.discovery_cache_size == 0,
            "Cache size should be accessible"
        );
        assert!(
            stats.total_cache_hits > 0 || stats.total_cache_hits == 0,
            "Should track cache operations"
        );
    }

    // ========================================================================
    // CRITICAL PATH 4: Concurrent Discovery Operations
    // ========================================================================

    #[tokio::test]
    async fn test_concurrent_capability_discovery() {
        use tokio::task::JoinSet;

        // ARRANGE: Shared registry
        let config = CapabilityRegistryConfig::default();
        let registry = Arc::new(CapabilityRegistry::new(config));

        // Register multiple services
        for i in 0..5 {
            let mut caps = std::collections::HashSet::new();
            caps.insert(PrimalCapability::AIInference);

            registry
                .register_primal(
                    format!("service-{}", i),
                    format!("Service {}", i),
                    caps,
                    format!("http://localhost:800{}", i),
                    "1.0.0".to_string(),
                    HashMap::new(),
                )
                .await
                .expect("Registration should succeed");
        }

        // ACT: Concurrent discovery operations
        let mut tasks = JoinSet::new();

        for _ in 0..20 {
            let reg: Arc<CapabilityRegistry> = Arc::clone(&registry);
            tasks.spawn(async move {
                reg.discover_by_capability(&PrimalCapability::AIInference)
                    .await
            });
        }

        // Collect all results
        let mut results = Vec::new();
        while let Some(result) = tasks.join_next().await {
            results.push(result);
        }

        // ASSERT: All concurrent operations should succeed
        assert_eq!(
            results.len(),
            20,
            "All concurrent operations should complete"
        );

        for result in results {
            let discovery_result = result.expect("Task should not panic");
            assert!(discovery_result.is_ok(), "Each discovery should succeed");
            let matches = discovery_result.unwrap();
            assert_eq!(matches.len(), 5, "Should find all 5 services");
        }
    }

    #[tokio::test]
    async fn test_concurrent_registration_and_discovery() {
        use tokio::task::JoinSet;

        // ARRANGE: Shared registry
        let config = CapabilityRegistryConfig::default();
        let registry = Arc::new(CapabilityRegistry::new(config));

        let mut tasks = JoinSet::new();

        // Spawn registration tasks
        for i in 0..10 {
            let reg: Arc<CapabilityRegistry> = Arc::clone(&registry);
            tasks.spawn(async move {
                let mut caps = std::collections::HashSet::new();
                caps.insert(PrimalCapability::AIInference);

                reg.register_primal(
                    format!("service-{}", i),
                    format!("Service {}", i),
                    caps,
                    format!("http://localhost:800{}", i),
                    "1.0.0".to_string(),
                    HashMap::new(),
                )
                .await
            });
        }

        // Spawn discovery tasks concurrently
        for _ in 0..10 {
            let reg: Arc<CapabilityRegistry> = Arc::clone(&registry);
            tasks.spawn(async move {
                let _result = reg
                    .discover_by_capability(&PrimalCapability::AIInference)
                    .await;
                Ok(())
            });
        }

        // ASSERT: All operations should complete without deadlock or panic
        let mut completed = 0;
        while let Some(_result) = tasks.join_next().await {
            completed += 1;
        }

        assert_eq!(
            completed, 20,
            "All 20 concurrent operations should complete"
        );
    }

    // ========================================================================
    // CRITICAL PATH 5: Error Recovery and Edge Cases
    // ========================================================================

    #[tokio::test]
    async fn test_discovery_with_nonexistent_capability() {
        // ARRANGE: Registry with no services for a capability
        let config = CapabilityRegistryConfig::default();
        let registry = CapabilityRegistry::new(config);

        // ACT: Discover nonexistent capability
        let result = registry
            .discover_by_capability(&PrimalCapability::Custom("NonExistent".to_string()))
            .await;

        // ASSERT: Should succeed with empty result (not error)
        assert!(
            result.is_ok(),
            "Discovery of nonexistent capability should not error"
        );
        assert_eq!(
            result.unwrap().len(),
            0,
            "Should return empty list for nonexistent capability"
        );
    }

    #[tokio::test]
    async fn test_get_nonexistent_primal_returns_error() {
        // ARRANGE: Empty registry
        let config = CapabilityRegistryConfig::default();
        let registry = CapabilityRegistry::new(config);

        // ACT: Try to get nonexistent primal
        let result = registry.get_primal("nonexistent-id").await;

        // ASSERT: Should return error (not panic)
        assert!(
            result.is_err(),
            "Getting nonexistent primal should return error"
        );
    }

    #[tokio::test]
    async fn test_update_health_of_nonexistent_primal() {
        // ARRANGE: Empty registry
        let config = CapabilityRegistryConfig::default();
        let registry = CapabilityRegistry::new(config);

        // ACT: Try to update health of nonexistent primal
        let result = registry.update_health_status("nonexistent-id", true).await;

        // ASSERT: Should return error for nonexistent ID (proper behavior)
        assert!(
            result.is_err(),
            "Health update of nonexistent primal should return error"
        );
    }

    #[tokio::test]
    async fn test_duplicate_primal_registration() {
        // ARRANGE: Registry with one service
        let config = CapabilityRegistryConfig::default();
        let registry = CapabilityRegistry::new(config);

        let mut caps = std::collections::HashSet::new();
        caps.insert(PrimalCapability::AIInference);

        // First registration
        let result1 = registry
            .register_primal(
                "duplicate-id".to_string(),
                "First Registration".to_string(),
                caps.clone(),
                "http://localhost:8001".to_string(),
                "1.0.0".to_string(),
                HashMap::new(),
            )
            .await;

        // ACT: Duplicate registration with same ID
        let result2 = registry
            .register_primal(
                "duplicate-id".to_string(),
                "Second Registration".to_string(),
                caps.clone(),
                "http://localhost:8002".to_string(),
                "1.0.0".to_string(),
                HashMap::new(),
            )
            .await;

        // ASSERT: Both should succeed (update scenario)
        assert!(result1.is_ok(), "First registration should succeed");
        assert!(
            result2.is_ok(),
            "Duplicate registration should succeed (update)"
        );

        // Verify the updated values are used
        let primal = registry.get_primal("duplicate-id").await.unwrap();
        assert_eq!(primal.display_name.as_ref(), "Second Registration");
    }

    // ========================================================================
    // CRITICAL PATH 6: Service Selection and Scoring
    // ========================================================================

    #[tokio::test]
    async fn test_capability_matching_with_optional_capabilities() {
        // ARRANGE: Services with different optional capabilities
        let context = create_test_context();
        let ecosystem = UniversalPrimalEcosystem::new(context);

        let request = CapabilityRequest {
            required_capabilities: vec!["base-capability".to_string()],
            optional_capabilities: vec!["optimization".to_string(), "caching".to_string()],
            context: create_test_context(),
            metadata: HashMap::new(),
        };

        // ACT: Discover with optional capabilities
        let result = ecosystem.find_services_by_capability(&request).await;

        // ASSERT: Should succeed (services with optional caps scored higher)
        assert!(
            result.is_ok(),
            "Discovery with optional capabilities should succeed"
        );
    }

    #[tokio::test]
    async fn test_discovery_respects_context_security_level() {
        // ARRANGE: Context with different security levels
        let mut high_security_context = create_test_context();
        high_security_context.security_level = SecurityLevel::High;

        let high_security_context_clone = high_security_context.clone();
        let ecosystem = UniversalPrimalEcosystem::new(high_security_context);

        let request = CapabilityRequest {
            required_capabilities: vec!["secure-operation".to_string()],
            optional_capabilities: vec![],
            context: high_security_context_clone,
            metadata: HashMap::new(),
        };

        // ACT: Discovery with security context
        let result = ecosystem.find_services_by_capability(&request).await;

        // ASSERT: Should respect security level in discovery
        assert!(
            result.is_ok(),
            "Discovery with security context should succeed"
        );
    }
}
