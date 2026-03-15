// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Monitoring and observability capability

use crate::error::PrimalError;
// Native async traits (Rust 1.75+) - no async_trait needed!
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A metric to be recorded
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    /// Metric name
    pub name: String,

    /// Metric value
    pub value: f64,

    /// Metric type (counter, gauge, histogram)
    pub metric_type: MetricType,

    /// Labels/tags
    pub labels: HashMap<String, String>,

    /// Timestamp (Unix timestamp in milliseconds)
    pub timestamp: u64,
}

/// Type of metric
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum MetricType {
    Counter,
    Gauge,
    Histogram,
}

/// Capability for recording metrics and monitoring

pub trait MonitoringCapability: Send + Sync {
    /// Record a metric
    async fn record_metric(&self, metric: Metric) -> Result<(), PrimalError>;

    /// Batch record metrics
    async fn record_metrics(&self, metrics: Vec<Metric>) -> Result<(), PrimalError>;

    /// Query metrics (returns JSON)
    async fn query_metrics(&self, query: String) -> Result<serde_json::Value, PrimalError>;
}
