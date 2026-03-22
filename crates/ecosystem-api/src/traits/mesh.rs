// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Service mesh client trait.

use crate::error::UniversalResult;
use crate::types::{EcosystemServiceRegistration, HealthStatus, ServiceMeshStatus};

use super::discovery::{ServiceInfo, ServiceQuery};

use async_trait::async_trait;

/// Service mesh client trait for interacting with Songbird
///
/// This trait provides the interface for communicating with the Songbird
/// service mesh for service discovery, registration, and health reporting.
#[async_trait]
pub trait ServiceMeshClient: Send + Sync {
    /// Register a service with the service mesh
    async fn register_service(
        &self,
        endpoint: &str,
        registration: EcosystemServiceRegistration,
    ) -> UniversalResult<String>;

    /// Deregister a service from the service mesh
    async fn deregister_service(&self, service_id: &str) -> UniversalResult<()>;

    /// Discover services in the service mesh
    async fn discover_services(&self, query: ServiceQuery) -> UniversalResult<Vec<ServiceInfo>>;

    /// Get service information by ID
    async fn get_service(&self, service_id: &str) -> UniversalResult<Option<ServiceInfo>>;

    /// Report health status
    async fn report_health(&self, service_id: &str, health: HealthStatus) -> UniversalResult<()>;

    /// Send heartbeat
    async fn heartbeat(&self, service_id: &str) -> UniversalResult<()>;

    /// Get service mesh status
    async fn get_mesh_status(&self) -> UniversalResult<ServiceMeshStatus>;
}
