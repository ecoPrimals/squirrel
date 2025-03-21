/// Module for monitoring alert functionality
///
/// This module provides alert generation, management, and notification capabilities
/// for system monitoring.

use serde::{Serialize, Deserialize};
use uuid::Uuid;
use std::sync::Arc;
use tokio::sync::RwLock;
use async_trait::async_trait;
use squirrel_core::error::{Result, SquirrelError};
use std::collections::HashMap;
use std::fmt::{Debug};
use std::time::{SystemTime, UNIX_EPOCH};
use log;

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

/// Alert severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AlertSeverity {
    /// Critical alerts require immediate attention
    Critical,
    /// High severity alerts should be addressed soon
    High,
    /// Medium severity alerts should be investigated
    Medium,
    /// Warning severity alerts need attention
    Warning,
    /// Low severity alerts are informational
    Low,
}

impl std::fmt::Display for AlertSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Critical => write!(f, "Critical"),
            Self::High => write!(f, "High"),
            Self::Medium => write!(f, "Medium"),
            Self::Warning => write!(f, "Warning"),
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
            Self::Medium | Self::Warning => "yellow",
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

    /// Updates the status of the alert and sets the `updated_at` timestamp
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

/// Alert configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertConfig {
    /// Enable alert generation
    pub enabled: bool,
    /// Alert check interval in seconds
    pub interval: u64,
    /// Maximum number of alerts to store
    pub max_alerts: usize,
    /// Minimum severity level for alerts to be processed
    pub min_severity: AlertSeverity,
}

impl Default for AlertConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval: 60,
            max_alerts: 1000,
            min_severity: AlertSeverity::Low,
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
            severity: alert.severity,
            status: alert.status,
            labels: alert.labels.clone(),
            created_at: alert.created_at,
            updated_at: alert.updated_at,
            message: alert.message.clone(),
            component: alert.component,
        }
    }
}

/// Trait for notification managers that can send alert notifications
#[async_trait]
pub trait NotificationManagerTrait: Send + Sync + Debug {
    /// Send a notification about an alert
    ///
    /// # Arguments
    /// * `notification` - The notification to send
    ///
    /// # Returns
    /// A result indicating success or error
    async fn send_notification(&self, notification: &AlertNotification) -> Result<()>;
}

/// Implementation for unit type to allow () to be used as a default notification manager
#[async_trait]
impl NotificationManagerTrait for () {
    /// No-op implementation that just returns success
    async fn send_notification(&self, _notification: &AlertNotification) -> Result<()> {
        // Unit type doesn't actually send notifications
        Ok(())
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
    /// * `alert` - The alert to add
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

/// Default implementation of the alert manager
#[derive(Debug)]
pub struct DefaultAlertManager<N: NotificationManagerTrait + 'static = ()> {
    /// Storage for alert records
    alerts: Arc<RwLock<Vec<Alert>>>,
    /// Alert manager configuration
    config: AlertConfig,
    /// Optional notification manager for sending alerts
    notification_manager: Option<Arc<N>>,
}

impl<N: NotificationManagerTrait + 'static> DefaultAlertManager<N> {
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
        notification_manager: Option<Arc<N>>,
    ) -> Self {
        Self {
            alerts: Arc::new(RwLock::new(Vec::new())),
            config,
            notification_manager,
        }
    }
}

impl<N: NotificationManagerTrait + 'static> Default for DefaultAlertManager<N> {
    fn default() -> Self {
        Self::new(AlertConfig::default())
    }
}

#[async_trait]
impl<N: NotificationManagerTrait + 'static> AlertManager for DefaultAlertManager<N> {
    /// Sends an alert through configured notification channels and stores it
    ///
    /// # Arguments
    /// * `alert` - The alert to send
    ///
    /// # Returns
    /// A Result indicating success or containing an error if:
    /// * There are issues sending the notification through configured channels
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
    /// * `alert` - The alert to add
    ///
    /// # Errors
    /// This function may return errors if there are issues accessing the internal alert storage
    async fn add_alert(&self, alert: Alert) -> Result<()> {
        let mut alerts = self.alerts.write().await;
        alerts.push(alert);
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
        if let Some(index) = alerts.iter().position(|a| a.id == alert.id) {
            alerts[index] = alert;
            Ok(())
        } else {
            Err(SquirrelError::generic("Alert not found"))
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