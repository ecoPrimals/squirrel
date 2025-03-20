//! Monitoring and metrics system for Squirrel
//!
//! This crate provides functionality for monitoring system health,
//! collecting metrics, and generating alerts.

#![allow(dead_code)] // Temporarily allow dead code during migration

use std::sync::Arc;
use thiserror::Error;
use serde::{Serialize, Deserialize};

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

#[cfg(test)]
mod tests; 