use dashboard_core::data::{
    Alert, AlertSeverity, CpuMetrics, DiskMetrics, DiskUsage, MemoryMetrics, Metrics,
    MetricsHistory, NetworkInterface, NetworkMetrics, Protocol, ProtocolData, ProtocolStatus
};
use dashboard_core::health::HealthCheck;
use std::fmt::Debug;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Metrics snapshot provides a moment-in-time view of system metrics
#[derive(Debug, Clone, Default)]
pub struct MetricsSnapshot {
    /// CPU usage percentage (0-100)
    pub cpu_usage: f64,
    /// CPU usage per core
    pub cpu_per_core: Vec<f64>,
    /// CPU temperature (if available)
    pub cpu_temperature: Option<f64>,
    /// Load average [1min, 5min, 15min]
    pub load_average: [f64; 3],
    /// Total memory in bytes
    pub memory_total: u64,
    /// Used memory in bytes
    pub memory_used: u64,
    /// Available memory in bytes
    pub memory_available: u64,
    /// Free memory in bytes
    pub memory_free: u64,
    /// Used swap in bytes
    pub swap_used: u64,
    /// Total swap in bytes
    pub swap_total: u64,
    /// Network interfaces
    pub network_interfaces: Vec<NetworkInterface>,
    /// Disk usage by mount point
    pub disk_usage: HashMap<String, dashboard_core::data::DiskUsage>,
    /// Total disk read operations
    pub disk_reads: u64,
    /// Total disk write operations
    pub disk_writes: u64,
    /// Total disk read bytes
    pub disk_read_bytes: u64,
    /// Total disk write bytes
    pub disk_write_bytes: u64,
    /// Historical metrics data
    pub history: dashboard_core::data::MetricsHistory,
    /// Timestamp when the snapshot was taken
    pub timestamp: DateTime<Utc>,
}

/// Trait for monitoring adapters that can provide metrics, health checks, alerts, and protocol status.
pub trait MonitoringAdapter: Send + Sync + Debug {
    /// Gets the current system metrics
    fn get_metrics(&self) -> Metrics;
    
    /// Gets the current health checks
    fn health_checks(&self) -> Vec<HealthCheck>;
    
    /// Gets the current alerts
    fn alerts(&self) -> Vec<Alert>;
    
    /// Gets the protocol status
    fn protocol_status(&self) -> Option<ProtocolData>;
}

/// Mock implementation that provides generated test data
pub mod mock;

/// Real implementation that connects to the monitoring system
/// This will be fully implemented once MCP refactoring is complete
pub mod real {
    use super::*;
    
    #[derive(Debug)]
    pub struct MonitoringService {
        pub mcp_client: Option<Box<dyn crate::adapter::McpClientAdapter>>,
    }
    
    impl MonitoringService {
        pub fn new() -> Self {
            Self {
                mcp_client: None,
            }
        }

        pub fn with_mcp_client(mut self, mcp_client: Box<dyn crate::adapter::McpClientAdapter>) -> Self {
            self.mcp_client = Some(mcp_client);
            self
        }
    }
    
    impl MonitoringAdapter for MonitoringService {
        fn get_metrics(&self) -> Metrics {
            Metrics::default() // Default implementation for now
        }
        
        fn health_checks(&self) -> Vec<HealthCheck> {
            Vec::new() // Default implementation for now
        }
        
        fn alerts(&self) -> Vec<Alert> {
            Vec::new() // Default implementation for now
        }
        
        fn protocol_status(&self) -> Option<ProtocolData> {
            None // Default implementation for now
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_monitoring_service_creation() {
        let service = crate::monitoring::real::MonitoringService::new();
        assert!(service.mcp_client.is_none());
    }
} 