//! Squirrel Application library
//!
//! This crate provides the application-level components for the Squirrel platform.
//! It includes state management, UI components, and business logic.

#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]
#![warn(clippy::undocumented_unsafe_blocks)]
#![doc(html_root_url = "https://docs.rs/squirrel-app")]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![allow(clippy::needless_raw_string_hashes)]
#![allow(clippy::module_name_repetitions)]
#![warn(clippy::todo)]
#![cfg_attr(test, allow(clippy::unwrap_used))]

pub use crate::core::Core;
pub use crate::adapter::AppAdapter;

/// The current application version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Core application functionality
pub mod core;

/// Application adapter for interfacing with the system
pub mod adapter;

/// Application context functionality
pub mod context;

/// Application commands
pub mod command;

/// Application error handling
pub mod error;

/// Application event system
pub mod event;

/// Application event handling
pub mod events;

/// Application metrics
pub mod metrics;

/// Application monitoring
pub mod monitoring;

/// Application plugin system
pub mod plugin;

/// Re-exports
#[doc = "Common types for convenience"]
pub mod prelude {
    pub use crate::core::Core;
    pub use crate::adapter::AppAdapter;
    pub use crate::core::AppConfig;
    pub use crate::plugin::{Plugin, PluginManager, PluginLoader};
}

/// Module containing application tests
#[cfg(test)]
pub mod tests;

/// Public modules
pub mod config;

/// Application state
pub mod state;

/// User interface components
pub mod ui;

/// Domain models
pub mod models;

/// API client for MCP 
pub mod client;

/// Re-export core error handling
pub use squirrel_core::error::{Result, SquirrelError};

// Re-export commands crate
pub use commands as commands_crate;

// Re-export key plugin system components for easy use
pub use plugin::{
    Plugin, PluginMetadata, PluginState, PluginStatus, PluginError,
    PluginManager, 
    CommandPlugin, ToolPlugin, McpPlugin, UiPlugin,
    CommandPluginBuilder, ToolPluginBuilder, McpPluginBuilder,
    CommandPluginImpl, ToolPluginImpl, McpPluginImpl,
    PluginDiscovery, FileSystemDiscovery, PluginLoader,
    EnhancedPluginDiscovery, EnhancedPluginLoader, GenericPlugin,
    PluginStateStorage, FileSystemStateStorage, MemoryStateStorage, PluginStateManager,
    PermissionLevel, ResourceLimits, SecurityContext, ResourceUsage,
    PluginSandbox, BasicPluginSandbox, SecurityValidator, SecurityError,
    EnhancedSecurityValidator, SecurityAuditEntry, SecurityValidatorEnum,
    PluginRegistry, PluginCatalogEntry,
    examples::{
        create_example_plugins, create_advanced_command_plugin, 
        create_advanced_mcp_plugin, create_advanced_tool_plugin
    },
    resource_monitor
};

/// Plugin system example usage and documentation
///
/// This module provides a complete plugin system for extending application functionality
/// through plugins that implement the [`Plugin`] trait.
///
/// # Example
///
/// ```rust,ignore
/// use squirrel_app::{
///     PluginManager, plugin::{discovery::EnhancedPluginDiscovery, types::CommandPluginBuilder},
///     PluginMetadata,
/// };
/// use uuid::Uuid;
/// use std::path::Path;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     // Create plugin discovery system
///     let discovery = EnhancedPluginDiscovery::new(Path::new("./plugins"))?;
///     
///     // Create plugin manager
///     let manager = PluginManager::new();
///     
///     // Create a command plugin
///     let plugin = CommandPluginBuilder::new(PluginMetadata {
///         id: Uuid::new_v4(),
///         name: "example".to_string(),
///         version: "0.1.0".to_string(),
///         description: "Example plugin".to_string(),
///         author: "Example Author".to_string(),
///         dependencies: vec![],
///         capabilities: vec!["command".to_string()],
///     }).build();
///     
///     // Register the plugin
///     manager.register_plugin(plugin).await?;
///     
///     // Load all plugins
///     manager.load_all_plugins().await?;
///     
///     // Get all command plugins
///     let command_plugins = manager.get_all_command_plugins().await;
///     
///     // Shutdown when done
///     manager.shutdown().await?;
///     
///     Ok(())
/// }
/// ```
///
/// # Plugin Types
///
/// The plugin system supports several specialized plugin types:
///
/// - [`CommandPlugin`] - Adds new commands to the application
/// - [`ToolPlugin`] - Provides tools for various operations
/// - [`McpPlugin`] - Extends the Machine Context Protocol
/// - [`UiPlugin`] - Extends the user interface (currently deprecated)
///
/// # Creating Plugins
///
/// To simplify plugin creation, builder patterns are available:
///
/// - [`CommandPluginBuilder`] - For creating command plugins
/// - [`ToolPluginBuilder`] - For creating tool plugins
/// - [`McpPluginBuilder`] - For creating MCP plugins
///
/// # Plugin Discovery
///
/// The [`EnhancedPluginDiscovery`] system provides automatic discovery of plugins
/// from a directory with support for caching and monitoring for changes.
///
/// # Plugin Management
///
/// The [`PluginManager`] provides comprehensive management of plugins, including:
///
/// - Registration and loading
/// - Dependency resolution
/// - State persistence
/// - Security sandboxing
/// - Resource management
/// - Lifecycle management
///
/// # Plugin Security
///
/// The plugin system includes security features to prevent plugins from causing harm:
///
/// - Permission levels for operations
/// - Resource usage limits and tracking
/// - Sandboxing of plugin execution
/// - Security validation for operations
///
/// # Resource Monitoring
///
/// The plugin system now includes enhanced resource monitoring capabilities:
///
/// - Real-time memory, CPU, storage, and thread tracking
/// - Cross-platform support (Windows, Linux, macOS)
/// - Process-level resource usage monitoring
/// - Configurable monitoring intervals
/// - Resource usage alerts
/// - Integration with security model

/// Documentation module containing more examples and usage patterns
pub mod docs {
    /// Example of creating and using a command plugin
    pub mod plugin_examples {}
}
