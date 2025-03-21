//! App adapter module
//!
//! This module provides an adapter for the App module following dependency injection patterns.
//! It ensures explicit initialization and proper error handling.

use std::sync::{Arc, RwLock};
use squirrel_core::error::{Result, SquirrelError, AppOperationError, AppInitializationError};
use crate::core::{Core, AppConfig};
use crate::error::CoreError;

/// Adapter for the App module to support dependency injection
#[derive(Debug)]
pub struct AppAdapter {
    /// The inner app instance
    inner: Option<Arc<RwLock<Core>>>,
    /// Whether the adapter has been initialized
    initialized: bool,
}

impl Default for AppAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl AppAdapter {
    /// Creates a new `AppAdapter`
    /// 
    /// The adapter will be in an uninitialized state and must be
    /// explicitly initialized before use.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: None,
            initialized: false,
        }
    }
    
    /// Initialize the `AppAdapter` with the given configuration.
    /// 
    /// # Errors
    ///
    /// Returns an error if:
    /// - The adapter is already initialized
    /// - The initialization fails
    pub fn initialize(&mut self, _config: &AppConfig) -> Result<()> {
        if self.initialized {
            return Err(SquirrelError::AppInitialization(AppInitializationError::AlreadyInitialized));
        }

        let core = Core::new();
        self.inner = Some(Arc::new(RwLock::new(core)));
        self.initialized = true;
        Ok(())
    }

    /// Check if the adapter is initialized
    #[must_use]
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Ensure the adapter is initialized
    /// 
    /// # Errors
    /// 
    /// Returns an error if the adapter is not initialized
    pub fn ensure_initialized(&self) -> Result<()> {
        if !self.initialized {
            return Err(SquirrelError::AppOperation(AppOperationError::NotInitialized));
        }
        
        let inner = self.inner.as_ref()
            .ok_or(SquirrelError::AppOperation(AppOperationError::NotInitialized))?;
            
        if inner.read().map_err(|_| SquirrelError::generic("Failed to acquire read lock"))?.version().is_empty() {
            return Err(SquirrelError::AppOperation(AppOperationError::OperationFailure("AppAdapter inner is invalid".to_string())));
        }
        
        Ok(())
    }

    /// Start the application
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The adapter is not initialized
    /// - The App start operation fails
    pub fn start(&mut self) -> Result<()> {
        let app = self.inner.as_ref()
            .ok_or(SquirrelError::AppOperation(AppOperationError::NotInitialized))?;
        let mut core = app.write().map_err(|_| SquirrelError::generic("Failed to acquire write lock"))?;
        core.start().map_err(|e| match e {
            CoreError::Config(msg) if msg.contains("App operation error: Application is already stopped") => {
                SquirrelError::AppOperation(AppOperationError::AlreadyStopped)
            }
            _ => SquirrelError::generic(e.to_string())
        })
    }

    /// Stop the application
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The adapter is not initialized
    /// - The App stop operation fails
    pub fn stop(&mut self) -> Result<()> {
        let app = self.inner.as_ref()
            .ok_or(SquirrelError::AppOperation(AppOperationError::NotInitialized))?;
        let mut core = app.write().map_err(|_| SquirrelError::generic("Failed to acquire write lock"))?;
        core.stop().map_err(|e| match e {
            CoreError::Config(msg) if msg.contains("App operation error: Application is already stopped") => {
                SquirrelError::AppOperation(AppOperationError::AlreadyStopped)
            }
            _ => SquirrelError::generic(e.to_string())
        })
    }

    /// Gets the app context
    /// 
    /// # Errors
    /// 
    /// Returns an error if:
    /// - The adapter is not initialized
    /// - The context is not available
    pub fn context(&self) -> Result<Arc<crate::context::AppContext>> {
        // Implementation placeholder - actual implementation will depend on the state storage
        Err(SquirrelError::generic("Context access not yet implemented".to_string()))
    }

    /// Gets the event emitter
    /// 
    /// # Errors
    /// 
    /// Returns an error if:
    /// - The adapter is not initialized
    /// - The event emitter is not available
    pub fn event_emitter(&self) -> Result<Arc<crate::events::DefaultEventEmitter>> {
        // Implementation placeholder - actual implementation will depend on the state storage
        Err(SquirrelError::generic("Event emitter access not yet implemented".to_string()))
    }

    /// Get the app configuration
    ///
    /// # Errors
    ///
    /// Returns an error if the adapter is not initialized
    pub fn config(&self) -> Result<AppConfig> {
        self.ensure_initialized()?;
        
        let app = self.inner.as_ref()
            .ok_or(SquirrelError::AppOperation(AppOperationError::NotInitialized))?;
            
        Ok(app.read().map_err(|_| SquirrelError::generic("Failed to acquire read lock"))?.config.clone())
    }

    /// Get the application version
    /// 
    /// # Errors
    /// 
    /// Returns an error if the adapter is not initialized
    pub fn version(&self) -> Result<String> {
        self.ensure_initialized()?;
        
        let app = self.inner.as_ref()
            .ok_or(SquirrelError::AppOperation(AppOperationError::NotInitialized))?;
            
        Ok(app.read().map_err(|_| SquirrelError::generic("Failed to acquire read lock"))?.version().to_string())
    }
}

/// Create an initialized `AppAdapter` with the given configuration
///
/// # Errors
///
/// Returns an error if:
/// - The `AppAdapter` initialization fails
/// - The App creation fails
pub fn create_initialized_app_adapter(config: &AppConfig) -> Result<AppAdapter> {
    let mut adapter = AppAdapter::new();
    adapter.initialize(config)?;
    Ok(adapter)
}

/// Create an `AppAdapter` with default configuration
///
/// # Errors
///
/// Returns an error if:
/// - The `AppAdapter` initialization fails
/// - The App creation fails
pub fn create_default_app_adapter() -> Result<AppAdapter> {
    create_initialized_app_adapter(&AppConfig::default())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_app_adapter_initialization() {
        // ARRANGE
        let mut adapter = AppAdapter::new();
        let config = AppConfig::default();
        
        // ACT
        let result = adapter.initialize(&config);
        
        // ASSERT
        assert!(result.is_ok());
        assert!(adapter.is_initialized());
    }
    
    #[tokio::test]
    async fn test_app_adapter_double_initialization() {
        // ARRANGE
        let mut adapter = AppAdapter::new();
        let config = AppConfig::default();
        
        // First initialization should succeed
        let _ = adapter.initialize(&config.clone());
        
        // ACT
        let result = adapter.initialize(&config);
        
        // ASSERT
        assert!(result.is_err());
        if let Err(SquirrelError::AppInitialization(AppInitializationError::AlreadyInitialized)) = result {
            // No need to assert anything here, the error type is correct
        } else {
            panic!("Expected SquirrelError::AppInitialization(AppInitializationError::AlreadyInitialized)");
        }
    }
    
    #[tokio::test]
    async fn test_app_adapter_factory() {
        // ARRANGE & ACT
        let config = AppConfig {
            name: "TestApp".to_string(),
            version: "0.1.0".to_string(),
            environment: "test".to_string(),
            debug: true,
        };
        
        let adapter_result = create_initialized_app_adapter(&config);
        
        // ASSERT
        assert!(adapter_result.is_ok());
        let adapter = adapter_result.unwrap();
        assert!(adapter.is_initialized());
    }
    
    #[tokio::test]
    async fn test_app_adapter_default_factory() {
        // ARRANGE & ACT
        let adapter_result = create_default_app_adapter();
        
        // ASSERT
        assert!(adapter_result.is_ok());
        let adapter = adapter_result.unwrap();
        assert!(adapter.is_initialized());
    }
    
    #[tokio::test]
    async fn test_uninitialized_operations() {
        // ARRANGE
        let mut adapter = AppAdapter::new();
        
        // ACT & ASSERT
        assert!(!adapter.is_initialized());
        
        let start_result = adapter.start();
        assert!(start_result.is_err());
        
        let stop_result = adapter.stop();
        assert!(stop_result.is_err());
        
        let context_result = adapter.context();
        assert!(context_result.is_err());
        
        let event_emitter_result = adapter.event_emitter();
        assert!(event_emitter_result.is_err());
    }

    #[tokio::test]
    async fn test_stop_not_started() {
        let mut adapter = AppAdapter::new();
        let config = AppConfig::default();
        assert!(adapter.initialize(&config).is_ok());
        
        let start_result = adapter.start();
        assert!(start_result.is_ok());
        
        let stop_result = adapter.stop();
        assert!(stop_result.is_ok());
        
        let context_result = adapter.context();
        assert!(context_result.is_err());
        
        let event_emitter_result = adapter.event_emitter();
        assert!(event_emitter_result.is_err());
    }
} 