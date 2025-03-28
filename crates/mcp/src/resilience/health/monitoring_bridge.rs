use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use chrono::Utc;
use log::{debug, error, info};
use tokio::sync::{Mutex, RwLock};
use tokio::task::JoinHandle;
use tokio::time::interval;

use crate::error::MCPError;
use crate::resilience::health::{HealthCheckResult, HealthMonitor, HealthStatus};
use crate::resilience::recovery::{FailureInfo, FailureSeverity, RecoveryStrategy};

/// Bridge that forwards MCP resilience health data to the global monitoring system
pub struct HealthMonitoringBridge {
    /// Reference to the MCP resilience health monitor
    resilience_monitor: Arc<HealthMonitor>,
    
    /// Reference to the monitoring system health checker adapter
    monitoring_system: Arc<dyn MonitoringAdapter + Send + Sync>,
    
    /// Configuration for the bridge
    config: HealthMonitoringBridgeConfig,
    
    /// Running state of the bridge
    running: Arc<RwLock<bool>>,
    
    /// Task handle for the bridge process
    task_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
}

/// Configuration for the health monitoring bridge
#[derive(Debug, Clone)]
pub struct HealthMonitoringBridgeConfig {
    /// How often to forward health data (in seconds)
    pub forward_interval: u64,
    
    /// Whether to forward all components or only unhealthy ones
    pub forward_all_components: bool,
    
    /// Whether to enable bidirectional integration
    pub bidirectional: bool,
}

impl Default for HealthMonitoringBridgeConfig {
    fn default() -> Self {
        Self {
            forward_interval: 10,
            forward_all_components: true,
            bidirectional: true,
        }
    }
}

/// Trait for adapting to different monitoring systems
#[async_trait]
pub trait MonitoringAdapter: std::fmt::Debug + Send + Sync {
    /// Forward health check results to the monitoring system
    async fn forward_health_data(&self, results: Vec<HealthCheckResult>) -> Result<(), MCPError>;
    
    /// Register an alert handler with the monitoring system
    async fn register_alert_handler(
        &self, 
        handler_id: &str, 
        recovery_strategy: Arc<Mutex<RecoveryStrategy>>
    ) -> Result<(), MCPError>;
}

/// Adapter for the monitoring system
/// 
/// Provides integration with external monitoring systems by forwarding
/// health data and handling alerts.
#[derive(Debug)]
pub struct MonitoringSystemAdapter {
    /// Endpoint for health data API
    health_api_endpoint: String,
    /// Endpoint for alert API
    alert_api_endpoint: String,
}

impl MonitoringSystemAdapter {
    /// Creates a new monitoring system adapter with the specified API endpoints
    ///
    /// # Arguments
    /// * `health_api_endpoint` - The URL for the health API
    /// * `alert_api_endpoint` - The URL for the alert API
    pub fn new(health_api_endpoint: String, alert_api_endpoint: String) -> Self {
        Self {
            health_api_endpoint,
            alert_api_endpoint,
        }
    }
}

#[async_trait]
impl MonitoringAdapter for MonitoringSystemAdapter {
    async fn forward_health_data(&self, results: Vec<HealthCheckResult>) -> Result<(), MCPError> {
        // In a real implementation, this would convert the health data to the monitoring system format
        // and make an API call to forward it
        
        info!("Forwarding {} health check results to monitoring system at {}", 
              results.len(), self.health_api_endpoint);
        
        for result in &results {
            debug!("Forwarding health data for component '{}': status={:?}, message='{}'",
                   result.component_id, result.status, result.message);
            
            // Here would be the actual API call to the monitoring system
        }
        
        Ok(())
    }
    
    async fn register_alert_handler(
        &self, 
        handler_id: &str, 
        recovery_strategy: Arc<Mutex<RecoveryStrategy>>
    ) -> Result<(), MCPError> {
        // In a real implementation, this would register a callback with the monitoring system
        // that would be triggered when an alert is fired
        
        info!("Registering alert handler '{}' with monitoring system at {}", 
              handler_id, self.alert_api_endpoint);
        
        // Here would be the actual API call to register the alert handler
        
        Ok(())
    }
}

/// Adapter that converts monitoring alerts to resilience recovery actions
#[derive(Debug)]
pub struct AlertToRecoveryAdapter {
    /// Reference to the recovery strategy
    recovery_strategy: Arc<Mutex<RecoveryStrategy>>,
}

impl AlertToRecoveryAdapter {
    /// Creates a new adapter for converting alerts to recovery actions
    ///
    /// # Arguments
    /// * `recovery_strategy` - The recovery strategy to use for handling alerts
    pub fn new(recovery_strategy: Arc<Mutex<RecoveryStrategy>>) -> Self {
        Self {
            recovery_strategy,
        }
    }
    
    /// Handle an alert from the monitoring system
    pub async fn handle_alert(&self, alert: MonitoringAlert) -> Result<(), MCPError> {
        // Extract component and severity information
        let component_id = alert.component_id.clone();
        let severity = match alert.severity {
            AlertSeverity::Info => FailureSeverity::Minor,
            AlertSeverity::Warning => FailureSeverity::Minor,
            AlertSeverity::Error => FailureSeverity::Moderate,
            AlertSeverity::Critical => FailureSeverity::Critical,
        };
        
        // Create failure info
        let failure_info = FailureInfo {
            message: alert.message.clone(),
            severity,
            context: component_id,
            recovery_attempts: 0,
        };
        
        // Trigger recovery action
        let mut recovery = self.recovery_strategy.lock().await;
        recovery.handle_failure(failure_info, || {
            // Default recovery action
            debug!("Executing default recovery action for alert: {}", alert.message);
            Ok(())
        })
        .map_err(|e| MCPError::General(format!("Recovery failed: {}", e)))
    }
}

/// Alert severity levels in the monitoring system
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AlertSeverity {
    /// Informational alert, lowest severity
    Info,
    /// Warning alert, low severity
    Warning,
    /// Error alert, moderate severity
    Error,
    /// Critical alert, highest severity
    Critical,
}

/// Alert from the monitoring system
#[derive(Debug, Clone)]
pub struct MonitoringAlert {
    /// Unique identifier for the alert
    pub id: Option<String>,
    /// Name of the alert
    pub name: String,
    /// Severity level
    pub severity: AlertSeverity,
    /// Alert message
    pub message: String,
    /// Source of the alert
    pub source: String,
    /// Component that triggered the alert
    pub component_id: String,
    /// When the alert was created
    pub timestamp: chrono::DateTime<Utc>,
    /// Additional metadata for the alert
    pub attributes: HashMap<String, String>,
}

impl HealthMonitoringBridge {
    /// Create a new health monitoring bridge
    pub fn new(
        resilience_monitor: Arc<HealthMonitor>,
        monitoring_system: Arc<dyn MonitoringAdapter + Send + Sync>,
        config: HealthMonitoringBridgeConfig,
    ) -> Self {
        Self {
            resilience_monitor,
            monitoring_system,
            config,
            running: Arc::new(RwLock::new(false)),
            task_handle: Arc::new(Mutex::new(None)),
        }
    }
    
    /// Start the health monitoring bridge
    pub async fn start(&self) -> Result<(), MCPError> {
        let mut running = self.running.write().await;
        if *running {
            return Ok(()); // Already running
        }
        
        *running = true;
        drop(running); // Release the write lock
        
        // Clone the required fields for the task
        let resilience_monitor = self.resilience_monitor.clone();
        let monitoring_system = self.monitoring_system.clone();
        let config = self.config.clone();
        let running = self.running.clone();
        
        // Start the forwarding task
        let handle = tokio::spawn(async move {
            let mut interval_timer = interval(Duration::from_secs(config.forward_interval));
            
            info!("Health monitoring bridge started with forward interval of {} seconds", 
                  config.forward_interval);
            
            loop {
                interval_timer.tick().await;
                
                // Check if we should still be running
                let is_running = {
                    let run_guard = running.read().await;
                    *run_guard
                };
                
                if !is_running {
                    break;
                }
                
                // Get all health checks
                let results = resilience_monitor.check_all().await;
                
                // Filter results if needed
                let filtered_results: Vec<HealthCheckResult> = if config.forward_all_components {
                    results.into_iter()
                          .map(|(_, result)| result)
                          .collect()
                } else {
                    results.into_iter()
                          .filter(|(_, check_result)| check_result.status != HealthStatus::Healthy)
                          .map(|(_, result)| result)
                          .collect()
                };
                
                // Forward the results
                if !filtered_results.is_empty() {
                    if let Err(e) = monitoring_system.forward_health_data(filtered_results).await {
                        error!("Failed to forward health data to monitoring system: {}", e);
                    }
                }
            }
            
            info!("Health monitoring bridge stopped");
        });
        
        // Store the task handle
        let mut task_handle = self.task_handle.lock().await;
        *task_handle = Some(handle);
        
        Ok(())
    }
    
    /// Stop the health monitoring bridge
    pub async fn stop(&self) -> Result<(), MCPError> {
        let mut running = self.running.write().await;
        if !*running {
            return Ok(()); // Already stopped
        }
        
        *running = false;
        drop(running); // Release the write lock
        
        // Wait for the task to complete
        let mut task_handle = self.task_handle.lock().await;
        if let Some(handle) = task_handle.take() {
            tokio::spawn(async move {
                if let Err(e) = handle.await {
                    error!("Error while stopping health monitoring bridge: {}", e);
                }
            });
        }
        
        info!("Health monitoring bridge stopping");
        Ok(())
    }
    
    /// Check if the bridge is running
    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }
    
    /// Register an alert handler with the monitoring system
    pub async fn register_alert_handler(
        &self,
        handler_id: &str,
        recovery_strategy: Arc<Mutex<RecoveryStrategy>>,
    ) -> Result<(), MCPError> {
        self.monitoring_system.register_alert_handler(handler_id, recovery_strategy).await
    }
}

/// Initialize the integrated health monitoring system
pub async fn initialize_integrated_health_monitoring(
    resilience_monitor: Arc<HealthMonitor>,
    monitoring_system: Arc<dyn MonitoringAdapter + Send + Sync>,
    recovery_strategy: Arc<Mutex<RecoveryStrategy>>,
) -> Result<HealthMonitoringBridge, MCPError> {
    // Configure bridge
    let bridge_config = HealthMonitoringBridgeConfig {
        forward_interval: 10,
        forward_all_components: true,
        bidirectional: true,
    };
    
    // Create bridge
    let bridge = HealthMonitoringBridge::new(
        resilience_monitor.clone(),
        monitoring_system.clone(),
        bridge_config,
    );
    
    // Start the bridge
    bridge.start().await?;
    
    // Register alert handler for recovery actions
    bridge.register_alert_handler("resilience_recovery", recovery_strategy.clone()).await?;
    
    Ok(bridge)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    
    use crate::resilience::health::{HealthCheck, HealthCheckResult, HealthStatus};
    use crate::resilience::recovery::RecoveryStrategy;
    
    // Test health check implementation
    struct TestHealthCheck {
        component_id: String,
        status: Arc<RwLock<HealthStatus>>,
    }
    
    impl TestHealthCheck {
        fn new(component_id: &str, status: HealthStatus) -> Self {
            Self {
                component_id: component_id.to_string(),
                status: Arc::new(RwLock::new(status)),
            }
        }
        
        async fn set_status(&self, status: HealthStatus) {
            let mut current = self.status.write().await;
            *current = status;
        }
    }
    
    #[async_trait]
    impl HealthCheck for TestHealthCheck {
        fn id(&self) -> &str {
            &self.component_id
        }
        
        async fn check(&self) -> HealthCheckResult {
            let status = *self.status.read().await;
            HealthCheckResult {
                component_id: self.component_id.clone(),
                status,
                message: format!("Test component status: {:?}", status),
                metrics: HashMap::new(),
            }
        }
    }
    
    // Test monitoring adapter
    struct TestMonitoringAdapter {
        forward_count: Arc<AtomicUsize>,
        last_results: Arc<Mutex<Vec<HealthCheckResult>>>,
    }
    
    impl TestMonitoringAdapter {
        fn new() -> Self {
            Self {
                forward_count: Arc::new(AtomicUsize::new(0)),
                last_results: Arc::new(Mutex::new(Vec::new())),
            }
        }
        
        async fn get_forward_count(&self) -> usize {
            self.forward_count.load(Ordering::SeqCst)
        }
        
        async fn get_last_results(&self) -> Vec<HealthCheckResult> {
            self.last_results.lock().await.clone()
        }
    }
    
    #[async_trait]
    impl MonitoringAdapter for TestMonitoringAdapter {
        async fn forward_health_data(&self, results: Vec<HealthCheckResult>) -> Result<(), MCPError> {
            self.forward_count.fetch_add(1, Ordering::SeqCst);
            let mut last_results = self.last_results.lock().await;
            *last_results = results;
            Ok(())
        }
        
        async fn register_alert_handler(
            &self, 
            _handler_id: &str, 
            _recovery_strategy: Arc<Mutex<RecoveryStrategy>>
        ) -> Result<(), MCPError> {
            // Just return success for the test
            Ok(())
        }
    }
    
    // Test recovery strategy
    struct TestRecoveryStrategy {
        recovery_count: AtomicUsize,
        last_failure: Mutex<Option<FailureInfo>>,
    }
    
    impl TestRecoveryStrategy {
        fn new() -> Self {
            Self {
                recovery_count: AtomicUsize::new(0),
                last_failure: Mutex::new(None),
            }
        }
        
        fn get_recovery_count(&self) -> usize {
            self.recovery_count.load(Ordering::SeqCst)
        }
        
        async fn get_last_failure(&self) -> Option<FailureInfo> {
            self.last_failure.lock().await.clone()
        }
    }
    
    impl RecoveryStrategy for TestRecoveryStrategy {
        fn handle_failure(
            &mut self, 
            failure: FailureInfo,
            _default_action: impl FnOnce() -> Result<(), Box<dyn std::error::Error>>
        ) -> Result<(), Box<dyn std::error::Error>> {
            self.recovery_count.fetch_add(1, Ordering::SeqCst);
            let mut last_failure = self.last_failure.lock().unwrap();
            *last_failure = Some(failure);
            Ok(())
        }
    }
    
    #[tokio::test]
    async fn test_health_monitoring_bridge() {
        // Set up test components
        let resilience_monitor = Arc::new(HealthMonitor::new());
        let monitoring_adapter = Arc::new(TestMonitoringAdapter::new());
        
        // Register test health checks
        let health_check1 = TestHealthCheck::new("test-component-1", HealthStatus::Healthy);
        let health_check2 = TestHealthCheck::new("test-component-2", HealthStatus::Degraded);
        
        resilience_monitor.register(Box::new(health_check1.clone())).await.unwrap();
        resilience_monitor.register(Box::new(health_check2.clone())).await.unwrap();
        
        // Create bridge with a short interval for testing
        let bridge_config = HealthMonitoringBridgeConfig {
            forward_interval: 1, // 1 second interval for faster testing
            forward_all_components: true,
            bidirectional: true,
        };
        
        let bridge = HealthMonitoringBridge::new(
            resilience_monitor.clone(),
            monitoring_adapter.clone(),
            bridge_config,
        );
        
        // Start bridge
        bridge.start().await.unwrap();
        
        // Wait for at least one forward
        tokio::time::sleep(Duration::from_secs(2)).await;
        
        // Verify data was forwarded
        assert!(monitoring_adapter.get_forward_count().await > 0);
        
        // Get the last results and verify they contain both components
        let results = monitoring_adapter.get_last_results().await;
        assert_eq!(results.len(), 2);
        
        // Find the results for each component
        let component1_result = results.iter().find(|r| r.component_id == "test-component-1");
        let component2_result = results.iter().find(|r| r.component_id == "test-component-2");
        
        assert!(component1_result.is_some());
        assert!(component2_result.is_some());
        
        // Verify the statuses
        assert_eq!(component1_result.unwrap().status, HealthStatus::Healthy);
        assert_eq!(component2_result.unwrap().status, HealthStatus::Degraded);
        
        // Stop the bridge
        bridge.stop().await.unwrap();
    }
    
    #[tokio::test]
    async fn test_alert_to_recovery() {
        // Set up test components
        let recovery_strategy = Arc::new(Mutex::new(TestRecoveryStrategy::new()));
        let adapter = AlertToRecoveryAdapter::new(recovery_strategy.clone());
        
        // Create test alert
        let alert = MonitoringAlert {
            id: Some("test-alert-1".to_string()),
            name: "component_unhealthy".to_string(),
            severity: AlertSeverity::Critical,
            message: "Component is unhealthy".to_string(),
            source: "test".to_string(),
            component_id: "test-component".to_string(),
            timestamp: chrono::Utc::now(),
            attributes: HashMap::new(),
        };
        
        // Handle the alert
        adapter.handle_alert(alert).await.unwrap();
        
        // Verify recovery was triggered
        let test_strategy = recovery_strategy.lock().await;
        let recovery_count = test_strategy.get_recovery_count();
        let last_failure = test_strategy.get_last_failure().await;
        
        assert_eq!(recovery_count, 1);
        assert!(last_failure.is_some());
        
        let failure = last_failure.unwrap();
        assert_eq!(failure.context, "test-component");
        assert_eq!(failure.severity, FailureSeverity::Critical);
        assert_eq!(failure.message, "Component is unhealthy");
    }
} 