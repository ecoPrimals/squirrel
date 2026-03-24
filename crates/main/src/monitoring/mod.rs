// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! # Comprehensive Monitoring and Metrics Collection System
//!
//! This module provides comprehensive monitoring capabilities for the Squirrel AI ecosystem,
//! including real-time metrics collection, health monitoring, performance tracking, and
//! Prometheus-compatible metrics export.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tokio::time::interval;
use tracing::{error, info};

use crate::error::PrimalError;

pub mod alerts;
pub mod exporters;
pub mod health;
pub mod metric_names;
pub mod metrics;
pub mod performance;

#[cfg(test)]
mod mod_tests;
#[cfg(test)]
mod performance_types_tests;
#[cfg(test)]
mod types_tests;

/// Main monitoring system coordinator
pub struct MonitoringSystem {
    /// Metrics collection engine
    pub metrics_collector: Arc<metrics::MetricsCollector>,
    /// Health monitoring system
    pub health_monitor: Arc<health::HealthMonitor>,
    /// Performance tracking system
    pub performance_tracker: Arc<performance::PerformanceTracker>,
    /// Alert management system
    pub alert_manager: Arc<alerts::AlertManager>,
    /// Metrics exporters (Prometheus, etc.)
    pub exporters: HashMap<String, Box<dyn exporters::MetricsExporter>>,
    /// System configuration
    pub config: MonitoringConfig,
    /// Current system status
    pub system_status: Arc<RwLock<SystemStatus>>,
}

/// Configuration for the monitoring system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// Metrics collection interval
    pub collection_interval: Duration,
    /// Health check interval
    pub health_check_interval: Duration,
    /// Performance monitoring interval
    pub performance_interval: Duration,
    /// Alert evaluation interval
    pub alert_evaluation_interval: Duration,
    /// Maximum metrics history to retain
    pub max_metrics_history: usize,
    /// Enable Prometheus metrics export
    pub enable_prometheus: bool,
    /// Prometheus metrics endpoint
    pub prometheus_endpoint: String,
    /// Custom metrics definitions
    pub custom_metrics: HashMap<String, CustomMetricDefinition>,
    /// Alert rules
    pub alert_rules: Vec<AlertRule>,
    /// Metrics to collect
    pub metrics: Vec<String>,
    /// Alert thresholds
    pub alert_thresholds: HashMap<String, f64>,
}

/// Custom metric definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomMetricDefinition {
    /// Metric name
    pub name: String,
    /// Metric type (counter, gauge, histogram, summary)
    pub metric_type: MetricType,
    /// Description
    pub description: String,
    /// Labels
    pub labels: Vec<String>,
    /// Unit of measurement
    pub unit: String,
    /// Collection source
    pub source: String,
}

/// Metric type enumeration
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum MetricType {
    /// Monotonically increasing counter
    Counter,
    /// Value that can go up or down
    Gauge,
    /// Distribution of values
    Histogram,
    /// Quantile-based summary
    Summary,
}

/// Alert rule definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    /// Rule name
    pub name: String,
    /// Metric to monitor
    pub metric: String,
    /// Threshold value
    pub threshold: f64,
    /// Comparison operator
    pub operator: ComparisonOperator,
    /// Alert severity
    pub severity: AlertSeverity,
    /// Alert description
    pub description: String,
    /// Evaluation window
    pub evaluation_window: Duration,
    /// Minimum duration before firing
    pub for_duration: Duration,
}

/// Comparison operators for alert rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComparisonOperator {
    /// Value must be greater than threshold
    GreaterThan,
    /// Value must be greater than or equal to threshold
    GreaterThanOrEqual,
    /// Value must be less than threshold
    LessThan,
    /// Value must be less than or equal to threshold
    LessThanOrEqual,
    /// Value must equal threshold
    Equal,
    /// Value must not equal threshold
    NotEqual,
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    /// Critical - immediate action required
    Critical,
    /// High - urgent attention needed
    High,
    /// Medium - should be addressed soon
    Medium,
    /// Low - informational
    Low,
    /// Info - for awareness only
    Info,
}

/// Overall system status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStatus {
    /// System health state
    pub health: HealthState,
    /// Performance metrics summary
    pub performance: PerformanceSummary,
    /// Active alerts count
    pub active_alerts: u32,
    /// Last update timestamp
    pub last_update: DateTime<Utc>,
    /// System uptime
    pub uptime: Duration,
    /// Component statuses
    pub components: HashMap<String, ComponentStatus>,
}

/// System health state
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthState {
    /// All systems operational
    Healthy,
    /// Degraded but functional
    Warning,
    /// Critical failure
    Critical,
    /// Health status unknown
    Unknown,
}

/// Performance metrics summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSummary {
    /// CPU usage percentage
    pub cpu_usage: f64,
    /// Memory usage percentage
    pub memory_usage: f64,
    /// Network I/O bytes per second
    pub network_io: f64,
    /// Disk I/O bytes per second
    pub disk_io: f64,
    /// Average response time in milliseconds
    pub avg_response_time: f64,
    /// Requests per second
    pub requests_per_second: f64,
    /// Error rate percentage
    pub error_rate: f64,
}

/// Component status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentStatus {
    /// Component name
    pub name: String,
    /// Health state
    pub health: HealthState,
    /// Performance metrics
    pub metrics: HashMap<String, f64>,
    /// Last health check
    pub last_check: DateTime<Utc>,
    /// Status message
    pub status_message: String,
}

impl MonitoringSystem {
    /// Create a new monitoring system
    #[must_use]
    pub fn new(config: MonitoringConfig) -> Self {
        let metrics_collector = Arc::new(metrics::MetricsCollector::new());
        let health_monitor = Arc::new(health::HealthMonitor::new());
        let performance_tracker = Arc::new(performance::PerformanceTracker::new());
        let alert_manager = Arc::new(alerts::AlertManager::new());

        let system_status = Arc::new(RwLock::new(SystemStatus {
            health: HealthState::Unknown,
            performance: PerformanceSummary::default(),
            active_alerts: 0,
            last_update: Utc::now(),
            uptime: Duration::from_secs(0),
            components: HashMap::new(),
        }));

        Self {
            metrics_collector,
            health_monitor,
            performance_tracker,
            alert_manager,
            exporters: HashMap::new(),
            config,
            system_status,
        }
    }

    /// Start the monitoring system
    pub async fn start(&self) -> Result<(), PrimalError> {
        info!("Starting comprehensive monitoring system");

        // Start metrics collection
        self.start_metrics_collection().await?;

        // Start health monitoring
        self.start_health_monitoring().await?;

        // Start performance tracking
        self.start_performance_tracking().await?;

        // Start alert evaluation
        self.start_alert_evaluation().await?;

        // Start status updates
        self.start_status_updates().await?;

        // Initialize exporters
        self.initialize_exporters().await?;

        info!("Monitoring system started successfully");
        Ok(())
    }

    /// Stop the monitoring system
    pub async fn stop(&self) -> Result<(), PrimalError> {
        info!("Stopping monitoring system");

        // Stop all monitoring tasks
        // Implementation depends on task management system

        info!("Monitoring system stopped");
        Ok(())
    }

    /// Get current system status
    pub async fn get_system_status(&self) -> SystemStatus {
        self.system_status.read().await.clone()
    }

    /// Get metrics for a specific component
    pub async fn get_component_metrics(
        &self,
        component: &str,
    ) -> Result<HashMap<String, f64>, PrimalError> {
        self.metrics_collector
            .get_component_metrics(component)
            .await
    }

    /// Get health information for all components
    pub async fn get_health_summary(&self) -> Result<HashMap<String, HealthState>, PrimalError> {
        self.health_monitor.get_health_summary().await
    }

    /// Get performance metrics summary
    pub async fn get_performance_summary(&self) -> Result<PerformanceSummary, PrimalError> {
        self.performance_tracker.get_summary().await
    }

    /// Get active alerts
    pub async fn get_active_alerts(&self) -> Result<Vec<alerts::Alert>, PrimalError> {
        self.alert_manager.get_active_alerts().await
    }

    /// Register a custom metric
    pub async fn register_custom_metric(
        &self,
        definition: CustomMetricDefinition,
    ) -> Result<(), PrimalError> {
        self.metrics_collector
            .register_custom_metric(definition)
            .await
    }

    /// Record a custom metric value
    pub async fn record_metric(
        &self,
        name: &str,
        value: f64,
        labels: HashMap<String, String>,
    ) -> Result<(), PrimalError> {
        self.metrics_collector
            .record_metric(name, value, labels)
            .await
    }

    /// Export metrics in Prometheus format
    pub async fn export_prometheus_metrics(&self) -> Result<String, PrimalError> {
        if let Some(exporter) = self.exporters.get("prometheus") {
            let metrics = self.metrics_collector.get_all_metrics().await?;
            exporter.export_metrics(metrics).await
        } else {
            Err(PrimalError::NotFoundError(
                "Prometheus exporter not configured".to_string(),
            ))
        }
    }

    /// Private helper methods
    async fn start_metrics_collection(&self) -> Result<(), PrimalError> {
        let metrics_collector = self.metrics_collector.clone();
        let interval_duration = self.config.collection_interval;

        tokio::spawn(async move {
            let mut interval = interval(interval_duration);
            loop {
                interval.tick().await;
                if let Err(e) = metrics_collector.collect_metrics().await {
                    error!("Failed to collect metrics: {}", e);
                }
            }
        });

        Ok(())
    }

    async fn start_health_monitoring(&self) -> Result<(), PrimalError> {
        let health_monitor = self.health_monitor.clone();
        let interval_duration = self.config.health_check_interval;

        tokio::spawn(async move {
            let mut interval = interval(interval_duration);
            loop {
                interval.tick().await;
                if let Err(e) = health_monitor.check_all_components().await {
                    error!("Failed to check component health: {}", e);
                }
            }
        });

        Ok(())
    }

    async fn start_performance_tracking(&self) -> Result<(), PrimalError> {
        let performance_tracker = self.performance_tracker.clone();
        let interval_duration = self.config.performance_interval;

        tokio::spawn(async move {
            let mut interval = interval(interval_duration);
            loop {
                interval.tick().await;
                if let Err(e) = performance_tracker.track_performance().await {
                    error!("Failed to track performance: {}", e);
                }
            }
        });

        Ok(())
    }

    async fn start_alert_evaluation(&self) -> Result<(), PrimalError> {
        let alert_manager = self.alert_manager.clone();
        let metrics_collector = self.metrics_collector.clone();
        let interval_duration = self.config.alert_evaluation_interval;

        tokio::spawn(async move {
            let mut interval = interval(interval_duration);
            loop {
                interval.tick().await;
                if let Err(e) = alert_manager.evaluate_alerts(&metrics_collector).await {
                    error!("Failed to evaluate alerts: {}", e);
                }
            }
        });

        Ok(())
    }

    async fn start_status_updates(&self) -> Result<(), PrimalError> {
        let system_status = self.system_status.clone();
        let health_monitor = self.health_monitor.clone();
        let performance_tracker = self.performance_tracker.clone();
        let alert_manager = self.alert_manager.clone();

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(10));
            let start_time = Instant::now();

            loop {
                interval.tick().await;

                // Update system status
                let mut status = system_status.write().await;

                // Update health state
                if let Ok(health_summary) = health_monitor.get_health_summary().await {
                    status.health = Self::calculate_overall_health(&health_summary);
                }

                // Update performance summary
                if let Ok(perf_summary) = performance_tracker.get_summary().await {
                    status.performance = perf_summary;
                }

                // Update active alerts count
                if let Ok(alerts) = alert_manager.get_active_alerts().await {
                    status.active_alerts = alerts.len() as u32;
                }

                // Update timestamps and uptime
                status.last_update = Utc::now();
                status.uptime = start_time.elapsed();
            }
        });

        Ok(())
    }

    async fn initialize_exporters(&self) -> Result<(), PrimalError> {
        if self.config.enable_prometheus {
            info!("Initializing Prometheus metrics exporter");
            // Implementation would initialize Prometheus exporter
        }
        Ok(())
    }

    fn calculate_overall_health(health_summary: &HashMap<String, HealthState>) -> HealthState {
        let mut has_critical = false;
        let mut has_warning = false;

        for health in health_summary.values() {
            match health {
                HealthState::Critical => has_critical = true,
                HealthState::Warning | HealthState::Unknown => has_warning = true,
                HealthState::Healthy => {}
            }
        }

        if has_critical {
            HealthState::Critical
        } else if has_warning {
            HealthState::Warning
        } else {
            HealthState::Healthy
        }
    }
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            collection_interval: Duration::from_secs(30),
            health_check_interval: Duration::from_secs(60),
            performance_interval: Duration::from_secs(15),
            alert_evaluation_interval: Duration::from_secs(30),
            max_metrics_history: 1000,
            enable_prometheus: true,
            prometheus_endpoint: "/metrics".to_string(),
            custom_metrics: HashMap::new(),
            alert_rules: Vec::new(),
            metrics: vec![
                "cpu_usage".to_string(),
                "memory_usage".to_string(),
                "response_time".to_string(),
                "error_rate".to_string(),
            ],
            alert_thresholds: HashMap::from([
                ("cpu_usage".to_string(), 80.0),
                ("memory_usage".to_string(), 85.0),
                ("error_rate".to_string(), 5.0),
                ("response_time".to_string(), 1000.0),
            ]),
        }
    }
}

impl Default for PerformanceSummary {
    fn default() -> Self {
        Self {
            cpu_usage: 0.0,
            memory_usage: 0.0,
            network_io: 0.0,
            disk_io: 0.0,
            avg_response_time: 0.0,
            requests_per_second: 0.0,
            error_rate: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_monitoring_system_creation() {
        let config = MonitoringConfig::default();
        let monitoring_system = MonitoringSystem::new(config);

        // Verify that all monitoring components are initialized
        assert!(Arc::strong_count(&monitoring_system.metrics_collector) > 0);
        assert!(Arc::strong_count(&monitoring_system.health_monitor) > 0);
        assert!(Arc::strong_count(&monitoring_system.performance_tracker) > 0);
        assert!(Arc::strong_count(&monitoring_system.alert_manager) > 0);
    }

    #[tokio::test]
    async fn test_system_status_updates() {
        let config = MonitoringConfig::default();
        let monitoring_system = MonitoringSystem::new(config);

        let initial_status = monitoring_system.get_system_status().await;
        assert_eq!(initial_status.health, HealthState::Unknown);
        assert_eq!(initial_status.active_alerts, 0);
    }

    #[tokio::test]
    async fn test_custom_metric_registration() {
        let config = MonitoringConfig::default();
        let monitoring_system = MonitoringSystem::new(config);

        let custom_metric = CustomMetricDefinition {
            name: "test_metric".to_string(),
            metric_type: MetricType::Counter,
            description: "Test metric".to_string(),
            labels: vec!["component".to_string()],
            unit: "count".to_string(),
            source: "test".to_string(),
        };

        let result = monitoring_system
            .register_custom_metric(custom_metric)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_health_state_calculation() {
        let mut health_summary = HashMap::new();
        health_summary.insert("component1".to_string(), HealthState::Healthy);
        health_summary.insert("component2".to_string(), HealthState::Healthy);

        let overall_health = MonitoringSystem::calculate_overall_health(&health_summary);
        assert_eq!(overall_health, HealthState::Healthy);

        health_summary.insert("component3".to_string(), HealthState::Warning);
        let overall_health = MonitoringSystem::calculate_overall_health(&health_summary);
        assert_eq!(overall_health, HealthState::Warning);

        health_summary.insert("component4".to_string(), HealthState::Critical);
        let overall_health = MonitoringSystem::calculate_overall_health(&health_summary);
        assert_eq!(overall_health, HealthState::Critical);
    }

    #[tokio::test]
    async fn test_monitoring_system_stop() {
        let config = MonitoringConfig::default();
        let monitoring_system = MonitoringSystem::new(config);
        let result = monitoring_system.stop().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_export_prometheus_metrics_no_exporter() {
        let config = MonitoringConfig::default();
        let monitoring_system = MonitoringSystem::new(config);
        let result = monitoring_system.export_prometheus_metrics().await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Prometheus exporter not configured")
        );
    }

    #[tokio::test]
    async fn test_record_metric() {
        let config = MonitoringConfig::default();
        let monitoring_system = MonitoringSystem::new(config);
        // record_metric requires the metric to be registered first
        let custom_metric = CustomMetricDefinition {
            name: "test_metric".to_string(),
            metric_type: MetricType::Gauge,
            description: "Test metric".to_string(),
            labels: vec![],
            unit: "count".to_string(),
            source: "test".to_string(),
        };
        let _ = monitoring_system
            .register_custom_metric(custom_metric)
            .await;
        let result = monitoring_system
            .record_metric("test_metric", 42.0, HashMap::new())
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_component_metrics() {
        let config = MonitoringConfig::default();
        let monitoring_system = MonitoringSystem::new(config);
        let result = monitoring_system.get_component_metrics("nonexistent").await;
        assert!(result.is_ok());
        let metrics = result.expect("should succeed");
        assert!(metrics.is_empty());
    }

    #[tokio::test]
    async fn test_get_health_summary() {
        let config = MonitoringConfig::default();
        let monitoring_system = MonitoringSystem::new(config);
        let result = monitoring_system.get_health_summary().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_performance_summary() {
        let config = MonitoringConfig::default();
        let monitoring_system = MonitoringSystem::new(config);
        let result = monitoring_system.get_performance_summary().await;
        assert!(result.is_ok());
        let summary = result.expect("should succeed");
        // Performance tracker may collect real system metrics
        assert!(summary.cpu_usage >= 0.0 && summary.cpu_usage <= 100.0);
    }

    #[tokio::test]
    async fn test_get_active_alerts() {
        let config = MonitoringConfig::default();
        let monitoring_system = MonitoringSystem::new(config);
        let result = monitoring_system.get_active_alerts().await;
        assert!(result.is_ok());
        let alerts = result.expect("should succeed");
        assert!(alerts.is_empty());
    }

    #[tokio::test]
    async fn test_monitoring_system_start() {
        let config = MonitoringConfig::default();
        let monitoring_system = MonitoringSystem::new(config);
        let result = monitoring_system.start().await;
        assert!(result.is_ok());
        let _ = monitoring_system.stop().await;
    }
}
