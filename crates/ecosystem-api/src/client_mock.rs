// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! In-memory [`ServiceMeshClient`](crate::traits::ServiceMeshClient) for tests.

use crate::error::UniversalResult;
use crate::traits::{ServiceInfo, ServiceMeshClient, ServiceQuery};
use crate::types::{EcosystemServiceRegistration, HealthStatus, ServiceMeshStatus};
use std::collections::HashMap;

/// Mock service mesh client for testing
#[derive(Clone)]
pub struct MockServiceMeshClient {
    services: std::sync::Arc<tokio::sync::RwLock<HashMap<String, ServiceInfo>>>,
    health_reports: std::sync::Arc<tokio::sync::RwLock<HashMap<String, HealthStatus>>>,
}

impl Default for MockServiceMeshClient {
    fn default() -> Self {
        Self::new()
    }
}

impl MockServiceMeshClient {
    /// Create a new mock client.
    #[must_use]
    pub fn new() -> Self {
        Self {
            services: std::sync::Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            health_reports: std::sync::Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }

    /// Add a service to the mock registry
    pub async fn add_service(&self, service_id: String, service: ServiceInfo) {
        let mut services = self.services.write().await;
        services.insert(service_id, service);
        drop(services);
    }

    /// Get all registered services
    pub async fn get_all_services(&self) -> Vec<ServiceInfo> {
        let services = self.services.read().await;
        let result = services.values().cloned().collect();
        drop(services);
        result
    }
}

impl ServiceMeshClient for MockServiceMeshClient {
    async fn register_service(
        &self,
        _endpoint: &str,
        registration: EcosystemServiceRegistration,
    ) -> UniversalResult<String> {
        let service_id = registration.service_id.to_string();
        let service_info = ServiceInfo {
            id: service_id.clone(),
            name: service_id.clone(),
            service_type: registration.primal_type.as_str().to_string(),
            primal_type: registration.primal_type,
            endpoint: registration.endpoints.health.clone(),
            capabilities: registration.capabilities.core.clone(),
            health_status: "healthy".to_string(),
            metadata: registration.metadata.clone(),
        };

        self.add_service(service_id.clone(), service_info).await;
        Ok(service_id)
    }

    async fn deregister_service(&self, service_id: &str) -> UniversalResult<()> {
        let mut services = self.services.write().await;
        services.remove(service_id);
        drop(services);
        Ok(())
    }

    async fn discover_services(&self, query: ServiceQuery) -> UniversalResult<Vec<ServiceInfo>> {
        let services = self.services.read().await;
        let mut results = Vec::new();

        for service in services.values() {
            let matches = query
                .service_type
                .as_ref()
                .is_none_or(|st| service.service_type == *st)
                && query.primal_type.is_none_or(|pt| service.primal_type == pt)
                && (query.capabilities.is_empty()
                    || query
                        .capabilities
                        .iter()
                        .all(|c| service.capabilities.contains(c)));

            if matches {
                results.push(service.clone());
            }
        }

        drop(services);
        Ok(results)
    }

    async fn get_service(&self, service_id: &str) -> UniversalResult<Option<ServiceInfo>> {
        let services = self.services.read().await;
        let result = services.get(service_id).cloned();
        drop(services);
        Ok(result)
    }

    async fn report_health(&self, service_id: &str, health: HealthStatus) -> UniversalResult<()> {
        let mut health_reports = self.health_reports.write().await;
        health_reports.insert(service_id.to_string(), health);
        drop(health_reports);
        Ok(())
    }

    async fn heartbeat(&self, service_id: &str) -> UniversalResult<()> {
        // Mock heartbeat - just record the time
        let mut services = self.services.write().await;
        if let Some(service) = services.get_mut(service_id) {
            service.metadata.insert(
                "last_heartbeat".to_string(),
                chrono::Utc::now().to_rfc3339(),
            );
        }
        drop(services);
        Ok(())
    }

    async fn get_mesh_status(&self) -> UniversalResult<ServiceMeshStatus> {
        Ok(ServiceMeshStatus {
            connected: true,
            service_mesh_endpoint: None, // endpoints discovered at runtime
            registration_time: Some(chrono::Utc::now()),
            last_heartbeat: Some(chrono::Utc::now()),
            metadata: HashMap::new(),
        })
    }
}
