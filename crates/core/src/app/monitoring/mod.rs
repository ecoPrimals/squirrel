//! Application monitoring service
//! 
//! This module provides functionality for:
//! - Service monitoring
//! - Resource tracking
//! - Performance monitoring
//! - Health checks

use std::sync::Arc;
use std::collections::HashMap;

use serde::{Serialize, Deserialize};

use crate::error::{Result, SquirrelError};
use crate::monitoring::{
    MonitoringConfig,
    MonitoringService,
    MonitoringServiceFactory,
    alerts::{Alert, AlertSeverity, AlertManager},
    health::HealthStatus,
    metrics::{Metric, MetricCollector, MetricType},
};

/// Application monitoring manager that provides a clean interface to the monitoring system
#[derive(Debug)]
pub struct AppMonitor {
    /// The underlying monitoring service
    service: Arc<MonitoringService>,
    /// Factory for creating monitoring services
    #[allow(dead_code)]
    factory: Arc<MonitoringServiceFactory>,
    /// Current configuration
    #[allow(dead_code)]
    config: MonitoringConfig,
}

/// Application monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppMonitorConfig {
    /// Base monitoring configuration
    pub monitoring: MonitoringConfig,
    /// Application-specific labels
    pub labels: HashMap<String, String>,
}

impl Default for AppMonitorConfig {
    fn default() -> Self {
        let mut labels = HashMap::new();
        labels.insert("component".to_string(), "app".to_string());
        
        Self {
            monitoring: MonitoringConfig::default(),
            labels,
        }
    }
}

impl AppMonitor {
    /// Create a new application monitor
    pub async fn new(config: AppMonitorConfig) -> Result<Self> {
        // Create a factory
        let factory = Arc::new(MonitoringServiceFactory::new(config.monitoring.clone()));
        
        // Create and start a service
        let service = factory.create_service();
        service.start().await?;
        
        Ok(Self { 
            service,
            factory,
            config: config.monitoring,
        })
    }

    /// Record a metric
    pub async fn record_metric(&self, name: &str, value: f64, metric_type: MetricType, labels: Option<HashMap<String, String>>) -> Result<()> {
        let metric = Metric::new(
            name.to_string(),
            value,
            metric_type,
            labels,
        );
        
        let collector = self.service.metric_collector();
        let record_future = collector.record_metric(metric);
        record_future.await.map_err(|e| SquirrelError::Metric(format!("Failed to record metric: {e}")))
    }

    /// Record an application event
    pub async fn record_event(&self, event_type: &str, message: &str, severity: AlertSeverity) -> Result<()> {
        let alert = Alert::new(
            event_type.to_string(),
            format!("App event: {event_type}"),
            severity,
            HashMap::new(),
            message.to_string(),
            "app".to_string()
        );
        
        let manager = self.service.alert_manager();
        let send_future = manager.send_alert(alert);
        send_future.await.map_err(|e| SquirrelError::Alert(format!("Failed to send alert: {e}")))
    }

    /// Get application health status
    pub async fn get_health(&self) -> Result<HealthStatus> {
        let future = self.service.get_system_status();
        futures::pin_mut!(future);
        future.await
    }

    /// Get application metrics
    pub async fn get_metrics(&self) -> Result<Vec<Metric>> {
        self.service.get_metrics().await
    }

    /// Get application alerts
    pub async fn get_alerts(&self) -> Result<Vec<Alert>> {
        self.service.get_alerts().await
    }

    /// Start the monitoring system
    pub async fn start(&self) -> Result<()> {
        self.service.start().await
    }

    /// Stop the monitoring system
    pub async fn stop(&self) -> Result<()> {
        self.service.stop().await
    }

    /// Get the underlying monitoring service
    pub fn service(&self) -> Arc<MonitoringService> {
        self.service.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::Duration;

    #[tokio::test]
    async fn test_app_monitor() {
        // Reset any previous monitoring state
        let _ = crate::monitoring::shutdown().await;
        
        let config = AppMonitorConfig::default();
        let monitor = AppMonitor::new(config).await.unwrap();

        // Test metric recording
        let mut labels = HashMap::new();
        labels.insert("test".to_string(), "true".to_string());
        assert!(monitor.record_metric("test_metric", 42.0, MetricType::Counter, Some(labels)).await.is_ok());

        // Test event recording
        assert!(monitor.record_event(
            "test_event",
            "Test event message",
            AlertSeverity::Info
        ).await.is_ok());

        // Allow time for metrics to be collected
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Test metric retrieval
        let metrics = monitor.get_metrics().await.unwrap();
        assert!(!metrics.is_empty());
        assert!(metrics.iter().any(|m| m.name == "test_metric"));

        // Test alert retrieval
        let alerts = monitor.get_alerts().await.unwrap();
        assert!(!alerts.is_empty());
        assert!(alerts.iter().any(|a| a.name == "test_event"));

        // Test health status
        let health = monitor.get_health().await.unwrap();
        assert_eq!(health, HealthStatus::Healthy);

        // Test shutdown
        assert!(monitor.stop().await.is_ok());
    }
} 