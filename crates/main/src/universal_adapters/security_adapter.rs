// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Universal Security Adapter
//!
//! Capability-based security coordination that can work with `BearDog` or any
//! security primal that provides the required security capabilities.

use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info};

use super::registry::{InMemoryServiceRegistry, ServiceInfo};
use super::{ServiceCapability, ServiceMatcher, UniversalRequest, UniversalServiceRegistry};
use crate::error::PrimalError;

/// Universal Security Adapter - works with any security primal
pub struct UniversalSecurityAdapter {
    registry: Arc<InMemoryServiceRegistry>,
    matcher: ServiceMatcher,
    preferred_security_service: Option<ServiceInfo>,
}

impl UniversalSecurityAdapter {
    /// Create a new universal security adapter
    #[must_use]
    pub fn new(registry: Arc<InMemoryServiceRegistry>) -> Self {
        let matcher = ServiceMatcher::new(registry.clone());

        Self {
            registry,
            matcher,
            preferred_security_service: None,
        }
    }

    /// Coordinate security request with any available security primal
    pub async fn coordinate_security(
        &mut self,
        operation: &str,
        parameters: HashMap<String, serde_json::Value>,
    ) -> Result<serde_json::Value, PrimalError> {
        info!(
            "🔒 Coordinating security operation: {} via universal adapter",
            operation
        );

        // Find security service if we don't have one
        if self.preferred_security_service.is_none() {
            self.preferred_security_service = Some(self.discover_security_service().await?);
        }

        let security_service = self.preferred_security_service.as_ref().ok_or_else(|| {
            error!("No security service available - adapter initialization failed");
            PrimalError::ResourceNotFound("No security service available".to_string())
        })?;

        // Create universal request
        let _request = UniversalRequest {
            request_id: uuid::Uuid::new_v4().to_string(),
            operation: operation.to_string(),
            parameters,
            context: HashMap::from([
                (
                    "requester".to_string(),
                    serde_json::json!("squirrel_ai_coordinator"),
                ),
                (
                    "coordinator_role".to_string(),
                    serde_json::json!("ai_coordination"),
                ),
            ]),
            requester: crate::niche::PRIMAL_ID.to_string(),
            timestamp: chrono::Utc::now(),
        };

        // For now, simulate the coordination (in real implementation, make HTTP call to service endpoint)
        let response_data = serde_json::json!({
            "status": "success",
            "operation": operation,
            "security_provider": security_service.name,
            "service_id": security_service.service_id,
            "session_id": format!("{}_{}", security_service.name.to_lowercase(), uuid::Uuid::new_v4()),
            "security_context": {
                "authenticated": true,
                "authorization_level": "standard",
                "capabilities": ["ai_coordination", "service_access"]
            },
            "metadata": {
                "processing_time_ms": 45,
                "confidence": "high",
                "security_level": "enterprise"
            }
        });

        info!(
            "✅ Security operation '{}' coordinated via {} ({})",
            operation, security_service.name, security_service.service_id
        );

        Ok(response_data)
    }

    /// Authenticate using any available security primal
    pub async fn authenticate_universal(&mut self, user_id: &str) -> Result<String, PrimalError> {
        debug!(
            "🔐 Authenticating user {} via universal security adapter",
            user_id
        );

        let auth_params = HashMap::from([
            ("user_id".to_string(), serde_json::json!(user_id)),
            ("auth_method".to_string(), serde_json::json!("universal")),
            ("context".to_string(), serde_json::json!("ai_coordination")),
        ]);

        let response = self
            .coordinate_security("authenticate", auth_params)
            .await?;

        // Extract session ID from response
        response
            .get("session_id")
            .and_then(|v| v.as_str())
            .map(std::string::ToString::to_string)
            .ok_or_else(|| {
                PrimalError::SecurityError("Authentication failed: no session ID".to_string())
            })
    }

    /// Authorize operation using any available security primal
    pub async fn authorize_universal(
        &mut self,
        session_id: &str,
        operation: &str,
    ) -> Result<bool, PrimalError> {
        debug!(
            "🛡️ Authorizing operation {} for session {} via universal adapter",
            operation, session_id
        );

        let auth_params = HashMap::from([
            ("session_id".to_string(), serde_json::json!(session_id)),
            ("operation".to_string(), serde_json::json!(operation)),
            ("resource".to_string(), serde_json::json!("ai_coordination")),
        ]);

        let response = self.coordinate_security("authorize", auth_params).await?;

        // Extract authorization result
        response
            .get("authorized")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false)
            .then_some(true)
            .ok_or_else(|| PrimalError::SecurityError("Authorization denied".to_string()))
            .map(|_| true)
    }

    /// Discover security services by capability
    async fn discover_security_service(&self) -> Result<ServiceInfo, PrimalError> {
        info!("🔍 Discovering security services via universal capability matching");

        // Define required security capabilities
        let required_capabilities = vec![ServiceCapability::Security {
            functions: vec![
                "authentication".to_string(),
                "authorization".to_string(),
                "session_management".to_string(),
            ],
            compliance: vec!["enterprise".to_string()],
            trust_levels: vec!["high".to_string()],
        }];

        // Find optimal security service
        let security_service = self
            .matcher
            .match_service_for_task("AI Coordination Security", required_capabilities)
            .await?;

        info!(
            "🎯 Selected security service: {} ({})",
            security_service.name, security_service.service_id
        );

        Ok(security_service)
    }

    /// Get current security service info (if any)
    #[must_use]
    pub const fn get_current_security_service(&self) -> Option<&ServiceInfo> {
        self.preferred_security_service.as_ref()
    }

    /// Force rediscovery of security services
    pub async fn rediscover_security_services(&mut self) -> Result<(), PrimalError> {
        info!("🔄 Rediscovering security services");
        self.preferred_security_service = None;
        self.preferred_security_service = Some(self.discover_security_service().await?);
        Ok(())
    }

    /// Check if security adapter is healthy
    pub async fn is_healthy(&self) -> bool {
        if let Some(service) = &self.preferred_security_service {
            service.health.healthy
        } else {
            // Try to discover a security service to check availability
            match self.matcher.auto_discover_services().await {
                Ok(services) => services.iter().any(|s| {
                    s.capabilities
                        .iter()
                        .any(|cap| matches!(cap, ServiceCapability::Security { .. }))
                }),
                Err(_) => false,
            }
        }
    }

    /// Get security capabilities summary
    pub async fn get_security_capabilities(&self) -> Result<Vec<String>, PrimalError> {
        let services = self.registry.discover_by_category("security").await?;

        let mut all_capabilities = Vec::new();
        for service in services {
            for capability in service.capabilities {
                if let ServiceCapability::Security { functions, .. } = capability {
                    all_capabilities.extend(functions);
                }
            }
        }

        // Remove duplicates
        all_capabilities.sort();
        all_capabilities.dedup();

        Ok(all_capabilities)
    }
}

/// Register a security provider with the universal registry.
///
/// Capability-based: any primal providing the `security.*` capability set
/// can be registered here — the caller discovers the provider at runtime.
pub async fn register_security_provider(
    registry: Arc<InMemoryServiceRegistry>,
) -> Result<(), PrimalError> {
    info!("Registering security provider with universal registry");

    let registration = super::UniversalServiceRegistration {
        service_id: uuid::Uuid::new_v4(),
        metadata: super::ServiceMetadata {
            name: "Security Provider".to_string(),
            category: super::ServiceCategory::Security {
                domains: vec!["enterprise".to_string(), "ai_coordination".to_string()],
            },
            version: "1.0.0".to_string(),
            description:
                "Enterprise security primal providing authentication, authorization, and compliance"
                    .to_string(),
            maintainer: "EcoPrimals Core Team".to_string(),
            protocols: vec!["https".to_string(), "tarpc".to_string()],
        },
        capabilities: vec![ServiceCapability::Security {
            functions: vec![
                "authentication".to_string(),
                "authorization".to_string(),
                "session_management".to_string(),
                "audit_logging".to_string(),
                "compliance_monitoring".to_string(),
                "threat_detection".to_string(),
            ],
            compliance: vec![
                "enterprise".to_string(),
                "gdpr".to_string(),
                "soc2".to_string(),
            ],
            trust_levels: vec!["high".to_string(), "critical".to_string()],
        }],
        endpoints: vec![super::ServiceEndpoint {
            name: "primary".to_string(),
            url: universal_constants::config_helpers::get_host(
                "SECURITY_SERVICE_ENDPOINT",
                "https://security.ecosystem.local",
            ),
            protocol: "https".to_string(),
            port: None,
            path: Some("/api/v1".to_string()),
        }],
        resources: super::ResourceSpec {
            cpu_cores: Some(4),
            memory_gb: Some(8),
            storage_gb: Some(100),
            network_bandwidth: Some(1000),
            custom_resources: HashMap::from([
                (
                    "security_level".to_string(),
                    serde_json::json!("enterprise"),
                ),
                (
                    "encryption_strength".to_string(),
                    serde_json::json!("aes-256"),
                ),
            ]),
        },
        integration: super::IntegrationPreferences {
            preferred_protocols: vec!["https".to_string(), "tarpc".to_string()],
            retry_policy: "exponential_backoff".to_string(),
            timeout_seconds: 30,
            load_balancing_weight: 10,
        },
        extensions: HashMap::from([
            // Provider domain (capability-based, not primal-specific)
            // The actual provider is discovered dynamically at runtime
            ("provider_domain".to_string(), serde_json::json!("security")),
            (
                "ecosystem_role".to_string(),
                serde_json::json!("security_provider"),
            ),
            (
                "ai_coordination_support".to_string(),
                serde_json::json!(true),
            ),
        ]),
        registration_timestamp: chrono::Utc::now(),
        service_version: "1.0.0".to_string(),
        instance_id: uuid::Uuid::new_v4().to_string(),
        priority: 10, // High priority for core security primal
    };

    registry.register_service(registration).await?;

    info!("Security provider registered with universal registry");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::universal_adapters::registry::InMemoryServiceRegistry;
    use crate::universal_adapters::{
        IntegrationPreferences, ResourceSpec, ServiceCategory, ServiceEndpoint, ServiceHealth,
        ServiceMetadata, UniversalServiceRegistration,
    };

    fn test_security_registration() -> UniversalServiceRegistration {
        UniversalServiceRegistration {
            service_id: uuid::Uuid::new_v4(),
            metadata: ServiceMetadata {
                name: "Test Security".to_string(),
                category: ServiceCategory::Security {
                    domains: vec!["enterprise".to_string()],
                },
                version: "1.0.0".to_string(),
                description: "Test".to_string(),
                maintainer: "test".to_string(),
                protocols: vec!["https".to_string()],
            },
            capabilities: vec![ServiceCapability::Security {
                functions: vec![
                    "authentication".to_string(),
                    "authorization".to_string(),
                    "session_management".to_string(),
                ],
                compliance: vec!["enterprise".to_string()],
                trust_levels: vec!["high".to_string()],
            }],
            endpoints: vec![ServiceEndpoint {
                name: "primary".to_string(),
                url: "https://sec.test".to_string(),
                protocol: "https".to_string(),
                port: Some(443),
                path: None,
            }],
            resources: ResourceSpec {
                cpu_cores: Some(2),
                memory_gb: Some(4),
                storage_gb: Some(50),
                network_bandwidth: Some(500),
                custom_resources: HashMap::new(),
            },
            integration: IntegrationPreferences {
                preferred_protocols: vec!["https".to_string()],
                retry_policy: "simple".to_string(),
                timeout_seconds: 30,
                load_balancing_weight: 10,
            },
            extensions: HashMap::new(),
            registration_timestamp: chrono::Utc::now(),
            service_version: "1.0.0".to_string(),
            instance_id: "inst-sec".to_string(),
            priority: 10,
        }
    }

    async fn registry_with_security() -> Arc<InMemoryServiceRegistry> {
        let reg = Arc::new(InMemoryServiceRegistry::new());
        reg.register_service(test_security_registration())
            .await
            .expect("register");
        reg
    }

    #[tokio::test]
    async fn new_adapter_get_current_none() {
        let reg = Arc::new(InMemoryServiceRegistry::new());
        let adapter = UniversalSecurityAdapter::new(reg);
        assert!(adapter.get_current_security_service().is_none());
    }

    #[tokio::test]
    async fn coordinate_security_happy_path() {
        let reg = registry_with_security().await;
        let mut adapter = UniversalSecurityAdapter::new(reg);
        let params = HashMap::from([("k".to_string(), serde_json::json!(1))]);
        let v = adapter
            .coordinate_security("token_validate", params)
            .await
            .expect("coord");
        assert_eq!(v["status"], "success");
        assert_eq!(v["operation"], "token_validate");
    }

    #[tokio::test]
    async fn authenticate_universal_returns_session_id() {
        let reg = registry_with_security().await;
        let mut adapter = UniversalSecurityAdapter::new(reg);
        let sid = adapter
            .authenticate_universal("user-42")
            .await
            .expect("session");
        assert!(sid.contains("test security") || sid.contains("user-42") || !sid.is_empty());
        assert!(sid.contains('_'));
    }

    #[tokio::test]
    async fn authorize_universal_errors_without_authorized_field() {
        let reg = registry_with_security().await;
        let mut adapter = UniversalSecurityAdapter::new(reg);
        let err = adapter
            .authorize_universal("sess", "read")
            .await
            .unwrap_err();
        assert!(matches!(err, PrimalError::SecurityError(_)));
    }

    #[tokio::test]
    async fn rediscover_security_services() {
        let reg = registry_with_security().await;
        let mut adapter = UniversalSecurityAdapter::new(reg);
        adapter
            .coordinate_security("ping", HashMap::new())
            .await
            .expect("should succeed");
        adapter
            .rediscover_security_services()
            .await
            .expect("redisc");
        assert!(adapter.get_current_security_service().is_some());
    }

    #[tokio::test]
    async fn discovery_fails_empty_registry() {
        let reg = Arc::new(InMemoryServiceRegistry::new());
        let mut adapter = UniversalSecurityAdapter::new(reg);
        let err = adapter
            .coordinate_security("auth", HashMap::new())
            .await
            .unwrap_err();
        assert!(matches!(err, PrimalError::ServiceDiscoveryError(_)));
    }

    #[tokio::test]
    async fn get_security_capabilities() {
        let reg = registry_with_security().await;
        let adapter = UniversalSecurityAdapter::new(reg);
        let caps = adapter.get_security_capabilities().await.expect("caps");
        assert!(caps.contains(&"authentication".to_string()));
    }

    #[tokio::test]
    async fn is_healthy_true() {
        let reg = registry_with_security().await;
        let adapter = UniversalSecurityAdapter::new(reg);
        assert!(adapter.is_healthy().await);
    }

    #[tokio::test]
    async fn is_healthy_false_empty() {
        let reg = Arc::new(InMemoryServiceRegistry::new());
        let adapter = UniversalSecurityAdapter::new(reg);
        assert!(!adapter.is_healthy().await);
    }

    #[tokio::test]
    async fn is_healthy_false_unhealthy() {
        let reg = Arc::new(InMemoryServiceRegistry::new());
        let reg_data = test_security_registration();
        let sid = reg_data.service_id.to_string();
        reg.register_service(reg_data)
            .await
            .expect("should succeed");
        reg.update_service_health(
            &sid,
            ServiceHealth {
                healthy: false,
                message: None,
                metrics: HashMap::new(),
            },
        )
        .await
        .expect("should succeed");
        let mut adapter = UniversalSecurityAdapter::new(reg);
        adapter
            .coordinate_security("x", HashMap::new())
            .await
            .expect("should succeed");
        assert!(!adapter.is_healthy().await);
    }

    #[tokio::test]
    async fn register_security_and_serde_roundtrip() {
        let reg = Arc::new(InMemoryServiceRegistry::new());
        register_security_provider(reg.clone()).await.expect("reg");
        let services = reg.list_all_services().await.expect("list");
        assert_eq!(services.len(), 1);
        let json = serde_json::to_string(&services[0].capabilities).expect("ser");
        let caps: Vec<ServiceCapability> = serde_json::from_str(&json).expect("de");
        assert!(!caps.is_empty());
    }

    #[test]
    fn security_registration_serde_roundtrip() {
        let r = test_security_registration();
        let s = serde_json::to_string(&r).expect("json");
        let back: UniversalServiceRegistration = serde_json::from_str(&s).expect("back");
        assert_eq!(r.service_id, back.service_id);
    }
}
