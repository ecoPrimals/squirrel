//! Plugin system for Squirrel
//!
//! This crate provides a plugin system for Squirrel.

// Modules
pub mod core;
pub mod discovery;
pub mod errors;
pub mod manager;
pub mod plugin;
pub mod state;
pub mod types;

#[cfg(feature = "web")]
pub mod web;

// Re-exports
pub use errors::{PluginError, Result};
pub use plugin::{Plugin, PluginMetadata, PluginStatus, WebEndpoint, WebPluginExt};
pub use manager::{PluginManager, PluginRegistry, PluginManagerTrait, DefaultPluginManager};
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
    ToolPlugin,
};

#[cfg(feature = "cli")]
pub use types::CliPlugin;

#[cfg(feature = "mcp")]
pub use types::McpPlugin; 