use std::sync::Arc;
use std::fmt::Debug;
use crate::error::{Result, SquirrelError};
use super::{NotificationManager, NotificationConfig, NotificationError};
use super::{Alert, AlertManager, DefaultAlertManager, AlertNotification, NotificationManagerTrait};
use async_trait::async_trait;

/// Adapter for the alert manager to support dependency injection
#[derive(Debug)]
pub struct AlertManagerAdapter<T: NotificationManagerTrait + 'static = ()> {
    inner: Option<Arc<DefaultAlertManager<T>>>,
}

impl<T: NotificationManagerTrait + 'static> AlertManagerAdapter<T> {
    /// Creates a new adapter without initializing it
    #[must_use] pub fn new() -> Self {
        Self { inner: None }
    }
    
    /// Creates an adapter with an existing manager
    #[must_use] pub fn with_manager(manager: Arc<DefaultAlertManager<T>>) -> Self {
        Self {
            inner: Some(manager),
        }
    }

    /// Send an alert through the manager
    ///
    /// # Errors
    /// Returns an error if the manager is not initialized or if the alert cannot be sent
    pub async fn send_alert(&self, alert: Alert) -> Result<()> {
        match &self.inner {
            Some(manager) => manager.send_alert(alert).await,
            None => Err(SquirrelError::alert("AlertManager not initialized")),
        }
    }

    /// Add an alert to the manager
    ///
    /// # Errors
    /// Returns an error if the manager is not initialized or if the alert cannot be added
    pub async fn add_alert(&self, alert: Alert) -> Result<()> {
        match &self.inner {
            Some(manager) => manager.add_alert(alert).await,
            None => Err(SquirrelError::alert("AlertManager not initialized")),
        }
    }

    /// Update an alert in the manager
    ///
    /// # Errors
    /// Returns an error if the manager is not initialized or if the alert cannot be updated
    pub async fn update_alert(&self, alert: Alert) -> Result<()> {
        match &self.inner {
            Some(manager) => manager.update_alert(alert).await,
            None => Err(SquirrelError::alert("AlertManager not initialized")),
        }
    }

    /// Get all alerts from the manager
    ///
    /// # Errors
    /// Returns an error if the manager is not initialized
    pub async fn get_alerts(&self) -> Result<Vec<Alert>> {
        match &self.inner {
            Some(manager) => manager.get_alerts().await,
            None => Err(SquirrelError::alert("AlertManager not initialized")),
        }
    }
}

#[async_trait]
impl<T: NotificationManagerTrait + 'static> AlertManager for AlertManagerAdapter<T> {
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
        match &self.inner {
            Some(manager) => manager.start().await,
            None => Err(SquirrelError::alert("AlertManager not initialized")),
        }
    }

    async fn stop(&self) -> Result<()> {
        match &self.inner {
            Some(manager) => manager.stop().await,
            None => Err(SquirrelError::alert("AlertManager not initialized")),
        }
    }
}

impl<T: NotificationManagerTrait + 'static> Clone for AlertManagerAdapter<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T: NotificationManagerTrait + 'static> Default for AlertManagerAdapter<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Creates a new AlertManagerAdapter with default settings.
#[must_use]
pub fn create_manager_adapter<T: NotificationManagerTrait + 'static>() -> Arc<AlertManagerAdapter<T>> {
    Arc::new(AlertManagerAdapter::new())
}

/// Creates a new AlertManagerAdapter with an existing DefaultAlertManager.
#[must_use]
pub fn create_manager_adapter_with_manager<T: NotificationManagerTrait + 'static>(
    manager: Arc<DefaultAlertManager<T>>
) -> Arc<AlertManagerAdapter<T>> {
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
    pub async fn send_notification(&self, alert: Alert) -> Result<()> {
        // Convert Alert to AlertNotification using the From implementation
        let notification: AlertNotification = alert.into();
        
        self.inner.send_notification(&notification).await.map_err(|e| SquirrelError::Alert(format!("Failed to send notification: {}", e)))
    }

    /// Updates the notification configuration
    ///
    /// # Errors
    /// Returns an error if the configuration is invalid
    pub async fn update_config(&self, config: NotificationConfig) -> Result<()> {
        self.inner.update_config(config).await.map_err(|e| SquirrelError::Alert(format!("Failed to update notification config: {}", e)))
    }
}

/// Creates a notification manager adapter with a new manager
///
/// # Errors
/// Returns an error if the notification manager cannot be created
pub fn create_notification_manager_adapter(
    config: NotificationConfig
) -> Result<Arc<NotificationManagerAdapter>> {
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