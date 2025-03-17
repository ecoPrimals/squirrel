use serde::{Serialize, Deserialize};

/// Health status of a component
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HealthStatus {
    /// Component is healthy
    Healthy,
    /// Component is degraded but still functioning
    Degraded,
    /// Component is unhealthy and not functioning properly
    Unhealthy,
    /// Component status is unknown
    Unknown,
} 