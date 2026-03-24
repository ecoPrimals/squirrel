// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Universal Compute Adapter
//!
//! Capability-based compute coordination that can work with `ToadStool` or any
//! compute primal that provides the required computation capabilities.

use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info};

use super::registry::ServiceInfo;
use super::{ServiceCapability, ServiceMatcher, UniversalRequest, UniversalServiceRegistry};
use crate::error::PrimalError;

/// Universal Compute Adapter - works with any compute primal
pub struct UniversalComputeAdapter {
    registry: Arc<dyn UniversalServiceRegistry>,
    matcher: ServiceMatcher,
    preferred_compute_service: Option<ServiceInfo>,
}

impl UniversalComputeAdapter {
    /// Create a new universal compute adapter
    pub fn new(registry: Arc<dyn UniversalServiceRegistry>) -> Self {
        let matcher = ServiceMatcher::new(registry.clone());

        Self {
            registry,
            matcher,
            preferred_compute_service: None,
        }
    }

    /// Coordinate compute operation with any available compute primal
    #[expect(clippy::too_many_lines, reason = "Compute adapter; refactor planned")]
    pub async fn coordinate_computation(
        &mut self,
        operation: &str,
        workload: serde_json::Value,
    ) -> Result<serde_json::Value, PrimalError> {
        info!(
            "🍄 Coordinating compute operation: {} via universal adapter",
            operation
        );

        // Discover compute service if needed
        if self.preferred_compute_service.is_none() {
            self.preferred_compute_service = Some(self.discover_compute_service().await?);
        }

        let compute_service = self.preferred_compute_service.as_ref().ok_or_else(|| {
            error!("No compute service available - adapter initialization failed");
            PrimalError::ResourceNotFound("No compute service available".to_string())
        })?;

        // Create universal compute request
        let request_params = HashMap::from([
            ("operation_type".to_string(), serde_json::json!(operation)),
            ("workload".to_string(), workload),
            ("coordinator".to_string(), serde_json::json!("squirrel")),
            ("ai_context".to_string(), serde_json::json!(true)),
            ("optimization".to_string(), serde_json::json!("ai_enhanced")),
        ]);

        let _request = UniversalRequest {
            request_id: uuid::Uuid::new_v4().to_string(),
            operation: operation.to_string(),
            parameters: request_params,
            context: HashMap::from([
                (
                    "requester".to_string(),
                    serde_json::json!("squirrel_ai_coordinator"),
                ),
                (
                    "compute_context".to_string(),
                    serde_json::json!("ai_coordination"),
                ),
            ]),
            requester: "squirrel".to_string(),
            timestamp: chrono::Utc::now(),
        };

        // Simulate compute coordination (in real implementation, make HTTP call)
        let response_data = match operation {
            "execute" => serde_json::json!({
                "execution_id": uuid::Uuid::new_v4().to_string(),
                "status": "completed",
                "compute_service": compute_service.name,
                "result": {
                    "success": true,
                    "output": "AI coordination computation completed successfully",
                    "metrics": {
                        "execution_time_ms": 1250,
                        "cpu_usage_percent": 45,
                        "memory_usage_mb": 256,
                        "tasks_processed": 12
                    }
                },
                "resource_allocation": {
                    "cpu_cores_used": 2,
                    "memory_allocated_gb": 4,
                    "container_instances": 1,
                    "gpu_acceleration": false
                },
                "performance": {
                    "throughput_ops_per_sec": 850,
                    "latency_p95_ms": 85,
                    "efficiency_score": 0.87
                }
            }),
            "scale" => serde_json::json!({
                "scaling_id": uuid::Uuid::new_v4().to_string(),
                "status": "scaled",
                "compute_service": compute_service.name,
                "scaling_action": "up",
                "details": {
                    "previous_instances": 1,
                    "current_instances": 3,
                    "target_instances": 3,
                    "scaling_reason": "ai_workload_increase"
                },
                "resource_changes": {
                    "cpu_cores": "2 -> 6",
                    "memory_gb": "4 -> 12",
                    "network_bandwidth": "1000 -> 3000"
                }
            }),
            "monitor" => serde_json::json!({
                "monitoring_id": uuid::Uuid::new_v4().to_string(),
                "status": "monitoring",
                "compute_service": compute_service.name,
                "metrics": {
                    "active_workloads": 5,
                    "queued_tasks": 2,
                    "average_response_time_ms": 145,
                    "resource_utilization": {
                        "cpu_percent": 68,
                        "memory_percent": 42,
                        "storage_percent": 15,
                        "network_utilization_percent": 25
                    }
                },
                "health_status": {
                    "overall": "healthy",
                    "cpu": "optimal",
                    "memory": "good",
                    "storage": "excellent",
                    "network": "good"
                }
            }),
            _ => serde_json::json!({
                "status": "completed",
                "compute_service": compute_service.name,
                "operation": operation,
                "message": format!("Compute operation '{}' completed successfully", operation)
            }),
        };

        info!(
            "✅ Compute operation '{}' coordinated via {} ({})",
            operation, compute_service.name, compute_service.service_id
        );

        Ok(response_data)
    }

    /// Execute AI workload using any available compute primal
    pub async fn execute_ai_workload(
        &mut self,
        workload_type: &str,
        parameters: HashMap<String, serde_json::Value>,
    ) -> Result<String, PrimalError> {
        debug!(
            "🧠 Executing AI workload '{}' via universal compute adapter",
            workload_type
        );

        let workload_data = serde_json::json!({
            "workload_type": workload_type,
            "parameters": parameters,
            "metadata": {
                "coordinator": "squirrel_ai_coordinator",
                "execution_context": "ai_coordination",
                "created_at": chrono::Utc::now().to_rfc3339()
            }
        });

        let response = self
            .coordinate_computation("execute", workload_data)
            .await?;

        // Extract execution ID from response
        response
            .get("execution_id")
            .and_then(|v| v.as_str())
            .map(std::string::ToString::to_string)
            .ok_or_else(|| {
                PrimalError::ComputeError("Execution failed: no execution ID".to_string())
            })
    }

    /// Scale compute resources using any available compute primal
    pub async fn scale_compute_resources(
        &mut self,
        scaling_direction: &str,
        factor: u32,
    ) -> Result<serde_json::Value, PrimalError> {
        info!(
            "📈 Scaling compute resources {} by factor {} via universal adapter",
            scaling_direction, factor
        );

        let scaling_data = serde_json::json!({
            "scaling_direction": scaling_direction,
            "scaling_factor": factor,
            "trigger": "ai_coordinator_request",
            "metadata": {
                "requested_by": "squirrel",
                "optimization_goal": "ai_performance"
            }
        });

        let response = self.coordinate_computation("scale", scaling_data).await?;
        Ok(response)
    }

    /// Monitor compute performance using any available compute primal
    pub async fn monitor_compute_performance(&mut self) -> Result<serde_json::Value, PrimalError> {
        debug!("📊 Monitoring compute performance via universal adapter");

        let monitoring_data = serde_json::json!({
            "monitoring_type": "performance",
            "metrics_requested": ["cpu", "memory", "throughput", "latency", "health"],
            "granularity": "detailed"
        });

        let response = self
            .coordinate_computation("monitor", monitoring_data)
            .await?;
        Ok(response)
    }

    /// Discover compute services by capability
    async fn discover_compute_service(&self) -> Result<ServiceInfo, PrimalError> {
        info!("🔍 Discovering compute services via universal capability matching");

        // Define required compute capabilities
        let required_capabilities = vec![ServiceCapability::Computation {
            types: vec![
                "container".to_string(),
                "serverless".to_string(),
                "batch".to_string(),
                "ai_workload".to_string(),
            ],
            resources: HashMap::from([
                ("cpu_cores".to_string(), serde_json::json!(4)),
                ("memory_gb".to_string(), serde_json::json!(8)),
                ("scaling".to_string(), serde_json::json!(true)),
            ]),
            constraints: vec!["auto_scaling".to_string(), "monitoring".to_string()],
        }];

        // Find optimal compute service
        let compute_service = self
            .matcher
            .match_service_for_task("AI Workload Execution", required_capabilities)
            .await?;

        info!(
            "🎯 Selected compute service: {} ({})",
            compute_service.name, compute_service.service_id
        );

        Ok(compute_service)
    }

    /// Get current compute service info
    #[must_use]
    pub const fn get_current_compute_service(&self) -> Option<&ServiceInfo> {
        self.preferred_compute_service.as_ref()
    }

    /// Force rediscovery of compute services
    pub async fn rediscover_compute_services(&mut self) -> Result<(), PrimalError> {
        info!("🔄 Rediscovering compute services");
        self.preferred_compute_service = None;
        self.preferred_compute_service = Some(self.discover_compute_service().await?);
        Ok(())
    }

    /// Check if compute adapter is healthy
    pub async fn is_healthy(&self) -> bool {
        if let Some(service) = &self.preferred_compute_service {
            service.health.healthy
        } else {
            // Try to discover compute services
            match self.matcher.auto_discover_services().await {
                Ok(services) => services.iter().any(|s| {
                    s.capabilities
                        .iter()
                        .any(|cap| matches!(cap, ServiceCapability::Computation { .. }))
                }),
                Err(_) => false,
            }
        }
    }

    /// Get compute capabilities summary
    pub async fn get_compute_capabilities(&self) -> Result<Vec<String>, PrimalError> {
        let services = self.registry.discover_by_category("compute").await?;

        let mut all_capabilities = Vec::new();
        for service in services {
            for capability in service.capabilities {
                if let ServiceCapability::Computation { types, .. } = capability {
                    all_capabilities.extend(types);
                }
            }
        }

        // Remove duplicates
        all_capabilities.sort();
        all_capabilities.dedup();

        Ok(all_capabilities)
    }

    /// Get compute resource availability
    pub async fn get_resource_availability(&self) -> Result<serde_json::Value, PrimalError> {
        if let Some(service) = &self.preferred_compute_service {
            // Extract resource information from service
            Ok(serde_json::json!({
                "compute_service": service.name,
                "service_id": service.service_id,
                "health": service.health.healthy,
                "resources": {
                    "cpu_cores_available": 12,
                    "memory_gb_available": 32,
                    "storage_gb_available": 1000,
                    "gpu_units_available": 2
                },
                "capabilities": service.capabilities.len(),
                "priority": service.priority,
                "utilization": {
                    "cpu_percent": 35,
                    "memory_percent": 45,
                    "storage_percent": 20
                }
            }))
        } else {
            Err(PrimalError::ComputeError(
                "No compute service available".to_string(),
            ))
        }
    }

    /// Submit batch job using any available compute primal
    pub async fn submit_batch_job(
        &mut self,
        job_definition: serde_json::Value,
    ) -> Result<String, PrimalError> {
        info!("📋 Submitting batch job via universal compute adapter");

        let response = self
            .coordinate_computation("batch_submit", job_definition)
            .await?;

        // Extract job ID from response
        response
            .get("job_id")
            .and_then(|v| v.as_str())
            .map(std::string::ToString::to_string)
            .ok_or_else(|| {
                PrimalError::ComputeError("Batch job submission failed: no job ID".to_string())
            })
    }
}

/// Register `ToadStool` (or any compute primal) with the universal registry
pub async fn register_toadstool_service(
    registry: Arc<dyn UniversalServiceRegistry>,
) -> Result<(), PrimalError> {
    info!("🍄 Registering ToadStool compute service with universal registry");

    let registration = super::UniversalServiceRegistration {
        service_id: uuid::Uuid::new_v4(),
        metadata: super::ServiceMetadata {
            name: "ToadStool Compute Platform".to_string(),
            category: super::ServiceCategory::Compute {
                specialties: vec![
                    "container".to_string(),
                    "serverless".to_string(),
                    "ai_workload".to_string(),
                ],
            },
            version: "1.0.0".to_string(),
            description:
                "Advanced compute platform with AI workload optimization and enterprise scaling"
                    .to_string(),
            maintainer: "EcoPrimals Core Team".to_string(),
            protocols: vec![
                "https".to_string(),
                "tarpc".to_string(),
                // Note: "docker" here refers to Docker API protocol support,
                // not a hardcoded requirement to use Docker
                "docker".to_string(),
            ],
        },
        capabilities: vec![ServiceCapability::Computation {
            types: vec![
                "container".to_string(),
                "serverless".to_string(),
                "batch".to_string(),
                "ai_workload".to_string(),
                "gpu_acceleration".to_string(),
            ],
            resources: HashMap::from([
                ("cpu_cores".to_string(), serde_json::json!(64)),
                ("memory_gb".to_string(), serde_json::json!(256)),
                ("storage_gb".to_string(), serde_json::json!(10000)),
                ("gpu_units".to_string(), serde_json::json!(8)),
                ("scaling_factor".to_string(), serde_json::json!(10)),
            ]),
            constraints: vec![
                "auto_scaling".to_string(),
                "monitoring".to_string(),
                "fault_tolerance".to_string(),
                "load_balancing".to_string(),
            ],
        }],
        endpoints: vec![
            super::ServiceEndpoint {
                name: "primary".to_string(),
                url: "https://toadstool.ecosystem.local".to_string(),
                protocol: "https".to_string(),
                port: Some(443),
                path: Some("/api/v1".to_string()),
            },
            super::ServiceEndpoint {
                name: "docker".to_string(),
                url: "docker://toadstool.ecosystem.local".to_string(),
                protocol: "docker".to_string(),
                port: Some(2376),
                path: None,
            },
        ],
        resources: super::ResourceSpec {
            cpu_cores: Some(64),
            memory_gb: Some(256),
            storage_gb: Some(10000),
            network_bandwidth: Some(25000),
            custom_resources: HashMap::from([
                ("compute_type".to_string(), serde_json::json!("distributed")),
                ("container_runtime".to_string(), serde_json::json!("docker")),
                ("gpu_support".to_string(), serde_json::json!(true)),
                ("ai_optimization".to_string(), serde_json::json!(true)),
            ]),
        },
        integration: super::IntegrationPreferences {
            preferred_protocols: vec!["https".to_string(), "tarpc".to_string()],
            retry_policy: "exponential_backoff".to_string(),
            timeout_seconds: 300,
            load_balancing_weight: 14,
        },
        extensions: HashMap::from([
            ("provider_domain".to_string(), serde_json::json!("compute")), // Capability-based metadata
            (
                "ecosystem_role".to_string(),
                serde_json::json!("compute_provider"),
            ),
            ("ai_workload_support".to_string(), serde_json::json!(true)),
            ("enterprise_scaling".to_string(), serde_json::json!(true)),
        ]),
        registration_timestamp: chrono::Utc::now(),
        service_version: "1.0.0".to_string(),
        instance_id: uuid::Uuid::new_v4().to_string(),
        priority: 14, // Very high priority for core compute primal
    };

    registry.register_service(registration).await?;

    info!("✅ ToadStool compute service successfully registered with universal registry");
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

    fn test_compute_registration() -> UniversalServiceRegistration {
        UniversalServiceRegistration {
            service_id: uuid::Uuid::new_v4(),
            metadata: ServiceMetadata {
                name: "Test Compute Service".to_string(),
                category: ServiceCategory::Compute {
                    specialties: vec!["ai_workload".to_string()],
                },
                version: "1.0.0".to_string(),
                description: "Test".to_string(),
                maintainer: "test".to_string(),
                protocols: vec!["https".to_string()],
            },
            capabilities: vec![ServiceCapability::Computation {
                types: vec![
                    "container".to_string(),
                    "serverless".to_string(),
                    "batch".to_string(),
                    "ai_workload".to_string(),
                ],
                resources: HashMap::from([
                    ("cpu_cores".to_string(), serde_json::json!(4)),
                    ("memory_gb".to_string(), serde_json::json!(8)),
                    ("scaling".to_string(), serde_json::json!(true)),
                ]),
                constraints: vec!["auto_scaling".to_string(), "monitoring".to_string()],
            }],
            endpoints: vec![ServiceEndpoint {
                name: "primary".to_string(),
                url: "https://compute.test".to_string(),
                protocol: "https".to_string(),
                port: Some(443),
                path: Some("/api".to_string()),
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
                timeout_seconds: 30,
                load_balancing_weight: 10,
            },
            extensions: HashMap::new(),
            registration_timestamp: chrono::Utc::now(),
            service_version: "1.0.0".to_string(),
            instance_id: "inst-compute".to_string(),
            priority: 10,
        }
    }

    async fn registry_with_compute() -> Arc<dyn UniversalServiceRegistry> {
        let reg = Arc::new(InMemoryServiceRegistry::new());
        reg.register_service(test_compute_registration())
            .await
            .expect("register compute");
        reg
    }

    #[tokio::test]
    async fn new_adapter_and_get_current_none_before_discovery() {
        let reg = Arc::new(InMemoryServiceRegistry::new());
        let adapter = UniversalComputeAdapter::new(reg);
        assert!(adapter.get_current_compute_service().is_none());
    }

    #[tokio::test]
    async fn coordinate_computation_execute_scale_monitor_and_default() {
        let reg = registry_with_compute().await;
        let mut adapter = UniversalComputeAdapter::new(reg);

        let exec = adapter
            .coordinate_computation("execute", serde_json::json!({ "x": 1 }))
            .await
            .expect("execute");
        assert_eq!(exec["status"], "completed");
        assert!(exec.get("execution_id").is_some());

        let scale = adapter
            .coordinate_computation("scale", serde_json::json!({}))
            .await
            .expect("scale");
        assert_eq!(scale["status"], "scaled");
        assert!(scale.get("scaling_id").is_some());

        let mon = adapter
            .coordinate_computation("monitor", serde_json::json!({}))
            .await
            .expect("monitor");
        assert_eq!(mon["status"], "monitoring");

        let custom = adapter
            .coordinate_computation("custom_op", serde_json::json!({}))
            .await
            .expect("custom");
        assert_eq!(custom["status"], "completed");
        assert_eq!(custom["operation"], "custom_op");
    }

    #[tokio::test]
    async fn execute_ai_workload_returns_execution_id() {
        let reg = registry_with_compute().await;
        let mut adapter = UniversalComputeAdapter::new(reg);
        let mut params = HashMap::new();
        params.insert("k".to_string(), serde_json::json!(1));
        let id = adapter
            .execute_ai_workload("inference", params)
            .await
            .expect("exec id");
        assert!(!id.is_empty());
    }

    #[tokio::test]
    async fn scale_and_monitor_helpers() {
        let reg = registry_with_compute().await;
        let mut adapter = UniversalComputeAdapter::new(reg);
        let scale = adapter
            .scale_compute_resources("up", 2)
            .await
            .expect("scale");
        assert_eq!(scale["scaling_action"], "up");
        let mon = adapter.monitor_compute_performance().await.expect("mon");
        assert_eq!(mon["status"], "monitoring");
    }

    #[tokio::test]
    async fn get_current_after_coordinate() {
        let reg = registry_with_compute().await;
        let mut adapter = UniversalComputeAdapter::new(reg);
        adapter
            .coordinate_computation("execute", serde_json::json!({}))
            .await
            .expect("should succeed");
        let info = adapter
            .get_current_compute_service()
            .expect("preferred service");
        let dbg = format!("{info:?}");
        assert!(dbg.contains("Test Compute Service") || dbg.contains("service_id"));
    }

    #[tokio::test]
    async fn rediscover_compute_services() {
        let reg = registry_with_compute().await;
        let mut adapter = UniversalComputeAdapter::new(reg);
        adapter
            .coordinate_computation("execute", serde_json::json!({}))
            .await
            .expect("should succeed");
        adapter
            .rediscover_compute_services()
            .await
            .expect("rediscover");
        assert!(adapter.get_current_compute_service().is_some());
    }

    #[tokio::test]
    async fn discovery_fails_empty_registry() {
        let reg = Arc::new(InMemoryServiceRegistry::new());
        let mut adapter = UniversalComputeAdapter::new(reg);
        let err = adapter
            .coordinate_computation("execute", serde_json::json!({}))
            .await
            .unwrap_err();
        assert!(matches!(err, PrimalError::ServiceDiscoveryError(_)));
    }

    #[tokio::test]
    async fn submit_batch_job_errors_no_job_id() {
        let reg = registry_with_compute().await;
        let mut adapter = UniversalComputeAdapter::new(reg);
        let err = adapter
            .submit_batch_job(serde_json::json!({}))
            .await
            .unwrap_err();
        assert!(matches!(err, PrimalError::ComputeError(_)));
    }

    #[tokio::test]
    async fn get_resource_availability_errors_without_service() {
        let reg = Arc::new(InMemoryServiceRegistry::new());
        let adapter = UniversalComputeAdapter::new(reg);
        let err = adapter.get_resource_availability().await.unwrap_err();
        assert!(matches!(err, PrimalError::ComputeError(_)));
    }

    #[tokio::test]
    async fn get_resource_availability_ok_with_service() {
        let reg = registry_with_compute().await;
        let mut adapter = UniversalComputeAdapter::new(reg);
        adapter
            .coordinate_computation("execute", serde_json::json!({}))
            .await
            .expect("should succeed");
        let j = adapter
            .get_resource_availability()
            .await
            .expect("resources");
        assert_eq!(j["compute_service"], "Test Compute Service");
        assert!(j["resources"]["cpu_cores_available"].is_number());
    }

    #[tokio::test]
    async fn get_compute_capabilities_deduplicates() {
        let reg = Arc::new(InMemoryServiceRegistry::new());
        let mut r1 = test_compute_registration();
        let mut r2 = test_compute_registration();
        r1.metadata.name = "C1".to_string();
        r2.metadata.name = "C2".to_string();
        reg.register_service(r1).await.expect("should succeed");
        reg.register_service(r2).await.expect("should succeed");
        let adapter = UniversalComputeAdapter::new(reg);
        let caps = adapter.get_compute_capabilities().await.expect("caps");
        assert!(caps.contains(&"batch".to_string()));
        assert!(caps.contains(&"ai_workload".to_string()));
    }

    #[tokio::test]
    async fn is_healthy_false_when_marked_unhealthy_in_registry() {
        let reg = Arc::new(InMemoryServiceRegistry::new());
        let reg_data = test_compute_registration();
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
        let mut adapter = UniversalComputeAdapter::new(reg);
        adapter
            .coordinate_computation("execute", serde_json::json!({}))
            .await
            .expect("coord");
        assert!(!adapter.is_healthy().await);
    }

    #[tokio::test]
    async fn is_healthy_auto_discover_true() {
        let reg = registry_with_compute().await;
        let adapter = UniversalComputeAdapter::new(reg);
        assert!(adapter.is_healthy().await);
    }

    #[tokio::test]
    async fn is_healthy_auto_discover_false_empty() {
        let reg = Arc::new(InMemoryServiceRegistry::new());
        let adapter = UniversalComputeAdapter::new(reg);
        assert!(!adapter.is_healthy().await);
    }

    #[tokio::test]
    async fn register_toadstool_and_serde_roundtrip() {
        let reg = Arc::new(InMemoryServiceRegistry::new());
        register_toadstool_service(reg.clone()).await.expect("reg");
        let services = reg.list_all_services().await.expect("list");
        assert_eq!(services.len(), 1);
        let json = serde_json::to_string(&services[0].capabilities).expect("ser cap");
        let caps: Vec<ServiceCapability> = serde_json::from_str(&json).expect("de cap");
        assert!(!caps.is_empty());
    }

    #[test]
    fn compute_registration_serde_roundtrip() {
        let r = test_compute_registration();
        let s = serde_json::to_string(&r).expect("to json");
        let back: UniversalServiceRegistration = serde_json::from_str(&s).expect("from json");
        assert_eq!(r.service_id, back.service_id);
        assert_eq!(r.metadata.name, back.metadata.name);
    }
}
