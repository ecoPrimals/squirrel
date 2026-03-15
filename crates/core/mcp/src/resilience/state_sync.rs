// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

/// State Synchronization Module
///
/// This module provides mechanisms for synchronizing state between primary
/// and backup/redundant systems in a distributed system. It supports
/// synchronizing different types of state data with configurable validation.

use std::fmt;
use std::time::{Duration, Instant};
use std::error::Error as StdError;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use serde::Serialize;
use std::sync::RwLock;
use serde_json;
use serde::de::DeserializeOwned;

/// Represents the type of state being synchronized
///
/// Different types of state may have different synchronization requirements,
/// validation rules, and priorities. This enum allows the synchronization
/// mechanism to handle each type appropriately.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StateType {
    /// Configuration state (system settings, parameters)
    ///
    /// This state type represents configuration settings that control system behavior.
    /// Examples include timeouts, feature flags, and other parameters.
    Configuration,
    
    /// Runtime state (current system state during operation)
    ///
    /// This state type represents the current operational state of the system.
    /// Examples include active connections, current workloads, and temporary data.
    Runtime,
    
    /// Recovery state (data needed for recovery procedures)
    ///
    /// This state type represents information needed to recover from failures.
    /// Examples include checkpoints, transaction logs, and backup references.
    Recovery,
    
    /// Audit state (logs, metrics, history data)
    ///
    /// This state type represents historical data for auditing and analysis.
    /// Examples include operation logs, performance metrics, and event history.
    Audit,
}

impl fmt::Display for StateType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Configuration => write!(f, "Configuration"),
            Self::Runtime => write!(f, "Runtime"),
            Self::Recovery => write!(f, "Recovery"),
            Self::Audit => write!(f, "Audit"),
        }
    }
}

/// Configuration for state synchronization
#[derive(Debug, Clone)]
pub struct StateSyncConfig {
    /// Timeout for synchronization operations
    pub sync_timeout: Duration,
    
    /// Maximum size of state data to synchronize (in bytes)
    pub max_state_size: usize,
    
    /// Whether to validate state before applying
    pub validate_state: bool,
}

impl Default for StateSyncConfig {
    fn default() -> Self {
        Self {
            sync_timeout: Duration::from_secs(10),
            max_state_size: 1024 * 1024, // 1MB max by default
            validate_state: true,
        }
    }
}

/// Error type for state synchronization operations
#[derive(Debug)]
pub enum StateSyncError {
    /// Synchronization timed out
    Timeout {
        /// Duration after which the operation timed out
        duration: Duration,
    },
    
    /// State data exceeds maximum allowed size
    SizeExceeded {
        /// Actual size of the state data in bytes
        size: usize,
        /// Maximum allowed size in bytes
        max_size: usize,
    },
    
    /// State validation failed
    ValidationFailed {
        /// Error message explaining the validation failure
        message: String,
    },
    
    /// Target system not found or unavailable
    TargetUnavailable {
        /// Identifier of the target system
        target: String,
    },
    
    /// Serialization or deserialization error
    SerializationError {
        /// Error message explaining the serialization failure
        message: String,
    },
    
    /// General error during synchronization
    SyncFailed {
        /// Error message explaining the synchronization failure
        message: String,
        /// Underlying source of the error, if available
        source: Option<Box<dyn StdError + Send + Sync + 'static>>,
    },

    /// State not found
    ///
    /// This error occurs when attempting to retrieve state that doesn't exist.
    NotFound(
        /// Identifier or description of the state that was not found
        String
    ),

    /// Deserialization failed
    ///
    /// This error occurs when the state data couldn't be properly deserialized.
    DeserializationFailed(
        /// Error message explaining the deserialization failure
        String
    ),
}

impl fmt::Display for StateSyncError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Timeout { duration } => {
                write!(f, "Synchronization timed out after {duration:?}")
            }
            Self::SizeExceeded { size, max_size } => {
                write!(f, "State size ({size} bytes) exceeds maximum allowed size ({max_size} bytes)")
            }
            Self::ValidationFailed { message } => {
                write!(f, "State validation failed: {message}")
            }
            Self::TargetUnavailable { target } => {
                write!(f, "Target system unavailable: {target}")
            }
            Self::SerializationError { message } => {
                write!(f, "Serialization error: {message}")
            }
            Self::SyncFailed { message, .. } => {
                write!(f, "Synchronization failed: {message}")
            }
            Self::NotFound(msg) => {
                write!(f, "State not found: {msg}")
            }
            Self::DeserializationFailed(msg) => {
                write!(f, "Deserialization failed: {msg}")
            }
        }
    }
}

impl StdError for StateSyncError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::SyncFailed { source, .. } => {
                source.as_ref().map(|s| s.as_ref() as &(dyn StdError + 'static))
            }
            _ => None,
        }
    }
}

/// Metrics for state synchronization operations
#[derive(Debug, Default, Clone)]
pub struct StateSyncMetrics {
    /// Count of successful synchronizations
    pub successful_syncs: HashMap<StateType, u32>,
    
    /// Count of failed synchronizations
    pub failed_syncs: HashMap<StateType, u32>,
    
    /// Total bytes synchronized
    pub total_bytes_synced: usize,
    
    /// Last synchronization time
    pub last_sync_time: Option<Instant>,

    /// Count of retrieve requests
    pub retrieve_requests: u32,

    /// Count of successful retrievals
    pub retrieve_success: u32,

    /// Count of failed retrievals
    pub retrieve_failures: u32,
}

impl StateSyncMetrics {
    /// Reset all metrics to default values
    pub fn reset(&mut self) {
        self.successful_syncs.clear();
        self.failed_syncs.clear();
        self.total_bytes_synced = 0;
        self.last_sync_time = None;
        self.retrieve_requests = 0;
        self.retrieve_success = 0;
        self.retrieve_failures = 0;
    }
}

/// A state synchronizer for maintaining consistent state across distributed systems
#[derive(Debug, Clone)]
pub struct StateSynchronizer {
    /// Configuration for the synchronizer
    config: StateSyncConfig,
    
    /// Metrics about synchronization operations
    metrics: Arc<Mutex<StateSyncMetrics>>,

    /// Storage for states (added this field since it was missing)
    states: Arc<RwLock<HashMap<StateType, StateData>>>,
}

impl StateSynchronizer {
    /// Create a new state synchronizer with the given configuration
    #[must_use] pub fn new(config: StateSyncConfig) -> Self {
        Self {
            config,
            metrics: Arc::new(Mutex::new(StateSyncMetrics::default())),
            states: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Create a new state synchronizer with default configuration
    #[must_use] pub fn default() -> Self {
        Self::new(StateSyncConfig::default())
    }
    
    /// Synchronize state data to a target system
    ///
    /// Serializes and transmits state data to a target system for synchronization.
    /// This is used to ensure consistency across distributed components.
    ///
    /// # Arguments
    ///
    /// * `state_type` - The type of state being synchronized
    /// * `_state_id` - The identifier for the specific state instance
    /// * `_target` - The target system to synchronize with
    /// * `state` - The state data to synchronize
    ///
    /// # Returns
    ///
    /// Success if the state was successfully synchronized
    ///
    /// # Errors
    ///
    /// Returns a `StateSyncError` if:
    /// * The state size exceeds the configured maximum size (`StateSyncError::SizeExceeded`)
    /// * The serialization of the state fails (`StateSyncError::SerializationError`)
    /// * The synchronization operation times out (`StateSyncError::Timeout`)
    /// * The metrics tracking fails to update (`StateSyncError::SyncFailed`)
    /// * Any other synchronization failure occurs
    pub async fn sync_state<T>(
        &self,
        state_type: StateType,
        _state_id: &str,
        _target: &str,
        state: T
    ) -> Result<(), StateSyncError>
    where
        T: Serialize,
    {
        let start_time = Instant::now();
        self.metrics.lock().map_err(|e| StateSyncError::SyncFailed {
            message: format!("Failed to lock metrics: {e}"),
            source: None,
        })?.last_sync_time = Some(start_time);
        
        // Serialize the state to estimate its size
        let serialized = serde_json::to_vec(&state)
            .map_err(|e| StateSyncError::SerializationError {
                message: e.to_string(),
            })?;
        
        // Check if state exceeds maximum size
        if serialized.len() > self.config.max_state_size {
            // Update metrics for failure - early drop optimization
            {
                let mut metrics = self.metrics.lock().map_err(|e| StateSyncError::SyncFailed {
                    message: format!("Failed to lock metrics: {e}"),
                    source: None,
                })?;
                *metrics.failed_syncs.entry(state_type).or_insert(0) += 1;
            } // metrics is dropped here
            
            return Err(StateSyncError::SizeExceeded {
                size: serialized.len(),
                max_size: self.config.max_state_size,
            });
        }
        
        // Validate state if configured to do so
        if self.config.validate_state {
            // In a real implementation, this would validate against schema, constraints, etc.
            // For now, we'll just assume it's valid
        }
        
        // Add a small delay to simulate network I/O
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        
        // Check timeout
        if start_time.elapsed() > self.config.sync_timeout {
            // Update metrics for failure - early drop optimization
            {
                let mut metrics = self.metrics.lock().map_err(|e| StateSyncError::SyncFailed {
                    message: format!("Failed to lock metrics: {e}"),
                    source: None,
                })?;
                *metrics.failed_syncs.entry(state_type).or_insert(0) += 1;
            } // metrics is dropped here
            
            return Err(StateSyncError::Timeout {
                duration: self.config.sync_timeout,
            });
        }
        
        // In a real implementation, this would communicate with the target system
        // For now, we'll just simulate success
        
        // Store the state in our local cache
        {
            let state_data = StateData {
                data: serde_json::from_slice(&serialized).map_err(|e| StateSyncError::SerializationError {
                    message: format!("Failed to deserialize serialized data: {e}"),
                })?,
                timestamp: Instant::now(),
            };
            
            let mut states = self.states.write().map_err(|e| StateSyncError::SyncFailed {
                message: format!("Failed to write states: {e}"),
                source: None,
            })?;
            
            states.insert(state_type, state_data);
        }
        
        // Update metrics for success and drop early
        {
            let mut metrics = self.metrics.lock().map_err(|e| StateSyncError::SyncFailed {
                message: format!("Failed to lock metrics: {e}"),
                source: None,
            })?;
            *metrics.successful_syncs.entry(state_type).or_insert(0) += 1;
            metrics.total_bytes_synced += serialized.len();
        } // metrics is dropped here
        
        Ok(())
    }
    
    /// Sync state with a timeout
    ///
    /// # Arguments
    /// * `state_type` - The type of state to sync
    /// * `state_id` - The ID of the state to sync
    /// * `target` - The target to sync with
    /// * `state` - The state to sync
    /// * `timeout` - The timeout for the sync operation
    ///
    /// # Errors
    /// * Returns `StateSyncError::Timeout` if the operation times out
    /// * Returns `StateSyncError::SyncFailed` if the sync operation fails
    /// * Returns `StateSyncError::InvalidStateType` if the state type is invalid
    pub async fn sync_state_with_timeout<T>(
        &self,
        state_type: StateType,
        _state_id: &str,
        _target: &str,
        state: T,
        timeout: Duration,
    ) -> Result<(), StateSyncError>
    where
        T: Serialize + Clone + Send + Sync + 'static
    {
        let _start_time = Instant::now();
        
        // Use tokio's timeout mechanism with map_or_else
        tokio::time::timeout(
            timeout,
            self.sync_state(state_type, _state_id, _target, state)
        ).await.map_or_else(
            |_elapsed_error| Err(StateSyncError::Timeout { duration: timeout }), // Err(Elapsed) case
            |inner_result| inner_result                                         // Ok(inner_result) case
        )
    }
    
    /// Get metrics for state sync operations
    /// 
    /// # Returns
    /// * `StateSyncMetrics` containing metrics about sync operations
    ///
    /// # Errors
    /// * Returns `StateSyncError::SyncFailed` if metrics cannot be accessed
    pub fn get_metrics(&self) -> Result<StateSyncMetrics, StateSyncError> {
        self.metrics.lock().map(|m| m.clone()).map_err(|e| StateSyncError::SyncFailed {
            message: format!("Failed to lock metrics: {e}"),
            source: None,
        })
    }
    
    /// Reset all metrics counters to zero
    ///
    /// # Errors
    /// * Returns `StateSyncError::SyncFailed` if metrics cannot be accessed
    pub fn reset_metrics(&self) -> Result<(), StateSyncError> {
        self.metrics.lock().map(|mut m| {
            m.reset();
        }).map_err(|e| StateSyncError::SyncFailed {
            message: format!("Failed to lock metrics: {e}"),
            source: None,
        })
    }
    
    /// Get the current configuration
    #[must_use] pub const fn get_config(&self) -> &StateSyncConfig {
        &self.config
    }
    
    /// Update the configuration
    pub fn update_config(&mut self, config: StateSyncConfig) {
        self.config = config;
    }

    /// Retrieve state from storage
    ///
    /// # Arguments
    /// * `state_type` - The type of state to retrieve
    /// * `_state_id` - The ID of the state to retrieve
    /// * `_query` - Optional query parameters
    ///
    /// # Errors
    /// * Returns `StateSyncError::NotFound` if the state is not found
    /// * Returns `StateSyncError::SyncFailed` if the retrieval fails
    /// * Returns `StateSyncError::InvalidStateType` if the state type is invalid
    pub async fn retrieve_state<T>(
        &self,
        state_type: StateType,
        _state_id: &str,
        _query: Option<&str>,
    ) -> Result<T, StateSyncError>
    where
        T: DeserializeOwned + Send + 'static,
    {
        // Update metrics
        let _start_time = Instant::now();
        
        // Eventually this will use state_id and target to retrieve specific states
        // For now, we'll just use the latest state by type
        
        // Track metrics - increment request counter
        {
            let mut metrics = self.metrics.lock().map_err(|e| StateSyncError::SyncFailed {
                message: format!("Failed to lock metrics: {e}"),
                source: None,
            })?;
            metrics.retrieve_requests += 1;
        } // metrics is dropped here
        
        // Get the cached state
        let state = {
            let states = self.states.read().map_err(|e| StateSyncError::SyncFailed {
                message: format!("Failed to read states: {e}"),
                source: None,
            })?;
            if let Some(state) = states.get(&state_type) { state.data.clone() } else {
                // Update metrics for failure
                {
                    let mut metrics = self.metrics.lock().map_err(|e| StateSyncError::SyncFailed {
                        message: format!("Failed to lock metrics: {e}"),
                        source: None,
                    })?;
                    metrics.retrieve_failures += 1;
                } // metrics is dropped here
                return Err(StateSyncError::NotFound("State not found".to_string()));
            }
        };
        
        // Deserialize the state
        match serde_json::from_value(state) {
            Ok(value) => {
                // Update success metric
                {
                    let mut metrics = self.metrics.lock().map_err(|e| StateSyncError::SyncFailed {
                        message: format!("Failed to lock metrics: {e}"),
                        source: None,
                    })?;
                    metrics.retrieve_success += 1;
                } // metrics is dropped here
                Ok(value)
            }
            Err(e) => {
                // Update failure metric
                {
                    let mut metrics = self.metrics.lock().map_err(|e| StateSyncError::SyncFailed {
                        message: format!("Failed to lock metrics: {e}"),
                        source: None,
                    })?;
                    metrics.retrieve_failures += 1;
                } // metrics is dropped here
                Err(StateSyncError::DeserializationFailed(e.to_string()))
            }
        }
    }

    /// Process a state update received from another node
    ///
    /// # Arguments
    /// * `_state_type` - The type of state being updated
    /// * `_state_data` - The new state data
    ///
    /// # Errors
    /// * Returns `StateSyncError::SyncFailed` if the update cannot be processed
    pub async fn process_state_update<T>(
        &self,
        _state_type: StateType,
        _state_data: T
    ) -> Result<(), StateSyncError>
    where
        T: serde::Serialize + Send + 'static,
    {
        let _start_time = Instant::now();
        
        // This is a stub implementation
        // In a real implementation, we would:
        // 1. Validate the state data
        // 2. Update metrics
        // 3. Store the state data
        // 4. Notify subscribers
        
        Ok(())
    }
}

/// Container for state data with metadata
#[derive(Debug, Clone)]
pub struct StateData {
    /// The actual state data as a JSON value
    pub data: serde_json::Value,
    /// Timestamp when the state was last updated
    pub timestamp: Instant,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;
    

    #[derive(Serialize, Clone)]
    struct TestState {
        name: String,
        value: u32,
        timestamp: u64,
    }
    
    #[tokio::test]
    async fn test_sync_state_success() {
        let syncer = StateSynchronizer::default();
        
        let test_state = TestState {
            name: "test".to_string(),
            value: 42,
            timestamp: 1234567890,
        };
        
        let result = syncer.sync_state(
            StateType::Configuration,
            "test-state",
            "backup-system",
            test_state
        ).await;
        
        assert!(result.is_ok());
        
        // Check metrics
        let metrics = syncer.get_metrics().unwrap();
        assert_eq!(*metrics.successful_syncs.get(&StateType::Configuration).unwrap_or(&0), 1);
        assert!(metrics.failed_syncs.is_empty());
        assert!(metrics.total_bytes_synced > 0);
    }
    
    #[tokio::test]
    async fn test_sync_state_size_exceeded() {
        // Create a synchronizer with a very small size limit
        let syncer = StateSynchronizer::new(StateSyncConfig {
            max_state_size: 10, // Only 10 bytes allowed
            ..StateSyncConfig::default()
        });
        
        // Create a state that will exceed the limit
        let test_state = TestState {
            name: "this is a very long name that will exceed the size limit".to_string(),
            value: 42,
            timestamp: 1234567890,
        };
        
        let result = syncer.sync_state(
            StateType::Runtime,
            "test-state",
            "backup-system",
            test_state
        ).await;
        
        assert!(result.is_err());
        
        // Check that the error is the expected type
        match result {
            Err(StateSyncError::SizeExceeded { size, max_size }) => {
                assert!(size > 10);
                assert_eq!(max_size, 10);
            },
            _ => panic!("Expected SizeExceeded error"),
        }
        
        // Check metrics
        let metrics = syncer.get_metrics().unwrap();
        assert_eq!(*metrics.failed_syncs.get(&StateType::Runtime).unwrap_or(&0), 1);
        assert!(metrics.successful_syncs.is_empty());
    }
    
    #[tokio::test]
    async fn test_sync_multiple_state_types() {
        let syncer = StateSynchronizer::default();
        
        // Sync configuration state
        let config_state = TestState {
            name: "config".to_string(),
            value: 1,
            timestamp: 1000,
        };
        
        let result1 = syncer.sync_state(
            StateType::Configuration,
            "config-state",
            "backup-system",
            config_state
        ).await;
        assert!(result1.is_ok());
        
        // Sync runtime state
        let runtime_state = TestState {
            name: "runtime".to_string(),
            value: 2,
            timestamp: 2000,
        };
        
        let result2 = syncer.sync_state(
            StateType::Runtime,
            "runtime-state",
            "backup-system",
            runtime_state
        ).await;
        assert!(result2.is_ok());
        
        // Sync recovery state
        let recovery_state = TestState {
            name: "recovery".to_string(),
            value: 3,
            timestamp: 3000,
        };
        
        let result3 = syncer.sync_state(
            StateType::Recovery,
            "recovery-state",
            "backup-system",
            recovery_state
        ).await;
        assert!(result3.is_ok());
        
        // Check metrics
        let metrics = syncer.get_metrics().unwrap();
        assert_eq!(*metrics.successful_syncs.get(&StateType::Configuration).unwrap_or(&0), 1);
        assert_eq!(*metrics.successful_syncs.get(&StateType::Runtime).unwrap_or(&0), 1);
        assert_eq!(*metrics.successful_syncs.get(&StateType::Recovery).unwrap_or(&0), 1);
        assert!(metrics.failed_syncs.is_empty());
    }
    
    #[tokio::test]
    async fn test_reset_metrics() {
        let syncer = StateSynchronizer::default();
        
        // Perform a sync operation
        let test_state = TestState {
            name: "test".to_string(),
            value: 42,
            timestamp: 1234567890,
        };
        
        let _ = syncer.sync_state(
            StateType::Configuration,
            "test-state",
            "backup-system",
            test_state
        ).await;
        
        // Verify metrics are updated
        let metrics1 = syncer.get_metrics().unwrap();
        assert!(!metrics1.successful_syncs.is_empty());
        assert!(metrics1.total_bytes_synced > 0);
        
        // Reset metrics
        syncer.reset_metrics().unwrap();
        
        // Verify metrics are reset
        let metrics2 = syncer.get_metrics().unwrap();
        assert!(metrics2.successful_syncs.is_empty());
        assert_eq!(metrics2.total_bytes_synced, 0);
    }
} 