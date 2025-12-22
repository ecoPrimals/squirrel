//! Tests for health API handlers

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::api::server::ServerState;
    use crate::ecosystem::EcosystemConfig;
    use crate::ecosystem::EcosystemManager;
    use crate::monitoring::metrics::MetricsCollector;
    use chrono::Utc;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    fn create_test_state() -> Arc<RwLock<ServerState>> {
        Arc::new(RwLock::new(ServerState {
            started_at: Utc::now(),
            request_count: 0,
            active_connections: 0,
            service_mesh_registered: false,
            last_songbird_heartbeat: None,
        }))
    }

    fn create_test_ecosystem_manager() -> Arc<EcosystemManager> {
        let config = EcosystemConfig::default();
        let metrics = Arc::new(MetricsCollector::new());
        Arc::new(EcosystemManager::new(config, metrics))
    }

    #[tokio::test]
    async fn test_handle_health_check_basic() {
        let state = create_test_state();
        let manager = create_test_ecosystem_manager();

        let result = handle_health_check(state, manager).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_health_check_not_registered() {
        let state = create_test_state();
        let manager = create_test_ecosystem_manager();

        // State shows not registered
        {
            let state_guard = state.read().await;
            assert!(!state_guard.service_mesh_registered);
        }

        let result = handle_health_check(state, manager).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_health_check_registered() {
        let state = Arc::new(RwLock::new(ServerState {
            started_at: Utc::now(),
            request_count: 0,
            active_connections: 0,
            service_mesh_registered: true,
            last_songbird_heartbeat: Some(Utc::now()),
        }));
        let manager = create_test_ecosystem_manager();

        let result = handle_health_check(state, manager).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_health_live() {
        let state = create_test_state();

        let result = handle_health_live(state).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_health_live_always_succeeds() {
        // Liveness should always succeed if the handler runs
        let state = create_test_state();

        for _ in 0..5 {
            let result = handle_health_live(Arc::clone(&state)).await;
            assert!(result.is_ok());
        }
    }

    #[tokio::test]
    async fn test_handle_health_ready_no_services() {
        let state = create_test_state();
        let manager = create_test_ecosystem_manager();

        let result = handle_health_ready(state, manager).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_health_ready_not_registered() {
        let state = Arc::new(RwLock::new(ServerState {
            started_at: Utc::now(),
            request_count: 0,
            active_connections: 0,
            service_mesh_registered: false,
            last_songbird_heartbeat: None,
        }));
        let manager = create_test_ecosystem_manager();

        let result = handle_health_ready(state, manager).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_health_ready_registered() {
        let state = Arc::new(RwLock::new(ServerState {
            started_at: Utc::now(),
            request_count: 0,
            active_connections: 0,
            service_mesh_registered: true,
            last_songbird_heartbeat: Some(Utc::now()),
        }));
        let manager = create_test_ecosystem_manager();

        let result = handle_health_ready(state, manager).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_concurrent_health_checks() {
        let state = create_test_state();
        let manager = create_test_ecosystem_manager();

        let mut handles = vec![];

        for _ in 0..10 {
            let state_clone = Arc::clone(&state);
            let manager_clone = Arc::clone(&manager);

            let handle = tokio::spawn(async move {
                let result = handle_health_check(state_clone, manager_clone).await;
                assert!(result.is_ok());
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.await.unwrap();
        }
    }

    #[tokio::test]
    async fn test_concurrent_liveness_checks() {
        let state = create_test_state();

        let mut handles = vec![];

        for _ in 0..10 {
            let state_clone = Arc::clone(&state);

            let handle = tokio::spawn(async move {
                let result = handle_health_live(state_clone).await;
                assert!(result.is_ok());
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.await.unwrap();
        }
    }

    #[tokio::test]
    async fn test_concurrent_readiness_checks() {
        let state = create_test_state();
        let manager = create_test_ecosystem_manager();

        let mut handles = vec![];

        for _ in 0..10 {
            let state_clone = Arc::clone(&state);
            let manager_clone = Arc::clone(&manager);

            let handle = tokio::spawn(async move {
                let result = handle_health_ready(state_clone, manager_clone).await;
                assert!(result.is_ok());
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.await.unwrap();
        }
    }

    #[tokio::test]
    async fn test_health_check_with_old_started_time() {
        use chrono::Duration;

        let old_time = Utc::now() - Duration::hours(1);
        let state = Arc::new(RwLock::new(ServerState {
            started_at: old_time,
            request_count: 0,
            active_connections: 0,
            service_mesh_registered: true,
            last_songbird_heartbeat: Some(Utc::now()),
        }));
        let manager = create_test_ecosystem_manager();

        let result = handle_health_check(state, manager).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_all_handlers_are_async() {
        // Verify all handlers work with async/await
        let state = create_test_state();
        let manager = create_test_ecosystem_manager();

        let health = handle_health_check(Arc::clone(&state), Arc::clone(&manager)).await;
        let live = handle_health_live(Arc::clone(&state)).await;
        let ready = handle_health_ready(state, manager).await;

        assert!(health.is_ok());
        assert!(live.is_ok());
        assert!(ready.is_ok());
    }
}
