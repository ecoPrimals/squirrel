//! Core application functionality
//! 
//! This module provides the main Core struct that represents the application's core functionality.

use crate::VERSION;

/// Application configuration
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// Name of the application
    pub name: String,
    /// Version of the application
    pub version: String,
    /// Environment the application is running in
    pub environment: String,
    /// Whether debug mode is enabled
    pub debug: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            name: "Squirrel".to_string(),
            version: VERSION.to_string(),
            environment: "development".to_string(),
            debug: false,
        }
    }
}

/// Core application struct
#[derive(Debug)]
pub struct Core {
    /// Application version
    version: String,
    /// Application configuration
    pub config: AppConfig,
}

impl Core {
    /// Creates a new Core instance
    #[must_use]
    pub fn new() -> Self {
        Self {
            version: VERSION.to_string(),
            config: AppConfig::default(),
        }
    }

    /// Creates a new Core instance with the given configuration
    #[must_use]
    pub fn with_config(config: AppConfig) -> Self {
        Self {
            version: VERSION.to_string(),
            config,
        }
    }

    /// Returns the application version
    #[must_use]
    pub fn version(&self) -> &str {
        &self.version
    }
    
    /// Start the application
    /// 
    /// # Errors
    /// 
    /// Returns an error if the application fails to start
    pub async fn start(&mut self) -> crate::error::Result<()> {
        // Implementation placeholder - will be expanded as needed
        Ok(())
    }
    
    /// Stop the application
    /// 
    /// # Errors
    /// 
    /// Returns an error if the application fails to stop
    pub async fn stop(&mut self) -> crate::error::Result<()> {
        // Implementation placeholder - will be expanded as needed
        Ok(())
    }
}

impl Default for Core {
    fn default() -> Self {
        Self {
            version: VERSION.to_string(),
            config: AppConfig::default(),
        }
    }
} 