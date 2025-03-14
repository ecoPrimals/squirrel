//! Application core functionality for the Squirrel project
//!
//! This module provides the main application structure and core functionality.
//! It serves as the central coordination point for the application, managing
//! configuration, state, and providing access to other core components.

use std::sync::Arc;
use tokio::sync::RwLock;
use sled::Config;

pub mod error;

/// The core structure that manages the application's state and configuration
#[derive(Debug)]
pub struct Core {
    /// The application configuration, wrapped in a thread-safe read-write lock
    config: Arc<RwLock<Config>>,
    /// The current version of the application
    version: String,
}

impl Default for Core {
    fn default() -> Self {
        Self::new()
    }
}

impl Core {
    /// Creates a new Core instance with default configuration
    #[must_use]
    pub fn new() -> Self {
        let config = Config::default();
        Self::with_config(config)
    }

    /// Creates a new Core instance with the specified configuration
    #[must_use]
    pub fn with_config(config: Config) -> Self {
        let config = Arc::new(RwLock::new(config));
        Self { 
            config,
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    /// Gets the current configuration
    #[must_use]
    pub fn config(&self) -> Arc<RwLock<Config>> {
        self.config.clone()
    }

    /// Gets the version of the core module
    #[must_use]
    pub fn version(&self) -> &str {
        &self.version
    }
}

/// A Result type alias for core operations
pub type Result<T> = std::result::Result<T, anyhow::Error>; 