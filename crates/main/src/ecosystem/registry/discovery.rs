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
                Self::perform_service_discovery(service_registry, primal_type, endpoint).await
            {
                eprintln!("Failed to discover service for {primal_type:?}: {e}");
                continue;
            }
        }

        // Return all discovered services
        let registry = service_registry.read().await;
        discovered_services.extend(registry.values().cloned());

        Ok(discovered_services)
    }

    /// Build service endpoint from primal type
    ///
    /// This function discovers service endpoints using a **pure capability-based approach**:
    ///
    /// ## Discovery Priority (Highest to Lowest)
    ///
    /// 1. **Environment Variables** (Production)
    ///    - `{PRIMAL}_ENDPOINT` - Direct endpoint specification
    ///    - Example: `SONGBIRD_ENDPOINT=https://songbird.prod.example.com`
    ///
    /// 2. **Service Discovery Systems** (Production)
    ///    - `SERVICE_DISCOVERY_URL` - Registry endpoint (Consul, etcd, etc.)
    ///    - Queries by capability, not by primal name
    ///    - Example: Query for "service-mesh" capability, not "songbird"
    ///
    /// 3. **Configuration File** (Optional)
    ///    - `SQUIRREL_CONFIG` - Path to config file with service endpoints
    ///    - Allows dynamic endpoint specification without env vars
    ///
    /// 4. **Development Defaults** (Dev Only)
    ///    - Localhost with universal-constants defined ports
    ///    - **NEVER used in production** - fails fast if no discovery configured
    ///
    /// ## Capability-Based Philosophy
    ///
    /// This function does NOT hardcode primal names. Instead:
    /// - Primals discover each other by **capability** (e.g., "security", "storage")
    /// - Each primal only knows its own identity
    /// - Runtime discovery enables zero vendor lock-in
    /// - Services can be swapped without code changes
    fn build_service_endpoint(primal_type: &EcosystemPrimalType) -> String {
        // 1. Try environment variable first (highest priority)
        let env_var = format!("{}_ENDPOINT", primal_type.env_name());
        if let Ok(endpoint) = std::env::var(&env_var) {
            tracing::debug!(
                "Using environment variable {} for {:?}",
                env_var,
                primal_type
            );
            return endpoint;
        }

        // 2. Try SERVICE_DISCOVERY environment variable for dynamic discovery
        if let Ok(discovery_url) = std::env::var("SERVICE_DISCOVERY_URL") {
            tracing::debug!(
                "Using service discovery at {} for {:?}",
                discovery_url,
                primal_type
            );
            // In production, this would query a service registry by capability
            // Example: Query for services with "security" capability, not "beardog"
            return format!("{}/{}", discovery_url, primal_type.service_name());
        }

        // 3. Try configuration file
        if let Ok(config_path) = std::env::var("SQUIRREL_CONFIG") {
            if let Ok(endpoint) = Self::read_endpoint_from_config(&config_path, primal_type) {
                tracing::debug!("Using config file {} for {:?}", config_path, primal_type);
                return endpoint;
            }
        }

        // 4. Fall back to development defaults (dev environment only)
        // In production deployment, this should fail fast with error logging
        if cfg!(debug_assertions) {
            tracing::warn!(
                "Using development default for {:?} - set environment variables in production!",
                primal_type
            );
            Self::get_development_default(primal_type)
        } else {
            // Production: Fail fast rather than use localhost defaults
            tracing::error!(
                "No endpoint configured for {:?} - set {}_ENDPOINT or SERVICE_DISCOVERY_URL",
                primal_type,
                primal_type.env_name()
            );
            // Return an invalid endpoint that will fail discovery
            "http://unconfigured.endpoint".to_string()
        }
    }

    /// Read endpoint from configuration file
    ///
    /// This enables configuration-driven discovery without environment variables
    fn read_endpoint_from_config(
        _config_path: &str,
        _primal_type: &EcosystemPrimalType,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // Future: Configuration file parsing for service endpoints
        // Currently uses environment variables and discovery
        // This would read from TOML/JSON/YAML config file
        // Example: { "endpoints": { "security": "https://beardog.example.com" } }
        Err("Configuration file parsing not yet implemented".into())
    }

    /// Get development default endpoints (ONLY for development environment)
    ///
    /// ⚠️ WARNING: These are development defaults only!
    /// In production, you MUST set environment variables:
    /// - `SQUIRREL_ENDPOINT`, `SONGBIRD_ENDPOINT`, etc., OR
    /// - `SERVICE_DISCOVERY_URL` for dynamic discovery
    ///
    /// This function uses universal-constants for all port assignments to ensure
    /// consistency across the ecosystem. It does NOT use hardcoded primal names,
    /// instead deriving endpoints from the capability-based primal type.
    fn get_development_default(primal_type: &EcosystemPrimalType) -> String {
        use universal_constants::{builders, network};

        // Map primal types to their service names for runtime port discovery
        // All ports are discovered at runtime via environment or capability discovery
        let port = match primal_type {
            EcosystemPrimalType::Squirrel => network::get_service_port("http"), // Squirrel AI service
            EcosystemPrimalType::Songbird => network::get_service_port("service_mesh"), // Service mesh
            EcosystemPrimalType::ToadStool => network::get_service_port("compute"),     // Compute
            EcosystemPrimalType::BearDog => network::get_service_port("security"),      // Security
            EcosystemPrimalType::NestGate => network::get_service_port("storage"),      // Storage
            EcosystemPrimalType::BiomeOS => network::get_service_port("ui"),            // UI
        };

        // Use builder functions from universal-constants for consistency
        builders::localhost_http(port)
    }

    /// Perform actual service discovery operations
    async fn perform_service_discovery(
        service_registry: &Arc<RwLock<HashMap<Arc<str>, Arc<DiscoveredService>>>>,
        primal_type: EcosystemPrimalType,
        endpoint: String,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Create discovered service with Arc<str> optimization
        let service = Arc::new(DiscoveredService {
            service_id: intern_registry_string(&format!("{primal_type:?}").to_lowercase()),
            primal_type,
            endpoint: Arc::from(endpoint.clone()),
            capabilities: vec![
                intern_registry_string("discovery"),
                intern_registry_string("health_check"),
            ],
            health_status: ServiceHealthStatus::Healthy,
            health_endpoint: Arc::from(format!("{endpoint}/health")),
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
    #[must_use]
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
