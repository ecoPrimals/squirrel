//! Context tracking functionality
//!
//! This module provides functionality for tracking context changes.

use std::sync::{Arc, Mutex};
use crate::{ContextError, ContextState};

/// Context tracker for managing state changes
#[derive(Debug)]
pub struct ContextTracker {
    /// Current state of the context
    state: Arc<Mutex<ContextState>>,
}

impl ContextTracker {
    /// Create a new context tracker with the given state
    #[must_use]
    pub fn new(state: ContextState) -> Self {
        Self {
            state: Arc::new(Mutex::new(state)),
        }
    }

    /// Get the current state
    pub fn get_state(&self) -> Result<ContextState, ContextError> {
        let state = self.state.lock()
            .map_err(|_| ContextError::InvalidState("Failed to acquire state lock".to_string()))?;
        Ok(state.clone())
    }

    /// Update the current state
    pub fn update_state(&self, state: ContextState) -> Result<(), ContextError> {
        let mut current_state = self.state.lock()
            .map_err(|_| ContextError::InvalidState("Failed to acquire state lock".to_string()))?;
        
        // Only update if the new state has a higher version
        if state.version > current_state.version {
            *current_state = state;
        }
        
        Ok(())
    }
} 