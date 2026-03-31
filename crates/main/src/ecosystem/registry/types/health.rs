// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Health-related types for discovered services and checks.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Service health status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ServiceHealthStatus {
    /// Health status not yet determined
    Unknown,
    /// Service is operating normally
    Healthy,
    /// Service is degraded but functional
    Degraded,
    /// Service is unhealthy
    Unhealthy,
    /// Service is offline
    Offline,
}

/// Health check result
#[derive(Debug, Clone)]
pub struct HealthCheckResult {
    /// Health status determined by the check
    pub status: ServiceHealthStatus,
    /// Time taken to perform the check
    pub processing_time: Duration,
    /// Error message if check failed
    pub error: Option<String>,
}
