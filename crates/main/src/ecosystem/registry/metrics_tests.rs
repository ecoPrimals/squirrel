//! Edge case tests for metrics operations
//!
//! Tests service statistics, concurrent access, and boundary conditions.

#[cfg(test)]
mod tests {
    use super::super::metrics::MetricsOps;
    use super::super::types::{DiscoveredService, ServiceHealthStatus};
    use crate::ecosystem::EcosystemPrimalType;
    use crate::monitoring::metrics::MetricsCollector;
    use arcstr::ArcStr;
    use chrono::Utc;
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    fn create_test_service(id: &str, primal_type: EcosystemPrimalType) -> DiscoveredService {
        DiscoveredService {
            service_id: id.into(),
            primal_type,
            endpoint: format!("http://localhost:8080/{}", id).into(),
            health_endpoint: format!("http://localhost:8080/{}/health", id).into(),
            api_version: "v1".into(),
            capabilities: vec!["test".into()],
            discovered_at: Utc::now(),
            last_health_check: None,
            health_status: ServiceHealthStatus::Unknown,
            metadata: HashMap::new(),
        }
    }

    #[tokio::test]
    async fn test_get_service_stats_empty_registry() {
        let registry = Arc::new(RwLock::new(HashMap::<String, DiscoveredService>::new()));

        let stats = MetricsOps::get_service_stats(&registry).await;

        assert_eq!(stats.total_services, 0);
        assert_eq!(stats.healthy_services, 0);
        assert_eq!(stats.unhealthy_services, 0);
        assert_eq!(stats.unknown_services, 0);
        assert_eq!(stats.service_types, 0);
    }

    #[tokio::test]
    async fn test_get_service_stats_single_service() {
        let mut services = HashMap::new();
        let service = create_test_service("test", EcosystemPrimalType::Squirrel);
        services.insert("test".to_string(), service);

        let registry = Arc::new(RwLock::new(services));

        let stats = MetricsOps::get_service_stats(&registry).await;

        assert_eq!(stats.total_services, 1);
        assert_eq!(stats.unknown_services, 1);
        assert_eq!(stats.service_types, 1);
    }

    #[tokio::test]
    async fn test_get_service_stats_mixed_health() {
        let mut services = HashMap::new();

        let mut healthy = create_test_service("healthy", EcosystemPrimalType::Squirrel);
        healthy.health_status = ServiceHealthStatus::Healthy;
        services.insert("healthy".to_string(), healthy);

        let mut unhealthy = create_test_service("unhealthy", EcosystemPrimalType::NestGate);
        unhealthy.health_status = ServiceHealthStatus::Unhealthy;
        services.insert("unhealthy".to_string(), unhealthy);

        let unknown = create_test_service("unknown", EcosystemPrimalType::ToadStool);
        services.insert("unknown".to_string(), unknown);

        let registry = Arc::new(RwLock::new(services));

        let stats = MetricsOps::get_service_stats(&registry).await;

        assert_eq!(stats.total_services, 3);
        assert_eq!(stats.healthy_services, 1);
        assert_eq!(stats.unhealthy_services, 1);
        assert_eq!(stats.unknown_services, 1);
        assert_eq!(stats.service_types, 3);
    }

    #[tokio::test]
    async fn test_get_service_stats_multiple_same_type() {
        let mut services = HashMap::new();

        for i in 0..5 {
            let service = create_test_service(&format!("squirrel_{}", i), EcosystemPrimalType::Squirrel);
            services.insert(format!("squirrel_{}", i), service);
        }

        let registry = Arc::new(RwLock::new(services));

        let stats = MetricsOps::get_service_stats(&registry).await;

        assert_eq!(stats.total_services, 5);
        assert_eq!(stats.service_types, 1); // All same type
    }

    #[tokio::test]
    async fn test_get_service_stats_all_healthy() {
        let mut services = HashMap::new();

        for i in 0..10 {
            let mut service = create_test_service(&format!("service_{}", i), EcosystemPrimalType::Squirrel);
            service.health_status = ServiceHealthStatus::Healthy;
            services.insert(format!("service_{}", i), service);
        }

        let registry = Arc::new(RwLock::new(services));

        let stats = MetricsOps::get_service_stats(&registry).await;

        assert_eq!(stats.total_services, 10);
        assert_eq!(stats.healthy_services, 10);
        assert_eq!(stats.unhealthy_services, 0);
        assert_eq!(stats.unknown_services, 0);
    }

    #[tokio::test]
    async fn test_get_service_stats_all_unhealthy() {
        let mut services = HashMap::new();

        for i in 0..5 {
            let mut service = create_test_service(&format!("service_{}", i), EcosystemPrimalType::Squirrel);
            service.health_status = ServiceHealthStatus::Unhealthy;
            services.insert(format!("service_{}", i), service);
        }

        let registry = Arc::new(RwLock::new(services));

        let stats = MetricsOps::get_service_stats(&registry).await;

        assert_eq!(stats.total_services, 5);
        assert_eq!(stats.healthy_services, 0);
        assert_eq!(stats.unhealthy_services, 5);
        assert_eq!(stats.unknown_services, 0);
    }

    #[tokio::test]
    async fn test_concurrent_stats_queries() {
        let mut services = HashMap::new();

        for i in 0..20 {
            let service = create_test_service(
                &format!("service_{}", i),
                if i % 2 == 0 {
                    EcosystemPrimalType::Squirrel
                } else {
                    EcosystemPrimalType::NestGate
                },
            );
            services.insert(format!("service_{}", i), service);
        }

        let registry = Arc::new(RwLock::new(services));
        let mut handles = vec![];

        for _ in 0..50 {
            let reg = Arc::clone(&registry);
            handles.push(tokio::spawn(async move {
                let stats = MetricsOps::get_service_stats(&reg).await;
                assert_eq!(stats.total_services, 20);
                assert_eq!(stats.service_types, 2);
            }));
        }

        for handle in handles {
            handle.await.unwrap();
        }
    }

    #[tokio::test]
    async fn test_collect_metrics_empty_registry() {
        let registry = Arc::new(RwLock::new(HashMap::<String, DiscoveredService>::new()));
        let metrics_collector = Arc::new(MetricsCollector::new());

        // Should not panic with empty registry
        MetricsOps::collect_metrics(&registry, &metrics_collector).await;
    }

    #[tokio::test]
    async fn test_collect_metrics_with_services() {
        let mut services = HashMap::new();
        let mut service = create_test_service("test", EcosystemPrimalType::Squirrel);
        service.health_status = ServiceHealthStatus::Healthy;
        services.insert("test".to_string(), service);

        let registry = Arc::new(RwLock::new(services));
        let metrics_collector = Arc::new(MetricsCollector::new());

        MetricsOps::collect_metrics(&registry, &metrics_collector).await;
    }

    #[tokio::test]
    async fn test_record_service_registration() {
        let metrics_collector = Arc::new(MetricsCollector::new());

        MetricsOps::record_service_registration(&metrics_collector, "test_service", "Squirrel").await;
    }

    #[tokio::test]
    async fn test_record_health_change() {
        let metrics_collector = Arc::new(MetricsCollector::new());

        MetricsOps::record_health_change(
            &metrics_collector,
            "test_service",
            &ServiceHealthStatus::Unknown,
            &ServiceHealthStatus::Healthy,
        )
        .await;
    }

    #[tokio::test]
    async fn test_service_stats_with_diverse_primal_types() {
        let mut services = HashMap::new();

        let primal_types = vec![
            EcosystemPrimalType::Squirrel,
            EcosystemPrimalType::NestGate,
            EcosystemPrimalType::ToadStool,
            EcosystemPrimalType::Songbird,
        ];

        for (i, primal_type) in primal_types.iter().enumerate() {
            let service = create_test_service(&format!("service_{}", i), primal_type.clone());
            services.insert(format!("service_{}", i), service);
        }

        let registry = Arc::new(RwLock::new(services));

        let stats = MetricsOps::get_service_stats(&registry).await;

        assert_eq!(stats.total_services, 4);
        assert_eq!(stats.service_types, 4);
    }

    #[tokio::test]
    async fn test_get_service_stats_consistency() {
        let mut services = HashMap::new();

        for i in 0..10 {
            let mut service = create_test_service(&format!("service_{}", i), EcosystemPrimalType::Squirrel);
            service.health_status = if i < 5 {
                ServiceHealthStatus::Healthy
            } else {
                ServiceHealthStatus::Unhealthy
            };
            services.insert(format!("service_{}", i), service);
        }

        let registry = Arc::new(RwLock::new(services));

        // Call multiple times and verify consistency
        for _ in 0..5 {
            let stats = MetricsOps::get_service_stats(&registry).await;
            assert_eq!(stats.total_services, 10);
            assert_eq!(stats.healthy_services, 5);
            assert_eq!(stats.unhealthy_services, 5);
        }
    }

    #[tokio::test]
    async fn test_concurrent_metrics_collection() {
        let mut services = HashMap::new();
        for i in 0..10 {
            let service = create_test_service(&format!("service_{}", i), EcosystemPrimalType::Squirrel);
            services.insert(format!("service_{}", i), service);
        }

        let registry = Arc::new(RwLock::new(services));
        let metrics_collector = Arc::new(MetricsCollector::new());
        let mut handles = vec![];

        for _ in 0..20 {
            let reg = Arc::clone(&registry);
            let mc = Arc::clone(&metrics_collector);
            handles.push(tokio::spawn(async move {
                MetricsOps::collect_metrics(&reg, &mc).await;
            }));
        }

        for handle in handles {
            handle.await.unwrap();
        }
    }
}

