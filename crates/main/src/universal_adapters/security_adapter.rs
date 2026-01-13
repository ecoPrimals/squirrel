//! Universal Security Adapter
//!
//! Capability-based security coordination that can work with `BearDog` or any
//! security primal that provides the required security capabilities.

use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info};

use super::registry::ServiceInfo;
use super::{ServiceCapability, ServiceMatcher, UniversalRequest, UniversalServiceRegistry};
use crate::error::PrimalError;

/// Universal Security Adapter - works with any security primal
pub struct UniversalSecurityAdapter {
    registry: Arc<dyn UniversalServiceRegistry>,
    matcher: ServiceMatcher,
    preferred_security_service: Option<ServiceInfo>,
}

impl UniversalSecurityAdapter {
    /// Create a new universal security adapter
    pub fn new(registry: Arc<dyn UniversalServiceRegistry>) -> Self {
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
        let request = UniversalRequest {
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
            requester: "squirrel".to_string(),
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
    pub fn get_current_security_service(&self) -> Option<&ServiceInfo> {
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

/// Register `BearDog` (or any security primal) with the universal registry
pub async fn register_beardog_service(
    registry: Arc<dyn UniversalServiceRegistry>,
) -> Result<(), PrimalError> {
    info!("🐻 Registering BearDog security service with universal registry");

    let registration = super::UniversalServiceRegistration {
        service_id: uuid::Uuid::new_v4(),
        metadata: super::ServiceMetadata {
            name: "BearDog Security Primal".to_string(),
            category: super::ServiceCategory::Security {
                domains: vec!["enterprise".to_string(), "ai_coordination".to_string()],
            },
            version: "1.0.0".to_string(),
            description:
                "Enterprise security primal providing authentication, authorization, and compliance"
                    .to_string(),
            maintainer: "EcoPrimals Core Team".to_string(),
            protocols: vec!["https".to_string(), "grpc".to_string()],
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
            url: "https://beardog.ecosystem.local".to_string(),
            protocol: "https".to_string(),
            port: Some(443),
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
            preferred_protocols: vec!["https".to_string(), "grpc".to_string()],
            retry_policy: "exponential_backoff".to_string(),
            timeout_seconds: 30,
            load_balancing_weight: 10,
        },
        extensions: HashMap::from([
            // Note: This is registry metadata, not a hardcoded dependency
            // The actual provider is discovered dynamically at runtime
            ("primal_type".to_string(), serde_json::json!("beardog")),
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

    info!("✅ BearDog security service successfully registered with universal registry");
    Ok(())
}
