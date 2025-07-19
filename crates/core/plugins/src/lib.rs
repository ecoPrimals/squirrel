//! Plugin system for Squirrel
//!
//! This crate provides the core plugin system functionality for Squirrel,
//! including plugin loading, management, and execution.
//!
//! ## Features
//!
//! - Plugin loading and unloading
//! - Plugin lifecycle management
//! - Plugin dependency resolution
//! - Plugin state management
//! - Plugin execution framework
//! - Plugin metadata and configuration
//! - Security handled by BearDog framework

pub mod cli;
pub mod commands;
pub mod core;
pub mod dependency_resolver;
pub mod discovery;
pub mod errors;
pub mod manager;
pub mod mcp;
pub mod monitoring;
pub mod plugin;
pub mod registry;
pub mod state;
pub mod types;

// Re-exports for convenience
pub use commands::{Command, CommandMetadata, CommandsPlugin, CommandsPluginBuilder};
pub use dependency_resolver::DependencyResolver;
pub use discovery::PluginDiscovery;
pub use errors::{PluginError, Result};
pub use manager::PluginManager;
pub use plugin::{Plugin, PluginMetadata, PluginStatus};
pub use registry::PluginRegistry;
pub use state::{FileStateManager, MemoryStateManager, PluginStateManager};
pub use types::*;

// Platform-specific modules
#[cfg(feature = "web")]
pub mod web;

#[cfg(feature = "web")]
pub use web::{
    PluginDashboard, PluginManagementAPI, PluginManagementFactory, PluginManagementInterface,
    PluginMarketplaceClient, WebEndpoint, WebPluginExt, WebSocketMessage,
};

// Dynamic plugin support
pub mod plugins;
pub use plugins::dynamic;

// Additional integrations
// pub use galaxy::GalaxyPlugin;  // Removed: Complex Galaxy tools moved to ToadStool
pub use mcp::McpPlugin;
pub use monitoring::MonitoringPlugin;

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

/// The author information for the plugin system.
pub const PLUGIN_SYSTEM_AUTHOR: &str = env!("CARGO_PKG_AUTHORS");

/// Default configuration for plugin system
#[derive(Debug, Clone)]
pub struct PluginConfig {
    /// Maximum number of plugins to load
    pub max_plugins: usize,
    /// Plugin discovery directory
    pub plugin_directory: String,
    /// Enable security verification
    pub verify_signatures: bool,
    /// Enable plugin sandboxing
    pub enable_sandboxing: bool,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            max_plugins: 100,
            plugin_directory: "./plugins".to_string(),
            verify_signatures: true,
            enable_sandboxing: true,
        }
    }
}

// Note: types are already re-exported above, no need for duplicate imports
