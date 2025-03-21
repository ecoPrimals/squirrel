// Alert manager module

use std::sync::Arc;
use squirrel_core::error::{Result, SquirrelError};
use super::config::AlertConfig;
use std::sync::RwLock;
use super::status::Alert;
use std::fmt::Debug;
use super::status::{AlertSeverity, AlertType};
use uuid::Uuid;
use std::collections::{HashMap, VecDeque};
use tokio::sync::mpsc;
use super::{NotificationManagerTrait, AlertNotification};
use thiserror::Error;

/// Errors that can occur during alert management
#[derive(Debug, Error)]
pub enum AlertError {
    /// Alert not found
    #[error("Alert not found: {0}")]
    NotFound(Uuid),
    /// Notification error
    #[error("Notification error: {0}")]
    NotificationError(String),
    /// Storage error
    #[error("Alert storage error: {0}")]
    StorageError(String),
    /// Configuration error
    #[error("Alert configuration error: {0}")]
    ConfigError(String),
}

// Implementation to convert AlertError to SquirrelError
impl From<AlertError> for SquirrelError {
    fn from(err: AlertError) -> Self {
        match err {
            AlertError::NotFound(id) => SquirrelError::alert(format!("Alert not found: {}", id)),
            AlertError::NotificationError(msg) => SquirrelError::alert(format!("Notification error: {}", msg)),
            AlertError::StorageError(msg) => SquirrelError::alert(format!("Alert storage error: {}", msg)),
            AlertError::ConfigError(msg) => SquirrelError::alert(format!("Alert configuration error: {}", msg)),
        }
    }
}

/// Alert manager for handling system alerts
#[derive(Debug)]
pub struct AlertManager<N = ()> 
where 
    N: NotificationManagerTrait + 'static
{
    /// Alert manager configuration
    config: AlertConfig,
    /// Active alerts
    alerts: Arc<RwLock<HashMap<Uuid, Alert>>>,
    /// Alert history
    history: Arc<RwLock<VecDeque<Alert>>>,
    /// Alert notification routers
    notification_manager: Option<Arc<N>>,
    /// Channel for sending alerts
    alert_tx: Option<mpsc::Sender<Alert>>,
    /// Alert metrics
    metrics: Arc<RwLock<AlertMetrics>>,
}

/// Metrics for the alert manager
#[derive(Debug, Default, Clone)]
pub struct AlertMetrics {
    /// Total number of alerts generated
    pub total_alerts: u64,
    /// Alerts by severity
    pub alerts_by_severity: HashMap<AlertSeverity, u64>,
    /// Alerts by source
    pub alerts_by_source: HashMap<String, u64>,
    /// Active alerts count
    pub active_alerts: u64,
}

impl<N> AlertManager<N> 
where 
    N: NotificationManagerTrait + 'static
{
    /// Creates a new alert manager with the specified configuration
    #[must_use]
    pub fn new(config: AlertConfig) -> Self {
        Self {
            config,
            alerts: Arc::new(RwLock::new(HashMap::new())),
            history: Arc::new(RwLock::new(VecDeque::new())),
            notification_manager: None,
            alert_tx: None,
            metrics: Arc::new(RwLock::new(AlertMetrics::default())),
        }
    }
    
    /// Initialize the alert manager
    pub fn initialize(&mut self) -> Result<()> {
        // Set up the alert channel for async processing
        let (tx, mut rx) = mpsc::channel(100);
        self.alert_tx = Some(tx);
        
        // Clone what we need for the alert processing task
        let alerts = Arc::clone(&self.alerts);
        let history = Arc::clone(&self.history);
        let metrics = Arc::clone(&self.metrics);
        let config = self.config.clone();
        let notification_manager = self.notification_manager.clone();
        
        // Spawn a task to process alerts asynchronously
        tokio::spawn(async move {
            while let Some(alert) = rx.recv().await {
                // Store the alert
                {
                    let mut alerts_lock = alerts.write().unwrap();
                    alerts_lock.insert(alert.id, alert.clone());
                }
                
                // Update history
                {
                    let mut history_lock = history.write().unwrap();
                    history_lock.push_back(alert.clone());
                    while history_lock.len() > config.history_limit {
                        history_lock.pop_front();
                    }
                }
                
                // Update metrics
                {
                    let mut metrics_lock = metrics.write().unwrap();
                    metrics_lock.total_alerts += 1;
                    *metrics_lock.alerts_by_severity.entry(alert.severity).or_insert(0) += 1;
                    *metrics_lock.alerts_by_source.entry(alert.source.clone()).or_insert(0) += 1;
                    metrics_lock.active_alerts += 1;
                }
                
                // Send notifications if configured
                if let Some(notification_manager) = &notification_manager {
                    if config.should_notify(alert.severity) {
                        // Convert Alert to AlertNotification
                        let notification = AlertNotification {
                            id: alert.id.to_string(),
                            name: format!("Alert: {}", alert.source),
                            description: alert.message.clone(),
                            severity: alert.severity.into(), // This assumes we have an implementation of Into<LegacyAlertSeverity> for AlertSeverity
                            status: super::AlertStatus::Active,
                            labels: alert.details.iter()
                                .filter_map(|(k, v)| {
                                    if let Some(s) = v.as_str() {
                                        Some((k.clone(), s.to_string()))
                                    } else {
                                        None
                                    }
                                })
                                .collect(),
                            created_at: alert.timestamp.timestamp(),
                            updated_at: alert.timestamp.timestamp(),
                            message: alert.message.clone(),
                            component: alert.source.clone(),
                        };
                        
                        if let Err(e) = notification_manager.send_notification(&notification).await {
                            tracing::error!("Failed to send notification: {}", e);
                        }
                    }
                }
            }
        });
        
        Ok(())
    }
    
    /// Set notification manager for the alert manager
    pub fn set_notification_manager(&mut self, notification_manager: Arc<N>) {
        self.notification_manager = Some(notification_manager);
    }
    
    /// Create a new alert
    pub async fn create_alert(
        &self,
        alert_type: AlertType,
        severity: AlertSeverity,
        source: String,
        message: String,
        details: Option<HashMap<String, serde_json::Value>>,
    ) -> Result<Alert> {
        // Create alert
        let mut alert = Alert::new(alert_type, severity, source, message);
        
        // Add details if provided
        if let Some(details_map) = details {
            alert = alert.with_details(details_map);
        }
        
        // Send to processing channel
        if let Some(tx) = &self.alert_tx {
            if let Err(e) = tx.send(alert.clone()).await {
                return Err(AlertError::StorageError(format!("Failed to send alert: {}", e)).into());
            }
        } else {
            return Err(AlertError::ConfigError("Alert manager not initialized".to_string()).into());
        }
        
        Ok(alert)
    }
    
    /// Acknowledge an alert
    pub async fn acknowledge_alert(&self, alert_id: Uuid, by: String) -> Result<()> {
        // Get the alert
        let mut alerts = self.alerts.write().unwrap();
        let alert = alerts.get_mut(&alert_id)
            .ok_or_else(|| AlertError::NotFound(alert_id))?;
        
        // Acknowledge it
        alert.acknowledge(by);
        
        // Update metrics
        let mut metrics = self.metrics.write().unwrap();
        metrics.active_alerts = metrics.active_alerts.saturating_sub(1);
        
        Ok(())
    }
    
    /// Get all active alerts
    pub async fn get_active_alerts(&self) -> Result<Vec<Alert>> {
        let alerts = self.alerts.read().unwrap();
        let active = alerts.values()
            .filter(|alert| !alert.acknowledged)
            .cloned()
            .collect();
        
        Ok(active)
    }
    
    /// Get alert history
    pub async fn get_alert_history(&self) -> Result<Vec<Alert>> {
        let history = self.history.read().unwrap();
        Ok(history.iter().cloned().collect())
    }
    
    /// Get a specific alert by ID
    pub async fn get_alert(&self, alert_id: Uuid) -> Result<Option<Alert>> {
        let alerts = self.alerts.read().unwrap();
        Ok(alerts.get(&alert_id).cloned())
    }
    
    /// Get metrics about alerts
    pub async fn get_metrics(&self) -> Result<AlertMetrics> {
        let metrics_guard = self.metrics.read().unwrap();
        // Clone the metrics from the guard
        let metrics_clone = metrics_guard.clone();
        Ok(metrics_clone)
    }
}

/// Adapter for the new AlertManager to provide backward compatibility
/// during transition
#[derive(Debug, Clone)]
pub struct AlertManagerAdapter {
    /// Reference to the manager
    manager: Arc<AlertManager<Box<dyn NotificationManagerTrait>>>,
}

impl AlertManagerAdapter {
    /// Create a new adapter with the specified manager
    #[must_use]
    pub fn with_manager(manager: Arc<AlertManager<Box<dyn NotificationManagerTrait>>>) -> Self {
        Self { manager }
    }
    
    /// Create alert using the manager
    pub async fn create_alert(
        &self,
        alert_type: AlertType,
        severity: AlertSeverity,
        source: String,
        message: String,
        details: Option<HashMap<String, serde_json::Value>>,
    ) -> Result<Alert> {
        self.manager.create_alert(alert_type, severity, source, message, details).await
    }
    
    /// Acknowledge an alert
    pub async fn acknowledge_alert(&self, alert_id: Uuid, by: String) -> Result<()> {
        self.manager.acknowledge_alert(alert_id, by).await
    }
    
    /// Get active alerts
    pub async fn get_active_alerts(&self) -> Result<Vec<Alert>> {
        self.manager.get_active_alerts().await
    }
    
    /// Get alert history
    pub async fn get_alert_history(&self) -> Result<Vec<Alert>> {
        self.manager.get_alert_history().await
    }
}

/// Creates a new alert manager adapter
pub fn create_manager_adapter() -> Result<Arc<AlertManagerAdapter>> {
    let manager = AlertManager::<Box<dyn NotificationManagerTrait>>::new(AlertConfig::default());
    let manager_arc = Arc::new(manager);
    let adapter = AlertManagerAdapter::with_manager(manager_arc);
    Ok(Arc::new(adapter))
}

/// Creates a new alert manager adapter with custom configuration
pub fn create_manager_adapter_with_config(config: AlertConfig) -> Result<Arc<AlertManagerAdapter>> {
    let manager = AlertManager::<Box<dyn NotificationManagerTrait>>::new(config);
    let manager_arc = Arc::new(manager);
    let adapter = AlertManagerAdapter::with_manager(manager_arc);
    Ok(Arc::new(adapter))
}

/// Creates a new alert manager adapter with an existing manager
pub fn create_manager_adapter_with_manager(
    manager: Arc<AlertManager<Box<dyn NotificationManagerTrait>>>
) -> Arc<AlertManagerAdapter> {
    let adapter = AlertManagerAdapter::with_manager(manager);
    Arc::new(adapter)
} 