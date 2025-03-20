// Health monitoring and status checking functionality
//
// This module provides components for monitoring system health, including:
// - Health status definitions and management
// - Component health checking
// - Health check scheduling and reporting

use std::{
    collections::HashMap,
    sync::{Arc, OnceLock},
    fmt::Debug,
};
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use crate::error::{Result, SquirrelError};
use async_trait::async_trait;
use self::status::Status;
use thiserror::Error;

// Define error types
/// Health check related errors
#[derive(Debug, Error)]
pub enum HealthCheckError {
    /// Health checker not initialized
    #[error("Health checker not initialized")]
    NotInitialized,

    /// Health checker already initialized
    #[error("Health checker already initialized")]
    AlreadyInitialized,

    /// Component error
    #[error("Component error: {0}")]
    ComponentError(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// General health check error
    #[error("Health check error: {0}")]
    General(String),
}

impl From<HealthCheckError> for SquirrelError {
    fn from(err: HealthCheckError) -> Self {
        SquirrelError::health(err.to_string())
    }
}

// Define the submodules
/// Health status definitions and reporting
pub mod status;
/// Component health checking and management
pub mod component;
/// Health checker implementations and scheduling
pub mod checker;
/// Health checker adapter for dependency injection
pub mod adapter;

// Re-export the types
pub use status::HealthStatus;
pub use component::ComponentHealth;
pub use checker::HealthChecker;
pub use adapter::HealthCheckerAdapter;

/// Health configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthConfig {
    /// Enable health checks
    pub enabled: bool,
    /// Health check interval in seconds
    pub interval: u64,
}

impl Default for HealthConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval: 60,
        }
    }
}

/// Default implementation of the health checker
#[derive(Debug)]
pub struct DefaultHealthChecker {
    /// Map of component health status records
    components: Arc<RwLock<HashMap<String, ComponentHealth>>>,
    /// Health checker configuration
    #[allow(dead_code)]
    config: HealthConfig,
}

impl DefaultHealthChecker {
    /// Creates a new instance of the default health checker
    ///
    /// This initializes an empty components map that will be populated
    /// with component health information when components are registered.
    #[must_use] pub fn new() -> Self {
        Self {
            components: Arc::new(RwLock::new(HashMap::new())),
            config: HealthConfig::default(),
        }
    }

    /// Creates a new instance with dependencies
    ///
    /// This constructor allows for dependency injection of required components
    #[must_use] pub fn with_dependencies(config: Option<HealthConfig>) -> Self {
        Self {
            components: Arc::new(RwLock::new(HashMap::new())),
            config: config.unwrap_or_default(),
        }
    }

    /// Register a component for health monitoring
    ///
    /// Adds a new component to the health monitoring system.
    /// If a component with the same name already exists, it will be replaced.
    ///
    /// # Arguments
    /// * `component` - The component health information to register
    ///
    /// # Errors
    /// This function will not produce errors, but returns a Result type for consistency
    /// with the async interface.
    pub async fn register_component(&self, component: ComponentHealth) -> Result<()> {
        let mut components = self.components.write().await;
        components.insert(component.name.clone(), component);
        Ok(())
    }

    /// Get all registered components
    ///
    /// Retrieves all components currently registered with the health checker
    ///
    /// # Returns
    /// A vector of component health information
    ///
    /// # Errors
    /// This function will not produce errors, but returns a Result type for consistency
    pub async fn get_components(&self) -> Result<Vec<ComponentHealth>> {
        let components = self.components.read().await;
        Ok(components.values().cloned().collect())
    }
}

impl Default for DefaultHealthChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl HealthChecker for DefaultHealthChecker {
    /// Check the overall health status of all registered components
    ///
    /// Determines the overall health status based on the status of individual components:
    /// - If any component is unhealthy, the overall status is unhealthy
    /// - If any component is degraded, the overall status is degraded
    /// - Otherwise, the overall status is healthy
    /// - If no components are registered, the status is healthy
    ///
    /// # Errors
    /// This function will not produce errors, but returns a Result type for consistency
    /// with the health checker interface.
    async fn check_health(&self) -> Result<HealthStatus> {
        let components = self.components.read().await;

        // If no components registered, return healthy
        if components.is_empty() {
            return Ok(HealthStatus::healthy(String::from("system"), String::from("All systems operational")));
        }

        // Check if any component is unhealthy
        for component in components.values() {
            if component.status == Status::Unhealthy {
                return Ok(HealthStatus::unhealthy(String::from("system"), String::from("One or more components are unhealthy")));
            }
        }

        // Check if any component is degraded
        for component in components.values() {
            if component.status == Status::Degraded {
                return Ok(HealthStatus::degraded(String::from("system"), String::from("One or more components are degraded")));
            }
        }

        // All components are healthy
        Ok(HealthStatus::healthy(String::from("system"), String::from("All systems operational")))
    }

    /// Retrieves the health status of a specific component
    ///
    /// # Arguments
    /// * `component` - The name of the component to check
    ///
    /// # Returns
    /// * `Ok(Some(ComponentHealth))` - The health status of the specified component
    /// * `Ok(None)` - The component is not registered with the health checker
    ///
    /// # Errors
    /// This function will not produce errors, but returns a Result type for consistency
    /// with the health checker interface.
    async fn get_component_health<'a>(&'a self, component: &'a str) -> Result<Option<ComponentHealth>> {
        let components = self.components.read().await;
        Ok(components.get(component).cloned())
    }

    /// Starts the health checker service
    ///
    /// # Errors
    /// This function will not produce errors, but returns a Result type for consistency
    /// with the health checker interface and future extensibility.
    async fn start(&self) -> Result<()> {
        Ok(())
    }

    /// Stops the health checker service
    ///
    /// # Errors
    /// This function will not produce errors, but returns a Result type for consistency
    /// with the health checker interface and future extensibility.
    async fn stop(&self) -> Result<()> {
        Ok(())
    }
}

/// Factory for creating health checkers with dependency injection
///
/// This factory provides a centralized way to create and configure health checker
/// instances with appropriate dependency injection. It supports creating both
/// standalone checkers and globally shared instances.
#[derive(Debug, Clone)]
pub struct HealthCheckerFactory {
    /// Configuration for creating health checkers
    config: HealthConfig,
}

impl HealthCheckerFactory {
    /// Creates a new health checker factory with default config
    /// 
    /// Initializes a factory with default health checking configuration
    /// settings. This is suitable for most standard monitoring scenarios.
    ///
    /// # Returns
    /// A new factory instance with default configuration
    #[must_use] pub fn new() -> Self {
        Self {
            config: HealthConfig::default(),
        }
    }
    
    /// Creates a new health checker factory with custom config
    /// 
    /// Initializes a factory with custom health checking configuration
    /// settings, allowing for tailored monitoring behavior.
    ///
    /// # Arguments
    /// * `config` - Custom health checker configuration
    ///
    /// # Returns
    /// A new factory instance with the specified configuration
    #[must_use] pub fn with_config(config: HealthConfig) -> Self {
        Self {
            config,
        }
    }

    /// Creates a health checker adapter
    /// 
    /// Creates an uninitialized health checker adapter. The adapter must be
    /// initialized before use by setting a concrete checker implementation.
    ///
    /// # Returns
    /// An uninitialized health checker adapter
    #[must_use] pub fn create_checker_adapter(&self) -> HealthCheckerAdapter {
        adapter::HealthCheckerAdapter::new()
    }
    
    /// Creates and initializes a health checker adapter
    /// 
    /// Creates a health checker adapter with a default implementation
    /// that is ready for immediate use.
    ///
    /// # Returns
    /// An initialized health checker adapter
    ///
    /// # Errors
    /// Returns an error if the adapter initialization fails
    pub fn create_initialized_checker(&self) -> Result<HealthCheckerAdapter> {
        let checker = Arc::new(DefaultHealthChecker::with_dependencies(Some(self.config.clone())));
        
        // Initialize the adapter with the created checker
        let adapter = adapter::HealthCheckerAdapter::with_checker(checker);
        
        Ok(adapter)
    }

    /// Creates a health checker with the specified config
    /// 
    /// Creates a health checker adapter with a custom configuration
    /// that is ready for immediate use.
    ///
    /// # Arguments
    /// * `config` - Custom health checker configuration
    ///
    /// # Returns
    /// An initialized health checker adapter with the specified configuration
    ///
    /// # Errors
    /// Returns an error if the adapter initialization fails
    pub fn create_checker_with_config(&self, config: HealthConfig) -> Result<HealthCheckerAdapter> {
        let checker = Arc::new(DefaultHealthChecker::with_dependencies(Some(config)));
        
        // Initialize the adapter with the created checker
        let adapter = adapter::HealthCheckerAdapter::with_checker(checker);
        
        Ok(adapter)
    }
    
    /// Initializes the global health checker
    /// 
    /// Creates and initializes a globally shared health checker instance
    /// that can be accessed from anywhere in the application.
    ///
    /// # Returns
    /// A reference to the global health checker adapter
    ///
    /// # Errors
    /// Returns an error if the global checker cannot be initialized
    pub async fn initialize_global_checker(&self) -> Result<Arc<HealthCheckerAdapter>> {
        initialize_global_checker().await?;
        self.get_global_checker().await
    }
    
    /// Gets the global health checker instance
    /// 
    /// Retrieves the globally shared health checker instance if it exists.
    ///
    /// # Returns
    /// A reference to the global health checker adapter
    ///
    /// # Errors
    /// Returns an error if the global checker is not initialized
    pub async fn get_global_checker(&self) -> Result<Arc<HealthCheckerAdapter>> {
        get_global_checker().await.map(|checker| Arc::new(HealthCheckerAdapter::with_checker(checker)))
    }
}

impl Default for HealthCheckerFactory {
    fn default() -> Self {
        Self::new()
    }
}

/// Creates an initialized health checker adapter with optional configuration
/// 
/// This is a convenience function that creates a new health checker adapter
/// with the provided configuration and initializes it.
///
/// # Arguments
/// * `config` - Optional configuration for the health checker
///
/// # Returns
/// An initialized health checker adapter
///
/// # Errors
/// Returns an error if the adapter initialization fails
pub fn create_initialized_checker_adapter(config: Option<HealthConfig>) -> Result<HealthCheckerAdapter> {
    let checker = Arc::new(DefaultHealthChecker::with_dependencies(config));
    let adapter = adapter::HealthCheckerAdapter::with_checker(checker);
    Ok(adapter)
}

/// Creates a health checker adapter with specific configuration
/// 
/// This is a convenience function that creates a new health checker adapter
/// with the specified configuration and initializes it.
///
/// # Arguments
/// * `config` - Required configuration for the health checker
///
/// # Returns
/// An initialized health checker adapter
///
/// # Errors
/// Returns an error if the adapter initialization fails
pub fn create_checker_adapter_with_config(config: HealthConfig) -> Result<HealthCheckerAdapter> {
    create_initialized_checker_adapter(Some(config))
}

/// Creates a new uninitialized health checker adapter
/// 
/// Creates a new adapter without an underlying health checker implementation.
/// The adapter will need to be initialized before it can perform health checks.
///
/// # Returns
/// An uninitialized health checker adapter wrapped in an Arc
#[must_use] pub fn create_checker_adapter() -> Arc<HealthCheckerAdapter> {
    Arc::new(adapter::HealthCheckerAdapter::new())
}

/// Initializes the health checker factory with global state
/// 
/// Sets up a global health checker factory with the specified configuration
/// that can be accessed from anywhere in the application.
///
/// # Arguments
/// * `config` - Optional configuration for the factory
///
/// # Returns
/// Success if the initialization was completed
///
/// # Errors
/// Returns an error if the global factory could not be initialized
pub fn initialize_factory_global(config: Option<HealthConfig>) -> Result<()> {
    static FACTORY: OnceLock<HealthCheckerFactory> = OnceLock::new();
    
    let factory = match config {
        Some(cfg) => HealthCheckerFactory::with_config(cfg),
        None => HealthCheckerFactory::new(),
    };
    
    let _ = FACTORY.set(factory);
    Ok(())
}

/// Gets the global health checker factory
/// 
/// Retrieves the globally shared health checker factory if it has been initialized.
///
/// # Returns
/// Some(HealthCheckerFactory) if the factory exists, None otherwise
pub fn get_factory() -> Option<HealthCheckerFactory> {
    static FACTORY: OnceLock<HealthCheckerFactory> = OnceLock::new();
    FACTORY.get().cloned()
}

/// Gets or creates the global health checker factory
/// 
/// Retrieves the globally shared health checker factory or creates a new one
/// with default configuration if it doesn't exist.
///
/// # Returns
/// A health checker factory instance
#[must_use] pub fn ensure_factory() -> HealthCheckerFactory {
    get_factory().unwrap_or_default()
}

/// Initializes the global health checker implementation
/// 
/// Creates and initializes a global default health checker that can be accessed
/// from anywhere in the application.
///
/// # Returns
/// A reference to the initialized health checker
///
/// # Errors
/// Returns an error if the health checker could not be initialized
pub async fn initialize() -> Result<Arc<DefaultHealthChecker>> {
    static CHECKER: OnceLock<Arc<DefaultHealthChecker>> = OnceLock::new();
    
    // If already initialized, return the existing checker
    if let Some(checker) = CHECKER.get() {
        return Ok(checker.clone());
    }
    
    // Create a new checker
    let checker = Arc::new(DefaultHealthChecker::new());
    
    // Attempt to store it in the global state
    let _ = CHECKER.set(checker.clone());
    
    Ok(checker)
}

/// Gets the global health checker
/// 
/// Retrieves the globally shared health checker if it has been initialized.
///
/// # Returns
/// A reference to the global health checker if initialized
///
/// # Errors
/// Returns an error if the global health checker is not initialized
pub async fn get_checker() -> Result<Arc<DefaultHealthChecker>> {
    static CHECKER: OnceLock<Arc<DefaultHealthChecker>> = OnceLock::new();
    
    // If already initialized, return the existing checker
    if let Some(checker) = CHECKER.get() {
        return Ok(checker.clone());
    }
    
    // Not initialized, return error
    Err(HealthCheckError::NotInitialized.into())
}

/// Performs a health check using the global health checker
/// 
/// Executes a health check on all registered components using the global
/// health checker instance.
///
/// # Returns
/// The overall health status of the system
///
/// # Errors
/// Returns an error if the global health checker is not initialized
/// or if the health check fails
pub async fn check_health() -> Result<HealthStatus> {
    let checker = get_checker().await?;
    checker.check_health().await
}

/// Initializes the global health checker with optional configuration
/// 
/// Creates and initializes a global health checker with the specified configuration
/// that can be accessed from anywhere in the application.
///
/// # Returns
/// A reference to the initialized health checker
///
/// # Errors
/// Returns an error if the health checker could not be initialized
pub async fn initialize_global_checker() -> Result<Arc<DefaultHealthChecker>> {
    static CHECKER: OnceLock<Arc<DefaultHealthChecker>> = OnceLock::new();
    
    // If already initialized, return the existing checker
    if let Some(checker) = CHECKER.get() {
        return Ok(checker.clone());
    }
    
    // Create a new checker with default config
    let checker = Arc::new(DefaultHealthChecker::new());
    
    // Attempt to store it in the global state
    if CHECKER.set(checker.clone()).is_err() {
        // Already initialized by another thread, get the existing one
        if let Some(existing) = CHECKER.get() {
            return Ok(existing.clone());
        }
    }
    
    Ok(checker)
}

/// Gets the global health checker
/// 
/// Retrieves the globally shared health checker if it has been initialized.
///
/// # Returns
/// A reference to the global health checker if initialized
///
/// # Errors
/// Returns an error if the global health checker is not initialized
pub async fn get_global_checker() -> Result<Arc<DefaultHealthChecker>> {
    static CHECKER: OnceLock<Arc<DefaultHealthChecker>> = OnceLock::new();
    
    // If initialized, return the existing checker
    if let Some(checker) = CHECKER.get() {
        return Ok(checker.clone());
    }
    
    // Not initialized, return error
    Err(HealthCheckError::NotInitialized.into())
} 