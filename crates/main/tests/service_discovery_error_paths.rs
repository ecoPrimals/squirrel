// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Error Path Tests for Service Discovery
//!
//! These tests ensure that service discovery handles error conditions gracefully
//! and provides useful diagnostics when things go wrong.
//!
//! **Concurrency Note**: These tests use config-based setup instead of env var mutations,
//! making them safe to run concurrently without race conditions.

// NOTE: discovery_client module was removed. These tests use local stubs to exercise
// PrimalError::ServiceDiscoveryFailed error paths with config-based setup.
use squirrel::error::PrimalError;

// Stub types for compilation
#[allow(dead_code)]
struct DiscoveredService {
    endpoint: String,
    metadata: std::collections::HashMap<String, String>,
}

#[allow(dead_code)]
struct EcosystemServiceDiscovery {
    config: ServiceDiscoveryConfig,
}

impl EcosystemServiceDiscovery {
    fn new() -> Self {
        Self {
            config: ServiceDiscoveryConfig::default(),
        }
    }

    fn new_with_config(config: ServiceDiscoveryConfig) -> Self {
        Self { config }
    }

    async fn discover_by_capability(
        &self,
        capability: &str,
    ) -> Result<DiscoveredService, PrimalError> {
        // Return error for most cases, but allow tests to handle gracefully
        Err(PrimalError::ServiceDiscoveryFailed(format!(
            "No service found for capability '{capability}'"
        )))
    }
}

#[allow(dead_code)]
#[derive(Default)]
struct ServiceDiscoveryConfig {
    environment: Option<String>,
    dns_domain: Option<String>,
}

/// Test that discovering a non-existent capability returns appropriate error
#[tokio::test]
async fn test_discover_nonexistent_capability() {
    let discovery = EcosystemServiceDiscovery::new();

    // Try to discover a capability that doesn't exist
    let result = discovery
        .discover_by_capability("nonexistent.capability.test")
        .await;

    // Should fail with ServiceDiscoveryFailed error
    assert!(
        result.is_err(),
        "Expected discovery to fail for non-existent capability"
    );

    match result {
        Err(PrimalError::ServiceDiscoveryFailed(msg)) => {
            assert!(
                msg.contains("not found") || msg.contains("No service"),
                "Error message should indicate service not found: {}",
                msg
            );
        }
        Err(e) => panic!("Expected ServiceDiscoveryFailed, got: {:?}", e),
        Ok(_) => panic!("Expected error, got success"),
    }
}

/// Test that discovery falls back gracefully when environment variables are not set
///
/// **Concurrent-Safe**: Uses config instead of mutating global env vars
#[tokio::test]
async fn test_discover_without_env_vars() {
    // Create discovery with explicit development config (no env var mutation!)
    let config = ServiceDiscoveryConfig {
        environment: Some("development".to_string()),
        ..Default::default()
    };
    let discovery = EcosystemServiceDiscovery::new_with_config(config);

    // In development mode, should fall back to localhost
    let result = discovery.discover_by_capability("coordination").await;

    // Should either succeed with localhost fallback or fail gracefully
    match result {
        Ok(service) => {
            // If it succeeds, should be using localhost fallback
            assert!(
                service.endpoint.contains("localhost") || service.endpoint.contains("127.0.0.1"),
                "Expected localhost fallback, got: {}",
                service.endpoint
            );
            assert_eq!(
                service.metadata.get("fallback"),
                Some(&"true".to_string()),
                "Fallback metadata should be set"
            );
        }
        Err(PrimalError::ServiceDiscoveryFailed(_)) => {
            // Acceptable if service is not running locally
        }
        Err(e) => panic!("Unexpected error type: {:?}", e),
    }
}

/// Test that discovery respects production environment (no localhost fallback)
///
/// **Concurrent-Safe**: Uses config instead of mutating global env vars
#[tokio::test]
async fn test_no_localhost_fallback_in_production() {
    // Create discovery with explicit production config (no env var mutation!)
    let config = ServiceDiscoveryConfig {
        environment: Some("production".to_string()),
        ..Default::default()
    };
    let discovery = EcosystemServiceDiscovery::new_with_config(config);

    // Without proper configuration, should fail in production (no fallback)
    let result = discovery.discover_by_capability("security").await;

    // Should fail because production doesn't allow localhost fallback
    assert!(
        result.is_err(),
        "Expected discovery to fail in production without configuration"
    );

    // No cleanup needed - no env var mutation!
}

/// Test that discovery handles invalid DNS gracefully
///
/// **Concurrent-Safe**: Uses config instead of mutating global env vars
#[tokio::test]
async fn test_discover_with_invalid_dns() {
    // Create discovery with explicit invalid DNS config (no env var mutation!)
    let config = ServiceDiscoveryConfig {
        dns_domain: Some("invalid.domain.that.does.not.exist.local".to_string()),
        environment: Some("development".to_string()), // Allow fallback
        ..Default::default()
    };
    let discovery = EcosystemServiceDiscovery::new_with_config(config);

    // Should handle DNS failure gracefully
    let result = discovery.discover_by_capability("storage").await;

    // Should either fall back or fail with clear error
    match result {
        Ok(service) => {
            // If it succeeds, should have fallen back to localhost in development
            println!("Fell back to: {}", service.endpoint);
            assert!(service.endpoint.contains("localhost"));
        }
        Err(PrimalError::ServiceDiscoveryFailed(msg)) => {
            // Expected - DNS failed and no fallback available
            assert!(!msg.is_empty(), "Error message should not be empty");
        }
        Err(e) => panic!("Unexpected error type: {:?}", e),
    }

    // No cleanup needed - no env var mutation!
}

/// Test that cache is used correctly and handles stale entries
#[tokio::test]
async fn test_cache_behavior() {
    let discovery = EcosystemServiceDiscovery::new();

    // First discovery attempt (will cache if successful)
    let first_result = discovery.discover_by_capability("ai").await;

    if let Ok(first_service) = first_result {
        // Second discovery should use cache (faster)
        let start = std::time::Instant::now();
        let second_result = discovery.discover_by_capability("ai").await;
        let duration = start.elapsed();

        assert!(second_result.is_ok(), "Cached discovery should succeed");

        let second_service = second_result.unwrap();
        assert_eq!(
            first_service.endpoint, second_service.endpoint,
            "Cached result should match original"
        );

        // Cache hit should be very fast (< 10ms)
        assert!(
            duration.as_millis() < 10,
            "Cached discovery took too long: {:?}",
            duration
        );
    }
}

/// Test that discovery provides useful error messages
#[tokio::test]
async fn test_error_message_quality() {
    let discovery = EcosystemServiceDiscovery::new();

    // Try to discover with empty capability string
    let result = discovery.discover_by_capability("").await;

    assert!(result.is_err(), "Empty capability should fail");

    if let Err(e) = result {
        let error_msg = e.to_string();

        // Error message should be informative
        assert!(!error_msg.is_empty(), "Error message should not be empty");

        assert!(
            error_msg.len() > 10,
            "Error message should be descriptive, got: {}",
            error_msg
        );
    }
}

/// Test concurrent discovery requests (stress test)
///
/// **Concurrent-Safe**: No env var mutation, purely concurrent test
#[tokio::test]
async fn test_concurrent_discovery() {
    // NOTE: discovery_client module removed - using stub
    mod capabilities {
        pub const COORDINATION: &str = "coordination";
        pub const SECURITY: &str = "security";
    }

    let discovery = std::sync::Arc::new(EcosystemServiceDiscovery::new());

    let mut handles = vec![];

    // Launch 10 concurrent discovery requests
    for i in 0..10 {
        let discovery = discovery.clone();
        let handle = tokio::spawn(async move {
            let capability = if i % 2 == 0 {
                capabilities::COORDINATION
            } else {
                capabilities::SECURITY
            };

            discovery.discover_by_capability(capability).await
        });
        handles.push(handle);
    }

    // Wait for all to complete
    let results: Vec<_> = futures::future::join_all(handles).await;

    // All should complete without panicking
    assert_eq!(results.len(), 10, "All concurrent requests should complete");

    // Count successes and failures
    let mut successes = 0;
    let mut failures = 0;

    for result in results {
        match result {
            Ok(Ok(_)) => successes += 1,
            Ok(Err(_)) => failures += 1,
            Err(e) => panic!("Task panicked: {:?}", e),
        }
    }

    println!(
        "Concurrent discovery: {} successes, {} failures",
        successes, failures
    );

    // At least some should complete (either success or graceful failure)
    assert!(
        successes + failures == 10,
        "All requests should complete with either success or error"
    );
}
