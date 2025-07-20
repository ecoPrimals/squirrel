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
