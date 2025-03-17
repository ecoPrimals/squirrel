//! Metrics system for the Squirrel project
//!
//! This module provides the metrics collection and reporting functionality.
//! It allows tracking various system metrics and exporting them for analysis.

use std::fmt;
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use thiserror::Error;
use crate::error::SquirrelError;

/// The type of metric
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MetricType {
    /// Counter metric
    Counter,
    /// Gauge metric
    Gauge,
    /// Histogram metric
    Histogram,
    /// Summary metric
    Summary,
}

/// A metric value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricValue {
    /// Counter value
    Counter(i64),
    /// Gauge value
    Gauge(f64),
    /// Histogram value
    Histogram(Vec<f64>),
    /// Summary value
    Summary(Vec<(f64, f64)>),
}

/// A metric in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    /// The name of the metric
    pub name: String,
    /// The type of metric
    pub metric_type: MetricType,
    /// The timestamp when the metric was recorded
    pub timestamp: i64,
    /// The value of the metric
    pub value: MetricValue,
    /// Labels associated with the metric
    pub labels: HashMap<String, String>,
}

/// Interface for collecting metrics
pub trait MetricsCollector: fmt::Debug + Send + Sync {
    /// Collect metrics from the given metrics source
    ///
    /// # Errors
    ///
    /// Returns an error if the metrics collection fails
    fn collect(&self, metrics: &Metrics) -> Result<()>;
}

/// Interface for exporting metrics
pub trait MetricsExporter: fmt::Debug + Send + Sync {
    /// Export metrics to the target destination
    ///
    /// # Errors
    ///
    /// Returns an error if the metrics export fails
    fn export(&self, metrics: &Metrics) -> Result<()>;
}

/// Data structure for storing metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsData {
    /// Time when metrics were collected
    pub timestamp: DateTime<Utc>,
    /// Collected metric values
    pub metrics: HashMap<String, MetricValue>,
    /// Metric quantiles
    pub quantiles: HashMap<String, f64>,
}

impl MetricsData {
    /// Create a new empty metrics data instance
    #[must_use]
    pub fn new() -> Self {
        Self {
            timestamp: Utc::now(),
            metrics: HashMap::new(),
            quantiles: HashMap::new(),
        }
    }

    /// Merge another metrics data instance into this one
    pub fn merge(&mut self, other: Self) {
        for (key, value) in other.metrics {
            self.metrics.insert(key, value);
        }
        for (key, value) in other.quantiles {
            self.quantiles.insert(key, value);
        }
    }
}

impl Default for MetricsData {
    fn default() -> Self {
        Self::new()
    }
}

/// Snapshot of metrics at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    /// Counter metrics
    pub counters: HashMap<String, u64>,
    /// Gauge metrics
    pub gauges: HashMap<String, f64>,
    /// Histogram metrics
    pub histograms: HashMap<String, Vec<f64>>,
}

/// Metrics collection and management
#[derive(Debug, Clone)]
pub struct Metrics {
    counters: Arc<RwLock<HashMap<String, u64>>>,
    gauges: Arc<RwLock<HashMap<String, f64>>>,
    histograms: Arc<RwLock<HashMap<String, Vec<f64>>>>,
}

impl Default for Metrics {
    fn default() -> Self {
        Self {
            counters: Arc::new(RwLock::new(HashMap::new())),
            gauges: Arc::new(RwLock::new(HashMap::new())),
            histograms: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Metrics {
    /// Create a new metrics instance
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a metric
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The metric type is invalid
    /// - The metric value is incompatible with its type
    /// - A counter metric has a negative value
    pub async fn record(&self, metric: Metric) -> Result<()> {
        match metric.metric_type {
            MetricType::Counter => {
                if let MetricValue::Counter(value) = metric.value {
                    let mut counters = self.counters.write().await;
                    if value >= 0 {
                        *counters.entry(metric.name).or_insert(0) += value.unsigned_abs();
                    } else {
                        return Err(MetricError::InvalidValue(format!("Counter value must be non-negative: {value}")));
                    }
                }
            }
            MetricType::Gauge => {
                if let MetricValue::Gauge(value) = metric.value {
                    let mut gauges = self.gauges.write().await;
                    gauges.insert(metric.name, value);
                }
            }
            MetricType::Histogram => {
                if let MetricValue::Histogram(values) = metric.value {
                    let mut histograms = self.histograms.write().await;
                    histograms.entry(metric.name).or_insert_with(Vec::new).extend(values);
                }
            }
            MetricType::Summary => {
                // Summary metrics are not supported yet
            }
        }
        Ok(())
    }

    /// Get the current value of a counter metric
    ///
    /// # Errors
    ///
    /// Returns an error if the counter metric cannot be accessed
    pub async fn get_counter(&self, name: &str) -> Result<u64> {
        let counters = self.counters.read().await;
        Ok(*counters.get(name).unwrap_or(&0))
    }

    /// Get the current value of a gauge metric
    ///
    /// # Errors
    ///
    /// Returns an error if the gauge metric cannot be accessed
    pub async fn get_gauge(&self, name: &str) -> Result<f64> {
        let gauges = self.gauges.read().await;
        Ok(*gauges.get(name).unwrap_or(&0.0))
    }

    /// Get the current values of a histogram metric
    ///
    /// # Errors
    ///
    /// Returns an error if the histogram metric cannot be accessed
    pub async fn get_histogram(&self, name: &str) -> Result<Vec<f64>> {
        let histograms = self.histograms.read().await;
        Ok(histograms.get(name).cloned().unwrap_or_default())
    }

    /// Create a snapshot of all current metrics
    #[must_use]
    pub async fn snapshot(&self) -> MetricsSnapshot {
        let counters = self.counters.read().await.clone();
        let gauges = self.gauges.read().await.clone();
        let histograms = self.histograms.read().await.clone();
        
        MetricsSnapshot {
            counters,
            gauges,
            histograms,
        }
    }
}

/// Registry for managing multiple metrics instances
#[derive(Debug)]
pub struct MetricsRegistry {
    metrics: Arc<RwLock<HashMap<String, Arc<Metrics>>>>,
}

impl MetricsRegistry {
    /// Create a new metrics registry
    #[must_use]
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a named metrics instance
    ///
    /// # Errors
    ///
    /// Returns an error if the metrics instance cannot be registered
    pub async fn register(&self, name: &str, metrics: Arc<Metrics>) -> Result<()> {
        let mut registry = self.metrics.write().await;
        registry.insert(name.to_string(), metrics);
        Ok(())
    }

    /// Get a named metrics instance if it exists
    ///
    /// # Errors
    ///
    /// Returns an error if the metrics registry cannot be accessed
    pub async fn get(&self, name: &str) -> Result<Option<Arc<Metrics>>> {
        let registry = self.metrics.read().await;
        Ok(registry.get(name).cloned())
    }

    /// Get all registered metrics instances
    ///
    /// # Errors
    ///
    /// Returns an error if the metrics registry cannot be accessed
    pub async fn get_all(&self) -> Result<HashMap<String, Arc<Metrics>>> {
        let registry = self.metrics.read().await;
        Ok(registry.clone())
    }
}

impl Default for MetricsRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for MetricType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Counter => write!(f, "Counter"),
            Self::Gauge => write!(f, "Gauge"),
            Self::Histogram => write!(f, "Histogram"),
            Self::Summary => write!(f, "Summary"),
        }
    }
}

impl fmt::Display for MetricValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Counter(c) => write!(f, "Counter({c})"),
            Self::Gauge(g) => write!(f, "Gauge({g})"),
            Self::Histogram(h) => write!(f, "Histogram({} values)", h.len()),
            Self::Summary(s) => write!(f, "Summary({} pairs)", s.len()),
        }
    }
}

impl fmt::Display for Metric {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Metric(name={}, type={}, timestamp={}, value={}, labels={})",
            self.name,
            self.metric_type,
            self.timestamp,
            self.value,
            self.labels.len()
        )
    }
}

/// Errors that can occur during metric operations
#[derive(Debug, Error)]
pub enum MetricError {
    /// Error when metric type is invalid
    #[error("Invalid metric type: {0}")]
    InvalidType(String),
    /// Error when metric value is invalid
    #[error("Invalid metric value: {0}")]
    InvalidValue(String),
    /// Other metric-related error
    #[error("Metric error: {0}")]
    Other(String),
}

/// Result type for metric operations
type Result<T> = std::result::Result<T, MetricError>;

impl From<MetricError> for SquirrelError {
    fn from(err: MetricError) -> Self {
        match err {
            MetricError::InvalidType(e) => Self::Other(format!("Invalid metric type: {e}")),
            MetricError::InvalidValue(e) => Self::Other(format!("Invalid metric value: {e}")),
            MetricError::Other(e) => Self::Other(e),
        }
    }
}

impl From<String> for MetricError {
    fn from(err: String) -> Self {
        Self::Other(err)
    }
}

impl From<Box<dyn std::error::Error + Send + Sync>> for MetricError {
    fn from(err: Box<dyn std::error::Error + Send + Sync>) -> Self {
        Self::Other(err.to_string())
    }
}

/// Initialize the metrics system
///
/// # Errors
///
/// Returns a `MetricError` if the metrics system cannot be initialized
pub const fn initialize() -> Result<()> {
    // TODO: Initialize metrics system
    Ok(())
}

/// Shutdown the metrics system
///
/// # Errors
///
/// Returns a `MetricError` if the metrics system cannot be shut down properly
pub const fn shutdown() -> Result<()> {
    // TODO: Cleanup metrics system resources
    Ok(())
}