//! Additional edge case tests for ecosystem endpoints
//!
//! Deep testing: error handling, edge cases, concurrent access

#[cfg(test)]
mod edge_case_tests {
    use super::super::*;
    use crate::ecosystem::{EcosystemConfig, EcosystemManager};
    use crate::monitoring::metrics::MetricsCollector;
    use std::sync::Arc;

    // ===== Helper Functions =====

    fn create_test_ecosystem_manager() -> Arc<EcosystemManager> {
        let config = EcosystemConfig::default();
        let metrics = Arc::new(MetricsCollector::new());
        Arc::new(EcosystemManager::new(config, metrics))
    }

    // ===== Ecosystem Status Edge Cases =====

    #[tokio::test]
    async fn test_ecosystem_status_no_services_discovered() {
        // Test: Ecosystem status when no services have been discovered yet
        let manager = create_test_ecosystem_manager();

        let result = handle_ecosystem_status(manager).await;
        assert!(result.is_ok());
        // Should return empty lists, not error
    }

    #[tokio::test]
    async fn test_ecosystem_status_concurrent_discovery() {
        // Test: Multiple concurrent status requests during service discovery
        let manager = create_test_ecosystem_manager();

        let mut handles = vec![];
        for _ in 0..50 {
            let manager_clone = manager.clone();
            handles.push(tokio::spawn(async move {
                handle_ecosystem_status(manager_clone).await
            }));
        }

        // All should succeed even if discovery is happening
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok());
        }
    }

    #[tokio::test]
    async fn test_ecosystem_status_read_only() {
        // Test: Status endpoint doesn't modify ecosystem state
        let manager = create_test_ecosystem_manager();

        // Call status endpoint multiple times
        for _ in 0..20 {
            let result = handle_ecosystem_status(manager.clone()).await;
            assert!(result.is_ok());
        }

        // Ecosystem manager state should be unchanged
        // (No way to directly verify, but no panics = good)
    }

    // ===== Service Mesh Status Edge Cases =====

    #[tokio::test]
    async fn test_service_mesh_status_always_succeeds() {
        // Test: Service mesh status endpoint is resilient
        let manager = create_test_ecosystem_manager();

        for _ in 0..10 {
            let result = handle_service_mesh_status(manager.clone()).await;
            assert!(result.is_ok());
        }
    }

    #[tokio::test]
    async fn test_service_mesh_status_concurrent_requests() {
        // Test: Handle many concurrent status requests
        let manager = create_test_ecosystem_manager();

        let mut handles = vec![];
        for _ in 0..100 {
            let manager_clone = manager.clone();
            handles.push(tokio::spawn(async move {
                handle_service_mesh_status(manager_clone).await
            }));
        }

        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok());
        }
    }

    // ===== Primals List Edge Cases =====

    #[tokio::test]
    async fn test_primals_list_empty_ecosystem() {
        // Test: Listing primals when ecosystem is empty
        let manager = create_test_ecosystem_manager();
        let base_url = "http://localhost:8080".to_string();

        let result = handle_primals_list(manager, base_url).await;
        assert!(result.is_ok());
        // Should return empty array, not error
    }

    #[tokio::test]
    async fn test_primals_list_with_various_base_urls() {
        // Test: Different base URL formats
        let manager = create_test_ecosystem_manager();

        let urls = vec![
            "http://localhost:8080",
            "https://api.example.com",
            "http://127.0.0.1:3000",
            "https://squirrel.local",
        ];

        for url in urls {
            let result = handle_primals_list(manager.clone(), url.to_string()).await;
            assert!(result.is_ok(), "Failed with URL: {}", url);
        }
    }

    #[tokio::test]
    async fn test_primals_list_concurrent_access() {
        // Test: Concurrent primal list requests
        let manager = create_test_ecosystem_manager();
        let base_url = "http://localhost:8080".to_string();

        let mut handles = vec![];
        for i in 0..50 {
            let manager_clone = manager.clone();
            let url_clone = format!("http://localhost:{}", 8080 + i);
            handles.push(tokio::spawn(async move {
                handle_primals_list(manager_clone, url_clone).await
            }));
        }

        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok());
        }
    }

    // ===== Primal Status Edge Cases =====

    #[tokio::test]
    async fn test_primal_status_nonexistent_primal() {
        // Test: Requesting status for non-existent primal
        let manager = create_test_ecosystem_manager();
        let base_url = "http://localhost:8080".to_string();

        let result = handle_primal_status(
            "nonexistent_primal".to_string(),
            manager,
            base_url,
        )
        .await;

        // Should handle gracefully (empty response or proper error)
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_primal_status_empty_name() {
        // Test: Empty primal name edge case
        let manager = create_test_ecosystem_manager();
        let base_url = "http://localhost:8080".to_string();

        let result = handle_primal_status(String::new(), manager, base_url).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_primal_status_special_characters_in_name() {
        // Test: Special characters in primal name
        let manager = create_test_ecosystem_manager();
        let base_url = "http://localhost:8080".to_string();

        let special_names = vec![
            "primal-with-dashes",
            "primal_with_underscores",
            "primal.with.dots",
            "primal123",
        ];

        for name in special_names {
            let result = handle_primal_status(
                name.to_string(),
                manager.clone(),
                base_url.clone(),
            )
            .await;
            assert!(result.is_ok(), "Failed with name: {}", name);
        }
    }

    #[tokio::test]
    async fn test_primal_status_concurrent_different_primals() {
        // Test: Concurrent requests for different primals
        let manager = create_test_ecosystem_manager();
        let base_url = "http://localhost:8080".to_string();

        let primal_names = vec!["beardog", "songbird", "nestgate", "toadstool"];

        let mut handles = vec![];
        for name in primal_names {
            for _ in 0..10 {
                let manager_clone = manager.clone();
                let url_clone = base_url.clone();
                let name_clone = name.to_string();
                handles.push(tokio::spawn(async move {
                    handle_primal_status(name_clone, manager_clone, url_clone).await
                }));
            }
        }

        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok());
        }
    }

    // ===== Services Endpoint Edge Cases =====

    #[tokio::test]
    async fn test_services_empty_ecosystem() {
        // Test: Services endpoint with no discovered services
        let manager = create_test_ecosystem_manager();
        let base_url = "http://localhost:8080".to_string();

        let result = handle_services(manager, base_url).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_services_concurrent_requests() {
        // Test: Concurrent services requests
        let manager = create_test_ecosystem_manager();
        let base_url = "http://localhost:8080".to_string();

        let mut handles = vec![];
        for _ in 0..100 {
            let manager_clone = manager.clone();
            let url_clone = base_url.clone();
            handles.push(tokio::spawn(async move {
                handle_services(manager_clone, url_clone).await
            }));
        }

        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok());
        }
    }

    // ===== Cross-Endpoint Interaction Tests =====

    #[tokio::test]
    async fn test_all_ecosystem_endpoints_together() {
        // Test: All ecosystem endpoints can be called together
        let manager = create_test_ecosystem_manager();
        let base_url = "http://localhost:8080".to_string();

        let r1 = handle_ecosystem_status(manager.clone()).await;
        let r2 = handle_service_mesh_status(manager.clone()).await;
        let r3 = handle_primals_list(manager.clone(), base_url.clone()).await;
        let r4 = handle_primal_status("beardog".to_string(), manager.clone(), base_url.clone())
            .await;
        let r5 = handle_services(manager, base_url).await;

        assert!(r1.is_ok());
        assert!(r2.is_ok());
        assert!(r3.is_ok());
        assert!(r4.is_ok());
        assert!(r5.is_ok());
    }

    #[tokio::test]
    async fn test_ecosystem_endpoints_sustained_mixed_load() {
        // Test: Sustained mixed load across all ecosystem endpoints
        let manager = create_test_ecosystem_manager();
        let base_url = "http://localhost:8080".to_string();

        let mut handles = vec![];
        for i in 0..200 {
            let manager_clone = manager.clone();
            let url_clone = base_url.clone();

            let handle = tokio::spawn(async move {
                match i % 5 {
                    0 => handle_ecosystem_status(manager_clone).await,
                    1 => handle_service_mesh_status(manager_clone).await,
                    2 => handle_primals_list(manager_clone, url_clone).await,
                    3 => {
                        handle_primal_status("test".to_string(), manager_clone, url_clone).await
                    }
                    _ => handle_services(manager_clone, url_clone).await,
                }
            });
            handles.push(handle);
        }

        // All should succeed
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok());
        }
    }

    // ===== Performance Tests =====

    #[tokio::test]
    async fn test_ecosystem_endpoints_fast_response() {
        // Test: All endpoints respond quickly
        let manager = create_test_ecosystem_manager();
        let base_url = "http://localhost:8080".to_string();

        let start = std::time::Instant::now();
        for _ in 0..20 {
            let _ = handle_ecosystem_status(manager.clone()).await;
            let _ = handle_service_mesh_status(manager.clone()).await;
            let _ = handle_primals_list(manager.clone(), base_url.clone()).await;
        }
        let duration = start.elapsed();

        // 60 total calls should complete in well under a second
        assert!(
            duration.as_millis() < 1000,
            "Ecosystem endpoints too slow: {}ms",
            duration.as_millis()
        );
    }
}

