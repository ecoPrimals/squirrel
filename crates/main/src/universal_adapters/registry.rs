// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Universal Service Registry
//!
//! Implements capability-based service discovery and matching following the
//! Universal Primal Architecture Standard.

use async_trait::async_trait; // KEEP: UniversalServiceRegistry used as trait object
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use super::{ServiceCapability, ServiceHealth, UniversalServiceRegistration};
use crate::error::PrimalError;

/// Universal Service Registry trait for capability-based discovery

#[async_trait]
pub trait UniversalServiceRegistry: Send + Sync {
    /// Register a service with its capabilities
    async fn register_service(
        &self,
        registration: UniversalServiceRegistration,
    ) -> Result<(), PrimalError>;

    /// Discover services by capability
    async fn discover_by_capability(
        &self,
        capability: ServiceCapability,
    ) -> Result<Vec<ServiceInfo>, PrimalError>;

    /// Find services by category
    async fn discover_by_category(&self, category: &str) -> Result<Vec<ServiceInfo>, PrimalError>;

    /// Get optimal service for specific requirements
    async fn find_optimal_service(
        &self,
        requirements: ServiceRequirements,
    ) -> Result<ServiceInfo, PrimalError>;

    /// Update service health status
    async fn update_service_health(
        &self,
        service_id: &str,
        health: ServiceHealth,
    ) -> Result<(), PrimalError>;

    /// Deregister service
    async fn deregister_service(&self, service_id: &str) -> Result<(), PrimalError>;

    /// List all registered services
    async fn list_all_services(&self) -> Result<Vec<ServiceInfo>, PrimalError>;
}

/// Service information for discovery results
#[derive(Debug, Clone)]
pub struct ServiceInfo {
    /// Unique service identifier
    pub service_id: String,
    /// Human-readable service name
    pub name: String,
    /// Service category for filtering
    pub category: String,
    /// Capabilities this service provides
    pub capabilities: Vec<ServiceCapability>,
    /// Service endpoint URLs
    pub endpoints: Vec<String>,
    /// Current health status
    pub health: ServiceHealth,
    /// Priority score (higher is preferred)
    pub priority: u8,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Service requirements for finding optimal services
#[derive(Debug, Clone)]
pub struct ServiceRequirements {
    /// Capabilities that must be present
    pub required_capabilities: Vec<ServiceCapability>,
    /// Capabilities that improve selection
    pub optional_capabilities: Vec<ServiceCapability>,
    /// Performance criteria (e.g., latency, throughput)
    pub performance_requirements: HashMap<String, serde_json::Value>,
    /// Geographic region preference
    pub geographic_preferences: Option<String>,
    /// Priority level (0-255)
    pub priority_level: u8,
}

/// In-memory service registry implementation
pub struct InMemoryServiceRegistry {
    services: Arc<RwLock<HashMap<String, RegisteredService>>>,
}

#[derive(Debug, Clone)]
struct RegisteredService {
    registration: UniversalServiceRegistration,
    health: ServiceHealth,
    last_seen: chrono::DateTime<chrono::Utc>,
}

impl Default for InMemoryServiceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryServiceRegistry {
    /// Creates a new in-memory service registry.
    #[must_use]
    pub fn new() -> Self {
        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl UniversalServiceRegistry for InMemoryServiceRegistry {
    async fn register_service(
        &self,
        registration: UniversalServiceRegistration,
    ) -> Result<(), PrimalError> {
        let service_id = registration.service_id.to_string();
        let service_name = registration.metadata.name.clone();

        info!(
            "🌌 Registering universal service: {} ({})",
            service_name, service_id
        );

        let registered_service = RegisteredService {
            registration: registration.clone(),
            health: ServiceHealth {
                healthy: true,
                message: Some("Newly registered".to_string()),
                metrics: HashMap::new(),
            },
            last_seen: chrono::Utc::now(),
        };

        let mut services = self.services.write().await;
        services.insert(service_id.clone(), registered_service);

        info!(
            "✅ Service {} successfully registered with {} capabilities",
            service_name,
            registration.capabilities.len()
        );

        Ok(())
    }

    async fn discover_by_capability(
        &self,
        target_capability: ServiceCapability,
    ) -> Result<Vec<ServiceInfo>, PrimalError> {
        debug!(
            "🔍 Discovering services by capability: {:?}",
            target_capability
        );

        let services = self.services.read().await;
        let mut matching_services = Vec::new();

        for (service_id, registered_service) in services.iter() {
            // Check if service has compatible capability
            for capability in &registered_service.registration.capabilities {
                if self.capabilities_match(&target_capability, capability) {
                    matching_services.push(ServiceInfo {
                        service_id: service_id.clone(),
                        name: registered_service.registration.metadata.name.clone(),
                        category: format!(
                            "{:?}",
                            registered_service.registration.metadata.category
                        ),
                        capabilities: registered_service.registration.capabilities.clone(),
                        endpoints: registered_service
                            .registration
                            .endpoints
                            .iter()
                            .map(|e| e.url.clone())
                            .collect(),
                        health: registered_service.health.clone(),
                        priority: registered_service.registration.priority,
                        metadata: registered_service.registration.extensions.clone(),
                    });
                    break;
                }
            }
        }

        debug!(
            "🎯 Found {} services with matching capability",
            matching_services.len()
        );
        Ok(matching_services)
    }

    async fn discover_by_category(&self, category: &str) -> Result<Vec<ServiceInfo>, PrimalError> {
        debug!("🔍 Discovering services by category: {}", category);

        let services = self.services.read().await;
        let mut matching_services = Vec::new();

        for (service_id, registered_service) in services.iter() {
            let service_category =
                format!("{:?}", registered_service.registration.metadata.category);

            if service_category
                .to_lowercase()
                .contains(&category.to_lowercase())
            {
                matching_services.push(ServiceInfo {
                    service_id: service_id.clone(),
                    name: registered_service.registration.metadata.name.clone(),
                    category: service_category,
                    capabilities: registered_service.registration.capabilities.clone(),
                    endpoints: registered_service
                        .registration
                        .endpoints
                        .iter()
                        .map(|e| e.url.clone())
                        .collect(),
                    health: registered_service.health.clone(),
                    priority: registered_service.registration.priority,
                    metadata: registered_service.registration.extensions.clone(),
                });
            }
        }

        debug!(
            "🎯 Found {} services in category '{}'",
            matching_services.len(),
            category
        );
        Ok(matching_services)
    }

    async fn find_optimal_service(
        &self,
        requirements: ServiceRequirements,
    ) -> Result<ServiceInfo, PrimalError> {
        debug!("🎯 Finding optimal service for requirements");

        // Start with services that have required capabilities
        let mut candidates = Vec::new();

        for required_capability in &requirements.required_capabilities {
            let services = self
                .discover_by_capability(required_capability.clone())
                .await?;
            if candidates.is_empty() {
                candidates = services;
            } else {
                // Keep only services that appear in both lists
                candidates.retain(|candidate| {
                    services
                        .iter()
                        .any(|s| s.service_id == candidate.service_id)
                });
            }
        }

        if candidates.is_empty() {
            return Err(PrimalError::ServiceDiscoveryError(
                "No services found matching required capabilities".to_string(),
            ));
        }

        // Score candidates based on priority, health, and optional capabilities
        let mut best_service = None;
        let mut best_score = 0.0f64;

        for candidate in candidates {
            let mut score = f64::from(candidate.priority);

            // Health bonus
            if candidate.health.healthy {
                score += 10.0;
            }

            // Optional capability bonuses
            for optional_capability in &requirements.optional_capabilities {
                if candidate
                    .capabilities
                    .iter()
                    .any(|c| self.capabilities_match(optional_capability, c))
                {
                    score += 5.0;
                }
            }

            if score > best_score {
                best_score = score;
                best_service = Some(candidate);
            }
        }

        best_service.ok_or_else(|| {
            PrimalError::ServiceDiscoveryError("No optimal service found".to_string())
        })
    }

    async fn update_service_health(
        &self,
        service_id: &str,
        health: ServiceHealth,
    ) -> Result<(), PrimalError> {
        let mut services = self.services.write().await;

        if let Some(registered_service) = services.get_mut(service_id) {
            registered_service.health = health;
            registered_service.last_seen = chrono::Utc::now();
            debug!("💓 Updated health for service: {}", service_id);
            Ok(())
        } else {
            Err(PrimalError::ServiceDiscoveryError(format!(
                "Service not found: {service_id}"
            )))
        }
    }

    async fn deregister_service(&self, service_id: &str) -> Result<(), PrimalError> {
        let mut services = self.services.write().await;

        if services.remove(service_id).is_some() {
            info!("🚫 Deregistered service: {}", service_id);
            Ok(())
        } else {
            warn!("⚠️ Attempted to deregister unknown service: {}", service_id);
            Err(PrimalError::ServiceDiscoveryError(format!(
                "Service not found: {service_id}"
            )))
        }
    }

    async fn list_all_services(&self) -> Result<Vec<ServiceInfo>, PrimalError> {
        let services = self.services.read().await;

        let service_list = services
            .iter()
            .map(|(service_id, registered_service)| ServiceInfo {
                service_id: service_id.clone(),
                name: registered_service.registration.metadata.name.clone(),
                category: format!("{:?}", registered_service.registration.metadata.category),
                capabilities: registered_service.registration.capabilities.clone(),
                endpoints: registered_service
                    .registration
                    .endpoints
                    .iter()
                    .map(|e| e.url.clone())
                    .collect(),
                health: registered_service.health.clone(),
                priority: registered_service.registration.priority,
                metadata: registered_service.registration.extensions.clone(),
            })
            .collect();

        Ok(service_list)
    }
}

impl InMemoryServiceRegistry {
    /// Check if two capabilities match
    fn capabilities_match(
        &self,
        required: &ServiceCapability,
        provided: &ServiceCapability,
    ) -> bool {
        use ServiceCapability::{
            ArtificialIntelligence, Computation, Coordination, Custom, DataManagement, Security,
        };

        match (required, provided) {
            (
                Security {
                    functions: req_funcs,
                    ..
                },
                Security {
                    functions: prov_funcs,
                    ..
                },
            ) => req_funcs.iter().any(|req| prov_funcs.contains(req)),
            (
                Coordination {
                    patterns: req_patterns,
                    ..
                },
                Coordination {
                    patterns: prov_patterns,
                    ..
                },
            ) => req_patterns.iter().any(|req| prov_patterns.contains(req)),
            (
                DataManagement {
                    operations: req_ops,
                    ..
                },
                DataManagement {
                    operations: prov_ops,
                    ..
                },
            ) => req_ops.iter().any(|req| prov_ops.contains(req)),
            (
                Computation {
                    types: req_types, ..
                },
                Computation {
                    types: prov_types, ..
                },
            ) => req_types.iter().any(|req| prov_types.contains(req)),
            (
                ArtificialIntelligence {
                    tasks: req_tasks, ..
                },
                ArtificialIntelligence {
                    tasks: prov_tasks, ..
                },
            ) => req_tasks.iter().any(|req| prov_tasks.contains(req)),
            (
                Custom {
                    domain: req_domain,
                    capability: req_cap,
                    ..
                },
                Custom {
                    domain: prov_domain,
                    capability: prov_cap,
                    ..
                },
            ) => req_domain == prov_domain && req_cap == prov_cap,
            _ => false,
        }
    }
}

/// Service matcher for intelligent service selection
pub struct ServiceMatcher {
    registry: Arc<dyn UniversalServiceRegistry>,
}

impl ServiceMatcher {
    /// Creates a new service matcher with the given registry.
    pub fn new(registry: Arc<dyn UniversalServiceRegistry>) -> Self {
        Self { registry }
    }

    /// Find the best service for a specific task
    pub async fn match_service_for_task(
        &self,
        task_description: &str,
        required_capabilities: Vec<ServiceCapability>,
    ) -> Result<ServiceInfo, PrimalError> {
        info!("🎯 Matching service for task: {}", task_description);

        let requirements = ServiceRequirements {
            required_capabilities,
            optional_capabilities: Vec::new(),
            performance_requirements: HashMap::new(),
            geographic_preferences: None,
            priority_level: 5,
        };

        self.registry.find_optimal_service(requirements).await
    }

    /// Auto-discover all available services
    pub async fn auto_discover_services(&self) -> Result<Vec<ServiceInfo>, PrimalError> {
        debug!("🌌 Auto-discovering all available services");
        self.registry.list_all_services().await
    }
}
