//! Core application functionality
//! 
//! This module provides the main Core struct that represents the application's core functionality.

use crate::VERSION;
use squirrel_core::error::{SquirrelError, AppOperationError};

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

/// Application state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppState {
    /// Application is initialized but not started
    Initialized,
    /// Application is running
    Running,
    /// Application is stopped
    Stopped,
}

/// Core application struct
#[derive(Debug)]
pub struct Core {
    /// Application version
    version: String,
    /// Application configuration
    pub config: AppConfig,
    /// Application state
    state: AppState,
}

impl Core {
    /// Creates a new Core instance
    #[must_use]
    pub fn new() -> Self {
        Self {
            version: VERSION.to_string(),
            config: AppConfig::default(),
            state: AppState::Initialized,
        }
    }

    /// Creates a new Core instance with the given configuration
    #[must_use]
    pub fn with_config(config: AppConfig) -> Self {
        Self {
            version: VERSION.to_string(),
            config,
            state: AppState::Initialized,
        }
    }

    /// Returns the application version
    #[must_use]
    pub fn version(&self) -> &str {
        &self.version
    }
    
    /// Starts the app
    /// 
    /// # Errors
    /// 
    /// Returns an error if:
    /// - The app is already running
    /// - The app is stopped
    pub fn start(&mut self) -> crate::error::Result<()> {
        match self.state {
            AppState::Running => Err(SquirrelError::AppOperation(AppOperationError::AlreadyStarted).into()),
            AppState::Stopped => Err(SquirrelError::AppOperation(AppOperationError::AlreadyStopped).into()),
            AppState::Initialized => {
                self.state = AppState::Running;
                Ok(())
            }
        }
    }
    
    /// Stops the app
    /// 
    /// # Errors
    /// 
    /// Returns an error if:
    /// - The app is not running
    /// - The app is already stopped
    pub fn stop(&mut self) -> crate::error::Result<()> {
        match self.state {
            AppState::Initialized => Err(SquirrelError::AppOperation(AppOperationError::NotStarted).into()),
            AppState::Stopped => Err(SquirrelError::AppOperation(AppOperationError::AlreadyStopped).into()),
            AppState::Running => {
                self.state = AppState::Stopped;
                Ok(())
            }
        }
    }
}

impl Default for Core {
    fn default() -> Self {
        Self {
            version: VERSION.to_string(),
            config: AppConfig::default(),
            state: AppState::Initialized,
        }
    }
} 