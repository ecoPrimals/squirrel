//! Dashboard data models.
//!
//! This module defines the core data structures used by the dashboard.

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// Representation of all dashboard data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardData {
    /// System metrics snapshot
    pub system: SystemSnapshot,
    /// Network metrics snapshot
    pub network: NetworkSnapshot,
    /// Alerts snapshot
    pub alerts: AlertsSnapshot,
    /// Application metrics 
    pub metrics: MetricsSnapshot,
    /// Timestamp when this data was collected
    pub timestamp: DateTime<Utc>,
}

/// System resource usage metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemSnapshot {
    /// CPU usage percentage (0-100)
    pub cpu_usage: f64,
    /// Memory usage in bytes
    pub memory_used: u64,
    /// Total memory in bytes
    pub memory_total: u64,
    /// Disk usage in bytes
    pub disk_used: u64,
    /// Total disk space in bytes
    pub disk_total: u64,
    /// System load average (1, 5, 15 minutes)
    pub load_average: [f64; 3],
    /// System uptime in seconds
    pub uptime: u64,
}

/// Network metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkSnapshot {
    /// Bytes received
    pub rx_bytes: u64,
    /// Bytes transmitted
    pub tx_bytes: u64,
    /// Packets received
    pub rx_packets: u64,
    /// Packets transmitted
    pub tx_packets: u64,
    /// Network interfaces statistics
    pub interfaces: HashMap<String, InterfaceStats>,
}

/// Network interface statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceStats {
    /// Interface name
    pub name: String,
    /// Bytes received
    pub rx_bytes: u64,
    /// Bytes transmitted
    pub tx_bytes: u64,
    /// Packets received
    pub rx_packets: u64,
    /// Packets transmitted
    pub tx_packets: u64,
    /// Is the interface up?
    pub is_up: bool,
}

/// Alert information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertsSnapshot {
    /// Current active alerts
    pub active: Vec<Alert>,
    /// Recent alerts (resolved)
    pub recent: Vec<Alert>,
    /// Total count of active alerts by severity
    pub counts: HashMap<AlertSeverity, u32>,
}

/// Alert severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AlertSeverity {
    /// Critical alert requiring immediate attention
    Critical,
    /// High severity alert
    High,
    /// Medium severity alert
    Medium,
    /// Low severity alert
    Low,
    /// Informational alert
    Info,
}

/// Alert information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    /// Unique identifier for the alert
    pub id: String,
    /// Alert title
    pub title: String,
    /// Alert description
    pub description: String,
    /// Alert severity
    pub severity: AlertSeverity,
    /// Time the alert was triggered
    pub triggered_at: DateTime<Utc>,
    /// Time the alert was resolved (if any)
    pub resolved_at: Option<DateTime<Utc>>,
    /// Has the alert been acknowledged?
    pub acknowledged: bool,
    /// Time the alert was acknowledged (if any)
    pub acknowledged_at: Option<DateTime<Utc>>,
    /// User who acknowledged the alert (if any)
    pub acknowledged_by: Option<String>,
}

/// Application metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    /// Generic metrics as key-value pairs
    pub values: HashMap<String, f64>,
    /// Counters as key-value pairs
    pub counters: HashMap<String, u64>,
    /// Gauges as key-value pairs
    pub gauges: HashMap<String, f64>,
    /// Histograms
    pub histograms: HashMap<String, Vec<f64>>,
} 