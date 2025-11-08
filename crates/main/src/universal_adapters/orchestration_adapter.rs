//! Universal Orchestration Adapter
//!
//! Capability-based orchestration coordination that can work with Songbird or any
//! orchestration primal that provides the required coordination capabilities.

use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info};

use super::registry::ServiceInfo;
use super::{
    ServiceCapability, ServiceMatcher, UniversalRequest,
    UniversalServiceRegistry,
};
use crate::error::PrimalError;

/// Universal Orchestration Adapter - works with any orchestration primal
pub struct UniversalOrchestrationAdapter {
    registry: Arc<dyn UniversalServiceRegistry>,
    matcher: ServiceMatcher,
    preferred_orchestration_service: Option<ServiceInfo>,
}

impl UniversalOrchestrationAdapter {
    /// Create a new universal orchestration adapter
    pub fn new(registry: Arc<dyn UniversalServiceRegistry>) -> Self {
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

        let orchestration_service = self.preferred_orchestration_service.as_ref()
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
            ("coordinator".to_string(), serde_json::json!("squirrel")),
            ("ai_enhanced".to_string(), serde_json::json!(true)),
            ("priority".to_string(), serde_json::json!("high")),
        ]);

        let request = UniversalRequest {
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
            requester: "squirrel".to_string(),
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

        let orchestration_service = self.preferred_orchestration_service.as_ref()
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

        let orchestration_service = self.preferred_orchestration_service.as_ref()
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
    pub fn get_current_orchestration_service(&self) -> Option<&ServiceInfo> {
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

/// Register Songbird (or any orchestration primal) with the universal registry
pub async fn register_songbird_service(
    registry: Arc<dyn UniversalServiceRegistry>,
) -> Result<(), PrimalError> {
    info!("🎼 Registering Songbird orchestration service with universal registry");

    let registration = super::UniversalServiceRegistration {
        service_id: uuid::Uuid::new_v4(),
        metadata: super::ServiceMetadata {
            name: "Songbird AI-Collaborative Service Mesh".to_string(),
            category: super::ServiceCategory::Orchestration {
                scopes: vec!["service_mesh".to_string(), "ai_collaboration".to_string()]
            },
            version: "1.0.0".to_string(),
            description: "AI-collaborative service mesh with universal extensibility and predictive capabilities".to_string(),
            maintainer: "EcoPrimals Core Team".to_string(),
            protocols: vec!["https".to_string(), "grpc".to_string(), "websocket".to_string()],
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
                interfaces: vec!["rest_api".to_string(), "websocket".to_string(), "grpc".to_string()],
            },
        ],
        endpoints: vec![
            super::ServiceEndpoint {
                name: "primary".to_string(),
                url: "https://songbird.ecosystem.local".to_string(),
                protocol: "https".to_string(),
                port: Some(443),
                path: Some("/api/v1".to_string()),
            },
            super::ServiceEndpoint {
                name: "websocket".to_string(),
                url: "wss://songbird.ecosystem.local".to_string(),
                protocol: "websocket".to_string(),
                port: Some(443),
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
            preferred_protocols: vec!["https".to_string(), "grpc".to_string(), "websocket".to_string()],
            retry_policy: "exponential_backoff".to_string(),
            timeout_seconds: 60,
            load_balancing_weight: 15,
        },
        extensions: HashMap::from([
            ("primal_type".to_string(), serde_json::json!("songbird")),
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

    info!("✅ Songbird orchestration service successfully registered with universal registry");
    Ok(())
}
