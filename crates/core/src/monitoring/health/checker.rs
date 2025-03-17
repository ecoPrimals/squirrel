use crate::error::Result;
use crate::monitoring::health::status::HealthStatus;
use crate::monitoring::health::component::ComponentHealth;
use std::fmt::Debug;
use async_trait::async_trait;

/// Health checker interface
#[async_trait]
pub trait HealthChecker: Send + Sync + Debug {
    /// Check the health of all components
    async fn check_health(&self) -> Result<HealthStatus>;

    /// Get health status for a specific component
    async fn get_component_health<'a>(&'a self, component: &'a str) -> Result<Option<ComponentHealth>>;

    /// Start the health checker
    async fn start(&self) -> Result<()>;

    /// Stop the health checker
    async fn stop(&self) -> Result<()>;
} 