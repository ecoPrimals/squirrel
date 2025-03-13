//! Metrics and measurements for analysis.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a metric calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    /// Name of the metric
    pub name: String,
    /// Value of the metric
    pub value: f64,
    /// Unit of measurement
    pub unit: Option<String>,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Represents a set of metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricSet {
    /// Unique identifier for the metric set
    pub id: String,
    /// Name of the metric set
    pub name: String,
    /// Collection of metrics
    pub metrics: HashMap<String, Metric>,
    /// Timestamp of the metric set
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl MetricSet {
    /// Creates a new metric set
    pub fn new(name: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            metrics: HashMap::new(),
            timestamp: chrono::Utc::now(),
        }
    }

    /// Adds a metric to the set
    pub fn add_metric(&mut self, metric: Metric) {
        self.metrics.insert(metric.name.clone(), metric);
    }
} 