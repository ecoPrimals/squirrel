// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Service registration and resource specification types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use super::health::HealthCheckConfig;
use super::primal::PrimalType;
use super::security::SecurityConfig;

/// Ecosystem service registration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemServiceRegistration {
    /// Service identifier (`Arc<str>` for O(1) clone when shared)
    pub service_id: Arc<str>,

    /// Primal type
    pub primal_type: PrimalType,

    /// Associated biome identifier (if applicable)
    pub biome_id: Option<Arc<str>>,

    /// Service capabilities
    pub capabilities: ServiceCapabilities,

    /// API endpoints
    pub endpoints: ServiceEndpoints,

    /// Resource requirements
    pub resource_requirements: ResourceSpec,

    /// Security configuration
    pub security_config: SecurityConfig,

    /// Health check configuration
    pub health_check: HealthCheckConfig,

    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Service capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceCapabilities {
    /// Core capabilities (required)
    pub core: Vec<String>,

    /// Extended capabilities (optional)
    pub extended: Vec<String>,

    /// Cross-primal integrations supported
    pub integrations: Vec<String>,
}

/// Service endpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEndpoints {
    /// Health check endpoint
    pub health: String,

    /// Metrics endpoint
    pub metrics: String,

    /// Admin/management endpoint
    pub admin: String,

    /// WebSocket endpoint (if supported)
    pub websocket: Option<String>,
}

/// Resource specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceSpec {
    /// CPU cores required
    pub cpu_cores: Option<f64>,

    /// Memory in MB required
    pub memory_mb: Option<u64>,

    /// Disk space in MB required
    pub disk_mb: Option<u64>,

    /// Network bandwidth in Mbps required
    pub network_bandwidth_mbps: Option<u64>,

    /// GPU count required
    pub gpu_count: Option<u32>,
}

/// Dynamic port information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicPortInfo {
    /// Port number
    pub port: u16,

    /// Protocol
    pub protocol: String,

    /// Assigned by Songbird
    pub assigned_by: String,

    /// Assignment timestamp
    pub assigned_at: DateTime<Utc>,

    /// Lease duration
    pub lease_duration: std::time::Duration,
}
