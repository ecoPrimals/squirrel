use async_trait::async_trait;
use squirrel_core::error::{Result, SquirrelError};
use std::sync::Arc;
use std::fmt::Debug;
use crate::alerts::status::{AlertSeverity, Alert, AlertStatus};
use crate::alerts::{NotificationManagerTrait, LegacyAlertSeverity};
use crate::alerts::AlertNotification;
use crate::alerts::notify::{NotificationManager, NotificationConfig};

/// Convert from LegacyAlertSeverity to AlertSeverity
fn convert_legacy_severity(legacy: &LegacyAlertSeverity) -> AlertSeverity {
    match legacy {
        LegacyAlertSeverity::Info => AlertSeverity::Info,
        LegacyAlertSeverity::Warning => AlertSeverity::Warning,
        LegacyAlertSeverity::Error => AlertSeverity::Error,
        LegacyAlertSeverity::Critical => AlertSeverity::Critical,
    }
}

/// Adapter for the alert manager to support dependency injection
#[derive(Debug)]
pub struct AlertManagerAdapter<T: NotificationManagerTrait + 'static = ()> {
    /// The inner alert manager instance
    inner: Option<Arc<DefaultAlertManager<T>>>,
}

/// Default alert manager type
pub type DefaultAlertManager<T> = crate::alerts::manager::AlertManager<T>;

/// Legacy alert type for backward compatibility
pub type LegacyAlert = crate::alerts::status::Alert;

/// Legacy alert config type for backward compatibility
pub type LegacyAlertConfig = crate::alerts::config::AlertConfig;

/// Type alias for legacy alert manager interface
pub type LegacyAlertManager = dyn AlertManagerTrait;

/// Add a trait definition (with a different name to avoid conflicts)
#[async_trait]
pub trait AlertManagerTrait: Send + Sync + Debug {
    /// Send an alert
    async fn send_alert(&self, alert: LegacyAlert) -> Result<()>;
    
    /// Get all alerts
    async fn get_alerts(&self) -> Result<Vec<LegacyAlert>>;
    
    /// Add an alert
    async fn add_alert(&self, alert: LegacyAlert) -> Result<()>;
    
    /// Update an alert
    async fn update_alert(&self, alert: LegacyAlert) -> Result<()>;
    
    /// Start alert manager
    async fn start(&self) -> Result<()>;
    
    /// Stop alert manager
    async fn stop(&self) -> Result<()>;
}

/// Legacy alert manager trait
#[async_trait]
pub trait LegacyAlertManagerTrait: Send + Sync + Debug {
    /// Send an alert
    async fn send_alert(&self, alert: LegacyAlert) -> Result<String>;
    
    /// Get all alerts
    async fn get_alerts(&self) -> Result<Vec<LegacyAlert>>;
    
    /// Add an alert
    async fn add_alert(&self, alert: LegacyAlert) -> Result<String>;
    
    /// Update an alert
    async fn update_alert(&self, id: &str, status: AlertStatus) -> Result<()>;
    
    /// Start the alert manager
    async fn start(&self) -> Result<()>;
    
    /// Stop the alert manager
    async fn stop(&self) -> Result<()>;
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
        
        let config = LegacyAlertConfig::default();
        let mut manager = DefaultAlertManager::new(config);
        // Make sure to initialize the inner manager
        manager.initialize()?;
        self.inner = Some(Arc::new(manager));
        Ok(())
    }
    
    /// Initializes the adapter with custom configuration
    pub fn initialize_with_config(&mut self, config: LegacyAlertConfig) -> Result<()> {
        if self.is_initialized() {
            return Err(SquirrelError::alert("AlertManager already initialized"));
        }
        
        let mut manager = DefaultAlertManager::new(config);
        // Make sure to initialize the inner manager
        manager.initialize()?;
        self.inner = Some(Arc::new(manager));
        Ok(())
    }

    /// Send an alert to the manager
    ///
    /// # Errors
    /// Returns an error if the manager is not initialized or if the alert cannot be sent
    pub async fn send_alert(&self, alert: Alert) -> Result<()> {
        match &self.inner {
            Some(manager) => {
                // Convert Alert to a new alert creation format
                let alert_type = alert.alert_type.clone();
                let severity = alert.severity;
                let source = alert.source.clone();
                let message = alert.message.clone();
                let details = Some(alert.details.clone());
                
                manager.create_alert(alert_type, severity, source, message, details).await?;
                Ok(())
            }
            None => Err(SquirrelError::alert("AlertManager not initialized")),
        }
    }

    /// Add an alert to the manager
    ///
    /// # Errors
    /// Returns an error if the manager is not initialized or if the alert cannot be added
    pub async fn add_alert(&self, alert: Alert) -> Result<()> {
        match &self.inner {
            Some(manager) => {
                // Convert Alert to a new alert creation format
                let alert_type = alert.alert_type.clone();
                let severity = alert.severity;
                let source = alert.source.clone();
                let message = alert.message.clone();
                let details = Some(alert.details.clone());
                
                manager.create_alert(alert_type, severity, source, message, details).await?;
                Ok(())
            }
            None => Err(SquirrelError::alert("AlertManager not initialized")),
        }
    }

    /// Update an alert in the manager
    ///
    /// # Errors
    /// Returns an error if the manager is not initialized or if the alert cannot be updated
    pub async fn update_alert(&self, alert: Alert) -> Result<()> {
        match &self.inner {
            Some(manager) => {
                // Currently using get_alert to check existence, since there's no direct update function
                manager.get_alert(alert.id).await?;
                // In a real implementation, we would update the alert here
                Ok(())
            }
            None => Err(SquirrelError::alert("AlertManager not initialized")),
        }
    }

    /// Get all alerts from the manager
    ///
    /// # Errors
    /// Returns an error if the manager is not initialized
    pub async fn get_alerts(&self) -> Result<Vec<Alert>> {
        match &self.inner {
            Some(manager) => {
                // Get active alerts from the manager
                manager.get_active_alerts().await
            }
            None => Err(SquirrelError::alert("AlertManager not initialized")),
        }
    }
}

#[async_trait]
impl<T: NotificationManagerTrait + 'static> AlertManagerTrait for AlertManagerAdapter<T> {
    async fn send_alert(&self, alert: LegacyAlert) -> Result<()> {
        match &self.inner {
            Some(manager) => {
                let alert_type = alert.alert_type.clone();
                
                // The alert.severity is already AlertSeverity, we don't need to convert it
                let severity = alert.severity;
                
                let source = alert.source.clone();
                let message = alert.message.clone();
                let details = Some(alert.details.clone());
                
                manager.create_alert(alert_type, severity, source, message, details).await?;
                Ok(())
            }
            None => Err(SquirrelError::alert("AlertManager not initialized")),
        }
    }
    
    async fn get_alerts(&self) -> Result<Vec<LegacyAlert>> {
        match &self.inner {
            Some(manager) => {
                // Get active alerts from the manager - LegacyAlert is the same as Alert type
                manager.get_active_alerts().await
            }
            None => Err(SquirrelError::alert("AlertManager not initialized")),
        }
    }
    
    async fn add_alert(&self, alert: LegacyAlert) -> Result<()> {
        match &self.inner {
            Some(manager) => {
                let alert_type = alert.alert_type.clone();
                let severity = alert.severity;
                let source = alert.source.clone();
                let message = alert.message.clone();
                let details = Some(alert.details.clone());
                
                manager.create_alert(alert_type, severity, source, message, details).await?;
                Ok(())
            }
            None => Err(SquirrelError::alert("AlertManager not initialized")),
        }
    }
    
    async fn update_alert(&self, alert: LegacyAlert) -> Result<()> {
        match &self.inner {
            Some(manager) => {
                // Currently using get_alert to check existence, since there's no direct update function
                manager.get_alert(alert.id).await?;
                // In a real implementation, we would update the alert here
                Ok(())
            }
            None => Err(SquirrelError::alert("AlertManager not initialized")),
        }
    }
    
    async fn start(&self) -> Result<()> {
        Ok(()) // No-op for now
    }
    
    async fn stop(&self) -> Result<()> {
        Ok(()) // No-op for now
    }
}

/// Convert legacy alert to the new alert format
fn convert_from_legacy_alert(legacy_alert: LegacyAlert) -> LegacyAlert {
    // Just return the legacy alert directly since they're now the same type
    legacy_alert
}

/// Convert from Alert to AlertNotification
impl From<Alert> for AlertNotification {
    fn from(alert: Alert) -> Self {
        // Convert from AlertSeverity to LegacyAlertSeverity
        let severity = match alert.severity {
            super::status::AlertSeverity::Info => super::LegacyAlertSeverity::Info,
            super::status::AlertSeverity::Warning => super::LegacyAlertSeverity::Warning,
            super::status::AlertSeverity::Error => super::LegacyAlertSeverity::Error,
            super::status::AlertSeverity::Critical => super::LegacyAlertSeverity::Critical,
        };
        
        Self {
            id: alert.id.to_string(),
            name: format!("Alert: {}", alert.source),
            description: alert.message.clone(),
            severity,
            status: "active".to_string(),
            labels: alert.details.iter()
                .filter_map(|(k, v)| {
                    v.as_str().map(|s| (k.clone(), s.to_string()))
                })
                .collect(),
            created_at: alert.timestamp.timestamp(),
            updated_at: alert.timestamp.timestamp(),
            message: alert.message.clone(),
            component: alert.source.clone(),
            source: alert.source.clone(),
            timestamp: alert.timestamp.timestamp() as u64,
            details: alert.details.clone(),
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
    config: LegacyAlertConfig
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

/// Send an alert with the specified severity type
pub async fn send_alert(_alert: Alert) -> Result<()> {
    // Since Alert now uses the standard AlertSeverity directly,
    // we don't need to do any conversion, just return the alert
    Ok(())
} 
