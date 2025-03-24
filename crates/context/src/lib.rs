//! Context management for the Squirrel AI system
//!
//! This crate provides the context management functionality for the Squirrel AI system.
//! It includes the plugin system for extending context management capabilities.

// Public modules
pub mod plugins;
mod manager;
mod error;
#[cfg(test)]
mod tests;

// Re-exports from interfaces for convenience
pub use squirrel_interfaces::context::{
    ContextPlugin, 
    ContextTransformation, 
    ContextAdapterPlugin, 
    AdapterMetadata
};

// Error types for context operations
pub use error::{ContextError, Result};

// Context manager implementation
pub use manager::{ContextManager as ContextManagerImpl, ContextManagerConfig};

// Plugin manager implementation
pub use plugins::ContextPluginManager;

/// Create a new context manager with default configuration
pub fn create_default_manager() -> ContextManagerImpl {
    ContextManagerImpl::new()
}

/// Create a new context manager with the given configuration
pub fn create_manager_with_config(config: ContextManagerConfig) -> ContextManagerImpl {
    ContextManagerImpl::with_config(config)
} 