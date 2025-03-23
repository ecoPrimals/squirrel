//! # Squirrel Plugin System
//!
//! This crate provides a plugin system for extending Squirrel functionality.
//! It allows for dynamic loading, management, and execution of plugins across
//! different components of the Squirrel platform.
//!
//! ## Core Features
//!
//! - Plugin discovery and loading
//! - Plugin lifecycle management
//! - Plugin state persistence
//! - Dependency resolution
//! - Security sandboxing
//!
//! ## Plugin Types
//!
//! - Core plugins: Extend core functionality
//! - MCP plugins: Extend Machine Context Protocol
//! - Web plugins: Extend web interface
//! - Tool plugins: Provide tool implementations

use std::any::Any;
use std::fmt::Debug;
use std::sync::Arc;

use async_trait::async_trait;
use futures::future::BoxFuture;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

mod core;
mod discovery;
mod manager;
mod state;
mod types;
mod plugin;

#[cfg(feature = "web")]
pub mod web;

#[cfg(feature = "mcp")]
pub mod mcp;

#[cfg(feature = "cli")]
pub mod cli;

// Re-export core types
pub use core::{Plugin, PluginMetadata, PluginStatus};
pub use discovery::{PluginDiscovery, PluginLoader};
pub use manager::{PluginManager, PluginRegistry};
pub use state::{PluginState, PluginStateManager};
pub use types::*;
pub use plugin::{Plugin as PluginTrait, PluginMetadata as PluginMetaData, WebPluginExt};

/// Plugin system error types
#[derive(Debug, Error)]
pub enum PluginError {
    /// Plugin not found
    #[error("Plugin not found: {0}")]
    NotFound(Uuid),
    
    /// Plugin already registered
    #[error("Plugin already registered: {0}")]
    AlreadyRegistered(Uuid),
    
    /// Plugin dependency not found
    #[error("Plugin dependency not found: {0}")]
    DependencyNotFound(String),
    
    /// Plugin dependency cycle detected
    #[error("Plugin dependency cycle detected: {0}")]
    DependencyCycle(Uuid),
    
    /// Plugin initialization failed
    #[error("Plugin initialization failed: {0}")]
    InitializationFailed(String),
    
    /// Plugin shutdown failed
    #[error("Plugin shutdown failed: {0}")]
    ShutdownFailed(String),
    
    /// Plugin state error
    #[error("Plugin state error: {0}")]
    StateError(String),
    
    /// Plugin loading error
    #[error("Plugin loading error: {0}")]
    LoadingError(String),
    
    /// Plugin validation error
    #[error("Plugin validation error: {0}")]
    ValidationError(String),
    
    /// Security constraint
    #[error("Security constraint: {0}")]
    SecurityConstraint(String),
    
    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    /// Other error
    #[error("Other error: {0}")]
    Other(String),
}

/// Result type for plugin operations
pub type Result<T> = std::result::Result<T, PluginError>; 