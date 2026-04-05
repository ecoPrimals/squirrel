// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Tests for ecosystem manager

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::ecosystem::config::EcosystemConfig;
    use crate::monitoring::metrics::MetricsCollector;
    use crate::primal_provider::SquirrelPrimalProvider;
    use crate::session::{SessionConfig, SessionManagerImpl};
    use crate::universal::PrimalCapability;
    use crate::universal_adapter_v2::UniversalAdapterV2;
    use squirrel_mcp_config::EcosystemConfig as McpEcosystemConfig;
    use std::collections::HashMap;
    use std::sync::Arc;

    async fn test_primal_provider() -> SquirrelPrimalProvider {
        let adapter = UniversalAdapterV2::awaken().await.expect("adapter");
        let mc = Arc::new(MetricsCollector::new());
        let em = Arc::new(EcosystemManager::new(
            EcosystemConfig::default(),
            Arc::clone(&mc),
        ));
        let sessions = Arc::new(SessionManagerImpl::new(SessionConfig::default()));
        SquirrelPrimalProvider::new(
            "mgr-test".to_string(),
            McpEcosystemConfig::default(),
            adapter,
            em,
            sessions,
        )
    }

    fn create_test_manager() -> EcosystemManager {
        let config = EcosystemConfig::default();
        let metrics = Arc::new(MetricsCollector::new());
        EcosystemManager::new(config, metrics)
    }

    #[test]
    fn test_ecosystem_manager_creation() {
        let manager = create_test_manager();

        assert!(!manager.config.service_name.is_empty());
        assert_eq!(manager.config.service_host, "localhost");
    }

    #[test]
    fn test_ecosystem_manager_with_custom_config() {
        let config =
            EcosystemConfig::new("custom-squirrel".to_string(), "0.0.0.0".to_string(), 9090);

        let metrics = Arc::new(MetricsCollector::new());
        let manager = EcosystemManager::new(config, metrics);

        assert_eq!(manager.config.service_name, "custom-squirrel");
        assert_eq!(manager.config.service_port, 9090);
    }

    #[tokio::test]
    async fn test_ecosystem_manager_initial_status() {
        let manager = create_test_manager();
        let status = manager.status.read().await;

        assert_eq!(status.status, "initializing");
        assert!(status.initialized_at.is_none());
        assert_eq!(status.active_registrations.len(), 0);
        assert_eq!(status.error_count, 0);
        assert!(status.last_error.is_none());
    }

    #[tokio::test]
    async fn test_ecosystem_manager_initialize() {
        let mut manager = create_test_manager();

        let result = manager.initialize().await;
        assert!(result.is_ok());

        let status = manager.status.read().await;
        assert_eq!(status.status, "initialized");
        assert!(status.initialized_at.is_some());
    }

    #[tokio::test]
    async fn test_get_ecosystem_status_empty() {
        let manager = create_test_manager();

        let ecosystem_status = manager.get_ecosystem_status().await;

        // When no peers are discovered, status is "degraded" and health is 0.5
        assert_eq!(ecosystem_status.status, "degraded");
        assert!((ecosystem_status.overall_health - 0.5).abs() < 0.01);
        assert_eq!(ecosystem_status.discovered_services.len(), 0);
        assert_eq!(ecosystem_status.active_integrations.len(), 0);
    }

    #[test]
    fn test_ecosystem_manager_with_biome() {
        let config = EcosystemConfig {
            biome_id: Some("test-biome".to_string()),
            ..Default::default()
        };

        let metrics = Arc::new(MetricsCollector::new());
        let manager = EcosystemManager::new(config, metrics);

        assert_eq!(manager.config.biome_id, Some("test-biome".to_string()));
    }

    #[test]
    fn test_metrics_collector_integration() {
        let manager = create_test_manager();

        // Verify metrics collector is properly integrated and accessible
        assert!(Arc::strong_count(&manager.metrics_collector) >= 1);
    }

    #[tokio::test]
    async fn test_health_status_initialization() {
        let manager = create_test_manager();
        let status = manager.status.read().await;

        assert!((status.health_status.health_score - 0.0).abs() < f64::EPSILON);
        assert_eq!(status.health_status.component_statuses.len(), 0);
        assert_eq!(status.health_status.health_errors.len(), 0);
    }

    #[test]
    fn test_multiple_managers_independence() {
        let manager1 = create_test_manager();
        let manager2 = create_test_manager();

        // Managers should be independent
        assert_ne!(Arc::as_ptr(&manager1.status), Arc::as_ptr(&manager2.status));
    }

    #[tokio::test]
    async fn test_concurrent_status_access() {
        let manager = Arc::new(create_test_manager());

        let mut handles = vec![];

        for _ in 0..10 {
            let manager_clone = Arc::clone(&manager);
            let handle = tokio::spawn(async move {
                let status = manager_clone.status.read().await;
                assert_eq!(status.status, "initializing");
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.await.expect("should succeed");
        }
    }

    #[test]
    fn test_config_default() {
        let config = EcosystemConfig::default();

        assert!(!config.service_id.is_empty());
        assert!(!config.service_name.is_empty());
        assert_eq!(config.service_port, 8002);
    }

    #[tokio::test]
    async fn test_ecosystem_status_structure() {
        let manager = create_test_manager();
        let status = manager.get_ecosystem_status().await;

        // Verify all status fields are present
        assert!(!status.status.is_empty());
        assert!(status.overall_health >= 0.0 && status.overall_health <= 1.0);
    }

    #[test]
    fn test_service_endpoint_format() {
        let config = EcosystemConfig::default();
        assert!(config.service_mesh_endpoint.starts_with("http"));
    }

    #[tokio::test]
    async fn test_initialize_ecosystem_integration() {
        let config = EcosystemConfig::default();
        let metrics = Arc::new(MetricsCollector::new());

        let result = initialize_ecosystem_integration(config, metrics).await;
        assert!(result.is_ok());

        let manager = result.expect("should succeed");
        let status = manager.status.read().await;
        assert_eq!(status.status, "initialized");
    }

    #[test]
    fn test_config_validation() {
        let config = EcosystemConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_new_constructor() {
        let config =
            EcosystemConfig::new("test-service".to_string(), "127.0.0.1".to_string(), 3000);

        assert_eq!(config.service_name, "test-service");
        assert_eq!(config.service_host, "127.0.0.1");
        assert_eq!(config.service_port, 3000);
    }

    #[test]
    fn test_config_from_env() {
        let config = EcosystemConfig::from_env();
        assert!(!config.service_id.is_empty());
        assert!(!config.service_name.is_empty());
    }

    #[tokio::test]
    async fn shutdown_sets_status_to_shutdown() {
        let manager = create_test_manager();
        manager.shutdown().await.expect("should succeed");
        let st = manager.get_manager_status().await;
        assert_eq!(st.status, "shutdown");
    }

    #[tokio::test]
    async fn update_health_status_recomputes_score() {
        let manager = create_test_manager();
        manager
            .update_health_status(
                "rpc",
                crate::ecosystem::ComponentHealth {
                    status: "healthy".to_string(),
                    last_check: chrono::Utc::now(),
                    error: None,
                    metadata: std::collections::HashMap::new(),
                },
            )
            .await
            .expect("should succeed");
        let st = manager.get_manager_status().await;
        assert!((st.health_status.health_score - 1.0).abs() < f64::EPSILON);
    }

    #[tokio::test]
    async fn discover_services_returns_empty_vec() {
        let manager = create_test_manager();
        let s = manager.discover_services().await.expect("should succeed");
        assert!(s.is_empty());
    }

    #[tokio::test]
    async fn find_services_by_type_deprecated_is_err() {
        let manager = create_test_manager();
        let e = manager
            .find_services_by_type(crate::ecosystem::EcosystemPrimalType::Squirrel)
            .await
            .unwrap_err();
        assert!(matches!(e, crate::error::PrimalError::Configuration(_)));
    }

    #[tokio::test]
    async fn call_primal_api_deprecated_is_err() {
        use crate::ecosystem::EcosystemPrimalType;
        use crate::ecosystem::registry::PrimalApiRequest;
        use std::time::Duration;

        let manager = create_test_manager();
        let req = PrimalApiRequest::new(
            "req-1",
            EcosystemPrimalType::Squirrel,
            EcosystemPrimalType::Squirrel,
            "noop",
            serde_json::json!({}),
            std::collections::HashMap::new(),
            Duration::from_secs(2),
        );
        let e = manager.call_primal_api(req).await.unwrap_err();
        assert!(matches!(e, crate::error::PrimalError::Configuration(_)));
    }

    #[tokio::test]
    async fn complete_coordination_ok() {
        let manager = create_test_manager();
        manager
            .complete_coordination("coord_test", true)
            .await
            .expect("should succeed");
    }

    #[tokio::test]
    async fn authenticate_universal_returns_session_id() {
        let manager = create_test_manager();
        let mut creds = std::collections::HashMap::new();
        creds.insert("user_id".to_string(), "u1".to_string());
        let sid = manager
            .authenticate_universal(creds)
            .await
            .expect("should succeed");
        assert!(sid.starts_with("beardog_session_"));
    }

    #[tokio::test]
    async fn deregister_from_service_mesh_ok() {
        let manager = create_test_manager();
        manager
            .deregister_from_service_mesh()
            .await
            .expect("should succeed");
    }

    #[tokio::test]
    async fn register_squirrel_service_updates_registrations() {
        let provider = test_primal_provider().await;
        let manager = create_test_manager();
        manager
            .register_squirrel_service(&provider)
            .await
            .expect("should succeed");
        let st = manager.get_manager_status().await;
        assert_eq!(st.active_registrations.len(), 1);
        assert!(st.last_registration.is_some());
    }

    #[tokio::test]
    async fn register_with_service_mesh_updates_registrations() {
        let provider = test_primal_provider().await;
        let manager = create_test_manager();
        manager
            .register_with_service_mesh(&provider)
            .await
            .expect("should succeed");
        let st = manager.get_manager_status().await;
        assert_eq!(st.active_registrations.len(), 1);
    }

    #[tokio::test]
    async fn start_coordination_fails_when_no_capability_providers() {
        let manager = create_test_manager();
        let err = manager
            .start_coordination_by_capabilities(vec!["nonexistent.capability.xyz"], HashMap::new())
            .await
            .unwrap_err();
        assert!(matches!(err, crate::error::PrimalError::Configuration(_)));
    }

    #[tokio::test]
    async fn update_health_status_degraded_and_unknown_affect_score() {
        let manager = create_test_manager();
        manager
            .update_health_status(
                "a",
                crate::ecosystem::ComponentHealth {
                    status: "healthy".to_string(),
                    last_check: chrono::Utc::now(),
                    error: None,
                    metadata: HashMap::new(),
                },
            )
            .await
            .expect("should succeed");
        manager
            .update_health_status(
                "b",
                crate::ecosystem::ComponentHealth {
                    status: "degraded".to_string(),
                    last_check: chrono::Utc::now(),
                    error: None,
                    metadata: HashMap::new(),
                },
            )
            .await
            .expect("should succeed");
        manager
            .update_health_status(
                "c",
                crate::ecosystem::ComponentHealth {
                    status: "failed".to_string(),
                    last_check: chrono::Utc::now(),
                    error: Some("e".to_string()),
                    metadata: HashMap::new(),
                },
            )
            .await
            .expect("should succeed");
        let st = manager.get_manager_status().await;
        let expected = (1.0 + 0.5 + 0.0) / 3.0;
        assert!((st.health_status.health_score - expected).abs() < 1e-9);
    }

    #[tokio::test]
    async fn get_discovered_primals_universal_runs() {
        let manager = create_test_manager();
        let primals = manager.get_discovered_primals_universal().await;
        let _ = primals.len();
    }

    #[tokio::test]
    async fn find_primals_by_capability_universal_runs() {
        let manager = create_test_manager();
        let cap = PrimalCapability::Authentication {
            methods: vec!["oauth".to_string()],
        };
        let _ = manager.find_primals_by_capability_universal(&cap).await;
    }

    #[tokio::test]
    async fn match_capabilities_universal_runs() {
        let manager = create_test_manager();
        let req = crate::universal_primal_ecosystem::CapabilityRequest {
            required_capabilities: vec!["test-cap".to_string()],
            optional_capabilities: vec![],
            context: crate::universal::PrimalContext::default(),
            metadata: HashMap::new(),
        };
        let _ = manager.match_capabilities_universal(&req).await;
    }

    #[tokio::test]
    async fn store_data_universal_errors_without_storage_service() {
        let mut manager = create_test_manager();
        manager.initialize().await.expect("should succeed");
        assert!(
            manager
                .store_data_universal("k", &[1, 2, 3], HashMap::new())
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn authenticate_universal_accepts_username_key() {
        let manager = create_test_manager();
        let mut creds = HashMap::new();
        creds.insert("username".to_string(), "alice".to_string());
        let sid = manager
            .authenticate_universal(creds)
            .await
            .expect("should succeed");
        assert!(sid.starts_with("beardog_session_"));
    }

    #[tokio::test]
    async fn get_manager_status_after_registration() {
        let provider = test_primal_provider().await;
        let manager = create_test_manager();
        manager
            .register_squirrel_service(&provider)
            .await
            .expect("should succeed");
        let st = manager.get_manager_status().await;
        assert_eq!(st.status, "initializing");
    }
}
