use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use uuid::Uuid;
use super::{ContextState, ContextError, ContextSnapshot};
use super::persistence::ContextPersistence;

/// Defines a strategy for selecting a context snapshot for recovery
///
/// This trait allows for different strategies to be implemented for
/// selecting which snapshot to use when recovering context state.
pub trait RecoveryStrategy {
    /// Selects a context snapshot from a collection of snapshots
    ///
    /// # Arguments
    /// * `snapshots` - A slice of available context snapshots
    ///
    /// # Returns
    /// The selected snapshot, or None if no suitable snapshot is found
    fn select_state<'a>(&self, snapshots: &'a [ContextSnapshot]) -> Option<&'a ContextSnapshot>;
}

/// Strategy that selects the most recent context snapshot by version number
pub struct LatestVersionStrategy;

impl LatestVersionStrategy {
    /// Creates a new instance of the latest version recovery strategy
    #[must_use] pub const fn new() -> Self {
        Self
    }
}

impl Default for LatestVersionStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl RecoveryStrategy for LatestVersionStrategy {
    fn select_state<'a>(&self, snapshots: &'a [ContextSnapshot]) -> Option<&'a ContextSnapshot> {
        snapshots.iter().max_by_key(move |s| s.state.version)
    }
}

/// Strategy that selects a context snapshot with a specific version number
pub struct SpecificVersionStrategy {
    version: u64,
}

impl SpecificVersionStrategy {
    /// Creates a new instance of the specific version recovery strategy
    ///
    /// # Arguments
    /// * `version` - The specific version number to recover
    #[must_use] pub const fn new(version: u64) -> Self {
        Self { version }
    }
}

impl RecoveryStrategy for SpecificVersionStrategy {
    fn select_state<'a>(&self, snapshots: &'a [ContextSnapshot]) -> Option<&'a ContextSnapshot> {
        snapshots.iter().find(move |s| s.state.version == self.version)
    }
}

/// Strategy that selects the most recent context snapshot before a specific timestamp
pub struct TimeBasedStrategy {
    timestamp: SystemTime,
}

impl TimeBasedStrategy {
    /// Creates a new instance of the time-based recovery strategy
    ///
    /// # Arguments
    /// * `timestamp` - The timestamp to use as the upper bound for recovery
    #[must_use] pub const fn new(timestamp: SystemTime) -> Self {
        Self { timestamp }
    }
}

impl RecoveryStrategy for TimeBasedStrategy {
    fn select_state<'a>(&self, snapshots: &'a [ContextSnapshot]) -> Option<&'a ContextSnapshot> {
        snapshots
            .iter()
            .filter(move |s| s.timestamp <= self.timestamp)
            .max_by_key(move |s| s.timestamp)
    }
}

/// Manages context snapshots and recovery operations
pub struct RecoveryManager {
    persistence: Arc<Mutex<ContextPersistence>>,
    snapshots: VecDeque<ContextSnapshot>,
    max_snapshots: usize,
}

impl RecoveryManager {
    /// Creates a new recovery manager
    ///
    /// # Arguments
    /// * `persistence` - The persistence layer for storing snapshots
    /// * `max_snapshots` - Maximum number of snapshots to keep in memory
    pub const fn new(
        persistence: Arc<Mutex<ContextPersistence>>,
        max_snapshots: usize,
    ) -> Self {
        Self {
            persistence,
            snapshots: VecDeque::new(),
            max_snapshots,
        }
    }

    /// Creates a new snapshot of the current context state
    ///
    /// # Arguments
    /// * `state` - The context state to snapshot
    ///
    /// # Returns
    /// The created snapshot if successful
    ///
    /// # Errors
    /// Returns a `ContextError` if the snapshot could not be created or saved
    pub fn create_snapshot(&mut self, state: &ContextState) -> Result<ContextSnapshot, ContextError> {
        let snapshot = ContextSnapshot {
            id: Uuid::new_v4().to_string(),
            timestamp: SystemTime::now(),
            state: state.clone(),
            metadata: None,
        };

        if let Ok(mut persistence) = self.persistence.lock() {
            persistence.save_snapshot(&snapshot)?;
        }

        self.snapshots.push_back(snapshot.clone());
        if self.snapshots.len() > self.max_snapshots {
            if let Some(old_snapshot) = self.snapshots.pop_front() {
                if let Ok(mut persistence) = self.persistence.lock() {
                    let _ = persistence.delete_snapshot(&old_snapshot.id);
                }
            }
        }

        Ok(snapshot)
    }

    /// Returns all available snapshots
    ///
    /// # Returns
    /// A reference to the collection of snapshots
    #[must_use] pub const fn get_snapshots(&self) -> &VecDeque<ContextSnapshot> {
        &self.snapshots
    }

    /// Restores a snapshot with the specified ID
    ///
    /// # Arguments
    /// * `id` - The ID of the snapshot to restore
    ///
    /// # Returns
    /// The restored context state if successful
    ///
    /// # Errors
    /// Returns a `ContextError::SnapshotNotFound` if no snapshot with the given ID exists
    pub fn restore_snapshot(&self, id: &str) -> Result<ContextState, ContextError> {
        if let Some(snapshot) = self.snapshots.iter().find(|s| s.id == id) {
            Ok(snapshot.state.clone())
        } else {
            Err(ContextError::SnapshotNotFound(format!("Snapshot with id '{id}' not found")))
        }
    }

    /// Deletes a snapshot with the specified ID
    ///
    /// # Errors
    ///
    /// Returns a `ContextError::SnapshotNotFound` if no snapshot with the given ID exists,
    /// or propagates a `ContextError::PersistenceError` if the deletion from persistent storage fails
    pub fn delete_snapshot(&mut self, id: &str) -> Result<(), ContextError> {
        if let Some(index) = self.snapshots.iter().position(|s| s.id == id) {
            self.snapshots.remove(index);
            if let Ok(mut persistence) = self.persistence.lock() {
                persistence.delete_snapshot(id)?;
            }
            Ok(())
        } else {
            Err(ContextError::SnapshotNotFound(format!("Snapshot with id '{id}' not found")))
        }
    }

    /// Recovers a context state using the specified recovery strategy
    ///
    /// # Errors
    ///
    /// Returns a `ContextError::NoValidSnapshot` if the strategy cannot find a valid snapshot to recover
    pub async fn recover_using_strategy<S: RecoveryStrategy + Send + Sync>(&self, strategy: &S) -> Option<ContextSnapshot> {
        if self.snapshots.is_empty() {
            return None;
        }
        
        // Create a temporary Vec of owned ContextSnapshot objects
        let snapshots: Vec<ContextSnapshot> = self.snapshots.iter().cloned().collect();
        
        // Now pass a slice of them to the strategy
        strategy.select_state(&snapshots).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tempfile::tempdir;
    use crate::context::persistence::{ContextPersistence, FileStorage, JsonSerializer};

    #[test]
    fn test_recovery_manager() {
        let temp_dir = tempdir().unwrap();
        let storage = Box::new(FileStorage::new(temp_dir.path().to_path_buf()).unwrap());
        let serializer = Box::new(JsonSerializer::new());
        let persistence = Arc::new(Mutex::new(ContextPersistence::new(
            storage,
            serializer,
            10,
            Duration::from_secs(60),
        )));

        let mut recovery = RecoveryManager::new(persistence.clone(), 10);

        // Create test state
        let state = ContextState {
            version: 1,
            data: serde_json::json!({"test": "value"}),
            last_modified: SystemTime::now(),
        };

        // Test snapshot creation
        let snapshot = recovery.create_snapshot(&state).unwrap();
        assert!(!snapshot.id.is_empty());

        // Test snapshot listing
        let snapshots = recovery.get_snapshots();
        assert_eq!(snapshots.len(), 1);
        assert_eq!(snapshots[0].state.version, state.version);

        // Test snapshot restoration
        let restored_state = recovery.restore_snapshot(&snapshot.id).unwrap();
        assert_eq!(restored_state.version, state.version);
        assert_eq!(restored_state.data, state.data);

        // Test snapshot deletion
        assert!(recovery.delete_snapshot(&snapshot.id).is_ok());
        let snapshots = recovery.get_snapshots();
        assert!(snapshots.is_empty());
    }

    #[tokio::test]
    async fn test_recovery_strategy() {
        let temp_dir = tempdir().unwrap();
        let storage = Box::new(FileStorage::new(temp_dir.path().to_path_buf()).unwrap());
        let serializer = Box::new(JsonSerializer::new());
        let persistence = Arc::new(Mutex::new(ContextPersistence::new(
            storage,
            serializer,
            10,
            Duration::from_secs(60),
        )));

        let mut recovery = RecoveryManager::new(persistence.clone(), 10);

        // Create test states with different versions
        for i in 1..=3 {
            let state = ContextState {
                version: i,
                data: serde_json::json!({"test": i}),
                last_modified: SystemTime::now(),
            };
            recovery.create_snapshot(&state).unwrap();
        }

        // Test LatestVersionStrategy
        let strategy = LatestVersionStrategy::new();
        let recovered = recovery.recover_using_strategy(&strategy).await.unwrap();
        assert_eq!(recovered.state.version, 3);

        // Test SpecificVersionStrategy
        let strategy = SpecificVersionStrategy::new(2);
        let recovered = recovery.recover_using_strategy(&strategy).await.unwrap();
        assert_eq!(recovered.state.version, 2);

        // Test TimeBasedStrategy
        let timestamp = SystemTime::now();
        let strategy = TimeBasedStrategy::new(timestamp);
        let recovered = recovery.recover_using_strategy(&strategy).await.unwrap();
        assert!(recovered.state.version > 0);
    }

    #[tokio::test]
    async fn test_recovery_error_handling() {
        let temp_dir = tempdir().unwrap();
        let storage = Box::new(FileStorage::new(temp_dir.path().to_path_buf()).unwrap());
        let serializer = Box::new(JsonSerializer::new());
        let persistence = Arc::new(Mutex::new(ContextPersistence::new(
            storage,
            serializer,
            10,
            Duration::from_secs(60),
        )));

        let recovery = RecoveryManager::new(persistence.clone(), 10);

        // Testing when no snapshots are available
        let strategy = LatestVersionStrategy::new();
        let result = recovery.recover_using_strategy(&strategy).await;
        assert!(result.is_none());

        // Test with strategy that won't find anything
        let strategy = SpecificVersionStrategy::new(999);
        match recovery.recover_using_strategy(&strategy).await {
            None => assert!(true),
            Some(_) => panic!("Expected None but got Some"),
        }
    }
} 