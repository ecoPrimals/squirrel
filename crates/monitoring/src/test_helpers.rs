use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::error::Result;
use crate::monitoring::metrics::{Metric, MetricType, DefaultMetricCollector, MetricCollector};
use crate::monitoring::health::{HealthStatus, HealthChecker};
use crate::monitoring::alerts::{Alert, AlertSeverity, AlertManager};
use crate::monitoring::{MonitoringService, MonitoringConfig, MonitoringIntervals, MonitoringServiceFactory};
use crate::monitoring::health::HealthConfig;
use crate::monitoring::metrics::MetricConfig;
use crate::monitoring::alerts::AlertConfig;
use crate::monitoring::network::NetworkConfig;

/// Configuration for test monitoring services
#[derive(Debug, Clone)]
pub struct TestMonitoringConfig {
    /// Base configuration for the monitoring service
    pub config: MonitoringConfig,
    /// Whether to start services automatically
    pub auto_start: bool,
}

impl Default for TestMonitoringConfig {
    fn default() -> Self {
        Self {
            config: MonitoringConfig {
                intervals: MonitoringIntervals {
                    health_check: 1,
                    metric_collection: 1,
                    network_monitoring: 1,
                },
                health: HealthConfig::default(),
                metrics: MetricConfig::default(),
                alerts: AlertConfig::default(),
                network: NetworkConfig::default(),
            },
            auto_start: false,
        }
    }
}

/// Creates a test monitoring service factory
pub fn create_test_factory(config: Option<TestMonitoringConfig>) -> MonitoringServiceFactory {
    let config = config.unwrap_or_default();
    MonitoringServiceFactory::with_config(config.config)
}

/// Creates a test monitoring service
pub async fn create_test_service(config: Option<TestMonitoringConfig>) -> Result<Arc<MonitoringService>> {
    let test_config = config.unwrap_or_default();
    let factory = create_test_factory(Some(test_config.clone()));
    let service = factory.create_service();
    
    if test_config.auto_start {
        service.start().await?;
    }
    
    Ok(service)
}

// Simple function to get a test metric
pub fn create_test_metric(name: &str, value: f64) -> Metric {
    Metric {
        name: name.to_string(),
        value,
        metric_type: MetricType::Gauge,
        labels: HashMap::new(),
        timestamp: 0,
    }
}

// Create a test collector
pub fn create_test_collector() -> impl MetricCollector {
    DefaultMetricCollector::default()
}

// Get a mock health status for testing
pub async fn get_status() -> Option<HealthStatus> {
    Some(HealthStatus::Healthy)
}

// Simple function to check if a health status is healthy
pub fn is_healthy(status: &HealthStatus) -> bool {
    matches!(status, HealthStatus::Healthy)
}

// Simple mock alert for testing
pub fn create_test_alert(name: &str, description: &str) -> Alert {
    Alert::new(
        name.to_string(),
        description.to_string(),
        AlertSeverity::Warning,
        HashMap::new(),
        "Test message".to_string(),
        "test".to_string(),
    )
}

// Simulate getting metrics from a service
pub async fn get_test_metrics() -> Result<Vec<Metric>> {
    let mut metrics = Vec::new();
    metrics.push(create_test_metric("cpu", 10.0));
    metrics.push(create_test_metric("memory", 50.0));
    Ok(metrics)
} 