// Allow certain linting issues that are too numerous to fix individually
#![allow(clippy::cast_possible_wrap)] // Allow u64 to i64 casts for timestamps
#![allow(clippy::missing_errors_doc)] // Temporarily allow missing error documentation
#![allow(clippy::manual_let_else)] // Allow manual let-else patterns
#![allow(clippy::unused_async)] // Allow unused async functions

//! Monitoring module for system health and performance tracking
//! 
//! This module provides functionality for:
//! - Health checks and status monitoring
//! - Resource usage tracking
//! - Performance metrics collection
//! - Alert management
//! - Monitoring agent coordination

/// Health monitoring and status checking functionality
pub mod health;
/// Metrics collection, processing, and export functionality
pub mod metrics;
/// Alert generation, management, and notification functionality
pub mod alerts;
/// Dashboard visualization and reporting functionality
pub mod dashboard;
/// Network monitoring and statistics functionality
pub mod network;

#[cfg(test)]
mod tests;

use std::{
    collections::HashMap,
    fmt::Debug,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use serde::{Serialize, Deserialize};
use thiserror::Error;
use tokio::sync::OnceCell;
use crate::error::{Result, SquirrelError};
use crate::monitoring::metrics::{MetricConfig, MetricCollector, DefaultMetricCollector, Metric};
use crate::monitoring::alerts::{AlertConfig, AlertManager, Alert, DefaultAlertManager};
use crate::monitoring::health::{HealthConfig, HealthChecker, HealthStatus, DefaultHealthChecker};
use crate::monitoring::network::{NetworkConfig, NetworkStats, NetworkMonitor};
use time::OffsetDateTime;
use tokio::time::sleep;
use serde_json;
use tokio::sync::mpsc;
use tracing::debug;

/// Converts a `SystemTime` to a Unix timestamp (seconds since Unix epoch)
#[must_use] pub fn system_time_to_timestamp(time: SystemTime) -> i64 {
    time.duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

/// Monitoring intervals configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringIntervals {
    /// Health check interval in seconds
    pub health_check: u64,
    /// Metric collection interval in seconds
    pub metric_collection: u64,
    /// Network monitoring interval in seconds
    pub network_monitoring: u64,
}

impl Default for MonitoringIntervals {
    fn default() -> Self {
        Self {
            health_check: 60,
            metric_collection: 30,
            network_monitoring: 60,
        }
    }
}

/// Configuration for the monitoring service
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MonitoringConfig {
    /// Monitoring intervals
    pub intervals: MonitoringIntervals,
    /// Health check configuration
    pub health: HealthConfig,
    /// Metric collection configuration
    pub metrics: MetricConfig,
    /// Alert configuration
    pub alerts: AlertConfig,
    /// Network monitoring configuration
    pub network: NetworkConfig,
}

/// Monitoring message types
#[derive(Debug, Clone)]
pub enum MonitoringMessage {
    /// Metrics have been collected
    MetricsCollected(Vec<Metric>),
    /// Alerts have been triggered
    AlertsTriggered(Vec<Alert>),
    /// Health status has changed
    HealthStatusChanged(HealthStatus),
    /// Network stats have been updated
    NetworkStatsUpdated(HashMap<String, NetworkStats>),
    /// Stopping the monitoring service
    Shutdown,
}

/// Monitoring service for managing system monitoring components
#[derive(Debug)]
pub struct MonitoringService {
    /// Service configuration
    pub config: MonitoringConfig,
    /// Health checker component
    pub health_checker: Arc<DefaultHealthChecker>,
    /// Metric collector component
    pub metric_collector: Arc<DefaultMetricCollector>,
    /// Alert manager component
    pub alert_manager: Arc<DefaultAlertManager>,
    /// Network monitor component
    pub network_monitor: Arc<NetworkMonitor>,
}

/// Factory for creating and managing monitoring service instances
#[derive(Debug, Clone)]
pub struct MonitoringServiceFactory {
    /// Default configuration to use when creating services
    pub default_config: MonitoringConfig,
}

/// Monitoring errors
#[derive(Debug, Error)]
pub enum MonitoringError {
    /// Errors related to health checks and status monitoring
    #[error("Health check error: {0}")]
    HealthError(String),
    /// Errors related to metric collection and processing
    #[error("Metric error: {0}")]
    MetricError(String),
    /// Errors related to alert generation and notification
    #[error("Alert error: {0}")]
    AlertError(String),
    /// Errors related to dashboard visualization and reporting
    #[error("Dashboard error: {0}")]
    DashboardError(String),
    /// Errors related to network monitoring and statistics
    #[error("Network error: {0}")]
    NetworkError(String),
    /// Errors related to communication protocols
    #[error("Protocol error: {0}")]
    ProtocolError(String),
    /// Errors related to tool usage and metrics
    #[error("Tool error: {0}")]
    ToolError(String),
    /// General system-level errors
    #[error("System error: {0}")]
    SystemError(String),
}

// Factory for global access (replaces direct service access)
static MONITORING_FACTORY: OnceCell<Arc<MonitoringServiceFactory>> = OnceCell::const_new();

impl MonitoringService {
    /// Create a new monitoring service
    #[must_use] pub fn new(config: MonitoringConfig) -> Self {
        let health_checker = Arc::new(DefaultHealthChecker::new());
        let metric_collector = Arc::new(DefaultMetricCollector::new());
        let alert_manager = Arc::new(DefaultAlertManager::new(config.alerts.clone()));
        let network_monitor = Arc::new(NetworkMonitor::new(config.network.clone()));

        Self {
            config,
            health_checker,
            metric_collector,
            alert_manager,
            network_monitor,
        }
    }

    /// Start the monitoring service
    ///
    /// Initializes and starts all monitoring components if enabled in the config.
    ///
    /// # Errors
    /// Returns an error if any of the following fail to start:
    /// * Health checker component
    /// * Metric collector component
    /// * Alert manager component
    /// * Network monitor component
    pub async fn start(&self) -> Result<()> {
        if !self.config.health.enabled && !self.config.metrics.enabled {
            return Ok(());
        }

        // Start the health checker
        self.health_checker.start().await?;
        
        // Start the metric collector
        self.metric_collector.start().await?;
        
        // Start the alert manager
        self.alert_manager.start().await?;
        
        // Start the network monitor
        self.network_monitor.start().await?;

        Ok(())
    }

    /// Stop the monitoring service
    ///
    /// Gracefully shuts down all monitoring components.
    ///
    /// # Errors
    /// Returns an error if any of the following fail to stop properly:
    /// * Health checker component
    /// * Metric collector component
    /// * Alert manager component
    /// * Network monitor component
    pub async fn stop(&self) -> Result<()> {
        // Stop the health checker
        self.health_checker.stop().await?;

        // Stop the metric collector
        self.metric_collector.stop().await?;

        // Stop the alert manager
        self.alert_manager.stop().await?;

        // Stop the network monitor
        self.network_monitor.stop().await?;

        Ok(())
    }

    /// Get the current system health status
    pub async fn get_health(&self) -> Result<HealthStatus> {
        self.health_checker.check_health().await
    }

    /// Get all collected metrics
    pub async fn get_metrics(&self) -> Result<Vec<Metric>> {
        self.metric_collector.collect_metrics().await
    }

    /// Get network statistics for all monitored interfaces
    pub async fn get_network_stats(&self) -> Result<HashMap<String, NetworkStats>> {
        self.network_monitor.get_stats().await
    }

    /// Get network statistics for a specific interface
    pub async fn get_interface_stats(&self, interface: &str) -> Result<Option<NetworkStats>> {
        self.network_monitor.get_interface_stats(interface).await
    }

    /// Get all alerts
    pub async fn get_alerts(&self) -> Result<Vec<Alert>> {
        self.alert_manager.get_alerts().await
    }

    /// Get the health checker component
    #[must_use] pub fn health_checker(&self) -> Arc<DefaultHealthChecker> {
        self.health_checker.clone()
    }

    /// Get the metric collector component
    #[must_use] pub fn metric_collector(&self) -> Arc<DefaultMetricCollector> {
        self.metric_collector.clone()
    }

    /// Get the alert manager component
    #[must_use] pub fn alert_manager(&self) -> Arc<DefaultAlertManager> {
        self.alert_manager.clone()
    }

    /// Get the network monitor component
    #[must_use] pub fn network_monitor(&self) -> Arc<NetworkMonitor> {
        self.network_monitor.clone()
    }

    /// Get the system status
    pub async fn get_system_status(&self) -> Result<HealthStatus> {
        self.health_checker.check_health().await
    }

    /// Runs the metrics collection and alert processing pipeline
    ///
    /// This method collects metrics from the metric collector and processes
    /// any alerts that may be triggered based on those metrics.
    ///
    /// # Returns
    /// Ok(()) if the metrics and alerts were processed successfully
    pub async fn run_and_process_metrics(&self) -> Result<()> {
        debug!("Running metric and alert handlers");

        // Process metrics from the metric collector
        let _metrics = self.metric_collector.collect_metrics().await?;
        
        // Process alerts based on collected metrics
        let _alerts = self.alert_manager.get_alerts().await?;
        
        // Handle metrics and alerts
        // self.alert_manager.handle_alerts(&alerts).await?;
        
        Ok(())
    }

    /// Collects metrics from the metric collector
    ///
    /// This method collects metrics and retrieves current alerts
    /// without processing them further.
    ///
    /// # Returns
    /// Ok(()) if the metrics were collected successfully
    pub async fn run_metrics(&self) -> Result<()> {
        // Collect metrics from the metric collector
        let _metrics = self.metric_collector.collect_metrics().await?;
        
        // Process alerts based on collected metrics
        let _alerts = self.alert_manager.get_alerts().await?;
        
        Ok(())
    }

    /// Runs a single metrics collection cycle and sends results through the provided channel
    ///
    /// # Arguments
    /// * `sender` - Channel to send monitoring messages through
    ///
    /// # Returns
    /// Ok(()) if the metrics collection was initiated successfully
    pub fn run_once(
        &self,
        sender: mpsc::Sender<MonitoringMessage>,
    ) -> Result<()> {
        let metric_collector = self.metric_collector.clone();
        let alert_manager = self.alert_manager.clone();
        
        tokio::spawn(async move {
            let metrics = match metric_collector.collect_metrics().await {
                Ok(m) => m,
                Err(_) => return,
            };

            // Check for alerts
            let _ = alert_manager.get_alerts().await;

            // Process metrics result
            let _ = sender.send(MonitoringMessage::MetricsCollected(metrics)).await;
        });

        Ok(())
    }

    /// Runs continuous metrics collection at the specified interval
    ///
    /// # Arguments
    /// * `interval` - Time between collection cycles
    /// * `sender` - Channel to send monitoring messages through
    ///
    /// # Returns
    /// Ok(()) if the continuous metrics collection was initiated successfully
    pub fn run_continuous(
        &self,
        interval: Duration,
        sender: mpsc::Sender<MonitoringMessage>,
    ) -> Result<()> {
        let interval_clone = interval;
        let metric_collector = self.metric_collector.clone();
        let alert_manager = self.alert_manager.clone();
        
        tokio::spawn(async move {
            loop {
                let metrics = match metric_collector.collect_metrics().await {
                    Ok(m) => m,
                    Err(_) => break,
                };

                // Check for alerts
                let _ = alert_manager.get_alerts().await;

                // Process metrics result
                let _ = sender.send(MonitoringMessage::MetricsCollected(metrics)).await;
                
                sleep(interval_clone).await;
            }
        });

        Ok(())
    }
}

impl MonitoringServiceFactory {
    /// Create a new factory with default configuration
    #[must_use] pub const fn new(default_config: MonitoringConfig) -> Self {
        Self { default_config }
    }
    
    /// Create a service using the default configuration
    #[must_use] pub fn create_service(&self) -> Arc<MonitoringService> {
        Arc::new(MonitoringService::new(self.default_config.clone()))
    }
    
    /// Create a service with a custom configuration
    #[must_use] pub fn create_service_with_config(&self, config: MonitoringConfig) -> Arc<MonitoringService> {
        Arc::new(MonitoringService::new(config))
    }
    
    /// Start a new service with the default configuration and return it
    pub async fn start_service(&self) -> Result<Arc<MonitoringService>> {
        let service = self.create_service();
        service.start().await?;
        Ok(service)
    }
    
    /// Start a new service with a custom configuration and return it
    pub async fn start_service_with_config(&self, config: MonitoringConfig) -> Result<Arc<MonitoringService>> {
        let service = self.create_service_with_config(config);
        service.start().await?;
        Ok(service)
    }
}

/// Initialize the monitoring system
///
/// This initializes the global monitoring factory singleton using the default configuration.
///
/// # Errors
/// Returns an error if the factory is already initialized
pub fn initialize_factory(config: Option<MonitoringConfig>) -> Result<Arc<MonitoringServiceFactory>> {
    let cfg = config.unwrap_or_default();
    
    // Create and initialize the factory
    let factory = Arc::new(MonitoringServiceFactory::new(cfg));
    
    // Set the factory in the global OnceCell
    MONITORING_FACTORY
        .set(factory.clone())
        .map_err(|_| SquirrelError::monitoring("Monitoring factory already initialized"))?;
    
    Ok(factory)
}

/// Get the global monitoring factory
///
/// # Returns
/// The global monitoring factory, if initialized
///
/// # Errors
/// Returns an error if the factory hasn't been initialized
pub fn get_factory() -> Result<Arc<MonitoringServiceFactory>> {
    MONITORING_FACTORY.get()
        .cloned()
        .ok_or_else(|| SquirrelError::monitoring("Monitoring factory not initialized"))
}

/// Initialize the monitoring system
///
/// # Errors
/// Returns an error if initialization fails
pub async fn initialize(config: MonitoringConfig) -> Result<Arc<MonitoringService>> {
    // Initialize component factories first
    health::initialize_factory(Some(config.health.clone()))?;
    metrics::performance::initialize_factory(None)?;
    dashboard::initialize_factory(None)?;
    
    // Initialize additional factories
    let _ = metrics::export::initialize_factory(None);
    let _ = metrics::resource::initialize_factory(None);
    
    // Initialize alert/notification factories
    let _ = alerts::initialize_factory(Some(config.alerts.clone()));
    let _ = network::initialize_factory(Some(config.network.clone()));
    
    // Initialize dashboard manager
    let _ = dashboard::initialize(None).await;
    
    // Create and initialize the factory
    let factory = initialize_factory(Some(config.clone()))?;
    
    // Create and start a service
    let service = factory.create_service_with_config(config);
    service.start().await?;
    
    Ok(service)
}

/// Start the monitoring service with the global factory
///
/// # Errors
/// Returns an error if the factory hasn't been initialized or if starting the service fails
pub async fn start_service() -> Result<Arc<MonitoringService>> {
    let factory = get_factory()?;
    factory.start_service().await
}

/// Initialize the monitoring service with default configuration
///
/// # Errors
/// Returns an error if the service is already initialized
pub fn initialize_service() -> Result<()> {
    // Initialize the factory with default config
    initialize_factory(None)?;
    
    Ok(())
}

/// Shutdown the monitoring system
pub async fn shutdown() -> Result<()> {
    if let Some(service) = MONITORING_FACTORY.get() {
        service.stop().await?;
        // Note: We don't reset the OnceCell here as it's not possible
        // with the current API. This is a limitation that needs addressing
        // in a separate PR.
    }
    Ok(())
}

/// Get protocol metrics
pub async fn get_protocol_metrics() -> Option<serde_json::Value> {
    let service = get_factory()?.create_service();
    let _metrics = service.metric_collector().collect_metrics().await.ok()?;
    
    let protocol_metrics = serde_json::json!({
        "messages_processed": 0,
        "message_latency": 0,
        "error_count": 0,
        "active_connections": 0,
        "queue_depth": 0
    });
    
    Some(protocol_metrics)
}

/// Get tool metrics
pub async fn get_tool_metrics(tool_name: &str) -> Option<serde_json::Value> {
    let service = get_factory()?.create_service();
    let _metrics = service.metric_collector().collect_metrics().await.ok()?;
    
    let tool_metrics = serde_json::json!({
        "name": tool_name,
        "usage_count": 0,
        "success_count": 0,
        "failure_count": 0,
        "average_duration": 0.0
    });
    
    Some(tool_metrics)
}

/// Get all tool metrics
pub async fn get_all_tool_metrics() -> Option<HashMap<String, serde_json::Value>> {
    let _service = get_factory()?.create_service();
    
    let mut result = HashMap::new();
    result.insert("default".to_string(), serde_json::json!({
        "name": "default",
        "usage_count": 0,
        "success_count": 0,
        "failure_count": 0,
        "average_duration": 0.0
    }));
    
    Some(result)
}

/// Wrapper for `OffsetDateTime` to implement From for `SystemTime`
pub struct TimeWrapper(pub OffsetDateTime);

impl From<TimeWrapper> for SystemTime {
    fn from(wrapper: TimeWrapper) -> Self {
        let dt = wrapper.0;
        let unix_timestamp = dt.unix_timestamp();
        let nanos = dt.nanosecond();
        
        if unix_timestamp >= 0 {
            UNIX_EPOCH + Duration::new(unix_timestamp.unsigned_abs(), nanos)
        } else {
            UNIX_EPOCH - Duration::new(unix_timestamp.unsigned_abs(), nanos)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_monitoring_service_factory_new() {
        let config = MonitoringConfig::default();
        let factory = MonitoringServiceFactory::new(config.clone());
        
        assert_eq!(factory.config, config);
    }
    
    #[test]
    fn test_monitoring_service_factory_create_service() {
        let config = MonitoringConfig::default();
        let factory = MonitoringServiceFactory::new(config);
        
        let service = factory.create_service();
        assert!(Arc::strong_count(&service) > 0);
    }
    
    #[test]
    fn test_monitoring_service_factory_create_service_with_config() {
        let default_config = MonitoringConfig::default();
        let factory = MonitoringServiceFactory::new(default_config);
        
        let custom_config = MonitoringConfig {
            logging: LoggingConfig {
                level: "trace".to_string(),
                ..Default::default()
            },
            ..Default::default()
        };
        
        let service = factory.create_service_with_config(custom_config.clone());
        assert_eq!(service.config, custom_config);
    }
    
    #[test]
    fn test_initialize_factory() {
        let result = initialize_factory(None);
        assert!(result.is_ok());
        
        // Second initialization should fail
        let result = initialize_factory(None);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_get_factory() {
        // Factory not initialized yet
        let result = get_factory();
        assert!(result.is_err());
        
        // Initialize factory and try again
        let _ = initialize_factory(None);
        let result = get_factory();
        assert!(result.is_ok());
    }
} 