//! Alert management for the MCP monitoring system
//!
//! This module provides alert management functionality for the MCP system.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use super::metrics::{MetricValue, MetricsCollector};
use crate::error::Result;
use crate::MCPError;

/// Alert severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum AlertSeverity {
    /// Informational alerts
    Info,
    /// Warning alerts
    Warning,
    /// Error alerts
    Error,
    /// Critical alerts
    Critical,
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

/// Alert condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertCondition {
    /// Metric exceeds a threshold
    MetricAbove {
        metric_name: String,
        threshold: MetricValue,
    },
    /// Metric falls below a threshold
    MetricBelow {
        metric_name: String,
        threshold: MetricValue,
    },
    /// Metric equals a value
    MetricEquals {
        metric_name: String,
        value: MetricValue,
    },
    /// Composite condition (AND)
    And(Vec<AlertCondition>),
    /// Composite condition (OR)
    Or(Vec<AlertCondition>),
    /// Metric changes by a percentage
    PercentageChange {
        metric_name: String,
        percentage: f64,
        duration: Duration,
    },
    /// Custom condition (evaluated by a callback)
    Custom(String),
}

/// Alert action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertAction {
    /// Log the alert
    Log,
    /// Send an email
    Email {
        recipients: Vec<String>,
        subject_template: String,
        body_template: String,
    },
    /// Send a webhook notification
    Webhook {
        url: String,
        payload_template: String,
    },
    /// Execute a command
    Command { command: String, args: Vec<String> },
    /// Custom action (handled by a callback)
    Custom(String),
}

/// Alert configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertConfiguration {
    /// Alert name
    pub name: String,
    /// Alert description
    pub description: String,
    /// Alert condition
    pub condition: AlertCondition,
    /// Alert severity
    pub severity: AlertSeverity,
    /// Actions to take when the alert is triggered
    pub actions: Vec<AlertAction>,
    /// How often to check the alert condition (in seconds)
    pub check_interval_seconds: u64,
    /// Minimum time between firing the same alert (in seconds)
    pub minimum_interval_seconds: u64,
    /// Whether the alert is enabled
    pub enabled: bool,
    /// Labels to attach to the alert
    pub labels: HashMap<String, String>,
}

/// Alert state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertState {
    /// Alert is ok
    Ok,
    /// Alert is firing
    Firing,
    /// Alert is acknowledged
    Acknowledged,
    /// Alert is suppressed
    Suppressed,
}

impl std::fmt::Display for AlertState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ok => write!(f, "OK"),
            Self::Firing => write!(f, "FIRING"),
            Self::Acknowledged => write!(f, "ACKNOWLEDGED"),
            Self::Suppressed => write!(f, "SUPPRESSED"),
        }
    }
}

/// Alert instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    /// Alert ID
    pub id: String,
    /// Alert configuration
    pub config: AlertConfiguration,
    /// Current state
    pub state: AlertState,
    /// First time the alert fired
    pub first_fired_at: Option<DateTime<Utc>>,
    /// Last time the alert fired
    pub last_fired_at: Option<DateTime<Utc>>,
    /// Last time the alert was checked
    pub last_checked_at: Option<DateTime<Utc>>,
    /// Value that triggered the alert
    pub triggered_value: Option<MetricValue>,
    /// Number of times the alert has fired
    pub firing_count: u64,
    /// User who acknowledged the alert
    pub acknowledged_by: Option<String>,
    /// Time the alert was acknowledged
    pub acknowledged_at: Option<DateTime<Utc>>,
}

impl Alert {
    /// Create a new alert
    pub fn new(config: AlertConfiguration) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            config,
            state: AlertState::Ok,
            first_fired_at: None,
            last_fired_at: None,
            last_checked_at: None,
            triggered_value: None,
            firing_count: 0,
            acknowledged_by: None,
            acknowledged_at: None,
        }
    }

    /// Check if the alert condition is met
    pub fn check_condition(&self, metrics_collector: &MetricsCollector) -> bool {
        match &self.config.condition {
            AlertCondition::MetricAbove {
                metric_name,
                threshold,
            } => {
                if let Some(metric) = metrics_collector.get_metric(metric_name) {
                    match (&metric.value, threshold) {
                        (MetricValue::Integer(val), MetricValue::Integer(threshold_val)) => {
                            *val > *threshold_val
                        }
                        (MetricValue::Float(val), MetricValue::Float(threshold_val)) => {
                            *val > *threshold_val
                        }
                        _ => false,
                    }
                } else {
                    false
                }
            }
            AlertCondition::MetricBelow {
                metric_name,
                threshold,
            } => {
                if let Some(metric) = metrics_collector.get_metric(metric_name) {
                    match (&metric.value, threshold) {
                        (MetricValue::Integer(val), MetricValue::Integer(threshold_val)) => {
                            *val < *threshold_val
                        }
                        (MetricValue::Float(val), MetricValue::Float(threshold_val)) => {
                            *val < *threshold_val
                        }
                        _ => false,
                    }
                } else {
                    false
                }
            }
            AlertCondition::MetricEquals { metric_name, value } => {
                if let Some(metric) = metrics_collector.get_metric(metric_name) {
                    match (&metric.value, value) {
                        (MetricValue::Integer(val), MetricValue::Integer(equals_val)) => {
                            *val == *equals_val
                        }
                        (MetricValue::Float(val), MetricValue::Float(equals_val)) => {
                            (*val - *equals_val).abs() < f64::EPSILON
                        }
                        (MetricValue::Boolean(val), MetricValue::Boolean(equals_val)) => {
                            *val == *equals_val
                        }
                        (MetricValue::String(val), MetricValue::String(equals_val)) => {
                            *val == *equals_val
                        }
                        _ => false,
                    }
                } else {
                    false
                }
            }
            AlertCondition::And(conditions) => conditions.iter().all(|condition| {
                let mut tmp_alert = self.clone();
                tmp_alert.config.condition = condition.clone();
                tmp_alert.check_condition(metrics_collector)
            }),
            AlertCondition::Or(conditions) => conditions.iter().any(|condition| {
                let mut tmp_alert = self.clone();
                tmp_alert.config.condition = condition.clone();
                tmp_alert.check_condition(metrics_collector)
            }),
            AlertCondition::PercentageChange {
                metric_name,
                percentage,
                duration,
            } => {
                // Get current metric value
                if let Some(metric) = metrics_collector.get_metric(metric_name) {
                    // Get historical values
                    if let Some(history) = metrics_collector.get_metric_history(metric_name) {
                        // Find a value from the past
                        let _now = Utc::now();
                        let target_time = _now - *duration;

                        // Find the closest historical value to the target time
                        let mut closest_index = 0;
                        let mut closest_diff = i64::MAX;

                        for (i, (timestamp, _)) in history.iter().enumerate() {
                            let diff = (timestamp.timestamp() - target_time.timestamp()).abs();
                            if diff < closest_diff {
                                closest_diff = diff;
                                closest_index = i;
                            }
                        }

                        if closest_diff < duration.num_seconds() {
                            let (_, past_value) = &history[closest_index];

                            // Calculate percentage change
                            match (&metric.value, past_value) {
                                (MetricValue::Integer(current), MetricValue::Integer(past)) => {
                                    if *past == 0 {
                                        false
                                    } else {
                                        let change =
                                            (*current as f64 - *past as f64) / *past as f64 * 100.0;
                                        change.abs() > *percentage
                                    }
                                }
                                (MetricValue::Float(current), MetricValue::Float(past)) => {
                                    if *past == 0.0 {
                                        false
                                    } else {
                                        let change = (*current - *past) / *past * 100.0;
                                        change.abs() > *percentage
                                    }
                                }
                                _ => false,
                            }
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            AlertCondition::Custom(_) => {
                // Custom conditions are not implemented yet
                false
            }
        }
    }

    /// Fire the alert
    pub fn fire(&mut self, triggered_value: Option<MetricValue>) {
        let _now = Utc::now();

        // Update state
        self.state = AlertState::Firing;

        // Update timestamps
        if self.first_fired_at.is_none() {
            self.first_fired_at = Some(_now);
        }
        self.last_fired_at = Some(_now);

        // Update triggered value
        self.triggered_value = triggered_value;

        // Increment firing count
        self.firing_count += 1;
    }

    /// Resolve the alert
    pub fn resolve(&mut self) {
        // Update state
        self.state = AlertState::Ok;

        // Clear acknowledged status
        self.acknowledged_by = None;
        self.acknowledged_at = None;
    }

    /// Acknowledge the alert
    pub fn acknowledge(&mut self, user: &str) {
        let _now = Utc::now();

        // Update state
        self.state = AlertState::Acknowledged;

        // Update acknowledged info
        self.acknowledged_by = Some(user.to_string());
        self.acknowledged_at = Some(_now);
    }

    /// Suppress the alert
    pub fn suppress(&mut self) {
        // Update state
        self.state = AlertState::Suppressed;
    }

    /// Update the last checked timestamp
    pub fn update_checked_timestamp(&mut self) {
        self.last_checked_at = Some(Utc::now());
    }

    /// Should this alert be checked now?
    pub fn should_check(&self) -> bool {
        // If not enabled, don't check
        if !self.config.enabled {
            return false;
        }

        // If never checked, definitely check
        if self.last_checked_at.is_none() {
            return true;
        }

        // Check if enough time has passed since the last check
        let _now = Utc::now();
        let duration = Duration::seconds(self.config.check_interval_seconds as i64);

        _now - self.last_checked_at.unwrap() >= duration
    }

    /// Should this alert fire now?
    pub fn should_fire(&self) -> bool {
        // If already firing or acknowledged, don't fire again
        if matches!(self.state, AlertState::Firing | AlertState::Acknowledged) {
            // Check if enough time has passed since the last firing
            if let Some(last_fired_at) = self.last_fired_at {
                let _now = Utc::now();
                let duration = Duration::seconds(self.config.minimum_interval_seconds as i64);

                return _now - last_fired_at >= duration;
            }
        }

        // If suppressed, don't fire
        if matches!(self.state, AlertState::Suppressed) {
            return false;
        }

        // If OK, fire
        matches!(self.state, AlertState::Ok)
    }
}

/// Alert manager
#[derive(Debug)]
pub struct AlertManager {
    /// Alerts by ID
    alerts: RwLock<HashMap<String, Alert>>,
    /// Metrics collector
    metrics_collector: Option<Arc<MetricsCollector>>,
    /// Whether the manager is running
    running: RwLock<bool>,
}

impl AlertManager {
    /// Create a new alert manager
    pub fn new() -> Self {
        Self {
            alerts: RwLock::new(HashMap::new()),
            metrics_collector: None,
            running: RwLock::new(false),
        }
    }

    /// Set the metrics collector
    pub fn set_metrics_collector(&mut self, metrics_collector: Arc<MetricsCollector>) {
        self.metrics_collector = Some(metrics_collector);
    }

    /// Add an alert
    pub fn add_alert(&self, config: AlertConfiguration) -> String {
        let alert = Alert::new(config);
        let id = alert.id.clone();

        let mut alerts = self.alerts.write().unwrap();
        alerts.insert(id.clone(), alert);

        id
    }

    /// Get an alert by ID
    pub fn get_alert(&self, id: &str) -> Option<Alert> {
        let alerts = self.alerts.read().unwrap();
        alerts.get(id).cloned()
    }

    /// Get all alerts
    pub fn get_all_alerts(&self) -> Vec<Alert> {
        let alerts = self.alerts.read().unwrap();
        alerts.values().cloned().collect()
    }

    /// Get active alerts (firing or acknowledged)
    pub fn get_active_alerts(&self) -> Vec<Alert> {
        let alerts = self.alerts.read().unwrap();
        alerts
            .values()
            .filter(|alert| matches!(alert.state, AlertState::Firing | AlertState::Acknowledged))
            .cloned()
            .collect()
    }

    /// Update an existing alert
    ///
    /// # Errors
    ///
    /// Returns an error if the alert is not found
    pub fn update_alert(&self, id: &str, config: AlertConfiguration) -> Result<()> {
        let mut alerts = self.alerts.write().unwrap();
        if let Some(alert) = alerts.get_mut(id) {
            alert.config = config;
            Ok(())
        } else {
            Err(AlertError::NotFound(id.to_string()).into())
        }
    }

    /// Remove an alert
    ///
    /// # Errors
    ///
    /// Returns an error if the alert is not found
    pub fn remove_alert(&self, id: &str) -> Result<()> {
        let mut alerts = self.alerts.write().unwrap();
        if alerts.remove(id).is_some() {
            Ok(())
        } else {
            Err(AlertError::NotFound(id.to_string()).into())
        }
    }

    /// Start the alert manager to begin monitoring and processing alerts
    ///
    /// # Errors
    ///
    /// Returns an error if the alert manager is already running
    pub async fn start(&self) -> Result<()> {
        // Mark the alert manager as running
        {
            let mut running = self.running.write().unwrap();
            if *running {
                return Err(AlertError::AlreadyRunning.into());
            }
            
            // Ensure we have a metrics collector
            if self.metrics_collector.is_none() {
                return Err(AlertError::NoMetricsCollector.into());
            }
            
            *running = true;
        } // Drop the write lock
        
        // Clone the references we need for the async task
        let alerts_ref = self.alerts.read().unwrap().clone();
        let alerts = Arc::new(RwLock::new(alerts_ref));
        let metrics_collector = self.metrics_collector.as_ref().unwrap().clone();
        let running = Arc::new(RwLock::new(true));
        
        // Spawn the alert checking loop
        tokio::spawn(async move {
            info!("Alert manager started");
            
            loop {
                // Check if we should keep running
                {
                    let running_guard = running.read().unwrap();
                    if !*running_guard {
                        break;
                    }
                }
                
                // Check all alerts
                check_alerts(&alerts, &metrics_collector).await;
                
                // Sleep for the check interval
                tokio::time::sleep(std::time::Duration::from_secs(10)).await;
            }
            
            info!("Alert manager stopped");
        });
        
        Ok(())
    }

    /// Stop the alert manager
    ///
    /// # Errors
    ///
    /// Returns an error if the alert manager is not running
    pub async fn stop(&self) -> Result<()> {
        let mut running = self.running.write().unwrap();
        if !*running {
            return Err(AlertError::Other("Alert manager is not running".to_string()).into());
        }
        *running = false;
        Ok(())
    }

    /// Acknowledge an alert
    ///
    /// # Errors
    ///
    /// Returns an error if the alert is not found
    pub fn acknowledge_alert(&self, id: &str, user: &str) -> Result<()> {
        let mut alerts = self.alerts.write().unwrap();
        if let Some(alert) = alerts.get_mut(id) {
            alert.state = AlertState::Acknowledged;
            alert.acknowledged_by = Some(user.to_string());
            alert.acknowledged_at = Some(Utc::now());
            Ok(())
        } else {
            Err(AlertError::NotFound(id.to_string()).into())
        }
    }

    /// Suppress an alert
    ///
    /// # Errors
    ///
    /// Returns an error if the alert is not found
    pub fn suppress_alert(&self, id: &str) -> Result<()> {
        let mut alerts = self.alerts.write().unwrap();
        if let Some(alert) = alerts.get_mut(id) {
            alert.state = AlertState::Suppressed;
            Ok(())
        } else {
            Err(AlertError::NotFound(id.to_string()).into())
        }
    }
}

impl Default for AlertManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Check all alerts
async fn check_alerts(
    alerts: &Arc<RwLock<HashMap<String, Alert>>>,
    metrics_collector: &Arc<MetricsCollector>,
) {
    let _now = Utc::now();
    let mut alerts_to_fire = Vec::new();

    // Check which alerts need to be fired
    {
        let mut alerts_map = alerts.write().unwrap();

        for (id, alert) in alerts_map.iter_mut() {
            // Update checked timestamp
            alert.update_checked_timestamp();

            // Check if we should check this alert
            if !alert.should_check() {
                continue;
            }

            // Check if the condition is met
            let condition_met = alert.check_condition(metrics_collector);

            if condition_met {
                // If condition is met and alert should fire, prepare to fire it
                if alert.should_fire() {
                    // Get the triggered value
                    let triggered_value = match &alert.config.condition {
                        AlertCondition::MetricAbove { metric_name, .. }
                        | AlertCondition::MetricBelow { metric_name, .. }
                        | AlertCondition::MetricEquals { metric_name, .. } => {
                            metrics_collector.get_metric(metric_name).map(|m| m.value)
                        }
                        _ => None,
                    };

                    // Fire the alert
                    alert.fire(triggered_value);

                    // Add to list of alerts to process
                    alerts_to_fire.push((id.clone(), alert.clone()));
                }
            } else {
                // If condition is not met but alert is firing, resolve it
                if matches!(alert.state, AlertState::Firing) {
                    alert.resolve();

                    debug!("Alert resolved: {} ({})", alert.config.name, id);
                }
            }
        }
    }

    // Process fired alerts
    for (id, alert) in alerts_to_fire {
        // Log the alert
        match alert.config.severity {
            AlertSeverity::Info => info!("Alert fired: {} ({})", alert.config.name, id),
            AlertSeverity::Warning => warn!("Alert fired: {} ({})", alert.config.name, id),
            AlertSeverity::Error => error!("Alert fired: {} ({})", alert.config.name, id),
            AlertSeverity::Critical => error!("CRITICAL ALERT: {} ({})", alert.config.name, id),
        }

        // Process alert actions
        for action in &alert.config.actions {
            process_alert_action(action, &alert).await;
        }
    }
}

/// Process an alert action
async fn process_alert_action(action: &AlertAction, _alert: &Alert) {
    match action {
        AlertAction::Log => {
            // Already logged above
        }
        AlertAction::Email {
            recipients,
            subject_template,
            body_template,
        } => {
            debug!(
                "Would send email to {} with subject template '{}' and body template '{}'",
                recipients.join(", "),
                subject_template,
                body_template
            );
            // In a real implementation, this would actually send an email
        }
        AlertAction::Webhook {
            url,
            payload_template,
        } => {
            debug!(
                "Would send webhook to {} with payload template '{}'",
                url, payload_template
            );
            // In a real implementation, this would actually send a webhook
        }
        AlertAction::Command { command, args } => {
            debug!("Would execute command '{}' with args {:?}", command, args);
            // In a real implementation, this would actually execute a command
        }
        AlertAction::Custom(name) => {
            debug!("Would execute custom action '{}'", name);
            // In a real implementation, this would call a custom action handler
        }
    }
}

// Alert errors
#[derive(Debug, Clone)]
pub enum AlertError {
    /// Alert manager is already running
    AlreadyRunning,
    /// No metrics collector available
    NoMetricsCollector,
    /// Alert not found
    NotFound(String),
    /// Other error
    Other(String),
}

impl std::fmt::Display for AlertError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AlreadyRunning => write!(f, "Alert manager is already running"),
            Self::NoMetricsCollector => write!(f, "No metrics collector available"),
            Self::NotFound(id) => write!(f, "Alert not found: {}", id),
            Self::Other(msg) => write!(f, "Alert error: {}", msg),
        }
    }
}

impl std::error::Error for AlertError {}

impl From<AlertError> for MCPError {
    fn from(err: AlertError) -> Self {
        MCPError::Monitoring(err.to_string())
    }
}
