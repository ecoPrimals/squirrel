// Alert status module
// TODO: Implement alert status functionality

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::hash::Hash;

/// Severity levels for alerts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AlertSeverity {
    /// Low severity, informational only
    Info,
    /// Warning level, requires attention but not critical
    Warning,
    /// Error level, requires action
    Error,
    /// Critical level, requires immediate action
    Critical,
}

impl AlertSeverity {
    /// Get the name of the severity level
    #[must_use] pub fn name(&self) -> &'static str {
        match self {
            Self::Info => "Info",
            Self::Warning => "Warning",
            Self::Error => "Error",
            Self::Critical => "Critical",
        }
    }
    
    /// Check if the severity is at least warning level
    #[must_use] pub fn is_warning_or_higher(&self) -> bool {
        matches!(self, Self::Warning | Self::Error | Self::Critical)
    }
    
    /// Check if the severity is at least error level
    #[must_use] pub fn is_error_or_higher(&self) -> bool {
        matches!(self, Self::Error | Self::Critical)
    }
    
    /// Check if the severity is critical
    #[must_use] pub fn is_critical(&self) -> bool {
        matches!(self, Self::Critical)
    }
}

/// Type of threshold comparison for alerts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThresholdType {
    /// Upper bound (alert if value > threshold)
    UpperBound,
    /// Lower bound (alert if value < threshold)
    LowerBound,
    /// Deviation (alert if abs(value - expected) > threshold)
    Deviation,
}

/// Type of resource being monitored
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceType {
    /// Memory usage
    Memory,
    /// CPU usage
    Cpu,
    /// Disk usage
    Disk,
    /// Network usage
    Network,
    /// Connection count
    Connections,
}

/// Type of error being reported
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorType {
    /// System-level error
    System,
    /// Application-level error
    Application,
    /// Protocol-related error
    Protocol,
    /// Tool-related error
    Tool,
}

/// Status of a health check
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// System is healthy
    Healthy,
    /// System is degraded but functioning
    Degraded,
    /// System has failed
    Failed,
}

/// Alert data for performance issues
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAlert {
    /// Type of threshold that was crossed
    pub threshold_type: ThresholdType,
    /// Name of the metric that triggered the alert
    pub metric_name: String,
    /// Current value of the metric
    pub current_value: f64,
    /// Threshold value that was crossed
    pub threshold_value: f64,
    /// How long the threshold has been crossed
    pub duration: Duration,
    /// Severity of the alert
    pub severity: AlertSeverity,
}

/// Alert data for resource usage issues
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAlert {
    /// Type of resource that triggered the alert
    pub resource_type: ResourceType,
    /// Current usage percentage
    pub usage_percentage: f64,
    /// Resource limit in absolute units
    pub limit: u64,
    /// Current usage in absolute units
    pub current: u64,
    /// Severity of the alert
    pub severity: AlertSeverity,
}

/// Alert data for error conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorAlert {
    /// Type of error that occurred
    pub error_type: ErrorType,
    /// Number of errors that have occurred
    pub error_count: u64,
    /// Error rate as percentage
    pub error_rate: f64,
    /// When the error was first seen
    pub first_seen: DateTime<Utc>,
    /// When the error was last seen
    pub last_seen: DateTime<Utc>,
    /// Severity of the alert
    pub severity: AlertSeverity,
}

/// Alert data for health check failures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthAlert {
    /// Name of the health check that failed
    pub check_name: String,
    /// Current status of the health check
    pub status: HealthStatus,
    /// Details about the failure
    pub details: String,
    /// When the health check last succeeded
    pub last_success: DateTime<Utc>,
    /// How long the health check has been failing
    pub failure_duration: Duration,
    /// Severity of the alert
    pub severity: AlertSeverity,
}

/// Types of alerts that can be generated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertType {
    /// Performance alert (metric threshold crossed)
    Performance(PerformanceAlert),
    /// Resource alert (resource usage crossed threshold)
    Resource(ResourceAlert),
    /// Error alert (errors detected)
    Error(ErrorAlert),
    /// Health alert (health check failed)
    Health(HealthAlert),
    /// Generic alert
    Generic,
}

/// Complete alert information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    /// Unique alert identifier
    pub id: Uuid,
    /// Type of alert
    pub alert_type: AlertType,
    /// Alert severity
    pub severity: AlertSeverity,
    /// When the alert was generated
    pub timestamp: DateTime<Utc>,
    /// Source of the alert (component name)
    pub source: String,
    /// Alert message
    pub message: String,
    /// Additional details about the alert
    pub details: HashMap<String, serde_json::Value>,
    /// Whether the alert has been acknowledged
    pub acknowledged: bool,
    /// Who acknowledged the alert
    pub acknowledged_by: Option<String>,
    /// When the alert was acknowledged
    pub acknowledged_at: Option<DateTime<Utc>>,
}

impl Alert {
    /// Create a new alert
    #[must_use] pub fn new(
        alert_type: AlertType,
        severity: AlertSeverity,
        source: String,
        message: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            alert_type,
            severity,
            timestamp: Utc::now(),
            source,
            message,
            details: HashMap::new(),
            acknowledged: false,
            acknowledged_by: None,
            acknowledged_at: None,
        }
    }
    
    /// Add details to the alert
    pub fn with_details(mut self, details: HashMap<String, serde_json::Value>) -> Self {
        self.details = details;
        self
    }
    
    /// Check if the alert is active (not acknowledged)
    #[must_use] pub fn is_active(&self) -> bool {
        !self.acknowledged
    }
    
    /// Acknowledge the alert
    pub fn acknowledge(&mut self, by: String) {
        self.acknowledged = true;
        self.acknowledged_by = Some(by);
        self.acknowledged_at = Some(Utc::now());
    }
    
    /// Get the age of the alert
    #[must_use] pub fn age(&self) -> Duration {
        let now = SystemTime::now();
        let alert_timestamp = self.timestamp.timestamp();
        let alert_time = SystemTime::UNIX_EPOCH + Duration::from_secs(alert_timestamp as u64);
        now.duration_since(alert_time).unwrap_or_default()
    }
}

/// Alert status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertStatus {
    /// The severity of the alert
    pub severity: AlertSeverity,
    /// Description of the alert
    pub message: String,
    /// Component that generated the alert
    pub component: String,
    /// Timestamp when the alert was generated
    pub timestamp: u64,
}

impl AlertStatus {
    /// Create a new alert status
    #[must_use] pub fn new(
        severity: AlertSeverity,
        message: String,
        component: String,
        timestamp: u64,
    ) -> Self {
        Self {
            severity,
            message,
            component,
            timestamp,
        }
    }
} 