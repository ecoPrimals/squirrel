use std::sync::Arc;
use crate::error::Result;
use crate::monitoring::dashboard::{
    DashboardManager, DashboardConfig, Layout, Component, Update, Data
};

/// Adapter for the Dashboard Manager to provide backward compatibility 
/// during the transition to dependency injection.
#[derive(Debug, Clone)]
pub struct DashboardManagerAdapter {
    /// Inner Dashboard Manager instance
    inner: Arc<DashboardManager>,
}

impl DashboardManagerAdapter {
    /// Creates a new adapter with an existing manager
    #[must_use]
    pub const fn with_manager(manager: Arc<DashboardManager>) -> Self {
        Self { inner: manager }
    }

    /// Creates a new adapter that creates a manager with default configuration
    #[must_use]
    pub fn new() -> Self {
        let manager = Arc::new(DashboardManager::default());
        Self { inner: manager }
    }

    /// Get the inner manager
    #[must_use]
    pub fn inner(&self) -> Arc<DashboardManager> {
        self.inner.clone()
    }

    /// Start the dashboard manager
    ///
    /// # Errors
    /// Returns an error if the dashboard manager fails to start
    pub async fn start(&self) -> Result<()> {
        self.inner.start().await
    }

    /// Stop the dashboard manager
    ///
    /// # Errors
    /// Returns an error if the dashboard manager fails to stop
    pub async fn stop(&self) -> Result<()> {
        self.inner.stop().await
    }
}

/// Creates a dashboard manager adapter with default configuration
#[must_use]
pub fn create_dashboard_manager_adapter() -> Arc<DashboardManagerAdapter> {
    Arc::new(DashboardManagerAdapter::new())
}

/// Creates a dashboard manager adapter with an existing manager
#[must_use]
pub fn create_dashboard_manager_adapter_with_manager(
    manager: Arc<DashboardManager>
) -> Arc<DashboardManagerAdapter> {
    Arc::new(DashboardManagerAdapter::with_manager(manager))
} 