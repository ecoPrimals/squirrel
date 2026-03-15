// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Resource manager type definitions
//!
//! Core types for resource management configuration and statistics.

use std::time::{Duration, Instant};

/// Resource management configuration
#[derive(Debug, Clone)]
pub struct ResourceManagerConfig {
    /// Interval for connection pool cleanup
    pub connection_cleanup_interval: Duration,

    /// Interval for memory cleanup
    pub memory_cleanup_interval: Duration,

    /// Interval for health monitoring
    pub health_check_interval: Duration,

    /// Maximum memory usage before cleanup (bytes)
    pub max_memory_threshold: u64,

    /// Enable automatic resource cleanup
    pub enable_auto_cleanup: bool,

    /// Resource cleanup timeout
    pub cleanup_timeout: Duration,
}

impl Default for ResourceManagerConfig {
    fn default() -> Self {
        Self {
            connection_cleanup_interval: Duration::from_secs(300), // 5 minutes
            memory_cleanup_interval: Duration::from_secs(600),     // 10 minutes
            health_check_interval: Duration::from_secs(60),        // 1 minute
            max_memory_threshold: 500 * 1024 * 1024,               // 500MB
            enable_auto_cleanup: true,
            cleanup_timeout: Duration::from_secs(30),
        }
    }
}

/// Resource usage statistics
#[derive(Debug, Clone, Default)]
pub struct ResourceUsageStats {
    /// Total memory allocated (estimated)
    pub memory_bytes: u64,

    /// Number of active connections
    pub active_connections: usize,

    /// Number of open file handles (estimated)
    pub file_handles: usize,

    /// Background tasks running
    pub background_tasks: usize,

    /// Last cleanup timestamp
    pub last_cleanup: Option<Instant>,

    /// Cleanup success rate
    pub cleanup_success_rate: f64,
}

/// Cleanup operation metrics
#[derive(Debug, Clone, Default)]
pub struct CleanupMetrics {
    pub total_runs: u64,
    pub successful_runs: u64,
    pub failed_runs: u64,
    pub avg_duration_ms: f64,
    pub resources_cleaned: u64,
    pub last_run: Option<Instant>,
}
