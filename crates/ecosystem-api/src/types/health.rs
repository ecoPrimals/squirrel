// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Health and resource types for primals.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Health status for all primals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalHealth {
    /// Health status
    pub status: HealthStatus,

    /// Primal version
    pub version: String,

    /// Uptime in seconds
    pub uptime_seconds: u64,

    /// Resource usage
    pub resource_usage: ResourceUsage,

    /// Capabilities currently online
    pub capabilities_online: Vec<String>,

    /// Last health check timestamp
    pub last_check: DateTime<Utc>,
}

/// Health status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HealthStatus {
    /// Service is healthy
    Healthy,
    /// Service is degraded but functional
    Degraded,
    /// Service is unhealthy
    Unhealthy,
    /// Health status is unknown
    Unknown,
}

/// Resource usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    /// CPU usage percentage
    pub cpu_percent: f64,

    /// Memory usage in bytes
    pub memory_bytes: u64,

    /// Disk usage in bytes
    pub disk_bytes: u64,

    /// Network usage in bytes per second
    pub network_bytes_per_sec: u64,
}

/// Primal endpoints information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalEndpoints {
    /// Primary endpoint
    pub primary: String,

    /// Health check endpoint
    pub health: String,

    /// Metrics endpoint
    pub metrics: Option<String>,

    /// Admin endpoint
    pub admin: Option<String>,

    /// WebSocket endpoint
    pub websocket: Option<String>,

    /// Service mesh endpoint
    pub service_mesh: String,
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    /// Health check path
    pub path: String,

    /// Check interval in seconds
    pub interval_seconds: u64,

    /// Timeout in seconds
    pub timeout_seconds: u64,

    /// Number of retries
    pub retries: u32,

    /// Initial delay in seconds
    pub initial_delay_seconds: u64,
}
