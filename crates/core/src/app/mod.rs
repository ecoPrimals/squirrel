//! Core application module for Squirrel.
//! 
//! This module provides the main application components including
//! the App struct and its adapter implementations.

use std::sync::{Arc, Mutex, RwLock};
use crate::error::{AppInitializationError, AppOperationError, SquirrelError};

#[cfg(test)]
pub mod tests;

/// App configuration
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// Name of the application
    pub name: String,
    /// Version of the application
    pub version: String,
    /// Additional configuration options
    pub options: Vec<(String, String)>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            name: "Squirrel".to_string(),
            version: "0.1.0".to_string(),
            options: Vec::new(),
        }
    }
}

/// Core application state
#[derive(Debug)]
pub struct AppState {
    /// Whether the application is initialized
    pub initialized: bool,
    /// The configuration of the application
    pub config: AppConfig,
}

impl AppState {
    /// Create a new `AppState` with the given configuration
    #[must_use] pub fn new(config: AppConfig) -> Self {
        Self {
            initialized: false,
            config,
        }
    }
}

/// The main application struct
pub struct App {
    /// The application state
    state: RwLock<AppState>,
}

impl App {
    /// Create a new App with the given configuration
    #[must_use] pub fn new(config: AppConfig) -> Self {
        Self {
            state: RwLock::new(AppState::new(config)),
        }
    }

    /// Initialize the application
    pub fn initialize(&self) -> Result<(), AppInitializationError> {
        let mut state = self.state.write().unwrap();
        if state.initialized {
            return Err(AppInitializationError::AlreadyInitialized);
        }
        
        // Perform initialization tasks
        state.initialized = true;
        Ok(())
    }

    /// Check if the application is initialized
    pub fn is_initialized(&self) -> bool {
        self.state.read().unwrap().initialized
    }
    
    /// Get the application configuration
    pub fn get_config(&self) -> Result<AppConfig, AppOperationError> {
        let state = self.state.read().unwrap();
        if !state.initialized {
            return Err(AppOperationError::NotInitialized);
        }
        
        Ok(state.config.clone())
    }
}

/// Interface for the application
pub trait AppInterface {
    /// Initialize the application
    fn initialize(&self) -> Result<(), SquirrelError>;
    
    /// Check if the application is initialized
    fn is_initialized(&self) -> bool;
    
    /// Get the application configuration
    fn get_config(&self) -> Result<AppConfig, SquirrelError>;
}

/// Adapter for the App struct
pub struct AppAdapter {
    /// The inner App instance
    app: Arc<App>,
    /// Mutex to ensure thread-safe initialization
    init_mutex: Mutex<()>,
}

impl AppAdapter {
    /// Create a new `AppAdapter`
    #[must_use] pub fn new(config: AppConfig) -> Self {
        Self {
            app: Arc::new(App::new(config)),
            init_mutex: Mutex::new(()),
        }
    }
    
    /// Create a new `AppAdapter` that is already initialized
    pub fn new_initialized(config: AppConfig) -> Result<Self, SquirrelError> {
        let adapter = Self::new(config);
        adapter.initialize()?;
        Ok(adapter)
    }
}

impl AppInterface for AppAdapter {
    fn initialize(&self) -> Result<(), SquirrelError> {
        let _lock = self.init_mutex.lock().unwrap();
        self.app.initialize().map_err(Into::into)
    }
    
    fn is_initialized(&self) -> bool {
        self.app.is_initialized()
    }
    
    fn get_config(&self) -> Result<AppConfig, SquirrelError> {
        self.app.get_config().map_err(Into::into)
    }
}

pub use crate::app::App as Core;

// Remove the duplicate tests module that's causing conflicts

// Import these from the crate root if they exist there
// pub use self::core::Core;
// pub use self::error::{CoreError, Result as CoreResult}; // Rename to avoid conflict
// pub use self::events::EventHandler; // Changed from EventManager
// pub use self::monitoring::MonitoringService; // Changed from MonitoringManager
// pub use self::metrics::Metrics; // Changed from MetricsManager
// pub use self::context::AppContext; // Changed from ContextManager
// pub use self::command::CommandHandler; // Changed from CommandManager 