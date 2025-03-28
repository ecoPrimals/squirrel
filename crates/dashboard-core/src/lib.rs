//! # Dashboard Core
//! 
//! Core functionality for the Squirrel monitoring dashboard.
//! This crate provides the data models, services, and traits for implementing
//! dashboard functionality in various UI interfaces.

pub mod data;
pub mod config;
pub mod service;
pub mod error;
pub mod update;
pub mod health;

// Re-export key components for external use
pub use data::{
    DashboardData, Metrics, CpuMetrics, MemoryMetrics, NetworkMetrics, DiskMetrics, Alert,
    MetricType, Protocol, ProtocolStatus, ProtocolData, AlertSeverity, NetworkInterface, MetricsHistory
};
pub use config::DashboardConfig;
pub use service::{DashboardService, DefaultDashboardService};
pub use error::{DashboardError, Result};
pub use update::DashboardUpdate;
pub use health::{HealthCheck, HealthStatus};

// This is a placeholder for the actual implementation
// Once migrated, we'll replace this with the real implementation

/// Core module for the dashboard functionality
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
} 