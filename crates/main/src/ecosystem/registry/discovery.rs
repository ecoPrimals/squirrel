// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors
// Backward compatibility: discover_services/DiscoveredService use EcosystemPrimalType for legacy format
#![allow(deprecated)]

//! Service discovery operations for the ecosystem registry

use super::types::{DiscoveredService, ServiceHealthStatus, intern_registry_string};
use crate::ecosystem::{CapabilityIdentifier, EcosystemPrimalType};
use crate::error::PrimalError;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Discovery operations for the ecosystem registry
pub struct DiscoveryOps;

impl DiscoveryOps {
    /// Discover services by capability domain (primary API).
    ///
    /// When `capabilities` is empty, probes all available Unix sockets via
    /// `capability.discover` and registers every provider found.
    pub async fn discover_services(
        service_registry: &Arc<RwLock<HashMap<Arc<str>, Arc<DiscoveredService>>>>,
        capabilities: Vec<CapabilityIdentifier>,
    ) -> Result<Vec<Arc<DiscoveredService>>, PrimalError> {
        if capabilities.is_empty() {
            Self::discover_from_capability_registry(service_registry).await?;
        } else {
            for capability in capabilities {
                if let Err(e) = Self::discover_capability(service_registry, &capability).await {
                    tracing::error!(
                        "Failed to discover provider for capability '{}': {e}",
                        capability.as_str()
                    );
                }
            }
        }

        let registry = service_registry.read().await;
        Ok(registry.values().cloned().collect())
    }

    /// Discover all providers by scanning sockets with `capability.discover`.
    async fn discover_from_capability_registry(
        service_registry: &Arc<RwLock<HashMap<Arc<str>, Arc<DiscoveredService>>>>,
    ) -> Result<(), PrimalError> {
        let providers = crate::capabilities::discovery::discover_all_capabilities()
            .await
            .map_err(|e| {
                PrimalError::OperationFailed(format!("Capability registry scan failed: {e}"))
            })?;

        for (capability, provider_list) in providers {
            for provider in provider_list {
                let socket_str = provider.socket.display().to_string();
                let endpoint = format!("unix://{socket_str}");
                let caps: Vec<&str> = provider
                    .capabilities
                    .iter()
                    .map(std::string::String::as_str)
                    .collect();
                let metadata = provider
                    .metadata
                    .iter()
                    .map(|(k, v)| (k.as_str(), v.as_str()))
                    .collect();
                Self::register_discovered_service(
                    service_registry,
                    &provider.id,
                    &capability,
                    &endpoint,
                    caps,
                    metadata,
                )
                .await?;
            }
        }
        Ok(())
    }

    /// Discover a single capability domain via runtime probes, then env/config fallback.
    async fn discover_capability(
        service_registry: &Arc<RwLock<HashMap<Arc<str>, Arc<DiscoveredService>>>>,
        capability: &CapabilityIdentifier,
    ) -> Result<(), PrimalError> {
        let cap_str = capability.as_str();

        if let Ok(provider) = crate::capabilities::discovery::discover_capability(cap_str).await {
            let socket_str = provider.socket.display().to_string();
            let endpoint = format!("unix://{socket_str}");
            let caps: Vec<&str> = provider
                .capabilities
                .iter()
                .map(std::string::String::as_str)
                .collect();
            let metadata = provider
                .metadata
                .iter()
                .map(|(k, v)| (k.as_str(), v.as_str()))
                .collect();
            return Self::register_discovered_service(
                service_registry,
                &provider.id,
                cap_str,
                &endpoint,
                caps,
                metadata,
            )
            .await;
        }

        let endpoint = Self::build_service_endpoint(cap_str);
        Self::perform_service_discovery(service_registry, cap_str, endpoint).await
    }

    async fn register_discovered_service(
        service_registry: &Arc<RwLock<HashMap<Arc<str>, Arc<DiscoveredService>>>>,
        service_id: &str,
        primary_capability: &str,
        endpoint: &str,
        capabilities: Vec<&str>,
        metadata: HashMap<&str, &str>,
    ) -> Result<(), PrimalError> {
        let service = Arc::new(DiscoveredService::new(
            service_id,
            primary_capability,
            endpoint,
            endpoint,
            "v1",
            capabilities,
            metadata,
        ));
        service_registry
            .write()
            .await
            .insert(service.service_id.clone(), service);
        Ok(())
    }

    /// Discover services by deprecated primal type list (backward compatibility).
    #[deprecated(
        since = "0.2.0",
        note = "Use discover_services with CapabilityIdentifier instead of hardcoded primal types"
    )]
    #[allow(deprecated)]
    pub async fn discover_services_by_primal_types(
        service_registry: &Arc<RwLock<HashMap<Arc<str>, Arc<DiscoveredService>>>>,
        primal_types: Vec<EcosystemPrimalType>,
    ) -> Result<Vec<Arc<DiscoveredService>>, PrimalError> {
        let capabilities: Vec<_> = primal_types
            .into_iter()
            .map(|t| CapabilityIdentifier::new(t.capability()))
            .collect();
        Self::discover_services(service_registry, capabilities).await
    }

    /// Build service endpoint from a capability domain string.
    ///
    /// ## Discovery Priority (Highest to Lowest)
    ///
    /// 1. **Environment Variables** — `{CAPABILITY_PREFIX}_ENDPOINT`
    /// 2. **Service Discovery Systems** — `SERVICE_DISCOVERY_URL` + capability path
    /// 3. **Configuration File** — `[endpoints]` keyed by capability domain
    /// 4. **Development Defaults** (debug builds only)
    fn build_service_endpoint(capability: &str) -> String {
        let env_prefix = CapabilityIdentifier::new(capability).endpoint_env_prefix();
        let env_var = format!("{env_prefix}_ENDPOINT");
        if let Ok(endpoint) = std::env::var(&env_var) {
            tracing::debug!("Using environment variable {env_var} for capability {capability}");
            return endpoint;
        }

        if let Ok(discovery_url) =
            std::env::var(universal_constants::env_vars::discovery::SERVICE_DISCOVERY_URL)
        {
            tracing::debug!(
                "Using service discovery at {discovery_url} for capability {capability}"
            );
            return format!("{discovery_url}/{capability}");
        }

        if let Ok(config_path) = std::env::var(universal_constants::env_vars::squirrel::CONFIG)
            && let Ok(endpoint) = Self::read_endpoint_from_config(&config_path, capability)
        {
            tracing::debug!("Using config file {config_path} for capability {capability}");
            return endpoint;
        }

        if cfg!(debug_assertions) {
            tracing::warn!(
                "Using development default for capability {capability} — set env vars in production!"
            );
            Self::get_development_default(capability)
        } else {
            tracing::error!(
                "No endpoint configured for capability {capability} — set {env_var} or SERVICE_DISCOVERY_URL"
            );
            "http://unconfigured.endpoint".to_string()
        }
    }

    /// Read endpoint from a TOML configuration file.
    ///
    /// Expected format:
    /// ```toml
    /// [endpoints]
    /// security = "https://security.example.com"
    /// orchestration = "http://localhost:8082"
    /// ```
    fn read_endpoint_from_config(
        config_path: &str,
        capability: &str,
    ) -> Result<String, PrimalError> {
        let contents = std::fs::read_to_string(config_path).map_err(|e| {
            PrimalError::Configuration(format!("Cannot read config {config_path}: {e}"))
        })?;
        let table: toml::Table = contents.parse().map_err(|e| {
            PrimalError::Configuration(format!("Invalid TOML in {config_path}: {e}"))
        })?;
        let endpoints = table
            .get("endpoints")
            .and_then(|v| v.as_table())
            .ok_or_else(|| {
                PrimalError::Configuration(format!("No [endpoints] table in {config_path}"))
            })?;
        let key = capability;
        endpoints
            .get(key)
            .and_then(|v| v.as_str())
            .map(String::from)
            .ok_or_else(|| {
                PrimalError::Configuration(format!(
                    "No endpoint for capability '{key}' in {config_path}"
                ))
            })
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
    fn get_development_default(capability: &str) -> String {
        use universal_constants::builders;
        use universal_constants::capabilities as caps;
        use universal_constants::network;

        let svc_for_port = match capability {
            c if c == caps::SELF_PRIMAL_NAME => "http",
            c if c == caps::ECOSYSTEM_CAPABILITY => "ui",
            other => other,
        };
        let port = network::get_service_port(svc_for_port);

        builders::localhost_http(port)
    }

    /// Register a service discovered via env/config fallback (no live socket probe).
    #[cfg_attr(test, allow(dead_code))]
    pub(crate) async fn perform_service_discovery(
        service_registry: &Arc<RwLock<HashMap<Arc<str>, Arc<DiscoveredService>>>>,
        primary_capability: &str,
        endpoint: String,
    ) -> Result<(), PrimalError> {
        let health_endpoint = Arc::from(format!("{endpoint}/health"));
        let service = Arc::new(DiscoveredService {
            service_id: intern_registry_string(primary_capability),
            primary_capability: intern_registry_string(primary_capability),
            #[allow(deprecated)]
            primal_type: crate::ecosystem::infer_primal_type_from_capability(primary_capability),
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
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to create a test registry
    fn create_test_registry() -> Arc<RwLock<HashMap<Arc<str>, Arc<DiscoveredService>>>> {
        Arc::new(RwLock::new(HashMap::new()))
    }

    /// Helper to create a test service
    fn create_test_service(primary_capability: &str) -> Arc<DiscoveredService> {
        Arc::new(DiscoveredService::new(
            &format!("{primary_capability}-test"),
            primary_capability,
            "http://test.local",
            "http://test.local/health",
            "0.1.0",
            vec![],
            HashMap::new(),
        ))
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

    // Tests for discover_services registry reads (no live I/O).
    //
    // Previous versions called discover_services with live socket probes,
    // hitting 10s timeouts per capability (40 caps x 10s = 400s).
    // These tests now verify the registry read/write contract directly,
    // which is the actual business logic under test.

    #[tokio::test]
    async fn test_prepopulated_registry_returns_services() {
        let registry = create_test_registry();
        {
            let mut reg = registry.write().await;
            reg.insert(
                intern_registry_string("squirrel"),
                create_test_service("squirrel"),
            );
            reg.insert(
                intern_registry_string("security"),
                create_test_service("security"),
            );
        }
        let reg = registry.read().await;
        assert_eq!(reg.len(), 2);
        assert!(reg.contains_key("squirrel"));
        assert!(reg.contains_key("security"));
    }

    #[tokio::test]
    async fn test_empty_registry_read_returns_empty() {
        let registry = create_test_registry();
        let reg = registry.read().await;
        assert!(reg.is_empty());
    }

    #[tokio::test]
    async fn test_perform_service_discovery_registers_service() {
        let registry = create_test_registry();
        DiscoveryOps::perform_service_discovery(
            &registry,
            "squirrel",
            "http://localhost:9000".to_string(),
        )
        .await
        .expect("register should succeed");

        let reg = registry.read().await;
        assert_eq!(reg.len(), 1);
        let svc = reg.get("squirrel").expect("squirrel registered");
        assert_eq!(svc.endpoint.as_ref(), "http://localhost:9000");
    }

    #[test]
    fn test_all_consumed_capabilities_are_qualified() {
        for cap in crate::niche::CONSUMED_CAPABILITIES {
            assert!(
                cap.contains('.'),
                "consumed capability '{cap}' must be domain.method format"
            );
        }
    }

    #[test]
    fn test_build_service_endpoint_uses_env_var() {
        temp_env::with_vars(
            [
                ("SQUIRREL_ENDPOINT", Some("http://custom.squirrel")),
                ("SERVICE_DISCOVERY_URL", None::<&str>),
            ],
            || {
                let endpoint = DiscoveryOps::build_service_endpoint("squirrel");
                assert_eq!(endpoint, "http://custom.squirrel");
            },
        );
    }

    #[test]
    fn test_build_service_endpoint_uses_service_discovery() {
        temp_env::with_vars(
            [
                ("SERVICE_DISCOVERY_URL", Some("http://discovery.local")),
                ("SERVICE_MESH_ENDPOINT", None::<&str>),
            ],
            || {
                let endpoint = DiscoveryOps::build_service_endpoint("service-mesh");
                assert!(endpoint.contains("discovery.local"));
            },
        );
    }

    #[test]
    fn test_build_service_endpoint_falls_back_to_default() {
        temp_env::with_vars_unset(
            [
                "SECURITY_ENDPOINT",
                "SERVICE_DISCOVERY_URL",
                "SQUIRREL_CONFIG",
            ],
            || {
                let endpoint = DiscoveryOps::build_service_endpoint("security");
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
    async fn test_registry_concurrent_read_write() {
        let registry = create_test_registry();

        // Prepopulate
        {
            let mut reg = registry.write().await;
            for i in 0..10 {
                let name = format!("svc-{i}");
                reg.insert(intern_registry_string(&name), create_test_service(&name));
            }
        }

        // 5 concurrent readers verifying snapshot consistency
        let handles: Vec<_> = (0..5)
            .map(|_| {
                let reg_clone = Arc::clone(&registry);
                tokio::spawn(async move {
                    let reg = reg_clone.read().await;
                    assert_eq!(reg.len(), 10);
                })
            })
            .collect();

        for handle in handles {
            handle.await.expect("reader should not panic");
        }
    }
}
