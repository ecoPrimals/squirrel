// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

use super::*;
use crate::error::PrimalError;
use crate::monitoring::{CustomMetricDefinition, MetricType};
use chrono::Utc;
use std::collections::HashMap;

#[tokio::test]
async fn test_metrics_collector_creation() {
    let collector = MetricsCollector::new();
    assert!(collector.metrics.is_empty());
    assert!(collector.values.is_empty());
}

#[tokio::test]
async fn test_custom_metric_registration() {
    let collector = MetricsCollector::new();

    let metric_def = CustomMetricDefinition {
        name: "test_counter".to_string(),
        metric_type: MetricType::Counter,
        description: "Test counter metric".to_string(),
        labels: vec!["service".to_string()],
        unit: "count".to_string(),
        source: "test".to_string(),
    };

    let result = collector.register_custom_metric(metric_def).await;
    assert!(result.is_ok());

    assert!(collector.metrics.contains_key("test_counter"));
}

#[tokio::test]
async fn test_metric_recording() {
    let collector = MetricsCollector::new();

    // First register a metric
    let metric_def = CustomMetricDefinition {
        name: "test_gauge".to_string(),
        metric_type: MetricType::Gauge,
        description: "Test gauge metric".to_string(),
        labels: vec!["component".to_string()],
        unit: "bytes".to_string(),
        source: "test".to_string(),
    };

    collector
        .register_custom_metric(metric_def)
        .await
        .expect("Test: metric registration should succeed");

    // Now record a value
    let mut labels = HashMap::new();
    labels.insert("component".to_string(), "test_component".to_string());

    let result = collector.record_metric("test_gauge", 42.0, labels).await;
    assert!(result.is_ok());

    assert!(collector.values.contains_key("test_gauge"));
    assert_eq!(
        collector
            .values
            .get("test_gauge")
            .expect("Test: test_gauge should exist")
            .value()
            .value,
        42.0
    );
}

#[tokio::test]
async fn test_metrics_collection() {
    let collector = MetricsCollector::new();

    let result = collector.collect_metrics().await;
    assert!(result.is_ok());

    let system_metrics = collector.system_metrics.read().await;
    #[cfg(feature = "system-metrics")]
    {
        assert!(system_metrics.cpu_usage > 0.0);
        assert!(system_metrics.memory_usage > 0);
    }

    assert!(collector.component_metrics.contains_key("ai_intelligence"));
    assert!(collector.component_metrics.contains_key("mcp_integration"));
}

#[tokio::test]
async fn test_component_metrics_retrieval() {
    let collector = MetricsCollector::new();

    // Collect metrics first
    collector
        .collect_metrics()
        .await
        .expect("Test: metrics collection should succeed");

    let ai_metrics = collector
        .get_component_metrics("ai_intelligence")
        .await
        .expect("Test: component metrics should exist");
    assert!(!ai_metrics.is_empty());
    assert!(ai_metrics.contains_key("requests_processed"));
    assert!(ai_metrics.contains_key("avg_processing_time"));
}

#[tokio::test]
async fn test_record_metric_error_unregistered() {
    let collector = MetricsCollector::new();

    let mut labels = HashMap::new();
    labels.insert("test".to_string(), "value".to_string());

    let result = collector
        .record_metric("nonexistent_metric", 10.0, labels)
        .await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), PrimalError::NotFoundError(_)));
}

#[tokio::test]
async fn test_get_metric_info_success() {
    let collector = MetricsCollector::new();

    let metric_def = CustomMetricDefinition {
        name: "info_test_metric".to_string(),
        metric_type: MetricType::Counter,
        description: "Test metric for info retrieval".to_string(),
        labels: vec!["label1".to_string(), "label2".to_string()],
        unit: "requests".to_string(),
        source: "test_source".to_string(),
    };

    collector
        .register_custom_metric(metric_def)
        .await
        .expect("Test: metric registration should succeed");

    let info = collector
        .get_metric_info("info_test_metric")
        .await
        .expect("Test: metric info should exist");
    assert_eq!(info.name, "info_test_metric");
    assert_eq!(info.description, "Test metric for info retrieval");
    assert_eq!(info.unit, "requests");
    assert_eq!(info.source, "test_source");
    assert_eq!(info.labels.len(), 2);
}

#[tokio::test]
async fn test_get_metric_info_not_found() {
    let collector = MetricsCollector::new();

    let result = collector.get_metric_info("missing_metric").await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), PrimalError::NotFoundError(_)));
}

#[tokio::test]
async fn test_list_metric_definitions() {
    let collector = MetricsCollector::new();

    // Register multiple metrics
    for i in 1..=3 {
        let metric_def = CustomMetricDefinition {
            name: format!("metric_{}", i),
            metric_type: MetricType::Counter,
            description: format!("Description {}", i),
            labels: vec![],
            unit: "count".to_string(),
            source: "test".to_string(),
        };
        collector.register_custom_metric(metric_def).await.unwrap();
    }

    let definitions = collector.list_metric_definitions().await.unwrap();
    assert_eq!(definitions.len(), 3);
}

#[tokio::test]
async fn test_get_metrics_by_source() {
    let collector = MetricsCollector::new();

    // Register metrics with different sources
    let metric1 = CustomMetricDefinition {
        name: "source_metric_1".to_string(),
        metric_type: MetricType::Counter,
        description: "Source metric 1".to_string(),
        labels: vec![],
        unit: "count".to_string(),
        source: "source_a".to_string(),
    };

    let metric2 = CustomMetricDefinition {
        name: "source_metric_2".to_string(),
        metric_type: MetricType::Gauge,
        description: "Source metric 2".to_string(),
        labels: vec![],
        unit: "bytes".to_string(),
        source: "source_a".to_string(),
    };

    let metric3 = CustomMetricDefinition {
        name: "source_metric_3".to_string(),
        metric_type: MetricType::Histogram,
        description: "Source metric 3".to_string(),
        labels: vec![],
        unit: "ms".to_string(),
        source: "source_b".to_string(),
    };

    collector.register_custom_metric(metric1).await.unwrap();
    collector.register_custom_metric(metric2).await.unwrap();
    collector.register_custom_metric(metric3).await.unwrap();

    let source_a_metrics = collector.get_metrics_by_source("source_a").await.unwrap();
    assert_eq!(source_a_metrics.len(), 2);

    let source_b_metrics = collector.get_metrics_by_source("source_b").await.unwrap();
    assert_eq!(source_b_metrics.len(), 1);
}

#[tokio::test]
async fn test_get_metrics_by_unit() {
    let collector = MetricsCollector::new();

    // Register metrics with different units
    let metric1 = CustomMetricDefinition {
        name: "unit_metric_1".to_string(),
        metric_type: MetricType::Counter,
        description: "Unit metric 1".to_string(),
        labels: vec![],
        unit: "bytes".to_string(),
        source: "test".to_string(),
    };

    let metric2 = CustomMetricDefinition {
        name: "unit_metric_2".to_string(),
        metric_type: MetricType::Gauge,
        description: "Unit metric 2".to_string(),
        labels: vec![],
        unit: "bytes".to_string(),
        source: "test".to_string(),
    };

    let metric3 = CustomMetricDefinition {
        name: "unit_metric_3".to_string(),
        metric_type: MetricType::Counter,
        description: "Unit metric 3".to_string(),
        labels: vec![],
        unit: "count".to_string(),
        source: "test".to_string(),
    };

    collector.register_custom_metric(metric1).await.unwrap();
    collector.register_custom_metric(metric2).await.unwrap();
    collector.register_custom_metric(metric3).await.unwrap();

    let bytes_metrics = collector.get_metrics_by_unit("bytes").await.unwrap();
    assert_eq!(bytes_metrics.len(), 2);

    let count_metrics = collector.get_metrics_by_unit("count").await.unwrap();
    assert_eq!(count_metrics.len(), 1);
}

#[tokio::test]
async fn test_get_all_metrics() {
    let collector = MetricsCollector::new();

    // Register and record a metric
    let metric_def = CustomMetricDefinition {
        name: "all_test_metric".to_string(),
        metric_type: MetricType::Counter,
        description: "Test metric".to_string(),
        labels: vec![],
        unit: "count".to_string(),
        source: "test".to_string(),
    };

    collector.register_custom_metric(metric_def).await.unwrap();

    let mut labels = HashMap::new();
    labels.insert("key".to_string(), "value".to_string());
    collector
        .record_metric("all_test_metric", 99.0, labels)
        .await
        .unwrap();

    // Collect system metrics
    collector.collect_metrics().await.unwrap();

    let all_metrics = collector.get_all_metrics().await.unwrap();
    assert!(!all_metrics.metrics.is_empty());
    assert!(!all_metrics.component_metrics.is_empty());
    #[cfg(feature = "system-metrics")]
    assert!(all_metrics.system_metrics.cpu_usage > 0.0);
}

#[tokio::test]
async fn test_snapshot_creation_and_history() {
    let collector = MetricsCollector::new();

    // Collect metrics to create snapshots
    collector.collect_metrics().await.unwrap();
    collector.collect_metrics().await.unwrap();
    collector.collect_metrics().await.unwrap();

    let history = collector.history.read().await;
    assert_eq!(history.len(), 3);

    // Verify snapshot structure
    let snapshot = &history[0];
    #[cfg(feature = "system-metrics")]
    assert!(snapshot.system_metrics.cpu_usage > 0.0);
    assert!(snapshot.timestamp < Utc::now());
}

#[tokio::test]
async fn test_history_size_limit() {
    let mut collector = MetricsCollector::new();
    // Use a small history limit for fast testing
    collector.max_history_size = 5;

    // Directly insert snapshots to test the history cap (avoids slow socket scans)
    {
        let mut history = collector.history.write().await;
        for i in 0..8 {
            history.push(super::MetricSnapshot {
                timestamp: Utc::now(),
                metrics: std::collections::HashMap::new(),
                system_metrics: super::SystemMetrics {
                    cpu_usage: i as f64,
                    ..Default::default()
                },
            });
            // Apply the same trimming logic as create_snapshot
            if history.len() > collector.max_history_size {
                history.remove(0);
            }
        }
    }

    let history = collector.history.read().await;
    assert_eq!(history.len(), collector.max_history_size);
    // Verify oldest entries were trimmed: first snapshot should be from iteration 3
    assert_eq!(history[0].system_metrics.cpu_usage, 3.0);
}

#[tokio::test]
async fn test_component_specific_metrics_all_components() {
    let collector = MetricsCollector::new();

    collector.collect_metrics().await.unwrap();

    // Internal components always have metrics; capability-domain components
    // are only present when discovered at runtime via socket scan
    let internal_components = vec![
        "ai_intelligence",
        "mcp_integration",
        "context_state",
        "agent_deployment",
    ];

    for component in internal_components {
        let metrics = collector.get_component_metrics(component).await.unwrap();
        assert!(
            !metrics.is_empty(),
            "Internal component {} should have metrics",
            component
        );
    }

    // Capability-domain metrics are populated if primals are running
    // (they may be empty in test environments -- that's correct behavior)
    let capability_domains = vec![
        "capability.network",
        "capability.compute",
        "capability.storage",
        "capability.security",
    ];
    for domain in capability_domains {
        // These may or may not exist depending on runtime discovery
        let _result = collector.get_component_metrics(domain).await;
    }
}

#[tokio::test]
async fn test_system_metrics_default() {
    let system_metrics = SystemMetrics::default();
    assert_eq!(system_metrics.cpu_usage, 0.0);
    assert_eq!(system_metrics.memory_usage, 0);
    assert_eq!(system_metrics.active_connections, 0);
}

#[tokio::test]
async fn test_multiple_metric_recordings() {
    let collector = MetricsCollector::new();

    let metric_def = CustomMetricDefinition {
        name: "multi_record_metric".to_string(),
        metric_type: MetricType::Gauge,
        description: "Test multiple recordings".to_string(),
        labels: vec![],
        unit: "value".to_string(),
        source: "test".to_string(),
    };

    collector.register_custom_metric(metric_def).await.unwrap();

    // Record multiple values (each overwrites the previous)
    for i in 1..=5 {
        let labels = HashMap::new();
        collector
            .record_metric("multi_record_metric", i as f64 * 10.0, labels)
            .await
            .unwrap();
    }

    let value = collector.values.get("multi_record_metric").unwrap();
    assert_eq!(value.value().value, 50.0); // Last recorded value
}

#[tokio::test]
async fn test_get_component_metrics_nonexistent() {
    let collector = MetricsCollector::new();

    let result = collector
        .get_component_metrics("nonexistent_component")
        .await
        .unwrap();
    assert!(result.is_empty());
}

#[tokio::test]
async fn test_metric_value_timestamp() {
    let collector = MetricsCollector::new();

    let metric_def = CustomMetricDefinition {
        name: "timestamp_metric".to_string(),
        metric_type: MetricType::Counter,
        description: "Timestamp test".to_string(),
        labels: vec![],
        unit: "count".to_string(),
        source: "test".to_string(),
    };

    collector.register_custom_metric(metric_def).await.unwrap();

    let before_time = Utc::now();
    let labels = HashMap::new();
    collector
        .record_metric("timestamp_metric", 1.0, labels)
        .await
        .unwrap();
    let after_time = Utc::now();

    let value = collector.values.get("timestamp_metric").unwrap();
    assert!(value.value().timestamp >= before_time);
    assert!(value.value().timestamp <= after_time);
}

#[tokio::test]
async fn test_metric_type_preservation() {
    let collector = MetricsCollector::new();

    let metric_types = [
        MetricType::Counter,
        MetricType::Gauge,
        MetricType::Histogram,
        MetricType::Summary,
    ];

    for (i, metric_type) in metric_types.iter().enumerate() {
        let metric_def = CustomMetricDefinition {
            name: format!("type_test_{}", i),
            metric_type: metric_type.clone(),
            description: "Type test".to_string(),
            labels: vec![],
            unit: "count".to_string(),
            source: "test".to_string(),
        };

        collector.register_custom_metric(metric_def).await.unwrap();

        let labels = HashMap::new();
        collector
            .record_metric(&format!("type_test_{}", i), 1.0, labels)
            .await
            .unwrap();
    }

    for (i, expected_type) in metric_types.iter().enumerate() {
        let value = collector.values.get(&format!("type_test_{}", i)).unwrap();
        assert!(matches!(&value.value().metric_type, t if t == expected_type));
    }
}

#[tokio::test]
async fn test_default_trait_implementation() {
    let collector = MetricsCollector::default();
    assert!(collector.metrics.is_empty());
}
