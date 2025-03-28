//! Monitoring and metrics system for Squirrel
//!
//! This crate provides functionality for monitoring system health,
//! collecting metrics, and generating alerts.

#![allow(dead_code)] // Temporarily allow dead code during migration

use std::sync::Arc;
use thiserror::Error;
use serde::{Serialize, Deserialize};
use std::fmt::Debug;
use squirrel_core::error::{Result, SquirrelError};
use tracing::error;

/// Module for alert functionality
pub mod alerts;

/// Module for health checking
pub mod health;

/// Module for metrics collection and reporting
///
/// # Examples
///
/// ```rust
/// use squirrel_monitoring::metrics::{Metric, MetricType, DefaultMetricCollector};
/// use std::collections::HashMap;
///
/// # async fn example() -> squirrel_core::error::Result<()> {
/// let mut collector = DefaultMetricCollector::new();
/// collector.initialize().await?;
///
/// let metric = Metric::new(
///     "request_count".to_string(),
///     1.0,
///     MetricType::Counter,
///     HashMap::new(),
/// );
///
/// // Record the metric using the collector's method
/// collector.record_metric(metric).await?;
/// # Ok(())
/// # }
/// ```
pub mod metrics;

/// Module for network monitoring
pub mod network;

/// Module for monitoring plugins
pub mod plugins;

/// Module for the Monitoring API
/// 
/// This module provides a clean API for accessing monitoring data from other crates.
pub mod api;

/// Module for adapters to connect monitoring to other systems
pub mod adapter;

#[cfg(feature = "analytics")]
/// Module for analytics
pub mod analytics;

#[cfg(feature = "websocket")]
/// Module for websocket (deprecated - use web crate instead)
/// 
/// Note: This module is being migrated to the web crate.
/// New code should use the monitoring API in the `api` module
/// and implement WebSocket functionality in the web crate.
pub mod websocket;

/// Configuration for the monitoring system
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct MonitoringConfig {
    /// Alert configuration
    pub alert_config: alerts::config::AlertConfig,
    /// Metrics configuration
    pub metrics_config: metrics::MetricConfig,
    /// Health check configuration
    pub health_config: health::HealthConfig,
    /// Monitoring intervals in seconds
    pub intervals: MonitoringIntervals,
    // Network config is commented out as NetworkConfig isn't implemented yet
    // pub network_config: network::NetworkConfig,
}

/// Interval settings for different monitoring components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringIntervals {
    /// Health check interval in seconds
    pub health_check_interval: u64,
    /// Metrics collection interval in seconds
    pub metrics_collection_interval: u64,
    /// Alert processing interval in seconds
    pub alert_processing_interval: u64,
    /// Network stats collection interval in seconds
    pub network_stats_interval: u64,
}

impl Default for MonitoringIntervals {
    fn default() -> Self {
        Self {
            health_check_interval: 60,
            metrics_collection_interval: 15,
            alert_processing_interval: 30,
            network_stats_interval: 60,
        }
    }
}

/// Monitoring service for managing all monitoring components
/// Common types used throughout the monitoring system
pub use health::ComponentHealth;
pub use metrics::Metric;

/// Factory for creating monitoring services
#[async_trait::async_trait]
pub trait MonitoringServiceFactory: Send + Sync {
    /// Create a new monitoring service with the given configuration
    async fn create_service(&self, config: MonitoringConfig) -> Result<Arc<dyn MonitoringService>>;
}

/// Monitoring service interface
#[async_trait::async_trait]
pub trait MonitoringService: Send + Sync {
    /// Start the monitoring service
    async fn start(&self) -> Result<()>;
    
    /// Stop the monitoring service
    async fn stop(&self) -> Result<()>;
    
    /// Get the current status of the monitoring service
    async fn status(&self) -> Result<MonitoringStatus>;

    /// Get a reference to the monitoring API
    fn get_api(&self) -> &dyn api::MonitoringAPI;
}

/// Status of the monitoring service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringStatus {
    /// Whether the service is running
    pub running: bool,
    /// Current health status
    pub health: health::SystemHealth,
    /// Last monitoring update timestamp
    pub last_update: chrono::DateTime<chrono::Utc>,
}

/// Errors that can occur during monitoring
#[derive(Debug, Error)]
pub enum MonitoringError {
    /// Error occurred during health check
    #[error("Health check error: {0}")]
    HealthCheckError(String),
    
    /// Error occurred during metrics collection
    #[error("Metrics error: {0}")]
    MetricsError(String),
    
    /// Error occurred during alert processing
    #[error("Alert error: {0}")]
    AlertError(String),
    
    /// Error occurred during network monitoring
    #[error("Network error: {0}")]
    NetworkError(String),
    
    /// General error in the monitoring system
    #[error("Monitoring system error: {0}")]
    SystemError(String),
}

#[cfg(test)]
mod tests;

/// # Monitoring Crate Documentation
/// 
/// This is a placeholder module that holds extended documentation for the monitoring crate.
pub mod documentation {
    /// # Monitoring Crate
    ///
    /// This crate provides a comprehensive monitoring system for applications,
    /// including metrics collection, health checks, alerts, and network monitoring.
    ///
    /// ## Features
    ///
    /// - Metrics collection and monitoring
    /// - Health checks for services and components
    /// - Network monitoring
    /// - Plugin system for extensibility
    /// - Analytics system for data analysis
    /// - Clean API for accessing monitoring data
    ///
    /// ## Examples
    ///
    /// ### Using the metrics system
    ///
    /// ```rust,no_run
    /// use squirrel_monitoring::metrics::{Metric, MetricType, DefaultMetricCollector};
    /// use std::collections::HashMap;
    ///
    /// # async fn example() -> squirrel_core::error::Result<()> {
    /// let collector = DefaultMetricCollector::new();
    /// collector.initialize().await?;
    ///
    /// // Create a new metric with name, value, type, and labels
    /// let metric = Metric::new(
    ///     "system_memory_usage".to_string(),
    ///     1024.0,
    ///     MetricType::Gauge,
    ///     HashMap::new(),
    /// );
    ///
    /// // Record the metric using the collector's method
    /// collector.record_metric(metric).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ### Using the health check system
    ///
    /// ```rust,no_run
    /// use squirrel_monitoring::health::{HealthStatus, ComponentHealth};
    /// use squirrel_monitoring::health::status::Status;
    /// use squirrel_monitoring::health::checker::HealthChecker;
    /// use squirrel_monitoring::health::DefaultHealthChecker;
    ///
    /// # async fn example() -> squirrel_core::error::Result<()> {
    /// // Create a health checker
    /// let health_checker = DefaultHealthChecker::new();
    ///
    /// // Register a component
    /// let component = ComponentHealth {
    ///     name: "database".to_string(),
    ///     status: Status::Healthy,
    ///     message: Some("Database connection is working".to_string()),
    ///     last_check: chrono::Utc::now(),
    ///     details: std::collections::HashMap::new(),
    /// };
    ///
    /// health_checker.register_component(component).await?;
    ///
    /// // Check health
    /// let system_health = health_checker.check_health().await?;
    /// println!("System status: {:?}", system_health.status);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ### Using the monitoring API
    /// 
    /// ```rust,no_run
    /// use squirrel_monitoring::api;
    /// 
    /// # async fn example() -> squirrel_core::error::Result<()> {
    /// // Get the monitoring API
    /// let monitoring_api = api::get_monitoring_api();
    /// 
    /// // Get available components
    /// let components = monitoring_api.get_available_components().await?;
    /// 
    /// // Get component data
    /// if let Some(component) = components.first() {
    ///     let data = monitoring_api.get_component_data(component).await?;
    ///     println!("Component data: {:?}", data);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// This module is a placeholder to help organize the documentation.
    pub struct Examples;
}

/// Default monitoring service implementation
#[derive(Debug, Clone)]
pub struct DefaultMonitoringService {
    // ... existing fields ...
    api_provider: Arc<api::MonitoringAPIProvider>,
    #[cfg(feature = "websocket")]
    websocket: Option<Arc<websocket::server::WebSocketServer>>,
}

impl DefaultMonitoringService {
    // ... existing methods ...

    #[cfg(feature = "websocket")]
    /// Get WebSocket API interface
    pub fn websocket(&self) -> Result<Arc<websocket::server::WebSocketServer>> {
        match &self.websocket {
            Some(websocket) => Ok(Arc::clone(websocket)),
            None => Err(SquirrelError::Generic("WebSocket server not initialized".to_string())),
        }
    }

    #[cfg(feature = "websocket")]
    /// Initialize WebSocket server
    pub async fn init_websocket(&mut self, config: websocket::WebSocketConfig) -> Result<()> {
        let websocket_server = websocket::server::WebSocketServer::new(config);
        self.websocket = Some(Arc::new(websocket_server));
        Ok(())
    }
} 