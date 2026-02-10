// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Metrics and observability for capability-based operations
//!
//! This module provides comprehensive metrics collection and analysis for
//! capability-based service discovery, selection, and coordination.

pub mod capability_metrics;

pub use capability_metrics::{
    CacheMetrics, CapabilityMetrics, DiscoveryMetrics, ErrorEvent, ErrorMetrics, MetricsSummary,
    RoutingMetrics, SelectionMetrics,
};
