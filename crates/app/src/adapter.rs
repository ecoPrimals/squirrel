//! App adapter module
//!
//! This module provides an adapter for the App module following dependency injection patterns.
//! It ensures explicit initialization and proper error handling.

use std::sync::Arc;
use tokio::sync::RwLock;
use squirrel_core::error::{Result, SquirrelError, AppInitializationError, AppOperationError};
use super::Core;
use crate::prelude::AppConfig;
use crate::context::AppContext;
use crate::events::DefaultEventEmitter;

/// Adapter for the App module to support dependency injection
#[derive(Debug)]
pub struct AppAdapter {
    /// The inner app instance
    inner: Option<Arc<RwLock<Core>>>,
    /// Whether the adapter has been initialized
    initialized: bool,
}

impl AppAdapter {
    /// Creates a new AppAdapter
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
    
    /// Initializes the adapter with the given config
    /// 
    /// # Errors
    /// 
    /// Returns an error if the initialization fails
    pub async fn initialize(&mut self, config: AppConfig) -> Result<()> {
        if self.initialized {
            return Err(SquirrelError::AppInitialization(AppInitializationError::AlreadyInitialized));
        }
        
        let app = Core::default();
        self.inner = Some(Arc::new(RwLock::new(app)));
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
    pub async fn ensure_initialized(&self) -> Result<()> {
        if !self.initialized {
            return Err(SquirrelError::AppOperation(AppOperationError::NotInitialized));
        }
        
        let inner = self.inner.as_ref()
            .ok_or_else(|| SquirrelError::AppOperation(AppOperationError::OperationFailure("AppAdapter inner is None".to_string())))?;
            
        if inner.read().await.version().is_empty() {
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
    pub async fn start(&self) -> Result<()> {
        self.ensure_initialized().await?;
        
        let app = self.inner.as_ref()
            .ok_or_else(|| SquirrelError::generic("AppAdapter inner is None".to_string()))?;
            
        app.write().await.start().await.map_err(|e| SquirrelError::generic(e.to_string()))
    }

    /// Stop the application
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The adapter is not initialized
    /// - The App stop operation fails
    pub async fn stop(&self) -> Result<()> {
        self.ensure_initialized().await?;
        
        let app = self.inner.as_ref()
            .ok_or_else(|| SquirrelError::generic("AppAdapter inner is None".to_string()))?;
            
        app.write().await.stop().await.map_err(|e| SquirrelError::generic(e.to_string()))
    }

    /// Get the application context
    /// 
    /// # Errors
    /// 
    /// Returns an error if the context cannot be accessed
    pub async fn context(&self) -> Result<Arc<crate::context::AppContext>> {
        // Implementation placeholder - actual implementation will depend on the state storage
        Err(SquirrelError::generic("Context access not yet implemented".to_string()))
    }

    /// Get the event emitter
    /// 
    /// # Errors
    /// 
    /// Returns an error if the event emitter cannot be accessed
    pub async fn event_emitter(&self) -> Result<Arc<crate::events::DefaultEventEmitter>> {
        // Implementation placeholder - actual implementation will depend on the state storage
        Err(SquirrelError::generic("Event emitter access not yet implemented".to_string()))
    }

    /// Get the app configuration
    ///
    /// # Errors
    ///
    /// Returns an error if the adapter is not initialized
    pub async fn config(&self) -> Result<AppConfig> {
        self.ensure_initialized().await?;
        
        let app = self.inner.as_ref()
            .ok_or_else(|| SquirrelError::generic("AppAdapter inner is None".to_string()))?;
            
        Ok(app.read().await.config.clone())
    }

    /// Get the application version
    /// 
    /// # Errors
    /// 
    /// Returns an error if the adapter is not initialized
    pub async fn version(&self) -> Result<String> {
        self.ensure_initialized().await?;
        
        let app = self.inner.as_ref()
            .ok_or_else(|| SquirrelError::generic("AppAdapter inner is None".to_string()))?;
            
        Ok(app.read().await.version().to_string())
    }
}

/// Create an initialized AppAdapter with the given configuration
///
/// # Errors
///
/// Returns an error if:
/// - The AppAdapter initialization fails
/// - The App creation fails
pub async fn create_initialized_app_adapter(config: AppConfig) -> Result<AppAdapter> {
    let mut adapter = AppAdapter::new();
    adapter.initialize(config).await?;
    Ok(adapter)
}

/// Create an AppAdapter with default configuration
///
/// # Errors
///
/// Returns an error if:
/// - The AppAdapter initialization fails
/// - The App creation fails
pub async fn create_default_app_adapter() -> Result<AppAdapter> {
    create_initialized_app_adapter(AppConfig::default()).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    
    #[tokio::test]
    async fn test_app_adapter_initialization() {
        // ARRANGE
        let mut adapter = AppAdapter::new();
        let config = AppConfig::default();
        
        // ACT
        let result = adapter.initialize(config).await;
        
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
        let _ = adapter.initialize(config.clone()).await;
        
        // ACT
        let result = adapter.initialize(config).await;
        
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
        
        let adapter_result = create_initialized_app_adapter(config).await;
        
        // ASSERT
        assert!(adapter_result.is_ok());
        let adapter = adapter_result.unwrap();
        assert!(adapter.is_initialized());
    }
    
    #[tokio::test]
    async fn test_app_adapter_default_factory() {
        // ARRANGE & ACT
        let adapter_result = create_default_app_adapter().await;
        
        // ASSERT
        assert!(adapter_result.is_ok());
        let adapter = adapter_result.unwrap();
        assert!(adapter.is_initialized());
    }
    
    #[tokio::test]
    async fn test_uninitialized_operations() {
        // ARRANGE
        let adapter = AppAdapter::new();
        
        // ACT & ASSERT
        assert!(!adapter.is_initialized());
        
        let start_result = adapter.start().await;
        assert!(start_result.is_err());
        
        let stop_result = adapter.stop().await;
        assert!(stop_result.is_err());
        
        let context_result = adapter.context().await;
        assert!(context_result.is_err());
        
        let event_emitter_result = adapter.event_emitter().await;
        assert!(event_emitter_result.is_err());
        
        let config_result = adapter.config().await;
        assert!(config_result.is_err());
    }
} 