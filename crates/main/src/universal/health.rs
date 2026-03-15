// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Health and dependency tracking for primals
//!
//! This module defines types for monitoring primal health and managing
//! dependencies between primals in the ecosystem.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::types::PrimalCapability;

/// Primal dependency specification
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PrimalDependency {
    pub primal_type: String,
    pub required_capabilities: Vec<PrimalCapability>,
    pub capabilities: Vec<PrimalCapability>,
    pub required: bool,
    pub optional: bool,
    pub preferred_instance: Option<String>,
    pub min_version: Option<String>,
}

/// Health status of a primal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalHealth {
    pub status: HealthStatus,
    pub healthy: bool,
    pub score: f64,
    pub last_check: DateTime<Utc>,
    pub message: Option<String>,
    pub details: Option<serde_json::Value>,
}

/// Health status enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}
