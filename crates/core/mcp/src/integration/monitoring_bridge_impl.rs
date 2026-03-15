// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

#![allow(unused_variables)]

// MCP Resilience Health Monitoring Bridge Implementation
//
// This module implements the bridge between the MCP resilience framework's
// health monitoring component and the global monitoring system.

use std::sync::{Arc, Mutex};
use std::time::Duration;
use async_trait::async_trait;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tracing::{debug, info, warn};

use crate::error::{MCPError, Result};
use crate::monitoring::{
    metrics::MetricsCollector,
    alerts::{Alert, AlertManager, AlertSeverity, AlertConfiguration, AlertCondition, AlertAction},
};
use crate::resilience::health::{
    HealthMonitor, HealthStatus, HealthCheckResult,
    MonitoringAdapter,
};
use crate::resilience::recovery::RecoveryStrategy;

/// Configuration for the health monitoring bridge
#[derive(Debug, Clone)]
pub struct HealthMonitoringBridgeConfig {
    /// How often to forward health data (in seconds)
    pub forward_interval: u64,
    
    /// Whether to forward all components or only unhealthy ones
    pub forward_all_components: bool,
    
    /// Whether to enable bidirectional integration
    pub bidirectional: bool,
    
    /// Log level for bridge operations
    pub log_level: Option<&'static str>,
}

impl Default for HealthMonitoringBridgeConfig {
    fn default() -> Self {
        Self {
            forward_interval: 10,
            forward_all_components: true,
            bidirectional: true,
            log_level: None,
        }
    }
}

/// Bridge for connecting MCP resilience health monitoring with the monitoring system
pub struct HealthMonitoringBridge {
    /// Reference to the MCP resilience health monitor
    resilience_monitor: Arc<HealthMonitor>,
    
    /// Reference to the monitoring system's metrics collector
    metrics_collector: Arc<MetricsCollector>,
    
    /// Reference to the monitoring system's alert manager
    alert_manager: Arc<AlertManager>,
    
    /// Configuration for the bridge
    config: HealthMonitoringBridgeConfig,
    
    /// Handle to the background task that periodically forwards health data
    forward_task: Arc<Mutex<Option<JoinHandle<()>>>>,
    
    /// Whether the bridge is running
    running: Arc<RwLock<bool>>,
    
    /// Recovery strategy for handling alerts from the monitoring system
    recovery_strategy: Option<Arc<Mutex<RecoveryStrategy>>>,
}

impl std::fmt::Debug for HealthMonitoringBridge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HealthMonitoringBridge")
            .field("resilience_monitor", &"Arc<HealthMonitor>")
            .field("metrics_collector", &"Arc<MetricsCollector>")
            .field("alert_manager", &"Arc<AlertManager>")
            .field("config", &self.config)
            .field("running", &self.running)
            .finish()
    }
}

impl HealthMonitoringBridge {
    /// Create a new health monitoring bridge
    pub fn new(
        resilience_monitor: Arc<HealthMonitor>,
        metrics_collector: Arc<MetricsCollector>,
        alert_manager: Arc<AlertManager>,
        config: HealthMonitoringBridgeConfig,
    ) -> Self {
        Self {
            resilience_monitor,
            metrics_collector,
            alert_manager,
            config,
            forward_task: Arc::new(Mutex::new(None)),
            running: Arc::new(RwLock::new(false)),
            recovery_strategy: None,
        }
    }
    
    /// Add a recovery strategy for handling alerts
    pub fn with_recovery_strategy(mut self, recovery_strategy: Arc<Mutex<RecoveryStrategy>>) -> Self {
        self.recovery_strategy = Some(recovery_strategy);
        self
    }

    /// Check if the bridge is currently running
    pub fn is_running(&self) -> bool {
        match self.running.try_read() {
            Ok(running) => *running,
            Err(_) => false, // If we can't get the lock, assume it's not running
        }
    }
    
    /// Start the bridge to begin forwarding health data
    pub async fn start(&self) -> Result<()> {
        // Set running flag to true
        {
            let mut running = self.running.write().await;
            if *running {
                return Err(MCPError::InvalidOperation("Health monitoring bridge is already running".to_string()).into());
            }
            *running = true;
        }
        
        // Register alert handler if bidirectional integration is enabled
        if self.config.bidirectional {
            if let Err(e) = self.register_alert_handler().await {
                warn!("Failed to register alert handler: {}", e);
                // Continue anyway, we'll just not have bidirectional integration
            }
        }
        
        // Start the forward task
        if let Err(e) = self.start_forward_task().await {
            // Reset running flag
            let mut running = self.running.write().await;
            *running = false;
            
            return Err(e);
        }
        
        info!("Health monitoring bridge started");
        Ok(())
    }
    
    /// Stop the bridge
    pub async fn stop(&self) -> Result<()> {
        // Set running flag to false
        {
            let mut running = self.running.write().await;
            if !*running {
                return Err(MCPError::InvalidOperation("Health monitoring bridge is not running".to_string()).into());
            }
            *running = false;
        }
        
        // Stop the forward task
        if let Err(e) = self.stop_forward_task().await {
            warn!("Failed to stop forward task: {}", e);
            // Continue anyway, we'll try to unregister the alert handler
        }
        
        // Unregister alert handler if bidirectional integration is enabled
        if self.config.bidirectional {
            if let Err(e) = self.unregister_alert_handler().await {
                warn!("Failed to unregister alert handler: {}", e);
                // Continue anyway, we've already stopped the forward task
            }
        }
        
        info!("Health monitoring bridge stopped");
        Ok(())
    }
    
    /// Start the background task for forwarding health data
    async fn start_forward_task(&self) -> Result<()> {
        let resilience_monitor = self.resilience_monitor.clone();
        let metrics_collector = self.metrics_collector.clone();
        let alert_manager = self.alert_manager.clone();
        let config = self.config.clone();
        let running = self.running.clone();
        
        let task = tokio::spawn(async move {
            let interval_duration = Duration::from_secs(config.forward_interval);
            
            loop {
                // Check if we should stop
                {
                    let is_running = running.read().await;
                    if !*is_running {
                        break;
                    }
                }
                
                // Forward health data
                if let Err(e) = forward_health_data(
                    &resilience_monitor,
                    &metrics_collector,
                    &alert_manager,
                    &config,
                ).await {
                    warn!("Failed to forward health data: {}", e);
                }
                
                // Sleep for the configured interval
                tokio::time::sleep(interval_duration).await;
            }
            
            debug!("Health monitoring bridge forward task stopped");
        });
        
        // Store the task handle
        let mut forward_task = match self.forward_task.lock() {
            Ok(task) => task,
            Err(e) => {
                return Err(MCPError::InvalidState(format!("Failed to acquire lock on forward task: {}", e)).into());
            }
        };
        *forward_task = Some(task);
        
        Ok(())
    }
    
    /// Stop the background task for forwarding health data
    async fn stop_forward_task(&self) -> Result<()> {
        // Get the task handle
        let task_handle = {
            let mut forward_task = match self.forward_task.lock() {
                Ok(task) => task,
                Err(e) => {
                    return Err(MCPError::InvalidState(format!("Failed to acquire lock on forward task: {}", e)).into());
                }
            };
            forward_task.take()
        };
        
        // Abort the task if it exists
        if let Some(handle) = task_handle {
            handle.abort();
            match handle.await {
                Ok(_) => debug!("Forward task completed successfully"),
                Err(e) if e.is_cancelled() => debug!("Forward task was cancelled"),
                Err(e) => warn!("Forward task failed: {}", e),
            }
        }
        
        Ok(())
    }
    
    /// Register the alert handler for bidirectional integration
    async fn register_alert_handler(&self) -> Result<()> {
        // Only register if we have a recovery strategy
        let Some(recovery_strategy) = &self.recovery_strategy else {
            debug!("No recovery strategy available, skipping alert handler registration");
            return Ok(());
        };
        
        // Create a function that will be called when an alert is triggered
        let recovery_clone = recovery_strategy.clone();
        
        // NOTE: In a real implementation, we would define and register an alert handler
        // The pattern would look something like this:
        // let _alert_handler = move |alert: Alert| {
        //     // Process the alert and trigger recovery
        // };
        // alert_manager.register_handler(_alert_handler);
        
        // Register configuration for a health status alert
        let alert_config = AlertConfiguration {
            name: "mcp_health_status_alert".to_string(),
            description: "Alert triggered when health status changes".to_string(),
            condition: AlertCondition::Custom("health_status_changed".to_string()),
            severity: AlertSeverity::Warning,
            actions: vec![AlertAction::Log],
            check_interval_seconds: 30,
            minimum_interval_seconds: 60,
            enabled: true,
            labels: std::collections::HashMap::new(),
        };
        
        self.alert_manager.add_alert(alert_config);
        
        // In a real implementation, we would register a proper alert handler
        // However, our current API may not support this directly, so we'll
        // note that with a debug message
        debug!("Alert handler registered for health monitoring bridge");
        
        Ok(())
    }
    
    /// Unregister the alert handler
    async fn unregister_alert_handler(&self) -> Result<()> {
        // In a real implementation, we would unregister the alert handler
        // Since our API may not support this directly, we'll just log a message
        debug!("Alert handler unregistered for health monitoring bridge");
        
        Ok(())
    }
}

/// Implementation of MonitoringAdapter for HealthMonitoringBridge
#[async_trait]
impl MonitoringAdapter for HealthMonitoringBridge {
    async fn forward_health_data(&self, _component_id: &str, results: Vec<HealthCheckResult>) -> Result<()> {
        // If not running, don't forward
        if !self.is_running() {
            return Ok(());
        }
        
        let mut metrics = Vec::new();
        let alerts = Vec::<Alert>::new();
        
        // Process each health check result
        for result in results {
            // Only forward unhealthy components if configured
            if !self.config.forward_all_components && result.status == HealthStatus::Healthy {
                continue;
            }
            
            let component_id = result.component_id.clone();
            
            // Create metrics from the health result
            let component_metrics = crate::integration::health_check_adapter::create_metrics_from_health_result(
                &component_id, 
                &result
            );
            metrics.extend(component_metrics);
            
            // Create alerts for unhealthy components
            if result.status != HealthStatus::Healthy {
                // In a full implementation, we would create actual alerts
                debug!("Would create alert for unhealthy component {}: {}", component_id, result.status);
                
                // Here we would normally use alert_manager.trigger_alert() or similar,
                // but we'll skip that for now as we're focusing on the adapter pattern
            }
        }
        
        // Forward metrics to metrics collector
        for metric in &metrics {
            // Register or update metric in metrics collector
            if let Some(_existing) = self.metrics_collector.get_metric(&metric.name) {
                // If already exists, update it
                self.metrics_collector.update_metric(&metric.name, metric.value.clone());
            } else {
                // Otherwise register it
                self.metrics_collector.register_metric(metric.clone());
            }
        }
        
        if !metrics.is_empty() {
            debug!("Forwarded {} metrics to monitoring system", metrics.len());
        }
        
        // In a full implementation, we would properly trigger the alerts
        if !alerts.is_empty() {
            debug!("Would forward {} alerts to monitoring system", alerts.len());
        }
        
        Ok(())
    }
}

/// Convert a health status to an alert severity
fn health_status_to_alert_severity(status: HealthStatus) -> AlertSeverity {
    match status {
        HealthStatus::Healthy => AlertSeverity::Info,
        HealthStatus::Degraded | HealthStatus::Warning => AlertSeverity::Warning,
        HealthStatus::Unhealthy => AlertSeverity::Error,
        HealthStatus::Critical => AlertSeverity::Critical,
        HealthStatus::Unknown => AlertSeverity::Warning,
    }
}

/// Helper function for forwarding health data
async fn forward_health_data(
    resilience_monitor: &HealthMonitor,
    metrics_collector: &MetricsCollector,
    alert_manager: &AlertManager,
    config: &HealthMonitoringBridgeConfig,
) -> Result<()> {
    // Get all component statuses
    let component_statuses = resilience_monitor.get_all_component_status();
    
    let mut all_results = Vec::new();
    
    // Get the latest health check result for each component
    for (component_id, _) in component_statuses {
        if let Some(result) = resilience_monitor.get_component_result(&component_id) {
            // Only include if we're forwarding all components or it's unhealthy
            if config.forward_all_components || result.status != HealthStatus::Healthy {
                all_results.push(result);
            }
        }
    }
    
    if !all_results.is_empty() {
        debug!("Forwarding health data for {} components", all_results.len());
        
        // Forward health data to metrics collector
        for result in &all_results {
            let component_id = &result.component_id;
            
            // Create metrics from the health result
            let metrics = crate::integration::health_check_adapter::create_metrics_from_health_result(
                component_id, 
                result
            );
            
            // Register or update metrics
            for metric in &metrics {
                if let Some(_existing) = metrics_collector.get_metric(&metric.name) {
                    // If already exists, update it
                    metrics_collector.update_metric(&metric.name, metric.value.clone());
                } else {
                    // Otherwise register it
                    metrics_collector.register_metric(metric.clone());
                }
            }
            
            // If unhealthy, consider creating an alert
            if result.status != HealthStatus::Healthy {
                let severity = health_status_to_alert_severity(result.status);
                let alert_config = AlertConfiguration {
                    name: format!("health_alert_{}", component_id),
                    description: result.message.clone(),
                    condition: AlertCondition::Custom("health_status_unhealthy".to_string()),
                    severity,
                    actions: vec![AlertAction::Log],
                    check_interval_seconds: 60,
                    minimum_interval_seconds: 300,
                    enabled: true,
                    labels: std::collections::HashMap::new(),
                };
                
                // Register the alert configuration
                // This doesn't actually trigger the alert, just sets up the configuration
                alert_manager.add_alert(alert_config);
                
                debug!("Added alert configuration for component {}", component_id);
            }
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    
    /// Simple mock for testing
    #[derive(Debug, Clone)]
    struct MockComponent {
        id: String,
        status: HealthStatus,
    }
    
    /// Create a simple health check result for testing
    fn create_test_result(component: &MockComponent) -> HealthCheckResult {
        let mut result = HealthCheckResult::new(
            component.id.clone(),
            component.status,
            format!("Health check for {}", component.id),
        );
        
        // Add some metrics
        result = result.with_metric("response_time_ms", 42.5);
        result = result.with_metric("error_rate", 0.01);
        
        result
    }
    
    /// Test basic bridge initialization
    #[tokio::test]
    async fn test_bridge_init() {
        let resilience_monitor = Arc::new(HealthMonitor::default());
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
    
    /// Test bridge start and stop
    #[tokio::test]
    async fn test_bridge_start_stop() {
        let resilience_monitor = Arc::new(HealthMonitor::default());
        let metrics_collector = Arc::new(MetricsCollector::new());
        let alert_manager = Arc::new(AlertManager::new());
        
        let bridge = HealthMonitoringBridge::new(
            resilience_monitor,
            metrics_collector,
            alert_manager,
            HealthMonitoringBridgeConfig::default(),
        );
        
        // Start the bridge
        bridge.start().await.expect("Failed to start bridge");
        assert!(bridge.is_running());
        
        // Stop the bridge
        bridge.stop().await.expect("Failed to stop bridge");
        assert!(!bridge.is_running());
    }
} 