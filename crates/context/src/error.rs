//! Error types for the context module

use crate::ContextError;

/// Result type alias for context errors
pub type Result<T> = std::result::Result<T, ContextError>;

/// Create a state error
#[must_use]
pub fn state_error(msg: &str) -> ContextError {
    ContextError::StateError(msg.to_string())
}

/// Create a persistence error
#[must_use]
pub fn persistence_error(msg: &str) -> ContextError {
    ContextError::PersistenceError(msg.to_string())
}

/// Create a recovery error
#[must_use]
pub fn recovery_error(msg: &str) -> ContextError {
    ContextError::RecoveryError(msg.to_string())
}

/// Create a snapshot not found error
#[must_use]
pub fn snapshot_not_found(msg: &str) -> ContextError {
    ContextError::SnapshotNotFound(msg.to_string())
}

/// Create an invalid state error
#[must_use]
pub fn invalid_state(msg: &str) -> ContextError {
    ContextError::InvalidState(msg.to_string())
}

/// Create a sync error
#[must_use]
pub fn sync_error(msg: &str) -> ContextError {
    ContextError::SyncError(msg.to_string())
}

/// Create a no valid snapshot error
#[must_use]
pub fn no_valid_snapshot(msg: &str) -> ContextError {
    ContextError::NoValidSnapshot(msg.to_string())
} 