//! Response types for legacy API endpoints
//!
//! This module contains all response type definitions used by the legacy API server.
//! These types follow ecosystem standards and maintain backward compatibility.
//!
//! # Zero-Copy Performance
//!
//! Uses `Arc<str>` for frequently-cloned strings to eliminate allocations during
//! API responses and service discovery.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::optimization::zero_copy::ArcStr;

/// Health check response following ecosystem standards
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    /// Status string
    pub status: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Uptime in seconds
    pub uptime_seconds: u64,
    /// Service mesh status
    pub service_mesh: ServiceMeshHealthStatus,
    /// Ecosystem integration status
    pub ecosystem: EcosystemHealthStatus,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Service mesh health status
#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceMeshHealthStatus {
    /// Registered with Songbird
    pub registered: bool,
    /// Last heartbeat
    pub last_heartbeat: Option<DateTime<Utc>>,
    /// Connection status
    pub connection_status: String,
    /// Load balancing active
    pub load_balancing_active: bool,
}

/// Ecosystem integration health status
#[derive(Debug, Serialize, Deserialize)]
pub struct EcosystemHealthStatus {
    /// Discovered primals
    pub discovered_primals: u32,
    /// Active integrations
    pub active_integrations: Vec<String>,
    /// Cross-primal communication status
    pub cross_primal_status: String,
    /// Ecosystem health score
    pub ecosystem_health_score: f64,
}

/// Ecosystem status response
#[derive(Debug, Serialize, Deserialize)]
pub struct EcosystemStatusResponse {
    /// Overall status
    pub status: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Active primals
    pub active_primals: Vec<String>,
    /// Service discovery status
    pub service_discovery: String,
    /// Service mesh status
    pub service_mesh_status: ServiceMeshStatusResponse,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Service mesh status response
#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceMeshStatusResponse {
    /// Enabled status
    pub enabled: bool,
    /// Registered status
    pub registered: bool,
    /// Load balancing info
    pub load_balancing: LoadBalancingResponse,
    /// Cross-primal communication info
    pub cross_primal_communication: CrossPrimalCommunicationResponse,
}

/// Load balancing response
#[derive(Debug, Serialize, Deserialize)]
pub struct LoadBalancingResponse {
    /// Enabled status
    pub enabled: bool,
    /// Algorithm used
    pub algorithm: String,
    /// Health score
    pub health_score: f64,
}

/// Cross-primal communication response
#[derive(Debug, Serialize, Deserialize)]
pub struct CrossPrimalCommunicationResponse {
    /// Enabled status
    pub enabled: bool,
    /// Active connections count
    pub active_connections: u32,
    /// Supported protocols
    pub supported_protocols: Vec<String>,
}

/// Primal status response
#[derive(Debug, Serialize, Deserialize)]
pub struct PrimalStatusResponse {
    /// Primal name
    pub name: String,
    /// Status
    pub status: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Endpoints
    pub endpoints: Vec<String>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Metrics response
#[derive(Debug, Serialize, Deserialize)]
pub struct MetricsResponse {
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// System metrics
    pub system: HashMap<String, String>,
    /// Application metrics
    pub application: HashMap<String, String>,
    /// Performance metrics
    pub performance: HashMap<String, String>,
    /// Ecosystem metrics
    pub ecosystem: HashMap<String, String>,
}

/// Services response
#[derive(Debug, Serialize, Deserialize)]
pub struct ServicesResponse {
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// List of services
    pub services: Vec<ServiceInfo>,
    /// Registry status
    pub registry_status: String,
    /// Service mesh status
    pub service_mesh_status: ServiceMeshIntegrationStatus,
}

/// Service mesh integration status
#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceMeshIntegrationStatus {
    /// Enabled status
    pub enabled: bool,
    /// Provider name
    pub provider: String,
    /// Load balancing enabled
    pub load_balancing: bool,
    /// Cross-primal communication enabled
    pub cross_primal_communication: bool,
}

/// Service information
///
/// # Zero-Copy Performance
///
/// Uses `Arc<str>` for frequently-cloned fields (name, service_type, health)
/// to eliminate allocations during API responses.
#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceInfo {
    /// Service name (zero-copy shared string)
    #[serde(serialize_with = "crate::optimization::zero_copy::serialize_arc_str")]
    #[serde(deserialize_with = "crate::optimization::zero_copy::deserialize_arc_str")]
    pub name: ArcStr,
    /// Service type (zero-copy shared string)
    #[serde(serialize_with = "crate::optimization::zero_copy::serialize_arc_str")]
    #[serde(deserialize_with = "crate::optimization::zero_copy::deserialize_arc_str")]
    pub service_type: ArcStr,
    /// Endpoints
    pub endpoints: Vec<String>,
    /// Health status (zero-copy shared string)
    #[serde(serialize_with = "crate::optimization::zero_copy::serialize_arc_str")]
    #[serde(deserialize_with = "crate::optimization::zero_copy::deserialize_arc_str")]
    pub health: ArcStr,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Songbird registration response
#[derive(Debug, Serialize, Deserialize)]
pub struct SongbirdRegistrationResponse {
    /// Registration success
    pub success: bool,
    /// Message
    pub message: String,
    /// Service ID
    pub service_id: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Songbird heartbeat response
#[derive(Debug, Serialize, Deserialize)]
pub struct SongbirdHeartbeatResponse {
    /// Heartbeat acknowledged
    pub acknowledged: bool,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Next heartbeat time
    pub next_heartbeat: DateTime<Utc>,
}

/// Shutdown response
#[derive(Debug, Serialize, Deserialize)]
pub struct ShutdownResponse {
    /// Shutdown acknowledged
    pub acknowledged: bool,
}
