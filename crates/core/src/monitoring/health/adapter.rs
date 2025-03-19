use std::sync::Arc;
use crate::error::Result;
use crate::monitoring::health::{
    HealthChecker,
    HealthStatus,
    ComponentHealth,
    DefaultHealthChecker,
    HealthConfig,
};
use async_trait::async_trait;

/// Adapter for the health checker to support dependency injection
#[derive(Debug)]
pub struct HealthCheckerAdapter {
    /// The inner health checker instance
    pub(crate) inner: Option<Arc<DefaultHealthChecker>>,
}

impl HealthCheckerAdapter {
    /// Creates a new health checker adapter without initializing it
    #[must_use] pub fn new() -> Self {
        Self { inner: None }
    }

    /// Creates a new health checker adapter with a specific config
    #[must_use] pub fn new_with_config(config: HealthConfig) -> Self {
        let checker = DefaultHealthChecker::with_dependencies(Some(config));
        Self {
            inner: Some(Arc::new(checker)),
        }
    }

    /// Creates a new health checker adapter with an existing checker
    #[must_use] pub fn with_checker(checker: Arc<DefaultHealthChecker>) -> Self {
        Self {
            inner: Some(checker),
        }
    }

    /// Initializes the adapter with default configuration
    pub fn initialize(&mut self) -> Result<()> {
        if self.inner.is_some() {
            // Already initialized
            return Ok(());
        }

        let checker = DefaultHealthChecker::with_dependencies(None);
        self.inner = Some(Arc::new(checker));
        Ok(())
    }

    /// Initializes the adapter with custom configuration
    pub fn initialize_with_config(&mut self, config: HealthConfig) -> Result<()> {
        if self.inner.is_some() {
            // Already initialized
            return Ok(());
        }

        let checker = DefaultHealthChecker::with_dependencies(Some(config));
        self.inner = Some(Arc::new(checker));
        Ok(())
    }

    /// Checks if the adapter is initialized
    #[must_use] pub fn is_initialized(&self) -> bool {
        self.inner.is_some()
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
        if let Some(checker) = &self.inner {
            checker.register_component(component).await
        } else {
            // Cannot register component if checker is not initialized
            Ok(())
        }
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
        if let Some(checker) = &self.inner {
            checker.get_components().await
        } else {
            Ok(Vec::new())
        }
    }
}

#[async_trait]
impl HealthChecker for HealthCheckerAdapter {
    async fn check_health(&self) -> Result<HealthStatus> {
        if let Some(checker) = &self.inner {
            checker.check_health().await
        } else {
            // Return healthy status when no checker is configured
            Ok(HealthStatus::healthy(
                String::from("system"),
                String::from("No health checker configured"),
            ))
        }
    }

    async fn get_component_health<'a>(&'a self, component: &'a str) -> Result<Option<ComponentHealth>> {
        if let Some(checker) = &self.inner {
            checker.get_component_health(component).await
        } else {
            Ok(None)
        }
    }

    async fn start(&self) -> Result<()> {
        if let Some(checker) = &self.inner {
            checker.start().await
        } else {
            Ok(())
        }
    }

    async fn stop(&self) -> Result<()> {
        if let Some(checker) = &self.inner {
            checker.stop().await
        } else {
            Ok(())
        }
    }
}

impl Default for HealthCheckerAdapter {
    fn default() -> Self {
        Self::new()
    }
}

/// Creates a new health checker adapter
#[must_use] pub fn create_checker_adapter() -> Arc<HealthCheckerAdapter> {
    Arc::new(HealthCheckerAdapter::new())
}

/// Creates a new health checker adapter with default configuration and initializes it
pub fn create_initialized_checker_adapter() -> Result<HealthCheckerAdapter> {
    let mut adapter = HealthCheckerAdapter::new();
    adapter.initialize()?;
    Ok(adapter)
}

/// Creates a new health checker adapter with custom configuration
pub fn create_checker_adapter_with_config(config: HealthConfig) -> Result<HealthCheckerAdapter> {
    let mut adapter = HealthCheckerAdapter::new();
    adapter.initialize_with_config(config)?;
    Ok(adapter)
} 