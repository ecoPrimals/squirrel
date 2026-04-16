// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Runtime state for [`super::ecosystem_service::EcosystemService`]: lifecycle status and coordination counters.

use chrono::{DateTime, Utc};
use std::sync::RwLock;

/// Converts counters to `f64` for monitoring; loss beyond `f64` mantissa is acceptable for KPIs.
#[expect(
    clippy::cast_precision_loss,
    reason = "Dashboard metrics: u64 counters may exceed f64 precision"
)]
pub const fn metric_u64_as_f64(x: u64) -> f64 {
    x as f64
}

/// Lifecycle and coordination mode of the ecosystem service.
#[derive(Debug, Clone)]
pub enum ServiceStatus {
    /// Service is initializing and not yet steady-state.
    Starting,
    /// Operating without ecosystem coordination.
    Standalone,
    /// Discovering peers or the registry before coordinating.
    Discovering,
    /// Actively coordinating tasks and health with the ecosystem.
    Coordinating,
    /// Partial coordination failures; still operational with reduced guarantees.
    Degraded,
    /// Shutting down and draining background work.
    Stopping,
}

#[derive(Debug)]
#[expect(dead_code, reason = "internal state — fields used via RwLock access")]
pub struct EcosystemState {
    pub service_id: String,
    pub node_id: String,
    pub status: RwLock<ServiceStatus>,
    pub registration_time: DateTime<Utc>,
    pub last_health_check: RwLock<DateTime<Utc>>,
    pub coordination_stats: RwLock<CoordinationStats>,
}

#[derive(Debug, Default)]
pub struct CoordinationStats {
    pub tasks_coordinated: u64,
    pub primals_discovered: u32,
    pub federation_nodes: u32,
    pub last_coordination: Option<DateTime<Utc>>,
    pub coordination_failures: u64,
}
