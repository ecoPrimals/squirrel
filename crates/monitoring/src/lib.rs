//! Monitoring and metrics system for Squirrel
//!
//! This crate provides functionality for monitoring system health,
//! collecting metrics, and generating alerts.

#![allow(dead_code)] // Temporarily allow dead code during migration

use std::sync::Arc;
use thiserror::Error;
use serde::{Serialize, Deserialize};
use async_trait;

/// Module for alert functionality
pub mod alerts;

/// Module for health checking
pub mod health;

/// Module for metrics collection and reporting
pub mod metrics;

/// Module for network monitoring
pub mod network;

/// Configuration for the monitoring system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// Alert configuration settings
    pub alert_config: alerts::AlertConfig,
    /// Health check configuration settings
    pub health_config: health::HealthConfig,
    /// Metrics collection configuration settings
    pub metrics_config: metrics::MetricConfig,
    /// Network monitoring configuration settings
    pub network_config: network::NetworkConfig,
    /// Monitoring intervals in seconds
    pub intervals: MonitoringIntervals,
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

/// Monitoring service for managing all monitoring components
// pub mod service;
// pub use service::MonitoringService;

/// Common types used throughout the monitoring system
pub use alerts::{Alert, AlertSeverity};
pub use health::ComponentHealth;
pub use metrics::Metric;
pub use network::NetworkStats;

/// Re-export common types from the core crate
pub use squirrel_core::error::{Result, SquirrelError};

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