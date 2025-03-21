//! Context management system for Squirrel
//!
//! This crate provides functionality for managing application context,
//! including state tracking, persistence, and recovery.

use thiserror::Error;
use serde::{Serialize, Deserialize};

/// Module for context state management
pub mod state;

/// Module for context tracking
pub mod tracker;

/// Module for context persistence
pub mod persistence;

/// Module for context recovery
pub mod recovery;

/// Module for context synchronization
pub mod sync;

/// Module for error types
pub mod error;

/// Errors that can occur during context operations
#[derive(Debug, Error)]
pub enum ContextError {
    /// Error related to state operations
    #[error("State error: {0}")]
    StateError(String),
    
    /// Error related to persistence operations
    #[error("Persistence error: {0}")]
    PersistenceError(String),
    
    /// Error related to recovery operations
    #[error("Recovery error: {0}")]
    RecoveryError(String),
    
    /// Error when a snapshot is not found
    #[error("Snapshot not found: {0}")]
    SnapshotNotFound(String),

    /// Error related to invalid state
    #[error("Invalid state: {0}")]
    InvalidState(String),

    /// Error related to synchronization operations
    #[error("Synchronization error: {0}")]
    SyncError(String),

    /// Error when no valid snapshot is found
    #[error("No valid snapshot: {0}")]
    NoValidSnapshot(String),
    
    /// Error when the context is not initialized
    #[error("Context not initialized")]
    NotInitialized,
}

/// Result type for context operations
pub type Result<T> = std::result::Result<T, ContextError>;

/// Manager for context module
pub mod manager;

/// Public exports for commonly used types
pub use persistence::PersistenceManager;
pub use tracker::ContextTracker;

/// Context state snapshot representation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContextSnapshot {
    /// Unique identifier for the snapshot
    pub id: String,
    /// Timestamp when the snapshot was created
    pub timestamp: u64,
    /// Serialized state data
    pub data: Vec<u8>,
}

/// Context state representation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContextState {
    /// Current version of the state
    pub version: u64,
    /// Timestamp of the last update - also accessible as last_modified for compatibility
    #[serde(rename = "last_modified")]
    pub last_updated: u64,
    /// State data
    pub data: Vec<u8>,
}

impl ContextState {
    /// Get the last modified timestamp
    #[must_use]
    pub fn last_modified(&self) -> u64 {
        self.last_updated
    }
}

#[cfg(test)]
mod tests; 