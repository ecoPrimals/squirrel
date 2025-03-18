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
pub use adapter::{HealthCheckerAdapter, create_checker_adapter};

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

/// Default health checker implementation
#[derive(Debug)]
pub struct DefaultHealthChecker {
    components: Arc<RwLock<HashMap<String, ComponentHealth>>>,
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
#[derive(Debug, Clone)]
pub struct HealthCheckerFactory {
    config: HealthConfig,
}

impl HealthCheckerFactory {
    /// Creates a new factory with default configuration
    #[must_use] pub fn new() -> Self {
        Self {
            config: HealthConfig::default(),
        }
    }

    /// Creates a new factory with specific configuration
    #[must_use] pub const fn with_config(config: HealthConfig) -> Self {
        Self { config }
    }

    /// Creates a health checker with dependencies
    #[must_use] pub fn create_checker_with_dependencies(&self) -> Arc<DefaultHealthChecker> {
        Arc::new(DefaultHealthChecker::with_dependencies(Some(self.config.clone())))
    }

    /// Creates a health checker adapter
    #[must_use] pub fn create_checker_adapter(&self) -> Arc<HealthCheckerAdapter> {
        let checker = self.create_checker_with_dependencies();
        Arc::new(HealthCheckerAdapter::with_checker(checker))
    }
}

impl Default for HealthCheckerFactory {
    fn default() -> Self {
        Self::new()
    }
}

/// Initialize the health checker factory
///
/// # Errors
/// Returns an error if the factory is already initialized
pub fn initialize_factory(config: Option<HealthConfig>) -> Result<Arc<HealthCheckerFactory>> {
    let factory = Arc::new(HealthCheckerFactory::with_config(config.unwrap_or_default()));
    Ok(factory)
}

/// Create a health checker adapter
#[must_use] pub fn create_checker_adapter() -> Arc<HealthCheckerAdapter> {
    adapter::create_checker_adapter()
}

/// Global factory for creating health checkers
static FACTORY: OnceLock<HealthCheckerFactory> = OnceLock::new();

/// Initialize the health checker factory
///
/// # Errors
/// Returns an error if the factory is already initialized
pub fn initialize_factory_global(config: Option<HealthConfig>) -> Result<()> {
    let factory = match config {
        Some(cfg) => HealthCheckerFactory::with_config(cfg),
        None => HealthCheckerFactory::new(),
    };
    
    FACTORY.set(factory)
        .map_err(|_| SquirrelError::health("Health checker factory already initialized"))?;
    Ok(())
}

/// Get the health checker factory
#[must_use]
pub fn get_factory() -> Option<HealthCheckerFactory> {
    FACTORY.get().cloned()
}

/// Get or create the health checker factory
#[must_use]
pub fn ensure_factory() -> HealthCheckerFactory {
    FACTORY.get_or_init(HealthCheckerFactory::new).clone()
}

/// Initialize the health checker
///
/// # Errors
/// Returns an error if the checker cannot be initialized
pub async fn initialize() -> Result<Arc<DefaultHealthChecker>> {
    let factory = ensure_factory();
    factory.initialize_global_checker().await
}

/// Get the global health checker
///
/// # Errors
/// Returns an error if the health checker is not initialized
pub async fn get_checker() -> Result<Arc<DefaultHealthChecker>> {
    ensure_factory().get_global_checker().await
}

/// Check health status
///
/// # Errors
/// Returns an error if the health check fails
pub async fn check_health() -> Result<HealthStatus> {
    let checker = get_checker().await?;
    checker.check_health().await
}

/// Initialize the health checker
///
/// # Errors
/// Returns an error if the checker cannot be initialized
pub async fn initialize_global_checker(&self) -> Result<Arc<DefaultHealthChecker>> {
    static GLOBAL_CHECKER: OnceLock<Arc<DefaultHealthChecker>> = OnceLock::new();

    let checker = self.create_checker();
    match GLOBAL_CHECKER.set(checker.clone()) {
        Ok(()) => Ok(checker),
        Err(_) => {
            // Already initialized, return the existing instance
            Ok(GLOBAL_CHECKER.get()
                .ok_or_else(|| SquirrelError::health("Failed to get global health checker"))?
                .clone())
        }
    }
}

/// Gets the global health checker, initializing it if necessary
///
/// # Errors
/// Returns an error if the health checker cannot be initialized
pub async fn get_global_checker(&self) -> Result<Arc<DefaultHealthChecker>> {
    static GLOBAL_CHECKER: OnceLock<Arc<DefaultHealthChecker>> = OnceLock::new();

    if let Some(checker) = GLOBAL_CHECKER.get() {
        return Ok(checker.clone());
    }

    self.initialize_global_checker().await
} 