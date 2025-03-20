/// Health status of a component
#[derive(Debug, Clone)]
pub struct HealthStatus {
    /// Whether the component is healthy
    pub is_healthy: bool,
    /// Status message
    pub message: String,
    /// Timestamp of the status check
    pub timestamp: i64,
}

/// Default implementation for HealthStatus
impl Default for HealthStatus {
    fn default() -> Self {
        Self {
            is_healthy: true,
            message: "Healthy".into(),
            timestamp: chrono::Utc::now().timestamp(),
        }
    }
}

/// System status information
#[derive(Debug, Clone, Default)]
pub struct SystemStatus {
    /// CPU usage percentage
    pub cpu_usage: f64,
    /// Memory usage percentage
    pub memory_usage: f64,
    /// Disk usage percentage
    pub disk_usage: f64,
    /// Network usage (bytes/sec)
    pub network_usage: f64,
} 