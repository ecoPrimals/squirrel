// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Comprehensive metrics for capability-based service discovery and selection
//!
//! This module provides detailed metrics and observability for the performance
//! and behavior of the capability-based ecosystem integration.
//!
//! ## Architecture
//!
//! The capability metrics system is organized into focused modules:
//!
//! - `types`: Core metric structures (DiscoveryMetrics, SelectionMetrics, etc.)
//! - `collector`: Main metrics collection implementation
//! - `scoring`: Health and performance score calculations
//! - `helpers`: Utility functions for bucketing and categorization
//!
//! ## Usage
//!
//! ```no_run
//! use squirrel::metrics::capability_metrics::CapabilityMetrics;
//! use std::time::Duration;
//!
//! # async fn example() {
//! let metrics = CapabilityMetrics::new();
//!
//! // Record discovery operation
//! metrics.record_discovery(
//!     &["authentication".to_string()],
//!     Duration::from_millis(15),
//!     3,
//!     true
//! ).await;
//!
//! // Get comprehensive summary
//! let summary = metrics.get_summary().await;
//! println!("Health score: {}", summary.health_score);
//! # }
//! ```

mod collector;
mod helpers;
mod scoring;
mod types;

// Re-export public API
pub use collector::CapabilityMetrics;
pub use types::{
    CacheMetrics, DiscoveryMetrics, ErrorEvent, ErrorMetrics, MetricsSummary, RoutingMetrics,
    SelectionMetrics,
};

// Re-export scoring functions for advanced users
pub use scoring::{
    calculate_health_score, calculate_performance_score, calculate_reliability_score,
};

// Re-export helper functions for custom bucketing
pub use helpers::{get_score_bucket, get_time_bucket};
