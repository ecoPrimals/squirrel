use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// Dashboard statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardStats {
    /// Number of connected clients
    pub client_count: usize,
    /// Number of registered components
    pub component_count: usize,
    /// Number of registered plugins
    pub plugin_count: usize,
    /// Number of active views
    pub active_view_count: usize,
    /// Server uptime in seconds
    pub uptime_seconds: u64,
    /// Memory usage in bytes
    pub memory_usage: u64,
    /// CPU usage percentage
    pub cpu_usage: f64,
    /// Timestamp of the statistics
    pub timestamp: DateTime<Utc>,
    /// Additional metrics
    pub additional_metrics: HashMap<String, serde_json::Value>,
}

impl Default for DashboardStats {
    fn default() -> Self {
        Self {
            client_count: 0,
            component_count: 0,
            plugin_count: 0,
            active_view_count: 0,
            uptime_seconds: 0,
            memory_usage: 0,
            cpu_usage: 0.0,
            timestamp: Utc::now(),
            additional_metrics: HashMap::new(),
        }
    }
}

/// Client information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    /// Client ID
    pub id: String,
    /// Client IP address
    pub ip: String,
    /// Client user agent
    pub user_agent: String,
    /// Connection time
    pub connected_at: DateTime<Utc>,
    /// Last activity time
    pub last_activity: DateTime<Utc>,
    /// Additional client data
    pub metadata: HashMap<String, String>,
} 