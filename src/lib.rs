//! Squirrel - A modular and maintainable Rust project

pub mod security;
pub mod monitoring;
pub mod data;
pub mod deployment;
pub mod core;
pub mod ai;
pub mod mcp;
pub mod mcp_tools;

#[cfg(feature = "ui")]
pub mod ui;

use anyhow::Result;

/// Initialize all systems
pub async fn initialize() -> Result<()> {
    // Initialize security systems
    security::auth::initialize().await?;
    security::encryption::initialize().await?;
    security::audit::initialize().await?;

    // Initialize monitoring systems
    monitoring::tracing::initialize().await?;
    monitoring::logging::initialize().await?;
    monitoring::metrics::initialize().await?;

    // Initialize data systems
    data::storage::initialize().await?;
    data::versioning::initialize().await?;
    data::migration::initialize().await?;

    // Initialize deployment systems
    deployment::initialize().await?;

    // Initialize UI if enabled
    #[cfg(feature = "ui")]
    ui::initialize().await?;

    Ok(())
}

/// Shutdown all systems
pub async fn shutdown() -> Result<()> {
    // Shutdown UI if enabled
    #[cfg(feature = "ui")]
    ui::shutdown().await?;

    // Shutdown deployment systems
    deployment::shutdown().await?;

    // Shutdown data systems
    data::migration::shutdown().await?;
    data::versioning::shutdown().await?;
    data::storage::shutdown().await?;

    // Shutdown monitoring systems
    monitoring::metrics::shutdown().await?;
    monitoring::logging::shutdown().await?;
    monitoring::tracing::shutdown().await?;

    // Shutdown security systems
    security::audit::shutdown().await?;
    security::encryption::shutdown().await?;
    security::auth::shutdown().await?;

    Ok(())
} 