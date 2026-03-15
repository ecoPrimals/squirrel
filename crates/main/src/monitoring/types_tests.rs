// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Tests for monitoring module types

#[cfg(test)]
mod tests {
    use crate::monitoring::*;
    use chrono::Utc;
    use std::collections::HashMap;
    use std::time::Duration;

    #[test]
    fn test_health_state_variants() {
        let healthy = HealthState::Healthy;
        let warning = HealthState::Warning;
        let critical = HealthState::Critical;
        let unknown = HealthState::Unknown;

        assert!(matches!(healthy, HealthState::Healthy));
        assert!(matches!(warning, HealthState::Warning));
        assert!(matches!(critical, HealthState::Critical));
        assert!(matches!(unknown, HealthState::Unknown));
    }

    #[test]
    fn test_health_state_equality() {
        assert_eq!(HealthState::Healthy, HealthState::Healthy);
        assert_ne!(HealthState::Healthy, HealthState::Warning);
    }

    #[test]
    fn test_metric_type_variants() {
        let counter = MetricType::Counter;
        let gauge = MetricType::Gauge;
        let histogram = MetricType::Histogram;
        let summary = MetricType::Summary;

        assert_eq!(counter, MetricType::Counter);
        assert_eq!(gauge, MetricType::Gauge);
        assert_eq!(histogram, MetricType::Histogram);
        assert_eq!(summary, MetricType::Summary);
    }

    #[test]
    fn test_comparison_operator_variants() {
        let gt = ComparisonOperator::GreaterThan;
        let gte = ComparisonOperator::GreaterThanOrEqual;
        let lt = ComparisonOperator::LessThan;
        let lte = ComparisonOperator::LessThanOrEqual;
        let eq = ComparisonOperator::Equal;
        let ne = ComparisonOperator::NotEqual;

        assert!(matches!(gt, ComparisonOperator::GreaterThan));
        assert!(matches!(gte, ComparisonOperator::GreaterThanOrEqual));
        assert!(matches!(lt, ComparisonOperator::LessThan));
        assert!(matches!(lte, ComparisonOperator::LessThanOrEqual));
        assert!(matches!(eq, ComparisonOperator::Equal));
        assert!(matches!(ne, ComparisonOperator::NotEqual));
    }

    #[test]
    fn test_alert_severity_variants() {
        let critical = AlertSeverity::Critical;
        let high = AlertSeverity::High;
        let medium = AlertSeverity::Medium;
        let low = AlertSeverity::Low;
        let info = AlertSeverity::Info;

        assert!(matches!(critical, AlertSeverity::Critical));
        assert!(matches!(high, AlertSeverity::High));
        assert!(matches!(medium, AlertSeverity::Medium));
        assert!(matches!(low, AlertSeverity::Low));
        assert!(matches!(info, AlertSeverity::Info));
    }

    #[test]
    fn test_custom_metric_definition_creation() {
        let metric = CustomMetricDefinition {
            name: "request_count".to_string(),
            metric_type: MetricType::Counter,
            description: "Total requests".to_string(),
            labels: vec!["endpoint".to_string(), "method".to_string()],
            unit: "requests".to_string(),
            source: "http_server".to_string(),
        };

        assert_eq!(metric.name, "request_count");
        assert_eq!(metric.metric_type, MetricType::Counter);
        assert_eq!(metric.labels.len(), 2);
    }

    #[test]
    fn test_alert_rule_creation() {
        let rule = AlertRule {
            name: "high_cpu".to_string(),
            metric: "cpu_usage".to_string(),
            threshold: 80.0,
            operator: ComparisonOperator::GreaterThan,
            severity: AlertSeverity::High,
            description: "CPU usage above 80%".to_string(),
            evaluation_window: Duration::from_secs(60),
            for_duration: Duration::from_secs(300),
        };

        assert_eq!(rule.name, "high_cpu");
        assert_eq!(rule.threshold, 80.0);
        assert!(matches!(rule.operator, ComparisonOperator::GreaterThan));
        assert!(matches!(rule.severity, AlertSeverity::High));
    }

    #[test]
    fn test_performance_summary_creation() {
        let perf = PerformanceSummary {
            cpu_usage: 45.5,
            memory_usage: 60.2,
            network_io: 1024.0,
            disk_io: 512.0,
            avg_response_time: 150.0,
            requests_per_second: 1000.0,
            error_rate: 0.5,
        };

        assert_eq!(perf.cpu_usage, 45.5);
        assert_eq!(perf.memory_usage, 60.2);
        assert_eq!(perf.requests_per_second, 1000.0);
        assert_eq!(perf.error_rate, 0.5);
    }

    #[test]
    fn test_component_status_creation() {
        let mut metrics = HashMap::new();
        metrics.insert("latency".to_string(), 25.5);
        metrics.insert("throughput".to_string(), 500.0);

        let status = ComponentStatus {
            name: "api_server".to_string(),
            health: HealthState::Healthy,
            metrics,
            last_check: Utc::now(),
            status_message: "All systems operational".to_string(),
        };

        assert_eq!(status.name, "api_server");
        assert_eq!(status.health, HealthState::Healthy);
        assert_eq!(status.metrics.len(), 2);
    }

    #[test]
    fn test_system_status_creation() {
        let mut components = HashMap::new();
        components.insert(
            "db".to_string(),
            ComponentStatus {
                name: "db".to_string(),
                health: HealthState::Healthy,
                metrics: HashMap::new(),
                last_check: Utc::now(),
                status_message: "OK".to_string(),
            },
        );

        let status = SystemStatus {
            health: HealthState::Healthy,
            performance: PerformanceSummary {
                cpu_usage: 30.0,
                memory_usage: 50.0,
                network_io: 800.0,
                disk_io: 400.0,
                avg_response_time: 100.0,
                requests_per_second: 500.0,
                error_rate: 0.1,
            },
            active_alerts: 0,
            last_update: Utc::now(),
            uptime: Duration::from_secs(3600),
            components,
        };

        assert_eq!(status.health, HealthState::Healthy);
        assert_eq!(status.active_alerts, 0);
        assert_eq!(status.components.len(), 1);
        assert_eq!(status.uptime.as_secs(), 3600);
    }

    #[test]
    fn test_monitoring_config_serialization() {
        let mut custom_metrics = HashMap::new();
        custom_metrics.insert(
            "custom1".to_string(),
            CustomMetricDefinition {
                name: "custom1".to_string(),
                metric_type: MetricType::Gauge,
                description: "Custom gauge".to_string(),
                labels: vec![],
                unit: "units".to_string(),
                source: "app".to_string(),
            },
        );

        let config = MonitoringConfig {
            collection_interval: Duration::from_secs(10),
            health_check_interval: Duration::from_secs(30),
            performance_interval: Duration::from_secs(60),
            alert_evaluation_interval: Duration::from_secs(15),
            max_metrics_history: 1000,
            enable_prometheus: true,
            prometheus_endpoint: "/metrics".to_string(),
            custom_metrics,
            alert_rules: vec![],
            metrics: vec!["cpu".to_string(), "memory".to_string()],
            alert_thresholds: HashMap::new(),
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: MonitoringConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.max_metrics_history, 1000);
        assert!(deserialized.enable_prometheus);
        assert_eq!(deserialized.metrics.len(), 2);
    }

    #[test]
    fn test_health_state_serialization() {
        let healthy = HealthState::Healthy;
        let json = serde_json::to_string(&healthy).unwrap();
        let deserialized: HealthState = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized, HealthState::Healthy);
    }

    #[test]
    fn test_alert_rule_serialization() {
        let rule = AlertRule {
            name: "test_alert".to_string(),
            metric: "test_metric".to_string(),
            threshold: 100.0,
            operator: ComparisonOperator::GreaterThan,
            severity: AlertSeverity::Medium,
            description: "Test alert".to_string(),
            evaluation_window: Duration::from_secs(60),
            for_duration: Duration::from_secs(120),
        };

        let json = serde_json::to_string(&rule).unwrap();
        let deserialized: AlertRule = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.name, "test_alert");
        assert_eq!(deserialized.threshold, 100.0);
    }

    #[test]
    fn test_performance_summary_serialization() {
        let perf = PerformanceSummary {
            cpu_usage: 25.0,
            memory_usage: 35.0,
            network_io: 500.0,
            disk_io: 250.0,
            avg_response_time: 75.0,
            requests_per_second: 300.0,
            error_rate: 0.2,
        };

        let json = serde_json::to_string(&perf).unwrap();
        let deserialized: PerformanceSummary = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.cpu_usage, 25.0);
        assert_eq!(deserialized.requests_per_second, 300.0);
    }

    #[test]
    fn test_component_status_serialization() {
        let status = ComponentStatus {
            name: "test_component".to_string(),
            health: HealthState::Warning,
            metrics: HashMap::new(),
            last_check: Utc::now(),
            status_message: "Minor issues detected".to_string(),
        };

        let json = serde_json::to_string(&status).unwrap();
        let deserialized: ComponentStatus = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.name, "test_component");
        assert_eq!(deserialized.health, HealthState::Warning);
    }

    #[test]
    fn test_metric_type_serialization() {
        let counter = MetricType::Counter;
        let json = serde_json::to_string(&counter).unwrap();
        let deserialized: MetricType = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized, MetricType::Counter);
    }

    #[test]
    fn test_comparison_operator_serialization() {
        let gt = ComparisonOperator::GreaterThan;
        let json = serde_json::to_string(&gt).unwrap();
        let deserialized: ComparisonOperator = serde_json::from_str(&json).unwrap();

        assert!(matches!(deserialized, ComparisonOperator::GreaterThan));
    }

    #[test]
    fn test_alert_severity_serialization() {
        let critical = AlertSeverity::Critical;
        let json = serde_json::to_string(&critical).unwrap();
        let deserialized: AlertSeverity = serde_json::from_str(&json).unwrap();

        assert!(matches!(deserialized, AlertSeverity::Critical));
    }

    #[test]
    fn test_custom_metric_definition_serialization() {
        let metric = CustomMetricDefinition {
            name: "test_metric".to_string(),
            metric_type: MetricType::Histogram,
            description: "Test histogram".to_string(),
            labels: vec!["label1".to_string()],
            unit: "ms".to_string(),
            source: "test_source".to_string(),
        };

        let json = serde_json::to_string(&metric).unwrap();
        let deserialized: CustomMetricDefinition = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.name, "test_metric");
        assert_eq!(deserialized.metric_type, MetricType::Histogram);
    }
}
