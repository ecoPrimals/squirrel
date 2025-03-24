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

// Re-export common types and traits
pub use plugin::{Plugin, PluginMetadata, PluginStatus};
pub use registry::{PluginRegistry, DefaultPluginRegistry, InMemoryPluginRegistry};
pub use discovery::{PluginDiscovery, DefaultPluginDiscovery};
pub use manager::PluginManager;
pub use monitoring::MonitoringPlugin;
pub use errors::{PluginError, Result};
pub use state::{PluginStateManager, MemoryStateManager, FileStateManager};

// Plugin system version
pub const PLUGIN_SYSTEM_VERSION: &str = env!("CARGO_PKG_VERSION");

// Plugin system name
pub const PLUGIN_SYSTEM_NAME: &str = env!("CARGO_PKG_NAME");

// Plugin system description
pub const PLUGIN_SYSTEM_DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

// Team-specific plugin types
pub use galaxy::GalaxyPlugin;
pub use commands::CommandsPlugin;

// Plugin system author
pub const PLUGIN_SYSTEM_AUTHOR: &str = env!("CARGO_PKG_AUTHORS");