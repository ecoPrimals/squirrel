//! Capability Discovery Error Path Tests
//!
//! These tests ensure robust error handling in capability-based service discovery.

use squirrel::ecosystem::discovery_client::{
    capabilities, EcosystemServiceDiscovery, ServiceDiscovery,
};
use squirrel::error::PrimalError;

#[cfg(test)]
mod capability_discovery_error_tests {
    use super::*;

    #[tokio::test]
    async fn test_discovery_with_invalid_capability() {
        let discovery = EcosystemServiceDiscovery::new();

        let result = discovery
            .discover_by_capability("nonexistent_capability_xyz_123")
            .await;

        assert!(result.is_err(), "Should fail for invalid capability");
        match result {
            Err(PrimalError::ServiceDiscoveryFailed(msg)) => {
                assert!(
                    msg.contains("No service found"),
                    "Error should mention no service found"
                );
            }
            _ => panic!("Expected ServiceDiscoveryFailed error"),
        }
    }

    #[tokio::test]
    async fn test_discovery_with_empty_capability() {
        let discovery = EcosystemServiceDiscovery::new();

        let result = discovery.discover_by_capability("").await;

        assert!(result.is_err(), "Should fail for empty capability");
    }

    #[tokio::test]
    async fn test_discovery_without_environment_vars() {
        // Clear environment variables that might interfere
        std::env::remove_var("SECURITY_SERVICE_ENDPOINT");
        std::env::remove_var("BEARDOG_URL");
        std::env::remove_var("SONGBIRD_ENDPOINT");
        std::env::remove_var("SERVICE_DISCOVERY_DOMAIN");

        let discovery = EcosystemServiceDiscovery::new();

        // Should gracefully handle missing configuration
        let result = discovery
            .discover_by_capability(capabilities::SECURITY)
            .await;

        // In development mode, might succeed with local fallback
        // In production mode, should fail with clear error
        if result.is_err() {
            match result {
                Err(PrimalError::ServiceDiscoveryFailed(_)) => {
                    // Expected in production mode
                }
                Err(e) => panic!("Unexpected error type: {:?}", e),
                _ => {}
            }
        }
    }

    #[tokio::test]
    async fn test_is_capability_available_for_invalid() {
        let discovery = EcosystemServiceDiscovery::new();

        let available = discovery.is_capability_available("invalid_cap_xyz").await;

        assert!(!available, "Invalid capability should not be available");
    }

    #[tokio::test]
    async fn test_get_service_by_nonexistent_id() {
        let discovery = EcosystemServiceDiscovery::new();

        let result = discovery
            .get_service_by_id("nonexistent-service-id-xyz")
            .await;

        assert!(result.is_err(), "Should fail for nonexistent service ID");
        match result {
            Err(PrimalError::ServiceDiscoveryFailed(msg)) => {
                assert!(
                    msg.contains("not found"),
                    "Error should mention service not found"
                );
            }
            _ => panic!("Expected ServiceDiscoveryFailed error"),
        }
    }

    #[tokio::test]
    async fn test_discover_all_with_invalid_capability() {
        let discovery = EcosystemServiceDiscovery::new();

        let result = discovery.discover_all_by_capability("invalid_xyz").await;

        assert!(result.is_err(), "Should fail for invalid capability");
    }

    #[tokio::test]
    async fn test_cache_invalidation() {
        let discovery = EcosystemServiceDiscovery::new();

        // Refresh should succeed even with empty cache
        let result = discovery.refresh().await;
        assert!(result.is_ok(), "Cache refresh should succeed");
    }

    #[tokio::test]
    async fn test_concurrent_discovery_requests() {
        let discovery = std::sync::Arc::new(EcosystemServiceDiscovery::new());

        // Spawn multiple concurrent discovery requests
        let mut handles = vec![];
        for i in 0..10 {
            let disc = discovery.clone();
            let handle = tokio::spawn(async move {
                disc.is_capability_available(&format!("test_cap_{}", i))
                    .await
            });
            handles.push(handle);
        }

        // All should complete without panicking
        for handle in handles {
            let result = handle.await;
            assert!(result.is_ok(), "Concurrent discovery should not panic");
        }
    }

    #[tokio::test]
    async fn test_discovery_with_malformed_dns_domain() {
        std::env::set_var("SERVICE_DISCOVERY_DOMAIN", "invalid..domain..test");

        let discovery = EcosystemServiceDiscovery::new();
        let result = discovery
            .discover_by_capability(capabilities::SECURITY)
            .await;

        // Should handle malformed DNS gracefully
        // Either succeed with fallback or fail with clear error
        if result.is_err() {
            match result {
                Err(PrimalError::ServiceDiscoveryFailed(_)) => {
                    // Expected
                }
                Err(e) => panic!("Unexpected error type: {:?}", e),
                _ => {}
            }
        }

        std::env::remove_var("SERVICE_DISCOVERY_DOMAIN");
    }
}
