//! Tests for Songbird service mesh API handlers
//!
//! Provides comprehensive coverage for registration and heartbeat functionality.

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::api::server::ServerState;
    use crate::ecosystem::{EcosystemConfig, EcosystemManager};
    use crate::monitoring::metrics::MetricsCollector;
    use chrono::{Duration, Utc};
    use std::sync::Arc;
    use tokio::sync::RwLock;

    fn create_test_ecosystem_manager() -> Arc<EcosystemManager> {
        let config = EcosystemConfig::default();
        let metrics = Arc::new(MetricsCollector::new());
        Arc::new(EcosystemManager::new(config, metrics))
    }

    fn create_test_state() -> Arc<RwLock<ServerState>> {
        Arc::new(RwLock::new(ServerState {
            started_at: Utc::now(),
            request_count: 0,
            active_connections: 0,
            service_mesh_registered: false,
            last_service_mesh_heartbeat: None,
        }))
    }

    #[tokio::test]
    async fn test_handle_service_mesh_register_basic() {
        let manager = create_test_ecosystem_manager();
        let result = handle_service_mesh_register(manager).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_service_mesh_register_multiple_calls() {
        let manager = create_test_ecosystem_manager();

        // Multiple registration attempts should all succeed with pending status
        for _ in 0..5 {
            let result = handle_service_mesh_register(Arc::clone(&manager)).await;
            assert!(result.is_ok());
        }
    }

    #[tokio::test]
    async fn test_handle_service_mesh_heartbeat_basic() {
        let state = create_test_state();
        let result = handle_service_mesh_heartbeat(state).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_service_mesh_heartbeat_updates_state() {
        let state = create_test_state();

        // Initially not registered
        {
            let guard = state.read().await;
            assert!(!guard.service_mesh_registered);
            assert!(guard.last_service_mesh_heartbeat.is_none());
        }

        // Send heartbeat
        let result = handle_service_mesh_heartbeat(Arc::clone(&state)).await;
        assert!(result.is_ok());

        // Now should be registered with heartbeat timestamp
        {
            let guard = state.read().await;
            assert!(guard.service_mesh_registered);
            assert!(guard.last_service_mesh_heartbeat.is_some());
        }
    }

    #[tokio::test]
    async fn test_handle_service_mesh_heartbeat_updates_timestamp() {
        let state = create_test_state();

        // First heartbeat
        let first_time = {
            handle_service_mesh_heartbeat(Arc::clone(&state)).await.unwrap();
            let guard = state.read().await;
            guard.last_service_mesh_heartbeat.unwrap()
        };

        // Second heartbeat - timestamps should be monotonically increasing
        // Note: Chrono::Utc::now() has microsecond precision, so successive calls
        // will have different timestamps without needing to sleep
        let second_time = {
            handle_service_mesh_heartbeat(Arc::clone(&state)).await.unwrap();
            let guard = state.read().await;
            guard.last_service_mesh_heartbeat.unwrap()
        };

        // Second timestamp should be later than or equal to first
        // (>= instead of > to handle rare cases where clock resolution is low)
        assert!(second_time >= first_time);
    }

    #[tokio::test]
    async fn test_concurrent_songbird_heartbeats() {
        let state = create_test_state();
        let mut handles = vec![];

        // Send multiple concurrent heartbeats
        for _ in 0..20 {
            let state_clone = Arc::clone(&state);
            let handle = tokio::spawn(async move {
                let result = handle_service_mesh_heartbeat(state_clone).await;
                assert!(result.is_ok());
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.await.unwrap();
        }

        // Should be registered after all heartbeats
        let guard = state.read().await;
        assert!(guard.service_mesh_registered);
        assert!(guard.last_service_mesh_heartbeat.is_some());
    }

    #[tokio::test]
    async fn test_concurrent_songbird_registrations() {
        let manager = create_test_ecosystem_manager();
        let mut handles = vec![];

        for _ in 0..10 {
            let manager_clone = Arc::clone(&manager);
            let handle = tokio::spawn(async move {
                let result = handle_service_mesh_register(manager_clone).await;
                assert!(result.is_ok());
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.await.unwrap();
        }
    }

    #[tokio::test]
    async fn test_heartbeat_idempotency() {
        let state = create_test_state();

        // Multiple heartbeats should all succeed
        for _ in 0..10 {
            let result = handle_service_mesh_heartbeat(Arc::clone(&state)).await;
            assert!(result.is_ok());
        }

        // State should remain consistent
        let guard = state.read().await;
        assert!(guard.service_mesh_registered);
        assert!(guard.last_service_mesh_heartbeat.is_some());
    }

    #[tokio::test]
    async fn test_heartbeat_after_long_gap() {
        let old_time = Utc::now() - Duration::hours(24);
        let state = Arc::new(RwLock::new(ServerState {
            started_at: Utc::now(),
            request_count: 0,
            active_connections: 0,
            service_mesh_registered: true,
            last_service_mesh_heartbeat: Some(old_time),
        }));

        // Send new heartbeat after long gap
        let result = handle_service_mesh_heartbeat(Arc::clone(&state)).await;
        assert!(result.is_ok());

        // Timestamp should be updated
        let guard = state.read().await;
        assert!(guard.last_service_mesh_heartbeat.unwrap() > old_time);
    }

    #[tokio::test]
    async fn test_mixed_register_and_heartbeat_calls() {
        let manager = create_test_ecosystem_manager();
        let state = create_test_state();
        let mut handles = vec![];

        for i in 0..20 {
            if i % 2 == 0 {
                let manager_clone = Arc::clone(&manager);
                handles.push(tokio::spawn(async move {
                    handle_service_mesh_register(manager_clone).await.unwrap();
                }));
            } else {
                let state_clone = Arc::clone(&state);
                handles.push(tokio::spawn(async move {
                    handle_service_mesh_heartbeat(state_clone).await.unwrap();
                }));
            }
        }

        for handle in handles {
            handle.await.unwrap();
        }
    }

    #[tokio::test]
    async fn test_heartbeat_with_high_frequency() {
        let state = create_test_state();

        // Simulate high-frequency heartbeats (like a busy mesh)
        for _ in 0..100 {
            let result = handle_service_mesh_heartbeat(Arc::clone(&state)).await;
            assert!(result.is_ok());
        }

        let guard = state.read().await;
        assert!(guard.service_mesh_registered);
    }

    #[tokio::test]
    async fn test_registration_without_context() {
        // Test that registration gracefully handles missing context
        let manager = create_test_ecosystem_manager();
        let result = handle_service_mesh_register(manager).await;

        // Should succeed with pending status (documented limitation)
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_heartbeat_maintains_registration_flag() {
        let state = create_test_state();

        // Multiple heartbeats should keep registration flag set
        for _ in 0..5 {
            handle_service_mesh_heartbeat(Arc::clone(&state)).await.unwrap();
            let guard = state.read().await;
            assert!(guard.service_mesh_registered);
        }
    }
}
