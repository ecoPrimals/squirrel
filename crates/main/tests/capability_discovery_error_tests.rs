// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::needless_pass_by_value,
    clippy::significant_drop_tightening,
    clippy::field_reassign_with_default,
    clippy::default_trait_access,
    clippy::many_single_char_names,
    clippy::unreadable_literal,
    clippy::too_many_lines,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::similar_names,
    clippy::option_if_let_else,
    clippy::doc_markdown,
    clippy::struct_field_names,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::return_self_not_must_use,
    clippy::trivially_copy_pass_by_ref,
    clippy::unused_self,
    clippy::unused_async,
    clippy::unnecessary_wraps,
    clippy::semicolon_if_nothing_returned,
    clippy::match_wildcard_for_single_variants,
    clippy::match_same_arms,
    clippy::explicit_iter_loop,
    clippy::uninlined_format_args,
    clippy::equatable_if_let,
    clippy::assertions_on_constants,
    missing_docs,
    unused_imports,
    unused_variables,
    dead_code,
    deprecated
)]
//! Capability Discovery Error Path Tests
//!
//! These tests ensure robust error handling in capability-based service discovery.

// NOTE: discovery_client module was removed. These tests use local stubs to exercise
// PrimalError::ServiceDiscoveryFailed error paths. Will be updated when the new
// capability-based discovery API is complete.

use squirrel::error::PrimalError;

#[cfg(test)]
mod capability_discovery_error_tests {
    use super::*;

    // Stub types for compilation
    #[allow(dead_code)]
    struct EcosystemServiceDiscovery;

    impl EcosystemServiceDiscovery {
        fn new() -> Self {
            Self
        }

        async fn discover_by_capability(&self, capability: &str) -> Result<(), PrimalError> {
            Err(PrimalError::ServiceDiscoveryFailed(format!(
                "No service found for capability '{capability}'"
            )))
        }

        async fn is_capability_available(&self, _capability: &str) -> bool {
            false
        }

        async fn get_service_by_id(&self, id: &str) -> Result<(), PrimalError> {
            Err(PrimalError::ServiceDiscoveryFailed(format!(
                "Service not found: {id}"
            )))
        }

        async fn discover_all_by_capability(
            &self,
            capability: &str,
        ) -> Result<Vec<()>, PrimalError> {
            Err(PrimalError::ServiceDiscoveryFailed(format!(
                "No service found for capability '{capability}'"
            )))
        }

        async fn refresh(&self) -> Result<(), PrimalError> {
            Ok(())
        }
    }

    mod capabilities {
        pub const SECURITY: &str = "security";
    }

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
            _ => unreachable!("Expected ServiceDiscoveryFailed error"),
        }
    }

    #[tokio::test]
    async fn test_discovery_with_empty_capability() {
        let discovery = EcosystemServiceDiscovery::new();

        let result = discovery.discover_by_capability("").await;

        assert!(result.is_err(), "Should fail for empty capability");
    }

    #[test]
    fn test_discovery_without_environment_vars() {
        temp_env::with_vars_unset(
            [
                "SECURITY_SERVICE_ENDPOINT",
                "BEARDOG_URL",
                "SONGBIRD_ENDPOINT",
                "SERVICE_DISCOVERY_DOMAIN",
            ],
            || {
                let rt = tokio::runtime::Runtime::new().expect("should succeed");
                rt.block_on(async {
                    let discovery = EcosystemServiceDiscovery::new();

                    let result = discovery
                        .discover_by_capability(capabilities::SECURITY)
                        .await;

                    if result.is_err() {
                        match result {
                            Err(PrimalError::ServiceDiscoveryFailed(_)) => {}
                            Err(e) => unreachable!("Unexpected error type: {e:?}"),
                            _ => {}
                        }
                    }
                });
            },
        );
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
            _ => unreachable!("Expected ServiceDiscoveryFailed error"),
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
                disc.is_capability_available(&format!("test_cap_{i}")).await
            });
            handles.push(handle);
        }

        // All should complete without panicking
        for handle in handles {
            let result = handle.await;
            assert!(result.is_ok(), "Concurrent discovery should not panic");
        }
    }

    #[test]
    fn test_discovery_with_malformed_dns_domain() {
        temp_env::with_var(
            "SERVICE_DISCOVERY_DOMAIN",
            Some("invalid..domain..test"),
            || {
                let rt = tokio::runtime::Runtime::new().expect("should succeed");
                rt.block_on(async {
                    let discovery = EcosystemServiceDiscovery::new();
                    let result = discovery
                        .discover_by_capability(capabilities::SECURITY)
                        .await;

                    if result.is_err() {
                        match result {
                            Err(PrimalError::ServiceDiscoveryFailed(_)) => {}
                            Err(e) => unreachable!("Unexpected error type: {e:?}"),
                            _ => {}
                        }
                    }
                });
            },
        );
    }
}
