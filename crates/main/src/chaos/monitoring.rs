//! Chaos Experiment Monitoring
//!
//! Real-time monitoring and metrics collection during chaos engineering experiments.
//! Provides system health tracking, performance metrics, and experiment observability.

use super::{ChaosError, MetricValue};
use crate::monitoring::MonitoringConfig;
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc,
};
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::{Mutex, RwLock};
use tokio::time::{interval, sleep};
use uuid::Uuid;

/// Monitoring system for chaos experiments
#[derive(Debug)]
pub struct ChaosMonitor {
    /// Active monitoring sessions
    active_sessions: Arc<RwLock<HashMap<String, MonitoringSession>>>,
    /// System metrics collector
    metrics_collector: Arc<SystemMetricsCollector>,
    /// Alert manager
    alert_manager: Arc<AlertManager>,
}

/// Active monitoring session
#[derive(Debug)]
pub struct MonitoringSession {
    /// Session identifier
    pub session_id: String,
    /// Monitoring configuration
    pub config: MonitoringConfig,
    /// Start time
    pub start_time: Instant,
    /// Collected metrics
    pub metrics: Arc<Mutex<Vec<MetricValue>>>,
    /// Cancellation token
    pub cancel_token: Arc<AtomicBool>,
    /// Collection task handle
    pub collection_handle: Option<tokio::task::JoinHandle<()>>,
}

/// System metrics collection implementation
#[derive(Debug)]
pub struct SystemMetricsCollector {
    /// CPU usage tracker
    cpu_tracker: Arc<CpuUsageTracker>,
    /// Memory usage tracker
    memory_tracker: Arc<MemoryUsageTracker>,
    /// Network metrics tracker
    network_tracker: Arc<NetworkMetricsTracker>,
    /// Request metrics tracker
    request_tracker: Arc<RequestMetricsTracker>,
}

/// CPU usage tracking
#[derive(Debug)]
pub struct CpuUsageTracker {
    /// Current CPU usage percentage
    current_usage: Arc<std::sync::atomic::AtomicU64>,
    /// Usage history
    usage_history: Arc<Mutex<Vec<(SystemTime, f64)>>>,
}

/// Memory usage tracking
#[derive(Debug)]
pub struct MemoryUsageTracker {
    /// Current memory usage in bytes
    current_usage: Arc<AtomicU64>,
    /// Memory usage history
    usage_history: Arc<Mutex<Vec<(SystemTime, u64)>>>,
}

/// Network metrics tracking
#[derive(Debug)]
pub struct NetworkMetricsTracker {
    /// Request count
    request_count: Arc<AtomicU64>,
    /// Response time measurements
    response_times: Arc<Mutex<Vec<(SystemTime, u64)>>>,
}

/// Request metrics tracking
#[derive(Debug)]
pub struct RequestMetricsTracker {
    /// Total requests processed
    total_requests: Arc<AtomicU64>,
    /// Successful requests
    successful_requests: Arc<AtomicU64>,
    /// Failed requests
    failed_requests: Arc<AtomicU64>,
    /// Response time measurements
    response_times: Arc<Mutex<Vec<u64>>>,
}

/// Alert management system
#[derive(Debug)]
pub struct AlertManager {
    /// Active alerts
    active_alerts: Arc<RwLock<HashMap<String, Alert>>>,
    /// Alert history
    alert_history: Arc<Mutex<Vec<Alert>>>,
}

/// System alert
#[derive(Debug, Clone)]
pub struct Alert {
    /// Alert ID
    pub id: String,
    /// Alert level
    pub level: AlertLevel,
    /// Metric that triggered the alert
    pub metric_name: String,
    /// Current value
    pub current_value: f64,
    /// Threshold that was exceeded
    pub threshold: f64,
    /// Timestamp when alert was triggered
    pub timestamp: SystemTime,
    /// Alert message
    pub message: String,
}

/// Alert severity levels
#[derive(Debug, Clone)]
pub enum AlertLevel {
    /// Information alert
    Info,
    /// Warning alert
    Warning,
    /// Critical alert
    Critical,
}

impl ChaosMonitor {
    /// Create a new chaos monitoring system
    pub fn new() -> Self {
        Self {
            active_sessions: Arc::new(RwLock::new(HashMap::new())),
            metrics_collector: Arc::new(SystemMetricsCollector::new()),
            alert_manager: Arc::new(AlertManager::new()),
        }
    }

    /// Start monitoring for an experiment
    pub async fn start_monitoring(&self, config: &MonitoringConfig) -> Result<String, ChaosError> {
        let session_id = Uuid::new_v4().to_string();
        let cancel_token = Arc::new(AtomicBool::new(false));
        let metrics = Arc::new(Mutex::new(Vec::new()));

        let session = MonitoringSession {
            session_id: session_id.clone(),
            config: config.clone(),
            start_time: Instant::now(),
            metrics: metrics.clone(),
            cancel_token: cancel_token.clone(),
            collection_handle: None,
        };

        // Start metrics collection task
        let collector = self.metrics_collector.clone();
        let alert_manager = self.alert_manager.clone();
        let config_clone = config.clone();
        let session_id_clone = session_id.clone();

        let collection_handle = tokio::spawn(async move {
            Self::metrics_collection_task(
                collector,
                alert_manager,
                config_clone,
                session_id_clone,
                metrics,
                cancel_token,
            )
            .await;
        });

        let mut session = session;
        session.collection_handle = Some(collection_handle);

        // Register monitoring session
        {
            let mut sessions = self.active_sessions.write().await;
            sessions.insert(session_id.clone(), session);
        }

        Ok(session_id)
    }

    /// Stop monitoring and return collected metrics
    pub async fn stop_monitoring(
        &self,
        session_id: String,
    ) -> Result<Vec<MetricValue>, ChaosError> {
        let session = {
            let mut sessions = self.active_sessions.write().await;
            sessions.remove(&session_id)
        };

        if let Some(session) = session {
            // Signal cancellation
            session.cancel_token.store(true, Ordering::SeqCst);

            // Wait for collection task to complete
            if let Some(handle) = session.collection_handle {
                let _ = handle.await;
            }

            // Return collected metrics
            let metrics = session.metrics.lock().await;
            Ok(metrics.clone())
        } else {
            Err(ChaosError::MonitoringError(format!(
                "Monitoring session not found: {}",
                session_id
            )))
        }
    }

    /// Metrics collection background task
    async fn metrics_collection_task(
        collector: Arc<SystemMetricsCollector>,
        alert_manager: Arc<AlertManager>,
        config: MonitoringConfig,
        session_id: String,
        metrics: Arc<Mutex<Vec<MetricValue>>>,
        cancel_token: Arc<AtomicBool>,
    ) {
        let mut interval = interval(config.collection_interval);

        while !cancel_token.load(Ordering::SeqCst) {
            interval.tick().await;

            // Collect all configured metrics
            let mut collected_metrics = Vec::new();
            let timestamp = SystemTime::now();

            for metric_name in &config.metrics {
                if let Ok(value) = collector.collect_metric(metric_name).await {
                    let metric_point = MetricValue {
                        value,
                        timestamp,
                        labels: HashMap::new(),
                    };

                    // Check alert thresholds
                    if let Some(threshold) = config.alert_thresholds.get(metric_name) {
                        if value > *threshold {
                            let alert = Alert {
                                id: Uuid::new_v4().to_string(),
                                level: AlertLevel::Warning,
                                metric_name: metric_name.clone(),
                                current_value: value,
                                threshold: *threshold,
                                timestamp,
                                message: format!(
                                    "Metric {} exceeded threshold: {} > {}",
                                    metric_name, value, threshold
                                ),
                            };

                            alert_manager.trigger_alert(alert).await;
                        }
                    }

                    collected_metrics.push(metric_point);
                }
            }

            // Store collected metrics
            {
                let mut metrics_guard = metrics.lock().await;
                metrics_guard.extend(collected_metrics);
            }
        }
    }

    /// Get real-time metrics for a monitoring session
    pub async fn get_real_time_metrics(
        &self,
        session_id: &str,
    ) -> Result<Vec<MetricValue>, ChaosError> {
        let sessions = self.active_sessions.read().await;
        if let Some(session) = sessions.get(session_id) {
            let metrics = session.metrics.lock().await;
            Ok(metrics.clone())
        } else {
            Err(ChaosError::MonitoringError(format!(
                "Monitoring session not found: {}",
                session_id
            )))
        }
    }

    /// Get active alerts for a monitoring session
    pub async fn get_active_alerts(&self, session_id: &str) -> Vec<Alert> {
        self.alert_manager.get_active_alerts().await
    }
}

impl SystemMetricsCollector {
    /// Create a new system metrics collector
    pub fn new() -> Self {
        Self {
            cpu_tracker: Arc::new(CpuUsageTracker::new()),
            memory_tracker: Arc::new(MemoryUsageTracker::new()),
            network_tracker: Arc::new(NetworkMetricsTracker::new()),
            request_tracker: Arc::new(RequestMetricsTracker::new()),
        }
    }

    /// Collect a specific metric by name
    pub async fn collect_metric(&self, metric_name: &str) -> Result<f64, ChaosError> {
        match metric_name {
            "cpu_usage" => Ok(self.cpu_tracker.get_current_usage().await),
            "memory_usage" => Ok(self.memory_tracker.get_current_usage_percent().await),
            "response_time_ms" => Ok(self.network_tracker.get_avg_response_time().await),
            "request_count" => Ok(self.request_tracker.get_total_requests().await as f64),
            "error_count" => Ok(self.request_tracker.get_failed_requests().await as f64),
            "successful_requests" => {
                Ok(self.request_tracker.get_successful_requests().await as f64)
            }
            "total_requests" => Ok(self.request_tracker.get_total_requests().await as f64),
            "error_rate" => Ok(self.request_tracker.get_error_rate().await),
            _ => Err(ChaosError::MonitoringError(format!(
                "Unknown metric: {}",
                metric_name
            ))),
        }
    }
}

impl CpuUsageTracker {
    pub fn new() -> Self {
        Self {
            current_usage: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            usage_history: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn get_current_usage(&self) -> f64 {
        // Simulate CPU usage collection
        // In a real implementation, this would query actual system metrics
        let usage = rand::random::<f64>() * 100.0;
        self.current_usage.store(usage as u64, Ordering::SeqCst);

        // Store in history
        {
            let mut history = self.usage_history.lock().await;
            history.push((SystemTime::now(), usage));

            // Keep only last 1000 entries
            if history.len() > 1000 {
                history.remove(0);
            }
        }

        usage
    }
}

impl MemoryUsageTracker {
    pub fn new() -> Self {
        Self {
            current_usage: Arc::new(AtomicU64::new(0)),
            usage_history: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn get_current_usage_percent(&self) -> f64 {
        // Simulate memory usage collection
        // In a real implementation, this would query actual system memory
        let usage_bytes = (rand::random::<f64>() * 1024.0 * 1024.0 * 1024.0) as u64; // Random GB
        let usage_percent = (usage_bytes as f64 / (8.0 * 1024.0 * 1024.0 * 1024.0)) * 100.0; // Assume 8GB total

        self.current_usage.store(usage_bytes, Ordering::SeqCst);

        // Store in history
        {
            let mut history = self.usage_history.lock().await;
            history.push((SystemTime::now(), usage_bytes));

            // Keep only last 1000 entries
            if history.len() > 1000 {
                history.remove(0);
            }
        }

        usage_percent
    }
}

impl NetworkMetricsTracker {
    pub fn new() -> Self {
        Self {
            request_count: Arc::new(AtomicU64::new(0)),
            response_times: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn get_avg_response_time(&self) -> f64 {
        let response_times = self.response_times.lock().await;
        if response_times.is_empty() {
            // Simulate response time
            rand::random::<f64>() * 500.0 // 0-500ms
        } else {
            let sum: u64 = response_times.iter().map(|(_, time)| *time).sum();
            sum as f64 / response_times.len() as f64
        }
    }

    pub fn record_request(&self, response_time_ms: u64) {
        self.request_count.fetch_add(1, Ordering::SeqCst);

        tokio::spawn({
            let response_times = self.response_times.clone();
            async move {
                let mut times = response_times.lock().await;
                times.push((SystemTime::now(), response_time_ms));

                // Keep only last 1000 entries
                if times.len() > 1000 {
                    times.remove(0);
                }
            }
        });
    }
}

impl RequestMetricsTracker {
    pub fn new() -> Self {
        Self {
            total_requests: Arc::new(AtomicU64::new(0)),
            successful_requests: Arc::new(AtomicU64::new(0)),
            failed_requests: Arc::new(AtomicU64::new(0)),
            response_times: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn get_total_requests(&self) -> u64 {
        self.total_requests.load(Ordering::SeqCst)
    }

    pub async fn get_successful_requests(&self) -> u64 {
        self.successful_requests.load(Ordering::SeqCst)
    }

    pub async fn get_failed_requests(&self) -> u64 {
        self.failed_requests.load(Ordering::SeqCst)
    }

    pub async fn get_error_rate(&self) -> f64 {
        let total = self.total_requests.load(Ordering::SeqCst);
        let failed = self.failed_requests.load(Ordering::SeqCst);

        if total == 0 {
            0.0
        } else {
            failed as f64 / total as f64
        }
    }

    pub fn record_request(&self, success: bool, response_time_ms: u64) {
        self.total_requests.fetch_add(1, Ordering::SeqCst);

        if success {
            self.successful_requests.fetch_add(1, Ordering::SeqCst);
        } else {
            self.failed_requests.fetch_add(1, Ordering::SeqCst);
        }

        tokio::spawn({
            let response_times = self.response_times.clone();
            async move {
                let mut times = response_times.lock().await;
                times.push(response_time_ms);

                // Keep only last 1000 entries
                if times.len() > 1000 {
                    times.remove(0);
                }
            }
        });
    }
}

impl AlertManager {
    pub fn new() -> Self {
        Self {
            active_alerts: Arc::new(RwLock::new(HashMap::new())),
            alert_history: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn trigger_alert(&self, alert: Alert) {
        // Add to active alerts
        {
            let mut active = self.active_alerts.write().await;
            active.insert(alert.id.clone(), alert.clone());
        }

        // Add to history
        {
            let mut history = self.alert_history.lock().await;
            history.push(alert);
        }
    }

    pub async fn get_active_alerts(&self) -> Vec<Alert> {
        let active = self.active_alerts.read().await;
        active.values().cloned().collect()
    }

    pub async fn clear_alert(&self, alert_id: &str) {
        let mut active = self.active_alerts.write().await;
        active.remove(alert_id);
    }
}
