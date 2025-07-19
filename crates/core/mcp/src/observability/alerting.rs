//! # Alerting System
//! 
//! This module provides an alerting system for the MCP, enabling notifications
//! about critical system events and issues.
//!
//! ## Key Components
//!
//! - **Alert**: Represents a notification about a system event
//! - **AlertSeverity**: Represents the severity level of an alert
//! - **AlertRule**: Defines conditions that trigger alerts
//! - **AlertManager**: Manages alert rules and notifications

use std::collections::HashMap;
use std::sync::RwLock;
use std::time::Duration;
use tokio::sync::broadcast;
use uuid::Uuid;
use crate::observability::{ObservabilityError, ObservabilityResult};
use crate::observability::health::HealthStatus;

/// Severity level of an alert
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub enum AlertSeverity {
    /// Informational alert
    Info = 0,
    /// Warning alert
    Warning = 1,
    /// Error alert
    Error = 2,
    /// Critical alert
    Critical = 3,
}

impl std::fmt::Display for AlertSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Info => write!(f, "INFO"),
            Self::Warning => write!(f, "WARNING"),
            Self::Error => write!(f, "ERROR"),
            Self::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// Type of alert
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AlertType {
    /// Health status change
    HealthStatus,
    /// System resource usage
    ResourceUsage,
    /// Security event
    Security,
    /// Performance issue
    Performance,
    /// Configuration issue
    Configuration,
    /// Custom alert type
    Custom,
    /// System alert type
    System,
}

impl std::fmt::Display for AlertType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::HealthStatus => write!(f, "HEALTH_STATUS"),
            Self::ResourceUsage => write!(f, "RESOURCE_USAGE"),
            Self::Security => write!(f, "SECURITY"),
            Self::Performance => write!(f, "PERFORMANCE"),
            Self::Configuration => write!(f, "CONFIGURATION"),
            Self::Custom => write!(f, "CUSTOM"),
            Self::System => write!(f, "SYSTEM"),
        }
    }
}

/// Current state of an alert
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AlertState {
    /// Alert is active
    Active,
    /// Alert is acknowledged by a user
    Acknowledged,
    /// Alert is resolved
    Resolved,
}

impl std::fmt::Display for AlertState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => write!(f, "ACTIVE"),
            Self::Acknowledged => write!(f, "ACKNOWLEDGED"),
            Self::Resolved => write!(f, "RESOLVED"),
        }
    }
}

/// Alert notification
#[derive(Debug, Clone, serde::Serialize)]
pub struct Alert {
    /// Unique identifier for the alert
    id: String,
    /// Source component that generated the alert
    source: String,
    /// Short summary of the alert
    summary: String,
    /// Detailed description of the alert
    description: String,
    /// Severity level of the alert
    severity: AlertSeverity,
    /// Type of alert
    alert_type: AlertType,
    /// Current state of the alert
    state: AlertState,
    /// When the alert was created
    created_at: std::time::SystemTime,
    /// When the alert was last updated
    updated_at: std::time::SystemTime,
    /// Additional context or metadata
    labels: HashMap<String, String>,
    /// Additional data about the alert
    annotations: HashMap<String, String>,
}

impl Alert {
    /// Create a new alert
    pub fn new(
        source: impl Into<String>,
        summary: impl Into<String>,
        description: impl Into<String>,
        severity: AlertSeverity,
        alert_type: AlertType,
    ) -> Self {
        let now = std::time::SystemTime::now();
        Self {
            id: Uuid::new_v4().to_string(),
            source: source.into(),
            summary: summary.into(),
            description: description.into(),
            severity,
            alert_type,
            state: AlertState::Active,
            created_at: now,
            updated_at: now,
            labels: HashMap::new(),
            annotations: HashMap::new(),
        }
    }

    /// Get the alert ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get the source component
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Get the summary
    pub fn summary(&self) -> &str {
        &self.summary
    }

    /// Get the description
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Get the severity
    pub fn severity(&self) -> AlertSeverity {
        self.severity
    }

    /// Get the alert type
    pub fn alert_type(&self) -> AlertType {
        self.alert_type
    }

    /// Get the state
    pub fn state(&self) -> AlertState {
        self.state
    }

    /// Get the creation time
    pub fn created_at(&self) -> std::time::SystemTime {
        self.created_at
    }

    /// Get the last update time
    pub fn updated_at(&self) -> std::time::SystemTime {
        self.updated_at
    }

    /// Get all labels
    pub fn labels(&self) -> &HashMap<String, String> {
        &self.labels
    }

    /// Get all annotations
    pub fn annotations(&self) -> &HashMap<String, String> {
        &self.annotations
    }

    /// Add a label to the alert
    pub fn with_label(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.labels.insert(key.into(), value.into());
        self
    }

    /// Add an annotation to the alert
    pub fn with_annotation(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.annotations.insert(key.into(), value.into());
        self
    }

    /// Acknowledge the alert
    pub fn acknowledge(&mut self) {
        self.state = AlertState::Acknowledged;
        self.updated_at = std::time::SystemTime::now();
    }

    /// Resolve the alert
    pub fn resolve(&mut self) {
        self.state = AlertState::Resolved;
        self.updated_at = std::time::SystemTime::now();
    }

    /// Create a new health status alert
    pub fn health_status(
        component: impl Into<String>,
        from: HealthStatus,
        to: HealthStatus,
        details: impl Into<String>,
    ) -> Self {
        let component_str = component.into();
        let summary = format!("Health status change: {} -> {}", from, to);
        let severity = match to {
            HealthStatus::Unhealthy => AlertSeverity::Critical,
            HealthStatus::Degraded => AlertSeverity::Warning,
            HealthStatus::Healthy => AlertSeverity::Info,
            HealthStatus::Unknown => AlertSeverity::Warning,
        };

        Self::new(
            component_str.clone(),
            summary,
            details,
            severity,
            AlertType::HealthStatus,
        )
        .with_label("component", component_str)
        .with_label("from_status", from.to_string())
        .with_label("to_status", to.to_string())
    }

    /// Create a new resource usage alert
    pub fn resource_usage(
        component: impl Into<String>,
        resource: impl Into<String>,
        usage: f64,
        threshold: f64,
        details: impl Into<String>,
    ) -> Self {
        let component_str = component.into();
        let resource_str = resource.into();
        let summary = format!("Resource usage alert: {} - {} ({}% of threshold)", 
            component_str, resource_str, (usage / threshold * 100.0) as u32);
            
        let severity = if usage >= threshold * 0.95 {
            AlertSeverity::Critical
        } else if usage >= threshold * 0.8 {
            AlertSeverity::Error
        } else if usage >= threshold * 0.6 {
            AlertSeverity::Warning
        } else {
            AlertSeverity::Info
        };

        Self::new(
            component_str.clone(),
            summary,
            details,
            severity,
            AlertType::ResourceUsage,
        )
        .with_label("component", component_str)
        .with_label("resource", resource_str)
        .with_annotation("usage", usage.to_string())
        .with_annotation("threshold", threshold.to_string())
    }
}

/// Configuration for the alert manager
#[derive(Debug, Clone)]
pub struct AlertManagerConfig {
    /// Default time to keep resolved alerts
    pub retention_time: Duration,
    /// Maximum number of alerts to keep in memory
    pub max_alerts: usize,
    /// Number of alerts in the notification buffer
    pub notification_buffer: usize,
    /// Maximum number of concurrent alerts
    pub max_concurrent_alerts: usize,
    /// Maximum number of alerts per component
    pub max_alerts_per_component: usize,
    /// Timeout for alert processing
    pub alert_timeout: Duration,
    /// Number of retry attempts for failed alerts
    pub retry_attempts: usize,
}

impl Default for AlertManagerConfig {
    fn default() -> Self {
        Self {
            retention_time: Duration::from_secs(86400), // 24 hours
            max_alerts: 10000,
            notification_buffer: 1000,
            max_concurrent_alerts: 100,
            max_alerts_per_component: 50,
            alert_timeout: Duration::from_secs(60),
            retry_attempts: 3,
        }
    }
}

/// The alert manager creates and dispatches alerts
#[derive(Debug)]
pub struct AlertManager {
    /// Alert manager configuration
    config: RwLock<AlertManagerConfig>,
    /// All active and recent alerts
    alerts: RwLock<HashMap<String, Alert>>,
    /// Channel for alert notifications
    alert_tx: broadcast::Sender<Alert>,
    /// Subscribers to alert notifications
    subscribers: RwLock<Vec<broadcast::Receiver<Alert>>>,
}

impl AlertManager {
    /// Create a new alert manager
    pub fn new() -> Self {
        let config = AlertManagerConfig::default();
        let (alert_tx, _) = broadcast::channel(config.notification_buffer);
        
        Self {
            config: RwLock::new(config),
            alerts: RwLock::new(HashMap::new()),
            alert_tx,
            subscribers: RwLock::new(Vec::new()),
        }
    }

    /// Initialize the alert manager
    pub fn initialize(&self) -> ObservabilityResult<()> {
        // Any initialization tasks would go here
        Ok(())
    }

    /// Set the alert manager configuration
    pub fn set_config(&self, config: AlertManagerConfig) -> ObservabilityResult<()> {
        let mut current_config = self.config.write().map_err(|e| 
            ObservabilityError::AlertingError(format!("Failed to acquire config write lock: {}", e)))?;
        *current_config = config;
        Ok(())
    }

    /// Create and publish a new alert
    pub fn alert(
        &self,
        source: impl Into<String>,
        summary: impl Into<String>,
        description: impl Into<String>,
        severity: AlertSeverity,
        alert_type: AlertType,
    ) -> ObservabilityResult<Alert> {
        let alert = Alert::new(source, summary, description, severity, alert_type);
        self.publish_alert(alert)
    }

    /// Add an alert to the system
    pub fn publish_alert(&self, alert: Alert) -> ObservabilityResult<Alert> {
        let alert_id = alert.id().to_string();
        
        // Store the alert
        {
            let mut alerts = self.alerts.write().map_err(|e| 
                ObservabilityError::AlertingError(format!("Failed to acquire alerts write lock: {}", e)))?;
            
            // Check if we need to evict old alerts
            let config = self.config.read().map_err(|e| 
                ObservabilityError::AlertingError(format!("Failed to acquire config read lock: {}", e)))?;
                
            if alerts.len() >= config.max_alerts {
                // Remove the oldest resolved alert
                let now = std::time::SystemTime::now();
                let mut oldest_id = None;
                let mut oldest_time = now;
                
                for (id, a) in alerts.iter() {
                    if a.state() == AlertState::Resolved && a.updated_at() < oldest_time {
                        oldest_id = Some(id.clone());
                        oldest_time = a.updated_at();
                    }
                }
                
                if let Some(id) = oldest_id {
                    alerts.remove(&id);
                } else {
                    // If no resolved alerts, remove the oldest alert
                    let mut oldest_id = None;
                    let mut oldest_time = now;
                    
                    for (id, a) in alerts.iter() {
                        if a.created_at() < oldest_time {
                            oldest_id = Some(id.clone());
                            oldest_time = a.created_at();
                        }
                    }
                    
                    if let Some(id) = oldest_id {
                        alerts.remove(&id);
                    }
                }
            }
            
            alerts.insert(alert_id.clone(), alert.clone());
        }
        
        // Broadcast the alert
        let _ = self.alert_tx.send(alert.clone());
        
        Ok(alert)
    }

    /// Get an alert by ID
    pub fn get_alert(&self, id: &str) -> ObservabilityResult<Option<Alert>> {
        let alerts = self.alerts.read().map_err(|e| 
            ObservabilityError::AlertingError(format!("Failed to acquire alerts read lock: {}", e)))?;
        
        Ok(alerts.get(id).cloned())
    }

    /// Get all alerts with optional filters
    pub fn get_alerts(
        &self,
        source: Option<&str>,
        severity: Option<AlertSeverity>,
        alert_type: Option<AlertType>,
        state: Option<AlertState>,
    ) -> ObservabilityResult<Vec<Alert>> {
        let alerts = self.alerts.read().map_err(|e| 
            ObservabilityError::AlertingError(format!("Failed to acquire alerts read lock: {}", e)))?;
        
        let mut result = Vec::new();
        for alert in alerts.values() {
            if let Some(s) = source {
                if alert.source() != s {
                    continue;
                }
            }
            
            if let Some(sev) = severity {
                if alert.severity() != sev {
                    continue;
                }
            }
            
            if let Some(t) = alert_type {
                if alert.alert_type() != t {
                    continue;
                }
            }
            
            if let Some(st) = state {
                if alert.state() != st {
                    continue;
                }
            }
            
            result.push(alert.clone());
        }
        
        // Sort by severity (highest first) and then by creation time (newest first)
        result.sort_by(|a, b| {
            let sev_cmp = b.severity().cmp(&a.severity());
            if sev_cmp == std::cmp::Ordering::Equal {
                // For equal severity, compare creation time
                b.created_at().cmp(&a.created_at())
            } else {
                sev_cmp
            }
        });
        
        Ok(result)
    }

    /// Subscribe to alert notifications
    pub fn subscribe(&self) -> broadcast::Receiver<Alert> {
        self.alert_tx.subscribe()
    }

    /// Acknowledge an alert
    pub fn acknowledge_alert(&self, id: &str) -> ObservabilityResult<bool> {
        let mut alerts = self.alerts.write().map_err(|e| 
            ObservabilityError::AlertingError(format!("Failed to acquire alerts write lock: {}", e)))?;
        
        if let Some(alert) = alerts.get_mut(id) {
            if alert.state() == AlertState::Active {
                alert.acknowledge();
                let _ = self.alert_tx.send(alert.clone());
                return Ok(true);
            }
        }
        
        Ok(false)
    }

    /// Resolve an alert
    pub fn resolve_alert(&self, id: &str) -> ObservabilityResult<bool> {
        let mut alerts = self.alerts.write().map_err(|e| 
            ObservabilityError::AlertingError(format!("Failed to acquire alerts write lock: {}", e)))?;
        
        if let Some(alert) = alerts.get_mut(id) {
            if alert.state() != AlertState::Resolved {
                alert.resolve();
                let _ = self.alert_tx.send(alert.clone());
                return Ok(true);
            }
        }
        
        Ok(false)
    }

    /// Clean up old resolved alerts
    pub fn cleanup_old_alerts(&self) -> ObservabilityResult<usize> {
        let config = self.config.read().map_err(|e| 
            ObservabilityError::AlertingError(format!("Failed to acquire config read lock: {}", e)))?;
            
        let mut alerts = self.alerts.write().map_err(|e| 
            ObservabilityError::AlertingError(format!("Failed to acquire alerts write lock: {}", e)))?;
            
        let now = std::time::SystemTime::now();
        let mut to_remove = Vec::new();
        
        for (id, alert) in alerts.iter() {
            if alert.state() == AlertState::Resolved {
                if let Ok(elapsed) = now.duration_since(alert.updated_at()) {
                    if elapsed > config.retention_time {
                        to_remove.push(id.clone());
                    }
                }
            }
        }
        
        for id in &to_remove {
            alerts.remove(id);
        }
        
        Ok(to_remove.len())
    }

    pub fn create_alert(
        &self,
        id: &str,
        name: &str,
        severity: AlertSeverity,
        message: Option<&str>,
        component: Option<&str>,
        _metadata: Option<HashMap<String, String>>,
    ) -> ObservabilityResult<Alert> {
        let alert = Alert::new(
            component.unwrap_or("system"),
            name,
            message.unwrap_or("No details provided"),
            severity,
            AlertType::System,
        );

        let mut alerts = self.alerts.write().map_err(|_| 
            ObservabilityError::AlertingError("Failed to acquire alerts lock".to_string()))?;
        alerts.insert(id.to_string(), alert.clone());

        Ok(alert)
    }

    pub fn new_with_config(config: AlertManagerConfig) -> ObservabilityResult<Self> {
        let (alert_tx, _) = broadcast::channel(config.notification_buffer);
        let manager = Self {
            alerts: RwLock::new(HashMap::new()),
            config: RwLock::new(config),
            alert_tx,
            subscribers: RwLock::new(Vec::new()),
        };
        Ok(manager)
    }

    /// Get recent alerts within a certain limit
    pub async fn get_recent_alerts(&self, limit: usize) -> ObservabilityResult<Vec<Alert>> {
        let alerts = self.alerts.read()
            .map_err(|_| ObservabilityError::AlertingError("Failed to read alerts".to_string()))?;
        
        let mut alert_list: Vec<Alert> = alerts.values().cloned().collect();
        alert_list.sort_by(|a, b| b.created_at().cmp(&a.created_at()));
        alert_list.truncate(limit);
        Ok(alert_list)
    }

    /// Get recent alerts (without limit - for dashboard sync)
    pub async fn get_all_recent_alerts(&self) -> ObservabilityResult<Vec<Alert>> {
        let alerts = self.alerts.read()
            .map_err(|_| ObservabilityError::AlertingError("Failed to read alerts".to_string()))?;
        
        let mut alert_list: Vec<Alert> = alerts.values().cloned().collect();
        alert_list.sort_by(|a, b| b.created_at().cmp(&a.created_at()));
        Ok(alert_list)
    }

    /// Trigger an alert with simplified parameters
    pub async fn trigger_alert(
        &self,
        alert_id: &str,
        severity: &str,
        message: &str,
    ) -> ObservabilityResult<()> {
        let severity = match severity.to_lowercase().as_str() {
            "info" => AlertSeverity::Info,
            "warning" => AlertSeverity::Warning,
            "error" => AlertSeverity::Error,
            "critical" => AlertSeverity::Critical,
            _ => AlertSeverity::Warning,
        };

        let alert = Alert::new(
            "system",
            alert_id,
            message,
            severity,
            AlertType::System,
        );

        self.publish_alert(alert)?;
        Ok(())
    }

    /// Get count of active alerts
    pub async fn get_active_alerts_count(&self) -> ObservabilityResult<usize> {
        let alerts = self.alerts.read()
            .map_err(|_| ObservabilityError::AlertingError("Failed to read alerts".to_string()))?;
        
        let active_count = alerts.values()
            .filter(|alert| alert.state() == AlertState::Active)
            .count();
        
        Ok(active_count)
    }
}

impl Default for AlertManager {
    fn default() -> Self {
        Self::new()
    }
} 