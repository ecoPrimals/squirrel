// Alert configuration module

use serde::{Serialize, Deserialize};
use squirrel_core::error::Result;
use std::fmt::Debug;
use std::collections::HashMap;
use std::time::Duration;
use super::status::AlertSeverity;

/// Configuration for the alert system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertConfig {
    /// Whether the alert system is enabled
    pub enabled: bool,
    /// Minimum severity level that will generate notifications
    pub severity_threshold: AlertSeverity,
    /// How often to check for alerts (in seconds)
    pub check_interval: u64,
    /// Maximum number of alerts to keep in history
    pub history_limit: usize,
    /// Notification channels to use for alerts
    pub notification_channels: Vec<NotificationChannel>,
    /// Custom settings for specific alert types
    pub custom_settings: HashMap<String, serde_json::Value>,
}

/// Notification channels for alerts
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum NotificationChannel {
    /// Console output
    Console,
    /// Log file
    Log,
    /// Email notification
    Email,
    /// Webhook callback
    Webhook,
    /// Export to metrics system
    Metrics,
}

impl AlertConfig {
    /// Create a new alert configuration with default settings
    #[must_use] pub fn new() -> Self {
        Self {
            enabled: true,
            severity_threshold: AlertSeverity::Warning,
            check_interval: 30,
            history_limit: 1000,
            notification_channels: vec![NotificationChannel::Console, NotificationChannel::Log],
            custom_settings: HashMap::new(),
        }
    }
    
    /// Load configuration from storage
    pub fn load() -> Result<Self> {
        // For now, just return default config
        // In a real implementation, this would load from a config file or database
        Ok(Self::new())
    }
    
    /// Save configuration to storage
    pub fn save(&self) -> Result<()> {
        // In a real implementation, this would save to a config file or database
        Ok(())
    }
    
    /// Get the check interval as a Duration
    #[must_use] pub fn check_interval_duration(&self) -> Duration {
        Duration::from_secs(self.check_interval)
    }
    
    /// Check if notifications should be sent for a given severity
    #[must_use] pub fn should_notify(&self, severity: AlertSeverity) -> bool {
        self.enabled && match (self.severity_threshold, severity) {
            (AlertSeverity::Info, _) => true,
            (AlertSeverity::Warning, AlertSeverity::Info) => false,
            (AlertSeverity::Warning, _) => true,
            (AlertSeverity::Error, AlertSeverity::Info | AlertSeverity::Warning) => false,
            (AlertSeverity::Error, _) => true,
            (AlertSeverity::Critical, AlertSeverity::Critical) => true,
            _ => false,
        }
    }
    
    /// Get custom settings for a specific alert type
    #[must_use] pub fn get_custom_setting(&self, alert_type: &str) -> Option<&serde_json::Value> {
        self.custom_settings.get(alert_type)
    }
}

impl Default for AlertConfig {
    fn default() -> Self {
        Self::new()
    }
} 