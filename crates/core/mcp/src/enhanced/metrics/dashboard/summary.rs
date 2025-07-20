//! Dashboard summary structures

use std::collections::HashMap;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

use super::{AlertSeverity, trends::TrendData};

/// Health summary
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HealthSummary {
    /// Overall health status
    pub overall_status: HealthStatus,
    
    /// Component healths
    pub component_healths: HashMap<String, HealthStatus>,
    
    /// Health score (0.0 to 1.0)
    pub health_score: f64,
    
    /// Issues count
    pub issues_count: u32,
    
    /// Warnings count
    pub warnings_count: u32,
}

/// Health status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// Healthy
    Healthy,
    /// Warning
    Warning,
    /// Critical
    Critical,
    /// Unknown
    Unknown,
}

impl Default for HealthStatus {
    fn default() -> Self {
        Self::Unknown
    }
}

/// Performance trends
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PerformanceTrends {
    /// Response time trend
    pub response_time: TrendData,
    
    /// Throughput trend
    pub throughput: TrendData,
    
    /// Error rate trend
    pub error_rate: TrendData,
    
    /// CPU usage trend
    pub cpu_usage: TrendData,
    
    /// Memory usage trend
    pub memory_usage: TrendData,
}

/// Component status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentStatus {
    /// Component name
    pub name: String,
    
    /// Status
    pub status: HealthStatus,
    
    /// Last response time
    pub last_response_time: Option<Duration>,
    
    /// Error count in last hour
    pub recent_errors: u32,
    
    /// Last check timestamp
    pub last_check: Option<DateTime<Utc>>,
    
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Dashboard overview
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardOverview {
    /// Health summary
    pub health: HealthSummary,
    
    /// Performance trends
    pub performance: PerformanceTrends,
    
    /// Alert summary
    pub alerts: AlertSummary,
    
    /// Component statuses
    pub components: Vec<ComponentStatus>,
    
    /// Last updated timestamp
    pub last_updated: Option<DateTime<Utc>>,
}

/// System overview
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SystemOverview {
    /// System uptime
    pub uptime: Duration,
    
    /// CPU usage percentage
    pub cpu_usage: f64,
    
    /// Memory usage percentage
    pub memory_usage: f64,
    
    /// Disk usage percentage
    pub disk_usage: f64,
    
    /// Network I/O
    pub network_io: NetworkStats,
    
    /// Active connections
    pub active_connections: u32,
    
    /// System load average
    pub load_average: [f64; 3],
}

/// Network statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NetworkStats {
    /// Bytes received
    pub bytes_received: u64,
    
    /// Bytes sent
    pub bytes_sent: u64,
    
    /// Packets received
    pub packets_received: u64,
    
    /// Packets sent
    pub packets_sent: u64,
    
    /// Errors
    pub errors: u32,
}

/// Performance summary
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PerformanceSummary {
    /// Average response time
    pub avg_response_time: Duration,
    
    /// 95th percentile response time
    pub p95_response_time: Duration,
    
    /// 99th percentile response time
    pub p99_response_time: Duration,
    
    /// Requests per second
    pub requests_per_second: f64,
    
    /// Error rate percentage
    pub error_rate: f64,
    
    /// Success rate percentage
    pub success_rate: f64,
    
    /// Throughput (bytes per second)
    pub throughput: f64,
}

/// Alert summary
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AlertSummary {
    /// Total active alerts
    pub total_active: u32,
    
    /// Critical alerts count
    pub critical_count: u32,
    
    /// High alerts count
    pub high_count: u32,
    
    /// Medium alerts count
    pub medium_count: u32,
    
    /// Low alerts count
    pub low_count: u32,
    
    /// Recent alerts (last hour)
    pub recent_alerts: Vec<RecentAlert>,
}

/// Recent alert information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentAlert {
    /// Alert ID
    pub id: String,
    
    /// Alert message
    pub message: String,
    
    /// Severity
    pub severity: AlertSeverity,
    
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    
    /// Component
    pub component: String,
} 