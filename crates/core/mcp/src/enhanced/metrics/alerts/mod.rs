//! Metrics Alert System
//!
//! This module provides intelligent alerting capabilities for the enhanced MCP
//! metrics system, including threshold-based alerts, anomaly detection,
//! and multi-channel notifications.

pub mod notification;
pub mod anomaly;
pub mod suppression;
pub mod channels;

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Mutex, mpsc};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use tracing::{debug, info, warn, error, instrument};

use crate::error::{Result, types::MCPError};
use super::aggregator::{AggregatedMetrics, TrendDirection};
use super::{AlertConfig, AlertThreshold, NotificationChannel, AlertSeverity};

pub use notification::*;
pub use anomaly::*;
pub use suppression::*;
pub use channels::*;

/// Alert manager for handling metric-based alerts
#[derive(Debug)]
pub struct MetricsAlertManager {
    /// Alert configuration
    config: AlertConfig,
    
    /// Active alerts
    active_alerts: Arc<RwLock<HashMap<String, Alert>>>,
    
    /// Alert history
    alert_history: Arc<RwLock<VecDeque<Alert>>>,
    
    /// Alert processors
    processors: Arc<RwLock<HashMap<String, Box<dyn AlertProcessor>>>>,
    
    /// Notification manager
    notification_manager: Arc<NotificationManager>,
    
    /// Anomaly detector
    anomaly_detector: Arc<AnomalyDetector>,
    
    /// Alert suppression manager
    suppression_manager: Arc<AlertSuppressionManager>,
    
    /// Alert state
    state: Arc<RwLock<AlertManagerState>>,
}

/// Alert information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    /// Alert ID
    pub id: String,
    
    /// Alert name
    pub name: String,
    
    /// Alert severity
    pub severity: AlertSeverity,
    
    /// Metric that triggered the alert
    pub metric_name: String,
    
    /// Current metric value
    pub current_value: f64,
    
    /// Threshold that was exceeded
    pub threshold_value: f64,
    
    /// Comparison operator
    pub operator: AlertOperator,
    
    /// Alert message
    pub message: String,
    
    /// Component that generated the alert
    pub component: String,
    
    /// Alert status
    pub status: AlertStatus,
    
    /// First triggered timestamp
    pub first_triggered: DateTime<Utc>,
    
    /// Last triggered timestamp
    pub last_triggered: DateTime<Utc>,
    
    /// Resolution timestamp
    pub resolved_at: Option<DateTime<Utc>>,
    
    /// Number of times triggered
    pub trigger_count: u64,
    
    /// Alert metadata
    pub metadata: HashMap<String, String>,
    
    /// Alert context
    pub context: AlertContext,
}

/// Alert comparison operators
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlertOperator {
    /// Greater than
    GreaterThan,
    /// Less than
    LessThan,
    /// Equal to
    EqualTo,
    /// Not equal to
    NotEqualTo,
}

/// Alert status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlertStatus {
    /// Alert is active and firing
    Active,
    /// Alert has been acknowledged
    Acknowledged,
    /// Alert has been resolved
    Resolved,
    /// Alert has been suppressed
    Suppressed,
}

/// Alert context information
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AlertContext {
    /// Service name
    pub service: Option<String>,
    
    /// Environment
    pub environment: Option<String>,
    
    /// Region
    pub region: Option<String>,
    
    /// Host information
    pub host: Option<String>,
    
    /// Process ID
    pub process_id: Option<u32>,
    
    /// Thread ID
    pub thread_id: Option<String>,
    
    /// Request ID
    pub request_id: Option<String>,
    
    /// Session ID
    pub session_id: Option<String>,
    
    /// User ID
    pub user_id: Option<String>,
    
    /// Additional context
    pub additional: HashMap<String, String>,
}

/// Alert processing result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertProcessingResult {
    /// Processing status
    pub status: ProcessingStatus,
    
    /// Processing duration
    pub duration: Duration,
    
    /// Alerts generated
    pub alerts_generated: usize,
    
    /// Alerts suppressed
    pub alerts_suppressed: usize,
    
    /// Notifications sent
    pub notifications_sent: usize,
    
    /// Errors encountered
    pub errors: Vec<String>,
}

/// Alert processing status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProcessingStatus {
    /// Processing successful
    Success,
    /// Processing failed with errors
    Failed,
    /// Processing partially successful
    Partial,
}

/// Alert processor capability
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlertCapability {
    /// Threshold-based alerts
    ThresholdAlerts,
    /// Anomaly detection
    AnomalyDetection,
    /// Trend analysis
    TrendAnalysis,
    /// Correlation analysis
    CorrelationAnalysis,
}

/// Alert manager state
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AlertManagerState {
    /// Manager status
    pub status: AlertManagerStatus,
    
    /// Total alerts processed
    pub total_alerts_processed: u64,
    
    /// Active alerts count
    pub active_alerts_count: usize,
    
    /// Suppressed alerts count
    pub suppressed_alerts_count: usize,
    
    /// Processing performance
    pub processing_performance: AlertProcessingPerformance,
    
    /// Last processing time
    pub last_processing_time: Option<DateTime<Utc>>,
}

/// Alert manager status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum AlertManagerStatus {
    /// Manager is initializing
    #[default]
    Initializing,
    /// Manager is active and processing
    Active,
    /// Manager is paused
    Paused,
    /// Manager has failed
    Failed,
}

/// Alert processing performance metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AlertProcessingPerformance {
    /// Average processing time per alert
    pub avg_processing_time: Duration,
    
    /// Total processing time
    pub total_processing_time: Duration,
    
    /// Processing throughput (alerts per second)
    pub throughput: f64,
}

/// Alert processor trait
pub trait AlertProcessor: Send + Sync + std::fmt::Debug {
    /// Process metrics and generate alerts
    fn process_metrics(&self, metrics: &AggregatedMetrics) -> Result<Vec<Alert>>;
    
    /// Get processor capabilities
    fn capabilities(&self) -> Vec<AlertCapability>;
    
    /// Get processor name
    fn name(&self) -> &str;
}

impl MetricsAlertManager {
    /// Create a new alert manager
    pub fn new(config: AlertConfig) -> Self {
        Self {
            config: config.clone(),
            active_alerts: Arc::new(RwLock::new(HashMap::new())),
            alert_history: Arc::new(RwLock::new(VecDeque::new())),
            processors: Arc::new(RwLock::new(HashMap::new())),
            notification_manager: Arc::new(NotificationManager::new(config.notification_config.clone())),
            anomaly_detector: Arc::new(AnomalyDetector::new(config.anomaly_config.clone())),
            suppression_manager: Arc::new(AlertSuppressionManager::new(config.suppression_config.clone())),
            state: Arc::new(RwLock::new(AlertManagerState::default())),
        }
    }

    /// Process metrics and generate alerts
    #[instrument(skip(self))]
    pub async fn process_metrics(&self, metrics: &AggregatedMetrics) -> Result<AlertProcessingResult> {
        let start_time = Instant::now();
        let mut alerts_generated = 0;
        let mut alerts_suppressed = 0;
        let mut notifications_sent = 0;
        let mut errors = Vec::new();

        debug!("🚨 Processing metrics for alerts: {} metrics", metrics.metric_values.len());

        // Process each metric
        for (metric_name, metric_value) in &metrics.metric_values {
            match self.process_metric(metric_name, *metric_value, metrics).await {
                Ok(result) => {
                    alerts_generated += result.alerts_generated;
                    alerts_suppressed += result.alerts_suppressed;
                    notifications_sent += result.notifications_sent;
                    errors.extend(result.errors);
                }
                Err(e) => {
                    error!("Failed to process metric {}: {}", metric_name, e);
                    errors.push(format!("Metric processing error for {}: {}", metric_name, e));
                }
            }
        }

        // Check for anomalies
        match self.anomaly_detector.detect_anomalies(metrics).await {
            Ok(anomalies) => {
                for anomaly in anomalies {
                    match self.handle_anomaly(anomaly).await {
                        Ok(alert) => {
                            if let Some(alert) = alert {
                                alerts_generated += 1;
                                if let Err(e) = self.send_alert_notification(&alert).await {
                                    errors.push(format!("Notification error: {}", e));
                                } else {
                                    notifications_sent += 1;
                                }
                            }
                        }
                        Err(e) => {
                            error!("Failed to handle anomaly: {}", e);
                            errors.push(format!("Anomaly handling error: {}", e));
                        }
                    }
                }
            }
            Err(e) => {
                error!("Anomaly detection failed: {}", e);
                errors.push(format!("Anomaly detection error: {}", e));
            }
        }

        let duration = start_time.elapsed();
        let status = if errors.is_empty() {
            ProcessingStatus::Success
        } else if alerts_generated > 0 {
            ProcessingStatus::Partial
        } else {
            ProcessingStatus::Failed
        };

        // Update state
        self.update_processing_state(duration, alerts_generated, alerts_suppressed).await;

        info!("🚨 Alert processing completed: {} alerts generated, {} suppressed, {} notifications sent in {:?}",
              alerts_generated, alerts_suppressed, notifications_sent, duration);

        Ok(AlertProcessingResult {
            status,
            duration,
            alerts_generated,
            alerts_suppressed,
            notifications_sent,
            errors,
        })
    }

    /// Process a single metric
    async fn process_metric(&self, metric_name: &str, metric_value: f64, context: &AggregatedMetrics) -> Result<AlertProcessingResult> {
        // Implementation continues...
        // This would include the rest of the MetricsAlertManager implementation
        // For now, returning a placeholder
        Ok(AlertProcessingResult {
            status: ProcessingStatus::Success,
            duration: Duration::from_millis(1),
            alerts_generated: 0,
            alerts_suppressed: 0,
            notifications_sent: 0,
            errors: Vec::new(),
        })
    }

    /// Handle anomaly detection result
    async fn handle_anomaly(&self, anomaly: AnomalyResult) -> Result<Option<Alert>> {
        // Placeholder implementation
        Ok(None)
    }

    /// Send alert notification
    async fn send_alert_notification(&self, alert: &Alert) -> Result<()> {
        // Placeholder implementation
        Ok(())
    }

    /// Update processing state
    async fn update_processing_state(&self, duration: Duration, alerts_generated: usize, alerts_suppressed: usize) {
        // Placeholder implementation
    }

    /// Get current state
    pub async fn get_state(&self) -> AlertManagerState {
        self.state.read().await.clone()
    }

    /// Get active alerts
    pub async fn get_active_alerts(&self) -> Vec<Alert> {
        self.active_alerts.read().await.values().cloned().collect()
    }

    /// Acknowledge alert
    pub async fn acknowledge_alert(&self, alert_id: &str) -> Result<()> {
        // Placeholder implementation
        Ok(())
    }

    /// Resolve alert
    pub async fn resolve_alert(&self, alert_id: &str) -> Result<()> {
        // Placeholder implementation
        Ok(())
    }
} 