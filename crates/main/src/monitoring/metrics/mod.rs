// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! # Metrics Collection Module
//!
//! Comprehensive metrics collection for the Squirrel AI ecosystem.
//!
//! ## Features
//!
//! - **Multi-Type Metrics**: Counters, gauges, histograms, summaries
//! - **System Monitoring**: CPU, memory, disk, network metrics
//! - **Zero-Copy Optimization**: Static string constants for metric names
//! - **Historical Data**: Configurable metric history tracking
//!
//! ## Module Organization
//!
//! - `types` - Metric type definitions and structures
//! - `collector` - Core metrics collection engine
//!
//! ## Usage Example
//!
//! ```ignore
//! use squirrel::monitoring::metrics::MetricsCollector;
//!
//! # async fn example() {
//! let collector = MetricsCollector::new();
//!
//! // Record a metric
//! collector.record("requests_total", 1.0).await;
//!
//! // Collect system metrics
//! collector.collect_system_metrics().await;
//!
//! // Get all metrics
//! let metrics = collector.get_all_metrics().await;
//! # }
//! ```

mod collector;
mod types;

#[cfg(test)]
mod collector_tests;

// Re-export public types
pub use collector::MetricsCollector;
pub use types::{AllMetrics, MetricInfo, MetricSnapshot, MetricValue, SystemMetrics};
