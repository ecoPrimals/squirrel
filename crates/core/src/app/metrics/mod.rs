//! Metrics system for the Squirrel project
//!
//! This module provides the metrics collection and reporting functionality.
//! It allows tracking various system metrics and exporting them for analysis.

use std::fmt;
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use crate::core::error::{Error as CoreError, Result as CoreResult};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::future::Future;
use std::pin::Pin;
use std::time::SystemTime;
use thiserror::Error;
use crate::core::error::{SquirrelError, Result, CoreResult};

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

pub trait MetricsCollector: std::fmt::Debug + Send + Sync {
    fn collect(&self, metrics: &Metrics) -> Result<()>;
}

pub trait MetricsExporter: std::fmt::Debug + Send + Sync {
    fn export(&self, metrics: &Metrics) -> Result<()>;
}

/// Metrics data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsData {
    pub timestamp: DateTime<Utc>,
    pub metrics: HashMap<String, MetricValue>,
    pub quantiles: HashMap<String, f64>,
}

impl MetricsData {
    pub fn new() -> Self {
        Self {
            timestamp: Utc::now(),
            metrics: HashMap::new(),
            quantiles: HashMap::new(),
        }
    }

    pub fn merge(&mut self, other: MetricsData) {
        for (key, value) in other.metrics {
            self.metrics.insert(key, value);
        }
        for (key, value) in other.quantiles {
            self.quantiles.insert(key, value);
        }
    }
}

/// A snapshot of metrics at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    pub counters: HashMap<String, u64>,
    pub gauges: HashMap<String, f64>,
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
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn record(&self, metric: Metric) -> CoreResult<()> {
        match metric.metric_type {
            MetricType::Counter => {
                if let MetricValue::Counter(value) = metric.value {
                    let mut counters = self.counters.write().await;
                    *counters.entry(metric.name).or_insert(0) += value as u64;
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

    pub async fn get_counter(&self, name: &str) -> CoreResult<u64> {
        let counters = self.counters.read().await;
        Ok(*counters.get(name).unwrap_or(&0))
    }

    pub async fn get_gauge(&self, name: &str) -> CoreResult<f64> {
        let gauges = self.gauges.read().await;
        Ok(*gauges.get(name).unwrap_or(&0.0))
    }

    pub async fn get_histogram(&self, name: &str) -> CoreResult<Vec<f64>> {
        let histograms = self.histograms.read().await;
        Ok(histograms.get(name).cloned().unwrap_or_default())
    }

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
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register(&self, name: &str, metrics: Arc<Metrics>) -> Result<()> {
        let mut registry = self.metrics.write().await;
        registry.insert(name.to_string(), metrics);
        Ok(())
    }

    pub async fn get(&self, name: &str) -> Result<Option<Arc<Metrics>>> {
        let registry = self.metrics.read().await;
        Ok(registry.get(name).cloned())
    }

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
            MetricType::Counter => write!(f, "Counter"),
            MetricType::Gauge => write!(f, "Gauge"),
            MetricType::Histogram => write!(f, "Histogram"),
            MetricType::Summary => write!(f, "Summary"),
        }
    }
}

impl fmt::Display for MetricValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MetricValue::Counter(c) => write!(f, "Counter({})", c),
            MetricValue::Gauge(g) => write!(f, "Gauge({})", g),
            MetricValue::Histogram(h) => write!(f, "Histogram({} values)", h.len()),
            MetricValue::Summary(s) => write!(f, "Summary({} pairs)", s.len()),
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

#[derive(Error, Debug)]
pub enum MetricsError {
    #[error("Invalid metric name: {0}")]
    InvalidName(String),
    #[error("Metric not found: {0}")]
    NotFound(String),
    #[error("Metric already exists: {0}")]
    AlreadyExists(String),
    #[error("Invalid metric value: {0}")]
    InvalidValue(String),
    #[error("Other error: {0}")]
    Other(String),
}

impl From<MetricsError> for SquirrelError {
    fn from(err: MetricsError) -> Self {
        match err {
            MetricsError::InvalidName(e) => SquirrelError::Other(format!("Invalid metric name: {}", e)),
            MetricsError::NotFound(e) => SquirrelError::Other(format!("Metric not found: {}", e)),
            MetricsError::AlreadyExists(e) => SquirrelError::Other(format!("Metric already exists: {}", e)),
            MetricsError::InvalidValue(e) => SquirrelError::Other(format!("Invalid metric value: {}", e)),
            MetricsError::Other(e) => SquirrelError::Other(e),
        }
    }
}

impl From<String> for MetricsError {
    fn from(err: String) -> Self {
        MetricsError::Other(err)
    }
}

impl From<Box<dyn std::error::Error + Send + Sync>> for MetricsError {
    fn from(err: Box<dyn std::error::Error + Send + Sync>) -> Self {
        MetricsError::Other(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, MetricsError>;

/// Initialize the metrics system
pub async fn initialize() -> Result<()> {
    // TODO: Initialize metrics system
    Ok(())
}

/// Shutdown the metrics system
pub async fn shutdown() -> Result<()> {
    // TODO: Cleanup metrics system resources
    Ok(())
}