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
    /// Type of primal this depends on
    pub primal_type: String,
    /// Capabilities that must be provided
    pub required_capabilities: Vec<PrimalCapability>,
    /// Capabilities this dependency provides
    pub capabilities: Vec<PrimalCapability>,
    /// Whether this dependency is required
    pub required: bool,
    /// Whether this dependency is optional
    pub optional: bool,
    /// Preferred instance ID if available
    pub preferred_instance: Option<String>,
    /// Minimum version requirement
    pub min_version: Option<String>,
}

/// Health status of a primal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalHealth {
    /// Overall health status
    pub status: HealthStatus,
    /// Whether the primal is considered healthy
    pub healthy: bool,
    /// Health score from 0.0 to 1.0
    pub score: f64,
    /// Timestamp of last health check
    pub last_check: DateTime<Utc>,
    /// Optional status message
    pub message: Option<String>,
    /// Additional details as JSON
    pub details: Option<serde_json::Value>,
}

/// Health status enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// Fully operational
    Healthy,
    /// Degraded but functional
    Degraded,
    /// Not functioning properly
    Unhealthy,
    /// Status unknown
    Unknown,
}
