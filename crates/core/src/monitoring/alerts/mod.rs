/// Module for monitoring alert functionality
///
/// This module provides alert generation, management, and notification capabilities
/// for system monitoring.
// Allow certain linting issues that are too numerous to fix individually
#[allow(clippy::module_name_repetitions)] // Allow module name in type names
#[allow(clippy::unused_async)] // Allow unused async functions

use serde::{Serialize, Deserialize};
use uuid::Uuid;
use std::sync::Arc;
use tokio::sync::RwLock;
use thiserror::Error;
use async_trait::async_trait;
use crate::error::{Result, SquirrelError};
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

/// Module for adapter implementations of alert functionality
pub mod adapter;

/// Module for alert notification functionality
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
    /// * `alert` - The alert to add to storage
    ///
    /// # Errors
    /// This function may return errors if there are issues accessing the internal alert storage
    async fn add_alert(&self, alert: Alert) -> Result<()> {
        // Skip alerts below the minimum severity threshold
        if !self.config.enabled || severity_value(alert.severity) < severity_value(self.config.min_severity) {
            log::debug!("Alert filtered out due to severity: {:?}", alert.severity);
            return Ok(());
        }
        
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
            Err(SquirrelError::alert(format!("Alert not found: {alert_id}", alert_id = alert.id)))
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

// Helper function to get numeric values for severity levels for comparison
/// Converts an alert severity enum to a numeric value for comparison
/// 
/// This function is used internally to compare severity levels,
/// with higher values indicating more severe alerts.
fn severity_value(severity: AlertSeverity) -> u8 {
    match severity {
        AlertSeverity::Critical => 5,
        AlertSeverity::High => 4,
        AlertSeverity::Medium => 3,
        AlertSeverity::Warning => 2,
        AlertSeverity::Low => 1,
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
        Self::alert(err.to_string())
    }
}

/// Factory for creating alert managers
#[derive(Debug, Clone)]
pub struct AlertManagerFactory<N: NotificationManagerTrait + 'static = ()> {
    /// Configuration for creating alert managers
    config: AlertConfig,
    /// Phantom type parameter for notification manager type
    _phantom: std::marker::PhantomData<N>,
}

impl<N: NotificationManagerTrait + 'static> AlertManagerFactory<N> {
    /// Creates a new factory with default configuration
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: AlertConfig::default(),
            _phantom: std::marker::PhantomData,
        }
    }

    /// Creates a new factory with specific configuration
    #[must_use]
    pub const fn with_config(config: AlertConfig) -> Self {
        Self { 
            config,
            _phantom: std::marker::PhantomData,
        }
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
        notification_manager: Option<Arc<N>>,
    ) -> Arc<DefaultAlertManager<N>> {
        Arc::new(DefaultAlertManager::with_dependencies(
            self.config.clone(),
            notification_manager,
        ))
    }

    /// Creates an alert manager with default configuration
    #[must_use]
    pub fn create_manager(&self) -> Arc<DefaultAlertManager<N>> {
        self.create_manager_with_dependencies(None)
    }

    /// Creates an alert manager adapter
    #[must_use]
    pub fn create_manager_adapter(&self) -> Arc<AlertManagerAdapter<N>> {
        let manager = self.create_manager();
        create_manager_adapter_with_manager(manager)
    }
}

impl<N: NotificationManagerTrait + 'static> Default for AlertManagerFactory<N> {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new alert manager adapter
#[must_use]
pub fn create_manager_adapter() -> Arc<AlertManagerAdapter<()>> {
    match create_initialized_manager_adapter() {
        Ok(adapter) => adapter,
        Err(e) => {
            log::error!("Failed to initialize AlertManagerAdapter: {}", e);
            // Fall back to uninitialized adapter, but log the error
            Arc::new(AlertManagerAdapter::new())
        }
    }
}

/// Create a new alert manager adapter with an existing manager
#[must_use]
pub fn create_manager_adapter_with_manager<N: NotificationManagerTrait + 'static>(
    manager: Arc<DefaultAlertManager<N>>
) -> Arc<AlertManagerAdapter<N>> {
    Arc::new(AlertManagerAdapter::with_manager(manager))
}

// Re-export adapter types
pub use adapter::AlertManagerAdapter;

// Public re-exports
pub use self::notify::*;
pub use self::adapter::*;

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_alert_manager_basic() {
        // Create a new alert manager with default configuration
        let manager: DefaultAlertManager<()> = DefaultAlertManager::new(AlertConfig::default());
        
        // Create a test alert
        let mut labels = HashMap::new();
        labels.insert("service".to_string(), "test".to_string());
        
        let alert = Alert::new(
            "Test Alert".to_string(),
            "This is a test alert".to_string(),
            AlertSeverity::Medium,
            labels,
            "Test alert message".to_string(),
            "test-component".to_string(),
        );
        
        // Send the alert
        assert!(manager.add_alert(alert.clone()).await.is_ok());
        
        // Get all alerts
        let alerts = manager.get_alerts().await.unwrap();
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].name, "Test Alert");
    }

    #[tokio::test]
    async fn test_alert_manager_adapter() {
        // Create manager adapter
        let adapter = create_manager_adapter();
        
        // Create test alert
        let alert = Alert::new(
            "Test Alert".to_string(),
            "Test Description".to_string(),
            AlertSeverity::Medium,
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
    }

    #[tokio::test]
    async fn test_alert_manager_with_config() {
        // Create custom config
        let config = AlertConfig {
            enabled: true,
            min_severity: AlertSeverity::High,
            max_alerts: 50,
        };
        
        // Create factory with explicit type annotation
        let factory: AlertManagerFactory<()> = AlertManagerFactory::with_config(config.clone());
        
        // Create manager
        let manager = factory.create_manager();
        
        // Create test alert (low severity - should be filtered)
        let low_alert = Alert::new(
            "Low Alert".to_string(),
            "Low severity test".to_string(),
            AlertSeverity::Low,
            HashMap::new(),
            "Test Message".to_string(),
            "test".to_string(),
        );
        
        // Create high severity alert (should pass filter)
        let high_alert = Alert::new(
            "High Alert".to_string(),
            "High severity test".to_string(),
            AlertSeverity::High,
            HashMap::new(),
            "Test Message".to_string(),
            "test".to_string(),
        );
        
        // Test alerts with different severities
        println!("Sending low severity alert");
        manager.send_alert(low_alert).await.unwrap();
        
        println!("Sending high severity alert");
        manager.send_alert(high_alert).await.unwrap();
        
        // Verify filtering (low severity should be filtered out)
        let alerts = manager.get_alerts().await.unwrap();
        println!("Retrieved {} alerts: {:?}", alerts.len(), alerts.iter().map(|a| &a.name).collect::<Vec<_>>());
        
        // Given our min_severity is High, we should only see the high alert
        assert_eq!(alerts.len(), 1, "Expected exactly 1 alert (High), but got {} alerts", alerts.len());
        
        // Make sure it's the High severity alert that passed through
        assert_eq!(alerts[0].name, "High Alert", "Expected High alert, but got {}", alerts[0].name);
        assert_eq!(alerts[0].severity, AlertSeverity::High, "Alert should have High severity");
    }
} 