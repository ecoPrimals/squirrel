//! Context management system
//!
//! This crate provides a robust context management system for tracking,
//! persisting, and synchronizing application state.

use std::sync::Arc;
use std::fmt;

// Internal modules
pub mod state;
pub mod manager;
pub mod tracker;
pub mod persistence;
pub mod recovery;
pub mod adapter;

// Public re-exports
pub use manager::{ContextManager, ContextManagerConfig};
pub use tracker::{ContextTracker, ContextTrackerFactory, ContextTrackerConfig};
pub use state::{State as ContextState, StateSnapshot as ContextSnapshot};
pub use adapter::{ContextAdapter, ContextAdapterConfig, ContextStatus};

/// Error types for context operations
#[derive(Debug, PartialEq, Eq)]
pub enum ContextError {
    /// Not initialized
    NotInitialized(String),
    /// Invalid state
    InvalidState(String),
    /// Persistence error
    Persistence(String),
    /// State error
    StateError(String),
    /// State not found
    NotFound(String),
    /// Snapshot not found
    SnapshotNotFound(String),
    /// No recovery points available
    NoRecoveryPoints(String),
}

// Implement Display for ContextError
impl fmt::Display for ContextError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContextError::NotInitialized(msg) => write!(f, "Not initialized: {}", msg),
            ContextError::InvalidState(msg) => write!(f, "Invalid state: {}", msg),
            ContextError::Persistence(msg) => write!(f, "Persistence error: {}", msg),
            ContextError::StateError(msg) => write!(f, "State error: {}", msg),
            ContextError::NotFound(msg) => write!(f, "Not found: {}", msg),
            ContextError::SnapshotNotFound(msg) => write!(f, "Snapshot not found: {}", msg),
            ContextError::NoRecoveryPoints(msg) => write!(f, "No recovery points: {}", msg),
        }
    }
}

// Implement std::error::Error for ContextError
impl std::error::Error for ContextError {}

/// Result type alias
pub type Result<T> = std::result::Result<T, ContextError>;

/// Create a new context manager with default configuration
///
/// # Returns
///
/// A new context manager instance with default configuration
#[must_use]
pub fn create_manager() -> Arc<ContextManager> {
    Arc::new(ContextManager::new())
}

/// Create a new context manager with custom configuration
///
/// # Arguments
///
/// * `config` - Configuration for the context manager
///
/// # Returns
///
/// A new context manager instance with the specified configuration
#[must_use]
pub fn create_manager_with_config(config: ContextManagerConfig) -> Arc<ContextManager> {
    Arc::new(ContextManager::with_config(config))
}

/// Create a new context adapter with default configuration
///
/// # Arguments
///
/// * `manager` - Reference to a context manager
///
/// # Returns
///
/// A new context adapter instance with default configuration
#[must_use]
pub fn create_adapter(manager: Arc<ContextManager>) -> ContextAdapter {
    ContextAdapter::new(manager)
}

/// Create a new context adapter with custom configuration
///
/// # Arguments
///
/// * `manager` - Reference to a context manager
/// * `config` - Configuration for the context adapter
///
/// # Returns
///
/// A new context adapter instance with the specified configuration
#[must_use]
pub fn create_adapter_with_config(
    manager: Arc<ContextManager>,
    config: ContextAdapterConfig,
) -> ContextAdapter {
    ContextAdapter::with_config(manager, config)
}

#[cfg(test)]
mod tests; 