// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Universal Service Registry
//!
//! Implements capability-based service discovery and matching following the
//! Universal Primal Architecture Standard.

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use super::{ServiceCapability, ServiceHealth, UniversalServiceRegistration};
use crate::error::PrimalError;

/// Universal Service Registry trait for capability-based discovery
///
/// Async methods return boxed futures so implementations remain object-safe without
/// the `async_trait` crate.
pub trait UniversalServiceRegistry: Send + Sync {
    /// Register a service with its capabilities
    fn register_service(
        &self,
        registration: UniversalServiceRegistration,
    ) -> Pin<Box<dyn Future<Output = Result<(), PrimalError>> + Send + '_>>;

    /// Discover services by capability
    fn discover_by_capability(
        &self,
        capability: ServiceCapability,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<ServiceInfo>, PrimalError>> + Send + '_>>;

    /// Find services by category
    fn discover_by_category(
        &self,
        category: &str,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<ServiceInfo>, PrimalError>> + Send + '_>>;

    /// Get optimal service for specific requirements
    fn find_optimal_service(
        &self,
        requirements: ServiceRequirements,
    ) -> Pin<Box<dyn Future<Output = Result<ServiceInfo, PrimalError>> + Send + '_>>;

    /// Update service health status
    fn update_service_health(
        &self,
        service_id: &str,
        health: ServiceHealth,
    ) -> Pin<Box<dyn Future<Output = Result<(), PrimalError>> + Send + '_>>;

    /// Deregister service
    fn deregister_service(
        &self,
        service_id: &str,
    ) -> Pin<Box<dyn Future<Output = Result<(), PrimalError>> + Send + '_>>;

    /// List all registered services
    fn list_all_services(
        &self,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<ServiceInfo>, PrimalError>> + Send + '_>>;
}

/// Service information for discovery results.
///
/// String-heavy fields use `Arc<str>` to avoid deep clones during
/// high-frequency capability discovery queries.
#[derive(Debug, Clone)]
pub struct ServiceInfo {
    /// Unique service identifier
    pub service_id: Arc<str>,
    /// Human-readable service name
    pub name: Arc<str>,
    /// Service category for filtering
    pub category: Arc<str>,
    /// Capabilities this service provides
    pub capabilities: Vec<ServiceCapability>,
    /// Service endpoint URLs
    pub endpoints: Vec<Arc<str>>,
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
#[derive(Clone)]
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

impl UniversalServiceRegistry for InMemoryServiceRegistry {
    fn register_service(
        &self,
        registration: UniversalServiceRegistration,
    ) -> Pin<Box<dyn Future<Output = Result<(), PrimalError>> + Send + '_>> {
        let services = Arc::clone(&self.services);
        Box::pin(async move {
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

            let mut services = services.write().await;
            services.insert(service_id.clone(), registered_service);

            info!(
                "✅ Service {} successfully registered with {} capabilities",
                service_name,
                registration.capabilities.len()
            );

            Ok(())
        })
    }

    fn discover_by_capability(
        &self,
        target_capability: ServiceCapability,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<ServiceInfo>, PrimalError>> + Send + '_>> {
        let services = Arc::clone(&self.services);
        let this = self.clone();
        Box::pin(async move {
            debug!(
                "🔍 Discovering services by capability: {:?}",
                target_capability
            );

            let services = services.read().await;
            let mut matching_services = Vec::new();

            for (service_id, registered_service) in services.iter() {
                // Check if service has compatible capability
                for capability in &registered_service.registration.capabilities {
                    if this.capabilities_match(&target_capability, capability) {
                        matching_services.push(ServiceInfo {
                            service_id: Arc::from(service_id.as_str()),
                            name: Arc::from(registered_service.registration.metadata.name.as_str()),
                            category: Arc::from(format!(
                                "{:?}",
                                registered_service.registration.metadata.category
                            )),
                            capabilities: registered_service.registration.capabilities.clone(),
                            endpoints: registered_service
                                .registration
                                .endpoints
                                .iter()
                                .map(|e| Arc::from(e.url.as_str()))
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
        })
    }

    fn discover_by_category(
        &self,
        category: &str,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<ServiceInfo>, PrimalError>> + Send + '_>> {
        let services = Arc::clone(&self.services);
        let category = category.to_string();
        Box::pin(async move {
            debug!("🔍 Discovering services by category: {}", category);

            let services = services.read().await;
            let mut matching_services = Vec::new();

            for (service_id, registered_service) in services.iter() {
                let service_category =
                    format!("{:?}", registered_service.registration.metadata.category);

                if service_category
                    .to_lowercase()
                    .contains(&category.to_lowercase())
                {
                    matching_services.push(ServiceInfo {
                        service_id: Arc::from(service_id.as_str()),
                        name: Arc::from(registered_service.registration.metadata.name.as_str()),
                        category: Arc::from(service_category),
                        capabilities: registered_service.registration.capabilities.clone(),
                        endpoints: registered_service
                            .registration
                            .endpoints
                            .iter()
                            .map(|e| Arc::from(e.url.as_str()))
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
        })
    }

    fn find_optimal_service(
        &self,
        requirements: ServiceRequirements,
    ) -> Pin<Box<dyn Future<Output = Result<ServiceInfo, PrimalError>> + Send + '_>> {
        let this = self.clone();
        Box::pin(async move {
            debug!("🎯 Finding optimal service for requirements");

            // Start with services that have required capabilities
            let mut candidates = Vec::new();

            for required_capability in &requirements.required_capabilities {
                let services = this
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
                        .any(|c| this.capabilities_match(optional_capability, c))
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
        })
    }

    fn update_service_health(
        &self,
        service_id: &str,
        health: ServiceHealth,
    ) -> Pin<Box<dyn Future<Output = Result<(), PrimalError>> + Send + '_>> {
        let services = Arc::clone(&self.services);
        let service_id = service_id.to_string();
        Box::pin(async move {
            let mut services = services.write().await;

            if let Some(registered_service) = services.get_mut(&service_id) {
                registered_service.health = health;
                registered_service.last_seen = chrono::Utc::now();
                debug!("💓 Updated health for service: {}", service_id);
                Ok(())
            } else {
                Err(PrimalError::ServiceDiscoveryError(format!(
                    "Service not found: {service_id}"
                )))
            }
        })
    }

    fn deregister_service(
        &self,
        service_id: &str,
    ) -> Pin<Box<dyn Future<Output = Result<(), PrimalError>> + Send + '_>> {
        let services = Arc::clone(&self.services);
        let service_id = service_id.to_string();
        Box::pin(async move {
            let mut services = services.write().await;

            if services.remove(&service_id).is_some() {
                info!("🚫 Deregistered service: {}", service_id);
                Ok(())
            } else {
                warn!("⚠️ Attempted to deregister unknown service: {}", service_id);
                Err(PrimalError::ServiceDiscoveryError(format!(
                    "Service not found: {service_id}"
                )))
            }
        })
    }

    fn list_all_services(
        &self,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<ServiceInfo>, PrimalError>> + Send + '_>> {
        let services = Arc::clone(&self.services);
        Box::pin(async move {
            let services = services.read().await;

            let service_list = services
                .iter()
                .map(|(service_id, registered_service)| ServiceInfo {
                    service_id: Arc::from(service_id.as_str()),
                    name: Arc::from(registered_service.registration.metadata.name.as_str()),
                    category: Arc::from(format!(
                        "{:?}",
                        registered_service.registration.metadata.category
                    )),
                    capabilities: registered_service.registration.capabilities.clone(),
                    endpoints: registered_service
                        .registration
                        .endpoints
                        .iter()
                        .map(|e| Arc::from(e.url.as_str()))
                        .collect(),
                    health: registered_service.health.clone(),
                    priority: registered_service.registration.priority,
                    metadata: registered_service.registration.extensions.clone(),
                })
                .collect();

            Ok(service_list)
        })
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
    registry: Arc<InMemoryServiceRegistry>,
}

impl ServiceMatcher {
    /// Creates a new service matcher with the given registry.
    #[must_use]
    pub const fn new(registry: Arc<InMemoryServiceRegistry>) -> Self {
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
