//! Plugin system for Squirrel
//!
//! This module provides a comprehensive plugin system with support for:
//! - Plugin loading and management
//! - Security validation and sandboxing
//! - Dependency resolution
//! - Plugin discovery
//! - State management

mod default_manager;
mod dependency_resolver;
mod discovery;
mod errors;
mod manager;
mod metrics;
mod plugin;
mod plugin_v2;
mod registry;
mod state;
mod traits;
mod types;

// Re-export public API
pub use default_manager::DefaultPluginManager;
pub use dependency_resolver::{
    DependencyResolver, EnhancedPluginDependency, ResolutionResult, ResolutionStatistics,
};
pub use discovery::{DefaultPluginDiscovery, PluginDiscovery};
pub use errors::{PluginError, Result};
pub use manager::PluginManager;
pub use metrics::{PluginManagerMetrics, PluginManagerStatus};
pub use plugin::{Plugin, PluginMetadata};
pub use plugin_v2::PluginV2;
pub use registry::PluginRegistry;
pub use state::{FileStateManager, MemoryStateManager, PluginStateManager};
pub use traits::PluginManagerTrait;
pub use types::{PluginConfig, PluginStatus}; // Add PluginConfig export
