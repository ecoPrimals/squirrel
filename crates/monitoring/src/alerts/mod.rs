/// Module for monitoring alert functionality
///
/// This module provides alert generation, management, and notification capabilities
use serde::{Serialize, Deserialize};
use async_trait::async_trait;
use squirrel_core::error::Result;
use std::collections::HashMap;
use std::fmt::{Debug};

/// Module for alert configuration
pub mod config;

/// Module for alert manager implementations
pub mod manager;

/// Module for alert status tracking
pub mod status;

/// Module for alert adapters
pub mod adapter;

/// Module for notification management
pub mod notify;

/// Module for alert type definitions
pub mod types;

/// Re-export common types
pub use status::AlertStatus;
pub use types::{Alert, AlertLevel};

/// Alert notification data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertNotification {
    /// Alert ID
    pub id: String,
    /// Alert name
    pub name: String,
    /// Alert description
    pub description: String,
    /// Alert severity
    pub severity: LegacyAlertSeverity,
    /// Alert status
    pub status: String,
    /// Alert message
    pub message: String,
    /// Alert source/component
    pub source: String,
    /// Component that generated the alert
    pub component: String,
    /// Alert timestamp
    pub timestamp: u64,
    /// Created timestamp
    pub created_at: i64,
    /// Updated timestamp
    pub updated_at: i64,
    /// Alert labels
    pub labels: HashMap<String, String>,
    /// Alert details
    pub details: HashMap<String, serde_json::Value>,
}

/// Legacy alert severity (for backwards compatibility)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LegacyAlertSeverity {
    /// Informational severity
    Info,
    /// Warning severity
    Warning,
    /// Error severity
    Error,
    /// Critical severity
    Critical,
}

/// Trait for notification manager
#[async_trait]
pub trait NotificationManagerTrait: Send + Sync + Debug {
    /// Send an alert notification
    async fn send_notification(&self, alert: &AlertNotification) -> Result<()>;
    
    /// Update notification configuration
    async fn update_config(&self, config: HashMap<String, serde_json::Value>) -> Result<()>;
} 