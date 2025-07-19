// This file contains a minimal implementation for testing monitoring integration.

use std::sync::{atomic::{AtomicUsize, Ordering}, Arc, Mutex};
use async_trait::async_trait;

use crate::error::MCPError;
use crate::resilience::health::HealthCheckResult;

/// Monitoring adapter trait for integrating with external monitoring systems
#[async_trait]
pub trait MonitoringAdapter: std::fmt::Debug + Send + Sync {
    /// Forward health check results to the monitoring system
    async fn forward_health_data(&self, component_id: &str, results: Vec<HealthCheckResult>) -> Result<(), MCPError>;
}

/// Test monitoring adapter for unit tests
///
/// This adapter is used in tests to verify health data forwarding behavior.
/// It tracks the number of times health data has been forwarded and stores
/// the last results that were forwarded.
#[derive(Debug, Default)]
pub struct TestMonitoringAdapter {
    /// Number of times health data has been forwarded
    pub forward_count: Arc<AtomicUsize>,
    /// Last health check results that were forwarded
    pub last_results: Arc<Mutex<Vec<HealthCheckResult>>>,
}

impl TestMonitoringAdapter {
    /// Create a new test monitoring adapter
    pub fn new() -> Self {
        Self {
            forward_count: Arc::new(AtomicUsize::new(0)),
            last_results: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

#[async_trait]
impl MonitoringAdapter for TestMonitoringAdapter {
    async fn forward_health_data(&self, _component_id: &str, results: Vec<HealthCheckResult>) -> Result<(), MCPError> {
        // Update forward count
        self.forward_count.fetch_add(1, Ordering::SeqCst);
        
        // Store last results
        let mut last_results = self.last_results.lock().unwrap();
        *last_results = results;
        
        Ok(())
    }
} 