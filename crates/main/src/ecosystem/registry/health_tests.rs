//! Edge case tests for health monitoring operations
//!
//! Tests timeout handling, concurrent checks, and error conditions.

#[cfg(test)]
mod tests {
    use super::super::health::HealthMonitor;
    use super::super::types::{DiscoveredService, ServiceHealthStatus};
    use super::super::config::HealthConfig;
    use crate::ecosystem::EcosystemPrimalType;
    use arcstr::ArcStr;
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::sync::RwLock;
    use chrono::Utc;

    fn create_test_service(id: &str, endpoint: &str) -> DiscoveredService {
        DiscoveredService {
            service_id: id.into(),
            primal_type: EcosystemPrimalType::Squirrel,
            endpoint: endpoint.into(),
            health_endpoint: format!("{}/health", endpoint).into(),
            api_version: "v1".into(),
            capabilities: vec!["test".into()],
            discovered_at: Utc::now(),
            last_health_check: None,
            health_status: ServiceHealthStatus::Unknown,
            metadata: HashMap::new(),
        }
    }

    #[tokio::test]
    async fn test_health_check_timeout() {
        let http_client = reqwest::Client::new();
        let service = create_test_service("timeout_test", "http://192.0.2.1:9999");
        let config = HealthConfig {
            check_interval: Duration::from_secs(30),
            timeout: Duration::from_millis(100),
            failure_threshold: 3,
            recovery_threshold: 2,
            grace_period: Duration::from_secs(30),
        };

        let result = HealthMonitor::check_service_health(&http_client, &service, &config).await;

        assert_eq!(result.status, ServiceHealthStatus::Unhealthy);
        assert!(result.error.is_some());
    }

    #[tokio::test]
    async fn test_health_check_invalid_url() {
        let http_client = reqwest::Client::new();
        let service = create_test_service("invalid_url", "not-a-valid-url");
        let config = HealthConfig {
            check_interval: Duration::from_secs(30),
            timeout: Duration::from_secs(5),
            failure_threshold: 3,
            recovery_threshold: 2,
            grace_period: Duration::from_secs(30),
        };

        let result = HealthMonitor::check_service_health(&http_client, &service, &config).await;

        assert_eq!(result.status, ServiceHealthStatus::Unhealthy);
        assert!(result.error.is_some());
    }

    #[tokio::test]
    async fn test_get_health_status_empty_registry() {
        let registry = Arc::new(RwLock::new(HashMap::<String, DiscoveredService>::new()));

        let status = HealthMonitor::get_health_status(&registry).await;

        assert!(status.is_empty());
    }

    #[tokio::test]
    async fn test_get_healthy_services_count_empty() {
        let registry = Arc::new(RwLock::new(HashMap::<String, DiscoveredService>::new()));

        let count = HealthMonitor::get_healthy_services_count(&registry).await;

        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_get_healthy_services_count_mixed() {
        let mut services = HashMap::new();
        
        let mut healthy_service = create_test_service("healthy", "http://localhost:8080");
        healthy_service.health_status = ServiceHealthStatus::Healthy;
        services.insert("healthy".to_string(), healthy_service);

        let mut unhealthy_service = create_test_service("unhealthy", "http://localhost:8081");
        unhealthy_service.health_status = ServiceHealthStatus::Unhealthy;
        services.insert("unhealthy".to_string(), unhealthy_service);

        let unknown_service = create_test_service("unknown", "http://localhost:8082");
        services.insert("unknown".to_string(), unknown_service);

        let registry = Arc::new(RwLock::new(services));

        let count = HealthMonitor::get_healthy_services_count(&registry).await;

        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn test_is_service_healthy_not_found() {
        let registry = Arc::new(RwLock::new(HashMap::<String, DiscoveredService>::new()));

        let is_healthy = HealthMonitor::is_service_healthy(&registry, "nonexistent").await;

        assert!(!is_healthy);
    }

    #[tokio::test]
    async fn test_is_service_healthy_unhealthy_service() {
        let mut services = HashMap::new();
        let mut service = create_test_service("unhealthy", "http://localhost:8080");
        service.health_status = ServiceHealthStatus::Unhealthy;
        services.insert("unhealthy".to_string(), service);

        let registry = Arc::new(RwLock::new(services));

        let is_healthy = HealthMonitor::is_service_healthy(&registry, "unhealthy").await;

        assert!(!is_healthy);
    }

    #[tokio::test]
    async fn test_is_service_healthy_healthy_service() {
        let mut services = HashMap::new();
        let mut service = create_test_service("healthy", "http://localhost:8080");
        service.health_status = ServiceHealthStatus::Healthy;
        services.insert("healthy".to_string(), service);

        let registry = Arc::new(RwLock::new(services));

        let is_healthy = HealthMonitor::is_service_healthy(&registry, "healthy").await;

        assert!(is_healthy);
    }

    #[tokio::test]
    async fn test_concurrent_health_status_queries() {
        let mut services = HashMap::new();
        for i in 0..10 {
            let mut service = create_test_service(&format!("service_{}", i), "http://localhost:8080");
            service.health_status = if i % 2 == 0 {
                ServiceHealthStatus::Healthy
            } else {
                ServiceHealthStatus::Unhealthy
            };
            services.insert(format!("service_{}", i), service);
        }

        let registry = Arc::new(RwLock::new(services));
        let mut handles = vec![];

        for _ in 0..20 {
            let reg = Arc::clone(&registry);
            handles.push(tokio::spawn(async move {
                let status = HealthMonitor::get_health_status(&reg).await;
                assert_eq!(status.len(), 10);
            }));
        }

        for handle in handles {
            handle.await.unwrap();
        }
    }

    #[tokio::test]
    async fn test_get_health_status_all_types() {
        let mut services = HashMap::new();
        
        let mut healthy = create_test_service("healthy", "http://localhost:8080");
        healthy.health_status = ServiceHealthStatus::Healthy;
        services.insert("healthy".to_string(), healthy);

        let mut unhealthy = create_test_service("unhealthy", "http://localhost:8081");
        unhealthy.health_status = ServiceHealthStatus::Unhealthy;
        services.insert("unhealthy".to_string(), unhealthy);

        let unknown = create_test_service("unknown", "http://localhost:8082");
        services.insert("unknown".to_string(), unknown);

        let registry = Arc::new(RwLock::new(services));

        let status = HealthMonitor::get_health_status(&registry).await;

        assert_eq!(status.len(), 3);
        assert_eq!(status.get("healthy").unwrap(), &ServiceHealthStatus::Healthy);
        assert_eq!(status.get("unhealthy").unwrap(), &ServiceHealthStatus::Unhealthy);
        assert_eq!(status.get("unknown").unwrap(), &ServiceHealthStatus::Unknown);
    }

    #[tokio::test]
    async fn test_concurrent_is_service_healthy_queries() {
        let mut services = HashMap::new();
        let mut service = create_test_service("test", "http://localhost:8080");
        service.health_status = ServiceHealthStatus::Healthy;
        services.insert("test".to_string(), service);

        let registry = Arc::new(RwLock::new(services));
        let mut handles = vec![];

        for _ in 0..50 {
            let reg = Arc::clone(&registry);
            handles.push(tokio::spawn(async move {
                let is_healthy = HealthMonitor::is_service_healthy(&reg, "test").await;
                assert!(is_healthy);
            }));
        }

        for handle in handles {
            handle.await.unwrap();
        }
    }

    #[tokio::test]
    async fn test_health_check_very_short_timeout() {
        let http_client = reqwest::Client::new();
        let service = create_test_service("short_timeout", "http://192.0.2.1:9999");
        let config = HealthConfig {
            check_interval: Duration::from_secs(30),
            timeout: Duration::from_millis(1), // Very short timeout
            failure_threshold: 3,
            recovery_threshold: 2,
            grace_period: Duration::from_secs(30),
        };

        let result = HealthMonitor::check_service_health(&http_client, &service, &config).await;

        assert_eq!(result.status, ServiceHealthStatus::Unhealthy);
    }

    #[tokio::test]
    async fn test_get_healthy_services_count_all_healthy() {
        let mut services = HashMap::new();
        for i in 0..10 {
            let mut service = create_test_service(&format!("service_{}", i), "http://localhost:8080");
            service.health_status = ServiceHealthStatus::Healthy;
            services.insert(format!("service_{}", i), service);
        }

        let registry = Arc::new(RwLock::new(services));

        let count = HealthMonitor::get_healthy_services_count(&registry).await;

        assert_eq!(count, 10);
    }

    #[tokio::test]
    async fn test_get_healthy_services_count_none_healthy() {
        let mut services = HashMap::new();
        for i in 0..5 {
            let mut service = create_test_service(&format!("service_{}", i), "http://localhost:8080");
            service.health_status = ServiceHealthStatus::Unhealthy;
            services.insert(format!("service_{}", i), service);
        }

        let registry = Arc::new(RwLock::new(services));

        let count = HealthMonitor::get_healthy_services_count(&registry).await;

        assert_eq!(count, 0);
    }
}

