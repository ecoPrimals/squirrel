use std::collections::HashMap;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use tracing::{error, info, instrument, warn};
use thiserror::Error;

use super::state::{State, StateError};
use super::persistence::{StatePersistence, PersistenceError};

#[derive(Debug, Error)]
pub enum RecoveryError {
    #[error("State error: {0}")]
    State(#[from] StateError),

    #[error("Persistence error: {0}")]
    Persistence(#[from] PersistenceError),

    #[error("Recovery point not found: {0}")]
    NotFound(String),

    #[error("Invalid recovery data: {0}")]
    InvalidData(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryPoint {
    pub id: Uuid,
    pub state_name: String,
    pub timestamp: DateTime<Utc>,
    pub state: State,
    pub metadata: RecoveryMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryMetadata {
    pub version: u64,
    pub reason: String,
    pub is_automatic: bool,
    pub dependencies: Vec<String>,
}

pub struct StateRecovery {
    recovery_points: RwLock<HashMap<String, Vec<RecoveryPoint>>>,
    max_points_per_state: usize,
    persistence: RwLock<StatePersistence>,
}

impl StateRecovery {
    pub fn new(persistence: StatePersistence, max_points_per_state: usize) -> Self {
        Self {
            recovery_points: RwLock::new(HashMap::new()),
            max_points_per_state,
            persistence: RwLock::new(persistence),
        }
    }

    #[instrument(skip(self, state))]
    pub async fn create_recovery_point(
        &self,
        state: State,
        reason: String,
        is_automatic: bool,
        dependencies: Vec<String>,
    ) -> Result<RecoveryPoint, RecoveryError> {
        let recovery_point = RecoveryPoint {
            id: Uuid::new_v4(),
            state_name: state.name.clone(),
            timestamp: Utc::now(),
            state: state.clone(),
            metadata: RecoveryMetadata {
                version: state.version,
                reason,
                is_automatic,
                dependencies,
            },
        };

        // Store recovery point
        let mut points = self.recovery_points.write().await;
        let state_points = points
            .entry(state.name.clone())
            .or_insert_with(Vec::new);

        // Add new point and maintain size limit
        state_points.push(recovery_point.clone());
        if state_points.len() > self.max_points_per_state {
            state_points.remove(0); // Remove oldest point
        }

        // Persist recovery point
        let mut persistence = self.persistence.write().await;
        persistence.save_state(state).await?;

        info!(
            state_name = %state.name,
            recovery_id = %recovery_point.id,
            "Recovery point created"
        );
        Ok(recovery_point)
    }

    #[instrument(skip(self))]
    pub async fn recover_state(
        &self,
        state_name: &str,
        point_id: Option<Uuid>,
    ) -> Result<State, RecoveryError> {
        let points = self.recovery_points.read().await;
        let state_points = points
            .get(state_name)
            .ok_or_else(|| RecoveryError::NotFound(state_name.to_string()))?;

        let recovery_point = if let Some(id) = point_id {
            // Find specific recovery point
            state_points
                .iter()
                .find(|p| p.id == id)
                .ok_or_else(|| RecoveryError::NotFound(format!("Recovery point {} not found", id)))?
        } else {
            // Use latest recovery point
            state_points
                .last()
                .ok_or_else(|| RecoveryError::NotFound(format!("No recovery points for {}", state_name)))?
        };

        // Restore state from recovery point
        let mut persistence = self.persistence.write().await;
        persistence.save_state(recovery_point.state.clone()).await?;

        info!(
            state_name = %state_name,
            recovery_id = %recovery_point.id,
            "State recovered from recovery point"
        );
        Ok(recovery_point.state.clone())
    }

    #[instrument(skip(self))]
    pub async fn list_recovery_points(&self, state_name: &str) -> Result<Vec<RecoveryPoint>, RecoveryError> {
        let points = self.recovery_points.read().await;
        Ok(points
            .get(state_name)
            .map(|p| p.clone())
            .unwrap_or_default())
    }

    #[instrument(skip(self))]
    pub async fn cleanup_old_points(&self, max_age_days: i64) -> Result<usize, RecoveryError> {
        let mut total_removed = 0;
        let mut points = self.recovery_points.write().await;
        let now = Utc::now();

        for points_vec in points.values_mut() {
            let original_len = points_vec.len();
            points_vec.retain(|point| {
                (now - point.timestamp).num_days() < max_age_days
            });
            total_removed += original_len - points_vec.len();
        }

        info!(points_removed = total_removed, "Cleaned up old recovery points");
        Ok(total_removed)
    }

    #[instrument(skip(self))]
    pub async fn verify_recovery_chain(&self, state_name: &str) -> Result<bool, RecoveryError> {
        let points = self.recovery_points.read().await;
        let state_points = points
            .get(state_name)
            .ok_or_else(|| RecoveryError::NotFound(state_name.to_string()))?;

        if state_points.is_empty() {
            return Ok(true);
        }

        // Verify version continuity
        for window in state_points.windows(2) {
            let prev = &window[0];
            let next = &window[1];

            if next.metadata.version <= prev.metadata.version {
                warn!(
                    state_name = %state_name,
                    prev_version = %prev.metadata.version,
                    next_version = %next.metadata.version,
                    "Recovery chain version mismatch"
                );
                return Ok(false);
            }
        }

        // Verify dependencies
        for point in state_points {
            for dep in &point.metadata.dependencies {
                if !points.contains_key(dep) {
                    warn!(
                        state_name = %state_name,
                        missing_dep = %dep,
                        "Recovery chain missing dependency"
                    );
                    return Ok(false);
                }
            }
        }

        Ok(true)
    }
} 