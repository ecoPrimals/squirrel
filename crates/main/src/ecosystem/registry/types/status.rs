// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Aggregated ecosystem and primal status snapshots used for reporting and dashboards.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::ecosystem::EcosystemPrimalType;

/// Ecosystem status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemStatus {
    /// Overall health score (0.0 to 1.0)
    pub overall_health: f64,
    /// Status of each primal
    pub primal_statuses: Vec<PrimalStatus>,
    /// Number of registered services
    pub registered_services: usize,
    /// Number of active coordinations
    pub active_coordinations: usize,
    /// Timestamp of last full sync
    pub last_full_sync: Option<DateTime<Utc>>,
    /// Size of discovery cache
    pub discovery_cache_size: usize,
}

/// Primal status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalStatus {
    /// Type of primal
    pub primal_type: EcosystemPrimalType,
    /// Current service status
    pub status: ServiceStatus,
    /// Service endpoint URL
    pub endpoint: String,
    /// Service version
    pub version: String,
    /// Capability identifiers
    pub capabilities: Vec<String>,
    /// Health score (0.0 to 1.0)
    pub health_score: f64,
    /// Average response time
    pub response_time: Duration,
    /// When the primal was last seen
    pub last_seen: DateTime<Utc>,
    /// Number of recent errors
    pub error_count: u32,
    /// Coordination features supported
    pub coordination_features: Vec<String>,
}

/// Service status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ServiceStatus {
    /// Status not yet determined
    Unknown,
    /// Currently discovering services
    Discovering,
    /// Currently registering
    Registering,
    /// Operating normally
    Healthy,
    /// Degraded but functional
    Degraded,
    /// Unhealthy
    Unhealthy,
    /// Offline
    Offline,
    /// Recovering from failure
    Recovering,
}
