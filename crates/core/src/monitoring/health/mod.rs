use std::{
    collections::HashMap,
    sync::Arc,
    fmt::Debug,
};
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use crate::error::Result;
use async_trait::async_trait;

// Define the submodules
pub mod status;
pub mod component;
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
    pub fn new() -> Self {
        Self {
            components: Arc::new(RwLock::new(HashMap::new())),
        }
    }

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

    async fn get_component_health<'a>(&'a self, component: &'a str) -> Result<Option<ComponentHealth>> {
        let components = self.components.read().await;
        Ok(components.get(component).cloned())
    }

    async fn start(&self) -> Result<()> {
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        Ok(())
    }
} 