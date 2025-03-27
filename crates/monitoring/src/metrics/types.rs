use serde::{Serialize, Deserialize};
use std::fmt;

/// Error type for metrics operations
#[derive(Debug, Clone)]
pub enum MetricsError {
    /// Error collecting metrics
    CollectionError(String),
    /// Error processing metrics
    ProcessingError(String),
    /// System-specific error
    SystemError(String),
}

impl fmt::Display for MetricsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MetricsError::CollectionError(msg) => write!(f, "Metrics collection error: {}", msg),
            MetricsError::ProcessingError(msg) => write!(f, "Metrics processing error: {}", msg),
            MetricsError::SystemError(msg) => write!(f, "System error: {}", msg),
        }
    }
}

impl std::error::Error for MetricsError {}

/// CPU metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuMetrics {
    /// CPU usage percentage (0-100)
    pub usage_percentage: f32,
}

/// Memory metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryMetrics {
    /// Total physical memory in bytes
    pub total_bytes: u64,
    /// Used physical memory in bytes
    pub used_bytes: u64,
    /// Available physical memory in bytes
    pub available_bytes: u64,
    /// Memory usage percentage (0-100)
    pub usage_percentage: f64,
}

/// Disk metrics for a single disk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskMetrics {
    /// Disk name
    pub name: String,
    /// Mount point
    pub mount_point: String,
    /// Total disk space in bytes
    pub total_bytes: u64,
    /// Used disk space in bytes
    pub used_bytes: u64,
    /// Available disk space in bytes
    pub available_bytes: u64,
    /// Disk usage percentage (0-100)
    pub usage_percentage: f64,
}

/// Network interface metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMetrics {
    /// Network interface name
    pub interface: String,
    /// Total bytes received
    pub received_bytes: u64,
    /// Total bytes transmitted
    pub transmitted_bytes: u64,
}

/// Trait for resource metrics collectors
pub trait ResourceMetricsCollector: Send + Sync {
    /// Collect CPU metrics
    fn collect_cpu_metrics(&mut self) -> Result<CpuMetrics, MetricsError>;
    
    /// Collect memory metrics
    fn collect_memory_metrics(&mut self) -> Result<MemoryMetrics, MetricsError>;
    
    /// Collect disk metrics for all disks
    fn collect_disk_metrics(&mut self) -> Result<Vec<DiskMetrics>, MetricsError>;
    
    /// Collect network metrics for all interfaces
    fn collect_network_metrics(&mut self) -> Result<Vec<NetworkMetrics>, MetricsError>;
}

/// Factory trait for creating metrics collectors
pub trait MetricsCollectorFactory<T: ?Sized>: Send + Sync {
    /// Create a new metrics collector
    fn create(&self) -> Box<dyn T>;
} 