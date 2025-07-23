//! Service discovery operations for the ecosystem registry

use super::types::{intern_registry_string, DiscoveredService, ServiceHealthStatus};
use crate::EcosystemPrimalType;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock; // Import from crate root

/// Discovery operations for the ecosystem registry
pub struct DiscoveryOps;

impl DiscoveryOps {
    /// Discover services in the ecosystem
    pub async fn discover_services(
        service_registry: &Arc<RwLock<HashMap<Arc<str>, Arc<DiscoveredService>>>>,
        primal_types: Vec<EcosystemPrimalType>,
    ) -> Result<Vec<Arc<DiscoveredService>>, Box<dyn std::error::Error + Send + Sync>> {
        let mut discovered_services = Vec::new();

        for primal_type in primal_types {
            let endpoint = Self::build_service_endpoint(&primal_type);

            // Perform discovery for this primal type
            if let Err(e) =
                Self::perform_service_discovery(service_registry, primal_type.clone(), endpoint)
                    .await
            {
                eprintln!("Failed to discover service for {:?}: {}", primal_type, e);
                continue;
            }
        }

        // Return all discovered services
        let registry = service_registry.read().await;
        discovered_services.extend(registry.values().cloned());

        Ok(discovered_services)
    }

    /// Build service endpoint from primal type
    fn build_service_endpoint(primal_type: &EcosystemPrimalType) -> String {
        match primal_type {
            EcosystemPrimalType::Squirrel => "http://localhost:8080".to_string(),
            EcosystemPrimalType::Songbird => "http://localhost:8081".to_string(),
            EcosystemPrimalType::ToadStool => "http://localhost:8082".to_string(),
            EcosystemPrimalType::BearDog => "http://localhost:8083".to_string(),
            EcosystemPrimalType::NestGate => "http://localhost:8084".to_string(),
            EcosystemPrimalType::BiomeOS => "http://localhost:8085".to_string(),
        }
    }

    /// Perform actual service discovery operations
    async fn perform_service_discovery(
        service_registry: &Arc<RwLock<HashMap<Arc<str>, Arc<DiscoveredService>>>>,
        primal_type: EcosystemPrimalType,
        endpoint: String,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Create discovered service with Arc<str> optimization
        let service = Arc::new(DiscoveredService {
            service_id: intern_registry_string(&format!("{:?}", primal_type).to_lowercase()),
            primal_type: primal_type.clone(),
            endpoint: Arc::from(endpoint.clone()),
            capabilities: vec![
                intern_registry_string("discovery"),
                intern_registry_string("health_check"),
            ],
            health_status: ServiceHealthStatus::Healthy,
            health_endpoint: Arc::from(format!("{}/health", endpoint)),
            discovered_at: chrono::Utc::now(),
            api_version: Arc::from("v1"),
            last_health_check: Some(chrono::Utc::now()),
            metadata: HashMap::new(),
        });

        // Add to registry with Arc<str> key
        let service_id = service.service_id.clone();
        service_registry.write().await.insert(service_id, service);

        Ok(())
    }

    /// Get capabilities for a primal type with Arc<str> optimization
    pub fn get_capabilities_for_primal(primal_type: &EcosystemPrimalType) -> Vec<Arc<str>> {
        match primal_type {
            EcosystemPrimalType::Squirrel => vec![
                intern_registry_string("ai_coordination"),
                intern_registry_string("request_routing"),
                intern_registry_string("response_aggregation"),
                intern_registry_string("context_management"),
            ],
            EcosystemPrimalType::Songbird => vec![
                intern_registry_string("service_mesh"),
                intern_registry_string("load_balancing"),
                intern_registry_string("health_monitoring"),
            ],
            EcosystemPrimalType::ToadStool => vec![
                intern_registry_string("compute"),
                intern_registry_string("storage"),
                intern_registry_string("scaling"),
            ],
            EcosystemPrimalType::BearDog => vec![
                intern_registry_string("security"),
                intern_registry_string("authentication"),
                intern_registry_string("authorization"),
                intern_registry_string("compliance"),
            ],
            EcosystemPrimalType::NestGate => vec![
                intern_registry_string("networking"),
                intern_registry_string("gateway"),
                intern_registry_string("routing"),
            ],
            EcosystemPrimalType::BiomeOS => vec![
                intern_registry_string("operating_system"),
                intern_registry_string("process_management"),
                intern_registry_string("resource_allocation"),
            ],
        }
    }
}
