//! Alert management for system monitoring
//! 
//! This module provides functionality for:
//! - Alert rule management
//! - Alert generation and notification
//! - Alert history tracking
//! - Alert severity levels
//! - Alert routing and escalation

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

pub mod notify;

/// Alert severity levels
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

impl std::fmt::Display for AlertSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AlertSeverity::Info => write!(f, "Info"),
            AlertSeverity::Warning => write!(f, "Warning"),
            AlertSeverity::Error => write!(f, "Error"),
            AlertSeverity::Critical => write!(f, "Critical"),
        }
    }
}

impl AlertSeverity {
    /// Get the color associated with this severity level
    pub fn color(&self) -> &'static str {
        match self {
            AlertSeverity::Info => "blue",
            AlertSeverity::Warning => "yellow",
            AlertSeverity::Error => "orange",
            AlertSeverity::Critical => "red",
        }
    }
}

/// Alert status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertStatus {
    Active,
    Acknowledged,
    Resolved,
}

impl std::fmt::Display for AlertStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AlertStatus::Active => write!(f, "Active"),
            AlertStatus::Acknowledged => write!(f, "Acknowledged"),
            AlertStatus::Resolved => write!(f, "Resolved"),
        }
    }
}

/// Alert with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: String,
    pub name: String,
    pub description: String,
    pub severity: AlertSeverity,
    pub status: AlertStatus,
    pub labels: HashMap<String, String>,
    pub created_at: i64,
    pub updated_at: i64,
    pub message: String,
    pub component: String,
}

impl Alert {
    pub fn new(
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
    pub retention_days: u32,
    pub notification_enabled: bool,
    pub max_alerts: usize,
}

impl Default for AlertConfig {
    fn default() -> Self {
        Self {
            retention_days: 30,
            notification_enabled: true,
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
            component: alert.component.clone(),
        }
    }
}

/// Alert manager trait
#[async_trait]
pub trait AlertManager: Debug + Send + Sync {
    async fn send_alert(&self, alert: Alert) -> Result<()>;
    async fn get_alerts(&self) -> Result<Vec<Alert>>;
    async fn add_alert(&self, alert: Alert) -> Result<()>;
    async fn update_alert(&self, alert: Alert) -> Result<()>;
    async fn start(&self) -> Result<()>;
    async fn stop(&self) -> Result<()>;
}

/// Default alert manager implementation
#[derive(Debug)]
pub struct DefaultAlertManager {
    alerts: Arc<RwLock<Vec<Alert>>>,
    config: AlertConfig,
}

impl DefaultAlertManager {
    pub fn new(config: AlertConfig) -> Self {
        Self {
            alerts: Arc::new(RwLock::new(Vec::new())),
            config,
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
    async fn send_alert(&self, alert: Alert) -> Result<()> {
        // TODO: Implement notification
        self.add_alert(alert).await
    }

    async fn get_alerts(&self) -> Result<Vec<Alert>> {
        let alerts = self.alerts.read().await.clone();
        Ok(alerts)
    }

    async fn add_alert(&self, alert: Alert) -> Result<()> {
        let mut alerts = self.alerts.write().await;
        alerts.push(alert);
        
        // Enforce max alerts limit
        if alerts.len() > self.config.max_alerts {
            alerts.remove(0);
        }
        
        Ok(())
    }

    async fn update_alert(&self, alert: Alert) -> Result<()> {
        let mut alerts = self.alerts.write().await;
        if let Some(idx) = alerts.iter().position(|a| a.id == alert.id) {
            alerts[idx] = alert;
            Ok(())
        } else {
            Err(SquirrelError::alert(&format!("Alert not found: {alert_id}", alert_id = alert.id)))
        }
    }

    async fn start(&self) -> Result<()> {
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        Ok(())
    }
}

/// Alert errors
#[derive(Debug, Error)]
pub enum AlertError {
    #[error("Alert not found: {0}")]
    NotFound(String),
    #[error("Invalid alert: {0}")]
    InvalidAlert(String),
    #[error("Storage error: {0}")]
    StorageError(String),
    #[error("Notification error: {0}")]
    NotificationError(String),
}

impl From<AlertError> for SquirrelError {
    fn from(err: AlertError) -> Self {
        SquirrelError::alert(&err.to_string())
    }
}

// Static instance for global access
static ALERT_MANAGER: tokio::sync::OnceCell<Arc<DefaultAlertManager>> = tokio::sync::OnceCell::const_new();

/// Initializes the alert manager with the given configuration
///
/// # Parameters
/// * `config` - The alert configuration to use, or the default configuration if None
///
/// # Errors
/// Returns an error if the alert manager is already initialized or if initialization fails
pub async fn initialize(config: Option<AlertConfig>) -> Result<()> {
    let config = config.unwrap_or_default();
    let manager = Arc::new(DefaultAlertManager::new(config));
    
    ALERT_MANAGER
        .set(manager)
        .map_err(|_| SquirrelError::alert("Alert manager already initialized"))?;
    
    Ok(())
}

/// Get the alert manager instance
pub fn get_manager() -> Option<Arc<DefaultAlertManager>> {
    ALERT_MANAGER.get().cloned()
}

/// Check if the alert system is initialized
pub fn is_initialized() -> bool {
    ALERT_MANAGER.get().is_some()
}

/// Shuts down the alert manager
///
/// # Errors
/// Returns an error if the alert manager is not initialized or if shutdown fails
pub async fn shutdown() -> Result<()> {
    if let Some(manager) = ALERT_MANAGER.get() {
        let stop_future = manager.stop();
        stop_future.await?;
    }
    
    Ok(())
} 