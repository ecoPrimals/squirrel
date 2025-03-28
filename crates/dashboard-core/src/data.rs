//! Dashboard data models.
//!
//! This module defines the core data structures used by the dashboard.

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// Metric type enumeration
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MetricType {
    /// Counter metric (only increases)
    Counter,
    /// Gauge metric (can go up and down)
    Gauge,
    /// Histogram metric (distribution of values)
    Histogram,
    /// Summary metric (percentiles)
    Summary,
    /// CPU usage metric
    CpuUsage,
    /// Memory usage metric
    MemoryUsage,
    /// Disk IO metric
    DiskIO,
    /// Network receive metric for a specific interface
    NetworkRx(String),
    /// Network transmit metric for a specific interface
    NetworkTx(String),
    /// Custom metric
    Custom(String),
}

/// Protocol type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Protocol {
    /// HTTP protocol
    Http,
    /// MQTT protocol
    Mqtt,
    /// WebSocket protocol
    WebSocket,
    /// gRPC protocol
    Grpc,
    /// Custom protocol
    Custom(u8),
}

/// Protocol status enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProtocolStatus {
    /// Connected
    Connected,
    /// Disconnected
    Disconnected,
    /// Connecting
    Connecting,
    /// Error
    Error,
    /// Running
    Running,
    /// Degraded
    Degraded,
    /// Stopped
    Stopped,
    /// Unknown
    Unknown,
}

/// Dashboard data structure.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DashboardData {
    /// System metrics
    pub metrics: Metrics,
    
    /// Protocol data
    pub protocol: ProtocolData,
    
    /// System alerts
    pub alerts: Vec<Alert>,
    
    /// Timestamp of the data
    pub timestamp: DateTime<Utc>,
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
    /// Network interfaces
    pub interfaces: Vec<NetworkInterface>,
    
    /// Total bytes received
    pub total_rx_bytes: u64,
    
    /// Total bytes transmitted
    pub total_tx_bytes: u64,
    
    /// Total packets received
    pub total_rx_packets: u64,
    
    /// Total packets transmitted
    pub total_tx_packets: u64,
}

/// Network interface
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NetworkInterface {
    /// Interface name
    pub name: String,
    
    /// Interface is up
    pub is_up: bool,
    
    /// Bytes received
    pub rx_bytes: u64,
    
    /// Bytes transmitted
    pub tx_bytes: u64,
    
    /// Packets received
    pub rx_packets: u64,
    
    /// Packets transmitted
    pub tx_packets: u64,
    
    /// Errors on receive
    pub rx_errors: u64,
    
    /// Errors on transmit
    pub tx_errors: u64,
}

/// Disk usage metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiskMetrics {
    /// Disk usages by mount point
    pub usage: HashMap<String, DiskUsage>,
    
    /// Total reads
    pub total_reads: u64,
    
    /// Total writes
    pub total_writes: u64,
    
    /// Read bytes
    pub read_bytes: u64,
    
    /// Written bytes
    pub written_bytes: u64,
}

/// Disk usage
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiskUsage {
    /// Mount point
    pub mount_point: String,
    
    /// Total space in bytes
    pub total: u64,
    
    /// Used space in bytes
    pub used: u64,
    
    /// Free space in bytes
    pub free: u64,
    
    /// Used percentage
    pub used_percentage: f64,
}

/// System alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    /// Alert ID
    pub id: String,
    
    /// Alert title
    pub title: String,
    
    /// Alert message
    pub message: String,
    
    /// Alert severity
    pub severity: AlertSeverity,
    
    /// Alert source
    pub source: String,
    
    /// Timestamp when the alert was created
    pub timestamp: DateTime<Utc>,
    
    /// Whether the alert has been acknowledged
    pub acknowledged: bool,
    
    /// User who acknowledged the alert
    pub acknowledged_by: Option<String>,
    
    /// Timestamp when the alert was acknowledged
    pub acknowledged_at: Option<DateTime<Utc>>,
}

/// Alert severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertSeverity {
    /// Informational alert
    Info,
    
    /// Warning alert
    Warning,
    
    /// Error alert
    Error,
    
    /// Critical alert
    Critical,
}

/// Metrics history
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MetricsHistory {
    /// CPU usage history
    pub cpu: Vec<(DateTime<Utc>, f64)>,
    
    /// Memory usage history
    pub memory: Vec<(DateTime<Utc>, f64)>,
    
    /// Network usage history (tx/rx)
    pub network: Vec<(DateTime<Utc>, (u64, u64))>,
    
    /// Custom metrics history
    pub custom: HashMap<String, Vec<(DateTime<Utc>, f64)>>,
}

/// Protocol data structure
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProtocolData {
    /// Protocol name
    pub name: String,
    
    /// Protocol type
    pub protocol_type: String,
    
    /// Protocol version
    pub version: String,
    
    /// Whether connected to the protocol
    pub connected: bool,
    
    /// Last connection time
    pub last_connected: Option<DateTime<Utc>>,
    
    /// Protocol status
    pub status: String,
    
    /// Error message if any
    pub error: Option<String>,
    
    /// Number of retries
    pub retry_count: u32,
    
    /// Additional protocol-specific metrics
    pub metrics: HashMap<String, f64>,
} 