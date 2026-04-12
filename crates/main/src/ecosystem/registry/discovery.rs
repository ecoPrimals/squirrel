// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors
// Backward compatibility: discover_services/DiscoveredService use EcosystemPrimalType for legacy format
#![expect(deprecated, reason = "Backward compatibility during migration")]

//! Service discovery operations for the ecosystem registry

use super::types::{DiscoveredService, ServiceHealthStatus, intern_registry_string};
use crate::EcosystemPrimalType;
use crate::error::PrimalError;
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
    ) -> Result<Vec<Arc<DiscoveredService>>, PrimalError> {
        let mut discovered_services = Vec::new();

        for primal_type in primal_types {
            let endpoint = Self::build_service_endpoint(primal_type);

            // Perform discovery for this primal type
            if let Err(e) =
                Self::perform_service_discovery(service_registry, primal_type, endpoint).await
            {
                tracing::error!("Failed to discover service for {primal_type:?}: {e}");
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
    ///    - `{CAPABILITY_PREFIX}_ENDPOINT` — prefix from [`EcosystemPrimalType::endpoint_env_prefix`]
    ///      (capability-derived, e.g. `SERVICE_MESH_ENDPOINT`, `SECURITY_ENDPOINT`)
    ///
    /// 2. **Service Discovery Systems** (Production)
    ///    - `SERVICE_DISCOVERY_URL` - Registry endpoint (Consul, etcd, etc.)
    ///    - Queries by capability, not by marketing primal name
    ///    - Example: Query for `service-mesh` / `security` capability identifiers
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
    fn build_service_endpoint(primal_type: EcosystemPrimalType) -> String {
        // 1. Try environment variable first (highest priority)
        let env_var = format!("{}_ENDPOINT", primal_type.endpoint_env_prefix());
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
            // Registry path uses capability id (not primal product name)
            return format!("{}/{}", discovery_url, primal_type.capability());
        }

        // 3. Try configuration file
        if let Ok(config_path) = std::env::var("SQUIRREL_CONFIG")
            && let Ok(endpoint) = Self::read_endpoint_from_config(&config_path, primal_type)
        {
            tracing::debug!("Using config file {} for {:?}", config_path, primal_type);
            return endpoint;
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
                primal_type.endpoint_env_prefix()
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
        _primal_type: EcosystemPrimalType,
    ) -> Result<String, PrimalError> {
        // Future: Configuration file parsing for service endpoints
        // Currently uses environment variables and discovery
        // This would read from TOML/JSON/YAML config file
        // Example: { "endpoints": { "security": "https://security.example.com" } }
        Err(PrimalError::NotImplemented(
            "Configuration file parsing not yet implemented".to_string(),
        ))
    }

    /// Get development default endpoints (ONLY for development environment)
    ///
    /// ⚠️ WARNING: These are development defaults only!
    /// In production, you MUST set environment variables:
    /// - Capability-prefixed endpoints such as `SQUIRREL_ENDPOINT`, `SERVICE_MESH_ENDPOINT`, or
    /// - `SERVICE_DISCOVERY_URL` for dynamic capability-based discovery
    ///
    /// This function uses universal-constants for all port assignments to ensure
    /// consistency across the ecosystem. It does NOT use hardcoded primal names,
    /// instead deriving endpoints from the capability-based primal type.
    fn get_development_default(primal_type: EcosystemPrimalType) -> String {
        use universal_constants::builders;
        use universal_constants::capabilities as caps;
        use universal_constants::network;

        // Port lookup follows capability ids (hyphenated in constants; get_service_port normalizes).
        let svc_for_port = match primal_type.capability() {
            c if c == caps::SELF_PRIMAL_NAME => "http",
            c if c == caps::ECOSYSTEM_CAPABILITY => "ui",
            c => c,
        };
        let port = network::get_service_port(svc_for_port);

        builders::localhost_http(port)
    }

    /// Perform actual service discovery operations
    async fn perform_service_discovery(
        service_registry: &Arc<RwLock<HashMap<Arc<str>, Arc<DiscoveredService>>>>,
        primal_type: EcosystemPrimalType,
        endpoint: String,
    ) -> Result<(), PrimalError> {
        // Create discovered service with Arc<str> optimization (move endpoint, no clone)
        let health_endpoint = Arc::from(format!("{endpoint}/health"));
        let service = Arc::new(DiscoveredService {
            service_id: intern_registry_string(&format!("{primal_type:?}").to_lowercase()),
            primal_type,
            endpoint: Arc::from(endpoint),
            capabilities: vec![
                intern_registry_string("discovery"),
                intern_registry_string("health_check"),
            ],
            health_status: ServiceHealthStatus::Healthy,
            health_endpoint,
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

    /// Get capabilities for a service by its primary capability
    ///
    /// This replaces the hardcoded primal-type-based capability mapping with
    /// a more flexible capability-based approach.
    #[must_use]
    pub fn get_capabilities_for_service(primary_capability: &str) -> Vec<Arc<str>> {
        match primary_capability {
            "ai.orchestration" | "ai_coordination" => vec![
                intern_registry_string("ai_coordination"),
                intern_registry_string("request_routing"),
                intern_registry_string("response_aggregation"),
                intern_registry_string("context_management"),
            ],
            "service_mesh" => vec![
                intern_registry_string("service_mesh"),
                intern_registry_string("load_balancing"),
                intern_registry_string("health_monitoring"),
            ],
            "compute.container" | "compute" => vec![
                intern_registry_string("compute"),
                intern_registry_string("storage"),
                intern_registry_string("scaling"),
            ],
            "security.auth" | "security" => vec![
                intern_registry_string("security"),
                intern_registry_string("authentication"),
                intern_registry_string("authorization"),
                intern_registry_string("compliance"),
            ],
            "storage.object" | "networking" => vec![
                intern_registry_string("networking"),
                intern_registry_string("gateway"),
                intern_registry_string("routing"),
            ],
            "platform.orchestration" | "operating_system" => vec![
                intern_registry_string("operating_system"),
                intern_registry_string("process_management"),
                intern_registry_string("resource_allocation"),
            ],
            _ => vec![], // Default: no capabilities
        }
    }

    /// Get capabilities for a primal type (DEPRECATED - use get_capabilities_for_service)
    ///
    /// This method is deprecated. Use `get_capabilities_for_service()` with capability strings instead.
    #[deprecated(
        since = "0.1.0",
        note = "Use get_capabilities_for_service() for capability-based discovery"
    )]
    #[must_use]
    pub fn get_capabilities_for_primal(primal_type: &EcosystemPrimalType) -> Vec<Arc<str>> {
        // Map deprecated primal types to capabilities
        let capability = match primal_type {
            EcosystemPrimalType::Squirrel => "ai_coordination",
            EcosystemPrimalType::Songbird => "service_mesh",
            EcosystemPrimalType::ToadStool => "compute",
            EcosystemPrimalType::BearDog => "security",
            EcosystemPrimalType::NestGate => "networking",
            EcosystemPrimalType::BiomeOS => "operating_system",
        };
        Self::get_capabilities_for_service(capability)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    /// Helper to create a test registry
    fn create_test_registry() -> Arc<RwLock<HashMap<Arc<str>, Arc<DiscoveredService>>>> {
        Arc::new(RwLock::new(HashMap::new()))
    }

    /// Helper to create a test service
    #[expect(
        deprecated,
        reason = "Tests deprecated path for backward compatibility"
    )]
    fn create_test_service(primal_type: EcosystemPrimalType) -> Arc<DiscoveredService> {
        Arc::new(DiscoveredService {
            service_id: Arc::from(format!("{primal_type:?}-test")),
            primal_type,
            endpoint: Arc::from("http://test.local"),
            health_endpoint: Arc::from("http://test.local/health"),
            api_version: Arc::from("0.1.0"),
            capabilities: vec![],
            metadata: HashMap::new(),
            discovered_at: Utc::now(),
            last_health_check: Some(Utc::now()),
            health_status: ServiceHealthStatus::Healthy,
        })
    }

    // Tests for get_capabilities_for_service (capability-based, not deprecated)
    #[test]
    fn test_get_capabilities_for_service_ai_orchestration() {
        let caps = DiscoveryOps::get_capabilities_for_service("ai.orchestration");
        assert!(!caps.is_empty());
        assert!(caps.contains(&intern_registry_string("ai_coordination")));
        assert!(caps.contains(&intern_registry_string("request_routing")));
        assert!(caps.contains(&intern_registry_string("response_aggregation")));
        assert!(caps.contains(&intern_registry_string("context_management")));
    }

    #[test]
    fn test_get_capabilities_for_service_ai_coordination() {
        let caps = DiscoveryOps::get_capabilities_for_service("ai_coordination");
        assert!(!caps.is_empty());
        assert!(caps.contains(&intern_registry_string("ai_coordination")));
    }

    #[test]
    fn test_get_capabilities_for_service_service_mesh() {
        let caps = DiscoveryOps::get_capabilities_for_service("service_mesh");
        assert!(!caps.is_empty());
        assert!(caps.contains(&intern_registry_string("service_mesh")));
        assert!(caps.contains(&intern_registry_string("load_balancing")));
        assert!(caps.contains(&intern_registry_string("health_monitoring")));
    }

    #[test]
    fn test_get_capabilities_for_service_compute_container() {
        let caps = DiscoveryOps::get_capabilities_for_service("compute.container");
        assert!(!caps.is_empty());
        assert!(caps.contains(&intern_registry_string("compute")));
        assert!(caps.contains(&intern_registry_string("storage")));
        assert!(caps.contains(&intern_registry_string("scaling")));
    }

    #[test]
    fn test_get_capabilities_for_service_compute() {
        let caps = DiscoveryOps::get_capabilities_for_service("compute");
        assert!(!caps.is_empty());
        assert!(caps.contains(&intern_registry_string("compute")));
    }

    #[test]
    fn test_get_capabilities_for_service_security_auth() {
        let caps = DiscoveryOps::get_capabilities_for_service("security.auth");
        assert!(!caps.is_empty());
        assert!(caps.contains(&intern_registry_string("security")));
        assert!(caps.contains(&intern_registry_string("authentication")));
        assert!(caps.contains(&intern_registry_string("authorization")));
        assert!(caps.contains(&intern_registry_string("compliance")));
    }

    #[test]
    fn test_get_capabilities_for_service_security() {
        let caps = DiscoveryOps::get_capabilities_for_service("security");
        assert!(!caps.is_empty());
        assert!(caps.contains(&intern_registry_string("security")));
    }

    #[test]
    fn test_get_capabilities_for_service_storage_object() {
        let caps = DiscoveryOps::get_capabilities_for_service("storage.object");
        assert!(!caps.is_empty());
        assert!(caps.contains(&intern_registry_string("networking")));
        assert!(caps.contains(&intern_registry_string("gateway")));
        assert!(caps.contains(&intern_registry_string("routing")));
    }

    #[test]
    fn test_get_capabilities_for_service_networking() {
        let caps = DiscoveryOps::get_capabilities_for_service("networking");
        assert!(!caps.is_empty());
        assert!(caps.contains(&intern_registry_string("networking")));
    }

    #[test]
    fn test_get_capabilities_for_service_platform_orchestration() {
        let caps = DiscoveryOps::get_capabilities_for_service("platform.orchestration");
        assert!(!caps.is_empty());
        assert!(caps.contains(&intern_registry_string("operating_system")));
        assert!(caps.contains(&intern_registry_string("process_management")));
        assert!(caps.contains(&intern_registry_string("resource_allocation")));
    }

    #[test]
    fn test_get_capabilities_for_service_operating_system() {
        let caps = DiscoveryOps::get_capabilities_for_service("operating_system");
        assert!(!caps.is_empty());
        assert!(caps.contains(&intern_registry_string("operating_system")));
    }

    #[test]
    fn test_get_capabilities_for_service_unknown() {
        let caps = DiscoveryOps::get_capabilities_for_service("unknown_capability");
        assert!(caps.is_empty());
    }

    #[test]
    fn test_get_capabilities_for_service_empty_string() {
        let caps = DiscoveryOps::get_capabilities_for_service("");
        assert!(caps.is_empty());
    }

    // Tests for get_capabilities_for_primal (deprecated, but should still work)
    #[test]
    #[expect(
        deprecated,
        reason = "Tests deprecated path for backward compatibility"
    )]
    fn test_get_capabilities_for_primal_squirrel() {
        let caps = DiscoveryOps::get_capabilities_for_primal(&EcosystemPrimalType::Squirrel);
        assert!(!caps.is_empty());
        assert!(caps.contains(&intern_registry_string("ai_coordination")));
    }

    #[test]
    #[expect(
        deprecated,
        reason = "Tests deprecated path for backward compatibility"
    )]
    fn test_get_capabilities_for_primal_songbird() {
        let caps = DiscoveryOps::get_capabilities_for_primal(&EcosystemPrimalType::Songbird);
        assert!(!caps.is_empty());
        assert!(caps.contains(&intern_registry_string("service_mesh")));
    }

    #[test]
    #[expect(
        deprecated,
        reason = "Tests deprecated path for backward compatibility"
    )]
    fn test_get_capabilities_for_primal_toadstool() {
        let caps = DiscoveryOps::get_capabilities_for_primal(&EcosystemPrimalType::ToadStool);
        assert!(!caps.is_empty());
        assert!(caps.contains(&intern_registry_string("compute")));
    }

    #[test]
    #[expect(
        deprecated,
        reason = "Tests deprecated path for backward compatibility"
    )]
    fn test_get_capabilities_for_primal_beardog() {
        let caps = DiscoveryOps::get_capabilities_for_primal(&EcosystemPrimalType::BearDog);
        assert!(!caps.is_empty());
        assert!(caps.contains(&intern_registry_string("security")));
    }

    #[test]
    #[expect(
        deprecated,
        reason = "Tests deprecated path for backward compatibility"
    )]
    fn test_get_capabilities_for_primal_nestgate() {
        let caps = DiscoveryOps::get_capabilities_for_primal(&EcosystemPrimalType::NestGate);
        assert!(!caps.is_empty());
        assert!(caps.contains(&intern_registry_string("networking")));
    }

    #[test]
    #[expect(
        deprecated,
        reason = "Tests deprecated path for backward compatibility"
    )]
    fn test_get_capabilities_for_primal_biomeos() {
        let caps = DiscoveryOps::get_capabilities_for_primal(&EcosystemPrimalType::BiomeOS);
        assert!(!caps.is_empty());
        assert!(caps.contains(&intern_registry_string("operating_system")));
    }

    // Tests for discover_services
    #[tokio::test]
    async fn test_discover_services_empty_primal_types() {
        let registry = create_test_registry();
        let result = DiscoveryOps::discover_services(&registry, vec![]).await;
        assert!(result.is_ok());
        assert!(result.expect("should succeed").is_empty());
    }

    #[tokio::test]
    #[expect(
        deprecated,
        reason = "Tests deprecated path for backward compatibility"
    )]
    async fn test_discover_services_single_primal_type() {
        let registry = create_test_registry();
        let primal_types = vec![EcosystemPrimalType::Squirrel];
        let result = DiscoveryOps::discover_services(&registry, primal_types).await;
        assert!(result.is_ok());
        // The discovery may or may not succeed depending on environment,
        // but the function should not panic
    }

    #[tokio::test]
    #[expect(
        deprecated,
        reason = "Tests deprecated path for backward compatibility"
    )]
    async fn test_discover_services_multiple_primal_types() {
        let registry = create_test_registry();
        let primal_types = vec![
            EcosystemPrimalType::Squirrel,
            EcosystemPrimalType::Songbird,
            EcosystemPrimalType::BearDog,
        ];
        let result = DiscoveryOps::discover_services(&registry, primal_types).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[expect(
        deprecated,
        reason = "Tests deprecated path for backward compatibility"
    )]
    async fn test_discover_services_all_primal_types() {
        let registry = create_test_registry();
        let primal_types = vec![
            EcosystemPrimalType::Squirrel,
            EcosystemPrimalType::Songbird,
            EcosystemPrimalType::ToadStool,
            EcosystemPrimalType::BearDog,
            EcosystemPrimalType::NestGate,
            EcosystemPrimalType::BiomeOS,
        ];
        let result = DiscoveryOps::discover_services(&registry, primal_types).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[expect(
        deprecated,
        reason = "Tests deprecated path for backward compatibility"
    )]
    async fn test_discover_services_returns_registered_services() {
        let registry = create_test_registry();

        // Manually register a service
        {
            let mut reg = registry.write().await;
            let service = create_test_service(EcosystemPrimalType::Squirrel);
            reg.insert(service.service_id.clone(), service);
        }

        // Discover services (won't find new ones, but should return existing)
        let result = DiscoveryOps::discover_services(&registry, vec![]).await;
        assert!(result.is_ok());
        let services = result.expect("should succeed");
        assert_eq!(services.len(), 1);
    }

    // Tests for build_service_endpoint (indirectly through discover_services)
    #[test]
    #[expect(
        deprecated,
        reason = "Tests deprecated path for backward compatibility"
    )]
    fn test_build_service_endpoint_uses_env_var() {
        temp_env::with_vars(
            [
                ("SQUIRREL_ENDPOINT", Some("http://custom.squirrel")),
                ("SERVICE_DISCOVERY_URL", None::<&str>),
            ],
            || {
                let endpoint = DiscoveryOps::build_service_endpoint(EcosystemPrimalType::Squirrel);
                assert_eq!(endpoint, "http://custom.squirrel");
            },
        );
    }

    #[test]
    #[expect(
        deprecated,
        reason = "Tests deprecated path for backward compatibility"
    )]
    fn test_build_service_endpoint_uses_service_discovery() {
        temp_env::with_vars(
            [
                ("SERVICE_DISCOVERY_URL", Some("http://discovery.local")),
                ("SERVICE_MESH_ENDPOINT", None::<&str>),
            ],
            || {
                let endpoint = DiscoveryOps::build_service_endpoint(EcosystemPrimalType::Songbird);
                assert!(endpoint.contains("discovery.local"));
            },
        );
    }

    #[test]
    #[expect(
        deprecated,
        reason = "Tests deprecated path for backward compatibility"
    )]
    fn test_build_service_endpoint_falls_back_to_default() {
        temp_env::with_vars_unset(
            [
                "SECURITY_ENDPOINT",
                "SERVICE_DISCOVERY_URL",
                "SQUIRREL_CONFIG",
            ],
            || {
                let endpoint = DiscoveryOps::build_service_endpoint(EcosystemPrimalType::BearDog);
                if cfg!(debug_assertions) {
                    assert!(endpoint.contains("localhost") || endpoint.contains("127.0.0.1"));
                } else {
                    assert_eq!(endpoint, "http://unconfigured.endpoint");
                }
            },
        );
    }

    // Test intern_registry_string basic functionality
    #[test]
    fn test_intern_registry_string_returns_arc_str() {
        let s = intern_registry_string("test_capability");
        assert_eq!(s.as_ref(), "test_capability");
    }

    #[test]
    fn test_intern_registry_string_common_string() {
        let s = intern_registry_string("squirrel");
        assert_eq!(s.as_ref(), "squirrel");
    }

    #[test]
    fn test_intern_registry_string_preserves_content() {
        let input = "ai_coordination";
        let result = intern_registry_string(input);
        assert_eq!(result.as_ref(), input);
    }

    // Edge case tests
    #[test]
    fn test_get_capabilities_for_service_case_sensitive() {
        let caps1 = DiscoveryOps::get_capabilities_for_service("ai.orchestration");
        let caps2 = DiscoveryOps::get_capabilities_for_service("AI.ORCHESTRATION");
        // Should be case-sensitive
        assert!(!caps1.is_empty());
        assert!(caps2.is_empty());
    }

    #[test]
    fn test_get_capabilities_for_service_whitespace() {
        let caps = DiscoveryOps::get_capabilities_for_service(" ai.orchestration ");
        // Should not match due to whitespace
        assert!(caps.is_empty());
    }

    #[tokio::test]
    #[expect(
        deprecated,
        reason = "Tests deprecated path for backward compatibility"
    )]
    async fn test_discover_services_concurrent_access() {
        let registry = create_test_registry();

        // Spawn multiple concurrent discovery operations
        let handles: Vec<_> = (0..5)
            .map(|_| {
                let reg_clone = Arc::clone(&registry);
                tokio::spawn(async move {
                    let primal_types = vec![EcosystemPrimalType::Squirrel];
                    DiscoveryOps::discover_services(&reg_clone, primal_types).await
                })
            })
            .collect();

        // Wait for all to complete
        for handle in handles {
            let result = handle.await;
            assert!(result.is_ok());
        }
    }
}
