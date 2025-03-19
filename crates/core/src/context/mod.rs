//! # Context Module
//!
//! The Context module is responsible for managing application state and persistence.
//! It provides a comprehensive framework for handling context-based operations
//! throughout the application.
//!
//! ## Features
//!
//! - **State Management**: Track and manage application state changes
//! - **Persistence**: Store and retrieve state across application restarts
//! - **Synchronization**: Coordinate state across distributed systems
//! - **Recovery**: Handle state recovery after failures
//! - **State Tracking**: Monitor and audit state changes
//!
//! ## Architecture
//!
//! The context system is built around the concept of snapshots and state transitions.
//! Each state change is recorded, allowing for versioning, rollback, and audit capabilities.
//! The system supports both in-memory and persistent storage options.

use std::collections::VecDeque;
use std::time::SystemTime;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use thiserror::Error;
use std::fmt::Debug;
use uuid;

// Context module for managing application state and persistence
//
// This module provides functionality for:
// - State management
// - Persistence
// - Synchronization
// - Recovery
// - State tracking

/// Persistence functionality for storing and loading context state
pub mod persistence;
/// Synchronization functionality for distributed context state
pub mod sync;
/// Recovery functionality for handling context state failures
pub mod recovery;
/// Tracking functionality for monitoring context state changes
// pub mod tracker; // Temporarily commented out due to encoding issues

/// A snapshot of context state at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextSnapshot {
    /// Unique identifier for the snapshot
    pub id: String,
    /// Time when the snapshot was created
    pub timestamp: SystemTime,
    /// State data at the time of snapshot
    pub state: ContextState,
    /// Additional metadata about the snapshot
    pub metadata: Option<Value>,
}

/// State data for a context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextState {
    /// Version number of the state
    pub version: u64,
    /// State data
    pub data: Value,
    /// Time of last modification
    pub last_modified: SystemTime,
}

/// Errors that can occur during context operations
#[derive(Debug, Error)]
pub enum ContextError {
    /// Invalid context state
    #[error("Invalid context state: {0}")]
    InvalidState(String),
    
    /// Snapshot not found
    #[error("Snapshot not found: {0}")]
    SnapshotNotFound(String),
    
    /// No valid snapshot
    #[error("No valid snapshot: {0}")]
    NoValidSnapshot(String),
    
    /// Persistence error
    #[error("Persistence error: {0}")]
    PersistenceError(String),
    
    /// Synchronization error
    #[error("Synchronization error: {0}")]
    SyncError(String),
}

/// Context subscriber for monitoring context state changes
pub trait ContextSubscriber: Send + Sync + Debug {
    /// Called when the context state changes
    fn on_state_change(&self, old_state: &ContextState, new_state: &ContextState);
    
    /// Called when an error occurs
    fn on_error(&self, error: &ContextError);
}

/// Tracks context state changes and notifies subscribers
#[derive(Debug)]
pub struct ContextTracker {
    subscribers: Vec<Box<dyn ContextSubscriber>>,
    history: VecDeque<ContextSnapshot>,
    max_history: usize,
}

impl Default for ContextTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl ContextTracker {
    /// Creates a new context tracker
    #[must_use] pub fn new() -> Self {
        Self {
            subscribers: Vec::new(),
            history: VecDeque::new(),
            max_history: 100,
        }
    }

    /// Subscribes to context state changes and errors
    ///
    /// # Arguments
    /// * `subscriber` - The subscriber to add
    pub fn subscribe(&mut self, subscriber: Box<dyn ContextSubscriber>) {
        self.subscribers.push(subscriber);
    }

    /// Gets the history of context state changes
    ///
    /// # Returns
    /// * `&VecDeque<ContextSnapshot>` - The history of state changes
    #[must_use] pub const fn get_history(&self) -> &VecDeque<ContextSnapshot> {
        &self.history
    }

    /// Gets the current context state
    pub fn get_state(&self) -> Result<ContextState, ContextError> {
        // Use the first item in history as the current state
        if let Some(snapshot) = self.history.front() {
            Ok(snapshot.state.clone())
        } else {
            // Return default state if no history exists
            Ok(ContextState {
                version: 0,
                data: serde_json::Value::Null,
                last_modified: std::time::SystemTime::now(),
            })
        }
    }
    
    /// Updates the context state
    pub fn update_state(&mut self, data: serde_json::Value) -> Result<(), ContextError> {
        let next_version = match self.history.back() {
            Some(snapshot) => snapshot.state.version + 1,
            None => 1,
        };
        
        let state = ContextState {
            version: next_version,
            data,
            last_modified: std::time::SystemTime::now(),
        };
        
        let old_state = self.get_state().unwrap_or(ContextState {
            version: 0,
            data: serde_json::Value::Null,
            last_modified: std::time::SystemTime::now(),
        });
        
        let snapshot = ContextSnapshot {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: std::time::SystemTime::now(),
            state: state.clone(),
            metadata: None,
        };
        
        self.history.push_back(snapshot);
        
        if self.history.len() > self.max_history {
            self.history.pop_front();
        }
        
        // Notify subscribers of state change
        for subscriber in &self.subscribers {
            subscriber.on_state_change(&old_state, &state);
        }
        
        Ok(())
    }
    
    /// Rolls back the context state to a specific version
    pub fn rollback_to(&mut self, version: u64) -> Result<(), ContextError> {
        // Find the snapshot with the requested version
        let target_index = match self.history.iter().position(|s| s.state.version == version) {
            Some(index) => index,
            None => return Err(ContextError::NoValidSnapshot(format!("No snapshot with version {version} found"))),
        };
        
        // Keep snapshots up to and including the target version
        let snapshots_to_keep = target_index + 1;
        
        // Remove snapshots after the target version
        if snapshots_to_keep < self.history.len() {
            self.history.truncate(snapshots_to_keep);
        }
        
        Ok(())
    }
}

/// Data associated with a context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextData {
    /// Unique identifier for the context
    pub id: String,
    /// Display name of the context
    pub name: String,
    /// Additional metadata about the context
    pub metadata: Option<Value>,
    /// Context data
    pub data: Value,
}

impl Default for ContextData {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            metadata: None,
            data: Value::Null,
        }
    }
}

/// Context manager module for handling context instances and operations
pub mod manager;
/// State management module for handling context state transitions
pub mod state;
// pub mod tracker;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_state_serialization() {
        let state = ContextState {
            version: 1,
            data: serde_json::json!({
                "key": "value",
                "number": 42
            }),
            last_modified: SystemTime::now(),
        };

        let serialized = serde_json::to_string(&state).unwrap();
        let deserialized: ContextState = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.version, state.version);
        assert_eq!(deserialized.data, state.data);
    }

    #[test]
    fn test_context_snapshot_serialization() {
        let state = ContextState {
            version: 1,
            data: serde_json::json!({
                "key": "value"
            }),
            last_modified: SystemTime::now(),
        };

        let snapshot = ContextSnapshot {
            id: "test_snapshot".to_string(),
            timestamp: SystemTime::now(),
            state,
            metadata: Some(serde_json::json!({
                "description": "Test snapshot"
            })),
        };

        let serialized = serde_json::to_string(&snapshot).unwrap();
        let deserialized: ContextSnapshot = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.id, snapshot.id);
        assert_eq!(deserialized.state.version, snapshot.state.version);
        assert_eq!(deserialized.metadata, snapshot.metadata);
    }

    #[test]
    fn test_context_tracker_new() {
        let tracker = ContextTracker::new();
        let state = tracker.get_state().unwrap();
        assert_eq!(state.version, 0);
        assert!(matches!(state.data, serde_json::Value::Null));
    }

    #[test]
    fn test_context_tracker_update_state() {
        let mut tracker = ContextTracker::new();
        let test_data = serde_json::json!({
            "key": "value",
            "number": 42
        });

        // Test state update
        assert!(tracker.update_state(test_data.clone()).is_ok());
        
        let state = tracker.get_state().unwrap();
        assert_eq!(state.version, 1);
        assert_eq!(state.data, test_data);
    }

    #[test]
    fn test_context_tracker_history() {
        let mut tracker = ContextTracker::new();
        
        // Add multiple states
        for i in 0..5 {
            let data = serde_json::json!({ "index": i });
            tracker.update_state(data).unwrap();
        }

        let history = tracker.get_history();
        assert_eq!(history.len(), 5);
        
        // Verify history order
        for (i, snapshot) in history.iter().enumerate() {
            assert_eq!(snapshot.state.version, (i + 1) as u64);
            assert_eq!(snapshot.state.data["index"], i);
        }
    }

    #[test]
    fn test_context_tracker_rollback() {
        let mut tracker = ContextTracker::new();
        
        // Add multiple states
        for i in 0..3 {
            let data = serde_json::json!({ "index": i });
            tracker.update_state(data).unwrap();
        }

        // Rollback to version 2
        assert!(tracker.rollback_to(2).is_ok());
        let state = tracker.get_state().unwrap();
        assert_eq!(state.version, 1);
        assert_eq!(state.data["index"], 0);

        // Test invalid rollback
        assert!(tracker.rollback_to(10).is_err());
    }

    #[test]
    fn test_context_tracker_subscribers() {
        use std::sync::atomic::{AtomicU64, Ordering};
        use std::sync::Arc;

        let mut tracker = ContextTracker::new();
        let counter = Arc::new(AtomicU64::new(0));
        let counter_clone = counter.clone();

        // Create a subscriber that counts state changes
        #[derive(Debug)]
        struct TestSubscriber {
            counter: Arc<AtomicU64>,
        }

        impl ContextSubscriber for TestSubscriber {
            fn on_state_change(&self, _old_state: &ContextState, _new_state: &ContextState) {
                self.counter.fetch_add(1, Ordering::SeqCst);
            }

            fn on_error(&self, _error: &ContextError) {}
        }

        // Add subscriber
        tracker.subscribe(Box::new(TestSubscriber { counter: counter_clone }));

        // Update state multiple times
        for i in 0..3 {
            let data = serde_json::json!({ "index": i });
            tracker.update_state(data).unwrap();
        }

        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn test_context_error_handling() {
        let mut tracker = ContextTracker::new();
        
        // Test invalid state error
        let result = tracker.rollback_to(999);
        assert!(matches!(result, Err(ContextError::NoValidSnapshot(_))));
    }
} 