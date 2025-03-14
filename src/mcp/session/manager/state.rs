use std::collections::HashMap;
use tokio::sync::RwLock;
use tracing::{error, info, instrument, warn};
use thiserror::Error;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use serde::{Serialize, Deserialize};

#[derive(Debug, Error)]
pub enum StateError {
    #[error("Invalid state transition: from {from} to {to}")]
    InvalidTransition { from: String, to: String },

    #[error("State validation error: {0}")]
    ValidationError(String),

    #[error("State not found: {0}")]
    NotFound(String),

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

#[derive(Debug)]
pub struct StateManager {
    states: RwLock<HashMap<String, State>>,
    transitions: RwLock<HashMap<String, Vec<StateTransition>>>,
    history: RwLock<Vec<StateHistoryEntry>>,
}

impl Clone for StateManager {
    fn clone(&self) -> Self {
        Self {
            states: RwLock::new(HashMap::new()),
            transitions: RwLock::new(HashMap::new()),
            history: RwLock::new(Vec::new()),
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
        
        Self {
            states: RwLock::new(HashMap::new()),
            transitions: RwLock::new(HashMap::new()),
            history: RwLock::new(Vec::new()),
        }
    }

    #[instrument(skip(self, state))]
    pub async fn register_state(&self, name: String, state: State) -> Result<(), StateError> {
        let mut states = self.states.write().await;
        states.insert(name.clone(), state);
        info!(state_name = %name, "State registered");
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
            metadata,
        };

        let mut history = self.history.write().await;
        history.push(history_entry);

        info!(
            from_state = %from_state,
            to_state = %to_state,
            "State transition completed"
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
        let states = self.states.read().await;
        states
            .get(name)
            .cloned()
            .ok_or(StateError::NotFound(name.to_string()))
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