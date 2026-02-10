// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Visualization Metrics
//!
//! This module provides metrics collection and reporting for the visualization system.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Visualization metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationMetrics {
    /// Total visualizations created
    pub total_created: u64,

    /// Total visualizations updated
    pub total_updated: u64,

    /// Total visualizations deleted
    pub total_deleted: u64,

    /// Total renderings performed
    pub total_renderings: u64,

    /// Cache hits
    pub cache_hits: u64,

    /// Cache misses
    pub cache_misses: u64,

    /// Average render time in milliseconds
    pub average_render_time_ms: f64,

    /// Last update timestamp
    pub last_updated: DateTime<Utc>,
}

impl Default for VisualizationMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl VisualizationMetrics {
    /// Create new metrics
    pub fn new() -> Self {
        Self {
            total_created: 0,
            total_updated: 0,
            total_deleted: 0,
            total_renderings: 0,
            cache_hits: 0,
            cache_misses: 0,
            average_render_time_ms: 0.0,
            last_updated: Utc::now(),
        }
    }

    /// Update timestamp
    pub fn update_timestamp(&mut self) {
        self.last_updated = Utc::now();
    }

    /// Increment visualizations created
    pub fn increment_visualizations_created(&mut self) {
        self.total_created += 1;
        self.update_timestamp();
    }

    /// Calculate cache hit rate
    pub fn cache_hit_rate(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total == 0 {
            0.0
        } else {
            self.cache_hits as f64 / total as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visualization_metrics_new() {
        let metrics = VisualizationMetrics::new();
        assert_eq!(metrics.total_created, 0);
        assert_eq!(metrics.total_updated, 0);
        assert_eq!(metrics.total_deleted, 0);
        assert_eq!(metrics.total_renderings, 0);
        assert_eq!(metrics.cache_hits, 0);
        assert_eq!(metrics.cache_misses, 0);
        assert_eq!(metrics.average_render_time_ms, 0.0);
    }

    #[test]
    fn test_visualization_metrics_default() {
        let metrics = VisualizationMetrics::default();
        assert_eq!(metrics.total_created, 0);
    }

    #[test]
    fn test_increment_visualizations_created() {
        let mut metrics = VisualizationMetrics::new();
        let ts_before = metrics.last_updated;
        metrics.increment_visualizations_created();
        assert_eq!(metrics.total_created, 1);
        assert!(metrics.last_updated >= ts_before);
        metrics.increment_visualizations_created();
        assert_eq!(metrics.total_created, 2);
    }

    #[test]
    fn test_cache_hit_rate_no_accesses() {
        let metrics = VisualizationMetrics::new();
        assert_eq!(metrics.cache_hit_rate(), 0.0);
    }

    #[test]
    fn test_cache_hit_rate_all_hits() {
        let mut metrics = VisualizationMetrics::new();
        metrics.cache_hits = 10;
        metrics.cache_misses = 0;
        assert_eq!(metrics.cache_hit_rate(), 1.0);
    }

    #[test]
    fn test_cache_hit_rate_all_misses() {
        let mut metrics = VisualizationMetrics::new();
        metrics.cache_hits = 0;
        metrics.cache_misses = 10;
        assert_eq!(metrics.cache_hit_rate(), 0.0);
    }

    #[test]
    fn test_cache_hit_rate_mixed() {
        let mut metrics = VisualizationMetrics::new();
        metrics.cache_hits = 7;
        metrics.cache_misses = 3;
        let rate = metrics.cache_hit_rate();
        assert!((rate - 0.7).abs() < 0.001);
    }

    #[test]
    fn test_update_timestamp() {
        let mut metrics = VisualizationMetrics::new();
        let ts_before = metrics.last_updated;
        std::thread::sleep(std::time::Duration::from_millis(10));
        metrics.update_timestamp();
        assert!(metrics.last_updated >= ts_before);
    }

    #[test]
    fn test_visualization_metrics_serde() {
        let metrics = VisualizationMetrics::new();
        let json = serde_json::to_string(&metrics).unwrap();
        let deserialized: VisualizationMetrics = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.total_created, 0);
        assert_eq!(deserialized.cache_hits, 0);
    }
}
