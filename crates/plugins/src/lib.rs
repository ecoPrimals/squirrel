//! # Squirrel Plugins
//! 
//! This crate provides the plugin system for Squirrel, enabling extensibility
//! and modular design across the application.
//!
//! The plugin system allows for dynamic loading of functionality, with a focus
//! on type safety and performance.
//!
//! The plugin system consists of several components:
//! - Plugin trait definition
//! - Plugin metadata
//! - Plugin manager
//! - Plugin discovery
//! - Plugin registry
//! - Plugin distribution
//! - Plugin security
//! - Plugin configuration
//! - Machine Context Protocol integration

pub mod plugin;
pub mod discovery;
pub mod manager;
pub mod registry;
pub mod distribution;
pub mod security;
pub mod monitoring;
pub mod galaxy;
pub mod mcp;
pub mod cli;
pub mod errors;
pub mod state;
pub mod commands;
pub mod tools;
pub mod simple_test;
pub mod plugins;
pub mod test_utils;
pub mod simple_test_utils;

// Include test modules
#[cfg(test)]
mod tests;

// Include standalone test module for simpler testing
#[cfg(test)]
mod standalone_test;

// Re-export common types and traits
pub use plugin::{Plugin, PluginMetadata, PluginStatus};
pub use registry::{PluginRegistry, DefaultPluginRegistry, InMemoryPluginRegistry};
pub use discovery::{PluginDiscovery, DefaultPluginDiscovery};
pub use manager::PluginManager;
pub use monitoring::MonitoringPlugin;
pub use errors::{PluginError, Result};
pub use state::{PluginStateManager, MemoryStateManager, FileStateManager};
pub use commands::{CommandsPlugin, CommandsPluginBuilder, Command, CommandMetadata};

// Export marketplace types
#[cfg(feature = "marketplace")]
pub use plugins::marketplace::{
    RepositoryManager,
    RepositoryProvider,
    HttpRepositoryProvider,
    RepositoryInfo,
    PluginPackageInfo,
    create_repository_manager,
};

// Export dynamic plugin types
pub use plugins::dynamic;

/// The current version of the plugin system.
/// 
/// This version is used for compatibility checking between plugins and the system.
pub const PLUGIN_SYSTEM_VERSION: &str = env!("CARGO_PKG_VERSION");

/// The name of the plugin system.
pub const PLUGIN_SYSTEM_NAME: &str = env!("CARGO_PKG_NAME");

/// The description of the plugin system.
/// 
/// Contains a brief explanation of the plugin system's purpose.
pub const PLUGIN_SYSTEM_DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

// Team-specific plugin types
pub use galaxy::GalaxyPlugin;

/// The author information for the plugin system.
pub const PLUGIN_SYSTEM_AUTHOR: &str = env!("CARGO_PKG_AUTHORS");