// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Metrics type definitions
//!
//! Core types for metrics collection and monitoring.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::monitoring::MetricType;

/// Internal metric definition
#[derive(Debug, Clone)]
pub(crate) struct MetricDefinition {
    pub name: String,
    pub metric_type: MetricType,
    pub description: String,
    pub labels: Vec<String>,
    pub unit: String,
    pub source: String,
}

/// Metric value with labels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricValue {
    /// The metric value
    pub value: f64,
    /// Label key-value pairs for dimensional metrics
    pub labels: HashMap<String, String>,
    /// When the value was recorded
    pub timestamp: DateTime<Utc>,
    /// Type of metric (counter, gauge, etc.)
    pub metric_type: MetricType,
}

/// System-wide metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    /// CPU usage percentage
    pub cpu_usage: f64,
    /// Memory usage in bytes
    pub memory_usage: u64,
    /// Memory usage percentage
    pub memory_percentage: f64,
    /// Disk usage percentage
    pub disk_usage: f64,
    /// Network bytes sent per second
    pub network_bytes_sent: f64,
    /// Network bytes received per second
    pub network_bytes_received: f64,
    /// Number of active connections
    pub active_connections: u32,
    /// Request rate (requests per second)
    pub request_rate: f64,
    /// Error rate (errors per second)
    pub error_rate: f64,
    /// Average response time in milliseconds
    pub avg_response_time: f64,
    /// System uptime in seconds
    pub uptime: u64,
}

/// Metric snapshot for history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricSnapshot {
    /// When the snapshot was taken
    pub timestamp: DateTime<Utc>,
    /// All metric values at snapshot time
    pub metrics: HashMap<String, MetricValue>,
    /// System-wide metrics at snapshot time
    pub system_metrics: SystemMetrics,
}

/// All metrics data for export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllMetrics {
    /// All metric values by name
    pub metrics: HashMap<String, MetricValue>,
    /// Per-component metrics
    pub component_metrics: HashMap<String, HashMap<String, f64>>,
    /// System-wide metrics
    pub system_metrics: SystemMetrics,
}

/// Metric information for metadata retrieval
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricInfo {
    /// Metric name
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// Label names for this metric
    pub labels: Vec<String>,
    /// Unit of measurement
    pub unit: String,
    /// Collection source
    pub source: String,
    /// Metric type
    pub metric_type: MetricType,
}
