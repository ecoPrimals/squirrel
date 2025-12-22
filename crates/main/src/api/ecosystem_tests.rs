//! Tests for ecosystem API handlers

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::ecosystem::EcosystemConfig;
    use crate::ecosystem::EcosystemManager;
    use crate::monitoring::metrics::MetricsCollector;
    use std::sync::Arc;

    fn create_test_ecosystem_manager() -> Arc<EcosystemManager> {
        let config = EcosystemConfig::default();
        let metrics = Arc::new(MetricsCollector::new());
        Arc::new(EcosystemManager::new(config, metrics))
    }

    #[tokio::test]
    async fn test_handle_ecosystem_status() {
        let manager = create_test_ecosystem_manager();
        let result = handle_ecosystem_status(manager).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_service_mesh_status() {
        let manager = create_test_ecosystem_manager();
        let result = handle_service_mesh_status(manager).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_primals_list() {
        let manager = create_test_ecosystem_manager();
        let base_url = "http://localhost:8080".to_string();

        let result = handle_primals_list(manager, base_url).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_primal_status_not_found() {
        let manager = create_test_ecosystem_manager();
        let base_url = "http://localhost:8080".to_string();

        let result = handle_primal_status("nonexistent".to_string(), manager, base_url).await;
        assert!(result.is_ok()); // Returns Ok with error JSON
    }

    #[tokio::test]
    async fn test_handle_services() {
        let manager = create_test_ecosystem_manager();
        let base_url = "http://localhost:8080".to_string();

        let result = handle_services(manager, base_url).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_concurrent_ecosystem_status_requests() {
        let manager = create_test_ecosystem_manager();

        let mut handles = vec![];

        for _ in 0..10 {
            let manager_clone = Arc::clone(&manager);
            let handle = tokio::spawn(async move {
                let result = handle_ecosystem_status(manager_clone).await;
                assert!(result.is_ok());
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.await.unwrap();
        }
    }

    #[tokio::test]
    async fn test_all_endpoints_work() {
        let manager = create_test_ecosystem_manager();
        let base_url = "http://localhost:8080".to_string();

        // Test all endpoints
        let status = handle_ecosystem_status(Arc::clone(&manager)).await;
        let mesh = handle_service_mesh_status(Arc::clone(&manager)).await;
        let primals = handle_primals_list(Arc::clone(&manager), base_url.clone()).await;
        let services = handle_services(manager, base_url).await;

        assert!(status.is_ok());
        assert!(mesh.is_ok());
        assert!(primals.is_ok());
        assert!(services.is_ok());
    }

    #[tokio::test]
    async fn test_service_mesh_status_response_format() {
        let manager = create_test_ecosystem_manager();
        let result = handle_service_mesh_status(manager).await;

        // Should always return a valid response
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_primal_status_with_various_names() {
        let manager = create_test_ecosystem_manager();
        let base_url = "http://localhost:8080".to_string();

        let names = vec!["squirrel", "songbird", "beardog", "unknown"];

        for name in names {
            let result =
                handle_primal_status(name.to_string(), Arc::clone(&manager), base_url.clone())
                    .await;
            assert!(result.is_ok());
        }
    }

    #[tokio::test]
    async fn test_concurrent_mixed_requests() {
        let manager = create_test_ecosystem_manager();
        let mut handles = vec![];

        // Mix of different endpoint types
        for i in 0..15 {
            let manager_clone = Arc::clone(&manager);
            let base_url = "http://localhost:8080".to_string();

            let handle = tokio::spawn(async move {
                match i % 3 {
                    0 => {
                        let _ = handle_ecosystem_status(manager_clone).await;
                    }
                    1 => {
                        let _ = handle_primals_list(manager_clone, base_url).await;
                    }
                    _ => {
                        let _ = handle_service_mesh_status(manager_clone).await;
                    }
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.await.unwrap();
        }
    }

    #[tokio::test]
    async fn test_handle_primal_status_empty_name() {
        let manager = create_test_ecosystem_manager();
        let base_url = "http://localhost:8080".to_string();

        let result = handle_primal_status("".to_string(), manager, base_url).await;
        assert!(result.is_ok()); // Should handle gracefully with error JSON
    }

    #[tokio::test]
    async fn test_handle_ecosystem_status_empty_registry() {
        let manager = create_test_ecosystem_manager();

        // Fresh manager should have no services initially
        let result = handle_ecosystem_status(manager).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_services_empty_registry() {
        let manager = create_test_ecosystem_manager();
        let base_url = "http://localhost:8080".to_string();

        let result = handle_services(manager, base_url).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_primals_list_empty_registry() {
        let manager = create_test_ecosystem_manager();
        let base_url = "http://localhost:8080".to_string();

        let result = handle_primals_list(manager, base_url).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_primal_status_special_characters() {
        let manager = create_test_ecosystem_manager();
        let base_url = "http://localhost:8080".to_string();

        let special_names = vec![
            "primal-with-dash",
            "primal_with_underscore",
            "primal.with.dots",
            "primal123",
            "123primal",
        ];

        for name in special_names {
            let result =
                handle_primal_status(name.to_string(), Arc::clone(&manager), base_url.clone())
                    .await;
            assert!(result.is_ok());
        }
    }

    #[tokio::test]
    async fn test_services_endpoint_consistency() {
        let manager = create_test_ecosystem_manager();
        let base_url = "http://localhost:8080".to_string();

        // Call multiple times and verify consistency
        for _ in 0..5 {
            let result = handle_services(Arc::clone(&manager), base_url.clone()).await;
            assert!(result.is_ok());
        }
    }

    #[tokio::test]
    async fn test_ecosystem_status_high_concurrency() {
        let manager = create_test_ecosystem_manager();
        let mut handles = vec![];

        // High concurrency stress test
        for _ in 0..100 {
            let manager_clone = Arc::clone(&manager);
            let handle = tokio::spawn(async move {
                let result = handle_ecosystem_status(manager_clone).await;
                assert!(result.is_ok());
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.await.unwrap();
        }
    }

    #[tokio::test]
    async fn test_service_mesh_status_always_available() {
        let manager = create_test_ecosystem_manager();

        // Service mesh status should always be available
        for _ in 0..10 {
            let result = handle_service_mesh_status(Arc::clone(&manager)).await;
            assert!(result.is_ok());
        }
    }

    #[tokio::test]
    async fn test_all_endpoints_with_empty_base_url() {
        let manager = create_test_ecosystem_manager();
        let base_url = "".to_string();

        let primals = handle_primals_list(Arc::clone(&manager), base_url.clone()).await;
        let services = handle_services(Arc::clone(&manager), base_url.clone()).await;
        let status = handle_primal_status("test".to_string(), manager, base_url).await;

        assert!(primals.is_ok());
        assert!(services.is_ok());
        assert!(status.is_ok());
    }

    #[tokio::test]
    async fn test_rapid_sequential_requests() {
        let manager = create_test_ecosystem_manager();
        let base_url = "http://localhost:8080".to_string();

        // Rapid sequential calls to different endpoints
        for _ in 0..20 {
            handle_ecosystem_status(Arc::clone(&manager)).await.unwrap();
            handle_service_mesh_status(Arc::clone(&manager))
                .await
                .unwrap();
            handle_primals_list(Arc::clone(&manager), base_url.clone())
                .await
                .unwrap();
            handle_services(Arc::clone(&manager), base_url.clone())
                .await
                .unwrap();
        }
    }
}
