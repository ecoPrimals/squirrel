// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! HTTP request/response DTOs for the legacy Songbird service mesh client.

use crate::traits::ServiceInfo;
use crate::types::{HealthStatus, ServiceMeshStatus};

/// Response body for service registration.
#[derive(serde::Deserialize)]
pub struct ServiceRegistrationResponse {
    /// Assigned service identifier.
    pub service_id: String,
}

/// Response body for service discovery listing.
#[derive(serde::Deserialize)]
pub struct ServiceDiscoveryResponse {
    /// Matching services.
    pub services: Vec<ServiceInfo>,
}

/// Response body for a single service lookup.
#[derive(serde::Deserialize)]
pub struct ServiceResponse {
    /// The service record.
    pub service: ServiceInfo,
}

/// Response body for mesh status.
#[derive(serde::Deserialize)]
pub struct MeshStatusResponse {
    /// Aggregated mesh status.
    pub status: ServiceMeshStatus,
}

/// Empty JSON object accepted by some Songbird endpoints.
#[derive(serde::Deserialize)]
pub struct EmptyResponse {}

/// Payload for reporting health to the mesh.
#[derive(serde::Serialize)]
pub struct HealthReport {
    /// Service this report refers to.
    pub service_id: String,
    /// Reported health state.
    pub status: HealthStatus,
    /// Timestamp of the report (UTC).
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Payload for heartbeat requests.
#[derive(serde::Serialize)]
pub struct HeartbeatData {
    /// Service sending the heartbeat.
    pub service_id: String,
    /// Heartbeat time (UTC).
    pub timestamp: chrono::DateTime<chrono::Utc>,
}
