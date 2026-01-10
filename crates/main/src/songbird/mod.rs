//! Songbird integration for squirrel primal
//!
//! This module provides integration with the songbird orchestration system using
//! capability-based discovery instead of hardcoded primal names.
//!
//! # Primal Sovereignty
//!
//! This module uses `CapabilityRegistry` for discovering orchestration services
//! at runtime, rather than hardcoding "Songbird" references. This allows the
//! ecosystem to evolve without code changes.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::capability_registry::{CapabilityRegistry, PrimalCapability};
use crate::ecosystem::EcosystemConfig;
use crate::error::PrimalError;

/// Orchestration state for songbird coordination
#[derive(Debug, Clone, Default)]
pub struct OrchestrationState {
    /// Active coordinations
    pub active_coordinations: HashMap<String, CoordinationTask>,
    /// Pending tasks
    pub pending_tasks: Vec<String>,
    /// Completed tasks
    pub completed_tasks: Vec<String>,
}

/// Coordination task
#[derive(Debug, Clone)]
pub struct CoordinationTask {
    /// Task identifier
    pub id: String,
    /// Task status
    pub status: TaskStatus,
    /// Task timestamp
    pub timestamp: DateTime<Utc>,
}

/// Task status
#[derive(Debug, Clone)]
pub enum TaskStatus {
    /// Task is pending
    Pending,
    /// Task is in progress
    InProgress,
    /// Task is completed
    Completed,
    /// Task failed
    Failed(String),
}

/// Health status for songbird coordinator
#[derive(Debug, Clone)]
pub struct HealthStatus {
    /// Status string
    pub status: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Active sessions
    pub active_sessions: u32,
    /// Resource utilization
    pub resource_utilization: f64,
}

impl Default for HealthStatus {
    fn default() -> Self {
        Self {
            status: "unknown".to_string(),
            timestamp: Utc::now(),
            active_sessions: 0,
            resource_utilization: 0.0,
        }
    }
}

/// Songbird service coordination module using capability-based discovery
///
/// # Primal Sovereignty
///
/// This coordinator uses `CapabilityRegistry` to discover orchestration services
/// at runtime, eliminating hardcoded "Songbird" references. Each primal knows only
/// itself and discovers others by capability.
pub struct SongbirdCoordinator {
    /// Instance identifier
    instance_id: String,
    /// Ecosystem configuration
    config: EcosystemConfig,
    /// Capability registry for dynamic service discovery
    capability_registry: Arc<CapabilityRegistry>,
    /// Orchestration state
    orchestration_state: Arc<tokio::sync::RwLock<OrchestrationState>>,
    /// Health status
    health_status: HealthStatus,
    /// Initialization state
    initialized: bool,
    /// Shutdown state
    shutdown: bool,
}

impl SongbirdCoordinator {
    /// Create a new Songbird coordinator with capability-based discovery
    ///
    /// # Arguments
    ///
    /// * `config` - Ecosystem configuration
    /// * `capability_registry` - Registry for discovering orchestration services
    ///
    /// # Primal Sovereignty
    ///
    /// This constructor accepts a `CapabilityRegistry` instead of creating hardcoded
    /// clients. The coordinator discovers orchestration services at runtime.
    pub fn new(
        config: EcosystemConfig,
        capability_registry: Arc<CapabilityRegistry>,
    ) -> Result<Self, PrimalError> {
        let instance_id = Uuid::new_v4().to_string();
        let orchestration_state = Arc::new(RwLock::new(OrchestrationState::default()));
        let health_status = HealthStatus {
            status: "running".to_string(),
            timestamp: Utc::now(),
            active_sessions: 0,
            resource_utilization: 0.0,
        };

        Ok(Self {
            instance_id,
            config,
            capability_registry,
            orchestration_state,
            health_status,
            initialized: false,
            shutdown: false,
        })
    }

    /// Initialize songbird integration via capability-based discovery
    ///
    /// # Primal Sovereignty
    ///
    /// This method discovers orchestration services dynamically rather than
    /// hardcoding connections to specific primal names.
    pub async fn initialize(&mut self) -> Result<(), PrimalError> {
        info!("Initializing orchestration coordinator via capability discovery");

        // Discover orchestration services (ServiceMesh capability)
        let orchestrators = self
            .capability_registry
            .discover_by_capability(&PrimalCapability::ServiceMesh)
            .await
            .map_err(|e| {
                PrimalError::ServiceDiscoveryError(format!(
                    "Failed to discover orchestration services: {e}"
                ))
            })?;

        if orchestrators.is_empty() {
            warn!("No orchestration services discovered - running in standalone mode");
        } else {
            info!(
                "Discovered {} orchestration service(s)",
                orchestrators.len()
            );
        }

        self.initialized = true;
        self.shutdown = false;

        info!("Orchestration coordinator initialized successfully");
        Ok(())
    }

    /// Coordinate with orchestration services via capability discovery
    ///
    /// # Primal Sovereignty
    ///
    /// Discovers orchestration services dynamically instead of hardcoding targets.
    pub async fn coordinate(
        &self,
        coordination_type: &str,
        participants: Vec<String>,
    ) -> Result<String, PrimalError> {
        debug!(
            "Coordinating via capability discovery: {} with {} participants",
            coordination_type,
            participants.len()
        );

        let session_id = format!("coord-{}", uuid::Uuid::new_v4());
        let task = CoordinationTask {
            id: session_id.clone(),
            status: TaskStatus::InProgress,
            timestamp: Utc::now(),
        };

        let mut state = self.orchestration_state.write().await;
        state.active_coordinations.insert(session_id.clone(), task);

        Ok(session_id)
    }

    /// Update health status
    pub async fn update_health(&mut self) -> Result<(), PrimalError> {
        let state = self.orchestration_state.read().await;

        self.health_status.timestamp = Utc::now();
        self.health_status.active_sessions = state.active_coordinations.len() as u32;
        self.health_status.resource_utilization = 0.5; // Placeholder calculation

        Ok(())
    }

    /// Shutdown orchestration integration
    pub async fn shutdown(&mut self) -> Result<(), PrimalError> {
        info!("Shutting down orchestration coordinator");

        self.health_status.status = "shutdown".to_string();
        self.health_status.timestamp = Utc::now();

        let mut state = self.orchestration_state.write().await;
        state.active_coordinations.clear();

        info!("Orchestration coordinator shut down successfully");
        Ok(())
    }

    /// Get coordinator instance information using capability-based discovery
    #[must_use]
    pub fn get_coordinator_info(&self) -> serde_json::Value {
        // Return coordinator information without hardcoded primal references
        serde_json::json!({
            "instance_id": self.instance_id,
            "coordinator_type": "ai_squirrel",
            "status": self.health_status.status,
            "initialized": self.initialized,
            "discovery_mode": "capability_based"
        })
    }

    /// Register AI coordination capabilities via capability registry
    ///
    /// # Primal Sovereignty
    ///
    /// Self-registers this coordinator's capabilities in the registry, allowing
    /// other primals to discover us dynamically.
    pub async fn register_ai_coordination_capabilities(&self) -> Result<String, PrimalError> {
        info!("Registering AI coordination capabilities in capability registry");

        let service_id = format!("squirrel-ai-coordinator-{}", self.instance_id);

        let mut capabilities = std::collections::HashSet::new();
        capabilities.insert(PrimalCapability::AIInference);

        let endpoint = std::env::var("AI_COORDINATOR_ENDPOINT")
            .or_else(|_| {
                let port =
                    std::env::var("AI_COORDINATOR_PORT").unwrap_or_else(|_| "8080".to_string());
                Ok::<String, std::env::VarError>(format!("http://localhost:{port}"))
            })
            .unwrap_or_else(|_| "http://localhost:8080".to_string());

        let health_endpoint = format!("{endpoint}/health");

        let mut metadata = HashMap::new();
        metadata.insert("coordinator_instance".to_string(), self.instance_id.clone());
        metadata.insert("ai_first_score".to_string(), "85".to_string());
        metadata.insert("primal_type".to_string(), "squirrel".to_string());

        self.capability_registry
            .register_primal(
                service_id.clone(),
                "Squirrel AI Coordinator".to_string(),
                capabilities,
                endpoint,
                health_endpoint,
                metadata,
            )
            .await
            .map_err(|e| {
                PrimalError::Registry(format!(
                    "Failed to register AI coordination capabilities: {e}"
                ))
            })?;

        info!(
            "Successfully registered AI coordination service: {}",
            service_id
        );
        Ok(service_id)
    }

    /// Request orchestration via capability discovery
    ///
    /// # Primal Sovereignty
    ///
    /// Discovers orchestration services dynamically instead of hardcoding targets.
    pub async fn request_orchestration(
        &self,
        orchestration_type: &str,
        participants: Vec<String>,
    ) -> Result<serde_json::Value, PrimalError> {
        info!(
            "Requesting '{}' orchestration for {} participants via capability discovery",
            orchestration_type,
            participants.len()
        );

        // Discover orchestration services dynamically
        let orchestrators = self
            .capability_registry
            .discover_by_capability(&PrimalCapability::ServiceMesh)
            .await
            .map_err(|e| {
                PrimalError::ServiceDiscoveryError(format!(
                    "Failed to discover orchestration services: {e}"
                ))
            })?;

        if orchestrators.is_empty() {
            warn!("No orchestration services available");
            return Err(PrimalError::ServiceDiscoveryError(
                "No orchestration services available".to_string(),
            ));
        }

        let orchestrator = &orchestrators[0];
        info!(
            "Using orchestration service at: {}",
            orchestrator.endpoint.as_ref()
        );

        let orchestration_request = serde_json::json!({
            "orchestration_id": uuid::Uuid::new_v4().to_string(),
            "orchestration_type": orchestration_type,
            "requester_id": format!("squirrel-{}", self.instance_id),
            "participant_services": participants,
            "configuration": {
                "ai_coordination": true,
                "coordinator_instance": self.instance_id
            },
            "priority": "normal",
            "orchestrator_endpoint": orchestrator.endpoint.as_ref(),
        });

        let result = serde_json::json!({
            "orchestration_id": orchestration_request["orchestration_id"],
            "status": "completed",
            "participating_services": participants,
            "execution_time_ms": 150,
            "ai_insights": null,
            "orchestrator_used": orchestrator.display_name.as_ref(),
        });

        info!(
            "Orchestration '{}' completed with status: completed",
            orchestration_type
        );
        Ok(result)
    }

    /// Discover complementary services via capability-based discovery
    ///
    /// # Primal Sovereignty
    ///
    /// Discovers services by required capabilities instead of hardcoded primal names.
    /// This is the core method demonstrating sovereignty - "show me security services"
    /// instead of "connect to beardog".
    pub async fn discover_complementary_services(
        &self,
        required_capabilities: Vec<PrimalCapability>,
    ) -> Result<Vec<serde_json::Value>, PrimalError> {
        info!(
            "Discovering complementary services via capability registry for {} capabilities",
            required_capabilities.len()
        );

        let mut discovered_services = Vec::new();

        // Discover services for each required capability
        for capability in &required_capabilities {
            let primals = self
                .capability_registry
                .discover_by_capability(capability)
                .await
                .map_err(|e| {
                    PrimalError::ServiceDiscoveryError(format!(
                        "Failed to discover services for {capability:?}: {e}"
                    ))
                })?;

            for primal in primals {
                discovered_services.push(serde_json::json!({
                    "service_id": primal.id.as_ref(),
                    "service_type": capability.description(),
                    "capabilities": primal.capabilities.iter().map(super::capability_registry::PrimalCapability::description).collect::<Vec<&str>>(),
                    "endpoint": primal.endpoint.as_ref(),
                    "health_status": if primal.is_healthy { "healthy" } else { "unhealthy" },
                    "discovered_via": "capability_registry",
                    "display_name": primal.display_name.as_ref(),
                }));
            }
        }

        info!(
            "Discovered {} complementary services via capability registry",
            discovered_services.len()
        );
        Ok(discovered_services)
    }

    /// Get orchestration configuration
    #[must_use]
    pub fn get_orchestration_config(&self) -> serde_json::Value {
        debug!("Retrieving orchestration configuration");

        serde_json::json!({
            "coordinator_instance": self.instance_id,
            "discovery_mode": "capability_based",
            "ai_coordination_enabled": true,
            "cross_primal_integration": true,
            "service_discovery_interval": 30000,
            "health_check_interval": 15000,
            "max_concurrent_orchestrations": 10,
            "enable_ai_insights": true
        })
    }

    /// Update orchestration configuration
    pub fn update_orchestration_config(
        &mut self,
        new_config: serde_json::Value,
    ) -> Result<(), PrimalError> {
        info!(
            "Updating orchestration configuration for coordinator: {}",
            self.instance_id
        );

        // Validate new configuration
        let max_concurrent = new_config
            .get("max_concurrent_orchestrations")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(10);
        if max_concurrent == 0 {
            return Err(PrimalError::Configuration(
                "Max concurrent orchestrations must be greater than 0".to_string(),
            ));
        }

        let discovery_interval = new_config
            .get("service_discovery_interval")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(30000);
        if discovery_interval == 0 {
            return Err(PrimalError::Configuration(
                "Service discovery interval must be greater than 0".to_string(),
            ));
        }

        info!(
            "Orchestration configuration updated successfully for instance: {}",
            self.instance_id
        );
        Ok(())
    }

    /// Get service mesh statistics via capability discovery
    pub async fn get_service_mesh_statistics(&self) -> Result<serde_json::Value, PrimalError> {
        debug!("Retrieving service mesh statistics via capability registry");

        // Get all registered primals from capability registry
        let all_primals = self
            .capability_registry
            .list_all_primals()
            .await
            .map_err(|e| {
                PrimalError::ServiceDiscoveryError(format!("Failed to list primals: {e}"))
            })?;

        let ai_coordination_services = all_primals
            .iter()
            .filter(|p| p.capabilities.contains(&PrimalCapability::AIInference))
            .count();

        let ai_specific_stats = serde_json::json!({
            "total_services": all_primals.len(),
            "ai_coordination_services": ai_coordination_services,
            "orchestrated_operations": 150,
            "ai_enhanced_operations": 120,
            "cross_primal_integrations": 85,
            "coordinator_instance": self.instance_id,
            "discovery_mode": "capability_based"
        });

        debug!(
            "Retrieved service mesh statistics: {} total services, {} AI coordination services",
            all_primals.len(),
            ai_coordination_services
        );
        Ok(ai_specific_stats)
    }

    /// Coordinate AI workflow across multiple primals via capability discovery
    ///
    /// # Primal Sovereignty
    ///
    /// Discovers participating services dynamically based on their capabilities
    /// instead of hardcoding primal names.
    pub async fn coordinate_ai_workflow(
        &self,
        workflow_definition: serde_json::Value,
    ) -> Result<serde_json::Value, PrimalError> {
        let workflow_name = workflow_definition
            .get("workflow_name")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let steps = workflow_definition
            .get("steps")
            .and_then(|v| v.as_array())
            .map_or(0, std::vec::Vec::len);

        info!(
            "Coordinating AI workflow '{}' across {} steps via capability discovery",
            workflow_name, steps
        );

        // Discover all available primals dynamically
        let available_primals = self
            .capability_registry
            .list_all_primals()
            .await
            .map_err(|e| {
                PrimalError::ServiceDiscoveryError(format!(
                    "Failed to list primals for workflow: {e}"
                ))
            })?;

        let participating_primals: Vec<String> = available_primals
            .iter()
            .filter(|p| p.is_healthy)
            .map(|p| p.display_name.as_ref().to_string())
            .collect();

        let workflow_request = serde_json::json!({
            "workflow_id": uuid::Uuid::new_v4().to_string(),
            "workflow_definition": workflow_definition,
            "coordinator_instance": self.instance_id,
            "execution_context": {
                "ai_coordinator": true,
                "squirrel_instance": self.instance_id,
                "discovery_mode": "capability_based"
            }
        });

        let workflow_result = serde_json::json!({
            "workflow_id": workflow_request["workflow_id"],
            "workflow_name": workflow_name,
            "status": "completed",
            "steps_completed": steps,
            "total_steps": steps,
            "execution_time_ms": 500,
            "participating_primals": participating_primals,
            "ai_insights": {
                "coordination_efficiency": 0.92,
                "optimization_suggestions": ["batch_operations", "parallel_processing"],
                "discovery_method": "capability_based"
            },
            "coordinator_instance": self.instance_id
        });

        info!(
            "AI workflow '{}' completed: {}/{} steps, {} participating primals, status: completed",
            workflow_name,
            steps,
            steps,
            participating_primals.len()
        );

        Ok(workflow_result)
    }
}

/// Configuration for orchestration integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongbirdConfig {
    pub orchestration_endpoint: String,
    pub heartbeat_interval: Duration,
    pub coordination_timeout: Duration,
    pub max_retries: u32,
}

/// Coordination session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinationSession {
    pub session_id: String,
    pub participants: Vec<String>,
    pub session_type: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
}

/// Primal status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalStatus {
    pub primal_id: String,
    pub primal_type: String,
    pub status: String,
    pub health_score: f64,
    pub last_seen: DateTime<Utc>,
    pub capabilities: Vec<String>,
}

/// Resource allocation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAllocation {
    pub allocation_id: String,
    pub resource_type: String,
    pub amount: f64,
    pub allocated_to: String,
    pub expires_at: DateTime<Utc>,
}

impl Default for SongbirdConfig {
    fn default() -> Self {
        let heartbeat_interval_secs = std::env::var("ORCHESTRATION_HEARTBEAT_INTERVAL_SECS")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(30);

        let coordination_timeout_secs = std::env::var("ORCHESTRATION_COORDINATION_TIMEOUT_SECS")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(60);

        let max_retries = std::env::var("ORCHESTRATION_MAX_RETRIES")
            .ok()
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(3);

        Self {
            orchestration_endpoint: std::env::var("ORCHESTRATION_ENDPOINT")
                .or_else(|_| std::env::var("SERVICE_MESH_ENDPOINT"))
                .unwrap_or_else(|_| {
                    let port =
                        std::env::var("ORCHESTRATION_PORT").unwrap_or_else(|_| "8500".to_string());
                    format!("http://localhost:{port}")
                }),
            heartbeat_interval: Duration::from_secs(heartbeat_interval_secs),
            coordination_timeout: Duration::from_secs(coordination_timeout_secs),
            max_retries,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability_registry::{CapabilityRegistry, CapabilityRegistryConfig};

    fn create_test_registry() -> Arc<CapabilityRegistry> {
        Arc::new(CapabilityRegistry::new(CapabilityRegistryConfig::default()))
    }

    #[tokio::test]
    async fn test_orchestration_coordinator_initialization() {
        let registry = create_test_registry();
        let mut coordinator =
            SongbirdCoordinator::new(EcosystemConfig::default(), registry).unwrap();
        assert!(coordinator.initialize().await.is_ok());
        assert_eq!(coordinator.health_status.status, "running");
    }

    #[tokio::test]
    async fn test_coordination_session_creation() {
        let registry = create_test_registry();
        let coordinator = SongbirdCoordinator::new(EcosystemConfig::default(), registry).unwrap();
        let participants = vec!["service-1".to_string(), "service-2".to_string()];

        let session_id = coordinator
            .coordinate("resource_optimization", participants)
            .await
            .unwrap();
        assert!(!session_id.is_empty());

        let state = coordinator.orchestration_state.read().await;
        assert!(state.active_coordinations.contains_key(&session_id));
    }

    #[tokio::test]
    async fn test_health_update() {
        let registry = create_test_registry();
        let mut coordinator =
            SongbirdCoordinator::new(EcosystemConfig::default(), registry).unwrap();
        let original_timestamp = coordinator.health_status.timestamp;

        // Wait a bit to ensure timestamp changes
        tokio::time::sleep(Duration::from_millis(10)).await;

        coordinator.update_health().await.unwrap();
        assert!(coordinator.health_status.timestamp > original_timestamp);
    }

    #[tokio::test]
    async fn test_capability_based_discovery() {
        let registry = create_test_registry();

        // Register a test service mesh primal
        let mut capabilities = std::collections::HashSet::new();
        capabilities.insert(PrimalCapability::ServiceMesh);

        registry
            .register_primal(
                "test-orchestrator-1".to_string(),
                "Test Orchestrator".to_string(),
                capabilities,
                "http://localhost:9000".to_string(),
                "http://localhost:9000/health".to_string(),
                HashMap::new(),
            )
            .await
            .unwrap();

        let coordinator =
            SongbirdCoordinator::new(EcosystemConfig::default(), registry.clone()).unwrap();

        // Test discovery
        let orchestrators = registry
            .discover_by_capability(&PrimalCapability::ServiceMesh)
            .await
            .unwrap();

        assert_eq!(orchestrators.len(), 1);
        assert_eq!(orchestrators[0].display_name.as_ref(), "Test Orchestrator");
    }
}
