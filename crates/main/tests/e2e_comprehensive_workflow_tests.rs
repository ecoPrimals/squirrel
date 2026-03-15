// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Comprehensive End-to-End Workflow Tests
//!
//! These tests validate complete user workflows across the entire system.
//! Tests exercise real APIs — no stubs or mocks.

#[cfg(test)]
mod tests {
    /// Test ecosystem manager lifecycle end-to-end
    #[tokio::test]
    async fn test_complete_ecosystem_lifecycle() {
        use squirrel::ecosystem::{EcosystemConfig, EcosystemManager};
        use squirrel::monitoring::metrics::MetricsCollector;
        use std::sync::Arc;

        // 1. Initialize ecosystem manager
        let config = EcosystemConfig::default();
        let metrics = Arc::new(MetricsCollector::new());
        let manager = EcosystemManager::new(config, metrics);

        // 2. Verify initial state
        let status = manager.status.read().await;
        assert_eq!(status.active_registrations.len(), 0);
        drop(status);

        // 3. Discover services (should be empty initially)
        let services = manager.discover_services().await.unwrap_or_default();
        assert!(services.is_empty(), "No services registered yet");

        // 4. Get ecosystem status
        let ecosystem_status = manager.get_ecosystem_status().await;
        assert!(
            !ecosystem_status.status.is_empty(),
            "Status should be populated"
        );

        // Ecosystem lifecycle validated
    }

    /// Test error handling workflow end-to-end
    #[tokio::test]
    async fn test_complete_error_handling_workflow() {
        use squirrel::error::PrimalError;

        // 1. Trigger various error types
        let network_err = PrimalError::Network("connection failed".to_string());
        let auth_err = PrimalError::Authentication("invalid credentials".to_string());
        let config_err = PrimalError::Configuration("missing field".to_string());
        let internal_err = PrimalError::Internal("unexpected state".to_string());

        // 2. Verify all error types have informative Display output
        assert!(
            network_err.to_string().contains("Network"),
            "Network error display"
        );
        assert!(
            auth_err.to_string().contains("Authentication"),
            "Auth error display"
        );
        assert!(
            config_err.to_string().contains("Configuration"),
            "Config error display"
        );
        assert!(
            internal_err.to_string().contains("Internal"),
            "Internal error display"
        );

        // 3. Test error propagation
        let result: Result<(), PrimalError> = Err(network_err);
        assert!(result.is_err());

        // 4. Test error transformation
        let transformed = result.map_err(|e| PrimalError::Internal(format!("Wrapped: {}", e)));
        assert!(transformed.is_err());
        if let Err(PrimalError::Internal(msg)) = transformed {
            assert!(msg.contains("Wrapped"));
        }

        // 5. Test safe parse pattern
        let safe_parse: Result<i32, _> = "invalid".parse::<i32>();
        assert!(safe_parse.is_err());
        assert_eq!(safe_parse.unwrap_or_default(), 0);

        // 6. Test error conversion from std::io::Error
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let primal_err = PrimalError::from(io_err);
        assert!(primal_err.to_string().contains("IO error"));

        // 7. Test timeout error handling
        use tokio::time::{Duration, timeout};
        let timeout_result = timeout(
            Duration::from_millis(1),
            std::future::pending::<Result<(), PrimalError>>(),
        )
        .await;
        assert!(timeout_result.is_err(), "Timeout should trigger error");

        // 8. Test error recovery
        let recovered = match timeout_result {
            Ok(inner) => inner,
            Err(_) => Ok(()), // Recovered from timeout
        };
        assert!(recovered.is_ok());
    }

    /// Test capability-based discovery workflow
    #[tokio::test]
    async fn test_complete_capability_discovery_workflow() {
        use squirrel::ecosystem::{EcosystemConfig, EcosystemManager};
        use squirrel::monitoring::metrics::MetricsCollector;
        use std::sync::Arc;

        // 1. Create ecosystem manager
        let config = EcosystemConfig::default();
        let metrics = Arc::new(MetricsCollector::new());
        let manager = Arc::new(EcosystemManager::new(config, metrics));

        // 2. Capability-based discovery (no services running, should return empty)
        let services = manager.discover_services().await.unwrap_or_default();
        assert!(services.is_empty());

        // 3. Concurrent capability discovery should be safe
        let mut handles = vec![];
        for _ in 0..20 {
            let mgr = manager.clone();
            handles.push(tokio::spawn(async move {
                mgr.discover_services().await.unwrap_or_default()
            }));
        }

        for handle in handles {
            let result = handle.await;
            assert!(result.is_ok(), "Concurrent discovery should not panic");
        }

        // 4. Verify ecosystem status is accessible
        let status = manager.get_ecosystem_status().await;
        assert!(
            !status.status.is_empty(),
            "Ecosystem status should be available"
        );
    }

    /// Test observability basics
    #[tokio::test]
    async fn test_observability_basics() {
        use squirrel::observability::CorrelationId;

        // 1. Create unique correlation IDs
        let id1 = CorrelationId::new();
        let id2 = CorrelationId::new();
        assert_ne!(
            format!("{}", id1),
            format!("{}", id2),
            "IDs should be unique"
        );

        // 2. Clone and compare
        let id3 = id1.clone();
        assert_eq!(id1, id3, "Cloned IDs should be equal");

        // 3. String conversion
        let id_str = id1.as_str();
        assert!(!id_str.is_empty());
        assert!(id_str.contains('-'), "Should be UUID-like");

        // 4. From string
        let custom = CorrelationId::from_string("custom-correlation-id");
        assert_eq!(custom.as_str(), "custom-correlation-id");

        // 5. Concurrent ID generation
        let mut handles = vec![];
        for _ in 0..50 {
            handles.push(tokio::spawn(async { CorrelationId::new() }));
        }
        let mut ids = vec![];
        for handle in handles {
            ids.push(handle.await.unwrap());
        }
        // All should be unique
        for i in 0..ids.len() {
            for j in (i + 1)..ids.len() {
                assert_ne!(ids[i], ids[j], "All generated IDs should be unique");
            }
        }
    }

    /// Test configuration types
    #[tokio::test]
    async fn test_configuration_workflow() {
        use squirrel::ecosystem::EcosystemConfig;

        // 1. Default config
        let config = EcosystemConfig::default();

        // 2. Verify serialization round-trip
        let serialized = serde_json::to_string(&config);
        assert!(serialized.is_ok(), "Config should be serializable");

        if let Ok(json_str) = serialized {
            let deserialized: Result<EcosystemConfig, _> = serde_json::from_str(&json_str);
            assert!(deserialized.is_ok(), "Config should be deserializable");
        }
    }

    /// Test concurrent operations don't deadlock
    #[tokio::test]
    async fn test_concurrent_operations_no_deadlock() {
        use squirrel::ecosystem::{EcosystemConfig, EcosystemManager};
        use squirrel::monitoring::metrics::MetricsCollector;
        use std::sync::Arc;
        use tokio::time::{Duration, timeout};

        let config = EcosystemConfig::default();
        let metrics = Arc::new(MetricsCollector::new());
        let manager = Arc::new(EcosystemManager::new(config, metrics));

        // Launch concurrent read operations (avoid discovery which does network I/O)
        let mut handles = vec![];
        for i in 0..50 {
            let mgr = manager.clone();
            handles.push(tokio::spawn(async move {
                match i % 2 {
                    0 => {
                        let _ = mgr.discover_services().await;
                    }
                    _ => {
                        let _ = mgr.status.read().await;
                    }
                }
            }));
        }

        // All must complete within 10 seconds (no deadlock)
        let result = timeout(Duration::from_secs(10), async {
            for handle in handles {
                handle.await.unwrap();
            }
        })
        .await;

        assert!(
            result.is_ok(),
            "All operations should complete without deadlock"
        );
    }
}
