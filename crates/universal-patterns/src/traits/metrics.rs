//! Metric value types.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

/// Metric value types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricValue {
    /// Integer counter
    Counter(u64),
    /// Floating point gauge
    Gauge(f64),
    /// Histogram data
    Histogram {
        /// Number of observations
        count: u64,
        /// Sum of all observations
        sum: f64,
        /// Histogram buckets with (upper_bound, count) pairs
        buckets: Vec<(f64, u64)>,
    },
    /// String value
    String(String),
    /// Boolean value
    Boolean(bool),
    /// Duration value
    Duration(Duration),
    /// Timestamp value
    Timestamp(DateTime<Utc>),
}
