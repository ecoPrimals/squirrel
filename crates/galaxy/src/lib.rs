/*!
 * Galaxy MCP Adapter Crate
 * 
 * This crate provides an adapter for integrating Galaxy bioinformatics tools
 * with the Machine Context Protocol (MCP). It enables AI assistants to discover,
 * execute, and orchestrate Galaxy tools through a standardized protocol.
 * 
 * The crate is structured as an adapter that leverages the existing MCP and
 * context crates while providing Galaxy-specific extensions for bioinformatics
 * data types and operations.
 */

// Re-exports for common access
pub use error::Error;
pub use models::GalaxyDataReference;
pub use adapter::GalaxyAdapter;
pub use config::GalaxyConfig;
pub use plugin::{
    GalaxyPlugin,
    GalaxyToolPlugin,
    GalaxyWorkflowPlugin,
    GalaxyDatasetPlugin,
    GalaxyPluginManager,
};
pub use plugin::tool_plugin::create_tool_plugin;
pub use plugin::workflow_plugin::create_workflow_plugin;
pub use plugin::dataset_plugin::create_dataset_plugin;

// Module declarations
pub mod adapter;
pub mod api;
pub mod client;
pub mod config;
pub mod error;
pub mod models;
pub mod plugin;
pub mod security;

#[cfg(test)]
pub mod tests;

/// Convenience function to create a new Galaxy adapter with default configuration
pub async fn create_adapter() -> Result<adapter::GalaxyAdapter, error::Error> {
    let config = config::GalaxyConfig::default();
    adapter::GalaxyAdapter::new(config).await
}

/// Convenience function to create a new Galaxy adapter with a specific configuration
pub async fn create_adapter_with_config(config: config::GalaxyConfig) -> Result<adapter::GalaxyAdapter, error::Error> {
    adapter::GalaxyAdapter::new(config).await
}

/// Convenience function to create a new plugin manager for a Galaxy adapter
pub fn create_plugin_manager(adapter: std::sync::Arc<adapter::GalaxyAdapter>) -> plugin::GalaxyPluginManager {
    plugin::GalaxyPluginManager::new(adapter)
}

/// Version information about the Galaxy adapter crate
pub mod version {
    /// The current version of the Galaxy adapter crate
    pub const VERSION: &str = env!("CARGO_PKG_VERSION");
    /// The name of the Galaxy adapter crate
    pub const NAME: &str = env!("CARGO_PKG_NAME");
}
