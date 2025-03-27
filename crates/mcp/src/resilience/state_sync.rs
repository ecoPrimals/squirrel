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
use serde::{Serialize, Deserialize};

/// Represents the type of state being synchronized
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StateType {
    /// Configuration state (system settings, parameters)
    Configuration,
    
    /// Runtime state (current system state during operation)
    Runtime,
    
    /// Recovery state (data needed for recovery procedures)
    Recovery,
    
    /// Audit state (logs, metrics, history data)
    Audit,
}

impl fmt::Display for StateType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StateType::Configuration => write!(f, "Configuration"),
            StateType::Runtime => write!(f, "Runtime"),
            StateType::Recovery => write!(f, "Recovery"),
            StateType::Audit => write!(f, "Audit"),
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
        duration: Duration,
    },
    
    /// State data exceeds maximum allowed size
    SizeExceeded {
        size: usize,
        max_size: usize,
    },
    
    /// State validation failed
    ValidationFailed {
        message: String,
    },
    
    /// Target system not found or unavailable
    TargetUnavailable {
        target: String,
    },
    
    /// Serialization or deserialization error
    SerializationError {
        message: String,
    },
    
    /// General error during synchronization
    SyncFailed {
        message: String,
        source: Option<Box<dyn StdError + Send + Sync + 'static>>,
    },
}

impl fmt::Display for StateSyncError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StateSyncError::Timeout { duration } => {
                write!(f, "Synchronization timed out after {:?}", duration)
            }
            StateSyncError::SizeExceeded { size, max_size } => {
                write!(f, "State size ({} bytes) exceeds maximum allowed size ({} bytes)", 
                    size, max_size)
            }
            StateSyncError::ValidationFailed { message } => {
                write!(f, "State validation failed: {}", message)
            }
            StateSyncError::TargetUnavailable { target } => {
                write!(f, "Target system unavailable: {}", target)
            }
            StateSyncError::SerializationError { message } => {
                write!(f, "Serialization error: {}", message)
            }
            StateSyncError::SyncFailed { message, .. } => {
                write!(f, "Synchronization failed: {}", message)
            }
        }
    }
}

impl StdError for StateSyncError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            StateSyncError::SyncFailed { source, .. } => {
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
}

impl StateSyncMetrics {
    /// Reset all metrics to default values
    pub fn reset(&mut self) {
        self.successful_syncs.clear();
        self.failed_syncs.clear();
        self.total_bytes_synced = 0;
        self.last_sync_time = None;
    }
}

/// A state synchronizer for maintaining consistent state across distributed systems
#[derive(Debug, Clone)]
pub struct StateSynchronizer {
    /// Configuration for the synchronizer
    config: StateSyncConfig,
    
    /// Metrics about synchronization operations
    metrics: Arc<Mutex<StateSyncMetrics>>,
}

impl StateSynchronizer {
    /// Create a new state synchronizer with the given configuration
    pub fn new(config: StateSyncConfig) -> Self {
        Self {
            config,
            metrics: Arc::new(Mutex::new(StateSyncMetrics::default())),
        }
    }
    
    /// Create a new state synchronizer with default configuration
    pub fn default() -> Self {
        Self::new(StateSyncConfig::default())
    }
    
    /// Synchronize state data to a target system
    pub fn sync_state<T>(&self, 
        state_type: StateType, 
        state_id: &str, 
        target: &str, 
        state: T
    ) -> Result<(), StateSyncError> 
    where 
        T: Serialize + Clone + Send + Sync + 'static
    {
        let start_time = Instant::now();
        self.metrics.lock().unwrap().last_sync_time = Some(start_time);
        
        // Serialize the state to estimate its size
        let serialized = serde_json::to_vec(&state)
            .map_err(|e| StateSyncError::SerializationError {
                message: e.to_string(),
            })?;
        
        // Check if state exceeds maximum size
        if serialized.len() > self.config.max_state_size {
            // Update metrics for failure
            let mut metrics = self.metrics.lock().unwrap();
            *metrics.failed_syncs.entry(state_type).or_insert(0) += 1;
            
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
        
        // Check timeout
        if start_time.elapsed() > self.config.sync_timeout {
            // Update metrics for failure
            let mut metrics = self.metrics.lock().unwrap();
            *metrics.failed_syncs.entry(state_type).or_insert(0) += 1;
            
            return Err(StateSyncError::Timeout {
                duration: self.config.sync_timeout,
            });
        }
        
        // In a real implementation, this would communicate with the target system
        // For now, we'll just simulate success
        
        // Update metrics for success
        let mut metrics = self.metrics.lock().unwrap();
        *metrics.successful_syncs.entry(state_type).or_insert(0) += 1;
        metrics.total_bytes_synced += serialized.len();
        
        Ok(())
    }
    
    /// Synchronize state with timeout
    pub fn sync_state_with_timeout<T>(&self,
        state_type: StateType,
        state_id: &str,
        target: &str,
        state: T,
        timeout: Duration
    ) -> Result<(), StateSyncError>
    where
        T: Serialize + Clone + Send + Sync + 'static
    {
        let start_time = Instant::now();
        
        // Create a custom timeout for this operation
        let result = self.sync_state(state_type, state_id, target, state);
        
        if start_time.elapsed() > timeout {
            Err(StateSyncError::Timeout { duration: timeout })
        } else {
            result
        }
    }
    
    /// Get the current synchronization metrics
    pub fn get_metrics(&self) -> StateSyncMetrics {
        self.metrics.lock().unwrap().clone()
    }
    
    /// Reset the synchronization metrics
    pub fn reset_metrics(&self) {
        self.metrics.lock().unwrap().reset();
    }
    
    /// Get the current configuration
    pub fn get_config(&self) -> &StateSyncConfig {
        &self.config
    }
    
    /// Update the configuration
    pub fn update_config(&mut self, config: StateSyncConfig) {
        self.config = config;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // A test data type
    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TestState {
        name: String,
        value: u32,
        timestamp: u64,
    }
    
    #[test]
    fn test_sync_state_success() {
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
        );
        
        assert!(result.is_ok());
        
        // Check metrics
        let metrics = syncer.get_metrics();
        assert_eq!(*metrics.successful_syncs.get(&StateType::Configuration).unwrap_or(&0), 1);
        assert!(metrics.failed_syncs.is_empty());
        assert!(metrics.total_bytes_synced > 0);
    }
    
    #[test]
    fn test_sync_state_size_exceeded() {
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
        );
        
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
        let metrics = syncer.get_metrics();
        assert_eq!(*metrics.failed_syncs.get(&StateType::Runtime).unwrap_or(&0), 1);
        assert!(metrics.successful_syncs.is_empty());
    }
    
    #[test]
    fn test_sync_multiple_state_types() {
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
        );
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
        );
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
        );
        assert!(result3.is_ok());
        
        // Check metrics
        let metrics = syncer.get_metrics();
        assert_eq!(*metrics.successful_syncs.get(&StateType::Configuration).unwrap_or(&0), 1);
        assert_eq!(*metrics.successful_syncs.get(&StateType::Runtime).unwrap_or(&0), 1);
        assert_eq!(*metrics.successful_syncs.get(&StateType::Recovery).unwrap_or(&0), 1);
        assert!(metrics.failed_syncs.is_empty());
    }
    
    #[test]
    fn test_reset_metrics() {
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
        );
        
        // Verify metrics are updated
        let metrics1 = syncer.get_metrics();
        assert!(!metrics1.successful_syncs.is_empty());
        assert!(metrics1.total_bytes_synced > 0);
        
        // Reset metrics
        syncer.reset_metrics();
        
        // Verify metrics are reset
        let metrics2 = syncer.get_metrics();
        assert!(metrics2.successful_syncs.is_empty());
        assert_eq!(metrics2.total_bytes_synced, 0);
    }
} 