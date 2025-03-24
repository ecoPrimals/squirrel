//! Alert type definitions and utilities

use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use uuid::Uuid;

use super::status::AlertStatusType;

/// Alert severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum AlertLevel {
    /// Informational alerts that don't require immediate action
    Info,
    /// Warnings that should be monitored but don't require immediate action
    Warning,
    /// Errors that require attention but aren't system-critical
    Error,
    /// Critical issues that require immediate attention
    Critical,
}

impl std::fmt::Display for AlertLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AlertLevel::Info => write!(f, "Info"),
            AlertLevel::Warning => write!(f, "Warning"),
            AlertLevel::Error => write!(f, "Error"),
            AlertLevel::Critical => write!(f, "Critical"),
        }
    }
}

/// Alert represents a notification about a significant event in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    /// Unique alert identifier
    pub id: String,
    /// Alert type category
    pub alert_type: String,
    /// Component or service that generated the alert
    pub source: String,
    /// Alert message describing the issue
    pub message: String,
    /// Severity level of the alert
    pub level: AlertLevel,
    /// Current status of the alert
    pub status: AlertStatusType,
    /// Time when the alert was created
    pub created_at: DateTime<Utc>,
    /// Time when the alert occurred
    pub occurred: DateTime<Utc>,
    /// Time when the alert was last updated
    pub last_updated: DateTime<Utc>,
    /// Time when the alert last occurred (for repeated alerts)
    pub last_occurred: DateTime<Utc>,
    /// Number of times this alert has occurred
    pub count: usize,
    /// Key-value pairs with additional alert details
    pub details: HashMap<String, String>,
}

impl Alert {
    /// Create a new alert with the given parameters
    pub fn new(
        alert_type: String,
        source: String,
        message: String,
        level: AlertLevel,
        details: HashMap<String, String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            alert_type,
            source,
            message,
            level,
            status: AlertStatusType::Active,
            created_at: now,
            occurred: now,
            last_updated: now,
            last_occurred: now,
            count: 1,
            details,
        }
    }
    
    /// Check if the alert is currently active
    pub fn is_active(&self) -> bool {
        self.status == AlertStatusType::Active
    }
    
    /// Mark the alert as acknowledged
    pub fn acknowledge(&mut self) {
        self.status = AlertStatusType::Acknowledged;
        self.last_updated = Utc::now();
    }
    
    /// Mark the alert as resolved
    pub fn resolve(&mut self) {
        self.status = AlertStatusType::Resolved;
        self.last_updated = Utc::now();
    }
} 