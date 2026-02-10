// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Analysis structures for error, throughput, and latency analysis

use std::collections::HashMap;
use std::time::Duration;
use serde::{Deserialize, Serialize};

/// Error analysis results
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ErrorAnalysis {
    /// Total error count
    pub total_errors: u64,
    
    /// Error rate (0.0 to 1.0)
    pub error_rate: f64,
    
    /// Error breakdown by type
    pub error_breakdown: HashMap<String, ErrorSummary>,
    
    /// Critical errors
    pub critical_errors: Vec<String>,
    
    /// Error trends
    pub error_trends: HashMap<String, f64>,
}

/// Error summary for a specific error type
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ErrorSummary {
    /// Error count
    pub count: u64,
    
    /// Error percentage of total
    pub percentage: f64,
    
    /// Recent trend (positive = increasing)
    pub trend: f64,
    
    /// Severity level
    pub severity: ErrorSeverity,
    
    /// Sample error messages
    pub sample_messages: Vec<String>,
}

/// Error severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum ErrorSeverity {
    #[default]
    Low,
    Medium,
    High,
    Critical,
}

/// Throughput analysis results
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ThroughputAnalysis {
    /// Current throughput (requests/operations per second)
    pub current_throughput: f64,
    
    /// Average throughput over time window
    pub average_throughput: f64,
    
    /// Peak throughput
    pub peak_throughput: f64,
    
    /// Throughput trend
    pub throughput_trend: f64,
    
    /// Bottleneck information
    pub bottlenecks: Vec<BottleneckInfo>,
    
    /// Throughput breakdown by component
    pub component_throughput: HashMap<String, f64>,
}

/// Bottleneck information
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BottleneckInfo {
    /// Component experiencing bottleneck
    pub component: String,
    
    /// Bottleneck type
    pub bottleneck_type: BottleneckType,
    
    /// Severity score (0.0 to 1.0)
    pub severity: f64,
    
    /// Impact on overall throughput
    pub impact: f64,
    
    /// Suggested remediation
    pub remediation: Option<String>,
}

/// Types of bottlenecks
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum BottleneckType {
    #[default]
    CPU,
    Memory,
    Network,
    Disk,
    Database,
    External,
}

/// Latency analysis results
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LatencyAnalysis {
    /// Average latency
    pub average_latency: Duration,
    
    /// Latency percentiles
    pub percentiles: LatencyPercentiles,
    
    /// High latency operations
    pub high_latency_operations: Vec<HighLatencyOperation>,
    
    /// Latency trend
    pub latency_trend: f64,
    
    /// Latency breakdown by operation type
    pub operation_latency: HashMap<String, Duration>,
}

/// Latency percentile measurements
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LatencyPercentiles {
    /// 50th percentile (median)
    pub p50: Duration,
    /// 95th percentile
    pub p95: Duration,
    /// 99th percentile
    pub p99: Duration,
    /// 99.9th percentile
    pub p999: Duration,
}

/// Information about high latency operations
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HighLatencyOperation {
    /// Operation name
    pub operation: String,
    
    /// Measured latency
    pub latency: Duration,
    
    /// Operation context
    pub context: HashMap<String, String>,
    
    /// Frequency of high latency
    pub frequency: f64,
    
    /// Impact severity
    pub severity: ErrorSeverity,
} 