// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

// This file contains a minimal implementation for testing monitoring integration.

use std::sync::{atomic::{AtomicUsize, Ordering}, Arc, Mutex};
use std::future::Future;

use crate::error::MCPError;
use crate::resilience::health::HealthCheckResult;

/// Monitoring adapter trait for integrating with external monitoring systems
pub trait MonitoringAdapter: std::fmt::Debug + Send + Sync {
    /// Forward health check results to the monitoring system
    fn forward_health_data(&self, component_id: &str, results: Vec<HealthCheckResult>) -> impl Future<Output = Result<(), MCPError>> + Send;
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

impl MonitoringAdapter for TestMonitoringAdapter {
    fn forward_health_data(&self, _component_id: &str, results: Vec<HealthCheckResult>) -> impl Future<Output = Result<(), MCPError>> + Send {
        let forward_count = Arc::clone(&self.forward_count);
        let last_results = Arc::clone(&self.last_results);
        
        async move {
            // Update forward count
            forward_count.fetch_add(1, Ordering::SeqCst);
            
            // Store last results
            let mut last_results = last_results.lock().expect("monitoring results lock poisoned");
            *last_results = results;
            
            Ok(())
        }
    }
} 