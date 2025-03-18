use std::sync::Arc;
use crate::error::Result;
use super::{Alert, AlertManager, DefaultAlertManager};
use async_trait::async_trait;
use super::{NotificationManager, NotificationConfig, AlertNotification, NotificationError};

/// Adapter for the alert manager to support dependency injection
#[derive(Debug)]
pub struct AlertManagerAdapter {
    inner: Option<Arc<DefaultAlertManager>>,
}

impl AlertManagerAdapter {
    /// Creates a new alert manager adapter
    #[must_use]
    pub fn new() -> Self {
        Self { inner: None }
    }

    /// Creates a new adapter with an existing alert manager
    #[must_use]
    pub fn with_manager(manager: Arc<DefaultAlertManager>) -> Self {
        Self {
            inner: Some(manager),
        }
    }

    /// Sends an alert through configured notification channels and stores it
    ///
    /// # Errors
    /// Returns an error if the alert cannot be sent or if no manager is available
    pub async fn send_alert(&self, alert: Alert) -> Result<()> {
        if let Some(manager) = &self.inner {
            manager.send_alert(alert).await
        } else {
            Err(format!("Alert manager not initialized via dependency injection").into())
        }
    }

    /// Adds a new alert to the storage without sending notifications
    ///
    /// # Errors
    /// Returns an error if the alert cannot be added or if no manager is available
    pub async fn add_alert(&self, alert: Alert) -> Result<()> {
        if let Some(manager) = &self.inner {
            manager.add_alert(alert).await
        } else {
            Err(format!("Alert manager not initialized via dependency injection").into())
        }
    }

    /// Updates an existing alert in the storage
    ///
    /// # Errors
    /// Returns an error if the alert cannot be updated or if no manager is available
    pub async fn update_alert(&self, alert: Alert) -> Result<()> {
        if let Some(manager) = &self.inner {
            manager.update_alert(alert).await
        } else {
            Err(format!("Alert manager not initialized via dependency injection").into())
        }
    }

    /// Retrieves all stored alerts
    ///
    /// # Errors
    /// Returns an error if the alerts cannot be retrieved or if no manager is available
    pub async fn get_alerts(&self) -> Result<Vec<Alert>> {
        if let Some(manager) = &self.inner {
            manager.get_alerts().await
        } else {
            Err(format!("Alert manager not initialized via dependency injection").into())
        }
    }
}

#[async_trait]
impl AlertManager for AlertManagerAdapter {
    async fn send_alert(&self, alert: Alert) -> Result<()> {
        self.send_alert(alert).await
    }

    async fn get_alerts(&self) -> Result<Vec<Alert>> {
        self.get_alerts().await
    }

    async fn add_alert(&self, alert: Alert) -> Result<()> {
        self.add_alert(alert).await
    }

    async fn update_alert(&self, alert: Alert) -> Result<()> {
        self.update_alert(alert).await
    }

    async fn start(&self) -> Result<()> {
        if let Some(manager) = &self.inner {
            manager.start().await
        } else {
            Err(format!("Alert manager not initialized via dependency injection").into())
        }
    }

    async fn stop(&self) -> Result<()> {
        if let Some(manager) = &self.inner {
            manager.stop().await
        } else {
            Err(format!("Alert manager not initialized via dependency injection").into())
        }
    }
}

impl Clone for AlertManagerAdapter {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl Default for AlertManagerAdapter {
    fn default() -> Self {
        Self::new()
    }
}

/// Creates a new alert manager adapter
#[must_use]
pub fn create_manager_adapter() -> Arc<AlertManagerAdapter> {
    Arc::new(AlertManagerAdapter::new())
}

/// Creates a new alert manager adapter with an existing manager
#[must_use]
pub fn create_manager_adapter_with_manager(manager: Arc<DefaultAlertManager>) -> Arc<AlertManagerAdapter> {
    Arc::new(AlertManagerAdapter::with_manager(manager))
}

/// Adapter for the Notification Manager to provide backward compatibility 
/// during the transition to dependency injection.
#[derive(Debug, Clone)]
pub struct NotificationManagerAdapter {
    /// Inner Notification Manager instance
    inner: Arc<NotificationManager>,
}

impl NotificationManagerAdapter {
    /// Creates a new adapter with an existing manager
    #[must_use]
    pub const fn with_manager(manager: Arc<NotificationManager>) -> Self {
        Self { inner: manager }
    }

    /// Get the inner manager
    #[must_use]
    pub fn inner(&self) -> Arc<NotificationManager> {
        self.inner.clone()
    }

    /// Sends a notification for the given alert
    ///
    /// # Errors
    /// Returns a error if the notification cannot be sent
    pub async fn send_notification(&self, alert: &AlertNotification) -> Result<(), NotificationError> {
        self.inner.send_notification(alert).await
    }

    /// Updates the notification configuration
    ///
    /// # Errors
    /// Returns an error if the configuration is invalid
    pub async fn update_config(&self, config: NotificationConfig) -> Result<(), NotificationError> {
        self.inner.update_config(config).await
    }
}

/// Creates a notification manager adapter with a new manager
///
/// # Errors
/// Returns an error if the notification manager cannot be created
pub fn create_notification_manager_adapter(
    config: NotificationConfig
) -> Result<Arc<NotificationManagerAdapter>, NotificationError> {
    let manager = NotificationManager::new(config)?;
    Ok(Arc::new(NotificationManagerAdapter::with_manager(Arc::new(manager))))
}

/// Creates a notification manager adapter with an existing manager
#[must_use]
pub fn create_notification_manager_adapter_with_manager(
    manager: Arc<NotificationManager>
) -> Arc<NotificationManagerAdapter> {
    Arc::new(NotificationManagerAdapter::with_manager(manager))
} 