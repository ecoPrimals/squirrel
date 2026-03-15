// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Statistical analysis structures for metrics

use std::collections::HashMap;
use std::time::Duration;
use serde::{Deserialize, Serialize};

/// Statistical summary for numeric values
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StatisticalSummary {
    /// Count of values
    pub count: u64,
    
    /// Sum of all values
    pub sum: f64,
    
    /// Mean value
    pub mean: f64,
    
    /// Median value
    pub median: f64,
    
    /// Standard deviation
    pub std_dev: f64,
    
    /// Minimum value
    pub min: f64,
    
    /// Maximum value
    pub max: f64,
    
    /// Percentiles (25th, 75th, 90th, 95th, 99th)
    pub percentiles: HashMap<u8, f64>,
}

/// Time-based statistical summary
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TimeBasedSummary {
    /// Duration statistics
    pub duration_stats: StatisticalSummary,
    
    /// Timestamp of first sample
    pub first_sample: Option<chrono::DateTime<chrono::Utc>>,
    
    /// Timestamp of last sample
    pub last_sample: Option<chrono::DateTime<chrono::Utc>>,
    
    /// Sample frequency (samples per second)
    pub sample_frequency: f64,
    
    /// Time window covered
    pub window_duration: Duration,
}

/// Resource utilization statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResourceUtilizationStats {
    /// CPU usage statistics
    pub cpu_stats: StatisticalSummary,
    
    /// Memory usage statistics
    pub memory_stats: StatisticalSummary,
    
    /// Network I/O statistics
    pub network_stats: HashMap<String, StatisticalSummary>,
    
    /// Disk I/O statistics
    pub disk_stats: HashMap<String, StatisticalSummary>,
    
    /// Custom resource statistics
    pub custom_stats: HashMap<String, StatisticalSummary>,
}

/// Component performance statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ComponentStats {
    /// Component name
    pub component: String,
    
    /// Request count statistics
    pub request_stats: StatisticalSummary,
    
    /// Response time statistics
    pub response_time_stats: TimeBasedSummary,
    
    /// Error rate statistics
    pub error_rate_stats: StatisticalSummary,
    
    /// Resource utilization
    pub resource_stats: ResourceUtilizationStats,
    
    /// Custom metrics
    pub custom_metrics: HashMap<String, StatisticalSummary>,
} 