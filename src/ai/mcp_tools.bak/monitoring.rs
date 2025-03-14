use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};
use serde::{Serialize, Deserialize};
use metrics::{counter, gauge, histogram};
use crate::ai::mcp_tools::{
    context::MachineContext,
    registry::RegistryService,
    types::MCPError,
};

/// Health status of a component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded(String),
    Unhealthy(String),
}

/// Health check information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    pub component: String,
    pub status: HealthStatus,
    pub last_check: SystemTime,
    pub details: Option<String>,
}

/// System metrics
#[derive(Debug, Clone)]
pub struct SystemMetrics {
    pub active_tools: usize,
    pub total_commands: usize,
    pub active_connections: usize,
    pub memory_usage: usize,
    pub cpu_usage: f64,
    pub error_count: usize,
    pub average_response_time: f64,
}

impl Default for SystemMetrics {
    fn default() -> Self {
        Self {
            active_tools: 0,
            total_commands: 0,
            active_connections: 0,
            memory_usage: 0,
            cpu_usage: 0.0,
            error_count: 0,
            average_response_time: 0.0,
        }
    }
}

/// Performance data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetric {
    pub timestamp: SystemTime,
    pub operation: String,
    pub duration: Duration,
    pub success: bool,
    pub context: HashMap<String, String>,
}

/// Monitoring configuration
#[derive(Debug, Clone)]
pub struct MonitoringConfig {
    pub health_check_interval: Duration,
    pub metrics_interval: Duration,
    pub retention_period: Duration,
} 