//! Dashboard data models.
//!
//! This module defines the core data structures used by the dashboard.

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// Dashboard data structure.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DashboardData {
    /// System metrics
    pub metrics: Metrics,
    
    /// Protocol data
    pub protocol: ProtocolData,
    
    /// System alerts
    pub alerts: Vec<Alert>,
}

/// System metrics data.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Metrics {
    /// CPU usage metrics
    pub cpu: CpuMetrics,
    
    /// Memory usage metrics
    pub memory: MemoryMetrics,
    
    /// Network metrics
    pub network: NetworkMetrics,
    
    /// Disk usage metrics
    pub disk: DiskMetrics,
    
    /// Time-series data for metrics
    pub history: MetricsHistory,
}

/// CPU usage metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CpuMetrics {
    /// Overall CPU usage in percent (0-100)
    pub usage: f64,
    
    /// CPU usage per core
    pub cores: Vec<f64>,
    
    /// CPU temperature (if available)
    pub temperature: Option<f64>,
    
    /// Load averages (1min, 5min, 15min)
    pub load: [f64; 3],
}

/// Memory usage metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MemoryMetrics {
    /// Total memory in bytes
    pub total: u64,
    
    /// Used memory in bytes
    pub used: u64,
    
    /// Available memory in bytes
    pub available: u64,
    
    /// Free memory in bytes
    pub free: u64,
    
    /// Used swap memory in bytes
    pub swap_used: u64,
    
    /// Total swap memory in bytes
    pub swap_total: u64,
}

/// Network metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NetworkMetrics {
    /// Received bytes per second
    pub rx_per_sec: f64,
    
    /// Transmitted bytes per second
    pub tx_per_sec: f64,
    
    /// Total received bytes
    pub rx_total: u64,
    
    /// Total transmitted bytes
    pub tx_total: u64,
    
    /// Network interfaces
    pub interfaces: HashMap<String, NetworkInterface>,
}

/// Network interface data
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NetworkInterface {
    /// Interface name
    pub name: String,
    
    /// Received bytes per second
    pub rx_per_sec: f64,
    
    /// Transmitted bytes per second
    pub tx_per_sec: f64,
    
    /// Total received bytes
    pub rx_total: u64,
    
    /// Total transmitted bytes
    pub tx_total: u64,
    
    /// Whether the interface is up
    pub is_up: bool,
}

/// Disk usage metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiskMetrics {
    /// Disks data
    pub disks: HashMap<String, DiskInfo>,
    
    /// IO operations per second
    pub io_per_sec: f64,
    
    /// Read bytes per second
    pub read_per_sec: f64,
    
    /// Write bytes per second
    pub write_per_sec: f64,
}

/// Disk information
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiskInfo {
    /// Mount point
    pub mount_point: String,
    
    /// Total size in bytes
    pub total: u64,
    
    /// Used space in bytes
    pub used: u64,
    
    /// Free space in bytes
    pub free: u64,
    
    /// Filesystem type
    pub fs_type: String,
}

/// Metrics history for time series data
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MetricsHistory {
    /// Timestamps for data points
    pub timestamps: Vec<DateTime<Utc>>,
    
    /// CPU usage history
    pub cpu_usage: Vec<f64>,
    
    /// Memory usage history
    pub memory_usage: Vec<f64>,
    
    /// Network receive history
    pub network_rx: Vec<f64>,
    
    /// Network transmit history
    pub network_tx: Vec<f64>,
    
    /// Disk IO history
    pub disk_io: Vec<f64>,
}

/// Protocol data structure
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProtocolData {
    /// Protocol status
    pub status: String,
    
    /// Protocol version
    pub version: String,
    
    /// Connection state
    pub connected: bool,
    
    /// Last connection time
    pub last_connected: Option<DateTime<Utc>>,
    
    /// Error message if any
    pub error: Option<String>,
    
    /// Retry count
    pub retry_count: u32,
    
    /// Protocol metrics
    pub metrics: HashMap<String, String>,
    
    /// Protocol specific data
    pub data: HashMap<String, String>,
}

/// System alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    /// Alert ID
    pub id: String,
    
    /// Alert severity ("critical", "warning", "info")
    pub severity: String,
    
    /// Alert message
    pub message: String,
    
    /// Alert details
    pub details: Option<String>,
    
    /// When the alert was generated
    pub timestamp: DateTime<Utc>,
    
    /// Whether the alert has been acknowledged
    pub acknowledged: bool,
    
    /// Who acknowledged the alert
    pub acknowledged_by: Option<String>,
    
    /// When the alert was acknowledged
    pub acknowledged_at: Option<DateTime<Utc>>,
}

impl Alert {
    /// Create a new alert
    pub fn new(id: &str, severity: &str, message: &str) -> Self {
        Self {
            id: id.to_string(),
            severity: severity.to_string(),
            message: message.to_string(),
            details: None,
            timestamp: Utc::now(),
            acknowledged: false,
            acknowledged_by: None,
            acknowledged_at: None,
        }
    }
    
    /// Add details to the alert
    pub fn with_details(mut self, details: &str) -> Self {
        self.details = Some(details.to_string());
        self
    }
    
    /// Acknowledge the alert
    pub fn acknowledge(&mut self, by: &str) {
        self.acknowledged = true;
        self.acknowledged_by = Some(by.to_string());
        self.acknowledged_at = Some(Utc::now());
    }
    
    /// Check if the alert is critical
    pub fn is_critical(&self) -> bool {
        self.severity == "critical"
    }
    
    /// Check if the alert is a warning
    pub fn is_warning(&self) -> bool {
        self.severity == "warning"
    }
    
    /// Check if the alert is informational
    pub fn is_info(&self) -> bool {
        self.severity == "info" 
    }
} 