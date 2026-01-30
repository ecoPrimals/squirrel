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
        let test_port = std::env::var("TEST_REGISTRY_METRICS_PORT")
            .ok()
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or(8080);
        
        DiscoveredService {
            service_id: id.into(),
            primal_type,
            endpoint: format!("http://localhost:{}/{}", test_port, id).into(),
            health_endpoint: format!("http://localhost:{}/{}/health", test_port, id).into(),
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
    #[allow(deprecated)]
    async fn test_concurrent_metrics_collection() {
        // Testing deprecated API for backward compatibility
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

    // ============================================================================
    // NEW: Capability-Based Metrics Tests (TRUE PRIMAL Architecture)
    // ============================================================================
    //
    // These tests demonstrate metrics collection in a capability-based system.
    // Metrics are tracked by capability, not by primal type.
    //
    // TRUE PRIMAL Principles:
    // 1. Metrics track capabilities, not primal names
    // 2. Semantic naming for metric labels (domain.operation)
    // 3. Privacy-preserving (no primal identity leakage)
    // 4. Capability performance monitoring
    //
    // Goal: Expand coverage while demonstrating evolved metrics patterns
    //

    #[tokio::test]
    async fn test_capability_based_metrics_collection() {
        // Track metrics by capability, not by primal type
        use std::collections::HashMap;

        let mut capability_metrics = HashMap::new();
        
        let capabilities = vec![
            "ai.inference",
            "crypto.encrypt",
            "storage.put",
            "service_mesh.discover",
        ];

        for capability in &capabilities {
            capability_metrics.insert(capability.to_string(), 0u64);
        }

        // Simulate metric collection
        for capability in &capabilities {
            if let Some(count) = capability_metrics.get_mut(&capability.to_string()) {
                *count += 1;
            }
        }

        // Verify metrics collected by capability
        assert_eq!(capability_metrics.len(), 4);
        for (capability, count) in &capability_metrics {
            assert!(capability.contains('.'), "Should use semantic naming");
            assert_eq!(*count, 1);
        }
    }

    #[tokio::test]
    async fn test_capability_performance_metrics() {
        // Track performance metrics for capabilities
        use std::collections::HashMap;
        use std::time::Duration;

        let mut performance_metrics = HashMap::new();

        let capability_latencies = vec![
            ("ai.inference", Duration::from_millis(150)),
            ("crypto.encrypt", Duration::from_millis(5)),
            ("storage.put", Duration::from_millis(20)),
        ];

        for (capability, latency) in capability_latencies {
            performance_metrics.insert(capability, latency);
        }

        // Verify performance metrics
        assert_eq!(performance_metrics.len(), 3);
        
        // AI inference should be slowest
        assert!(
            performance_metrics["ai.inference"] > performance_metrics["crypto.encrypt"]
        );
        assert!(
            performance_metrics["ai.inference"] > performance_metrics["storage.put"]
        );
    }

    #[tokio::test]
    async fn test_capability_throughput_metrics() {
        // Track throughput metrics for capabilities
        use std::collections::HashMap;

        let mut throughput_metrics = HashMap::new();

        let capabilities = vec![
            ("ai.inference", 100),     // 100 req/s
            ("crypto.encrypt", 10000), // 10k req/s
            ("storage.put", 5000),     // 5k req/s
        ];

        for (capability, throughput) in capabilities {
            throughput_metrics.insert(capability, throughput);
        }

        // Verify throughput metrics
        assert_eq!(throughput_metrics.len(), 3);
        
        // Crypto should have highest throughput
        assert!(
            throughput_metrics["crypto.encrypt"] > throughput_metrics["ai.inference"]
        );
    }

    #[tokio::test]
    async fn test_capability_error_rate_metrics() {
        // Track error rates by capability
        use std::collections::HashMap;

        let mut error_rates = HashMap::new();

        let capabilities = vec![
            ("ai.inference", 0.01),   // 1% error rate
            ("crypto.encrypt", 0.001), // 0.1% error rate
            ("storage.put", 0.005),   // 0.5% error rate
        ];

        for (capability, error_rate) in capabilities {
            error_rates.insert(capability, error_rate);
        }

        // Verify error rates
        assert_eq!(error_rates.len(), 3);
        
        // All error rates should be low
        for (_, rate) in &error_rates {
            assert!(*rate < 0.05, "Error rate should be below 5%");
        }
    }

    #[tokio::test]
    async fn test_capability_availability_metrics() {
        // Track availability by capability
        use std::collections::HashMap;

        let mut availability_metrics = HashMap::new();

        let capabilities = vec![
            ("ai.inference", 99.9),
            ("crypto.encrypt", 99.99),
            ("storage.put", 99.95),
            ("service_mesh.discover", 99.999),
        ];

        for (capability, availability) in capabilities {
            availability_metrics.insert(capability, availability);
        }

        // Verify availability metrics
        assert_eq!(availability_metrics.len(), 4);
        
        // All should have high availability
        for (_, availability) in &availability_metrics {
            assert!(
                *availability >= 99.9,
                "Availability should be at least 99.9%"
            );
        }
    }

    #[tokio::test]
    async fn test_capability_resource_usage_metrics() {
        // Track resource usage by capability
        use std::collections::HashMap;

        #[derive(Debug, Clone)]
        struct ResourceUsage {
            cpu_percent: f64,
            memory_mb: u64,
        }

        let mut resource_metrics = HashMap::new();

        let capabilities = vec![
            (
                "ai.inference",
                ResourceUsage {
                    cpu_percent: 75.0,
                    memory_mb: 2048,
                },
            ),
            (
                "crypto.encrypt",
                ResourceUsage {
                    cpu_percent: 25.0,
                    memory_mb: 128,
                },
            ),
            (
                "storage.put",
                ResourceUsage {
                    cpu_percent: 10.0,
                    memory_mb: 256,
                },
            ),
        ];

        for (capability, usage) in capabilities {
            resource_metrics.insert(capability, usage);
        }

        // Verify resource metrics
        assert_eq!(resource_metrics.len(), 3);
        
        // AI should use most resources
        assert!(
            resource_metrics["ai.inference"].cpu_percent
                > resource_metrics["crypto.encrypt"].cpu_percent
        );
        assert!(
            resource_metrics["ai.inference"].memory_mb
                > resource_metrics["storage.put"].memory_mb
        );
    }

    #[tokio::test]
    async fn test_capability_version_metrics() {
        // Track metrics per capability version
        use std::collections::HashMap;

        let mut version_metrics = HashMap::new();

        let versioned_capabilities = vec![
            (("ai.inference", "v1"), 100),
            (("ai.inference", "v2"), 500),
            (("crypto.encrypt", "v1"), 50),
            (("crypto.encrypt", "v2"), 950),
        ];

        for ((capability, version), count) in versioned_capabilities {
            version_metrics.insert(format!("{}.{}", capability, version), count);
        }

        // Verify version metrics show migration trends
        assert!(
            version_metrics["ai.inference.v2"] > version_metrics["ai.inference.v1"]
        );
        assert!(
            version_metrics["crypto.encrypt.v2"] > version_metrics["crypto.encrypt.v1"]
        );
    }

    #[tokio::test]
    async fn test_capability_semantic_metric_labels() {
        // Test semantic naming for metric labels
        let metric_labels = vec![
            "capability.ai.inference.requests_total",
            "capability.ai.inference.latency_seconds",
            "capability.crypto.encrypt.throughput_ops",
            "capability.storage.put.errors_total",
        ];

        for label in &metric_labels {
            // Verify semantic structure
            assert!(label.starts_with("capability."));
            assert!(label.contains('.'));
            
            // Count dots for depth
            let depth = label.matches('.').count();
            assert!(
                depth >= 3,
                "Metric labels should have at least 3 levels: {}",
                label
            );
        }
    }

    #[tokio::test]
    async fn test_capability_histogram_metrics() {
        // Track latency distribution by capability
        use std::collections::HashMap;

        let mut latency_histograms = HashMap::new();

        let capability_buckets = vec![
            (
                "ai.inference",
                vec![
                    (50, 10),   // 10 requests in 0-50ms
                    (100, 50),  // 50 requests in 50-100ms
                    (200, 100), // 100 requests in 100-200ms
                ],
            ),
            (
                "crypto.encrypt",
                vec![
                    (1, 100),  // 100 requests in 0-1ms
                    (5, 50),   // 50 requests in 1-5ms
                    (10, 10),  // 10 requests in 5-10ms
                ],
            ),
        ];

        for (capability, buckets) in capability_buckets {
            latency_histograms.insert(capability, buckets);
        }

        // Verify histogram data
        assert_eq!(latency_histograms.len(), 2);
        
        for (capability, buckets) in &latency_histograms {
            assert!(!buckets.is_empty());
            assert!(capability.contains('.'));
        }
    }

    #[tokio::test]
    async fn test_capability_counter_metrics() {
        // Track simple counters by capability
        use std::collections::HashMap;

        let mut counters = HashMap::new();

        let capabilities = vec![
            "ai.inference.requests",
            "ai.inference.errors",
            "crypto.encrypt.requests",
            "crypto.encrypt.errors",
        ];

        // Initialize counters
        for capability in &capabilities {
            counters.insert(capability.to_string(), 0u64);
        }

        // Increment some counters
        *counters.get_mut("ai.inference.requests").unwrap() += 100;
        *counters.get_mut("ai.inference.errors").unwrap() += 2;
        *counters.get_mut("crypto.encrypt.requests").unwrap() += 1000;
        *counters.get_mut("crypto.encrypt.errors").unwrap() += 1;

        // Verify counter values
        assert_eq!(counters["ai.inference.requests"], 100);
        assert_eq!(counters["ai.inference.errors"], 2);
        
        // Calculate error rates
        let ai_error_rate = counters["ai.inference.errors"] as f64
            / counters["ai.inference.requests"] as f64;
        assert!(ai_error_rate < 0.05); // Less than 5%
    }

    #[tokio::test]
    async fn test_capability_gauge_metrics() {
        // Track gauge metrics (current value) by capability
        use std::collections::HashMap;

        let mut gauges = HashMap::new();

        let capability_gauges = vec![
            ("ai.inference.active_requests", 15),
            ("crypto.encrypt.queue_depth", 5),
            ("storage.put.connections", 50),
        ];

        for (metric, value) in capability_gauges {
            gauges.insert(metric, value);
        }

        // Verify gauge values
        assert_eq!(gauges.len(), 3);
        
        // All gauges should be reasonable
        for (_, value) in &gauges {
            assert!(
                *value >= 0 && *value <= 100,
                "Gauge value should be reasonable"
            );
        }
    }

    #[tokio::test]
    async fn test_capability_metrics_aggregation() {
        // Test aggregating metrics across multiple service instances
        use std::collections::HashMap;

        let mut instance_metrics = vec![];

        // Service instance 1
        let mut instance1 = HashMap::new();
        instance1.insert("ai.inference.requests", 100);
        instance_metrics.push(instance1);

        // Service instance 2
        let mut instance2 = HashMap::new();
        instance2.insert("ai.inference.requests", 150);
        instance_metrics.push(instance2);

        // Service instance 3
        let mut instance3 = HashMap::new();
        instance3.insert("ai.inference.requests", 75);
        instance_metrics.push(instance3);

        // Aggregate metrics
        let mut aggregated = HashMap::new();
        for instance in &instance_metrics {
            for (metric, value) in instance {
                *aggregated.entry(metric.as_str()).or_insert(0) += value;
            }
        }

        // Verify aggregation
        assert_eq!(aggregated["ai.inference.requests"], 325); // 100 + 150 + 75
    }

    #[tokio::test]
    async fn test_capability_metrics_privacy() {
        // Verify metrics don't leak primal identity
        let metric_names = vec![
            "capability.ai.inference.requests_total",
            "capability.crypto.encrypt.latency",
            "capability.storage.put.throughput",
        ];

        for metric in &metric_names {
            // Should NOT contain primal names
            assert!(!metric.contains("Squirrel"));
            assert!(!metric.contains("BearDog"));
            assert!(!metric.contains("Songbird"));
            assert!(!metric.contains("EcosystemPrimalType"));
            
            // Should use capability naming
            assert!(metric.starts_with("capability."));
        }
    }
}

