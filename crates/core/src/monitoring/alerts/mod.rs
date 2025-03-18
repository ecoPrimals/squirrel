//! Alert management for system monitoring
//! 
//! This module provides functionality for:
//! - Alert rule management
//! - Alert generation and notification
//! - Alert history tracking
//! - Alert severity levels
//! - Alert routing and escalation

// Allow certain linting issues that are too numerous to fix individually
#![allow(clippy::module_name_repetitions)] // Allow module name in type names
#![allow(clippy::unused_async)] // Allow unused async functions

use serde::{Serialize, Deserialize};
use std::{
    collections::HashMap,
    sync::Arc,
    fmt::Debug,
};
use tokio::sync::RwLock;
use thiserror::Error;
use uuid::Uuid;
use crate::error::{Result, SquirrelError};
use std::time::{SystemTime, UNIX_EPOCH};
use async_trait::async_trait;
use time::OffsetDateTime;

pub mod config;
pub mod manager;
pub mod notify;
pub mod status;
pub mod adapter;

/// Alert severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AlertSeverity {
    /// Critical alerts require immediate attention
    Critical,
    /// High severity alerts should be addressed soon
    High,
    /// Medium severity alerts should be investigated
    Medium,
    /// Low severity alerts are informational
    Low,
}

impl std::fmt::Display for AlertSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Critical => write!(f, "Critical"),
            Self::High => write!(f, "High"),
            Self::Medium => write!(f, "Medium"),
            Self::Low => write!(f, "Low"),
        }
    }
}

impl AlertSeverity {
    /// Get the color associated with this severity level
    #[must_use] pub const fn color(&self) -> &'static str {
        match self {
            Self::Critical => "red",
            Self::High => "orange",
            Self::Medium => "yellow",
            Self::Low => "blue",
        }
    }
}

/// Alert status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AlertStatus {
    /// Alert is active and unacknowledged
    Active,
    /// Alert has been acknowledged
    Acknowledged,
    /// Alert has been resolved
    Resolved,
}

impl std::fmt::Display for AlertStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => write!(f, "Active"),
            Self::Acknowledged => write!(f, "Acknowledged"),
            Self::Resolved => write!(f, "Resolved"),
        }
    }
}

/// Alert notification for a system event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    /// Unique identifier for the alert
    pub id: String,
    /// Short name describing the alert
    pub name: String,
    /// Detailed description of the alert
    pub description: String,
    /// Severity level of the alert
    pub severity: AlertSeverity,
    /// Current status of the alert
    pub status: AlertStatus,
    /// Key-value pairs for additional alert metadata
    pub labels: HashMap<String, String>,
    /// Unix timestamp when the alert was created
    pub created_at: i64,
    /// Unix timestamp when the alert was last updated
    pub updated_at: i64,
    /// Human-readable message describing the alert
    pub message: String,
    /// System component that generated the alert
    pub component: String,
}

impl Alert {
    /// Creates a new alert with the given parameters
    ///
    /// # Arguments
    /// * `name` - Short name describing the alert
    /// * `description` - Detailed description of the alert
    /// * `severity` - Severity level of the alert
    /// * `labels` - Key-value pairs for additional alert metadata
    /// * `message` - Human-readable message describing the alert
    /// * `component` - System component that generated the alert
    #[must_use] pub fn new(
        name: String,
        description: String,
        severity: AlertSeverity,
        labels: HashMap<String, String>,
        message: String,
        component: String,
    ) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        Self {
            id: Uuid::new_v4().to_string(),
            name,
            description,
            severity,
            status: AlertStatus::Active,
            labels,
            created_at: now,
            updated_at: now,
            message,
            component,
        }
    }

    /// Updates the status of the alert and sets the updated_at timestamp
    ///
    /// # Arguments
    /// * `status` - The new status to set for the alert
    pub fn update_status(&mut self, status: AlertStatus) {
        self.status = status;
        self.updated_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
    }
}

/// Configuration for the alert management system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertConfig {
    /// Whether to enable alerts
    pub enabled: bool,
    /// Minimum severity level to trigger alerts
    pub min_severity: AlertSeverity,
    /// Maximum number of alerts to retain
    pub max_alerts: usize,
}

impl Default for AlertConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_severity: AlertSeverity::Low,
            max_alerts: 1000,
        }
    }
}

/// Alert notification data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertNotification {
    /// Alert ID
    pub id: String,
    /// Alert name
    pub name: String,
    /// Alert description
    pub description: String,
    /// Alert severity
    pub severity: AlertSeverity,
    /// Alert status
    pub status: AlertStatus,
    /// Alert labels
    pub labels: HashMap<String, String>,
    /// Alert created timestamp
    pub created_at: i64,
    /// Alert updated timestamp
    pub updated_at: i64,
    /// Alert message
    pub message: String,
    /// Alert component
    pub component: String,
}

impl From<Alert> for AlertNotification {
    fn from(alert: Alert) -> Self {
        Self {
            id: alert.id.clone(),
            name: alert.name.clone(),
            description: alert.description.clone(),
            severity: alert.severity.clone(),
            status: alert.status.clone(),
            labels: alert.labels.clone(),
            created_at: alert.created_at,
            updated_at: alert.updated_at,
            message: alert.message.clone(),
            component: alert.component,
        }
    }
}

/// Alert manager trait
#[async_trait]
pub trait AlertManager: Debug + Send + Sync {
    /// Sends an alert through configured notification channels and stores it
    ///
    /// # Arguments
    /// * `alert` - The alert to send and store
    ///
    /// # Errors
    /// This function may return errors if:
    /// * The notification system fails to send the alert
    /// * There are issues storing the alert in the internal storage
    async fn send_alert(&self, alert: Alert) -> Result<()>;
    
    /// Retrieves all stored alerts
    ///
    /// # Returns
    /// A vector containing all stored alerts
    ///
    /// # Errors
    /// This function may return errors if there are issues accessing the internal alert storage
    async fn get_alerts(&self) -> Result<Vec<Alert>>;
    
    /// Adds a new alert to the storage without sending notifications
    ///
    /// # Arguments
    /// * `alert` - The alert to add to storage
    ///
    /// # Errors
    /// This function may return errors if there are issues accessing the internal alert storage
    async fn add_alert(&self, alert: Alert) -> Result<()>;
    
    /// Updates an existing alert in the storage
    ///
    /// # Arguments
    /// * `alert` - The alert with updated fields
    ///
    /// # Errors
    /// This function may return errors if:
    /// * The alert does not exist
    /// * There are issues accessing the internal alert storage
    async fn update_alert(&self, alert: Alert) -> Result<()>;
    
    /// Starts the alert manager and initializes resources
    ///
    /// # Errors
    /// This function may return errors if there are issues initializing resources
    async fn start(&self) -> Result<()>;
    
    /// Stops the alert manager and releases resources
    ///
    /// # Errors
    /// This function may return errors if there are issues releasing resources
    async fn stop(&self) -> Result<()>;
}

/// Default alert manager implementation
#[derive(Debug)]
pub struct DefaultAlertManager {
    alerts: Arc<RwLock<Vec<Alert>>>,
    config: AlertConfig,
    notification_manager: Option<Arc<NotificationManager>>,
}

impl DefaultAlertManager {
    /// Creates a new default alert manager with the specified configuration
    ///
    /// # Arguments
    /// * `config` - The alert configuration settings
    ///
    /// # Returns
    /// A new instance of `DefaultAlertManager` initialized with the provided configuration
    #[must_use] pub fn new(config: AlertConfig) -> Self {
        Self {
            alerts: Arc::new(RwLock::new(Vec::new())),
            config,
            notification_manager: None,
        }
    }

    /// Creates a new alert manager with dependencies
    ///
    /// # Arguments
    /// * `config` - The alert configuration settings
    /// * `notification_manager` - Optional notification manager for sending alerts
    ///
    /// # Returns
    /// A new instance of `DefaultAlertManager` with the specified dependencies
    #[must_use] pub fn with_dependencies(
        config: AlertConfig,
        notification_manager: Option<Arc<NotificationManager>>,
    ) -> Self {
        Self {
            alerts: Arc::new(RwLock::new(Vec::new())),
            config,
            notification_manager,
        }
    }
}

impl Default for DefaultAlertManager {
    fn default() -> Self {
        Self::new(AlertConfig::default())
    }
}

#[async_trait]
impl AlertManager for DefaultAlertManager {
    /// Sends an alert through configured notification channels and stores it
    ///
    /// # Arguments
    /// * `alert` - The alert to send and store
    ///
    /// # Errors
    /// This function may return errors if:
    /// * The notification system fails to send the alert
    /// * There are issues storing the alert in the internal storage
    async fn send_alert(&self, alert: Alert) -> Result<()> {
        // Store the alert
        self.add_alert(alert.clone()).await?;

        // Send notification if configured
        if let Some(notification_manager) = &self.notification_manager {
            let notification = AlertNotification::from(alert);
            if let Err(e) = notification_manager.send_notification(&notification).await {
                log::error!("Failed to send notification: {}", e);
            }
        }

        Ok(())
    }

    /// Retrieves all stored alerts
    ///
    /// # Returns
    /// A vector containing all stored alerts
    ///
    /// # Errors
    /// This function may return errors if there are issues accessing the internal alert storage
    async fn get_alerts(&self) -> Result<Vec<Alert>> {
        let alerts = self.alerts.read().await;
        Ok(alerts.clone())
    }

    /// Adds a new alert to the storage without sending notifications
    ///
    /// # Arguments
    /// * `alert` - The alert to add to storage
    ///
    /// # Errors
    /// This function may return errors if there are issues accessing the internal alert storage
    async fn add_alert(&self, alert: Alert) -> Result<()> {
        let mut alerts = self.alerts.write().await;
        alerts.push(alert);
        
        // Enforce max alerts limit
        if alerts.len() > self.config.max_alerts {
            alerts.sort_by(|a, b| b.created_at.cmp(&a.created_at));
            alerts.truncate(self.config.max_alerts);
        }
        
        Ok(())
    }

    /// Updates an existing alert in the storage
    ///
    /// # Arguments
    /// * `alert` - The alert with updated fields
    ///
    /// # Errors
    /// This function may return errors if:
    /// * The alert does not exist
    /// * There are issues accessing the internal alert storage
    async fn update_alert(&self, alert: Alert) -> Result<()> {
        let mut alerts = self.alerts.write().await;
        if let Some(idx) = alerts.iter().position(|a| a.id == alert.id) {
            alerts[idx] = alert;
            Ok(())
        } else {
            Err(SquirrelError::alert(&format!("Alert not found: {alert_id}", alert_id = alert.id)))
        }
    }

    /// Starts the alert manager and initializes resources
    ///
    /// # Errors
    /// This function may return errors if there are issues initializing resources
    async fn start(&self) -> Result<()> {
        Ok(())
    }

    /// Stops the alert manager and releases resources
    ///
    /// # Errors
    /// This function may return errors if there are issues releasing resources
    async fn stop(&self) -> Result<()> {
        Ok(())
    }
}

/// Errors that can occur during alert operations
#[derive(Error, Debug)]
pub enum AlertError {
    /// The requested alert could not be found
    #[error("Alert not found: {0}")]
    NotFound(String),
    
    /// The alert data is invalid or malformed
    #[error("Invalid alert: {0}")]
    InvalidAlert(String),
    
    /// An error occurred while accessing or modifying alert storage
    #[error("Storage error: {0}")]
    StorageError(String),
    
    /// An error occurred while sending alert notifications
    #[error("Notification error: {0}")]
    NotificationError(String),
}

impl From<AlertError> for SquirrelError {
    fn from(err: AlertError) -> Self {
        Self::alert(&err.to_string())
    }
}

/// Factory for creating alert managers
#[derive(Debug, Clone)]
pub struct AlertManagerFactory {
    /// Configuration for creating alert managers
    config: AlertConfig,
}

impl AlertManagerFactory {
    /// Creates a new factory with default configuration
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: AlertConfig::default(),
        }
    }

    /// Creates a new factory with specific configuration
    #[must_use]
    pub const fn with_config(config: AlertConfig) -> Self {
        Self { config }
    }

    /// Creates an alert manager with dependencies
    ///
    /// # Arguments
    /// * `notification_manager` - Optional notification manager for sending alerts
    ///
    /// # Returns
    /// A new alert manager instance with the specified dependencies
    #[must_use]
    pub fn create_manager_with_dependencies(
        &self,
        notification_manager: Option<Arc<NotificationManager>>,
    ) -> Arc<DefaultAlertManager> {
        Arc::new(DefaultAlertManager::with_dependencies(
            self.config.clone(),
            notification_manager,
        ))
    }

    /// Creates an alert manager with default configuration
    #[must_use]
    pub fn create_manager(&self) -> Arc<DefaultAlertManager> {
        self.create_manager_with_dependencies(None)
    }

    /// Creates an alert manager adapter
    #[must_use]
    pub fn create_manager_adapter(&self) -> Arc<AlertManagerAdapter> {
        let manager = self.create_manager();
        create_manager_adapter_with_manager(manager)
    }
}

impl Default for AlertManagerFactory {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new alert manager adapter
#[must_use]
pub fn create_manager_adapter() -> Arc<AlertManagerAdapter> {
    AlertManagerFactory::new().create_manager_adapter()
}

/// Create a new alert manager adapter with a specific manager
#[must_use]
pub fn create_manager_adapter_with_manager(
    manager: Arc<DefaultAlertManager>
) -> Arc<AlertManagerAdapter> {
    Arc::new(AlertManagerAdapter::with_manager(manager))
}

// Re-export adapter types
pub use adapter::{AlertManagerAdapter, create_manager_adapter, create_manager_adapter_with_manager};

// Re-export modules
pub use self::config::*;
pub use self::manager::*;
pub use self::notify::*;
pub use self::status::*;
pub use self::adapter::*;

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_alert_manager_basic() {
        let manager = DefaultAlertManager::new(AlertConfig::default());
        
        // Create test alert
        let alert = Alert::new(
            "Test Alert".to_string(),
            "Test Description".to_string(),
            AlertSeverity::High,
            HashMap::new(),
            "Test Message".to_string(),
            "test".to_string(),
        );
        
        // Send alert
        manager.send_alert(alert.clone()).await.unwrap();
        
        // Get alerts
        let alerts = manager.get_alerts().await.unwrap();
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].name, "Test Alert");
        
        // Update alert status
        let mut updated_alert = alert;
        updated_alert.update_status(AlertStatus::Acknowledged);
        manager.update_alert(updated_alert.clone()).await.unwrap();
        
        // Verify update
        let alerts = manager.get_alerts().await.unwrap();
        assert_eq!(alerts[0].status, AlertStatus::Acknowledged);
    }

    #[tokio::test]
    async fn test_alert_manager_adapter() {
        let factory = AlertManagerFactory::new();
        let adapter = factory.create_manager_adapter();
        
        // Create test alert
        let alert = Alert::new(
            "Test Alert".to_string(),
            "Test Description".to_string(),
            AlertSeverity::High,
            HashMap::new(),
            "Test Message".to_string(),
            "test".to_string(),
        );
        
        // Send alert
        adapter.send_alert(alert.clone()).await.unwrap();
        
        // Get alerts
        let alerts = adapter.get_alerts().await.unwrap();
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].name, "Test Alert");
        
        // Update alert status
        let mut updated_alert = alert;
        updated_alert.update_status(AlertStatus::Acknowledged);
        adapter.update_alert(updated_alert).await.unwrap();
        
        // Verify update
        let alerts = adapter.get_alerts().await.unwrap();
        assert_eq!(alerts[0].status, AlertStatus::Acknowledged);
    }

    #[tokio::test]
    async fn test_alert_manager_with_config() {
        let config = AlertConfig {
            enabled: true,
            min_severity: AlertSeverity::High,
            max_alerts: 5,
        };
        
        let factory = AlertManagerFactory::with_config(config.clone());
        let manager = factory.create_manager();
        
        // Create alerts with different severities
        let high_alert = Alert::new(
            "High Alert".to_string(),
            "High Description".to_string(),
            AlertSeverity::High,
            HashMap::new(),
            "High Message".to_string(),
            "test".to_string(),
        );
        
        let medium_alert = Alert::new(
            "Medium Alert".to_string(),
            "Medium Description".to_string(),
            AlertSeverity::Medium,
            HashMap::new(),
            "Medium Message".to_string(),
            "test".to_string(),
        );
        
        // Send alerts
        manager.send_alert(high_alert).await.unwrap();
        manager.send_alert(medium_alert).await.unwrap();
        
        // Only high severity alert should be stored
        let alerts = manager.get_alerts().await.unwrap();
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].severity, AlertSeverity::High);
    }
} 