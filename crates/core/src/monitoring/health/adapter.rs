use std::sync::Arc;
use crate::error::Result;
use crate::monitoring::health::{
    HealthChecker,
    HealthStatus,
    ComponentHealth,
    DefaultHealthChecker,
};
use async_trait::async_trait;

/// Adapter for the health checker to support dependency injection
#[derive(Debug)]
pub struct HealthCheckerAdapter {
    inner: Option<Arc<DefaultHealthChecker>>,
}

impl HealthCheckerAdapter {
    /// Creates a new health checker adapter
    #[must_use] pub fn new() -> Self {
        Self { inner: None }
    }

    /// Creates a new health checker adapter with an existing checker
    #[must_use] pub fn with_checker(checker: Arc<DefaultHealthChecker>) -> Self {
        Self {
            inner: Some(checker),
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