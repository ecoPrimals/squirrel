use std::sync::Arc;
use async_trait::async_trait;
use squirrel_core::error::{Result, SquirrelError};
use crate::alerts::{Alert, AlertConfig, DefaultAlertManager};
use crate::alerts::AlertManager;
use crate::alerts::notify::{NotificationManager, NotificationConfig};
use super::NotificationManagerTrait;
use super::AlertNotification;

/// Adapter for the alert manager to support dependency injection
#[derive(Debug)]
pub struct AlertManagerAdapter<T: NotificationManagerTrait + 'static = ()> {
    /// The inner alert manager instance
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
    
    /// Checks if the adapter is initialized
    #[must_use]
    pub fn is_initialized(&self) -> bool {
        self.inner.is_some()
    }
    
    /// Initializes the adapter with default configuration
    pub fn initialize(&mut self) -> Result<()> {
        if self.is_initialized() {
            return Err(SquirrelError::alert("AlertManager already initialized"));
        }
        
        let config = AlertConfig::default();
        let manager = DefaultAlertManager::new(config);
        self.inner = Some(Arc::new(manager));
        Ok(())
    }
    
    /// Initializes the adapter with custom configuration
    pub fn initialize_with_config(&mut self, config: AlertConfig) -> Result<()> {
        if self.is_initialized() {
            return Err(SquirrelError::alert("AlertManager already initialized"));
        }
        
        let manager = DefaultAlertManager::new(config);
        self.inner = Some(Arc::new(manager));
        Ok(())
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
        match &self.inner {
            Some(manager) => manager.send_alert(alert).await,
            None => Err(SquirrelError::alert("AlertManager not initialized")),
        }
    }

    async fn get_alerts(&self) -> Result<Vec<Alert>> {
        match &self.inner {
            Some(manager) => manager.get_alerts().await,
            None => Err(SquirrelError::alert("AlertManager not initialized")),
        }
    }

    async fn add_alert(&self, alert: Alert) -> Result<()> {
        match &self.inner {
            Some(manager) => manager.add_alert(alert).await,
            None => Err(SquirrelError::alert("AlertManager not initialized")),
        }
    }

    async fn update_alert(&self, alert: Alert) -> Result<()> {
        match &self.inner {
            Some(manager) => manager.update_alert(alert).await,
            None => Err(SquirrelError::alert("AlertManager not initialized")),
        }
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

/// Creates a new `AlertManagerAdapter` with default settings.
#[must_use]
pub fn create_manager_adapter<T: NotificationManagerTrait + 'static>() -> Arc<AlertManagerAdapter<T>> {
    Arc::new(AlertManagerAdapter::new())
}

/// Creates a new `AlertManagerAdapter` with an existing `DefaultAlertManager`.
#[must_use]
pub fn create_manager_adapter_with_manager<T: NotificationManagerTrait + 'static>(
    manager: Arc<DefaultAlertManager<T>>
) -> Arc<AlertManagerAdapter<T>> {
    Arc::new(AlertManagerAdapter::with_manager(manager))
}

/// Creates and initializes an alert manager adapter with default configuration
///
/// # Errors
/// Returns an error if initialization fails
pub fn create_initialized_manager_adapter<T: NotificationManagerTrait + 'static>() -> Result<Arc<AlertManagerAdapter<T>>> {
    let mut adapter = AlertManagerAdapter::new();
    adapter.initialize()?;
    Ok(Arc::new(adapter))
}

/// Creates and initializes an alert manager adapter with custom configuration
///
/// # Errors
/// Returns an error if initialization fails
pub fn create_manager_adapter_with_config<T: NotificationManagerTrait + 'static>(
    config: AlertConfig
) -> Result<Arc<AlertManagerAdapter<T>>> {
    let mut adapter = AlertManagerAdapter::new();
    adapter.initialize_with_config(config)?;
    Ok(Arc::new(adapter))
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
        
        self.inner.send_notification(&notification).await.map_err(|e| SquirrelError::alert(format!("Failed to send notification: {e}")))
    }

    /// Updates the notification configuration
    ///
    /// # Errors
    /// Returns an error if the configuration is invalid
    pub async fn update_config(&self, config: NotificationConfig) -> Result<()> {
        self.inner.update_config(config).await.map_err(|e| SquirrelError::alert(format!("Failed to update notification config: {e}")))
    }
}

/// Creates a new notification manager adapter with the given configuration
/// 
/// # Arguments
/// 
/// * `config` - The notification configuration to use
/// 
/// # Returns
/// 
/// Returns a Result containing the adapter if successful
pub async fn create_notification_manager_adapter(
    config: NotificationConfig
) -> Result<Arc<NotificationManagerAdapter>> {
    let manager = match NotificationManager::new(config) {
        Ok(mgr) => mgr,
        Err(e) => return Err(SquirrelError::metric(format!("Failed to create notification manager: {}", e))),
    };
    
    Ok(Arc::new(NotificationManagerAdapter {
        inner: Arc::new(manager),
    }))
}

/// Creates a notification manager adapter with an existing manager
#[must_use]
pub fn create_notification_manager_adapter_with_manager(
    manager: Arc<NotificationManager>
) -> Arc<NotificationManagerAdapter> {
    Arc::new(NotificationManagerAdapter::with_manager(manager))
} 