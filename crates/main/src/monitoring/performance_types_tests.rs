// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Tests for performance monitoring types

#[cfg(test)]
mod tests {
    use crate::monitoring::performance::*;
    use crate::monitoring::PerformanceSummary;
    use chrono::Utc;
    use std::collections::HashMap;
    use std::time::Duration;

    #[test]
    fn test_trend_direction_variants() {
        let improving = TrendDirection::Improving;
        let stable = TrendDirection::Stable;
        let degrading = TrendDirection::Degrading;
        let unknown = TrendDirection::Unknown;

        assert!(matches!(improving, TrendDirection::Improving));
        assert!(matches!(stable, TrendDirection::Stable));
        assert!(matches!(degrading, TrendDirection::Degrading));
        assert!(matches!(unknown, TrendDirection::Unknown));
    }

    #[test]
    fn test_threshold_direction_variants() {
        let above = ThresholdDirection::Above;
        let below = ThresholdDirection::Below;

        assert!(matches!(above, ThresholdDirection::Above));
        assert!(matches!(below, ThresholdDirection::Below));
    }

    #[test]
    fn test_performance_metric_creation() {
        let metric = PerformanceMetric {
            name: "cpu_usage".to_string(),
            current_value: 45.5,
            average_value: 42.3,
            min_value: 20.0,
            max_value: 80.0,
            std_deviation: 10.5,
            sample_count: 1000,
            last_update: Utc::now(),
            trend: TrendDirection::Stable,
        };

        assert_eq!(metric.name, "cpu_usage");
        assert_eq!(metric.current_value, 45.5);
        assert_eq!(metric.sample_count, 1000);
        assert!(matches!(metric.trend, TrendDirection::Stable));
    }

    #[test]
    fn test_performance_metric_serialization() {
        let metric = PerformanceMetric {
            name: "memory_usage".to_string(),
            current_value: 60.0,
            average_value: 55.0,
            min_value: 30.0,
            max_value: 75.0,
            std_deviation: 8.5,
            sample_count: 500,
            last_update: Utc::now(),
            trend: TrendDirection::Degrading,
        };

        let json = serde_json::to_string(&metric).unwrap();
        let deserialized: PerformanceMetric = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.name, "memory_usage");
        assert_eq!(deserialized.current_value, 60.0);
        assert!(matches!(deserialized.trend, TrendDirection::Degrading));
    }

    #[test]
    fn test_resource_utilization_creation() {
        let utilization = ResourceUtilization {
            cpu_percent: 35.5,
            memory_percent: 50.2,
            disk_io_percent: 15.8,
            network_io_percent: 20.3,
            active_threads: 42,
            file_descriptors: 128,
            active_connections: 75,
        };

        assert_eq!(utilization.cpu_percent, 35.5);
        assert_eq!(utilization.memory_percent, 50.2);
        assert_eq!(utilization.active_threads, 42);
        assert_eq!(utilization.file_descriptors, 128);
    }

    #[test]
    fn test_resource_utilization_serialization() {
        let utilization = ResourceUtilization {
            cpu_percent: 40.0,
            memory_percent: 60.0,
            disk_io_percent: 25.0,
            network_io_percent: 30.0,
            active_threads: 50,
            file_descriptors: 256,
            active_connections: 100,
        };

        let json = serde_json::to_string(&utilization).unwrap();
        let deserialized: ResourceUtilization = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.cpu_percent, 40.0);
        assert_eq!(deserialized.active_threads, 50);
    }

    #[test]
    fn test_performance_baseline_creation() {
        let mut values = HashMap::new();
        values.insert("cpu".to_string(), 25.0);
        values.insert("memory".to_string(), 40.0);

        let baseline = PerformanceBaseline {
            name: "production_baseline".to_string(),
            values,
            timestamp: Utc::now(),
            description: "Production baseline metrics".to_string(),
        };

        assert_eq!(baseline.name, "production_baseline");
        assert_eq!(baseline.values.len(), 2);
        assert!(baseline.values.contains_key("cpu"));
    }

    #[test]
    fn test_performance_baseline_serialization() {
        let mut values = HashMap::new();
        values.insert("latency".to_string(), 100.0);

        let baseline = PerformanceBaseline {
            name: "test_baseline".to_string(),
            values,
            timestamp: Utc::now(),
            description: "Test baseline".to_string(),
        };

        let json = serde_json::to_string(&baseline).unwrap();
        let deserialized: PerformanceBaseline = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.name, "test_baseline");
        assert!(deserialized.values.contains_key("latency"));
    }

    #[test]
    fn test_performance_snapshot_creation() {
        let summary = PerformanceSummary {
            cpu_usage: 35.0,
            memory_usage: 45.0,
            network_io: 500.0,
            disk_io: 250.0,
            avg_response_time: 120.0,
            requests_per_second: 800.0,
            error_rate: 0.5,
        };

        let mut component_metrics = HashMap::new();
        let mut api_metrics = HashMap::new();
        api_metrics.insert("latency".to_string(), 150.0);
        component_metrics.insert("api_server".to_string(), api_metrics);

        let utilization = ResourceUtilization {
            cpu_percent: 35.0,
            memory_percent: 45.0,
            disk_io_percent: 20.0,
            network_io_percent: 25.0,
            active_threads: 30,
            file_descriptors: 64,
            active_connections: 50,
        };

        let snapshot = PerformanceSnapshot {
            timestamp: Utc::now(),
            summary,
            component_metrics,
            resource_utilization: utilization,
        };

        assert_eq!(snapshot.summary.cpu_usage, 35.0);
        assert_eq!(snapshot.component_metrics.len(), 1);
        assert_eq!(snapshot.resource_utilization.active_threads, 30);
    }

    #[test]
    fn test_performance_snapshot_serialization() {
        let summary = PerformanceSummary {
            cpu_usage: 30.0,
            memory_usage: 40.0,
            network_io: 400.0,
            disk_io: 200.0,
            avg_response_time: 100.0,
            requests_per_second: 600.0,
            error_rate: 0.3,
        };

        let snapshot = PerformanceSnapshot {
            timestamp: Utc::now(),
            summary,
            component_metrics: HashMap::new(),
            resource_utilization: ResourceUtilization {
                cpu_percent: 30.0,
                memory_percent: 40.0,
                disk_io_percent: 15.0,
                network_io_percent: 20.0,
                active_threads: 25,
                file_descriptors: 48,
                active_connections: 40,
            },
        };

        let json = serde_json::to_string(&snapshot).unwrap();
        let deserialized: PerformanceSnapshot = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.summary.cpu_usage, 30.0);
        assert_eq!(deserialized.resource_utilization.active_threads, 25);
    }

    #[test]
    fn test_performance_threshold_creation() {
        let threshold = PerformanceThreshold {
            name: "cpu_high".to_string(),
            warning_threshold: 70.0,
            critical_threshold: 90.0,
            direction: ThresholdDirection::Above,
        };

        assert_eq!(threshold.name, "cpu_high");
        assert_eq!(threshold.warning_threshold, 70.0);
        assert_eq!(threshold.critical_threshold, 90.0);
        assert!(matches!(threshold.direction, ThresholdDirection::Above));
    }

    #[test]
    fn test_performance_threshold_serialization() {
        let threshold = PerformanceThreshold {
            name: "disk_low".to_string(),
            warning_threshold: 20.0,
            critical_threshold: 10.0,
            direction: ThresholdDirection::Below,
        };

        let json = serde_json::to_string(&threshold).unwrap();
        let deserialized: PerformanceThreshold = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.name, "disk_low");
        assert!(matches!(deserialized.direction, ThresholdDirection::Below));
    }

    #[test]
    fn test_performance_config_default() {
        let config = PerformanceConfig::default();

        assert_eq!(config.max_history_size, 1000);
        assert_eq!(config.sampling_window.as_secs(), 60);
        assert_eq!(config.trend_window.as_secs(), 300);
        assert!(config.thresholds.is_empty());
    }

    #[test]
    fn test_performance_config_custom() {
        let mut thresholds = HashMap::new();
        thresholds.insert(
            "cpu".to_string(),
            PerformanceThreshold {
                name: "cpu_threshold".to_string(),
                warning_threshold: 75.0,
                critical_threshold: 95.0,
                direction: ThresholdDirection::Above,
            },
        );

        let config = PerformanceConfig {
            max_history_size: 500,
            sampling_window: Duration::from_secs(30),
            trend_window: Duration::from_secs(180),
            thresholds,
        };

        assert_eq!(config.max_history_size, 500);
        assert_eq!(config.sampling_window.as_secs(), 30);
        assert_eq!(config.thresholds.len(), 1);
    }

    #[test]
    fn test_performance_config_serialization() {
        let config = PerformanceConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: PerformanceConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.max_history_size, 1000);
        assert_eq!(deserialized.sampling_window.as_secs(), 60);
    }

    #[test]
    fn test_performance_tracker_default() {
        let tracker = PerformanceTracker::default();
        let new_tracker = PerformanceTracker::new();

        // Both should be constructible
        assert!(std::ptr::addr_of!(tracker) != std::ptr::addr_of!(new_tracker));
    }
}
