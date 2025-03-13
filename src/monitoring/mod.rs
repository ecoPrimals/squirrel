//! Monitoring module for Squirrel
//!
//! This module provides monitoring functionality including tracing,
//! logging, and metrics collection.

pub mod tracing;
pub mod logging;
pub mod metrics;

// Re-export commonly used types
pub use tracing::{TraceCollector, TraceExporter, Span, TraceError};
pub use logging::{LogCollector, LogExporter, Log, LogError};
pub use metrics::{MetricCollector, MetricExporter, Metric, MetricError};

/// Initialize the monitoring system
pub async fn initialize() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing::initialize().await?;
    
    // Initialize logging
    logging::initialize().await?;
    
    // Initialize metrics
    metrics::initialize().await?;
    
    Ok(())
}

/// Shutdown the monitoring system
pub async fn shutdown() -> Result<(), Box<dyn std::error::Error>> {
    // Shutdown in reverse order
    metrics::shutdown().await?;
    logging::shutdown().await?;
    tracing::shutdown().await?;
    
    Ok(())
}

/// Get the current monitoring configuration
pub fn get_config() -> MonitoringConfig {
    MonitoringConfig {
        tracing: tracing::get_config(),
        logging: logging::get_config(),
        metrics: metrics::get_config(),
    }
}

/// Configuration for the monitoring system
#[derive(Debug, Clone)]
pub struct MonitoringConfig {
    pub tracing: tracing::TraceConfig,
    pub logging: logging::LogConfig,
    pub metrics: metrics::MetricConfig,
} 