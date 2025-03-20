// Alert status module
// TODO: Implement alert status functionality

use serde::{Serialize, Deserialize};

/// Severity levels for alerts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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