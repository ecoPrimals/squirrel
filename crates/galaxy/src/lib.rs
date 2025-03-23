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

// Module declarations
pub mod adapter;
pub mod api;
pub mod client;
pub mod config;
pub mod error;
pub mod models;
pub mod security;

#[cfg(test)]
pub mod tests;

/// Convenience function to create a new Galaxy adapter with default configuration
pub fn create_adapter() -> Result<adapter::GalaxyAdapter, error::Error> {
    let config = config::GalaxyConfig::default();
    adapter::GalaxyAdapter::new(config)
}

/// Convenience function to create a new Galaxy adapter with a specific configuration
pub fn create_adapter_with_config(config: config::GalaxyConfig) -> Result<adapter::GalaxyAdapter, error::Error> {
    adapter::GalaxyAdapter::new(config)
}

/// Version information about the Galaxy adapter crate
pub mod version {
    /// The current version of the Galaxy adapter crate
    pub const VERSION: &str = env!("CARGO_PKG_VERSION");
    /// The name of the Galaxy adapter crate
    pub const NAME: &str = env!("CARGO_PKG_NAME");
}
