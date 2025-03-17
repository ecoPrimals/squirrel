use std::collections::VecDeque;
use std::sync::{Arc, RwLock};
use std::time::SystemTime;
use serde_json::Value;
use super::{ContextState, ContextError, ContextSnapshot, ContextSubscriber};

/// Tracks and manages context state changes and history
pub struct ContextTracker {
    state: Arc<RwLock<ContextState>>,
    history: VecDeque<ContextSnapshot>,
    subscribers: Vec<Box<dyn ContextSubscriber>>,
}

impl ContextTracker {
    /// Creates a new context tracker with an empty initial state
    #[must_use] pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(ContextState {
                version: 0,
                data: Value::Null,
                last_modified: SystemTime::now(),
            })),
            history: VecDeque::with_capacity(100), // Keep last 100 snapshots
            subscribers: Vec::new(),
        }
    }

    /// Registers a subscriber to be notified of context state changes
    ///
    /// # Arguments
    /// * `subscriber` - The subscriber implementation to register
    pub fn subscribe(&mut self, subscriber: Box<dyn ContextSubscriber>) {
        self.subscribers.push(subscriber);
    }

    /// Updates the context state with new data
    ///
    /// # Arguments
    /// * `new_data` - The new data to update the context with
    ///
    /// # Errors
    ///
    /// Returns a `ContextError` if:
    /// - Unable to acquire the write lock for the state
    /// - The state update fails
    pub fn update_state(&mut self, new_data: Value) -> Result<(), ContextError> {
        let mut state = self.state.write().map_err(|_| {
            ContextError::InvalidState("Failed to acquire write lock".to_string())
        })?;

        let old_state = state.clone();
        
        // Update state
        state.version += 1;
        state.data = new_data;
        state.last_modified = SystemTime::now();

        // Create snapshot
        let snapshot = ContextSnapshot {
            id: format!("snapshot_{}", state.version),
            timestamp: state.last_modified,
            state: state.clone(),
            metadata: None,
        };

        // Add to history
        if self.history.len() >= 100 {
            self.history.pop_front();
        }
        self.history.push_back(snapshot);

        // Notify subscribers
        for subscriber in &self.subscribers {
            subscriber.on_state_change(&old_state, &state);
        }

        Ok(())
    }

    /// Gets the current context state
    ///
    /// # Returns
    /// The current context state if successful
    ///
    /// # Errors
    ///
    /// Returns a `ContextError` if:
    /// - Unable to acquire the read lock for the state
    pub fn get_state(&self) -> Result<ContextState, ContextError> {
        self.state.read()
            .map(|state| state.clone())
            .map_err(|_| ContextError::InvalidState("Failed to acquire read lock".to_string()))
    }

    /// Returns the history of context state changes
    ///
    /// # Returns
    /// A reference to the collection of historical snapshots
    #[must_use] pub const fn get_history(&self) -> &VecDeque<ContextSnapshot> {
        &self.history
    }

    /// Rolls back the context state to a specific version
    ///
    /// # Arguments
    /// * `version` - The version number to roll back to
    ///
    /// # Errors
    ///
    /// Returns a `ContextError` if:
    /// - The specified version is not found in the history
    /// - Unable to acquire the write lock for the state
    pub fn rollback_to(&mut self, version: u64) -> Result<(), ContextError> {
        if let Some(snapshot) = self.history.iter().find(|s| s.state.version == version) {
            let mut state = self.state.write().map_err(|_| {
                ContextError::InvalidState("Failed to acquire write lock".to_string())
            })?;

            let old_state = state.clone();
            *state = snapshot.state.clone();

            // Notify subscribers
            for subscriber in &self.subscribers {
                subscriber.on_state_change(&old_state, &state);
            }

            Ok(())
        } else {
            Err(ContextError::InvalidState(format!("Version {version} not found")))
        }
    }
}

impl Default for ContextTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_tracker_new() {
        let tracker = ContextTracker::new();
        let state = tracker.get_state().unwrap();
        assert_eq!(state.version, 0);
        assert!(matches!(state.data, Value::Null));
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
        assert_eq!(state.version, 2);
        assert_eq!(state.data["index"], 1);

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
        assert!(matches!(result, Err(ContextError::InvalidState(_))));
    }
} 