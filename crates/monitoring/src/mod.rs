// Allow certain linting issues that are too numerous to fix individually
#![allow(clippy::cast_possible_wrap)] // Allow u64 to i64 casts for timestamps
#![allow(clippy::missing_errors_doc)] // Temporarily allow missing error documentation
#![allow(clippy::manual_let_else)] // Allow manual let-else patterns
#![allow(clippy::unused_async)] // Allow unused async functions

//! # Monitoring Module
//!
//! Comprehensive monitoring system for application health, metrics, and performance tracking.
//! 
//! This module provides functionality for:
//! - Health checks and status monitoring
//! - Resource usage tracking and metrics collection
//! - Performance evaluation and benchmarking
//! - Alert generation and notification
//! - Real-time dashboard visualization
//! - Network traffic analysis and statistics
//!
//! The monitoring system is designed with a modular architecture, allowing components
//! to be used independently or as part of a comprehensive monitoring solution.

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
/// Adapter functionality for dependency injection and testing
pub mod adapter;

#[cfg(test)]
mod tests;

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use thiserror::Error;
use time::OffsetDateTime;
use crate::error::Result;
use crate::monitoring::health::{HealthCheckerAdapter, create_checker_adapter, HealthStatus, HealthConfig, checker::HealthChecker};
use crate::monitoring::metrics::{MetricCollector, DefaultMetricCollector, Metric, MetricConfig};
use crate::monitoring::alerts::{Alert, AlertConfig, AlertManagerAdapter, AlertManager, AlertSeverity};
use crate::monitoring::network::{NetworkStats, NetworkConfig, NetworkMonitorAdapter};
use log::error;
use serde::{Serialize, Deserialize};
use tokio::sync::mpsc;
use tracing::debug;

/// Converts a `SystemTime` to a Unix timestamp (seconds since Unix epoch)
/// 
/// This utility function handles the conversion from Rust's `SystemTime` to 
/// a standard Unix timestamp (seconds since January 1, 1970 00:00:00 UTC).
/// 
/// # Arguments
/// 
/// * `time` - The `SystemTime` to convert
/// 
/// # Returns
/// 
/// The Unix timestamp as an i64
#[must_use] pub fn system_time_to_timestamp(time: SystemTime) -> i64 {
    time.duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

/// Monitoring intervals configuration
/// 
/// Defines the frequency at which different monitoring operations are performed.
/// Each interval is specified in seconds.
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
/// 
/// Holds configuration settings for all monitoring components,
/// allowing centralized configuration of the entire monitoring system.
/// Each component's configuration is stored in a separate field, enabling
/// fine-grained control over the monitoring behavior.
#[derive(Debug, Clone, Default)]
pub struct MonitoringConfig {
    /// Health check configuration
    pub health: HealthConfig,
    /// Metric collection configuration
    pub metrics: MetricConfig,
    /// Alert configuration
    pub alerts: AlertConfig,
    /// Network monitoring configuration
    pub network: NetworkConfig,
    /// Monitoring intervals for various components
    /// Defines frequency of health checks, metric collection, and network monitoring
    pub intervals: MonitoringIntervals,
}

/// Monitoring message types for internal communication
/// 
/// Used for async communication between monitoring components
/// via channels.
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

/// Events emitted by the monitoring service
/// 
/// External events that can be subscribed to by consumers
/// of the monitoring service.
#[derive(Debug, Clone)]
pub enum MonitoringEvent {
    /// Health status has changed
    HealthStatusChanged(HealthStatus),
    /// Network stats have been updated
    NetworkStatsUpdated(HashMap<String, NetworkStats>),
    /// Stopping the monitoring service
    Shutdown,
}

/// Core monitoring service implementation
/// 
/// Central service that coordinates all monitoring components
/// and provides a unified interface for monitoring operations.
#[derive(Debug)]
pub struct MonitoringService {
    /// Service configuration
    pub config: MonitoringConfig,
    /// Health checker component
    pub health_checker: Arc<HealthCheckerAdapter>,
    /// Metric collector component
    pub metric_collector: Arc<DefaultMetricCollector>,
    /// Alert manager component
    pub alert_manager: Arc<AlertManagerAdapter>,
    /// Network monitor component
    pub network_monitor: Arc<NetworkMonitorAdapter>,
}

/// Factory for creating monitoring service instances
/// 
/// Provides a flexible way to create and configure monitoring services
/// with various component implementations.
#[derive(Debug)]
pub struct MonitoringServiceFactory<N: alerts::NotificationManagerTrait + 'static = ()> {
    /// Default configuration to use when creating services
    pub default_config: MonitoringConfig,
    /// Factory for creating health checker components
    health_factory: Option<Arc<health::HealthCheckerFactory>>,
    /// Factory for creating metric collector components
    metric_factory: Option<Arc<metrics::MetricCollectorFactory>>,
    /// Factory for creating alert manager components
    alert_factory: Option<Arc<alerts::AlertManagerFactory<N>>>,
    /// Factory for creating network monitor components
    network_factory: Option<Arc<network::NetworkMonitorFactory>>,
}

/// Monitoring errors
/// 
/// Comprehensive error type for all monitoring-related errors
/// with specific variants for different monitoring components.
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

impl MonitoringService {
    /// Create a new monitoring service with default implementations
    /// 
    /// Creates a monitoring service using the provided configuration
    /// and default implementations for all components.
    #[must_use] pub fn new(config: MonitoringConfig) -> Self {
        // Create components with adapters for DI
        let health_checker = create_checker_adapter();
        let metric_collector = Arc::new(DefaultMetricCollector::new());
        let alert_manager = alerts::create_manager_adapter();
        let network_monitor = network::create_monitor_adapter();

        Self {
            config,
            health_checker,
            metric_collector,
            alert_manager,
            network_monitor
        }
    }
    
    /// Create a new monitoring service with explicit dependencies
    /// 
    /// This constructor allows explicit dependency injection for all components,
    /// providing maximum flexibility for testing and custom implementations.
    #[must_use] pub fn with_dependencies(
        config: MonitoringConfig,
        health_checker: Arc<HealthCheckerAdapter>,
        metric_collector: Arc<DefaultMetricCollector>,
        alert_manager: Arc<AlertManagerAdapter>,
        network_monitor: Arc<NetworkMonitorAdapter>,
    ) -> Self {
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
    /// Initializes and starts all monitoring components,
    /// preparing them to collect metrics, check health, etc.
    ///
    /// # Errors
    /// Returns an error if any component fails to start
    pub async fn start(&self) -> Result<()> {
        // Start all monitoring components
        self.health_checker.start().await?;
        self.metric_collector.start().await?;
        self.alert_manager.start().await?;
        self.network_monitor.start().await?;
        Ok(())
    }

    /// Stop the monitoring service
    ///
    /// Stops all monitoring components gracefully,
    /// ensuring any in-progress operations are completed.
    ///
    /// # Errors
    /// Returns an error if any component fails to stop
    pub async fn stop(&self) -> Result<()> {
        // Stop all monitoring components
        self.health_checker.stop().await?;
        self.metric_collector.stop().await?;
        self.alert_manager.stop().await?;
        self.network_monitor.stop().await?;
        Ok(())
    }

    /// Retrieves the current health status of the system
    ///
    /// # Errors
    /// Returns an error if querying health status fails
    pub async fn health_status(&self) -> Result<HealthStatus> {
        self.health_checker.check_health().await
    }

    /// Get the health checker component
    ///
    /// Provides access to the health checker for more detailed operations.
    #[must_use] pub fn health_checker(&self) -> Arc<HealthCheckerAdapter> {
        self.health_checker.clone()
    }

    /// Get the metric collector component
    ///
    /// Provides access to the metric collector for more detailed operations.
    #[must_use] pub fn metric_collector(&self) -> Arc<DefaultMetricCollector> {
        self.metric_collector.clone()
    }

    /// Get the alert manager component
    ///
    /// Provides access to the alert manager for more detailed operations.
    #[must_use] pub fn alert_manager(&self) -> Arc<AlertManagerAdapter> {
        self.alert_manager.clone()
    }

    /// Get the network monitor component
    ///
    /// Provides access to the network monitor for more detailed operations.
    #[must_use] pub fn network_monitor(&self) -> Arc<NetworkMonitorAdapter> {
        self.network_monitor.clone()
    }

    /// Get all collected metrics
    ///
    /// Returns all metrics that have been collected by the metric collector.
    ///
    /// # Errors
    /// Returns an error if retrieving metrics fails
    pub async fn get_metrics(&self) -> Result<Vec<Metric>> {
        self.metric_collector.get_metrics().await
    }

    /// Get all network statistics
    ///
    /// Returns network statistics for all available network interfaces.
    ///
    /// # Errors
    /// Returns an error if retrieving network stats fails
    pub async fn get_network_stats(&self) -> Result<HashMap<String, NetworkStats>> {
        self.network_monitor.get_stats().await
    }

    /// Get statistics for a specific network interface
    ///
    /// Returns network statistics for the specified interface if available.
    ///
    /// # Errors
    /// Returns an error if retrieving interface stats fails
    pub async fn get_interface_stats(&self, interface: &str) -> Result<Option<NetworkStats>> {
        self.network_monitor.get_interface_stats(interface).await
    }

    /// Get all active alerts
    ///
    /// Returns all currently active alerts managed by the alert manager.
    ///
    /// # Errors
    /// Returns an error if retrieving alerts fails
    pub async fn get_alerts(&self) -> Result<Vec<Alert>> {
        self.alert_manager.get_alerts().await
    }

    /// Run and process metrics in a single operation
    ///
    /// Collects metrics and processes them, potentially triggering alerts
    /// if conditions are met.
    ///
    /// # Errors
    /// Returns an error if collecting or processing metrics fails
    pub async fn run_and_process_metrics(&self) -> Result<()> {
        // Collect metrics
        let metrics = self.metric_collector.collect_metrics().await?;
        
        // Process metrics for potential alerts
        for metric in &metrics {
            if metric.should_alert() {
                // Generate alerts for metrics exceeding thresholds
                let alert = Alert::new(
                    format!("Metric Alert: {}", metric.name),
                    format!("Metric {} exceeded threshold: {}", metric.name, metric.value),
                    AlertSeverity::Medium, // Default medium severity
                    HashMap::new(), // No labels for now
                    format!("Value: {}", metric.value),
                    "metrics".to_string(),
                );
                self.alert_manager.add_alert(alert).await?;
            }
        }
        
        Ok(())
    }

    /// Run metric collection
    ///
    /// Collects metrics but does not process them for alerts.
    ///
    /// # Errors
    /// Returns an error if collecting metrics fails
    pub async fn run_metrics(&self) -> Result<()> {
        let metrics = self.metric_collector.collect_metrics().await?;
        debug!("Collected {} metrics", metrics.len());
        Ok(())
    }

    /// Run metrics collection once and send results through the channel
    ///
    /// This function collects metrics from all registered sources and sends the results
    /// through the provided channel.
    pub fn run_once(
        &self,
        sender: &mpsc::Sender<MonitoringMessage>,
    ) -> Result<()> {
        // Get tokio runtime for async tasks
        let rt = tokio::runtime::Handle::current();
        
        // Spawn health check task
        let health_checker = self.health_checker.clone();
        let health_sender = sender.clone();
        rt.spawn(async move {
            match health_checker.check_health().await {
                Ok(status) => {
                    if let Err(e) = health_sender.send(MonitoringMessage::HealthStatusChanged(status)).await {
                        error!("Failed to send health status: {}", e);
                    }
                },
                Err(e) => error!("Health check failed: {}", e),
            }
        });
        
        // Spawn metrics collection task
        let metric_collector = self.metric_collector.clone();
        let metrics_sender = sender.clone();
        rt.spawn(async move {
            match metric_collector.collect_metrics().await {
                Ok(metrics) => {
                    if let Err(e) = metrics_sender.send(MonitoringMessage::MetricsCollected(metrics)).await {
                        error!("Failed to send metrics: {}", e);
                    }
                },
                Err(e) => error!("Metrics collection failed: {}", e),
            }
        });
        
        Ok(())
    }

    /// Run continuous monitoring
    ///
    /// Starts a background task that performs monitoring cycles
    /// at the specified interval and sends results through the provided channel.
    ///
    /// # Errors
    /// Returns an error if setting up continuous monitoring fails
    pub fn run_continuous(
        &self,
        interval: Duration,
        sender: mpsc::Sender<MonitoringMessage>,
    ) -> Result<()> {
        // Get tokio runtime for async tasks
        let rt = tokio::runtime::Handle::current();
        
        // Clone components for async task
        let health_checker = self.health_checker.clone();
        let metric_collector = self.metric_collector.clone();
        let _network_monitor = self.network_monitor.clone();
        
        // Spawn continuous monitoring task
        rt.spawn(async move {
            loop {
                // Check health
                match health_checker.check_health().await {
                    Ok(status) => {
                        if let Err(e) = sender.send(MonitoringMessage::HealthStatusChanged(status)).await {
                            error!("Failed to send health status: {}", e);
                            break;
                        }
                    },
                    Err(e) => error!("Health check failed: {}", e),
                }
                
                // Collect metrics
                match metric_collector.collect_metrics().await {
                    Ok(metrics) => {
                        if let Err(e) = sender.send(MonitoringMessage::MetricsCollected(metrics)).await {
                            error!("Failed to send metrics: {}", e);
                            break;
                        }
                    },
                    Err(e) => error!("Metrics collection failed: {}", e),
                }
                
                // Wait for next interval
                tokio::time::sleep(interval).await;
            }
        });
        
        Ok(())
    }

    /// Record a metric with the given name and value
    ///
    /// Convenience method to record a metric directly through the service
    ///
    /// # Arguments
    /// * `name` - The name of the metric
    /// * `value` - The value to record
    ///
    /// # Errors
    /// Returns an error if recording the metric fails
    pub async fn record_metric(&self, name: &str, value: f64) -> Result<()> {
        let metric = metrics::Metric::new(
            name.to_string(),
            value,
            metrics::MetricType::Gauge, // Default to gauge type
            std::collections::HashMap::new(),
        );
        self.metric_collector.record_metric(metric).await
    }

    /// Add an alert to the alert manager
    ///
    /// Convenience method to add an alert directly through the service
    ///
    /// # Arguments
    /// * `alert` - The alert to add
    ///
    /// # Errors
    /// Returns an error if adding the alert fails
    pub async fn add_alert(&self, alert: Alert) -> Result<()> {
        self.alert_manager.add_alert(alert).await
    }

    /// Send an alert to the alert manager
    ///
    /// This is an alias for `add_alert` for backward compatibility
    pub async fn send_alert(&self, alert: Alert) -> Result<()> {
        self.add_alert(alert).await
    }
}

impl<N: alerts::NotificationManagerTrait + Send + Sync + std::fmt::Debug + 'static> MonitoringServiceFactory<N> {
    /// Create a new factory with default settings
    /// 
    /// Initializes a monitoring service factory with default configuration and no
    /// specialized component factories. This is the simplest way to create a factory
    /// when you don't need custom component implementations.
    #[must_use]
    pub fn new() -> Self {
        Self {
            default_config: MonitoringConfig::default(),
            health_factory: None,
            metric_factory: None,
            alert_factory: None,
            network_factory: None,
        }
    }

    /// Create a new factory with the specified configuration
    /// 
    /// Initializes a monitoring service factory with custom configuration but default
    /// component factories. This allows customizing monitoring behavior while using
    /// standard component implementations.
    /// 
    /// # Arguments
    /// * `config` - Custom configuration to use for created services
    #[must_use]
    pub fn with_config(config: MonitoringConfig) -> Self {
        Self {
            default_config: config,
            health_factory: None,
            metric_factory: None,
            alert_factory: None,
            network_factory: None,
        }
    }

    /// Creates a new factory with all component factories
    ///
    /// Initializes a fully customized monitoring service factory with specialized
    /// factories for each monitoring component. This provides maximum flexibility
    /// for creating services with custom implementations.
    ///
    /// # Arguments
    /// * `config` - Configuration to use for created services
    /// * `health_factory` - Factory for creating custom health checkers
    /// * `metric_factory` - Factory for creating custom metric collectors
    /// * `alert_factory` - Factory for creating custom alert managers
    /// * `network_factory` - Factory for creating custom network monitors
    ///
    /// # Returns
    /// A new factory instance with all component factories set
    #[must_use]
    pub fn with_factories(
        config: MonitoringConfig,
        health_factory: Arc<health::HealthCheckerFactory>,
        metric_factory: Arc<metrics::MetricCollectorFactory>,
        alert_factory: Arc<alerts::AlertManagerFactory<N>>,
        network_factory: Arc<network::NetworkMonitorFactory>,
    ) -> Self {
        Self {
            default_config: config,
            health_factory: Some(health_factory),
            metric_factory: Some(metric_factory),
            alert_factory: Some(alert_factory),
            network_factory: Some(network_factory),
        }
    }

    /// Creates a monitoring service with the default configuration
    ///
    /// Creates and returns a new monitoring service using the factory's default
    /// configuration and component implementations.
    ///
    /// # Returns
    /// A new monitoring service with default configuration wrapped in an Arc
    #[must_use]
    pub fn create_service(&self) -> Arc<MonitoringService> {
        self.create_service_with_config(self.default_config.clone())
    }

    /// Creates a monitoring service with a custom configuration
    ///
    /// Creates and returns a new monitoring service using the provided configuration
    /// but with default component implementations.
    ///
    /// # Arguments
    /// * `config` - Configuration to use for the service
    ///
    /// # Returns
    /// A new monitoring service with the specified configuration wrapped in an Arc
    #[must_use]
    pub fn create_service_with_config(&self, config: MonitoringConfig) -> Arc<MonitoringService> {
        // Create components with adapters
        let health_checker = create_checker_adapter();
        let metric_collector = Arc::new(DefaultMetricCollector::new());
        let alert_manager = alerts::create_manager_adapter();
        let network_monitor = network::create_monitor_adapter();

        Arc::new(MonitoringService {
            config,
            health_checker,
            metric_collector,
            alert_manager,
            network_monitor
        })
    }

    /// Creates a monitoring service with explicit dependencies
    ///
    /// Creates and returns a new monitoring service using the provided configuration
    /// and explicitly provided component implementations. This provides maximum control
    /// over the service's behavior and is primarily used for testing.
    ///
    /// # Arguments
    /// * `config` - Configuration to use for the service
    /// * `health_checker` - Health checker component to use
    /// * `metric_collector` - Metric collector component to use
    /// * `alert_manager` - Alert manager component to use
    /// * `network_monitor` - Network monitor component to use
    ///
    /// # Returns
    /// A new monitoring service with the specified dependencies wrapped in an Arc
    #[must_use]
    pub fn create_service_with_dependencies(
        &self,
        config: MonitoringConfig,
        health_checker: Arc<HealthCheckerAdapter>,
        metric_collector: Arc<DefaultMetricCollector>,
        alert_manager: Arc<AlertManagerAdapter>,
        network_monitor: Arc<NetworkMonitorAdapter>,
    ) -> Arc<MonitoringService> {
        Arc::new(MonitoringService {
            config,
            health_checker,
            metric_collector,
            alert_manager,
            network_monitor
        })
    }

    /// Creates a monitoring service using adapter pattern for ongoing transition
    ///
    /// Creates and returns a new monitoring service using adapters for all components.
    /// This method supports the transition to a fully dependency-injected architecture
    /// and provides a bridge between old and new implementation approaches.
    ///
    /// # Returns
    /// A new monitoring service with components created from adapters wrapped in an Arc
    #[must_use]
    pub fn create_service_with_adapters(&self) -> Arc<MonitoringService> {
        // Create components with adapters
        let health_checker = create_checker_adapter();
        let metric_collector = Arc::new(DefaultMetricCollector::new());
        let alert_manager = alerts::create_manager_adapter();
        let network_monitor = network::create_monitor_adapter();

        Arc::new(MonitoringService {
            config: self.default_config.clone(),
            health_checker,
            metric_collector,
            alert_manager,
            network_monitor
        })
    }

    /// Starts a new service with the default configuration
    ///
    /// Creates a new monitoring service with the factory's default configuration,
    /// starts all monitoring components, and returns the started service.
    ///
    /// # Returns
    /// A Result containing the started service if successful, or an error if any
    /// component fails to start
    ///
    /// # Errors
    /// Returns an error if any monitoring component fails to start
    pub async fn start_service(&self) -> Result<Arc<MonitoringService>> {
        let service = self.create_service();
        service.start().await?;
        Ok(service)
    }

    /// Starts a new service with a custom configuration
    ///
    /// Creates a new monitoring service with the provided configuration,
    /// starts all monitoring components, and returns the started service.
    ///
    /// # Arguments
    /// * `config` - Configuration to use for the service
    ///
    /// # Returns
    /// A Result containing the started service if successful, or an error if any
    /// component fails to start
    ///
    /// # Errors
    /// Returns an error if any monitoring component fails to start
    pub async fn start_service_with_config(&self, config: MonitoringConfig) -> Result<Arc<MonitoringService>> {
        let service = self.create_service_with_config(config);
        service.start().await?;
        Ok(service)
    }
}

impl<N: alerts::NotificationManagerTrait + 'static> Clone for MonitoringServiceFactory<N> {
    fn clone(&self) -> Self {
        Self {
            default_config: self.default_config.clone(),
            health_factory: self.health_factory.clone(),
            metric_factory: self.metric_factory.clone(),
            alert_factory: self.alert_factory.clone(),
            network_factory: self.network_factory.clone(),
        }
    }
}

impl<N: alerts::NotificationManagerTrait + 'static> Default for MonitoringServiceFactory<N> {
    fn default() -> Self {
        Self::new()
    }
}

/// Wrapper struct for time conversion between `OffsetDateTime` and `SystemTime`
/// 
/// This struct provides conversion utilities between the time crate's time types
/// and the standard library's time types.
#[derive(Debug, Clone)]
pub struct TimeWrapper(pub OffsetDateTime);

/// The wrapped value is an `OffsetDateTime` that can be converted to `SystemTime`.
impl From<TimeWrapper> for SystemTime {
    /// Converts a `TimeWrapper` to `SystemTime`
    /// 
    /// This conversion enables interoperability between the time crate's `OffsetDateTime` 
    /// and the standard library's `SystemTime`, making it easier to work with
    /// both APIs in the monitoring system.
    #[must_use]
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