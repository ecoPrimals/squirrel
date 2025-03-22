use std::sync::Arc;
use squirrel_core::error::Result;
use crate::dashboard::DashboardManager;
use std::sync::RwLock;
use squirrel_core::error::SquirrelError;

/// Adapter for the Dashboard Manager to provide backward compatibility 
/// during the transition to dependency injection.
#[derive(Debug, Clone)]
pub struct DashboardManagerAdapter {
    /// Inner Dashboard Manager instance
    inner: Arc<RwLock<DashboardManager>>,
}

impl Default for DashboardManagerAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl DashboardManagerAdapter {
    /// Creates a new adapter with an existing manager
    #[must_use]
    pub fn with_manager(manager: DashboardManager) -> Self {
        Self { inner: Arc::new(RwLock::new(manager)) }
    }

    /// Creates a new adapter that creates a manager with default configuration
    #[must_use]
    pub fn new() -> Self {
        let manager = DashboardManager::default();
        Self { inner: Arc::new(RwLock::new(manager)) }
    }

    /// Get the inner manager
    #[must_use]
    pub fn inner(&self) -> Arc<RwLock<DashboardManager>> {
        self.inner.clone()
    }

    /// Start the dashboard manager
    ///
    /// # Errors
    /// Returns an error if the dashboard manager fails to start
    pub async fn start(&self) -> Result<()> {
        let lock = self.inner.write();
        let mut manager = lock.map_err(|e| SquirrelError::other(format!("Failed to acquire lock: {}", e)))?;
        manager.start().await
    }

    /// Stop the dashboard manager
    ///
    /// # Errors
    /// Returns an error if the dashboard manager fails to stop
    pub async fn stop(&self) -> Result<()> {
        let lock = self.inner.write();
        let mut manager = lock.map_err(|e| SquirrelError::other(format!("Failed to acquire lock: {}", e)))?;
        manager.stop().await
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
    manager: DashboardManager
) -> Arc<DashboardManagerAdapter> {
    Arc::new(DashboardManagerAdapter::with_manager(manager))
} 