//! Metrics collection for the MCP monitoring system
//!
//! This module provides metrics collection functionality for the MCP system.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;
use tracing::{error, warn};

/// Metric type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MetricType {
    /// Counter metric (only increases)
    Counter,
    /// Gauge metric (can go up and down)
    Gauge,
    /// Histogram metric (distribution of values)
    Histogram,
    /// Summary metric (percentiles)
    Summary,
}

/// Metric value types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricValue {
    /// Integer value
    Integer(i64),
    /// Float value
    Float(f64),
    /// Boolean value
    Boolean(bool),
    /// String value
    String(String),
    /// Histogram values
    Histogram(Vec<f64>),
    /// Summary values with percentiles
    Summary(HashMap<String, f64>),
}

impl MetricValue {
    /// Increment a metric value
    pub fn increment(&mut self) {
        match self {
            Self::Integer(val) => *val += 1,
            Self::Float(val) => *val += 1.0,
            _ => warn!("Cannot increment non-numeric metric value"),
        }
    }

    /// Decrement a metric value
    pub fn decrement(&mut self) {
        match self {
            Self::Integer(val) => *val -= 1,
            Self::Float(val) => *val -= 1.0,
            _ => warn!("Cannot decrement non-numeric metric value"),
        }
    }

    /// Add to a metric value
    pub fn add(&mut self, value: f64) {
        match self {
            Self::Integer(val) => *val += value as i64,
            Self::Float(val) => *val += value,
            Self::Histogram(vals) => vals.push(value),
            Self::Summary(vals) => {
                // We need to avoid multiple mutable borrows
                // First, get the current sum and count
                let current_sum = vals.get("sum").copied().unwrap_or(0.0);
                let current_count = vals.get("count").copied().unwrap_or(0.0);
                
                // Calculate new values
                let new_sum = current_sum + value;
                let new_count = current_count + 1.0;
                let new_mean = if new_count > 0.0 { new_sum / new_count } else { 0.0 };
                
                // Update the values
                vals.insert("sum".to_string(), new_sum);
                vals.insert("count".to_string(), new_count);
                vals.insert("mean".to_string(), new_mean);
            }
            _ => warn!("Cannot add to non-numeric metric value"),
        }
    }

    /// Set a metric value
    pub fn set(&mut self, value: Self) {
        *self = value;
    }
}

/// Metric definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    /// Metric name
    pub name: String,
    /// Metric description
    pub description: String,
    /// Metric type
    pub metric_type: MetricType,
    /// Metric labels
    pub labels: HashMap<String, String>,
    /// Metric value
    pub value: MetricValue,
    /// Timestamp of the last update
    pub last_updated: DateTime<Utc>,
}

impl Metric {
    /// Create a new metric
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        metric_type: MetricType,
        value: MetricValue,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            metric_type,
            labels: HashMap::new(),
            value,
            last_updated: Utc::now(),
        }
    }

    /// Adds a label to the metric.
    #[must_use]
    pub fn with_label(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.labels.insert(key.into(), value.into());
        self
    }

    /// Add multiple labels to the metric
    #[must_use] pub fn with_labels(mut self, labels: HashMap<String, String>) -> Self {
        self.labels.extend(labels);
        self
    }

    /// Update the metric value
    pub fn update(&mut self, value: MetricValue) {
        self.value = value;
        self.last_updated = Utc::now();
    }
}

/// Metric history entry
type MetricHistoryEntry = (DateTime<Utc>, MetricValue);

/// Metric history collection
type MetricHistory = HashMap<String, Vec<MetricHistoryEntry>>;

/// Metrics collector for the MCP system
#[derive(Debug)]
pub struct MetricsCollector {
    /// Metrics by name
    metrics: RwLock<HashMap<String, Metric>>,
    /// Metrics history
    history: RwLock<MetricHistory>,
    /// Maximum history length
    max_history_length: usize,
}

impl MetricsCollector {
    /// Create a new metrics collector
    #[must_use] pub fn new() -> Self {
        Self {
            metrics: RwLock::new(HashMap::new()),
            history: RwLock::new(HashMap::new()),
            max_history_length: 100, // Default to 100 data points per metric
        }
    }

    /// Set the maximum history length
    pub fn set_max_history_length(&mut self, length: usize) {
        self.max_history_length = length;
    }

    /// Register a new metric
    pub fn register_metric(&self, metric: Metric) {
        match self.metrics.write() {
            Ok(mut metrics) => {
                metrics.insert(metric.name.clone(), metric);
            }
            Err(e) => error!("Failed to acquire metrics write lock for registration: {}", e),
        }
    }

    /// Get a metric by name
    pub fn get_metric(&self, name: &str) -> Option<Metric> {
        match self.metrics.read() {
            Ok(metrics) => metrics.get(name).cloned(),
            Err(e) => {
                error!("Failed to acquire metrics read lock for get_metric: {}", e);
                None
            }
        }
    }

    /// Get all metrics
    pub fn get_all_metrics(&self) -> HashMap<String, Metric> {
        match self.metrics.read() {
            Ok(metrics) => metrics.clone(),
            Err(e) => {
                error!("Failed to acquire metrics read lock for get_all_metrics: {}", e);
                HashMap::new()
            }
        }
    }

    /// Update a metric value
    pub fn update_metric(&self, name: &str, value: MetricValue) {
        let mut metrics = match self.metrics.write() {
            Ok(guard) => guard,
            Err(e) => {
                error!("Failed to acquire metrics write lock for update_metric: {}", e);
                return;
            }
        };

        if let Some(metric) = metrics.get_mut(name) {
            // Update the metric value
            metric.value = value.clone();
            metric.last_updated = Utc::now();

            // Update history
            match self.history.write() {
                Ok(mut history) => {
                    let metric_history = history.entry(name.to_string()).or_default();
                    metric_history.push((Utc::now(), value));
                    if metric_history.len() > self.max_history_length {
                        let excess = metric_history.len() - self.max_history_length;
                        metric_history.drain(0..excess);
                    }
                }
                Err(e) => error!("Failed to acquire history write lock for update_metric: {}", e),
            }
        } else {
            warn!("Attempted to update non-existent metric: {}", name);
        }
    }

    /// Increment a counter metric
    pub fn increment_counter(&self, name: &str) {
        let mut metrics = match self.metrics.write() {
            Ok(guard) => guard,
            Err(e) => {
                error!("Failed to acquire metrics write lock for increment_counter: {}", e);
                return;
            }
        };

        if let Some(metric) = metrics.get_mut(name) {
            let updated = match &mut metric.value {
                MetricValue::Integer(val) => { *val += 1; true },
                MetricValue::Float(val) => { *val += 1.0; true },
                _ => {
                    warn!("Attempted to increment non-counter metric: {}", name);
                    false
                }
            };

            if updated {
                metric.last_updated = Utc::now();
                // Update history
                let value = metric.value.clone();
                match self.history.write() {
                    Ok(mut history) => {
                        let metric_history = history.entry(name.to_string()).or_default();
                        metric_history.push((Utc::now(), value));
                        if metric_history.len() > self.max_history_length {
                            let excess = metric_history.len() - self.max_history_length;
                            metric_history.drain(0..excess);
                        }
                    }
                    Err(e) => error!("Failed to acquire history write lock for increment_counter: {}", e),
                }
            }
        } else {
            warn!("Attempted to increment non-existent metric: {}", name);
        }
    }

    /// Add a value to a histogram metric
    pub fn observe_histogram(&self, name: &str, value: f64) {
        let mut metrics = match self.metrics.write() {
            Ok(guard) => guard,
            Err(e) => {
                error!("Failed to acquire metrics write lock for observe_histogram: {}", e);
                return;
            }
        };

        if let Some(metric) = metrics.get_mut(name) {
            if let MetricValue::Histogram(values) = &mut metric.value {
                values.push(value);
                metric.last_updated = Utc::now();

                // Update history
                let value_clone = metric.value.clone();
                match self.history.write() {
                    Ok(mut history) => {
                        let metric_history = history.entry(name.to_string()).or_default();
                        metric_history.push((Utc::now(), value_clone));
                        if metric_history.len() > self.max_history_length {
                            let excess = metric_history.len() - self.max_history_length;
                            metric_history.drain(0..excess);
                        }
                    }
                    Err(e) => error!("Failed to acquire history write lock for observe_histogram: {}", e),
                }
            } else {
                warn!("Attempted to observe non-histogram metric: {}", name);
            }
        } else {
            warn!("Attempted to observe non-existent metric: {}", name);
        }
    }

    /// Get metric history
    pub fn get_metric_history(&self, name: &str) -> Option<Vec<(DateTime<Utc>, MetricValue)>> {
        match self.history.read() {
            Ok(history) => history.get(name).cloned(),
            Err(e) => {
                error!("Failed to acquire history read lock for get_metric_history: {}", e);
                None
            }
        }
    }

    /// Create a performance snapshot
    pub fn create_performance_snapshot(&self) -> PerformanceSnapshot {
        let metrics_guard = match self.metrics.read() {
            Ok(guard) => guard,
            Err(e) => {
                error!("Failed to acquire metrics read lock for snapshot: {}", e);
                // Return a default snapshot if lock fails
                return PerformanceSnapshot {
                    timestamp: Utc::now(),
                    message_latency_ms: 0.0,
                    command_execution_time_ms: 0.0,
                    memory_usage_bytes: 0u64,
                    error_rate: 0.0,
                    message_throughput: 0.0,
                };
            }
        };

        let get_metric_value = |name: &str| -> Option<MetricValue> {
            metrics_guard.get(name).map(|m| m.value.clone())
        };

        let get_float = |name: &str| -> f64 {
            get_metric_value(name).and_then(|v| match v {
                MetricValue::Float(f) => Some(f),
                _ => None,
            }).unwrap_or(0.0)
        };

        let _get_int = |name: &str| -> i64 {
            get_metric_value(name).and_then(|v| match v {
                MetricValue::Integer(i) => Some(i),
                _ => None,
            }).unwrap_or(0)
        };

        // Placeholder for memory usage collection
        // TODO: Implement correct memory usage calculation
        let memory_usage_bytes = 0u64;

        // Placeholder for CPU usage collection
        // TODO: Implement correct CPU usage calculation
        let _cpu_usage_percent = 0.0;

        PerformanceSnapshot {
            timestamp: Utc::now(),
            message_latency_ms: get_float("message_latency_ms"),
            command_execution_time_ms: get_float("command_execution_time_ms"),
            memory_usage_bytes,
            error_rate: get_float("error_rate"),
            message_throughput: get_float("message_throughput"),
        }
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Performance snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSnapshot {
    /// Timestamp of the snapshot
    pub timestamp: DateTime<Utc>,
    /// Message latency in milliseconds
    pub message_latency_ms: f64,
    /// Command execution time in milliseconds
    pub command_execution_time_ms: f64,
    /// Memory usage in bytes
    pub memory_usage_bytes: u64,
    /// Error rate
    pub error_rate: f64,
    /// Message throughput
    pub message_throughput: f64,
}
