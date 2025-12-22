//! Service discovery logic for ecosystem management
//!
//! This module handles capability-based service discovery, registration,
//! and selection within the ecoPrimals ecosystem. Following the principle
//! of "primal code has self-knowledge and discovers others at runtime".

use std::sync::Arc;
use tracing;

use crate::error::PrimalError;
use crate::monitoring::MetricsCollector;
use crate::primal_provider::SquirrelPrimalProvider;

use super::types::*;
use super::EcosystemRegistryManager;

/// Service discovery coordinator
///
/// Handles finding services by capability, not by hardcoded names.
/// This ensures we follow the self-knowledge principle where primals
/// only know about themselves and discover others at runtime.
pub struct ServiceDiscovery {
    registry_manager: Arc<EcosystemRegistryManager>,
    metrics_collector: Arc<MetricsCollector>,
}

impl ServiceDiscovery {
    /// Create new service discovery instance
    pub fn new(
        registry_manager: Arc<EcosystemRegistryManager>,
        metrics_collector: Arc<MetricsCollector>,
    ) -> Self {
        Self {
            registry_manager,
            metrics_collector,
        }
    }

    /// Discover services by capability (NOT by primal name)
    ///
    /// This follows the capability-based discovery pattern where we query
    /// for what a service can do, not who provides it.
    ///
    /// # Examples
    /// ```ignore
    /// // CORRECT: Discover by capability
    /// let security_services = discovery.discover_by_capability("security").await?;
    ///
    /// // WRONG: Don't hardcode primal names
    /// // let beardog = ecosystem.get_service("beardog").await?;
    /// ```
    pub async fn discover_by_capability(
        &self,
        capability: &str,
    ) -> Result<Vec<DiscoveredService>, PrimalError> {
        tracing::debug!("Discovering services with capability: {}", capability);

        // Query ecosystem registry for services with this capability
        let services = self
            .registry_manager
            .find_services_by_capability(capability)
            .await?;

        // Record metrics
        self.metrics_collector
            .record_service_discovery(capability, services.len());

        Ok(services)
    }

    /// Register a service with the ecosystem
    ///
    /// Services register their capabilities, not their identity.
    /// The ecosystem then matches capabilities at runtime.
    pub async fn register_service(
        &self,
        registration: EcosystemServiceRegistration,
    ) -> Result<(), PrimalError> {
        tracing::info!(
            "Registering service: {} with capabilities: {:?}",
            registration.service_id,
            registration.capabilities.capabilities
        );

        // Register with ecosystem registry
        self.registry_manager
            .register_service(registration)
            .await?;

        Ok(())
    }

    /// Unregister a service from the ecosystem
    pub async fn unregister_service(&self, service_id: &str) -> Result<(), PrimalError> {
        tracing::info!("Unregistering service: {}", service_id);

        self.registry_manager
            .unregister_service(service_id)
            .await?;

        Ok(())
    }

    /// Select the best service from a list based on health, load, etc.
    ///
    /// This implements intelligent service selection without hardcoding
    /// preferences for specific primals.
    pub async fn select_best_service(
        &self,
        services: &[DiscoveredService],
    ) -> Option<DiscoveredService> {
        if services.is_empty() {
            return None;
        }

        // TODO: Implement sophisticated selection algorithm
        // For now, return the first healthy service
        services
            .iter()
            .find(|s| s.health_status == "healthy")
            .cloned()
            .or_else(|| services.first().cloned())
    }

    /// Create service registration from provider
    ///
    /// This converts our internal provider structure to the standardized
    /// ecosystem registration format.
    pub fn create_registration(
        &self,
        provider: &SquirrelPrimalProvider,
    ) -> Result<EcosystemServiceRegistration, PrimalError> {
        let config = provider.config();

        Ok(EcosystemServiceRegistration {
            service_id: config.service_id.clone(),
            service_name: "Squirrel AI Primal".to_string(),
            service_type: "ai_agent_platform".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            endpoints: ServiceEndpoints {
                http: Some(format!("http://{}:{}", config.host, config.port)),
                grpc: config.grpc_port.map(|p| format!("{}:{}", config.host, p)),
                websocket: None,
            },
            capabilities: ServiceCapabilities {
                capabilities: vec![
                    "ai_inference".to_string(),
                    "agent_platform".to_string(),
                    "context_management".to_string(),
                ],
            },
            health_check: HealthCheckConfig::default(),
            resource_requirements: ResourceRequirements::default(),
            security: SecurityConfig::default(),
            metadata: std::collections::HashMap::new(),
        })
    }
}

/// Discovered service information
#[derive(Debug, Clone)]
pub struct DiscoveredService {
    pub service_id: String,
    pub service_type: String,
    pub endpoint: String,
    pub capabilities: Vec<String>,
    pub health_status: String,
}

