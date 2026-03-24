// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Workflow Monitoring System
//!
//! Provides real-time monitoring, metrics collection, and alerting for workflows.
//! Tracks performance, errors, and resource usage.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};

use crate::error::{Result, types::MCPError};

/// Workflow monitoring system
///
/// Provides real-time monitoring, metrics collection, and alerting for workflows.
/// Tracks performance, errors, and resource usage.
#[derive(Debug)]
pub struct WorkflowMonitoring {
    /// Metrics storage
    metrics: Arc<RwLock<MonitoringMetrics>>,
    
    /// Alert rules
    alert_rules: Arc<RwLock<Vec<AlertRule>>>,
    
    /// Monitoring configuration
    config: MonitoringConfig,
    
    /// Active alerts
    active_alerts: Arc<RwLock<Vec<Alert>>>,
}

/// Monitoring configuration
#[derive(Debug, Clone)]
pub struct MonitoringConfig {
    /// Enable monitoring
    pub enabled: bool,
    
    /// Metrics retention period
    pub retention_period: Duration,
    
    /// Alert check interval
    pub alert_check_interval: Duration,
    
    /// Maximum alerts to keep
    pub max_alerts: usize,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            retention_period: Duration::from_secs(86400), // 24 hours
            alert_check_interval: Duration::from_secs(60),
            max_alerts: 1000,
        }
    }
}

/// Monitoring metrics
#[derive(Debug, Clone, Default)]
pub struct MonitoringMetrics {
    /// Total workflows executed
    pub total_workflows: u64,
    
    /// Successful workflows
    pub successful_workflows: u64,
    
    /// Failed workflows
    pub failed_workflows: u64,
    
    /// Average execution time (milliseconds)
    pub avg_execution_time: f64,
    
    /// Peak execution time (milliseconds)
    pub peak_execution_time: u64,
    
    /// Active workflows
    pub active_workflows: u64,
    
    /// Total steps executed
    pub total_steps: u64,
    
    /// Failed steps
    pub failed_steps: u64,
    
    /// Metrics by workflow ID
    pub workflow_metrics: HashMap<String, WorkflowMetricData>,
    
    /// Last updated
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// Per-workflow metric data
#[derive(Debug, Clone)]
pub struct WorkflowMetricData {
    /// Execution count
    pub execution_count: u64,
    
    /// Success count
    pub success_count: u64,
    
    /// Failure count
    pub failure_count: u64,
    
    /// Average duration
    pub avg_duration: f64,
    
    /// Last execution
    pub last_execution: chrono::DateTime<chrono::Utc>,
}

/// Alert rule
#[derive(Debug, Clone)]
pub struct AlertRule {
    /// Rule ID
    pub id: String,
    
    /// Rule name
    pub name: String,
    
    /// Condition type
    pub condition: AlertCondition,
    
    /// Threshold value
    pub threshold: f64,
    
    /// Alert severity
    pub severity: AlertSeverity,
    
    /// Enabled
    pub enabled: bool,
}

/// Alert condition types
#[derive(Debug, Clone)]
pub enum AlertCondition {
    /// Failure rate exceeds threshold
    FailureRate,
    
    /// Execution time exceeds threshold
    ExecutionTime,
    
    /// Active workflows exceed threshold
    ActiveWorkflows,
    
    /// Step failure rate exceeds threshold
    StepFailureRate,
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    /// Informational
    Info,
    
    /// Warning
    Warning,
    
    /// Error
    Error,
    
    /// Critical
    Critical,
}

/// Active alert
#[derive(Debug, Clone)]
pub struct Alert {
    /// Alert ID
    pub id: String,
    
    /// Rule ID that triggered
    pub rule_id: String,
    
    /// Alert message
    pub message: String,
    
    /// Severity
    pub severity: AlertSeverity,
    
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    
    /// Acknowledged
    pub acknowledged: bool,
    
    /// Metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl WorkflowMonitoring {
    /// Create a new monitoring system
    pub fn new(config: MonitoringConfig) -> Self {
        Self {
            metrics: Arc::new(RwLock::new(MonitoringMetrics::default())),
            alert_rules: Arc::new(RwLock::new(Vec::new())),
            config,
            active_alerts: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    /// Record workflow execution
    pub async fn record_execution(
        &self,
        workflow_id: &str,
        success: bool,
        duration_ms: u64,
    ) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }
        
        let mut metrics = self.metrics.write().await;
        
        // Update global metrics
        metrics.total_workflows += 1;
        if success {
            metrics.successful_workflows += 1;
        } else {
            metrics.failed_workflows += 1;
        }
        
        // Update average execution time
        let total_time = metrics.avg_execution_time * (metrics.total_workflows - 1) as f64;
        metrics.avg_execution_time =
            (total_time + duration_ms as f64) / metrics.total_workflows as f64;
        
        // Update peak execution time
        if duration_ms > metrics.peak_execution_time {
            metrics.peak_execution_time = duration_ms;
        }
        
        // Update per-workflow metrics
        let workflow_metric = metrics
            .workflow_metrics
            .entry(workflow_id.to_string())
            .or_insert_with(|| WorkflowMetricData {
                execution_count: 0,
                success_count: 0,
                failure_count: 0,
                avg_duration: 0.0,
                last_execution: chrono::Utc::now(),
            });
        
        workflow_metric.execution_count += 1;
        if success {
            workflow_metric.success_count += 1;
        } else {
            workflow_metric.failure_count += 1;
        }
        
        let total_duration =
            workflow_metric.avg_duration * (workflow_metric.execution_count - 1) as f64;
        workflow_metric.avg_duration =
            (total_duration + duration_ms as f64) / workflow_metric.execution_count as f64;
        workflow_metric.last_execution = chrono::Utc::now();
        
        metrics.last_updated = chrono::Utc::now();
        
        // Check alert rules
        drop(metrics); // Release write lock before checking alerts
        self.check_alerts_async().await?;
        
        Ok(())
    }
    
    /// Record step execution
    pub async fn record_step(&self, success: bool) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }
        
        let mut metrics = self.metrics.write().await;
        metrics.total_steps += 1;
        if !success {
            metrics.failed_steps += 1;
        }
        
        Ok(())
    }
    
    /// Update active workflow count
    pub async fn update_active_count(&self, count: u64) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }
        
        let mut metrics = self.metrics.write().await;
        metrics.active_workflows = count;
        
        Ok(())
    }
    
    /// Get current metrics
    pub async fn get_metrics(&self) -> Result<MonitoringMetrics> {
        let metrics = self.metrics.read().await;
        Ok(metrics.clone())
    }
    
    /// Add alert rule
    pub async fn add_alert_rule(&self, rule: AlertRule) -> Result<()> {
        let mut rules = self.alert_rules.write().await;
        rules.push(rule);
        Ok(())
    }
    
    /// Check alert rules (async wrapper)
    async fn check_alerts_async(&self) -> Result<()> {
        let metrics = self.metrics.read().await;
        self.check_alerts(&metrics).await
    }
    
    /// Check alert rules
    async fn check_alerts(&self, metrics: &MonitoringMetrics) -> Result<()> {
        let rules = self.alert_rules.read().await;
        
        for rule in rules.iter() {
            if !rule.enabled {
                continue;
            }
            
            let triggered = match &rule.condition {
                AlertCondition::FailureRate => {
                    let rate = if metrics.total_workflows > 0 {
                        metrics.failed_workflows as f64 / metrics.total_workflows as f64
                    } else {
                        0.0
                    };
                    rate > rule.threshold
                }
                AlertCondition::ExecutionTime => metrics.avg_execution_time > rule.threshold,
                AlertCondition::ActiveWorkflows => {
                    metrics.active_workflows as f64 > rule.threshold
                }
                AlertCondition::StepFailureRate => {
                    let rate = if metrics.total_steps > 0 {
                        metrics.failed_steps as f64 / metrics.total_steps as f64
                    } else {
                        0.0
                    };
                    rate > rule.threshold
                }
            };
            
            if triggered {
                drop(rules); // Release read lock before creating alert
                self.create_alert(rule).await?;
                return Ok(()); // Exit after first triggered alert
            }
        }
        
        Ok(())
    }
    
    /// Create alert
    async fn create_alert(&self, rule: &AlertRule) -> Result<()> {
        let alert = Alert {
            id: uuid::Uuid::new_v4().to_string(),
            rule_id: rule.id.clone(),
            message: format!("Alert triggered: {}", rule.name),
            severity: rule.severity.clone(),
            timestamp: chrono::Utc::now(),
            acknowledged: false,
            metadata: HashMap::new(),
        };
        
        let mut alerts = self.active_alerts.write().await;
        alerts.push(alert);
        
        // Keep only max_alerts
        if alerts.len() > self.config.max_alerts {
            alerts.remove(0);
        }
        
        Ok(())
    }
    
    /// Get active alerts
    pub async fn get_alerts(&self, unacknowledged_only: bool) -> Result<Vec<Alert>> {
        let alerts = self.active_alerts.read().await;
        
        if unacknowledged_only {
            Ok(alerts
                .iter()
                .filter(|a| !a.acknowledged)
                .cloned()
                .collect())
        } else {
            Ok(alerts.clone())
        }
    }
    
    /// Acknowledge alert
    pub async fn acknowledge_alert(&self, alert_id: &str) -> Result<()> {
        let mut alerts = self.active_alerts.write().await;
        
        if let Some(alert) = alerts.iter_mut().find(|a| a.id == alert_id) {
            alert.acknowledged = true;
            Ok(())
        } else {
            Err(MCPError::InvalidArgument(format!("Alert not found: {}", alert_id)).into())
        }
    }
    
    /// Get workflow-specific metrics
    pub async fn get_workflow_metrics(
        &self,
        workflow_id: &str,
    ) -> Result<Option<WorkflowMetricData>> {
        let metrics = self.metrics.read().await;
        Ok(metrics.workflow_metrics.get(workflow_id).cloned())
    }
    
    /// Get alert rules
    pub async fn get_alert_rules(&self) -> Result<Vec<AlertRule>> {
        let rules = self.alert_rules.read().await;
        Ok(rules.clone())
    }
    
    /// Remove alert rule
    pub async fn remove_alert_rule(&self, rule_id: &str) -> Result<()> {
        let mut rules = self.alert_rules.write().await;
        if let Some(pos) = rules.iter().position(|r| r.id == rule_id) {
            rules.remove(pos);
            Ok(())
        } else {
            Err(MCPError::InvalidArgument(format!("Alert rule not found: {}", rule_id)).into())
        }
    }
    
    /// Clear all metrics
    pub async fn clear_metrics(&self) -> Result<()> {
        let mut metrics = self.metrics.write().await;
        *metrics = MonitoringMetrics::default();
        Ok(())
    }
    
    /// Get configuration
    pub fn config(&self) -> &MonitoringConfig {
        &self.config
    }
}

