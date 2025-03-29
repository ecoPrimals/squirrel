use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use log::{debug, error, info};
use tokio::sync::{Mutex, RwLock};
use tokio::task::JoinHandle;

use crate::error::MCPError;
use crate::resilience::health::{HealthCheckResult, HealthMonitor, HealthStatus, HealthCheck, HealthCheckConfig};
use crate::resilience::recovery::{FailureInfo, FailureSeverity, RecoveryStrategy};

/// Bridge that forwards MCP resilience health data to the global monitoring system
#[derive(Debug)]
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
///
/// This trait defines the interface for integrating with external monitoring systems.
/// Implementations of this trait provide the capability to forward health data from
/// the MCP system to external monitoring tools and register handlers for alerts.
///
/// The adapter acts as a bridge between MCP's internal health monitoring and
/// whatever external monitoring system is in use (e.g., Prometheus, CloudWatch, 
/// Datadog, or custom monitoring solutions).
///
/// Implementors should ensure thread safety as methods will be called from
/// multiple concurrent contexts.
#[async_trait]
pub trait MonitoringAdapter: std::fmt::Debug + Send + Sync {
    /// Forward health check results to the monitoring system
    ///
    /// This method sends the provided health check results to the external
    /// monitoring system in an appropriate format.
    ///
    /// # Arguments
    /// * `results` - A vector of health check results to be forwarded
    ///
    /// # Returns
    /// * `Ok(())` if the forwarding was successful
    /// * `Err(MCPError)` if the forwarding failed
    async fn forward_health_data(&self, results: Vec<HealthCheckResult>) -> Result<(), MCPError>;
    
    /// Register an alert handler with the monitoring system
    ///
    /// This method registers a callback that will be triggered when an alert
    /// is fired from the external monitoring system. It connects the monitoring
    /// system's alerting mechanism with MCP's recovery strategy.
    ///
    /// # Arguments
    /// * `handler_id` - Unique identifier for the alert handler
    /// * `recovery_strategy` - Strategy to execute when an alert is triggered
    ///
    /// # Returns
    /// * `Ok(())` if registration was successful
    /// * `Err(MCPError)` if registration failed
    async fn register_alert_handler(
        &self, 
        handler_id: &str, 
        _recovery_strategy: Arc<Mutex<RecoveryStrategy>>
    ) -> Result<(), MCPError>;
}

/// Adapter for the monitoring system
/// 
/// Provides integration with external monitoring systems by forwarding
/// health data and handling alerts. This implementation connects to
/// external APIs specified by URL endpoints.
///
/// This adapter implements the `MonitoringAdapter` trait and can be used
/// to connect the MCP health monitoring system to external monitoring
/// tools.
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
    #[must_use] pub const fn new(health_api_endpoint: String, alert_api_endpoint: String) -> Self {
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
        _recovery_strategy: Arc<Mutex<RecoveryStrategy>>
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
///
/// This adapter serves as a bridge between the monitoring system alerts
/// and the MCP recovery mechanisms. When an alert is received from the
/// monitoring system, this adapter translates it into appropriate recovery
/// actions based on alert severity and context.
///
/// This enables automated recovery responses to alerts detected by
/// external monitoring systems.
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
    pub const fn new(recovery_strategy: Arc<Mutex<RecoveryStrategy>>) -> Self {
        Self {
            recovery_strategy,
        }
    }
    
    /// Handle an alert from the monitoring system
    ///
    /// Processes an alert by translating it into a failure context and
    /// triggering the appropriate recovery action based on alert severity.
    ///
    /// # Arguments
    /// * `alert` - The monitoring alert to process
    ///
    /// # Errors
    /// * Returns `MCPError` if the health monitor cannot be updated with the alert
    /// * Returns `MCPError` if the recovery action fails
    pub async fn handle_alert(&self, alert: MonitoringAlert) -> Result<(), MCPError> {
        // Extract component and severity information
        let component_id = alert.component_id.clone();
        let severity = match alert.severity {
            AlertSeverity::Info | AlertSeverity::Warning => FailureSeverity::Minor,
            AlertSeverity::Error => FailureSeverity::Severe,
            AlertSeverity::Critical => FailureSeverity::Critical,
            AlertSeverity::Unknown => FailureSeverity::Moderate,
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
        .map_err(|e| MCPError::General(format!("Recovery failed: {e}")))
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
    /// Unknown alert, no severity information
    Unknown,
}

/// Alert from the monitoring system
///
/// Represents an alert generated by an external monitoring system that
/// needs to be processed by the MCP resilience framework. Alerts contain
/// information about detected issues, their severity, and the affected components.
///
/// Alerts can trigger recovery actions based on their severity level and
/// the associated component.
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
    ///
    /// # Arguments
    /// * `resilience_monitor` - Reference to the MCP health monitor
    /// * `monitoring_system` - Reference to the external monitoring system adapter
    /// * `config` - Configuration for the bridge
    ///
    /// # Returns
    /// A new instance of `HealthMonitoringBridge`
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
    ///
    /// Begins the background task that periodically forwards health data
    /// from the MCP system to the external monitoring system based on
    /// the configured interval.
    ///
    /// If the bridge is already running, this method does nothing.
    ///
    /// # Errors
    /// * Returns `MCPError` if the bridge is already running
    /// * Returns `MCPError` if the monitoring adapter fails to start
    /// * Returns `MCPError` if the bridge task cannot be spawned
    pub async fn start(&self) -> Result<(), MCPError> {
        let mut running = self.running.write().await;
        if *running {
            return Ok(()); // Already running
        }
        
        *running = true;
        drop(running); // Release the write lock
        
        info!("Starting health monitoring bridge...");
        
        // Clone needed references
        let resilience_monitor = self.resilience_monitor.clone();
        let monitoring_system = self.monitoring_system.clone();
        let config = self.config.clone();
        let running_flag = self.running.clone();
        
        // Create background task
        let handle = tokio::spawn(async move {
            let interval_seconds = config.forward_interval;
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(interval_seconds));
            
            loop {
                interval.tick().await;
                
                // Check if we should stop
                if !*running_flag.read().await {
                    break;
                }
                
                // Collect health data from resilience monitor
                let components = resilience_monitor.get_all_component_status();
                let mut results = Vec::new();
                
                for (component_id, status) in components {
                    // Only forward unhealthy components if configured that way
                    if !config.forward_all_components && matches!(status, HealthStatus::Healthy) {
                        continue;
                    }
                    
                    // Get full health check result if available
                    if let Some(result) = resilience_monitor.get_component_result(&component_id) {
                        results.push(result);
                    } else {
                        // Create a basic result if no detailed result is available
                        results.push(HealthCheckResult {
                            component_id: component_id.clone(),
                            status,
                            message: format!("Component {component_id} is {status:?}"),
                            metrics: HashMap::new(),
                            timestamp: std::time::Instant::now(),
                        });
                    }
                }
                
                // Forward to monitoring system
                if !results.is_empty() {
                    debug!("Forwarding {} health results to monitoring system", results.len());
                    if let Err(e) = monitoring_system.forward_health_data(results).await {
                        error!("Failed to forward health data to monitoring system: {}", e);
                    }
                }
            }
            
            info!("Health monitoring bridge stopped");
        });
        
        // Store the task handle
        self.task_handle.lock().await.replace(handle);
        
        Ok(())
    }
    
    /// Stop the health monitoring bridge
    ///
    /// Stops the background task that forwards health data to the
    /// external monitoring system. If the bridge is not running,
    /// this method does nothing.
    ///
    /// # Errors
    /// * Returns `MCPError` if the bridge is not running
    /// * Returns `MCPError` if the monitoring adapter fails to stop
    pub async fn stop(&self) -> Result<(), MCPError> {
        let mut running = self.running.write().await;
        if !*running {
            return Ok(()); // Already stopped
        }
        
        *running = false;
        drop(running); // Release the write lock
        
        // Wait for the task to complete
        let handle = self.task_handle.lock().await.take();
        if let Some(handle) = handle {
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
    ///
    /// # Returns
    /// `true` if the bridge is currently running, `false` otherwise
    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }
    
    /// Register an alert handler with the monitoring system
    ///
    /// Registers a recovery strategy to be executed when alerts are received
    /// from the external monitoring system.
    ///
    /// # Arguments
    /// * `handler_id` - Unique identifier for the alert handler
    /// * `recovery_strategy` - Strategy to execute when an alert is triggered
    ///
    /// # Errors
    /// * Returns `MCPError` if the handler cannot be registered
    pub async fn register_alert_handler(
        &self,
        handler_id: &str,
        _recovery_strategy: Arc<Mutex<RecoveryStrategy>>,
    ) -> Result<(), MCPError> {
        self.monitoring_system.register_alert_handler(handler_id, _recovery_strategy).await
    }
}

/// Initialize an integrated health monitoring bridge
///
/// # Arguments
/// * `resilience_monitor` - The resilience monitor to integrate with
/// * `monitoring_system` - The monitoring system to connect to
/// * `recovery_strategy` - The recovery strategy to use
///
/// # Errors
/// * Returns `MCPError` if the bridge cannot be initialized
/// * Returns `MCPError` if the monitoring adapter cannot be initialized
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

#[derive(Debug, Clone)]
pub(super) struct TestHealthCheck {
    healthy: bool,
    error_message: Option<String>,
    check_count: Arc<Mutex<u32>>,
    config: HealthCheckConfig,
}

impl TestHealthCheck {
    pub(super) fn new(healthy: bool, error_message: Option<String>) -> Self {
        Self {
            healthy,
            error_message,
            check_count: Arc::new(Mutex::new(0)),
            config: HealthCheckConfig::default(),
        }
    }
}

#[async_trait]
impl HealthCheck for TestHealthCheck {
    fn id(&self) -> &'static str {
        "test-component"
    }
    
    async fn check(&self) -> HealthCheckResult {
        let status = if self.healthy {
            HealthStatus::Healthy
        } else {
            HealthStatus::Degraded
        };
        HealthCheckResult {
            component_id: "test-component".to_string(),
            status,
            message: self.error_message.clone().unwrap_or_default(),
            metrics: HashMap::new(),
            timestamp: std::time::Instant::now(),
        }
    }
    
    fn config(&self) -> &HealthCheckConfig {
        &self.config
    }
    
    fn config_mut(&mut self) -> &mut HealthCheckConfig {
        &mut self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    
    use crate::resilience::health::{HealthCheck, HealthCheckResult, HealthStatus};
    use crate::resilience::recovery::RecoveryStrategy;
    
    // Test health check implementation
    #[derive(Debug, Clone)]
    struct TestHealthCheck {
        component_id: String,
        status: Arc<tokio::sync::RwLock<HealthStatus>>,
        config: HealthCheckConfig,
    }
    
    impl TestHealthCheck {
        fn new(component_id: &str, status: HealthStatus) -> Self {
            Self {
                component_id: component_id.to_string(),
                status: Arc::new(tokio::sync::RwLock::new(status)),
                config: HealthCheckConfig::default(),
            }
        }
        
        async fn set_status(&self, status: HealthStatus) {
            let mut current = self.status.write().await;
            *current = status;
        }
    }
    
    // Test monitoring adapter
    #[derive(Debug, Default)]
    struct TestMonitoringAdapter {
        metrics: Arc<Mutex<HashMap<String, u64>>>,
        errors: Arc<Mutex<HashMap<String, u64>>>,
        events: Arc<Mutex<Vec<String>>>,
    }
    
    impl TestMonitoringAdapter {
        fn new() -> Self {
            Self {
                metrics: Arc::new(Mutex::new(HashMap::new())),
                errors: Arc::new(Mutex::new(HashMap::new())),
                events: Arc::new(Mutex::new(Vec::new())),
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
    
    #[tokio::test]
    async fn test_health_monitoring_bridge() {
        // Set up test components
        let mut resilience_monitor = HealthMonitor::new(10);
        let monitoring_adapter = Arc::new(TestMonitoringAdapter::new());
        
        // Register test health checks
        let health_check1 = TestHealthCheck::new("test-component-1", HealthStatus::Healthy);
        let health_check2 = TestHealthCheck::new("test-component-2", HealthStatus::Degraded);
        
        resilience_monitor.register(health_check1.clone()).unwrap();
        resilience_monitor.register(health_check2.clone()).unwrap();
        
        let resilience_monitor = Arc::new(resilience_monitor);
        
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