//! Plugin system for Squirrel
//!
//! This module provides a comprehensive plugin system with support for:
//! - Plugin loading and management
//! - Security validation and sandboxing
//! - Dependency resolution
//! - Plugin discovery
//! - State management

// Allow deprecated items during plugin system migration to squirrel_interfaces
#![allow(deprecated)]

mod default_manager;
mod dependency_resolver;
mod discovery;
mod errors;
mod manager;
mod metrics;
mod performance_optimizer;
mod plugin;
mod plugin_v2;
mod registry;
mod state;
mod traits;
mod types;
mod unified_manager;
mod zero_copy;

// Platform-specific modules
pub mod cli;
pub mod mcp;
pub mod web;

// Re-export public API
pub use default_manager::DefaultPluginManager;
pub use dependency_resolver::{
    DependencyResolver, EnhancedPluginDependency, ResolutionResult, ResolutionStatistics,
};
pub use discovery::{DefaultPluginDiscovery, PluginDiscovery};
pub use errors::{PluginError, Result};
pub use manager::PluginManager;
pub use metrics::{PluginManagerMetrics, PluginManagerStatus};
pub use performance_optimizer::{
    get_global_optimizer, init_global_optimizer, optimized_ops, BatchProcessingConfig,
    HotPathCacheConfig, MemoryOptimizationConfig, OptimizerMetrics, PerformanceOptimizerConfig,
    PluginPerformanceOptimizer, PredictiveLoadingConfig,
};
pub use plugin::Plugin;
// Re-export canonical PluginMetadata from interfaces
pub use plugin_v2::PluginV2;
pub use registry::PluginRegistry;
pub use squirrel_interfaces::plugins::PluginMetadata;
pub use state::{FileStateManager, MemoryStateManager, PluginStateManager};
pub use traits::PluginManagerTrait;
pub use types::{PluginConfig, PluginStatus}; // Add PluginConfig export
pub use unified_manager::{
    ManagerMetrics, PluginEventBus, PluginSecurityManager, UnifiedManagerConfig,
    UnifiedPluginLoader, UnifiedPluginManager,
};
pub use zero_copy::{
    PluginEvent, PluginMetadataBuilder, PluginMetrics, RegistryStats, ResourceLimits,
    SecuritySettings, StateTransition, ZeroCopyPlugin, ZeroCopyPluginConfig, ZeroCopyPluginEntry,
    ZeroCopyPluginMetadata, ZeroCopyPluginRegistry, ZeroCopyPluginState,
};
