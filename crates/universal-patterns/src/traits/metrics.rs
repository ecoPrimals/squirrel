// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metric_value_counter_serde() {
        let val = MetricValue::Counter(42);
        let json = serde_json::to_string(&val).expect("should succeed");
        let deserialized: MetricValue = serde_json::from_str(&json).expect("should succeed");
        if let MetricValue::Counter(v) = deserialized {
            assert_eq!(v, 42);
        } else {
            unreachable!("Expected Counter variant");
        }
    }

    #[test]
    fn test_metric_value_gauge_serde() {
        let val = MetricValue::Gauge(std::f64::consts::PI);
        let json = serde_json::to_string(&val).expect("should succeed");
        let deserialized: MetricValue = serde_json::from_str(&json).expect("should succeed");
        if let MetricValue::Gauge(v) = deserialized {
            assert!((v - std::f64::consts::PI).abs() < f64::EPSILON);
        } else {
            unreachable!("Expected Gauge variant");
        }
    }

    #[test]
    fn test_metric_value_histogram_serde() {
        let val = MetricValue::Histogram {
            count: 100,
            sum: 500.0,
            buckets: vec![(1.0, 10), (5.0, 50), (10.0, 90)],
        };
        let json = serde_json::to_string(&val).expect("should succeed");
        let deserialized: MetricValue = serde_json::from_str(&json).expect("should succeed");
        if let MetricValue::Histogram {
            count,
            sum,
            buckets,
        } = deserialized
        {
            assert_eq!(count, 100);
            assert!((sum - 500.0).abs() < f64::EPSILON);
            assert_eq!(buckets.len(), 3);
        } else {
            unreachable!("Expected Histogram variant");
        }
    }

    #[test]
    fn test_metric_value_string_serde() {
        let val = MetricValue::String("hello".to_string());
        let json = serde_json::to_string(&val).expect("should succeed");
        let deserialized: MetricValue = serde_json::from_str(&json).expect("should succeed");
        if let MetricValue::String(v) = deserialized {
            assert_eq!(v, "hello");
        } else {
            unreachable!("Expected String variant");
        }
    }

    #[test]
    fn test_metric_value_boolean_serde() {
        let val = MetricValue::Boolean(true);
        let json = serde_json::to_string(&val).expect("should succeed");
        let deserialized: MetricValue = serde_json::from_str(&json).expect("should succeed");
        if let MetricValue::Boolean(v) = deserialized {
            assert!(v);
        } else {
            unreachable!("Expected Boolean variant");
        }
    }

    #[test]
    fn test_metric_value_duration_serde() {
        let val = MetricValue::Duration(Duration::seconds(60));
        let json = serde_json::to_string(&val).expect("should succeed");
        let deserialized: MetricValue = serde_json::from_str(&json).expect("should succeed");
        if let MetricValue::Duration(v) = deserialized {
            assert_eq!(v.num_seconds(), 60);
        } else {
            unreachable!("Expected Duration variant");
        }
    }

    #[test]
    fn test_metric_value_timestamp_serde() {
        let now = Utc::now();
        let val = MetricValue::Timestamp(now);
        let json = serde_json::to_string(&val).expect("should succeed");
        let deserialized: MetricValue = serde_json::from_str(&json).expect("should succeed");
        if let MetricValue::Timestamp(v) = deserialized {
            assert_eq!(v.timestamp(), now.timestamp());
        } else {
            unreachable!("Expected Timestamp variant");
        }
    }
}
