// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Tests for monitoring system types and enums

use super::*;

// ============================================================================
// MONITORING CONFIG TESTS
// ============================================================================

#[test]
fn test_monitoring_config_default() {
    let config = MonitoringConfig::default();

    assert_eq!(config.collection_interval, Duration::from_secs(30));
    assert_eq!(config.health_check_interval, Duration::from_secs(60));
    assert_eq!(config.performance_interval, Duration::from_secs(15));
    assert_eq!(config.alert_evaluation_interval, Duration::from_secs(30));
    assert_eq!(config.max_metrics_history, 1000);
    assert!(config.enable_prometheus);
    assert_eq!(config.prometheus_endpoint, "/metrics");
    assert_eq!(config.metrics.len(), 4);
}

#[test]
fn test_monitoring_config_serialization() {
    let config = MonitoringConfig::default();
    let json = serde_json::to_string(&config).unwrap();
    let deserialized: MonitoringConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(config.enable_prometheus, deserialized.enable_prometheus);
    assert_eq!(config.max_metrics_history, deserialized.max_metrics_history);
}

// ============================================================================
// METRIC TYPE TESTS
// ============================================================================

#[test]
fn test_metric_type_variants() {
    let counter = MetricType::Counter;
    let gauge = MetricType::Gauge;
    let histogram = MetricType::Histogram;
    let summary = MetricType::Summary;

    assert!(matches!(counter, MetricType::Counter));
    assert!(matches!(gauge, MetricType::Gauge));
    assert!(matches!(histogram, MetricType::Histogram));
    assert!(matches!(summary, MetricType::Summary));
}

#[test]
fn test_metric_type_equality() {
    assert_eq!(MetricType::Counter, MetricType::Counter);
    assert_ne!(MetricType::Counter, MetricType::Gauge);
}

#[test]
fn test_metric_type_serialization() {
    let metric_type = MetricType::Counter;
    let json = serde_json::to_string(&metric_type).unwrap();
    let deserialized: MetricType = serde_json::from_str(&json).unwrap();

    assert_eq!(metric_type, deserialized);
}

#[test]
fn test_metric_type_clone() {
    let original = MetricType::Histogram;
    let cloned = original.clone();

    assert_eq!(original, cloned);
}

// ============================================================================
// CUSTOM METRIC DEFINITION TESTS
// ============================================================================

#[test]
fn test_custom_metric_definition() {
    let metric = CustomMetricDefinition {
        name: "request_count".to_string(),
        metric_type: MetricType::Counter,
        description: "Total HTTP requests".to_string(),
        labels: vec!["method".to_string(), "status".to_string()],
        unit: "count".to_string(),
        source: "http_server".to_string(),
    };

    assert_eq!(metric.name, "request_count");
    assert_eq!(metric.metric_type, MetricType::Counter);
    assert_eq!(metric.labels.len(), 2);
}

#[test]
fn test_custom_metric_definition_serialization() {
    let metric = CustomMetricDefinition {
        name: "test".to_string(),
        metric_type: MetricType::Gauge,
        description: "Test metric".to_string(),
        labels: vec![],
        unit: "bytes".to_string(),
        source: "test".to_string(),
    };

    let json = serde_json::to_string(&metric).unwrap();
    let deserialized: CustomMetricDefinition = serde_json::from_str(&json).unwrap();

    assert_eq!(metric.name, deserialized.name);
    assert_eq!(metric.metric_type, deserialized.metric_type);
}

// ============================================================================
// COMPARISON OPERATOR TESTS
// ============================================================================

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
fn test_comparison_operator_serialization() {
    let op = ComparisonOperator::GreaterThan;
    let json = serde_json::to_string(&op).unwrap();
    let deserialized: ComparisonOperator = serde_json::from_str(&json).unwrap();

    assert!(matches!(deserialized, ComparisonOperator::GreaterThan));
}

// ============================================================================
// ALERT SEVERITY TESTS
// ============================================================================

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
fn test_alert_severity_serialization() {
    let severity = AlertSeverity::Critical;
    let json = serde_json::to_string(&severity).unwrap();
    let deserialized: AlertSeverity = serde_json::from_str(&json).unwrap();

    assert!(matches!(deserialized, AlertSeverity::Critical));
}

// ============================================================================
// ALERT RULE TESTS
// ============================================================================

#[test]
fn test_alert_rule() {
    let rule = AlertRule {
        name: "high_cpu_usage".to_string(),
        metric: "cpu_usage".to_string(),
        threshold: 80.0,
        operator: ComparisonOperator::GreaterThan,
        severity: AlertSeverity::High,
        description: "CPU usage above 80%".to_string(),
        evaluation_window: Duration::from_secs(300),
        for_duration: Duration::from_secs(60),
    };

    assert_eq!(rule.name, "high_cpu_usage");
    assert_eq!(rule.threshold, 80.0);
    assert!(matches!(rule.severity, AlertSeverity::High));
}

#[test]
fn test_alert_rule_serialization() {
    let rule = AlertRule {
        name: "test_rule".to_string(),
        metric: "test_metric".to_string(),
        threshold: 100.0,
        operator: ComparisonOperator::GreaterThanOrEqual,
        severity: AlertSeverity::Medium,
        description: "Test rule".to_string(),
        evaluation_window: Duration::from_secs(60),
        for_duration: Duration::from_secs(30),
    };

    let json = serde_json::to_string(&rule).unwrap();
    let deserialized: AlertRule = serde_json::from_str(&json).unwrap();

    assert_eq!(rule.name, deserialized.name);
    assert_eq!(rule.threshold, deserialized.threshold);
}

// ============================================================================
// HEALTH STATE TESTS
// ============================================================================

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
fn test_health_state_serialization() {
    let state = HealthState::Healthy;
    let json = serde_json::to_string(&state).unwrap();
    let deserialized: HealthState = serde_json::from_str(&json).unwrap();

    assert_eq!(state, deserialized);
}

// ============================================================================
// PERFORMANCE SUMMARY TESTS
// ============================================================================

#[test]
fn test_performance_summary_default() {
    let summary = PerformanceSummary::default();

    assert_eq!(summary.cpu_usage, 0.0);
    assert_eq!(summary.memory_usage, 0.0);
    assert_eq!(summary.network_io, 0.0);
    assert_eq!(summary.disk_io, 0.0);
    assert_eq!(summary.avg_response_time, 0.0);
    assert_eq!(summary.requests_per_second, 0.0);
    assert_eq!(summary.error_rate, 0.0);
}

#[test]
fn test_performance_summary_custom() {
    let summary = PerformanceSummary {
        cpu_usage: 45.5,
        memory_usage: 60.2,
        network_io: 1024.0,
        disk_io: 512.0,
        avg_response_time: 150.0,
        requests_per_second: 100.0,
        error_rate: 0.5,
    };

    assert_eq!(summary.cpu_usage, 45.5);
    assert_eq!(summary.memory_usage, 60.2);
    assert_eq!(summary.requests_per_second, 100.0);
}

#[test]
fn test_performance_summary_serialization() {
    let summary = PerformanceSummary::default();
    let json = serde_json::to_string(&summary).unwrap();
    let deserialized: PerformanceSummary = serde_json::from_str(&json).unwrap();

    assert_eq!(summary.cpu_usage, deserialized.cpu_usage);
    assert_eq!(summary.error_rate, deserialized.error_rate);
}

// ============================================================================
// COMPONENT STATUS TESTS
// ============================================================================

#[test]
fn test_component_status() {
    let mut metrics = HashMap::new();
    metrics.insert("cpu".to_string(), 50.0);

    let status = ComponentStatus {
        name: "web_server".to_string(),
        health: HealthState::Healthy,
        metrics,
        last_check: Utc::now(),
        status_message: "Operating normally".to_string(),
    };

    assert_eq!(status.name, "web_server");
    assert_eq!(status.health, HealthState::Healthy);
    assert_eq!(status.metrics.len(), 1);
}

#[test]
fn test_component_status_serialization() {
    let status = ComponentStatus {
        name: "test_component".to_string(),
        health: HealthState::Warning,
        metrics: HashMap::new(),
        last_check: Utc::now(),
        status_message: "Test message".to_string(),
    };

    let json = serde_json::to_string(&status).unwrap();
    let deserialized: ComponentStatus = serde_json::from_str(&json).unwrap();

    assert_eq!(status.name, deserialized.name);
    assert_eq!(status.health, deserialized.health);
}

// ============================================================================
// SYSTEM STATUS TESTS
// ============================================================================

#[test]
fn test_system_status() {
    let status = SystemStatus {
        health: HealthState::Healthy,
        performance: PerformanceSummary::default(),
        active_alerts: 0,
        last_update: Utc::now(),
        uptime: Duration::from_secs(3600),
        components: HashMap::new(),
    };

    assert_eq!(status.health, HealthState::Healthy);
    assert_eq!(status.active_alerts, 0);
    assert_eq!(status.uptime, Duration::from_secs(3600));
}

#[test]
fn test_system_status_with_alerts() {
    let status = SystemStatus {
        health: HealthState::Warning,
        performance: PerformanceSummary::default(),
        active_alerts: 5,
        last_update: Utc::now(),
        uptime: Duration::from_secs(7200),
        components: HashMap::new(),
    };

    assert_eq!(status.health, HealthState::Warning);
    assert_eq!(status.active_alerts, 5);
}

#[test]
fn test_system_status_serialization() {
    let status = SystemStatus {
        health: HealthState::Critical,
        performance: PerformanceSummary::default(),
        active_alerts: 10,
        last_update: Utc::now(),
        uptime: Duration::from_secs(100),
        components: HashMap::new(),
    };

    let json = serde_json::to_string(&status).unwrap();
    let deserialized: SystemStatus = serde_json::from_str(&json).unwrap();

    assert_eq!(status.health, deserialized.health);
    assert_eq!(status.active_alerts, deserialized.active_alerts);
}

// ============================================================================
// INTEGRATION TESTS
// ============================================================================

#[test]
fn test_monitoring_config_custom() {
    let mut custom_metrics = HashMap::new();
    custom_metrics.insert(
        "custom_1".to_string(),
        CustomMetricDefinition {
            name: "custom_1".to_string(),
            metric_type: MetricType::Counter,
            description: "Custom metric 1".to_string(),
            labels: vec![],
            unit: "count".to_string(),
            source: "test".to_string(),
        },
    );

    let config = MonitoringConfig {
        collection_interval: Duration::from_secs(10),
        health_check_interval: Duration::from_secs(30),
        performance_interval: Duration::from_secs(5),
        alert_evaluation_interval: Duration::from_secs(15),
        max_metrics_history: 500,
        enable_prometheus: false,
        prometheus_endpoint: "/custom/metrics".to_string(),
        custom_metrics,
        alert_rules: vec![],
        metrics: vec!["cpu".to_string()],
        alert_thresholds: HashMap::new(),
    };

    assert_eq!(config.collection_interval, Duration::from_secs(10));
    assert!(!config.enable_prometheus);
    assert_eq!(config.custom_metrics.len(), 1);
}

#[test]
fn test_alert_rule_complex() {
    let rule = AlertRule {
        name: "complex_alert".to_string(),
        metric: "response_time".to_string(),
        threshold: 500.0,
        operator: ComparisonOperator::GreaterThanOrEqual,
        severity: AlertSeverity::Critical,
        description: "Response time exceeds 500ms for extended period".to_string(),
        evaluation_window: Duration::from_secs(600),
        for_duration: Duration::from_secs(120),
    };

    assert_eq!(rule.evaluation_window, Duration::from_secs(600));
    assert_eq!(rule.for_duration, Duration::from_secs(120));
}
