//! Tests for metrics API handler

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::api::server::ServerState;
    use crate::monitoring::metrics::MetricsCollector;
    use chrono::Utc;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    fn create_test_state() -> Arc<RwLock<ServerState>> {
        Arc::new(RwLock::new(ServerState {
            started_at: Utc::now(),
            request_count: 100,
            active_connections: 5,
            service_mesh_registered: true,
            last_service_mesh_heartbeat: Some(Utc::now()),
        }))
    }

    fn create_test_metrics_collector() -> Arc<MetricsCollector> {
        Arc::new(MetricsCollector::new())
    }

    #[tokio::test]
    async fn test_handle_metrics_basic() {
        let state = create_test_state();
        let metrics = create_test_metrics_collector();

        let result = handle_metrics(state, metrics).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_metrics_with_uptime() {
        use chrono::Duration;

        let old_time = Utc::now() - Duration::hours(2);
        let state = Arc::new(RwLock::new(ServerState {
            started_at: old_time,
            request_count: 1000,
            active_connections: 10,
            service_mesh_registered: true,
            last_service_mesh_heartbeat: Some(Utc::now()),
        }));
        let metrics = create_test_metrics_collector();

        let result = handle_metrics(state, metrics).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_metrics_zero_requests() {
        let state = Arc::new(RwLock::new(ServerState {
            started_at: Utc::now(),
            request_count: 0,
            active_connections: 0,
            service_mesh_registered: false,
            last_service_mesh_heartbeat: None,
        }));
        let metrics = create_test_metrics_collector();

        let result = handle_metrics(state, metrics).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_concurrent_metrics_requests() {
        let state = create_test_state();
        let metrics = create_test_metrics_collector();

        let mut handles = vec![];

        for _ in 0..20 {
            let state_clone = Arc::clone(&state);
            let metrics_clone = Arc::clone(&metrics);

            let handle = tokio::spawn(async move {
                let result = handle_metrics(state_clone, metrics_clone).await;
                assert!(result.is_ok());
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.await.unwrap();
        }
    }

    #[tokio::test]
    async fn test_metrics_handler_always_succeeds() {
        let state = create_test_state();
        let metrics = create_test_metrics_collector();

        // Should always succeed as long as state is accessible
        for _ in 0..5 {
            let result = handle_metrics(Arc::clone(&state), Arc::clone(&metrics)).await;
            assert!(result.is_ok());
        }
    }

    #[tokio::test]
    async fn test_metrics_with_high_request_count() {
        let state = Arc::new(RwLock::new(ServerState {
            started_at: Utc::now(),
            request_count: 1_000_000,
            active_connections: 100,
            service_mesh_registered: true,
            last_service_mesh_heartbeat: Some(Utc::now()),
        }));
        let metrics = create_test_metrics_collector();

        let result = handle_metrics(state, metrics).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_metrics_state_read_lock() {
        let state = create_test_state();
        let metrics = create_test_metrics_collector();

        // Multiple concurrent reads should work fine
        let h1 = tokio::spawn(handle_metrics(Arc::clone(&state), Arc::clone(&metrics)));
        let h2 = tokio::spawn(handle_metrics(Arc::clone(&state), Arc::clone(&metrics)));
        let h3 = tokio::spawn(handle_metrics(state, metrics));

        assert!(h1.await.unwrap().is_ok());
        assert!(h2.await.unwrap().is_ok());
        assert!(h3.await.unwrap().is_ok());
    }
}
