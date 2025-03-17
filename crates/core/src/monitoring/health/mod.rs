// Health monitoring and status checking functionality
//
// This module provides components for monitoring system health, including:
// - Health status definitions and management
// - Component health checking
// - Health check scheduling and reporting

use std::{
    collections::HashMap,
    sync::Arc,
    fmt::Debug,
};
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use crate::error::Result;
use async_trait::async_trait;

// Allow certain linting issues that are too numerous to fix individually
#[allow(clippy::missing_errors_doc)] // Temporarily allow missing error documentation
#[allow(clippy::unused_async)] // Allow unused async functions
// Define the submodules
/// Health status definitions and reporting
pub mod status;
/// Component health checking and management
pub mod component;
/// Health checker implementations and scheduling
pub mod checker;

// Re-export the types
pub use status::HealthStatus;
pub use component::ComponentHealth;
pub use checker::HealthChecker;

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
}

impl DefaultHealthChecker {
    /// Creates a new instance of the default health checker
    ///
    /// This initializes an empty components map that will be populated
    /// with component health information when components are registered.
    pub fn new() -> Self {
        Self {
            components: Arc::new(RwLock::new(HashMap::new())),
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
            return Ok(HealthStatus::Healthy);
        }

        // Check if any component is unhealthy
        for component in components.values() {
            if component.status == HealthStatus::Unhealthy {
                return Ok(HealthStatus::Unhealthy);
            }
        }

        // Check if any component is degraded
        for component in components.values() {
            if component.status == HealthStatus::Degraded {
                return Ok(HealthStatus::Degraded);
            }
        }

        // All components are healthy
        Ok(HealthStatus::Healthy)
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