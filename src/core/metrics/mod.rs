//! Metrics system for the Squirrel project
//!
//! This module provides the metrics collection and reporting functionality.
//! It allows tracking various system metrics and exporting them for analysis.

use std::fmt;
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use crate::core::error::{Result, SquirrelError};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::core::error::Error;
use std::future::Future;
use std::pin::Pin;

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

pub trait MetricsCollector: Send + Sync {
    fn as_async(&self) -> &dyn MetricsCollectorAsync;
}

pub trait MetricsCollectorAsync: Send + Sync {
    /// Collect metrics
    fn collect<'a>(&'a self) -> Pin<Box<dyn Future<Output = std::result::Result<MetricsData, MetricsError>> + Send + 'a>>;
}

pub trait MetricsExporter: Send + Sync {
    fn as_async(&self) -> &dyn MetricsExporterAsync;
}

pub trait MetricsExporterAsync: Send + Sync {
    /// Export metrics
    fn export<'a>(&'a self, metrics: &'a MetricsData) -> Pin<Box<dyn Future<Output = std::result::Result<(), MetricsError>> + Send + 'a>>;
}

/// Metrics data
#[derive(Debug, Clone)]
pub struct MetricsData {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub metrics: serde_json::Value,
}

/// Metrics error types
#[derive(Debug, thiserror::Error)]
pub enum MetricsError {
    #[error("Collection error: {0}")]
    Collection(String),
    
    #[error("Export error: {0}")]
    Export(String),
    
    #[error("Invalid metrics data: {0}")]
    InvalidData(String),
}

/// Metrics system
pub struct Metrics {
    collectors: Arc<RwLock<Vec<Arc<dyn MetricsCollector>>>>,
    exporters: Arc<RwLock<Vec<Arc<dyn MetricsExporter>>>>,
    initialized: Arc<RwLock<bool>>,
}

impl Metrics {
    /// Create a new metrics system
    pub fn new() -> Self {
        Self {
            collectors: Arc::new(RwLock::new(Vec::new())),
            exporters: Arc::new(RwLock::new(Vec::new())),
            initialized: Arc::new(RwLock::new(false)),
        }
    }
    
    /// Register a metrics collector
    pub async fn register_collector(&self, collector: Arc<dyn MetricsCollector>) {
        self.collectors.write().await.push(collector);
    }
    
    /// Register a metrics exporter
    pub async fn register_exporter(&self, exporter: Arc<dyn MetricsExporter>) {
        self.exporters.write().await.push(exporter);
    }
    
    /// Collect and export metrics
    pub async fn collect_and_export(&self) -> std::result::Result<(), MetricsError> {
        // Collect metrics from all collectors
        let mut all_metrics = Vec::new();
        for collector in self.collectors.read().await.iter() {
            let metrics = collector.as_async().collect().await?;
            all_metrics.push(metrics);
        }
        
        // Export metrics through all exporters
        for metrics in all_metrics {
            for exporter in self.exporters.read().await.iter() {
                exporter.as_async().export(&metrics).await?;
            }
        }
        
        Ok(())
    }

    pub async fn initialize(&self) -> Result<(), MetricsError> {
        let mut initialized = self.initialized.write().await;
        if !*initialized {
            *initialized = true;
        }
        Ok(())
    }

    pub async fn shutdown(&self) -> Result<(), MetricsError> {
        let mut initialized = self.initialized.write().await;
        if *initialized {
            *initialized = false;
        }
        Ok(())
    }

    pub async fn collect_metrics(&self) -> Result<MetricsData, MetricsError> {
        let collectors = self.collectors.read().await;
        let mut metrics_data = MetricsData::new();

        for collector in collectors.iter() {
            let collector_data = collector.as_async().collect().await?;
            metrics_data.merge(collector_data);
        }

        Ok(metrics_data)
    }

    pub async fn export_metrics(&self, metrics: &MetricsData) -> Result<(), MetricsError> {
        let exporters = self.exporters.read().await;
        for exporter in exporters.iter() {
            exporter.as_async().export(metrics).await?;
        }
        Ok(())
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Initialize the metrics system
pub async fn initialize() -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // TODO: Initialize metrics system
    Ok(())
}

/// Shutdown the metrics system
pub async fn shutdown() -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // TODO: Cleanup metrics system resources
    Ok(())
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