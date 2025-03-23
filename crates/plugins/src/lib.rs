//! # Squirrel Plugins
//! 
//! This crate provides the plugin system for Squirrel, enabling extensibility
//! and modular design across the application.
//!
//! The plugin system allows for dynamic loading of functionality, with a focus
//! on type safety and performance.

// Modules
pub mod discovery;
pub mod errors;
pub mod manager;
pub mod plugin;
pub mod state;
pub mod types;

// New specialized plugin modules
pub mod mcp;
pub mod tools;
pub mod security;
pub mod registry;
pub mod distribution;

// Team-specific crate plugin modules
pub mod web;
pub mod monitoring;
pub mod galaxy;
pub mod cli;
pub mod app;
pub mod context;
pub mod commands;
pub mod test_utils;
pub mod context_adapter;

// Re-exports
pub use errors::{PluginError, Result};
pub use plugin::{Plugin, PluginMetadata, PluginStatus, WebEndpoint, WebPluginExt};
pub use manager::{PluginManager, PluginManagerTrait, DefaultPluginManager};
pub use discovery::DefaultPluginDiscovery;
pub use state::{PluginState, PluginStateManager, MemoryStateManager, FileStateManager};

// Type re-exports
pub use types::{
    PLUGIN_TYPE_CORE,
    PLUGIN_TYPE_WEB,
    PLUGIN_TYPE_MCP,
    PLUGIN_TYPE_TOOL,
    PLUGIN_TYPE_CLI,
    CorePlugin,
};

#[cfg(feature = "cli")]
pub use types::CliPlugin;

// Specialized plugin types
pub use mcp::McpPlugin;
pub use tools::ToolPlugin;
pub use security::{SecurityManager, DefaultSecurityManager};
pub use distribution::{PluginDistribution, DefaultPluginDistribution};
pub use registry::PluginRegistry as PluginRegistryImpl;

// Team-specific plugin types
pub use web::WebPlugin;
pub use monitoring::MonitoringPlugin;
pub use galaxy::GalaxyPlugin;
pub use cli::CliPlugin as CliPluginExt;
pub use app::AppPlugin;
pub use context::ContextPlugin;
pub use commands::CommandsPlugin;
pub use test_utils::TestUtilsPlugin;
pub use context_adapter::ContextAdapterPlugin;

// Re-export the plugin registry type
pub use registry::PluginRegistry; 