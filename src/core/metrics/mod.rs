//! Metrics system for the Squirrel project
//!
//! This module provides the metrics collection and reporting functionality.
//! It allows tracking various system metrics and exporting them for analysis.

use std::fmt;
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

use crate::core::error::{Error, MetricsError};

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
    Counter(u64),
    /// Gauge value
    Gauge(f64),
    /// Histogram value
    Histogram(Vec<f64>),
    /// Summary value
    Summary {
        /// Sum of all values
        sum: f64,
        /// Count of values
        count: u64,
        /// Quantiles
        quantiles: HashMap<f64, f64>,
    },
}

/// A metric in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    /// The name of the metric
    pub name: String,
    /// The type of metric
    pub metric_type: MetricType,
    /// The timestamp when the metric was recorded
    pub timestamp: DateTime<Utc>,
    /// The value of the metric
    pub value: MetricValue,
    /// Labels associated with the metric
    pub labels: HashMap<String, String>,
}

/// The main metrics collector
#[derive(Debug, Default)]
pub struct Metrics {
    /// The metrics store
    metrics: HashMap<String, Vec<Metric>>,
}

impl Metrics {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self {
            metrics: HashMap::new(),
        }
    }

    /// Initialize the metrics collector
    pub fn initialize(&mut self) -> Result<(), Error> {
        Ok(())
    }

    /// Shutdown the metrics collector
    pub fn shutdown(&mut self) -> Result<(), Error> {
        Ok(())
    }

    /// Record a metric
    pub fn record(&mut self, metric: Metric) -> Result<(), Error> {
        let metrics = self.metrics.entry(metric.name.clone()).or_insert_with(Vec::new);
        metrics.push(metric);
        Ok(())
    }

    /// Get all metrics
    pub fn get_metrics(&self) -> &HashMap<String, Vec<Metric>> {
        &self.metrics
    }

    /// Clear all metrics
    pub fn clear(&mut self) {
        self.metrics.clear();
    }
}

impl Default for Metrics {
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
            MetricValue::Summary { sum, count, quantiles } => {
                write!(
                    f,
                    "Summary(sum={}, count={}, quantiles={})",
                    sum,
                    count,
                    quantiles.len()
                )
            }
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

/// Trait for metrics collectors
pub trait MetricsCollector: Send + Sync {
    /// Record a metric
    fn record(&mut self, metric: Metric) -> Result<(), Error>;
    /// Get all metrics
    fn get_metrics(&self) -> &HashMap<String, Vec<Metric>>;
    /// Clear all metrics
    fn clear(&mut self);
}

/// Trait for metrics exporters
pub trait MetricsExporter: Send + Sync {
    /// Export metrics
    fn export(&self, metrics: &HashMap<String, Vec<Metric>>) -> Result<(), Error>;
} 