// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Tests for ecosystem manager

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::ecosystem::config::EcosystemConfig;
    use crate::monitoring::metrics::MetricsCollector;
    use std::sync::Arc;

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
        let mut config = EcosystemConfig::default();
        config.biome_id = Some("test-biome".to_string());

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

        assert_eq!(status.health_status.health_score, 0.0);
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
            handle.await.unwrap();
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

        let manager = result.unwrap();
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
}
