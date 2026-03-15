// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Trend analysis for metrics

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Trend direction indicator
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrendDirection {
    /// Upward trend
    Up,
    /// Downward trend
    Down,
    /// Stable/flat trend
    Stable,
    /// Oscillating/volatile trend
    Volatile,
}

impl Default for TrendDirection {
    fn default() -> Self {
        Self::Stable
    }
}

/// Trend analysis results
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TrendAnalysis {
    /// Primary trend direction
    pub primary_direction: TrendDirection,
    
    /// Trend confidence (0.0 to 1.0)
    pub confidence: f64,
    
    /// Rate of change
    pub rate_of_change: f64,
    
    /// Trend duration
    pub duration_minutes: u64,
    
    /// Trend volatility
    pub volatility: f64,
    
    /// Supporting metrics
    pub supporting_metrics: HashMap<String, f64>,
}

/// Component trend analysis
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ComponentTrends {
    /// Component name
    pub component: String,
    
    /// Response time trends
    pub response_time_trend: TrendAnalysis,
    
    /// Error rate trends
    pub error_rate_trend: TrendAnalysis,
    
    /// Throughput trends
    pub throughput_trend: TrendAnalysis,
    
    /// Resource usage trends
    pub resource_trends: HashMap<String, TrendAnalysis>,
} 