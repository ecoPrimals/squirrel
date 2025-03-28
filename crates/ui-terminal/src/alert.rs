use chrono::{DateTime, Utc};
use std::collections::HashMap;
use uuid::Uuid;
use std::sync::Mutex;
use serde::{Serialize, Deserialize};
use dashboard_core::data::{Alert as DashboardAlert, AlertSeverity as DashboardAlertSeverity};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Alert severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertSeverity {
    /// Critical alert - requires immediate attention
    Critical,
    /// Warning alert - important but not critical
    Warning,
    /// Info alert - informational only
    Info,
    /// Error alert - indicates an error condition
    Error,
}

impl AlertSeverity {
    /// Convert severity to string
    pub fn as_str(&self) -> &'static str {
        match self {
            AlertSeverity::Critical => "critical",
            AlertSeverity::Warning => "warning",
            AlertSeverity::Info => "info",
            AlertSeverity::Error => "error",
        }
    }
    
    /// Convert string to severity
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "critical" => Some(AlertSeverity::Critical),
            "warning" => Some(AlertSeverity::Warning),
            "info" => Some(AlertSeverity::Info),
            "error" => Some(AlertSeverity::Error),
            _ => None,
        }
    }
    
    /// Convert from dashboard AlertSeverity
    pub fn from_dashboard_severity(severity: DashboardAlertSeverity) -> Self {
        match severity {
            DashboardAlertSeverity::Critical => AlertSeverity::Critical,
            DashboardAlertSeverity::Warning => AlertSeverity::Warning,
            DashboardAlertSeverity::Info => AlertSeverity::Info,
            DashboardAlertSeverity::Error => AlertSeverity::Error,
        }
    }
    
    /// Convert to dashboard AlertSeverity
    pub fn to_dashboard_severity(&self) -> DashboardAlertSeverity {
        match self {
            AlertSeverity::Critical => DashboardAlertSeverity::Critical,
            AlertSeverity::Warning => DashboardAlertSeverity::Warning,
            AlertSeverity::Info => DashboardAlertSeverity::Info,
            AlertSeverity::Error => DashboardAlertSeverity::Error,
        }
    }
}

/// Alert data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    /// Unique alert ID
    pub id: String,
    /// Alert severity
    pub severity: AlertSeverity,
    /// Alert message
    pub message: String,
    /// Alert details
    pub details: Option<String>,
    /// When the alert was generated
    pub timestamp: DateTime<Utc>,
    /// Whether the alert has been acknowledged
    pub acknowledged: bool,
    /// Who acknowledged the alert
    pub acknowledged_by: Option<String>,
    /// When the alert was acknowledged
    pub acknowledged_at: Option<DateTime<Utc>>,
    /// The source of the alert
    pub source: String,
    /// The category of the alert
    pub category: String,
}

impl Alert {
    /// Create a new alert
    pub fn new(severity: AlertSeverity, message: String, source: String, category: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            severity,
            message,
            details: None,
            timestamp: Utc::now(),
            acknowledged: false,
            acknowledged_by: None,
            acknowledged_at: None,
            source,
            category,
        }
    }
    
    /// Add details to the alert
    pub fn with_details(mut self, details: String) -> Self {
        self.details = Some(details);
        self
    }
    
    /// Convert to dashboard alert format
    pub fn to_dashboard_alert(&self) -> DashboardAlert {
        DashboardAlert {
            id: self.id.clone(),
            title: self.category.clone(),
            message: self.message.clone(),
            severity: self.severity.to_dashboard_severity(),
            source: self.source.clone(),
            timestamp: self.timestamp,
            acknowledged: self.acknowledged,
            acknowledged_by: self.acknowledged_by.clone(),
            acknowledged_at: self.acknowledged_at,
        }
    }
    
    /// Create from dashboard alert
    pub fn from_dashboard_alert(alert: &DashboardAlert) -> Self {
        Self {
            id: alert.id.clone(),
            severity: AlertSeverity::from_dashboard_severity(alert.severity),
            message: alert.message.clone(),
            details: None,
            timestamp: alert.timestamp,
            acknowledged: alert.acknowledged,
            acknowledged_by: alert.acknowledged_by.clone(),
            acknowledged_at: alert.acknowledged_at,
            source: alert.source.clone(),
            category: alert.title.clone(),
        }
    }
}

/// AlertManager handles the creation and management of alerts
/// It tracks active and recently cleared alerts
#[derive(Debug)]
pub struct AlertManager {
    /// Active alerts
    active_alerts: Mutex<Vec<Alert>>,
    /// Recently cleared alerts
    recent_alerts: Mutex<Vec<Alert>>,
    /// Alert counts by category
    alert_counts: Mutex<HashMap<String, usize>>,
    /// Maximum number of recent alerts to keep
    max_recent_alerts: usize,
    /// Current alert ID counter
    alert_id_counter: AtomicUsize,
}

impl Clone for AlertManager {
    fn clone(&self) -> Self {
        // Create a new instance with the same settings
        let mut new_instance = AlertManager::new();
        
        // Update the max recent alerts setting
        new_instance.max_recent_alerts = self.max_recent_alerts;
        
        // Clone the current alert ID counter
        let counter = self.alert_id_counter.load(Ordering::SeqCst);
        new_instance.alert_id_counter.store(counter, Ordering::SeqCst);
        
        // Do not clone the active_alerts, recent_alerts, and alert_counts
        // as they should be managed independently in the new instance
        
        new_instance
    }
}

impl AlertManager {
    /// Create a new alert manager
    pub fn new() -> Self {
        Self {
            active_alerts: Mutex::new(Vec::new()),
            recent_alerts: Mutex::new(Vec::new()),
            alert_counts: Mutex::new(HashMap::new()),
            max_recent_alerts: 100,
            alert_id_counter: AtomicUsize::new(0),
        }
    }
    
    /// Create a new alert manager with maximum recent alerts
    pub fn with_max_recent(max_recent: usize) -> Self {
        Self {
            active_alerts: Mutex::new(Vec::new()),
            recent_alerts: Mutex::new(Vec::new()),
            alert_counts: Mutex::new(HashMap::new()),
            max_recent_alerts: max_recent,
            alert_id_counter: AtomicUsize::new(0),
        }
    }
    
    /// Create and add a new alert
    pub fn add_alert(&self, severity: AlertSeverity, message: String, source: String, category: String) -> Alert {
        let alert = Alert::new(severity, message, source, category.clone());
        self.track_alert(&alert);
        alert
    }
    
    /// Add an existing alert
    pub fn track_alert(&self, alert: &Alert) {
        let mut active = self.active_alerts.lock().unwrap();
        active.push(alert.clone());
        
        // Update alert counts
        let mut counts = self.alert_counts.lock().unwrap();
        *counts.entry(alert.category.clone()).or_insert(0) += 1;
    }
    
    /// Acknowledge an alert
    pub fn acknowledge_alert(&self, alert_id: &str, acknowledged_by: &str) -> bool {
        let mut active = self.active_alerts.lock().unwrap();
        let mut recent = self.recent_alerts.lock().unwrap();
        
        // Find the alert in active alerts
        if let Some(index) = active.iter().position(|a| a.id == alert_id) {
            let mut alert = active.remove(index);
            
            // Update acknowledgment info
            alert.acknowledged = true;
            alert.acknowledged_by = Some(acknowledged_by.to_string());
            alert.acknowledged_at = Some(Utc::now());
            
            // Move to recent alerts
            recent.push(alert);
            
            // Trim recent alerts if necessary
            if recent.len() > self.max_recent_alerts {
                recent.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
                recent.truncate(self.max_recent_alerts);
            }
            
            true
        } else {
            false
        }
    }
    
    /// Get active alerts
    pub fn get_active_alerts(&self) -> Vec<Alert> {
        self.active_alerts.lock().unwrap().clone()
    }
    
    /// Get recent alerts
    pub fn get_recent_alerts(&self) -> Vec<Alert> {
        self.recent_alerts.lock().unwrap().clone()
    }
    
    /// Get alert counts by category
    pub fn get_alert_counts(&self) -> HashMap<String, usize> {
        self.alert_counts.lock().unwrap().clone()
    }
    
    /// Clear all active alerts
    pub fn clear_active_alerts(&self) {
        let mut active = self.active_alerts.lock().unwrap();
        active.clear();
    }
    
    /// Clear all recent alerts
    pub fn clear_recent_alerts(&self) {
        let mut recent = self.recent_alerts.lock().unwrap();
        recent.clear();
    }
    
    /// Reset alert counts
    pub fn reset_alert_counts(&self) {
        let mut counts = self.alert_counts.lock().unwrap();
        counts.clear();
    }
    
    /// Get alerts snapshot for dashboard integration
    pub fn get_alerts_snapshot(&self) -> AlertsSnapshot {
        AlertsSnapshot {
            active: self.get_active_alerts(),
            recent: self.get_recent_alerts(),
            counts: self.get_alert_counts(),
        }
    }
    
    /// Convert alerts to dashboard format
    pub fn to_dashboard_alerts(&self) -> Vec<DashboardAlert> {
        let active = self.get_active_alerts();
        active.iter().map(|a| a.to_dashboard_alert()).collect()
    }
}

/// Alerts snapshot for dashboard integration
#[derive(Debug, Clone)]
pub struct AlertsSnapshot {
    /// Active alerts
    pub active: Vec<Alert>,
    /// Recent alerts
    pub recent: Vec<Alert>,
    /// Alert counts by category
    pub counts: HashMap<String, usize>,
} 