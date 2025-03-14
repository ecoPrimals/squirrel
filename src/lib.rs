#![deny(missing_docs)]
#![deny(clippy::missing_docs_in_private_items)]
#![warn(clippy::undocumented_unsafe_blocks)]

//! Machine Context Protocol (MCP) implementation for the DataScienceBioLab project.
//! 
//! This crate provides the core functionality for component-based UI management,
//! event handling, and layout management.

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

use std::error::Error as StdError;

/// A boxed error type that can be sent between threads
pub type Error = Box<dyn std::error::Error + Send + Sync>;
/// A result type that uses our Error type
pub type Result<T> = std::result::Result<T, Error>;

/// Initialize all systems
pub async fn initialize() -> Result<()> {
    monitoring::metrics::initialize().await.map_err(Into::into)?;
    monitoring::logging::initialize().await.map_err(Into::into)?;
    security::audit::initialize().await.map_err(Into::into)?;
    security::auth::initialize().await.map_err(Into::into)?;
    security::encryption::initialize().await.map_err(Into::into)?;
    data::storage::initialize().await.map_err(Into::into)?;
    data::versioning::initialize().await.map_err(Into::into)?;
    data::migration::initialize().await.map_err(Into::into)?;
    Ok(())
}

/// Shutdown all systems
pub async fn shutdown() -> Result<()> {
    data::migration::shutdown().await.map_err(Into::into)?;
    data::versioning::shutdown().await.map_err(Into::into)?;
    data::storage::shutdown().await.map_err(Into::into)?;
    security::encryption::shutdown().await.map_err(Into::into)?;
    security::auth::shutdown().await.map_err(Into::into)?;
    security::audit::shutdown().await.map_err(Into::into)?;
    monitoring::logging::shutdown().await.map_err(Into::into)?;
    monitoring::metrics::shutdown().await.map_err(Into::into)?;
    Ok(())
} 