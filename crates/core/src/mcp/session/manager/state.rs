use std::collections::HashMap;
use tokio::sync::{RwLock, mpsc};
use tracing::{error, info, instrument, warn};
use thiserror::Error;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use std::path::PathBuf;

use super::persistence::{StatePersistence, PersistenceError};
use super::recovery::{StateRecovery, RecoveryError, RecoveryPoint};

#[derive(Debug, Error)]
pub enum StateError {
    #[error("Invalid state transition: from {from} to {to}")]
    InvalidTransition { from: String, to: String },

    #[error("State validation error: {0}")]
    ValidationError(String),

    #[error("State not found: {0}")]
    NotFound(String),

    #[error("Persistence error: {0}")]
    Persistence(#[from] PersistenceError),

    #[error("Recovery error: {0}")]
    Recovery(#[from] RecoveryError),

    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    pub id: Uuid,
    pub name: String,
    pub data: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateTransition {
    pub from_state: String,
    pub to_state: String,
    pub conditions: Vec<String>,
    pub validation_rules: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateSyncMessage {
    pub id: Uuid,
    pub state: State,
    pub version: u64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateValidationRule {
    pub name: String,
    pub condition: String,
    pub error_message: String,
}

#[derive(Debug)]
pub struct StateManager {
    states: RwLock<HashMap<String, State>>,
    transitions: RwLock<HashMap<String, Vec<StateTransition>>>,
    history: RwLock<Vec<StateHistoryEntry>>,
    validation_rules: RwLock<Vec<StateValidationRule>>,
    sync_tx: mpsc::Sender<StateSyncMessage>,
    sync_rx: mpsc::Receiver<StateSyncMessage>,
    persistence: RwLock<StatePersistence>,
    recovery: RwLock<StateRecovery>,
}

impl Clone for StateManager {
    fn clone(&self) -> Self {
        let (tx, rx) = mpsc::channel(100);
        let persistence = StatePersistence::new("states");
        Self {
            states: RwLock::new(HashMap::new()),
            transitions: RwLock::new(HashMap::new()),
            history: RwLock::new(Vec::new()),
            validation_rules: RwLock::new(Vec::new()),
            sync_tx: tx,
            sync_rx: rx,
            persistence: RwLock::new(persistence.clone()),
            recovery: RwLock::new(StateRecovery::new(persistence, 10)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StateHistoryEntry {
    id: Uuid,
    from_state: String,
    to_state: String,
    timestamp: DateTime<Utc>,
    metadata: Option<serde_json::Value>,
}

impl StateManager {
    #[instrument]
    pub fn new() -> Self {
        info!("Initializing MCP state manager");
        let (tx, rx) = mpsc::channel(100);
        let persistence = StatePersistence::new("states");
        
        Self {
            states: RwLock::new(HashMap::new()),
            transitions: RwLock::new(HashMap::new()),
            history: RwLock::new(Vec::new()),
            validation_rules: RwLock::new(Vec::new()),
            sync_tx: tx,
            sync_rx: rx,
            persistence: RwLock::new(persistence.clone()),
            recovery: RwLock::new(StateRecovery::new(persistence, 10)),
        }
    }

    #[instrument]
    pub fn with_storage_path<P: Into<PathBuf>>(storage_path: P) -> Self {
        info!("Initializing MCP state manager with custom storage path");
        let (tx, rx) = mpsc::channel(100);
        let persistence = StatePersistence::new(storage_path.into());
        
        Self {
            states: RwLock::new(HashMap::new()),
            transitions: RwLock::new(HashMap::new()),
            history: RwLock::new(Vec::new()),
            validation_rules: RwLock::new(Vec::new()),
            sync_tx: tx,
            sync_rx: rx,
            persistence: RwLock::new(persistence.clone()),
            recovery: RwLock::new(StateRecovery::new(persistence, 10)),
        }
    }

    #[instrument(skip(self, state))]
    pub async fn register_state(&self, name: String, state: State) -> Result<(), StateError> {
        let mut states = self.states.write().await;
        states.insert(name.clone(), state.clone());
        
        // Persist state
        let mut persistence = self.persistence.write().await;
        persistence.save_state(state.clone()).await?;

        // Create initial recovery point
        let mut recovery = self.recovery.write().await;
        recovery.create_recovery_point(
            state,
            "Initial state registration".to_string(),
            true,
            Vec::new(),
        ).await?;
        
        info!(state_name = %name, "State registered, persisted, and recovery point created");
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn register_transition(&self, transition: StateTransition) -> Result<(), StateError> {
        let mut transitions = self.transitions.write().await;
        transitions
            .entry(transition.from_state.clone())
            .or_insert_with(Vec::new)
            .push(transition.clone());

        info!(
            from_state = %transition.from_state,
            to_state = %transition.to_state,
            "State transition registered"
        );
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn transition_state(
        &self,
        from_state: &str,
        to_state: &str,
        metadata: Option<serde_json::Value>,
    ) -> Result<(), StateError> {
        // Validate transition
        self.validate_transition(from_state, to_state).await?;

        // Update state
        let mut states = self.states.write().await;
        let state = states.get_mut(to_state).ok_or(StateError::NotFound(to_state.to_string()))?;
        state.updated_at = Utc::now();

        // Record history
        let history_entry = StateHistoryEntry {
            id: Uuid::new_v4(),
            from_state: from_state.to_string(),
            to_state: to_state.to_string(),
            timestamp: Utc::now(),
            metadata: metadata.clone(),
        };

        let mut history = self.history.write().await;
        history.push(history_entry.clone());

        // Create recovery point
        let mut recovery = self.recovery.write().await;
        recovery.create_recovery_point(
            state.clone(),
            format!("Transition from {} to {}", from_state, to_state),
            true,
            vec![from_state.to_string()],
        ).await?;

        // Persist updated state
        let mut persistence = self.persistence.write().await;
        persistence.save_state(state.clone()).await?;

        info!(
            from_state = %from_state,
            to_state = %to_state,
            "State transition completed with recovery point"
        );
        Ok(())
    }

    #[instrument(skip(self))]
    async fn validate_transition(&self, from_state: &str, to_state: &str) -> Result<(), StateError> {
        let transitions = self.transitions.read().await;
        
        if let Some(valid_transitions) = transitions.get(from_state) {
            if !valid_transitions.iter().any(|t| t.to_state == to_state) {
                return Err(StateError::InvalidTransition {
                    from: from_state.to_string(),
                    to: to_state.to_string(),
                });
            }
        } else {
            return Err(StateError::NotFound(from_state.to_string()));
        }

        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn get_state(&self, name: &str) -> Result<State, StateError> {
        // Try in-memory cache first
        let states = self.states.read().await;
        if let Some(state) = states.get(name) {
            return Ok(state.clone());
        }

        // Load from persistence
        let mut persistence = self.persistence.write().await;
        let state = persistence.load_state(name).await?;

        // Update in-memory cache
        drop(states);
        let mut states = self.states.write().await;
        states.insert(name.to_string(), state.clone());

        Ok(state)
    }

    #[instrument(skip(self))]
    pub async fn get_valid_transitions(&self, from_state: &str) -> Result<Vec<String>, StateError> {
        let transitions = self.transitions.read().await;
        Ok(transitions
            .get(from_state)
            .map(|t| t.iter().map(|t| t.to_state.clone()).collect())
            .unwrap_or_default())
    }

    #[instrument(skip(self))]
    pub async fn get_history(&self) -> Vec<StateHistoryEntry> {
        self.history.read().await.clone()
    }

    #[instrument(skip(self))]
    pub async fn sync_state(&self, state_name: &str) -> Result<(), StateError> {
        let states = self.states.read().await;
        let state = states.get(state_name)
            .ok_or(StateError::NotFound(state_name.to_string()))?
            .clone();

        let sync_message = StateSyncMessage {
            id: state.id,
            state: state.clone(),
            version: state.version,
            timestamp: Utc::now(),
        };

        self.sync_tx.send(sync_message).await.map_err(|e| {
            error!("Failed to send sync message: {}", e);
            StateError::ValidationError(format!("Sync failed: {}", e))
        })?;

        info!(state_name = %state_name, "State synchronized");
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn handle_sync_message(&self, message: StateSyncMessage) -> Result<(), StateError> {
        let mut states = self.states.write().await;
        
        // Check if we have a newer version
        if let Some(existing_state) = states.get(&message.state.name) {
            if existing_state.version >= message.version {
                warn!(
                    state_name = %message.state.name,
                    existing_version = %existing_state.version,
                    received_version = %message.version,
                    "Received older state version, ignoring"
                );
                return Ok(());
            }
        }

        // Update state
        states.insert(message.state.name.clone(), message.state);
        info!(state_name = %message.state.name, "State updated from sync");
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn add_validation_rule(&self, rule: StateValidationRule) -> Result<(), StateError> {
        let mut rules = self.validation_rules.write().await;
        rules.push(rule);
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn validate_state(&self, state: &State) -> Result<(), StateError> {
        let rules = self.validation_rules.read().await;
        
        for rule in rules.iter() {
            // Here we would evaluate the rule condition
            // For now, we'll just log that validation occurred
            info!(
                state_name = %state.name,
                rule_name = %rule.name,
                "Validating state"
            );
        }

        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn start_sync_handler(&self) -> Result<(), StateError> {
        let state_manager = self.clone();
        
        tokio::spawn(async move {
            info!("Starting state sync handler");
            
            while let Some(sync_message) = state_manager.sync_rx.recv().await {
                if let Err(e) = state_manager.handle_sync_message(sync_message).await {
                    error!("Failed to handle sync message: {}", e);
                }
            }
            
            warn!("State sync handler stopped");
        });

        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn load_persisted_states(&self) -> Result<(), StateError> {
        let mut persistence = self.persistence.write().await;
        let state_names = persistence.list_states().await?;

        let mut states = self.states.write().await;
        for name in state_names {
            let state = persistence.load_state(&name).await?;
            states.insert(name.clone(), state);
            info!(state_name = %name, "State loaded from persistence");
        }

        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn recover_state(&self, state_name: &str, point_id: Option<Uuid>) -> Result<State, StateError> {
        let recovery = self.recovery.read().await;
        let state = recovery.recover_state(state_name, point_id).await?;

        // Update in-memory state
        let mut states = self.states.write().await;
        states.insert(state_name.to_string(), state.clone());

        info!(
            state_name = %state_name,
            point_id = ?point_id,
            "State recovered from recovery point"
        );
        Ok(state)
    }

    #[instrument(skip(self))]
    pub async fn list_recovery_points(&self, state_name: &str) -> Result<Vec<RecoveryPoint>, StateError> {
        let recovery = self.recovery.read().await;
        Ok(recovery.list_recovery_points(state_name).await?)
    }

    #[instrument(skip(self))]
    pub async fn verify_state_integrity(&self, state_name: &str) -> Result<bool, StateError> {
        let recovery = self.recovery.read().await;
        Ok(recovery.verify_recovery_chain(state_name).await?)
    }

    #[instrument(skip(self))]
    pub async fn cleanup_recovery_points(&self, max_age_days: i64) -> Result<usize, StateError> {
        let recovery = self.recovery.write().await;
        Ok(recovery.cleanup_old_points(max_age_days).await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_state_registration() {
        let manager = StateManager::new();
        let state = State {
            id: Uuid::new_v4(),
            name: "test_state".to_string(),
            data: serde_json::json!({}),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            metadata: None,
        };

        assert!(manager.register_state("test_state".to_string(), state).await.is_ok());
    }

    #[tokio::test]
    async fn test_transition_registration() {
        let manager = StateManager::new();
        let transition = StateTransition {
            from_state: "state1".to_string(),
            to_state: "state2".to_string(),
            conditions: vec![],
            validation_rules: vec![],
        };

        assert!(manager.register_transition(transition).await.is_ok());
    }

    #[tokio::test]
    async fn test_invalid_transition() {
        let manager = StateManager::new();
        let result = manager.transition_state("invalid", "state", None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_state_history() {
        let manager = StateManager::new();
        let state1 = State {
            id: Uuid::new_v4(),
            name: "state1".to_string(),
            data: serde_json::json!({}),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            metadata: None,
        };
        let state2 = State {
            id: Uuid::new_v4(),
            name: "state2".to_string(),
            data: serde_json::json!({}),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            metadata: None,
        };

        manager.register_state("state1".to_string(), state1).await.unwrap();
        manager.register_state("state2".to_string(), state2).await.unwrap();
        manager.register_transition(StateTransition {
            from_state: "state1".to_string(),
            to_state: "state2".to_string(),
            conditions: vec![],
            validation_rules: vec![],
        }).await.unwrap();

        manager.transition_state("state1", "state2", None).await.unwrap();
        let history = manager.get_history().await;
        assert_eq!(history.len(), 1);
    }
} 