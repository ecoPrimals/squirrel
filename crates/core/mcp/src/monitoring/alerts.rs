// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

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
        /// Name of the metric to monitor
        metric_name: String,
        /// Threshold value that triggers the alert when exceeded
        threshold: MetricValue,
    },
    /// Metric falls below a threshold
    MetricBelow {
        /// Name of the metric to monitor
        metric_name: String,
        /// Threshold value that triggers the alert when the metric falls below it
        threshold: MetricValue,
    },
    /// Metric equals a value
    MetricEquals {
        /// Name of the metric to monitor
        metric_name: String,
        /// Exact value that triggers the alert when matched
        value: MetricValue,
    },
    /// Composite condition (AND)
    And(Vec<AlertCondition>),
    /// Composite condition (OR)
    Or(Vec<AlertCondition>),
    /// Metric changes by a percentage
    PercentageChange {
        /// Name of the metric to monitor
        metric_name: String,
        /// Percentage change that triggers the alert (e.g., 20.0 for 20%)
        percentage: f64,
        /// Time window over which to observe the change
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
        /// List of email addresses to receive the alert
        recipients: Vec<String>,
        /// Template for the email subject line
        subject_template: String,
        /// Template for the email body content
        body_template: String,
    },
    /// Send a webhook notification
    Webhook {
        /// Target URL for the webhook POST request
        url: String,
        /// Template for the JSON payload to be sent
        payload_template: String,
    },
    /// Execute a command
    Command { 
        /// Command to execute
        command: String, 
        /// Command-line arguments
        args: Vec<String> 
    },
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
    #[must_use] pub fn new(config: AlertConfiguration) -> Self {
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
                        let now = Utc::now();
                        let target_time = now - *duration;

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
        let now = Utc::now();

        // Update state
        self.state = AlertState::Firing;

        // Update timestamps
        if self.first_fired_at.is_none() {
            self.first_fired_at = Some(now);
        }
        self.last_fired_at = Some(now);

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
        let now = Utc::now();

        // Update state
        self.state = AlertState::Acknowledged;

        // Update acknowledged info
        self.acknowledged_by = Some(user.to_string());
        self.acknowledged_at = Some(now);
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

    /// Determines if the alert should be checked based on the last checked time and interval.
    fn should_check(&self) -> bool {
        self.last_checked_at.map_or(true, |last_checked| {
            // Check if enough time has passed since the last check
            let now = Utc::now();
            // Convert check_interval_seconds (u64) to i64 for Duration::seconds
            let duration = i64::try_from(self.config.check_interval_seconds).map_or_else(|_| {
                warn!("Invalid check_interval_seconds: {}. Using default of 60s", self.config.check_interval_seconds);
                Duration::seconds(60) // Default to 60 seconds if cast fails
            }, |secs| Duration::seconds(secs));
             // Use signed_duration_since for DateTime subtraction
            now.signed_duration_since(last_checked) >= duration
        })
    }

    /// Should this alert fire now?
    #[must_use] pub fn should_fire(&self) -> bool {
        // If already firing or acknowledged, don't fire again
        if matches!(self.state, AlertState::Firing | AlertState::Acknowledged) {
            // Check if enough time has passed since the last firing
            if let Some(last_fired_at) = self.last_fired_at {
                let now = Utc::now();
                #[allow(clippy::cast_possible_wrap)] // Allow u64->i64 for Duration
                let duration = Duration::seconds(self.config.minimum_interval_seconds as i64);

                return now - last_fired_at >= duration;
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
    alerts: Arc<RwLock<HashMap<String, Alert>>>,
    /// Metrics collector
    metrics_collector: Option<Arc<MetricsCollector>>,
    /// Whether the manager is running (shared state for task)
    running: Arc<RwLock<bool>>,
    /// Check interval in seconds (default: 10)
    check_interval_secs: u64,
}

impl AlertManager {
    /// Create a new `AlertManager` with default check interval (10 seconds)
    #[must_use]
    pub fn new() -> Self {
        Self {
            alerts: Arc::new(RwLock::new(HashMap::new())),
            metrics_collector: None,
            running: Arc::new(RwLock::new(false)), // Wrap in Arc
            check_interval_secs: 10,
        }
    }

    /// Create a new `AlertManager` with custom check interval
    #[must_use]
    pub fn with_check_interval(check_interval_secs: u64) -> Self {
        Self {
            alerts: Arc::new(RwLock::new(HashMap::new())),
            metrics_collector: None,
            running: Arc::new(RwLock::new(false)),
            check_interval_secs,
        }
    }

    /// Set the check interval for alert monitoring
    pub fn set_check_interval(&mut self, check_interval_secs: u64) {
        self.check_interval_secs = check_interval_secs;
    }

    /// Set the metrics collector
    pub fn set_metrics_collector(&mut self, metrics_collector: Arc<MetricsCollector>) {
        self.metrics_collector = Some(metrics_collector);
    }

    /// Add an alert
    pub fn add_alert(&self, config: AlertConfiguration) -> String {
        let alert = Alert::new(config);
        let id = alert.id.clone();

        // Corrected logic: Directly attempt to get write lock
        match self.alerts.write() {
            Ok(mut alerts) => {
                alerts.insert(id.clone(), alert);
            }
            Err(e) => {
                error!("Failed to acquire alerts write lock for add_alert: {}", e);
                // Alert might not be added, but we still return the generated ID
            }
        }

        id
    }

    /// Get an alert by ID
    pub fn get_alert(&self, id: &str) -> Option<Alert> {
        match self.alerts.read() {
            Ok(alerts) => alerts.get(id).cloned(),
            Err(e) => {
                error!("Failed to acquire alerts read lock for get_alert: {}", e);
                None
            }
        }
    }

    /// Get all alerts
    pub fn get_all_alerts(&self) -> Vec<Alert> {
        match self.alerts.read() {
            Ok(alerts) => alerts.values().cloned().collect(),
            Err(e) => {
                error!("Failed to acquire alerts read lock for get_all_alerts: {}", e);
                Vec::new()
            }
        }
    }

    /// Get active alerts (firing or acknowledged)
    pub fn get_active_alerts(&self) -> Vec<Alert> {
        match self.alerts.read() {
            Ok(alerts) => alerts
                .values()
                .filter(|alert| matches!(alert.state, AlertState::Firing | AlertState::Acknowledged))
                .cloned()
                .collect(),
            Err(e) => {
                error!("Failed to acquire alerts read lock for get_active_alerts: {}", e);
                Vec::new()
            }
        }
    }

    /// Update an existing alert
    ///
    /// # Errors
    ///
    /// Returns an error if the alert is not found or the lock is poisoned
    pub fn update_alert(&self, id: &str, config: AlertConfiguration) -> Result<()> {
        let mut alerts = match self.alerts.write() {
            Ok(guard) => guard,
            Err(e) => {
                error!("Failed to acquire alerts lock for update_alert: {}", e);
                return Err(AlertError::Other("Failed to acquire lock".to_string()).into());
            }
        };
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
    /// Returns an error if the alert is not found or the lock is poisoned
    pub fn remove_alert(&self, id: &str) -> Result<()> {
        let mut alerts = match self.alerts.write() {
            Ok(guard) => guard,
            Err(e) => {
                error!("Failed to acquire alerts lock for remove_alert: {}", e);
                return Err(AlertError::Other("Failed to acquire lock".to_string()).into());
            }
        };
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
    /// Returns an error if the alert manager is already running, no metrics collector is set, or a lock is poisoned.
    pub async fn start(&self) -> Result<()> {
        // Retrieve the shared MetricsCollector
        let Some(metrics_collector) = self.metrics_collector.clone() else {
            return Err(AlertError::NoMetricsCollector.into());
        };

        // Mark the alert manager as running
        {
            let mut running_guard = match self.running.write() { // Use self.running directly
                Ok(guard) => guard,
                Err(e) => {
                    error!("Failed to acquire running write lock for start: {}", e);
                    return Err(AlertError::Other("Failed to acquire lock".to_string()).into());
                }
            };
            if *running_guard {
                return Err(AlertError::AlreadyRunning.into());
            }
            *running_guard = true;
        } // Drop the write lock
        
        // Clone the alerts map (needs read lock)
        let alerts_map = match self.alerts.read() {
            Ok(alerts_guard) => alerts_guard.clone(),
            Err(e) => {
                error!("Failed to acquire alerts read lock for start: {}", e);
                 // Attempt to reset running state before returning error
                {
                    match self.running.write() {
                        Ok(mut running_guard) => *running_guard = false,
                        Err(e_run) => error!("Failed to re-acquire running lock to reset state: {}", e_run),
                    }
                }
                return Err(AlertError::Other("Failed to acquire lock".to_string()).into());
            }
        };

        // Create a new Arc<RwLock> for the cloned alerts map for the task
        let task_alerts = Arc::new(RwLock::new(alerts_map)); 
        let task_running = self.running.clone(); // Clone the Arc<RwLock<bool>> 
        let check_interval_secs = self.check_interval_secs; // Capture check interval
        
        // Spawn the alert checking loop
        tokio::spawn(async move {
            info!("Alert manager started");
            
            loop {
                // Check if we should keep running
                let should_run = match task_running.read() { // Use task_running
                    Ok(running_guard) => *running_guard,
                    Err(e) => {
                        error!("Failed to acquire running read lock in alert loop: {}", e);
                        false // Stop loop if lock is poisoned
                    }
                };

                if !should_run {
                    break;
                }
                
                // Check all alerts using the task's alerts map
                check_alerts(&task_alerts, &metrics_collector).await; // Pass task_alerts
                
                // Sleep for the configured check interval
                tokio::time::sleep(std::time::Duration::from_secs(check_interval_secs)).await;
            }
            
            info!("Alert manager stopped");
        });
        
        Ok(())
    }

    /// Stop the alert manager
    ///
    /// # Errors
    ///
    /// Returns an error if the alert manager is not running or the lock is poisoned
    pub async fn stop(&self) -> Result<()> {
        let mut running_guard = match self.running.write() { // Use self.running
            Ok(guard) => guard,
            Err(e) => {
                error!("Failed to acquire running lock for stop: {}", e);
                return Err(AlertError::Other("Failed to acquire lock".to_string()).into());
            }
        };
        if !*running_guard {
            // Changed error type to match function description (was Other)
            return Err(AlertError::Other("Alert manager is not running".to_string()).into()); 
        }
        *running_guard = false;
        drop(running_guard);

        Ok(())
    }

    /// Acknowledge an alert
    ///
    /// # Errors
    ///
    /// Returns an error if the alert is not found or the lock is poisoned
    pub fn acknowledge_alert(&self, id: &str, user: &str) -> Result<()> {
        let mut alerts = match self.alerts.write() {
            Ok(guard) => guard,
            Err(e) => {
                error!("Failed to acquire alerts lock for acknowledge_alert: {}", e);
                return Err(AlertError::Other("Failed to acquire lock".to_string()).into());
            }
        };
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
    /// Returns an error if the alert is not found or the lock is poisoned
    pub fn suppress_alert(&self, id: &str) -> Result<()> {
        let mut alerts = match self.alerts.write() {
            Ok(guard) => guard,
            Err(e) => {
                error!("Failed to acquire alerts lock for suppress_alert: {}", e);
                return Err(AlertError::Other("Failed to acquire lock".to_string()).into());
            }
        };
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
        let mut alerts_map = match alerts.write() {
            Ok(guard) => guard,
            Err(e) => {
                error!("Failed to acquire alerts lock for check_alerts: {}", e);
                return; // Exit if we can't get the lock
            }
        };

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

/// Errors that can occur during alert operations
#[derive(Debug)]
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
            Self::NotFound(id) => write!(f, "Alert not found: {id}"),
            Self::Other(msg) => write!(f, "Alert error: {msg}"),
        }
    }
}

impl std::error::Error for AlertError {}

impl From<AlertError> for MCPError {
    fn from(err: AlertError) -> Self {
        Self::Monitoring(err.to_string())
    }
}
