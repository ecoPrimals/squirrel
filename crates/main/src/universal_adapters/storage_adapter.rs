// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Universal Storage Adapter
//!
//! Capability-based storage coordination that can work with `NestGate` or any
//! storage primal that provides the required data management capabilities.

use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info};

use super::registry::ServiceInfo;
use super::{ServiceCapability, ServiceMatcher, UniversalRequest, UniversalServiceRegistry};
use crate::error::PrimalError;

/// Universal Storage Adapter - works with any storage primal
pub struct UniversalStorageAdapter {
    registry: Arc<dyn UniversalServiceRegistry>,
    matcher: ServiceMatcher,
    preferred_storage_service: Option<ServiceInfo>,
}

impl UniversalStorageAdapter {
    /// Create a new universal storage adapter
    pub fn new(registry: Arc<dyn UniversalServiceRegistry>) -> Self {
        let matcher = ServiceMatcher::new(registry.clone());

        Self {
            registry,
            matcher,
            preferred_storage_service: None,
        }
    }

    /// Coordinate storage operation with any available storage primal
    pub async fn coordinate_storage(
        &mut self,
        operation: &str,
        data: serde_json::Value,
    ) -> Result<serde_json::Value, PrimalError> {
        info!(
            "🏠 Coordinating storage operation: {} via universal adapter",
            operation
        );

        // Discover storage service if needed
        if self.preferred_storage_service.is_none() {
            self.preferred_storage_service = Some(self.discover_storage_service().await?);
        }

        let storage_service = self.preferred_storage_service.as_ref().ok_or_else(|| {
            error!("No storage service available - adapter initialization failed");
            PrimalError::ResourceNotFound("No storage service available".to_string())
        })?;

        // Create universal storage request
        let request_params = HashMap::from([
            ("operation_type".to_string(), serde_json::json!(operation)),
            ("data".to_string(), data),
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
                    "storage_context".to_string(),
                    serde_json::json!("ai_data_management"),
                ),
            ]),
            requester: "squirrel".to_string(),
            timestamp: chrono::Utc::now(),
        };

        // Simulate storage coordination (in real implementation, make HTTP call)
        let response_data = match operation {
            "store" => serde_json::json!({
                "storage_id": uuid::Uuid::new_v4().to_string(),
                "status": "stored",
                "storage_service": storage_service.name,
                "location": format!("{}/ai_data/{}", storage_service.name.to_lowercase(), uuid::Uuid::new_v4()),
                "metadata": {
                    "compression": "ai_optimized",
                    "encryption": "aes-256-gcm",
                    "replication_factor": 3,
                    "deduplication": true
                },
                "performance": {
                    "storage_time_ms": 45,
                    "compression_ratio": 0.65,
                    "estimated_retrieval_time_ms": 25
                }
            }),
            "retrieve" => serde_json::json!({
                "retrieval_id": uuid::Uuid::new_v4().to_string(),
                "status": "retrieved",
                "storage_service": storage_service.name,
                "data_available": true,
                "metadata": {
                    "original_size_bytes": 2048,
                    "compressed_size_bytes": 1331,
                    "last_accessed": chrono::Utc::now().to_rfc3339()
                },
                "performance": {
                    "retrieval_time_ms": 28,
                    "cache_hit": false,
                    "network_latency_ms": 12
                }
            }),
            "backup" => serde_json::json!({
                "backup_id": uuid::Uuid::new_v4().to_string(),
                "status": "backed_up",
                "storage_service": storage_service.name,
                "backup_locations": [
                    format!("{}/backup/primary", storage_service.name.to_lowercase()),
                    format!("{}/backup/secondary", storage_service.name.to_lowercase())
                ],
                "metadata": {
                    "backup_type": "incremental",
                    "consistency_check": "passed",
                    "retention_days": 90
                }
            }),
            _ => serde_json::json!({
                "status": "completed",
                "storage_service": storage_service.name,
                "operation": operation,
                "message": format!("Operation '{}' completed successfully", operation)
            }),
        };

        info!(
            "✅ Storage operation '{}' coordinated via {} ({})",
            operation, storage_service.name, storage_service.service_id
        );

        Ok(response_data)
    }

    /// Store AI context data using any available storage primal
    pub async fn store_ai_context(
        &mut self,
        context_id: &str,
        context_data: serde_json::Value,
    ) -> Result<String, PrimalError> {
        debug!(
            "💾 Storing AI context {} via universal storage adapter",
            context_id
        );

        let storage_data = serde_json::json!({
            "context_id": context_id,
            "context_data": context_data,
            "metadata": {
                "data_type": "ai_context",
                "created_by": "squirrel_ai_coordinator",
                "created_at": chrono::Utc::now().to_rfc3339()
            }
        });

        let response = self.coordinate_storage("store", storage_data).await?;

        // Extract storage ID from response
        response
            .get("storage_id")
            .and_then(|v| v.as_str())
            .map(std::string::ToString::to_string)
            .ok_or_else(|| PrimalError::StorageError("Storage failed: no storage ID".to_string()))
    }

    /// Retrieve AI context data using any available storage primal
    pub async fn retrieve_ai_context(
        &mut self,
        storage_id: &str,
    ) -> Result<serde_json::Value, PrimalError> {
        debug!(
            "📥 Retrieving AI context {} via universal storage adapter",
            storage_id
        );

        let retrieval_data = serde_json::json!({
            "storage_id": storage_id,
            "retrieval_type": "ai_context"
        });

        let response = self.coordinate_storage("retrieve", retrieval_data).await?;

        // Extract context data from response
        response.get("context_data").cloned().ok_or_else(|| {
            PrimalError::StorageError("Retrieval failed: no context data".to_string())
        })
    }

    /// Backup AI data using any available storage primal
    pub async fn backup_ai_data(&mut self, backup_set: &str) -> Result<String, PrimalError> {
        info!(
            "💿 Creating AI data backup '{}' via universal storage adapter",
            backup_set
        );

        let backup_data = serde_json::json!({
            "backup_set": backup_set,
            "backup_type": "ai_data",
            "includes": ["ai_contexts", "model_cache", "coordination_logs"],
            "metadata": {
                "coordinator": "squirrel",
                "backup_strategy": "ai_optimized"
            }
        });

        let response = self.coordinate_storage("backup", backup_data).await?;

        // Extract backup ID from response
        response
            .get("backup_id")
            .and_then(|v| v.as_str())
            .map(std::string::ToString::to_string)
            .ok_or_else(|| PrimalError::StorageError("Backup failed: no backup ID".to_string()))
    }

    /// Discover storage services by capability
    async fn discover_storage_service(&self) -> Result<ServiceInfo, PrimalError> {
        info!("🔍 Discovering storage services via universal capability matching");

        // Define required storage capabilities
        let required_capabilities = vec![ServiceCapability::DataManagement {
            operations: vec![
                "store".to_string(),
                "retrieve".to_string(),
                "backup".to_string(),
                "compress".to_string(),
            ],
            consistency: "strong".to_string(),
            durability: "replicated".to_string(),
        }];

        // Find optimal storage service
        let storage_service = self
            .matcher
            .match_service_for_task("AI Data Management", required_capabilities)
            .await?;

        info!(
            "🎯 Selected storage service: {} ({})",
            storage_service.name, storage_service.service_id
        );

        Ok(storage_service)
    }

    /// Get current storage service info
    #[must_use]
    pub const fn get_current_storage_service(&self) -> Option<&ServiceInfo> {
        self.preferred_storage_service.as_ref()
    }

    /// Force rediscovery of storage services
    pub async fn rediscover_storage_services(&mut self) -> Result<(), PrimalError> {
        info!("🔄 Rediscovering storage services");
        self.preferred_storage_service = None;
        self.preferred_storage_service = Some(self.discover_storage_service().await?);
        Ok(())
    }

    /// Check if storage adapter is healthy
    pub async fn is_healthy(&self) -> bool {
        if let Some(service) = &self.preferred_storage_service {
            service.health.healthy
        } else {
            // Try to discover storage services
            match self.matcher.auto_discover_services().await {
                Ok(services) => services.iter().any(|s| {
                    s.capabilities
                        .iter()
                        .any(|cap| matches!(cap, ServiceCapability::DataManagement { .. }))
                }),
                Err(_) => false,
            }
        }
    }

    /// Get storage capabilities summary
    pub async fn get_storage_capabilities(&self) -> Result<Vec<String>, PrimalError> {
        let services = self.registry.discover_by_category("storage").await?;

        let mut all_capabilities = Vec::new();
        for service in services {
            for capability in service.capabilities {
                if let ServiceCapability::DataManagement { operations, .. } = capability {
                    all_capabilities.extend(operations);
                }
            }
        }

        // Remove duplicates
        all_capabilities.sort();
        all_capabilities.dedup();

        Ok(all_capabilities)
    }

    /// Get storage performance metrics
    pub async fn get_storage_metrics(&self) -> Result<serde_json::Value, PrimalError> {
        if let Some(service) = &self.preferred_storage_service {
            Ok(serde_json::json!({
                "storage_service": service.name,
                "service_id": service.service_id,
                "health": service.health.healthy,
                "metrics": service.health.metrics,
                "capabilities": service.capabilities.len(),
                "priority": service.priority
            }))
        } else {
            Err(PrimalError::StorageError(
                "No storage service available".to_string(),
            ))
        }
    }
}

/// Register `NestGate` (or any storage primal) with the universal registry
pub async fn register_nestgate_service(
    registry: Arc<dyn UniversalServiceRegistry>,
) -> Result<(), PrimalError> {
    info!("🏠 Registering NestGate storage service with universal registry");

    let registration = super::UniversalServiceRegistration {
        service_id: uuid::Uuid::new_v4(),
        metadata: super::ServiceMetadata {
            name: "NestGate Storage Primal".to_string(),
            category: super::ServiceCategory::Storage {
                types: vec!["distributed".to_string(), "ai_optimized".to_string()],
            },
            version: "1.0.0".to_string(),
            description:
                "Advanced distributed storage system with AI optimization and enterprise features"
                    .to_string(),
            maintainer: "EcoPrimals Core Team".to_string(),
            protocols: vec!["https".to_string(), "s3".to_string(), "nfs".to_string()],
        },
        capabilities: vec![ServiceCapability::DataManagement {
            operations: vec![
                "store".to_string(),
                "retrieve".to_string(),
                "backup".to_string(),
                "compress".to_string(),
                "deduplicate".to_string(),
                "encrypt".to_string(),
                "replicate".to_string(),
            ],
            consistency: "strong".to_string(),
            durability: "replicated".to_string(),
        }],
        endpoints: vec![
            super::ServiceEndpoint {
                name: "primary".to_string(),
                url: "https://nestgate.ecosystem.local".to_string(),
                protocol: "https".to_string(),
                port: Some(443),
                path: Some("/api/v1".to_string()),
            },
            super::ServiceEndpoint {
                name: "s3_compatible".to_string(),
                url: "https://s3.nestgate.ecosystem.local".to_string(),
                protocol: "s3".to_string(),
                port: Some(443),
                path: None,
            },
        ],
        resources: super::ResourceSpec {
            cpu_cores: Some(6),
            memory_gb: Some(32),
            storage_gb: Some(10000),
            network_bandwidth: Some(5000),
            custom_resources: HashMap::from([
                ("storage_type".to_string(), serde_json::json!("distributed")),
                (
                    "compression_algorithms".to_string(),
                    serde_json::json!(["lz4", "zstd", "gzip"]),
                ),
                ("replication_factor".to_string(), serde_json::json!(3)),
            ]),
        },
        integration: super::IntegrationPreferences {
            preferred_protocols: vec!["https".to_string(), "s3".to_string()],
            retry_policy: "exponential_backoff".to_string(),
            timeout_seconds: 120,
            load_balancing_weight: 12,
        },
        extensions: HashMap::from([
            ("provider_domain".to_string(), serde_json::json!("storage")),
            (
                "ecosystem_role".to_string(),
                serde_json::json!("storage_provider"),
            ),
            ("ai_optimization".to_string(), serde_json::json!(true)),
            ("enterprise_features".to_string(), serde_json::json!(true)),
        ]),
        registration_timestamp: chrono::Utc::now(),
        service_version: "1.0.0".to_string(),
        instance_id: uuid::Uuid::new_v4().to_string(),
        priority: 12, // High priority for core storage primal
    };

    registry.register_service(registration).await?;

    info!("✅ NestGate storage service successfully registered with universal registry");
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

    fn test_storage_registration() -> UniversalServiceRegistration {
        UniversalServiceRegistration {
            service_id: uuid::Uuid::new_v4(),
            metadata: ServiceMetadata {
                name: "Test Storage".to_string(),
                category: ServiceCategory::Storage {
                    types: vec!["distributed".to_string()],
                },
                version: "1.0.0".to_string(),
                description: "Test".to_string(),
                maintainer: "test".to_string(),
                protocols: vec!["https".to_string()],
            },
            capabilities: vec![ServiceCapability::DataManagement {
                operations: vec![
                    "store".to_string(),
                    "retrieve".to_string(),
                    "backup".to_string(),
                    "compress".to_string(),
                ],
                consistency: "strong".to_string(),
                durability: "replicated".to_string(),
            }],
            endpoints: vec![ServiceEndpoint {
                name: "primary".to_string(),
                url: "https://storage.test".to_string(),
                protocol: "https".to_string(),
                port: Some(443),
                path: None,
            }],
            resources: ResourceSpec {
                cpu_cores: Some(4),
                memory_gb: Some(16),
                storage_gb: Some(500),
                network_bandwidth: Some(2000),
                custom_resources: HashMap::new(),
            },
            integration: IntegrationPreferences {
                preferred_protocols: vec!["https".to_string()],
                retry_policy: "simple".to_string(),
                timeout_seconds: 120,
                load_balancing_weight: 12,
            },
            extensions: HashMap::new(),
            registration_timestamp: chrono::Utc::now(),
            service_version: "1.0.0".to_string(),
            instance_id: "inst-storage".to_string(),
            priority: 12,
        }
    }

    async fn registry_with_storage() -> Arc<dyn UniversalServiceRegistry> {
        let reg = Arc::new(InMemoryServiceRegistry::new());
        reg.register_service(test_storage_registration())
            .await
            .expect("register");
        reg
    }

    #[tokio::test]
    async fn new_adapter_get_current_none() {
        let reg = Arc::new(InMemoryServiceRegistry::new());
        let adapter = UniversalStorageAdapter::new(reg);
        assert!(adapter.get_current_storage_service().is_none());
    }

    #[tokio::test]
    async fn coordinate_storage_store_retrieve_backup_default() {
        let reg = registry_with_storage().await;
        let mut adapter = UniversalStorageAdapter::new(reg);
        let store = adapter
            .coordinate_storage("store", serde_json::json!({ "k": 1 }))
            .await
            .expect("store");
        assert_eq!(store["status"], "stored");
        assert!(store.get("storage_id").is_some());

        let ret = adapter
            .coordinate_storage("retrieve", serde_json::json!({}))
            .await
            .expect("retrieve");
        assert_eq!(ret["status"], "retrieved");

        let bak = adapter
            .coordinate_storage("backup", serde_json::json!({}))
            .await
            .expect("backup");
        assert_eq!(bak["status"], "backed_up");

        let other = adapter
            .coordinate_storage("list", serde_json::json!({}))
            .await
            .expect("other");
        assert_eq!(other["status"], "completed");
    }

    #[tokio::test]
    async fn store_ai_context_returns_storage_id() {
        let reg = registry_with_storage().await;
        let mut adapter = UniversalStorageAdapter::new(reg);
        let id = adapter
            .store_ai_context("ctx-1", serde_json::json!({ "x": 1 }))
            .await
            .expect("id");
        assert!(!id.is_empty());
    }

    #[tokio::test]
    async fn retrieve_ai_context_errors_no_context_data() {
        let reg = registry_with_storage().await;
        let mut adapter = UniversalStorageAdapter::new(reg);
        let err = adapter.retrieve_ai_context("sid-1").await.unwrap_err();
        assert!(matches!(err, PrimalError::StorageError(_)));
    }

    #[tokio::test]
    async fn backup_ai_data_returns_backup_id() {
        let reg = registry_with_storage().await;
        let mut adapter = UniversalStorageAdapter::new(reg);
        let id = adapter.backup_ai_data("set-a").await.expect("backup id");
        assert!(!id.is_empty());
    }

    #[tokio::test]
    async fn get_storage_metrics_errors_without_service() {
        let reg = Arc::new(InMemoryServiceRegistry::new());
        let adapter = UniversalStorageAdapter::new(reg);
        let err = adapter.get_storage_metrics().await.unwrap_err();
        assert!(matches!(err, PrimalError::StorageError(_)));
    }

    #[tokio::test]
    async fn get_storage_metrics_ok() {
        let reg = registry_with_storage().await;
        let mut adapter = UniversalStorageAdapter::new(reg);
        adapter
            .coordinate_storage("store", serde_json::json!({}))
            .await
            .expect("should succeed");
        let m = adapter.get_storage_metrics().await.expect("metrics");
        assert_eq!(m["storage_service"], "Test Storage");
    }

    #[tokio::test]
    async fn get_storage_capabilities() {
        let reg = registry_with_storage().await;
        let adapter = UniversalStorageAdapter::new(reg);
        let caps = adapter.get_storage_capabilities().await.expect("caps");
        assert!(caps.contains(&"store".to_string()));
        assert!(caps.contains(&"retrieve".to_string()));
    }

    #[tokio::test]
    async fn rediscover_storage_services() {
        let reg = registry_with_storage().await;
        let mut adapter = UniversalStorageAdapter::new(reg);
        adapter
            .coordinate_storage("store", serde_json::json!({}))
            .await
            .expect("should succeed");
        adapter.rediscover_storage_services().await.expect("redisc");
        assert!(adapter.get_current_storage_service().is_some());
    }

    #[tokio::test]
    async fn discovery_fails_empty_registry() {
        let reg = Arc::new(InMemoryServiceRegistry::new());
        let mut adapter = UniversalStorageAdapter::new(reg);
        let err = adapter
            .coordinate_storage("store", serde_json::json!({}))
            .await
            .unwrap_err();
        assert!(matches!(err, PrimalError::ServiceDiscoveryError(_)));
    }

    #[tokio::test]
    async fn is_healthy_true() {
        let reg = registry_with_storage().await;
        let adapter = UniversalStorageAdapter::new(reg);
        assert!(adapter.is_healthy().await);
    }

    #[tokio::test]
    async fn is_healthy_false_empty() {
        let reg = Arc::new(InMemoryServiceRegistry::new());
        let adapter = UniversalStorageAdapter::new(reg);
        assert!(!adapter.is_healthy().await);
    }

    #[tokio::test]
    async fn is_healthy_false_unhealthy_service() {
        let reg = Arc::new(InMemoryServiceRegistry::new());
        let reg_data = test_storage_registration();
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
        let mut adapter = UniversalStorageAdapter::new(reg);
        adapter
            .coordinate_storage("store", serde_json::json!({}))
            .await
            .expect("should succeed");
        assert!(!adapter.is_healthy().await);
    }

    #[tokio::test]
    async fn register_nestgate_and_serde_roundtrip() {
        let reg = Arc::new(InMemoryServiceRegistry::new());
        register_nestgate_service(reg.clone()).await.expect("reg");
        let services = reg.list_all_services().await.expect("list");
        assert_eq!(services.len(), 1);
        let json = serde_json::to_string(&services[0].capabilities).expect("ser");
        let caps: Vec<ServiceCapability> = serde_json::from_str(&json).expect("de");
        assert!(!caps.is_empty());
    }

    #[test]
    fn storage_registration_serde_roundtrip() {
        let r = test_storage_registration();
        let s = serde_json::to_string(&r).expect("json");
        let back: UniversalServiceRegistration = serde_json::from_str(&s).expect("back");
        assert_eq!(r.service_id, back.service_id);
    }
}
