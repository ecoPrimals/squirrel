//! Songbird integration for squirrel primal
//!
//! This module provides integration with the songbird orchestration system.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, info};

use crate::error::PrimalError;

/// Songbird integration for orchestration
#[derive(Debug)]
pub struct SongbirdIntegration {
    pub config: SongbirdConfig,
    pub orchestration_state: Arc<RwLock<OrchestrationState>>,
    pub health_status: HealthStatus,
}

/// Configuration for songbird integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongbirdConfig {
    pub songbird_endpoint: String,
    pub heartbeat_interval: Duration,
    pub coordination_timeout: Duration,
    pub max_retries: u32,
}

/// Orchestration state
#[derive(Debug, Clone)]
#[derive(Default)]
pub struct OrchestrationState {
    pub active_coordinations: HashMap<String, CoordinationSession>,
    pub primal_status: HashMap<String, PrimalStatus>,
    pub resource_allocations: HashMap<String, ResourceAllocation>,
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

/// Health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub status: String,
    pub timestamp: DateTime<Utc>,
    pub coordinator_status: String,
    pub active_sessions: u32,
    pub resource_utilization: f64,
}

impl SongbirdIntegration {
    pub fn new() -> Self {
        Self {
            config: SongbirdConfig::default(),
            orchestration_state: Arc::new(RwLock::new(OrchestrationState::default())),
            health_status: HealthStatus {
                status: "initializing".to_string(),
                timestamp: Utc::now(),
                coordinator_status: "starting".to_string(),
                active_sessions: 0,
                resource_utilization: 0.0,
            },
        }
    }

    /// Initialize songbird integration
    pub async fn initialize(&mut self) -> Result<(), PrimalError> {
        info!("Initializing songbird integration");

        self.health_status.status = "running".to_string();
        self.health_status.coordinator_status = "running".to_string();
        self.health_status.timestamp = Utc::now();

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
        let session = CoordinationSession {
            session_id: session_id.clone(),
            participants,
            session_type: coordination_type.to_string(),
            status: "active".to_string(),
            created_at: Utc::now(),
            last_activity: Utc::now(),
        };

        let mut state = self.orchestration_state.write().await;
        state
            .active_coordinations
            .insert(session_id.clone(), session);

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

impl Default for SongbirdConfig {
    fn default() -> Self {
        Self {
            songbird_endpoint: "http://localhost:8080".to_string(),
            heartbeat_interval: Duration::from_secs(30),
            coordination_timeout: Duration::from_secs(60),
            max_retries: 3,
        }
    }
}


impl Default for SongbirdIntegration {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_songbird_integration_initialization() {
        let mut integration = SongbirdIntegration::new();
        assert!(integration.initialize().await.is_ok());
        assert_eq!(integration.health_status.status, "running");
    }

    #[tokio::test]
    async fn test_coordination_session_creation() {
        let integration = SongbirdIntegration::new();
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
        let mut integration = SongbirdIntegration::new();
        let original_timestamp = integration.health_status.timestamp;

        // Wait a bit to ensure timestamp changes
        tokio::time::sleep(Duration::from_millis(10)).await;

        integration.update_health().await.unwrap();
        assert!(integration.health_status.timestamp > original_timestamp);
    }
}
