use std::fmt::Debug;
use async_trait::async_trait;

/// Dashboard service trait for the terminal UI
#[async_trait]
pub trait TerminalDashboardService: Send + Sync + Debug {
    // Minimal requirements for the demo mode
}

/// Default implementation of the dashboard service
#[derive(Debug, Default)]
pub struct DashboardServiceImpl {
    // Implementation not needed for our minimal version
}

impl DashboardServiceImpl {
    /// Create a new dashboard service implementation
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl TerminalDashboardService for DashboardServiceImpl {
    // Empty implementation is sufficient for our demo mode
} 