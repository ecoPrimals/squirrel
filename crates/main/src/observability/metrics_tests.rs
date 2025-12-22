//! Tests for observability metrics module

#[cfg(test)]
mod tests {
    use super::super::*;
    use std::collections::HashMap;
    use std::time::Duration;

    #[test]
    fn test_metrics_config_default() {
        let config = MetricsConfig::default();

        assert_eq!(config.collection_interval, Duration::from_secs(30));
        assert!(config.enable_prometheus);
        assert_eq!(config.custom_endpoints.len(), 0);
        assert_eq!(config.max_metrics_history, 10000);
    }

    #[test]
    fn test_metrics_config_custom() {
        let config = MetricsConfig {
            collection_interval: Duration::from_secs(60),
            enable_prometheus: false,
            custom_endpoints: vec!["http://localhost:9090".to_string()],
            max_metrics_history: 5000,
        };

        assert_eq!(config.collection_interval, Duration::from_secs(60));
        assert!(!config.enable_prometheus);
        assert_eq!(config.custom_endpoints.len(), 1);
        assert_eq!(config.max_metrics_history, 5000);
    }

    #[test]
    fn test_metric_value_creation() {
        let mut labels = HashMap::new();
        labels.insert("service".to_string(), "squirrel".to_string());

        let metric = MetricValue {
            name: "requests_total".to_string(),
            value: 1000.0,
            labels,
            timestamp: chrono::Utc::now(),
            source_primal: Some("squirrel".to_string()),
        };

        assert_eq!(metric.name, "requests_total");
        assert_eq!(metric.value, 1000.0);
        assert_eq!(metric.labels.len(), 1);
        assert!(metric.source_primal.is_some());
    }

    #[test]
    fn test_metric_capability_variants() {
        let capabilities = vec![
            MetricCapability::Counter,
            MetricCapability::Gauge,
            MetricCapability::Histogram,
            MetricCapability::Summary,
            MetricCapability::Custom("custom_metric".to_string()),
        ];

        assert_eq!(capabilities.len(), 5);
    }

    #[test]
    fn test_metrics_endpoint_creation() {
        let endpoint = MetricsEndpoint {
            primal_type: "squirrel".to_string(),
            endpoint: "http://localhost:8080/metrics".to_string(),
            capabilities: vec![MetricCapability::Counter, MetricCapability::Gauge],
            discovered_at: chrono::Utc::now(),
        };

        assert_eq!(endpoint.primal_type, "squirrel");
        assert!(endpoint.endpoint.contains("/metrics"));
        assert_eq!(endpoint.capabilities.len(), 2);
    }

    #[test]
    fn test_universal_metrics_collector_creation() {
        let config = MetricsConfig::default();
        let collector = UniversalMetricsCollector::new(config);

        // Verify collector is created successfully
        assert_eq!(Arc::strong_count(&collector.config), 1);
    }

    #[tokio::test]
    async fn test_metrics_snapshot_empty() {
        let config = MetricsConfig::default();
        let collector = UniversalMetricsCollector::new(config);

        let snapshot = collector.get_metrics_snapshot().await;
        assert_eq!(snapshot.len(), 0);
    }

    #[tokio::test]
    async fn test_discover_metrics_endpoints() {
        let config = MetricsConfig {
            collection_interval: Duration::from_secs(30),
            enable_prometheus: true,
            custom_endpoints: vec!["http://localhost:9090".to_string()],
            max_metrics_history: 10000,
        };
        let collector = UniversalMetricsCollector::new(config);

        let result = collector.discover_metrics_endpoints().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_initialize_metrics() {
        let result = initialize_metrics().await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_metric_value_serialization() {
        let metric = MetricValue {
            name: "test_metric".to_string(),
            value: 42.0,
            labels: HashMap::new(),
            timestamp: chrono::Utc::now(),
            source_primal: Some("test".to_string()),
        };

        let json = serde_json::to_string(&metric).unwrap();
        assert!(json.contains("test_metric"));
        assert!(json.contains("42"));
    }

    #[test]
    fn test_metrics_endpoint_serialization() {
        let endpoint = MetricsEndpoint {
            primal_type: "test_primal".to_string(),
            endpoint: "http://test:8080".to_string(),
            capabilities: vec![MetricCapability::Counter],
            discovered_at: chrono::Utc::now(),
        };

        let json = serde_json::to_string(&endpoint).unwrap();
        let deserialized: MetricsEndpoint = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.primal_type, "test_primal");
    }

    #[test]
    fn test_metric_capability_custom() {
        let custom = MetricCapability::Custom("my_custom_metric".to_string());

        match custom {
            MetricCapability::Custom(name) => assert_eq!(name, "my_custom_metric"),
            _ => panic!("Expected Custom variant"),
        }
    }

    #[tokio::test]
    async fn test_concurrent_metrics_access() {
        let config = MetricsConfig::default();
        let collector = Arc::new(UniversalMetricsCollector::new(config));

        let mut handles = vec![];

        for _ in 0..10 {
            let collector_clone = Arc::clone(&collector);
            let handle = tokio::spawn(async move {
                let snapshot = collector_clone.get_metrics_snapshot().await;
                assert!(snapshot.len() >= 0); // Always true, just testing concurrent access
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.await.unwrap();
        }
    }

    #[test]
    fn test_metrics_config_clone() {
        let config = MetricsConfig::default();
        let cloned = config.clone();

        assert_eq!(config.collection_interval, cloned.collection_interval);
        assert_eq!(config.enable_prometheus, cloned.enable_prometheus);
    }
}
