use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

/// Health status of a service or component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    /// Name of the service or component
    pub service: String,
    /// Current health status
    pub status: Status,
    /// Detailed message about the health status
    pub message: String,
    /// Timestamp when the status was last updated
    pub timestamp: DateTime<Utc>,
}

/// Health status values
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status {
    /// Service is healthy and operating normally
    Healthy,
    /// Service is degraded but still functioning
    Degraded,
    /// Service is unhealthy or not functioning
    Unhealthy,
    /// Service status is unknown
    Unknown,
}

impl Default for HealthStatus {
    fn default() -> Self {
        Self {
            service: String::new(),
            status: Status::Unknown,
            message: String::new(),
            timestamp: Utc::now(),
        }
    }
}

impl HealthStatus {
    /// Creates a new health status
    #[must_use] pub fn new(service: String, status: Status, message: String) -> Self {
        Self {
            service,
            status,
            message,
            timestamp: Utc::now(),
        }
    }

    /// Creates a new healthy status
    #[must_use] pub fn healthy(service: String, message: String) -> Self {
        Self::new(service, Status::Healthy, message)
    }

    /// Creates a new degraded status
    #[must_use] pub fn degraded(service: String, message: String) -> Self {
        Self::new(service, Status::Degraded, message)
    }

    /// Creates a new unhealthy status
    #[must_use] pub fn unhealthy(service: String, message: String) -> Self {
        Self::new(service, Status::Unhealthy, message)
    }

    /// Creates a new unknown status
    #[must_use] pub fn unknown(service: String, message: String) -> Self {
        Self::new(service, Status::Unknown, message)
    }
} 