//! Songbird integration for squirrel primal
//!
//! This module provides integration with the songbird orchestration system.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use std::time::SystemTime;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::ecosystem::EcosystemConfig;
use crate::error::PrimalError;
use crate::universal::{PrimalCapability, PrimalRequest, UniversalResult};
use squirrel_mcp_config::DefaultConfigManager;

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

/// Songbird service coordination module
pub struct SongbirdCoordinator {
    /// Instance identifier
    instance_id: String,
    /// Ecosystem configuration
    config: EcosystemConfig,
    /// Service mesh client
    service_mesh_client: Arc<Box<dyn ecosystem_api::traits::ServiceMeshClient + Send + Sync>>,
    /// Configuration manager
    config_manager: DefaultConfigManager,
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
    /// Create a new Songbird coordinator
    pub fn new(config: EcosystemConfig) -> Result<Self, PrimalError> {
        let instance_id = Uuid::new_v4().to_string();
        let service_mesh_client = Arc::new(Box::new(
            ecosystem_api::SongbirdClient::new(
                config.registry_config.songbird_endpoint.clone(),
                None,
                ecosystem_api::traits::RetryConfig::default(),
            )
            .map_err(|e| {
                PrimalError::NetworkError(format!("Failed to create Songbird client: {}", e))
            })?,
        ) as Box<dyn ecosystem_api::traits::ServiceMeshClient + Send + Sync>);
        let config_manager = DefaultConfigManager::new();
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
            service_mesh_client,
            config_manager,
            orchestration_state,
            health_status,
            initialized: false,
            shutdown: false,
        })
    }

    /// Initialize songbird integration
    pub async fn initialize(&mut self) -> Result<(), PrimalError> {
        info!("Initializing songbird integration");

        self.initialized = true;
        self.shutdown = false;

        info!("Songbird integration initialized successfully");
        Ok(())
    }

    /// Coordinate with songbird
    pub async fn coordinate(
        &self,
        coordination_type: &str,
        participants: Vec<String>,
    ) -> Result<String, PrimalError> {
        debug!(
            "Coordinating with songbird: {} with participants: {:?}",
            coordination_type, participants
        );

        let session_id = format!("songbird-coord-{}", uuid::Uuid::new_v4());
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

    /// Shutdown songbird integration
    pub async fn shutdown(&mut self) -> Result<(), PrimalError> {
        info!("Shutting down songbird integration");

        self.health_status.status = "shutdown".to_string();
        self.health_status.timestamp = Utc::now();

        let mut state = self.orchestration_state.write().await;
        state.active_coordinations.clear();

        info!("Songbird integration shut down successfully");
        Ok(())
    }
}

/// Configuration for songbird integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongbirdConfig {
    pub songbird_endpoint: String,
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
        let heartbeat_interval_secs = std::env::var("SONGBIRD_HEARTBEAT_INTERVAL_SECS")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(30);

        let coordination_timeout_secs = std::env::var("SONGBIRD_COORDINATION_TIMEOUT_SECS")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(60);

        let max_retries = std::env::var("SONGBIRD_MAX_RETRIES")
            .ok()
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(3);

        Self {
            songbird_endpoint: std::env::var("SONGBIRD_ENDPOINT")
                .unwrap_or_else(|_| "http://localhost:8080".to_string()),
            heartbeat_interval: Duration::from_secs(heartbeat_interval_secs),
            coordination_timeout: Duration::from_secs(coordination_timeout_secs),
            max_retries,
        }
    }
}

impl Default for SongbirdCoordinator {
    fn default() -> Self {
        Self::new(EcosystemConfig::default()).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_songbird_integration_initialization() {
        let mut integration = SongbirdCoordinator::new(EcosystemConfig::default()).unwrap();
        assert!(integration.initialize().await.is_ok());
        assert_eq!(integration.health_status.status, "running");
    }

    #[tokio::test]
    async fn test_coordination_session_creation() {
        let integration = SongbirdCoordinator::new(EcosystemConfig::default()).unwrap();
        let participants = vec!["squirrel".to_string(), "toadstool".to_string()];

        let session_id = integration
            .coordinate("resource_optimization", participants)
            .await
            .unwrap();
        assert!(!session_id.is_empty());

        let state = integration.orchestration_state.read().await;
        assert!(state.active_coordinations.contains_key(&session_id));
    }

    #[tokio::test]
    async fn test_health_update() {
        let mut integration = SongbirdCoordinator::new(EcosystemConfig::default()).unwrap();
        let original_timestamp = integration.health_status.timestamp;

        // Wait a bit to ensure timestamp changes
        tokio::time::sleep(Duration::from_millis(10)).await;

        integration.update_health().await.unwrap();
        assert!(integration.health_status.timestamp > original_timestamp);
    }
}
