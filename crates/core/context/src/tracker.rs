// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Context tracking functionality
//!
//! This module provides functionality for tracking context changes.
//!
//! ## Concurrency and Locking
//!
//! The context tracker uses tokio's asynchronous locks (`Mutex`, `RwLock`) to ensure
//! thread safety while maintaining good performance in an async environment.
//! Key locking practices implemented in this module:
//!
//! - Using scope-based locking to minimize lock duration
//! - Avoiding holding locks across `.await` points
//! - Using read locks for operations that don't modify data
//! - Using write locks for operations that modify data
//! - Dropping locks explicitly before async operations
//!
//! When working with the context tracker in asynchronous code, it's important to
//! follow these same patterns to avoid potential deadlocks or performance issues.

use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::{Mutex, RwLock};
use tokio::time::{Duration, Instant};
use uuid::Uuid;

use crate::{manager::ContextManager, ContextError, ContextState, Result};

/// Configuration for context tracker
#[derive(Debug, Clone)]
pub struct ContextTrackerConfig {
    /// Automatic state synchronization interval in seconds (0 = disabled)
    pub sync_interval_seconds: u64,
    /// Whether to create recovery points automatically
    pub auto_recovery: bool,
    /// Maximum number of recovery points to maintain
    pub max_recovery_points: usize,
}

impl Default for ContextTrackerConfig {
    fn default() -> Self {
        Self {
            sync_interval_seconds: 60,
            auto_recovery: true,
            max_recovery_points: 10,
        }
    }
}

/// Context tracker for managing state changes
#[derive(Debug)]
pub struct ContextTracker {
    /// Current state of the context
    state: Arc<Mutex<ContextState>>,
    /// Configuration
    config: ContextTrackerConfig,
    /// Manager reference if available
    manager: Option<Arc<ContextManager>>,
    /// Active context ID
    active_context_id: Arc<RwLock<Option<String>>>,
    /// Last sync time
    last_sync: Arc<RwLock<Instant>>,
}

impl ContextTracker {
    /// Create a new context tracker with the given state
    #[must_use]
    pub fn new(state: ContextState) -> Self {
        Self {
            state: Arc::new(Mutex::new(state)),
            config: ContextTrackerConfig::default(),
            manager: None,
            active_context_id: Arc::new(RwLock::new(None)),
            last_sync: Arc::new(RwLock::new(Instant::now())),
        }
    }

    /// Create a new context tracker with configuration and manager
    #[must_use]
    pub fn with_config_and_manager(
        state: ContextState,
        config: ContextTrackerConfig,
        manager: Option<Arc<ContextManager>>,
    ) -> Self {
        Self {
            state: Arc::new(Mutex::new(state)),
            config,
            manager,
            active_context_id: Arc::new(RwLock::new(None)),
            last_sync: Arc::new(RwLock::new(Instant::now())),
        }
    }

    /// Get the current state
    pub async fn get_state(&self) -> Result<ContextState> {
        let state = self.state.lock().await;
        Ok(state.clone())
    }

    /// Update the current state
    ///
    /// This method updates the current context state.
    /// It follows best practices for async lock management by:
    /// 1. Acquiring a lock only to read the current state
    /// 2. Dropping the lock before conditional logic
    /// 3. Acquiring a separate lock for the update operation
    /// 4. Using separate lock scopes to minimize lock duration
    /// 5. Not holding any locks during manager operations
    ///
    /// # Errors
    ///
    /// Returns errors when:
    /// - Failed to acquire lock
    /// - Failed to create recovery point (if auto_recovery is enabled)
    pub async fn update_state(&self, state: ContextState) -> Result<()> {
        // First check if we need to update
        let should_update = {
            let current_state = self.state.lock().await;
            state.version > current_state.version
        }; // Lock is dropped here

        if should_update {
            // Update the state
            {
                let mut current_state = self.state.lock().await;
                *current_state = state.clone();
            } // Lock is dropped here

            // Update the last sync time
            {
                let mut last_sync = self.last_sync.write().await;
                *last_sync = Instant::now();
            } // Lock is dropped here

            // Trigger automatic recovery point if enabled
            if self.config.auto_recovery {
                if let Some(manager) = &self.manager {
                    // Get state for recovery point
                    let state = self.get_state().await?;

                    // Create recovery point
                    manager.create_recovery_point(&state).await?;
                }
            }

            Ok(())
        } else {
            // No update needed, same version or older
            Ok(())
        }
    }

    /// Activate a context by ID
    pub async fn activate_context(&self, id: &str) -> Result<()> {
        if let Some(manager) = &self.manager {
            // Load the context from the manager
            let state = manager.get_context_state(id).await?;

            // Set the active context ID
            {
                let mut active_id = self.active_context_id.write().await;
                *active_id = Some(id.to_string());
            }

            // Update the state
            self.update_state(state).await?;

            Ok(())
        } else {
            Err(ContextError::NotInitialized)
        }
    }

    /// Deactivate the current context
    pub async fn deactivate_context(&self) -> Result<()> {
        let mut active_id = self.active_context_id.write().await;
        // Clear the active context ID
        *active_id = None;
        Ok(())
    }

    /// Get the active context ID
    pub async fn get_active_context_id(&self) -> Result<Option<String>> {
        let active_id = self.active_context_id.read().await;
        Ok(active_id.clone())
    }

    /// Synchronize state with persistence
    ///
    /// This method synchronizes the current state with the manager.
    /// It follows best practices for async lock management by:
    /// 1. Getting references outside of locks
    /// 2. Getting state without holding other locks
    /// 3. Reading active context ID without holding state lock
    /// 4. Using separate lock scopes to minimize lock duration
    /// 5. Not holding any locks during manager operations
    ///
    /// # Errors
    ///
    /// Returns errors when:
    /// - No active context
    /// - Context manager not set
    /// - Failed to acquire lock
    /// - Failed to update context state
    pub async fn sync_state(&self) -> Result<()> {
        if let Some(manager) = &self.manager {
            // Get the current state
            let state = self.get_state().await?;

            // Get active context ID without holding lock across await
            let active_id_option = self.active_context_id.read().await.clone();

            if let Some(id) = active_id_option {
                // Update the context state in the manager
                manager.update_context_state(&id, state).await?;

                // Update the last sync time
                {
                    let mut last_sync = self.last_sync.write().await;
                    *last_sync = Instant::now();
                } // Lock is dropped here

                return Ok(());
            }

            // If no active context, return an error
            Err(ContextError::NotInitialized)
        } else {
            Err(ContextError::NotInitialized)
        }
    }

    /// Start automatic synchronization for this tracker
    pub async fn synchronize(self: &Arc<Self>) -> Result<()> {
        // Implementation left for actual code
        Ok(())
    }

    /// Create a recovery point for the current state
    pub async fn create_recovery_point(&self) -> Result<()> {
        if let Some(manager) = &self.manager {
            // Get the current state
            let state = self.get_state().await?;

            // Create a recovery point
            manager.create_recovery_point(&state).await?;

            Ok(())
        } else {
            Err(ContextError::NotInitialized)
        }
    }

    /// Check if synchronization is needed
    pub async fn is_sync_needed(&self) -> bool {
        if self.config.sync_interval_seconds == 0 {
            // Auto-sync disabled
            return false;
        }

        // Check time since last sync
        let interval = Duration::from_secs(self.config.sync_interval_seconds);
        let last_sync_time = *self.last_sync.read().await;
        let elapsed = last_sync_time.elapsed();

        elapsed > interval
    }
}

/// Factory for creating context trackers
#[derive(Debug)]
pub struct ContextTrackerFactory {
    /// Optional manager reference
    manager: Option<Arc<ContextManager>>,
    /// Optional configuration
    config: Option<ContextTrackerConfig>,
    /// Default state for new trackers
    default_state: Option<ContextState>,
}

impl ContextTrackerFactory {
    /// Create a new factory with the given manager
    pub fn new(manager: Option<Arc<ContextManager>>) -> Self {
        Self {
            manager,
            config: None,
            default_state: None,
        }
    }

    /// Create a new factory with the given manager and config
    pub fn with_config(manager: Option<Arc<ContextManager>>, config: ContextTrackerConfig) -> Self {
        Self {
            manager,
            config: Some(config),
            default_state: None,
        }
    }

    /// Set the default state for new trackers
    pub fn set_default_state(&mut self, state: ContextState) {
        self.default_state = Some(state);
    }

    /// Create a new context tracker
    pub fn create(&self) -> Result<ContextTracker> {
        // Create a default state if none provided
        let state = if let Some(state) = &self.default_state {
            state.clone()
        } else {
            // Create a new empty state
            ContextState {
                id: Uuid::new_v4().to_string(),
                version: 1,
                timestamp: Utc::now().timestamp() as u64,
                data: serde_json::json!({}),
                metadata: HashMap::new(),
                synchronized: false,
                last_modified: SystemTime::now(),
            }
        };

        // Create with config if provided
        if let Some(config) = &self.config {
            Ok(ContextTracker::with_config_and_manager(
                state,
                config.clone(),
                self.manager.clone(),
            ))
        } else {
            // Create with defaults
            let tracker = ContextTracker::new(state);

            // Set manager if provided
            if self.manager.is_some() {
                // Create a new tracker with our configuration and manager
                let config = tracker.config.clone();
                let manager_clone = self.manager.clone();

                // Create a new state
                let empty_state = ContextState {
                    id: Uuid::new_v4().to_string(),
                    version: 1,
                    timestamp: Utc::now().timestamp() as u64,
                    data: serde_json::json!({}),
                    metadata: HashMap::new(),
                    synchronized: false,
                    last_modified: SystemTime::now(),
                };

                Ok(ContextTracker::with_config_and_manager(
                    empty_state,
                    config,
                    manager_clone,
                ))
            } else {
                Ok(tracker)
            }
        }
    }

    /// Create a new tracker as an Arc
    pub fn create_tracker(&self, state: ContextState) -> Arc<ContextTracker> {
        let config = self.config.clone().unwrap_or_default();

        Arc::new(ContextTracker::with_config_and_manager(
            state,
            config,
            self.manager.clone(),
        ))
    }

    /// Create a new tracker with the given config
    pub fn create_with_config(&self, config: ContextTrackerConfig) -> Result<ContextTracker> {
        // Create a default state if none provided
        let state = if let Some(state) = &self.default_state {
            state.clone()
        } else {
            // Create a new empty state
            ContextState {
                id: Uuid::new_v4().to_string(),
                version: 1,
                timestamp: Utc::now().timestamp() as u64,
                data: serde_json::json!({}),
                metadata: HashMap::new(),
                synchronized: false,
                last_modified: SystemTime::now(),
            }
        };

        Ok(ContextTracker::with_config_and_manager(
            state,
            config,
            self.manager.clone(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_state(version: u64) -> ContextState {
        ContextState {
            id: Uuid::new_v4().to_string(),
            version,
            timestamp: Utc::now().timestamp() as u64,
            data: json!({"test": true}),
            metadata: HashMap::new(),
            synchronized: false,
            last_modified: SystemTime::now(),
        }
    }

    // ContextTrackerConfig tests
    #[test]
    fn test_tracker_config_default() {
        let config = ContextTrackerConfig::default();
        assert_eq!(config.sync_interval_seconds, 60);
        assert!(config.auto_recovery);
        assert_eq!(config.max_recovery_points, 10);
    }

    // ContextTracker tests
    #[tokio::test]
    async fn test_tracker_new() {
        let state = make_state(1);
        let tracker = ContextTracker::new(state.clone());
        let got = tracker.get_state().await.unwrap();
        assert_eq!(got.version, 1);
    }

    #[tokio::test]
    async fn test_tracker_with_config_and_manager() {
        let state = make_state(1);
        let config = ContextTrackerConfig {
            sync_interval_seconds: 30,
            auto_recovery: false,
            max_recovery_points: 5,
        };
        let tracker = ContextTracker::with_config_and_manager(state, config, None);
        let got = tracker.get_state().await.unwrap();
        assert_eq!(got.version, 1);
    }

    #[tokio::test]
    async fn test_tracker_update_state_newer_version() {
        let state1 = make_state(1);
        let tracker = ContextTracker::new(state1);

        let state2 = make_state(2);
        let result = tracker.update_state(state2).await;
        assert!(result.is_ok());

        let got = tracker.get_state().await.unwrap();
        assert_eq!(got.version, 2);
    }

    #[tokio::test]
    async fn test_tracker_update_state_older_version_ignored() {
        let state2 = make_state(2);
        let tracker = ContextTracker::new(state2);

        let state1 = make_state(1);
        let result = tracker.update_state(state1).await;
        assert!(result.is_ok());

        // State should still be version 2
        let got = tracker.get_state().await.unwrap();
        assert_eq!(got.version, 2);
    }

    #[tokio::test]
    async fn test_tracker_active_context_id() {
        let state = make_state(1);
        let tracker = ContextTracker::new(state);

        // Initially no active context
        let active = tracker.get_active_context_id().await.unwrap();
        assert!(active.is_none());
    }

    #[tokio::test]
    async fn test_tracker_deactivate_context() {
        let state = make_state(1);
        let tracker = ContextTracker::new(state);
        let result = tracker.deactivate_context().await;
        assert!(result.is_ok());

        let active = tracker.get_active_context_id().await.unwrap();
        assert!(active.is_none());
    }

    #[tokio::test]
    async fn test_tracker_activate_no_manager_fails() {
        let state = make_state(1);
        let tracker = ContextTracker::new(state);
        let result = tracker.activate_context("some-id").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_tracker_sync_no_manager_fails() {
        let state = make_state(1);
        let tracker = ContextTracker::new(state);
        let result = tracker.sync_state().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_tracker_create_recovery_point_no_manager_fails() {
        let state = make_state(1);
        let tracker = ContextTracker::new(state);
        let result = tracker.create_recovery_point().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_tracker_is_sync_needed_disabled() {
        let state = make_state(1);
        let config = ContextTrackerConfig {
            sync_interval_seconds: 0, // disabled
            auto_recovery: false,
            max_recovery_points: 5,
        };
        let tracker = ContextTracker::with_config_and_manager(state, config, None);
        assert!(!tracker.is_sync_needed().await);
    }

    #[tokio::test]
    async fn test_tracker_is_sync_needed_not_due() {
        let state = make_state(1);
        let config = ContextTrackerConfig {
            sync_interval_seconds: 3600, // 1 hour
            auto_recovery: false,
            max_recovery_points: 5,
        };
        let tracker = ContextTracker::with_config_and_manager(state, config, None);
        assert!(!tracker.is_sync_needed().await);
    }

    // ContextTrackerFactory tests
    #[test]
    fn test_factory_new() {
        let factory = ContextTrackerFactory::new(None);
        let tracker = factory.create();
        assert!(tracker.is_ok());
    }

    #[test]
    fn test_factory_with_config() {
        let config = ContextTrackerConfig {
            sync_interval_seconds: 120,
            auto_recovery: true,
            max_recovery_points: 20,
        };
        let factory = ContextTrackerFactory::with_config(None, config);
        let tracker = factory.create();
        assert!(tracker.is_ok());
    }

    #[test]
    fn test_factory_set_default_state() {
        let mut factory = ContextTrackerFactory::new(None);
        let state = make_state(5);
        factory.set_default_state(state);
        let tracker = factory.create().unwrap();

        // The tracker should use the default state
        let rt = tokio::runtime::Runtime::new().unwrap();
        let got = rt.block_on(tracker.get_state()).unwrap();
        assert_eq!(got.version, 5);
    }

    #[test]
    fn test_factory_create_tracker_arc() {
        let factory = ContextTrackerFactory::new(None);
        let state = make_state(3);
        let tracker = factory.create_tracker(state);
        let rt = tokio::runtime::Runtime::new().unwrap();
        let got = rt.block_on(tracker.get_state()).unwrap();
        assert_eq!(got.version, 3);
    }

    #[test]
    fn test_factory_create_with_config() {
        let factory = ContextTrackerFactory::new(None);
        let config = ContextTrackerConfig {
            sync_interval_seconds: 10,
            auto_recovery: false,
            max_recovery_points: 3,
        };
        let tracker = factory.create_with_config(config);
        assert!(tracker.is_ok());
    }
}
