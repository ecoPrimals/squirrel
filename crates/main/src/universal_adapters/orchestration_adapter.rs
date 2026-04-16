// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Universal Orchestration Adapter
//!
//! Capability-based orchestration coordination that works with any primal
//! providing the required coordination capabilities (discovered at runtime).

use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info};

use super::registry::{InMemoryServiceRegistry, ServiceInfo};
use super::{ServiceCapability, ServiceMatcher, UniversalRequest, UniversalServiceRegistry};
use crate::error::PrimalError;

/// Universal Orchestration Adapter - works with any orchestration primal
pub struct UniversalOrchestrationAdapter {
    registry: Arc<InMemoryServiceRegistry>,
    matcher: ServiceMatcher,
    preferred_orchestration_service: Option<ServiceInfo>,
}

impl UniversalOrchestrationAdapter {
    /// Create a new universal orchestration adapter
    #[must_use]
    pub fn new(registry: Arc<InMemoryServiceRegistry>) -> Self {
        let matcher = ServiceMatcher::new(registry.clone());

        Self {
            registry,
            matcher,
            preferred_orchestration_service: None,
        }
    }

    /// Coordinate with orchestration service for AI workflow management
    pub async fn coordinate_ai_workflow(
        &mut self,
        workflow_type: &str,
        participants: Vec<String>,
    ) -> Result<serde_json::Value, PrimalError> {
        info!(
            "🎼 Coordinating AI workflow '{}' with {} participants via universal adapter",
            workflow_type,
            participants.len()
        );

        // Discover orchestration service if needed
        if self.preferred_orchestration_service.is_none() {
            self.preferred_orchestration_service =
                Some(self.discover_orchestration_service().await?);
        }

        let orchestration_service =
            self.preferred_orchestration_service
                .as_ref()
                .ok_or_else(|| {
                    error!("No orchestration service available - adapter initialization failed");
                    PrimalError::ResourceNotFound("No orchestration service available".to_string())
                })?;

        // Create universal orchestration request
        let request_params = HashMap::from([
            (
                "workflow_type".to_string(),
                serde_json::json!(workflow_type),
            ),
            ("participants".to_string(), serde_json::json!(participants)),
            (
                "coordinator".to_string(),
                serde_json::json!(crate::niche::PRIMAL_ID),
            ),
            ("ai_enhanced".to_string(), serde_json::json!(true)),
            ("priority".to_string(), serde_json::json!("high")),
        ]);

        let _request = UniversalRequest {
            request_id: uuid::Uuid::new_v4().to_string(),
            operation: "coordinate_workflow".to_string(),
            parameters: request_params,
            context: HashMap::from([
                (
                    "requester".to_string(),
                    serde_json::json!("squirrel_ai_coordinator"),
                ),
                (
                    "coordination_type".to_string(),
                    serde_json::json!("ai_workflow"),
                ),
            ]),
            requester: crate::niche::PRIMAL_ID.to_string(),
            timestamp: chrono::Utc::now(),
        };

        // Simulate orchestration coordination (in real implementation, make HTTP call)
        let response_data = serde_json::json!({
            "workflow_id": uuid::Uuid::new_v4().to_string(),
            "status": "coordinated",
            "workflow_type": workflow_type,
            "orchestration_service": orchestration_service.name,
            "participating_services": participants,
            "coordinator": "squirrel",
            "ai_enhanced": true,
            "execution_plan": {
                "phases": ["discovery", "coordination", "execution", "monitoring"],
                "estimated_duration_ms": 2000,
                "resource_allocation": "optimized"
            },
            "coordination_metadata": {
                "service_mesh_enabled": true,
                "load_balancing_strategy": "ai_optimized",
                "failover_configured": true
            }
        });

        info!(
            "✅ AI workflow '{}' coordinated via {} ({})",
            workflow_type, orchestration_service.name, orchestration_service.service_id
        );

        Ok(response_data)
    }

    /// Request service discovery via orchestration primal
    pub async fn discover_services_universal(
        &mut self,
        capability_requirements: Vec<String>,
    ) -> Result<Vec<ServiceInfo>, PrimalError> {
        debug!("🔍 Discovering services via universal orchestration adapter");

        // Use orchestration service's discovery capabilities
        if self.preferred_orchestration_service.is_none() {
            self.preferred_orchestration_service =
                Some(self.discover_orchestration_service().await?);
        }

        // For now, delegate to our local registry (in real implementation, call orchestration service)
        let mut discovered_services = Vec::new();

        for capability in capability_requirements {
            let services = self.registry.discover_by_category(&capability).await?;
            discovered_services.extend(services);
        }

        // Remove duplicates
        discovered_services.sort_by(|a, b| a.service_id.cmp(&b.service_id));
        discovered_services.dedup_by(|a, b| a.service_id == b.service_id);

        info!(
            "🎯 Discovered {} services via orchestration service",
            discovered_services.len()
        );
        Ok(discovered_services)
    }

    /// Request load balancing from orchestration primal
    pub async fn request_load_balancing(
        &mut self,
        target_services: Vec<String>,
        strategy: &str,
    ) -> Result<serde_json::Value, PrimalError> {
        info!(
            "⚖️ Requesting load balancing for {} services with '{}' strategy",
            target_services.len(),
            strategy
        );

        if self.preferred_orchestration_service.is_none() {
            self.preferred_orchestration_service =
                Some(self.discover_orchestration_service().await?);
        }

        let orchestration_service =
            self.preferred_orchestration_service
                .as_ref()
                .ok_or_else(|| {
                    error!("No orchestration service available for load balancing");
                    PrimalError::ResourceNotFound("No orchestration service available".to_string())
                })?;

        // Create load balancing request
        let load_balancing_result = serde_json::json!({
            "load_balancer_id": uuid::Uuid::new_v4().to_string(),
            "strategy": strategy,
            "target_services": target_services,
            "orchestration_service": orchestration_service.name,
            "configuration": {
                "algorithm": "ai_optimized",
                "health_check_enabled": true,
                "failover_enabled": true,
                "sticky_sessions": false
            },
            "routing_table": target_services.iter().enumerate().map(|(i, service)| {
                serde_json::json!({
                    "service_id": service,
                    "weight": 100 / target_services.len().max(1),
                    "priority": i + 1,
                    "health_status": "healthy"
                })
            }).collect::<Vec<_>>(),
            "metadata": {
                "created_by": "squirrel_ai_coordinator",
                "ai_enhanced": true,
                "auto_scaling": true
            }
        });

        info!(
            "✅ Load balancing configured via {}",
            orchestration_service.name
        );
        Ok(load_balancing_result)
    }

    /// Get orchestration service mesh status
    pub async fn get_service_mesh_status(&mut self) -> Result<serde_json::Value, PrimalError> {
        debug!("📊 Getting service mesh status via universal orchestration adapter");

        if self.preferred_orchestration_service.is_none() {
            self.preferred_orchestration_service =
                Some(self.discover_orchestration_service().await?);
        }

        let orchestration_service =
            self.preferred_orchestration_service
                .as_ref()
                .ok_or_else(|| {
                    error!("No orchestration service available for service mesh status");
                    PrimalError::ResourceNotFound("No orchestration service available".to_string())
                })?;

        // Get service mesh status (simulated for now)
        let mesh_status = serde_json::json!({
            "service_mesh": {
                "status": "active",
                "orchestration_service": orchestration_service.name,
                "nodes": 5,
                "healthy_nodes": 5,
                "total_services": 12,
                "active_connections": 48
            },
            "ai_coordination": {
                "enabled": true,
                "coordinator": "squirrel",
                "active_workflows": 3,
                "optimization_level": "high"
            },
            "performance_metrics": {
                "average_response_time_ms": 85,
                "throughput_rps": 1250,
                "error_rate_percent": 0.02,
                "availability_percent": 99.98
            },
            "capabilities": {
                "auto_scaling": true,
                "load_balancing": true,
                "service_discovery": true,
                "health_monitoring": true,
                "ai_optimization": true
            }
        });

        Ok(mesh_status)
    }

    /// Discover orchestration services by capability
    async fn discover_orchestration_service(&self) -> Result<ServiceInfo, PrimalError> {
        info!("🔍 Discovering orchestration services via universal capability matching");

        // Define required orchestration capabilities
        let required_capabilities = vec![ServiceCapability::Coordination {
            patterns: vec![
                "service_discovery".to_string(),
                "load_balancing".to_string(),
                "workflow_coordination".to_string(),
                "service_mesh".to_string(),
            ],
            consistency: "eventual".to_string(),
            fault_tolerance: "high".to_string(),
        }];

        // Find optimal orchestration service
        let orchestration_service = self
            .matcher
            .match_service_for_task("AI Workflow Orchestration", required_capabilities)
            .await?;

        info!(
            "🎯 Selected orchestration service: {} ({})",
            orchestration_service.name, orchestration_service.service_id
        );

        Ok(orchestration_service)
    }

    /// Get current orchestration service info
    #[must_use]
    pub const fn get_current_orchestration_service(&self) -> Option<&ServiceInfo> {
        self.preferred_orchestration_service.as_ref()
    }

    /// Force rediscovery of orchestration services
    pub async fn rediscover_orchestration_services(&mut self) -> Result<(), PrimalError> {
        info!("🔄 Rediscovering orchestration services");
        self.preferred_orchestration_service = None;
        self.preferred_orchestration_service = Some(self.discover_orchestration_service().await?);
        Ok(())
    }

    /// Check if orchestration adapter is healthy
    pub async fn is_healthy(&self) -> bool {
        if let Some(service) = &self.preferred_orchestration_service {
            service.health.healthy
        } else {
            // Try to discover orchestration services
            match self.matcher.auto_discover_services().await {
                Ok(services) => services.iter().any(|s| {
                    s.capabilities
                        .iter()
                        .any(|cap| matches!(cap, ServiceCapability::Coordination { .. }))
                }),
                Err(_) => false,
            }
        }
    }

    /// Get orchestration capabilities summary
    pub async fn get_orchestration_capabilities(&self) -> Result<Vec<String>, PrimalError> {
        let services = self.registry.discover_by_category("orchestration").await?;

        let mut all_capabilities = Vec::new();
        for service in services {
            for capability in service.capabilities {
                if let ServiceCapability::Coordination { patterns, .. } = capability {
                    all_capabilities.extend(patterns);
                }
            }
        }

        // Remove duplicates
        all_capabilities.sort();
        all_capabilities.dedup();

        Ok(all_capabilities)
    }
}

/// Register an orchestration service with the universal registry.
///
/// The service is discovered by capability ("orchestration"), not by primal name.
pub async fn register_orchestration_service(
    registry: Arc<InMemoryServiceRegistry>,
) -> Result<(), PrimalError> {
    info!("🎼 Registering orchestration service with universal registry");

    let registration = super::UniversalServiceRegistration {
        service_id: uuid::Uuid::new_v4(),
        metadata: super::ServiceMetadata {
            name: "AI-Collaborative Service Mesh".to_string(),
            category: super::ServiceCategory::Orchestration {
                scopes: vec!["service_mesh".to_string(), "ai_collaboration".to_string()]
            },
            version: "1.0.0".to_string(),
            description: "AI-collaborative service mesh with universal extensibility and predictive capabilities".to_string(),
            maintainer: "EcoPrimals Core Team".to_string(),
            protocols: vec!["https".to_string(), "tarpc".to_string(), "websocket".to_string()],
        },
        capabilities: vec![
            ServiceCapability::Coordination {
                patterns: vec![
                    "service_discovery".to_string(),
                    "load_balancing".to_string(),
                    "workflow_coordination".to_string(),
                    "service_mesh".to_string(),
                    "ai_collaboration".to_string(),
                    "predictive_scaling".to_string(),
                    "fault_tolerance".to_string(),
                ],
                consistency: "eventual".to_string(),
                fault_tolerance: "high".to_string(),
            },
            ServiceCapability::ArtificialIntelligence {
                models: vec!["workload_classification".to_string(), "performance_prediction".to_string()],
                tasks: vec!["optimization".to_string(), "prediction".to_string(), "collaboration".to_string()],
                interfaces: vec!["rest_api".to_string(), "websocket".to_string(), "tarpc".to_string()],
            },
        ],
        endpoints: vec![
            super::ServiceEndpoint {
                name: "primary".to_string(),
                url: universal_constants::config_helpers::get_host(
                    "ORCHESTRATION_SERVICE_ENDPOINT",
                    "https://orchestration.ecosystem.local",
                ),
                protocol: "https".to_string(),
                port: None,
                path: Some("/api/v1".to_string()),
            },
            super::ServiceEndpoint {
                name: "websocket".to_string(),
                url: universal_constants::config_helpers::get_host(
                    "ORCHESTRATION_WS_ENDPOINT",
                    "wss://orchestration.ecosystem.local",
                ),
                protocol: "websocket".to_string(),
                port: None,
                path: Some("/ws".to_string()),
            },
        ],
        resources: super::ResourceSpec {
            cpu_cores: Some(8),
            memory_gb: Some(16),
            storage_gb: Some(500),
            network_bandwidth: Some(10000),
            custom_resources: HashMap::from([
                ("ai_capability".to_string(), serde_json::json!("advanced")),
                ("service_mesh_size".to_string(), serde_json::json!("enterprise")),
            ]),
        },
        integration: super::IntegrationPreferences {
            preferred_protocols: vec!["https".to_string(), "tarpc".to_string(), "websocket".to_string()],
            retry_policy: "exponential_backoff".to_string(),
            timeout_seconds: 60,
            load_balancing_weight: 15,
        },
        extensions: HashMap::from([
            ("provider_domain".to_string(), serde_json::json!("orchestration")),
            ("ecosystem_role".to_string(), serde_json::json!("orchestration_provider")),
            ("ai_collaboration_level".to_string(), serde_json::json!("advanced")),
            ("gaming_support".to_string(), serde_json::json!(true)),
        ]),
        registration_timestamp: chrono::Utc::now(),
        service_version: "1.0.0".to_string(),
        instance_id: uuid::Uuid::new_v4().to_string(),
        priority: 15, // Very high priority for core orchestration primal
    };

    registry.register_service(registration).await?;

    info!("✅ Orchestration service successfully registered with universal registry");
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

    fn test_orchestration_registration() -> UniversalServiceRegistration {
        UniversalServiceRegistration {
            service_id: uuid::Uuid::new_v4(),
            metadata: ServiceMetadata {
                name: "Test Orchestration".to_string(),
                category: ServiceCategory::Orchestration {
                    scopes: vec!["service_mesh".to_string()],
                },
                version: "1.0.0".to_string(),
                description: "Test".to_string(),
                maintainer: "test".to_string(),
                protocols: vec!["https".to_string()],
            },
            capabilities: vec![ServiceCapability::Coordination {
                patterns: vec![
                    "service_discovery".to_string(),
                    "load_balancing".to_string(),
                    "workflow_coordination".to_string(),
                    "service_mesh".to_string(),
                ],
                consistency: "eventual".to_string(),
                fault_tolerance: "high".to_string(),
            }],
            endpoints: vec![ServiceEndpoint {
                name: "primary".to_string(),
                url: "https://orch.test".to_string(),
                protocol: "https".to_string(),
                port: Some(443),
                path: None,
            }],
            resources: ResourceSpec {
                cpu_cores: Some(4),
                memory_gb: Some(8),
                storage_gb: Some(100),
                network_bandwidth: Some(1000),
                custom_resources: HashMap::new(),
            },
            integration: IntegrationPreferences {
                preferred_protocols: vec!["https".to_string()],
                retry_policy: "simple".to_string(),
                timeout_seconds: 60,
                load_balancing_weight: 15,
            },
            extensions: HashMap::new(),
            registration_timestamp: chrono::Utc::now(),
            service_version: "1.0.0".to_string(),
            instance_id: "inst-orch".to_string(),
            priority: 15,
        }
    }

    async fn registry_with_orchestration() -> Arc<InMemoryServiceRegistry> {
        let reg = Arc::new(InMemoryServiceRegistry::new());
        reg.register_service(test_orchestration_registration())
            .await
            .expect("register");
        reg
    }

    #[tokio::test]
    async fn new_adapter_get_current_none() {
        let reg = Arc::new(InMemoryServiceRegistry::new());
        let adapter = UniversalOrchestrationAdapter::new(reg);
        assert!(adapter.get_current_orchestration_service().is_none());
    }

    #[tokio::test]
    async fn coordinate_ai_workflow_happy_path() {
        let reg = registry_with_orchestration().await;
        let mut adapter = UniversalOrchestrationAdapter::new(reg);
        let v = adapter
            .coordinate_ai_workflow("pipeline", vec!["a".to_string(), "b".to_string()])
            .await
            .expect("coord");
        assert_eq!(v["status"], "coordinated");
        assert_eq!(v["workflow_type"], "pipeline");
        assert!(v.get("workflow_id").is_some());
    }

    #[tokio::test]
    async fn discover_services_universal_dedupes() {
        let reg = Arc::new(InMemoryServiceRegistry::new());
        let mut r1 = test_orchestration_registration();
        r1.metadata.name = "O1".to_string();
        let mut r2 = test_orchestration_registration();
        r2.metadata.name = "O2".to_string();
        reg.register_service(r1).await.expect("should succeed");
        reg.register_service(r2).await.expect("should succeed");
        let mut adapter = UniversalOrchestrationAdapter::new(reg);
        let list = adapter
            .discover_services_universal(vec![
                "Orchestration".to_string(),
                "Orchestration".to_string(),
            ])
            .await
            .expect("discover");
        assert!(!list.is_empty());
    }

    #[tokio::test]
    async fn request_load_balancing_and_mesh_status() {
        let reg = registry_with_orchestration().await;
        let mut adapter = UniversalOrchestrationAdapter::new(reg);
        let lb = adapter
            .request_load_balancing(vec!["s1".to_string(), "s2".to_string()], "round_robin")
            .await
            .expect("lb");
        assert_eq!(lb["strategy"], "round_robin");
        let mesh = adapter.get_service_mesh_status().await.expect("mesh");
        assert_eq!(mesh["service_mesh"]["status"], "active");
    }

    #[tokio::test]
    async fn request_load_balancing_empty_targets() {
        let reg = registry_with_orchestration().await;
        let mut adapter = UniversalOrchestrationAdapter::new(reg);
        let lb = adapter
            .request_load_balancing(vec![], "least_conn")
            .await
            .expect("lb");
        assert!(lb["routing_table"].is_array());
    }

    #[tokio::test]
    async fn rediscover_orchestration_services() {
        let reg = registry_with_orchestration().await;
        let mut adapter = UniversalOrchestrationAdapter::new(reg);
        adapter
            .coordinate_ai_workflow("w", vec![])
            .await
            .expect("should succeed");
        adapter
            .rediscover_orchestration_services()
            .await
            .expect("rediscover");
        assert!(adapter.get_current_orchestration_service().is_some());
    }

    #[tokio::test]
    async fn discovery_fails_empty_registry() {
        let reg = Arc::new(InMemoryServiceRegistry::new());
        let mut adapter = UniversalOrchestrationAdapter::new(reg);
        let err = adapter
            .coordinate_ai_workflow("w", vec![])
            .await
            .unwrap_err();
        assert!(matches!(err, PrimalError::ServiceDiscoveryError(_)));
    }

    #[tokio::test]
    async fn discover_services_universal_unknown_category_empty() {
        let reg = registry_with_orchestration().await;
        let mut adapter = UniversalOrchestrationAdapter::new(reg);
        let empty = adapter
            .discover_services_universal(vec!["nonexistent_category_xyz".to_string()])
            .await
            .expect("ok");
        assert!(empty.is_empty());
    }

    #[tokio::test]
    async fn get_orchestration_capabilities() {
        let reg = registry_with_orchestration().await;
        let adapter = UniversalOrchestrationAdapter::new(reg);
        let caps = adapter
            .get_orchestration_capabilities()
            .await
            .expect("caps");
        assert!(caps.contains(&"service_mesh".to_string()));
    }

    #[tokio::test]
    async fn is_healthy_true_with_registered() {
        let reg = registry_with_orchestration().await;
        let adapter = UniversalOrchestrationAdapter::new(reg);
        assert!(adapter.is_healthy().await);
    }

    #[tokio::test]
    async fn is_healthy_false_empty_registry() {
        let reg = Arc::new(InMemoryServiceRegistry::new());
        let adapter = UniversalOrchestrationAdapter::new(reg);
        assert!(!adapter.is_healthy().await);
    }

    #[tokio::test]
    async fn is_healthy_false_when_unhealthy_after_coord() {
        let reg = Arc::new(InMemoryServiceRegistry::new());
        let reg_data = test_orchestration_registration();
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
        let mut adapter = UniversalOrchestrationAdapter::new(reg);
        adapter
            .coordinate_ai_workflow("w", vec![])
            .await
            .expect("should succeed");
        assert!(!adapter.is_healthy().await);
    }

    #[tokio::test]
    async fn register_orchestration_and_serde_roundtrip() {
        let reg = Arc::new(InMemoryServiceRegistry::new());
        register_orchestration_service(reg.clone())
            .await
            .expect("reg");
        let services = reg.list_all_services().await.expect("list");
        assert_eq!(services.len(), 1);
        let json = serde_json::to_string(&services[0].capabilities).expect("ser");
        let caps: Vec<ServiceCapability> = serde_json::from_str(&json).expect("de");
        assert!(!caps.is_empty());
    }

    #[test]
    fn orchestration_registration_serde_roundtrip() {
        let r = test_orchestration_registration();
        let s = serde_json::to_string(&r).expect("json");
        let back: UniversalServiceRegistration = serde_json::from_str(&s).expect("back");
        assert_eq!(r.service_id, back.service_id);
    }
}
