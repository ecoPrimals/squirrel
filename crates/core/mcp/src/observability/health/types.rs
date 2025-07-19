//! Basic types for health checking

use std::fmt;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Health status enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub enum HealthStatus {
    /// Unknown status (initial state or cannot determine)
    Unknown,
    /// Healthy status
    Healthy,
    /// Degraded but still functioning
    Degraded,
    /// Unhealthy and not functioning
    Unhealthy,
}

impl fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HealthStatus::Unknown => write!(f, "UNKNOWN"),
            HealthStatus::Healthy => write!(f, "HEALTHY"),
            HealthStatus::Degraded => write!(f, "DEGRADED"),
            HealthStatus::Unhealthy => write!(f, "UNHEALTHY"),
        }
    }
}

impl HealthStatus {
    /// Convert a string to a HealthStatus
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "unknown" => Ok(HealthStatus::Unknown),
            "healthy" => Ok(HealthStatus::Healthy),
            "degraded" => Ok(HealthStatus::Degraded),
            "unhealthy" => Ok(HealthStatus::Unhealthy),
            _ => Err(format!("Invalid health status: {}", s)),
        }
    }
}

/// Type of health check
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum HealthCheckType {
    /// Basic health check
    Basic,
    /// Simple liveness check
    Liveness,
    /// More thorough readiness check
    Readiness,
    /// Comprehensive health check
    Comprehensive,
}

impl fmt::Display for HealthCheckType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HealthCheckType::Basic => write!(f, "basic"),
            HealthCheckType::Liveness => write!(f, "liveness"),
            HealthCheckType::Readiness => write!(f, "readiness"),
            HealthCheckType::Comprehensive => write!(f, "comprehensive"),
        }
    }
}

/// Overall system health report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealthReport {
    /// Overall system health status
    pub overall_status: HealthStatus,
    /// Individual component health statuses
    pub components: HashMap<String, ComponentHealthInfo>,
    /// System-wide health metrics
    pub metrics: SystemHealthMetrics,
    /// Timestamp when the report was generated
    pub timestamp: std::time::SystemTime,
    /// Additional system information
    pub system_info: SystemInfo,
}

/// Health information for a specific component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealthInfo {
    /// Component health status
    pub status: HealthStatus,
    /// Component name
    pub name: String,
    /// Last health check timestamp
    pub last_check: Option<std::time::SystemTime>,
    /// Health check details/messages
    pub details: Option<String>,
    /// Component-specific metrics
    pub metrics: HashMap<String, f64>,
}

/// System-wide health metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealthMetrics {
    /// Number of healthy components
    pub healthy_components: usize,
    /// Number of degraded components
    pub degraded_components: usize,
    /// Number of unhealthy components
    pub unhealthy_components: usize,
    /// Number of unknown status components
    pub unknown_components: usize,
    /// System uptime in seconds
    pub uptime_seconds: u64,
    /// Memory usage percentage
    pub memory_usage_percent: f64,
    /// CPU usage percentage
    pub cpu_usage_percent: f64,
}

/// System information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    /// Service name
    pub service_name: String,
    /// Service version
    pub version: String,
    /// Environment (e.g., "production", "development")
    pub environment: String,
    /// Host information
    pub host: String,
} 