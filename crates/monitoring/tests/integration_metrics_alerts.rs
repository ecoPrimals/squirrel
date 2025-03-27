use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time;
use chrono::Utc;
use rand::Rng;

use squirrel_monitoring::metrics::{Metric, MetricType};
use squirrel_monitoring::metrics::performance::OperationType;
use squirrel_monitoring::alerts::{Alert, manager::AlertManager};
use squirrel_monitoring::alerts::types::AlertLevel;
use squirrel_monitoring::alerts::status::AlertType;
use squirrel_monitoring::alerts::status::AlertStatusType;
use squirrel_monitoring::alerts::config::AlertConfig;
use squirrel_core::error::Result;

// Import the test harness for creating mock data
use crate::test_harness::{AlertGenerator};

/// Test harness for metrics & alerts integration
struct MetricsAlertsTestHarness {
    /// Metric generator
    metric_generator: Arc<Mutex<crate::test_harness::TestMetricGenerator>>,
    /// Alert generator
    alert_generator: Arc<Mutex<AlertGenerator>>,
    /// Alert manager to test alert generation from metrics
    alert_manager: Arc<AlertManager>,
    /// Thresholds for metrics that trigger alerts
    metric_thresholds: HashMap<String, f64>,
}

impl MetricsAlertsTestHarness {
    /// Create a new test harness
    fn new() -> Self {
        let metric_generator = Arc::new(Mutex::new(crate::test_harness::TestMetricGenerator::new()));
        let alert_generator = Arc::new(Mutex::new(AlertGenerator::new()));
        
        // Create alert manager with default config
        let config = AlertConfig::default();
        let alert_manager = Arc::new(AlertManager::new(config));
        
        // Set up metric thresholds that will trigger alerts
        let mut metric_thresholds = HashMap::new();
        metric_thresholds.insert("cpu_usage".to_string(), 80.0); // Alert if CPU > 80%
        metric_thresholds.insert("memory_usage".to_string(), 90.0); // Alert if memory > 90%
        metric_thresholds.insert("disk_usage".to_string(), 85.0); // Alert if disk > 85%
        
        Self {
            metric_generator,
            alert_generator,
            alert_manager,
            metric_thresholds,
        }
    }
    
    /// Generate a metric-based alert if the metric exceeds its threshold
    fn generate_alert_from_metric(&self, metric: &Metric) -> Option<Alert> {
        if let Some(threshold) = self.metric_thresholds.get(&metric.name) {
            if metric.value > *threshold {
                // Determine alert level based on how much the threshold is exceeded
                let level = if metric.value > (*threshold * 1.2) {
                    AlertLevel::Critical
                } else if metric.value > (*threshold * 1.1) {
                    AlertLevel::Warning
                } else {
                    AlertLevel::Info
                };
                
                let now = Utc::now();
                
                // Create alert
                let alert = Alert {
                    id: format!("metric-alert-{}", uuid::Uuid::new_v4()),
                    alert_type: String::from("Performance"),
                    source: "metrics_monitor".to_string(),
                    message: format!("{} exceeded threshold: {:.2} > {:.2}", 
                        metric.name, metric.value, threshold),
                    level,
                    status: AlertStatusType::Active,
                    created_at: now,
                    occurred: now,
                    last_updated: now,
                    last_occurred: now,
                    count: 1,
                    details: HashMap::new(),
                };
                
                return Some(alert);
            }
        }
        
        None
    }
    
    /// Process metrics and generate alerts
    fn process_metrics(&self, metrics: &[Metric]) -> Vec<Alert> {
        let mut alerts = Vec::new();
        
        for metric in metrics {
            if let Some(alert) = self.generate_alert_from_metric(metric) {
                alerts.push(alert);
            }
        }
        
        alerts
    }
    
    /// Generate high-value metrics that would trigger alerts
    fn generate_high_value_metrics(&self) -> Vec<Metric> {
        let mut metrics = Vec::new();
        let mut generator = self.metric_generator.lock().unwrap();
        
        // Generate CPU usage above threshold
        let cpu_metric = Metric {
            name: "cpu_usage".to_string(),
            value: 95.0, // Well above the 80% threshold
            metric_type: MetricType::Gauge,
            labels: HashMap::new(),
            timestamp: Utc::now().timestamp(),
            operation_type: OperationType::Unknown,
        };
        
        // Generate memory usage above threshold
        let memory_metric = Metric {
            name: "memory_usage".to_string(),
            value: 92.0, // Above the 90% threshold
            metric_type: MetricType::Gauge,
            labels: HashMap::new(),
            timestamp: Utc::now().timestamp(),
            operation_type: OperationType::Unknown,
        };
        
        // Generate disk usage at threshold
        let disk_metric = Metric {
            name: "disk_usage".to_string(),
            value: 85.0, // At the 85% threshold
            metric_type: MetricType::Gauge,
            labels: HashMap::new(),
            timestamp: Utc::now().timestamp(),
            operation_type: OperationType::Unknown,
        };
        
        // Generate network usage below threshold
        let network_metric = Metric {
            name: "network_usage".to_string(),
            value: 60.0, // No threshold defined, so no alert
            metric_type: MetricType::Gauge,
            labels: HashMap::new(),
            timestamp: Utc::now().timestamp(),
            operation_type: OperationType::Unknown,
        };
        
        metrics.push(cpu_metric);
        metrics.push(memory_metric);
        metrics.push(disk_metric);
        metrics.push(network_metric);
        
        metrics
    }
}

/// Test that alerts are properly generated from metrics based on thresholds
#[tokio::test]
async fn test_alerts_from_metrics() {
    let harness = MetricsAlertsTestHarness::new();
    
    // Generate metrics that should trigger alerts
    let high_metrics = harness.generate_high_value_metrics();
    
    // Process metrics and generate alerts
    let alerts = harness.process_metrics(&high_metrics);
    
    // Verify alerts were generated correctly
    assert_eq!(alerts.len(), 2, "Expected exactly 2 alerts from 4 metrics");
    
    // Check for CPU alert
    let cpu_alert = alerts.iter().find(|a| a.message.contains("cpu_usage"));
    assert!(cpu_alert.is_some(), "Expected an alert for CPU usage");
    assert_eq!(cpu_alert.unwrap().level, AlertLevel::Warning, "Expected a Warning level for CPU");
    
    // Check for memory alert
    let memory_alert = alerts.iter().find(|a| a.message.contains("memory_usage"));
    assert!(memory_alert.is_some(), "Expected an alert for memory usage");
    assert_eq!(memory_alert.unwrap().level, AlertLevel::Info, "Expected an Info level for memory");
    
    // Ensure no disk alert (exactly at threshold)
    let disk_alert = alerts.iter().find(|a| a.message.contains("disk_usage"));
    assert!(disk_alert.is_none(), "Did not expect an alert for disk at threshold");
}

/// Test that batch processing of metrics works correctly
#[tokio::test]
async fn test_batch_metric_alert_processing() {
    let harness = MetricsAlertsTestHarness::new();
    let mut metrics = Vec::new();
    
    // Create 100 metrics with varying values
    for i in 0..100 {
        let metric_name = match i % 4 {
            0 => "cpu_usage",
            1 => "memory_usage",
            2 => "disk_usage",
            _ => "network_usage",
        };
        
        // Make some metrics exceed thresholds
        let value = if i % 5 == 0 {
            95.0 // Every 5th will be high
        } else {
            50.0 // Otherwise normal
        };
        
        let metric = Metric {
            name: metric_name.to_string(),
            value,
            metric_type: MetricType::Gauge,
            labels: HashMap::new(),
            timestamp: Utc::now().timestamp(),
            operation_type: OperationType::Unknown,
        };
        
        metrics.push(metric);
    }
    
    // Process all metrics
    let alerts = harness.process_metrics(&metrics);
    
    // We should have alerts for every 5th metric that's also cpu, memory, or disk
    let expected_alerts = (100 / 5) * 3 / 4; // 3 out of 4 metric types have thresholds
    assert!(alerts.len() >= expected_alerts, "Expected at least {} alerts", expected_alerts);
    
    // Verify alert properties
    for alert in &alerts {
        assert_eq!(alert.alert_type, "Performance", "Expected Performance alert type");
        assert_eq!(alert.status, AlertStatusType::Active, "Alert should be active initially");
        assert_eq!(alert.count, 1, "Alert should have occurred exactly once");
    }
}

/// Mock implementation of test harness file to avoid circular dependencies
mod test_harness {
    use std::collections::HashMap;
    use chrono::Utc;
    use uuid::Uuid;
    
    /// Metric generator for testing
    pub struct TestMetricGenerator {}
    
    impl TestMetricGenerator {
        pub fn new() -> Self {
            Self {}
        }
    }
    
    /// Alert generator for testing
    pub struct AlertGenerator {}
    
    impl AlertGenerator {
        pub fn new() -> Self {
            Self {}
        }
    }
}