//! Additional edge case tests for metrics endpoints
//!
//! Deep testing: boundary conditions, concurrent access, error scenarios

#[cfg(test)]
mod edge_case_tests {
    use super::super::*;
    use crate::api::server::ServerState;
    use crate::monitoring::metrics::MetricsCollector;
    use chrono::{Duration, Utc};
    use std::sync::Arc;
    use tokio::sync::RwLock;

    // ===== Boundary Value Tests =====

    #[tokio::test]
    async fn test_metrics_with_max_request_count() {
        // Test: Maximum possible request count
        let state = Arc::new(RwLock::new(ServerState {
            started_at: Utc::now(),
            request_count: u64::MAX,
            active_connections: 0,
            service_mesh_registered: false,
            last_service_mesh_heartbeat: None,
        }));
        let metrics = Arc::new(MetricsCollector::new());

        let result = handle_metrics(state, metrics).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_metrics_with_zero_values() {
        // Test: All metrics at zero (startup scenario)
        let state = Arc::new(RwLock::new(ServerState {
            started_at: Utc::now(),
            request_count: 0,
            active_connections: 0,
            service_mesh_registered: false,
            last_service_mesh_heartbeat: None,
        }));
        let metrics = Arc::new(MetricsCollector::new());

        let result = handle_metrics(state.clone(), metrics).await;
        assert!(result.is_ok());

        // Verify state unchanged
        let state_guard = state.read().await;
        assert_eq!(state_guard.request_count, 0);
        assert_eq!(state_guard.active_connections, 0);
    }

    #[tokio::test]
    async fn test_metrics_with_max_active_connections() {
        // Test: High number of active connections
        let state = Arc::new(RwLock::new(ServerState {
            started_at: Utc::now(),
            request_count: 1000,
            active_connections: 10_000,
            service_mesh_registered: true,
            last_service_mesh_heartbeat: Some(Utc::now()),
        }));
        let metrics = Arc::new(MetricsCollector::new());

        let result = handle_metrics(state, metrics).await;
        assert!(result.is_ok());
    }

    // ===== Time-Based Edge Cases =====

    #[tokio::test]
    async fn test_metrics_with_very_long_uptime() {
        // Test: System running for months
        let old_start = Utc::now() - Duration::days(90);
        let state = Arc::new(RwLock::new(ServerState {
            started_at: old_start,
            request_count: 100_000_000,
            active_connections: 500,
            service_mesh_registered: true,
            last_service_mesh_heartbeat: Some(Utc::now()),
        }));
        let metrics = Arc::new(MetricsCollector::new());

        let result = handle_metrics(state, metrics).await;
        assert!(result.is_ok());
        // Uptime should be approximately 7_776_000 seconds (90 days)
    }

    #[tokio::test]
    async fn test_metrics_with_negative_uptime_protection() {
        // Test: Edge case - started_at in future (clock skew)
        let future_start = Utc::now() + Duration::minutes(10);
        let state = Arc::new(RwLock::new(ServerState {
            started_at: future_start,
            request_count: 0,
            active_connections: 0,
            service_mesh_registered: false,
            last_service_mesh_heartbeat: None,
        }));
        let metrics = Arc::new(MetricsCollector::new());

        let result = handle_metrics(state, metrics).await;
        // Should handle gracefully (negative uptime becomes 0 or is handled)
        assert!(result.is_ok());
    }

    // ===== Concurrent Access Tests =====

    #[tokio::test]
    async fn test_metrics_heavy_concurrent_load() {
        // Test: 200 concurrent metrics requests
        let state = Arc::new(RwLock::new(ServerState {
            started_at: Utc::now(),
            request_count: 5000,
            active_connections: 50,
            service_mesh_registered: true,
            last_service_mesh_heartbeat: Some(Utc::now()),
        }));
        let metrics = Arc::new(MetricsCollector::new());

        let mut handles = vec![];
        for _ in 0..200 {
            let state_clone = state.clone();
            let metrics_clone = metrics.clone();
            handles.push(tokio::spawn(async move {
                handle_metrics(state_clone, metrics_clone).await
            }));
        }

        // All should succeed
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok());
        }
    }

    #[tokio::test]
    async fn test_metrics_interleaved_reads() {
        // Test: Metrics reads interleaved with no writes
        let initial_count = 12345;
        let state = Arc::new(RwLock::new(ServerState {
            started_at: Utc::now(),
            request_count: initial_count,
            active_connections: 10,
            service_mesh_registered: true,
            last_service_mesh_heartbeat: Some(Utc::now()),
        }));
        let metrics = Arc::new(MetricsCollector::new());

        // Multiple sequential reads
        for _ in 0..10 {
            let result = handle_metrics(state.clone(), metrics.clone()).await;
            assert!(result.is_ok());
        }

        // Verify state unchanged
        let state_guard = state.read().await;
        assert_eq!(state_guard.request_count, initial_count);
    }

    // ===== State Immutability Tests =====

    #[tokio::test]
    async fn test_metrics_handler_is_read_only() {
        // Test: Metrics handler never modifies state
        let initial_data = (99999, 777, true, Some(Utc::now()));
        let state = Arc::new(RwLock::new(ServerState {
            started_at: Utc::now(),
            request_count: initial_data.0,
            active_connections: initial_data.1,
            service_mesh_registered: initial_data.2,
            last_service_mesh_heartbeat: initial_data.3,
        }));
        let metrics = Arc::new(MetricsCollector::new());

        // Call handler 100 times
        for _ in 0..100 {
            let _ = handle_metrics(state.clone(), metrics.clone()).await;
        }

        // Verify exact state preservation
        let state_guard = state.read().await;
        assert_eq!(state_guard.request_count, initial_data.0);
        assert_eq!(state_guard.active_connections, initial_data.1);
        assert_eq!(state_guard.service_mesh_registered, initial_data.2);
        assert_eq!(state_guard.last_service_mesh_heartbeat, initial_data.3);
    }

    // ===== Performance Characteristics =====

    #[tokio::test]
    async fn test_metrics_response_time_consistency() {
        // Test: Metrics handler has consistent response time
        let state = Arc::new(RwLock::new(ServerState {
            started_at: Utc::now(),
            request_count: 1000,
            active_connections: 10,
            service_mesh_registered: false,
            last_service_mesh_heartbeat: None,
        }));
        let metrics = Arc::new(MetricsCollector::new());

        // All calls should complete quickly (no blocking operations)
        for _ in 0..50 {
            let start = std::time::Instant::now();
            let result = handle_metrics(state.clone(), metrics.clone()).await;
            let duration = start.elapsed();

            assert!(result.is_ok());
            // Should complete in microseconds, not milliseconds
            assert!(duration.as_millis() < 10, "Metrics handler too slow");
        }
    }

    // ===== Collector Integration Tests =====

    #[tokio::test]
    async fn test_metrics_with_different_collectors() {
        // Test: Metrics handler works with any collector
        let state = Arc::new(RwLock::new(ServerState {
            started_at: Utc::now(),
            request_count: 100,
            active_connections: 5,
            service_mesh_registered: false,
            last_service_mesh_heartbeat: None,
        }));

        // Test with multiple independent collectors
        let collector1 = Arc::new(MetricsCollector::new());
        let collector2 = Arc::new(MetricsCollector::new());
        let collector3 = Arc::new(MetricsCollector::new());

        let r1 = handle_metrics(state.clone(), collector1).await;
        let r2 = handle_metrics(state.clone(), collector2).await;
        let r3 = handle_metrics(state, collector3).await;

        assert!(r1.is_ok());
        assert!(r2.is_ok());
        assert!(r3.is_ok());
    }

    // ===== Stress Test =====

    #[tokio::test]
    async fn test_metrics_sustained_load() {
        // Test: Sustained load over time
        let state = Arc::new(RwLock::new(ServerState {
            started_at: Utc::now(),
            request_count: 0,
            active_connections: 0,
            service_mesh_registered: false,
            last_service_mesh_heartbeat: None,
        }));
        let metrics = Arc::new(MetricsCollector::new());

        // Simulate sustained load (1000 requests)
        for batch in 0..10 {
            let mut handles = vec![];
            for _ in 0..100 {
                let state_clone = state.clone();
                let metrics_clone = metrics.clone();
                handles.push(tokio::spawn(async move {
                    handle_metrics(state_clone, metrics_clone).await
                }));
            }

            // All requests in batch should succeed
            for handle in handles {
                let result = handle.await.unwrap();
                assert!(result.is_ok(), "Failed in batch {}", batch);
            }
        }
    }
}

