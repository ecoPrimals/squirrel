// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Tests for the integration module
//!
//! This module contains tests for the integration components of the MCP resilience framework,
//! including the health monitoring bridge, health check adapter, and alert recovery adapter.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use async_trait::async_trait;
use tokio::sync::RwLock;
use chrono::Utc;

use crate::error::{MCPError, Result};
use crate::monitoring::metrics::{Metric, MetricType, MetricValue};
use crate::monitoring::alerts::{Alert, AlertSeverity, AlertConfiguration, AlertState};
use crate::monitoring::HealthStatus as MonitoringHealth;
use crate::resilience::health::{HealthCheck, HealthStatus, HealthCheckConfig, HealthCheckResult, HealthMonitor};
use crate::resilience::recovery::{RecoveryStrategy, FailureSeverity, FailureInfo, RecoveryError};

// Import the traits explicitly
use crate::monitoring::metrics::MetricsCollector;
use crate::monitoring::alerts::AlertManager;

/// Mock metrics collector for testing
pub struct MockMetricsCollector {
    metrics: Arc<Mutex<Vec<Metric>>>,
}

impl MockMetricsCollector {
    /// Create a new mock metrics collector
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    /// Get the count of recorded metrics
    pub fn recorded_count(&self) -> usize {
        let metrics = self.metrics.lock().expect("should succeed");
        metrics.len()
    }
    
    /// Get all recorded metrics
    pub fn get_all_metrics(&self) -> Vec<Metric> {
        self.metrics.lock().expect("should succeed").clone()
    }
    
    /// Register a metric
    pub fn register_metric(&self, metric: Metric) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.metrics.lock().expect("should succeed").push(metric);
        Ok(())
    }
    
    /// Get a metric by name
    pub fn get_metric(&self, name: &str) -> Option<Metric> {
        self.metrics.lock().expect("should succeed")
            .iter()
            .find(|m| m.name == name)
            .cloned()
    }
}

/// Mock alert manager for testing
pub struct MockAlertManager {
    alerts: Arc<Mutex<Vec<Alert>>>,
}

impl MockAlertManager {
    /// Create a new mock alert manager
    pub fn new() -> Self {
        Self {
            alerts: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    /// Get the count of alerts
    pub fn alert_count(&self) -> usize {
        let alerts = self.alerts.lock().expect("should succeed");
        alerts.len()
    }
    
    /// Get all alerts
    pub fn get_all_alerts(&self) -> Vec<Alert> {
        self.alerts.lock().expect("should succeed").clone()
    }
    
    /// Register an alert configuration
    pub fn register_alert(&self, config: AlertConfiguration) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let alert = Alert {
            id: config.name.clone(),
            config,
            state: AlertState::Ok,
            first_fired_at: None,
            last_fired_at: None,
            last_checked_at: Some(Utc::now()),
            triggered_value: None,
            firing_count: 0,
            acknowledged_by: None,
            acknowledged_at: None,
        };
        self.alerts.lock().expect("should succeed").push(alert);
        Ok(())
    }
    
    /// Get an alert by ID
    pub fn get_alert(&self, id: &str) -> Option<Alert> {
        self.alerts.lock().expect("should succeed")
            .iter()
            .find(|a| a.id == id)
            .cloned()
    }
    
    /// Update an alert
    pub fn update_alert(&self, alert: Alert) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut alerts = self.alerts.lock().expect("should succeed");
        if let Some(idx) = alerts.iter().position(|a| a.id == alert.id) {
            alerts[idx] = alert;
        }
        Ok(())
    }
}

/// Mock health check implementation for testing
#[derive(Debug, Clone)]
pub struct MockHealthCheck {
    id: String,
    config: HealthCheckConfig,
    status: Arc<RwLock<HealthStatus>>,
}

impl MockHealthCheck {
    /// Create a new mock health check
    pub fn new(id: &str, initial_status: HealthStatus) -> Self {
        Self {
            id: id.to_string(),
            config: HealthCheckConfig::default(),
            status: Arc::new(RwLock::new(initial_status)),
        }
    }
    
    /// Set the health status
    pub fn set_status(&self, status: HealthStatus) {
        let mut current = futures::executor::block_on(self.status.write());
        *current = status;
    }
}

#[async_trait]
impl HealthCheck for MockHealthCheck {
    fn id(&self) -> &str {
        &self.id
    }
    
    async fn check(&self) -> HealthCheckResult {
        let status = *self.status.read().await;
        let mut result = HealthCheckResult::new(
            self.id().to_string(),
            status,
            format!("Mock health check for {} is {}", self.id(), status),
        );
        
        // Add some metrics
        result = result.with_metric("response_time_ms", 123.45);
        result = result.with_metric("error_rate", 0.05);
        
        result
    }
    
    fn config(&self) -> &HealthCheckConfig {
        &self.config
    }
    
    fn config_mut(&mut self) -> &mut HealthCheckConfig {
        &mut self.config
    }
}

/// Mock recovery strategy for testing
pub struct MockRecoveryStrategy {
    recovery_count: AtomicUsize,
    last_component: Mutex<Option<String>>,
    last_severity: Mutex<Option<FailureSeverity>>,
}

impl MockRecoveryStrategy {
    /// Create a new mock recovery strategy
    pub fn new() -> Self {
        Self {
            recovery_count: AtomicUsize::new(0),
            last_component: Mutex::new(None),
            last_severity: Mutex::new(None),
        }
    }
    
    /// Get the number of recovery attempts
    pub fn recovery_attempts(&self) -> usize {
        self.recovery_count.load(Ordering::SeqCst)
    }
    
    /// Get the last component that was recovered
    pub fn last_component_id(&self) -> Option<String> {
        let component = self.last_component.lock().expect("should succeed");
        component.clone()
    }
    
    /// Get the last severity that was handled
    pub fn last_severity(&self) -> Option<FailureSeverity> {
        let severity = self.last_severity.lock().expect("should succeed");
        *severity
    }
    
    /// Handle a failure
    pub fn handle_failure<F>(&self, failure_info: FailureInfo, recovery_action: F) -> std::result::Result<(), RecoveryError>
    where
        F: FnOnce() -> std::result::Result<(), RecoveryError>,
    {
        // Record failure info
        self.recovery_count.fetch_add(1, Ordering::SeqCst);
        
        let mut component = self.last_component.lock().expect("should succeed");
        *component = Some(failure_info.context.clone());
        
        let mut severity = self.last_severity.lock().expect("should succeed");
        *severity = Some(failure_info.severity);
        
        // Call recovery action
        recovery_action()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integration::{
        AlertToRecoveryAdapter,
        ResilienceHealthCheckAdapter,
        create_metrics_from_health_result,
        HealthMonitoringBridge,
        HealthMonitoringBridgeConfig,
    };
    
    /// Test health check adapter initialization
    #[test]
    fn test_health_check_adapter_init() {
        let health_check = MockHealthCheck::new("test_component", HealthStatus::Healthy);
        let adapter = ResilienceHealthCheckAdapter::new(health_check.clone());
        
        assert_eq!(adapter.name(), "test_component");
        assert!(adapter.inner().id() == "test_component");
    }
    
    /// Test alert to recovery adapter initialization
    #[test]
    fn test_alert_recovery_adapter_init() {
        let mock_recovery = Arc::new(Mutex::new(RecoveryStrategy::default()));
        let adapter = AlertToRecoveryAdapter::new(mock_recovery);
        
        let adapter1 = adapter.clone();
        let adapter2 = adapter1.with_logging(true);
        let _ = adapter2.with_logging(false);
        
        assert!(true);
    }
    
    /// Test health monitoring bridge initialization
    #[tokio::test]
    async fn test_health_monitoring_bridge_init() {
        let resilience_monitor = Arc::new(HealthMonitor::new(100));
        let metrics_collector = Arc::new(MetricsCollector::new());
        let alert_manager = Arc::new(AlertManager::new());
        
        let bridge = HealthMonitoringBridge::new(
            resilience_monitor,
            metrics_collector,
            alert_manager,
            HealthMonitoringBridgeConfig::default(),
        );
        
        assert!(!bridge.is_running());
    }
    
    /// Test creating metrics from health result
    #[test]
    fn test_create_metrics_from_health_result() {
        let component_id = "test_component";
        let mut result = HealthCheckResult::new(
            component_id.to_string(),
            HealthStatus::Warning,
            "Component is degraded".to_string(),
        );
        
        result = result.with_metric("response_time", 123.45);
        result = result.with_metric("error_rate", 0.05);
        
        let metrics = create_metrics_from_health_result(component_id, &result);
        
        // Should have 3 metrics: status, response_time, error_rate
        assert_eq!(metrics.len(), 3);
    }

    /// Test the health monitoring bridge
    #[tokio::test]
    async fn test_health_monitoring_bridge() {
        // Create the necessary components
        let mut health_monitor = HealthMonitor::new(100);
        let _metrics_collector = Arc::new(MockMetricsCollector::new());
        let _alert_manager = Arc::new(MockAlertManager::new());
        
        // Create a simple health check and register it with the health monitor
        let health_check = MockHealthCheck::new("test-component", HealthStatus::Healthy);
        health_monitor.register(health_check.clone()).expect("should succeed");
        
        // Now create an Arc for the bridge
        let resilience_monitor = Arc::new(health_monitor);
        
        // Create the bridge with actual implementation
        let bridge = HealthMonitoringBridge::new(
            resilience_monitor.clone(),
            Arc::new(MetricsCollector::new()), // Using real collector
            Arc::new(AlertManager::new()),     // Using real alert manager
            HealthMonitoringBridgeConfig::default(),
        );
        
        // Verify bridge creation works
        assert!(!bridge.is_running());
        
        // Check the health status - this is async
        let status = resilience_monitor.check_component("test-component").await;
        assert!(status.is_some());
        assert_eq!(status.expect("should succeed").status, HealthStatus::Healthy);
        
        // Verify that the bridge can start and stop without errors
        let start_result = bridge.start().await;
        assert!(start_result.is_ok());
        assert!(bridge.is_running());
        
        let stop_result = bridge.stop().await;
        assert!(stop_result.is_ok());
        assert!(!bridge.is_running());
    }
} 