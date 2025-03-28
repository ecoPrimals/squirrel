//! Health monitoring components for the MCP resilience framework.
//! 
//! This module provides health monitoring capabilities for MCP components,
//! including integration with the global monitoring system.

mod monitoring_bridge;

pub use monitoring_bridge::{
    AlertSeverity,
    AlertToRecoveryAdapter,
    HealthMonitoringBridge,
    HealthMonitoringBridgeConfig,
    MonitoringAdapter,
    MonitoringAlert,
    MonitoringSystemAdapter,
    initialize_integrated_health_monitoring,
};

// Re-export the core health types directly
pub use super::recovery::{RecoveryStrategy, FailureInfo, FailureSeverity};

// Import common libraries used in the health module
use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};
use std::error::Error as StdError;
use async_trait::async_trait;
use thiserror::Error;

// Re-export types that were previously in health.rs
// This will be the new canonical location for these types
// (Instead of referencing them from the parent health.rs file)
pub use super::resilience_error::{ResilienceError, Result};

// Move all the health.rs contents here
// (Instead of re-exporting from parent health.rs)

/// Health status of a component
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HealthStatus {
    /// Component is healthy
    Healthy,
    /// Component is degraded but still operational
    Degraded,
    /// Component is in warning state
    Warning,
    /// Component is unhealthy and requires attention
    Unhealthy,
    /// Component is critical failure state
    Critical,
    /// Component status is unknown
    Unknown,
}

impl fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HealthStatus::Healthy => write!(f, "Healthy"),
            HealthStatus::Degraded => write!(f, "Degraded"),
            HealthStatus::Warning => write!(f, "Warning"),
            HealthStatus::Unhealthy => write!(f, "Unhealthy"),
            HealthStatus::Critical => write!(f, "Critical"),
            HealthStatus::Unknown => write!(f, "Unknown"),
        }
    }
}

impl HealthStatus {
    /// Convert health status to failure severity
    pub fn to_failure_severity(&self) -> Option<FailureSeverity> {
        match self {
            HealthStatus::Healthy => None, // No failure
            HealthStatus::Degraded => Some(FailureSeverity::Minor),
            HealthStatus::Warning => Some(FailureSeverity::Minor),
            HealthStatus::Unhealthy => Some(FailureSeverity::Moderate),
            HealthStatus::Critical => Some(FailureSeverity::Critical),
            HealthStatus::Unknown => Some(FailureSeverity::Moderate),
        }
    }
    
    /// Check if this status requires recovery action
    pub fn requires_recovery(&self) -> bool {
        matches!(self, 
            HealthStatus::Unhealthy | 
            HealthStatus::Critical |
            HealthStatus::Degraded)
    }
}

/// Health check result with details
#[derive(Debug, Clone)]
pub struct HealthCheckResult {
    /// Component ID being checked
    pub component_id: String,
    
    /// Status of the component
    pub status: HealthStatus,
    
    /// Message with additional details
    pub message: String,
    
    /// Timestamp when the check was performed
    pub timestamp: Instant,
    
    /// Any metrics associated with the health check
    pub metrics: HashMap<String, f64>,
}

impl HealthCheckResult {
    /// Create a new health check result
    pub fn new(component_id: String, status: HealthStatus, message: String) -> Self {
        Self {
            component_id,
            status,
            message,
            timestamp: Instant::now(),
            metrics: HashMap::new(),
        }
    }
    
    /// Add a metric to the health check result
    pub fn with_metric(mut self, key: &str, value: f64) -> Self {
        self.metrics.insert(key.to_string(), value);
        self
    }
    
    /// Check if this result requires recovery
    pub fn requires_recovery(&self) -> bool {
        self.status.requires_recovery()
    }
}

/// Health check configuration
#[derive(Debug, Clone)]
pub struct HealthCheckConfig {
    /// How often to run the health check
    pub check_interval: Duration,
    
    /// Timeout for health check operations
    pub check_timeout: Duration,
    
    /// Number of consecutive fails before changing status
    pub failure_threshold: u32,
    
    /// Number of consecutive passes before changing status
    pub recovery_threshold: u32,
    
    /// Whether to automatically trigger recovery
    pub auto_recovery: bool,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            check_interval: Duration::from_secs(60),
            check_timeout: Duration::from_secs(5),
            failure_threshold: 3,
            recovery_threshold: 2,
            auto_recovery: true,
        }
    }
}

/// Trait for implementing health checks
#[async_trait]
pub trait HealthCheck: Send + Sync {
    /// Unique identifier for this health check
    fn id(&self) -> &str;
    
    /// Perform the health check
    async fn check(&self) -> HealthCheckResult;
    
    /// Get the configuration for this health check
    fn config(&self) -> &HealthCheckConfig;
    
    /// Get mutable access to the configuration
    fn config_mut(&mut self) -> &mut HealthCheckConfig;
}

/// Metrics for health monitoring
#[derive(Debug, Default, Clone)]
pub struct HealthMonitoringMetrics {
    /// Total health checks performed
    pub total_checks: u64,
    
    /// Health checks by status
    pub checks_by_status: HashMap<HealthStatus, u64>,
    
    /// Recovery actions triggered
    pub recovery_actions: u64,
    
    /// Last check time
    pub last_check_time: Option<Instant>,
}

impl HealthMonitoringMetrics {
    /// Reset all metrics to default values
    pub fn reset(&mut self) {
        self.total_checks = 0;
        self.checks_by_status.clear();
        self.recovery_actions = 0;
        self.last_check_time = None;
    }
    
    /// Record a health check result
    pub fn record_check(&mut self, result: &HealthCheckResult) {
        self.total_checks += 1;
        *self.checks_by_status.entry(result.status).or_insert(0) += 1;
        self.last_check_time = Some(Instant::now());
    }
    
    /// Record a recovery action
    pub fn record_recovery(&mut self) {
        self.recovery_actions += 1;
    }
}

/// Tracks the health status history of a component
#[derive(Debug)]
struct ComponentHealthTracker {
    /// Component identifier
    component_id: String,
    
    /// Current health status
    current_status: HealthStatus,
    
    /// Previous health status
    previous_status: HealthStatus,
    
    /// Number of consecutive checks with current status
    consecutive_count: u32,
    
    /// Last check result
    last_result: Option<HealthCheckResult>,
    
    /// History of check results (limited size)
    history: Vec<HealthCheckResult>,
    
    /// Maximum history size
    max_history: usize,
}

impl ComponentHealthTracker {
    /// Create a new component health tracker
    fn new(component_id: String, max_history: usize) -> Self {
        Self {
            component_id,
            current_status: HealthStatus::Unknown,
            previous_status: HealthStatus::Unknown,
            consecutive_count: 0,
            last_result: None,
            history: Vec::with_capacity(max_history),
            max_history,
        }
    }
    
    /// Update the health tracker with a new check result
    fn update(&mut self, result: HealthCheckResult) {
        // Check if status has changed
        if result.status != self.current_status {
            self.previous_status = self.current_status;
            self.current_status = result.status;
            self.consecutive_count = 1;
        } else {
            self.consecutive_count += 1;
        }
        
        // Store the result in history
        self.last_result = Some(result.clone());
        self.history.push(result);
        
        // Trim history if it exceeds max size
        if self.history.len() > self.max_history {
            self.history.remove(0);
        }
    }
    
    /// Check if recovery should be triggered based on current status
    fn should_trigger_recovery(&self, failure_threshold: u32) -> bool {
        match self.current_status {
            HealthStatus::Healthy | HealthStatus::Warning | HealthStatus::Unknown => false,
            HealthStatus::Degraded | HealthStatus::Unhealthy | HealthStatus::Critical => {
                // Only trigger recovery if we've been in this status for at least
                // the failure threshold count
                self.consecutive_count >= failure_threshold
            }
        }
    }
    
    /// Get the last health check result if available
    fn last_result(&self) -> Option<&HealthCheckResult> {
        self.last_result.as_ref()
    }
}

/// Health monitoring system for MCP components
pub struct HealthMonitor {
    /// Health checks to run
    health_checks: HashMap<String, Box<dyn HealthCheck>>,
    
    /// Component health trackers
    component_trackers: RwLock<HashMap<String, ComponentHealthTracker>>,
    
    /// Recovery strategy for unhealthy components
    recovery: Option<Arc<Mutex<RecoveryStrategy>>>,
    
    /// Metrics for health monitoring
    metrics: Arc<Mutex<HealthMonitoringMetrics>>,
    
    /// Max history size for component trackers
    max_history_size: usize,
}

impl HealthMonitor {
    /// Create a new health monitor
    pub fn new(max_history_size: usize) -> Self {
        Self {
            health_checks: HashMap::new(),
            component_trackers: RwLock::new(HashMap::new()),
            recovery: None,
            metrics: Arc::new(Mutex::new(HealthMonitoringMetrics::default())),
            max_history_size,
        }
    }
    
    /// Create a default health monitor
    pub fn default() -> Self {
        Self::new(100) // Default to 100 history entries
    }
    
    /// Set the recovery strategy for this health monitor
    pub fn with_recovery(mut self, recovery: Arc<Mutex<RecoveryStrategy>>) -> Self {
        self.recovery = Some(recovery);
        self
    }
    
    /// Register a health check with the monitor
    pub fn register<H: HealthCheck + 'static>(&mut self, health_check: H) {
        let id = health_check.id().to_string();
        self.health_checks.insert(id.clone(), Box::new(health_check));
        
        // Initialize a tracker for this component if it doesn't exist
        let mut trackers = self.component_trackers.write().unwrap();
        if !trackers.contains_key(&id) {
            trackers.insert(
                id.clone(),
                ComponentHealthTracker::new(id, self.max_history_size)
            );
        }
    }
    
    /// Unregister a health check from the monitor
    pub fn unregister(&mut self, id: &str) -> bool {
        let result = self.health_checks.remove(id).is_some();
        
        // We keep the tracker in case we want to keep history
        // but we could also remove it here if desired
        
        result
    }
    
    /// Get the current status of a component
    pub fn component_status(&self, component_id: &str) -> HealthStatus {
        let trackers = self.component_trackers.read().unwrap();
        trackers.get(component_id)
            .map(|tracker| tracker.current_status)
            .unwrap_or(HealthStatus::Unknown)
    }
    
    /// Get the most recent health check result for a component
    pub fn last_check_result(&self, component_id: &str) -> Option<HealthCheckResult> {
        let trackers = self.component_trackers.read().unwrap();
        trackers.get(component_id)
            .and_then(|tracker| tracker.last_result().cloned())
    }
    
    /// Get the current status of all components
    pub fn all_statuses(&self) -> HashMap<String, HealthStatus> {
        let trackers = self.component_trackers.read().unwrap();
        trackers.iter()
            .map(|(id, tracker)| (id.clone(), tracker.current_status))
            .collect()
    }
    
    /// Run a health check for a specific component
    pub async fn check_component(&self, component_id: &str) -> Option<HealthCheckResult> {
        if let Some(check) = self.health_checks.get(component_id) {
            // Perform the health check
            let result = check.check().await;
            
            // Update the tracker
            let should_trigger_recovery = {
                let mut trackers = self.component_trackers.write().unwrap();
                let should_recover = if let Some(tracker) = trackers.get_mut(component_id) {
                    tracker.update(result.clone());
                    
                    // Check if we should trigger recovery
                    tracker.should_trigger_recovery(check.config().failure_threshold) && check.config().auto_recovery
                } else {
                    false
                };
                
                should_recover
            }; // trackers lock is dropped here
            
            // Trigger recovery outside the lock if needed
            if should_trigger_recovery {
                let _ = self.trigger_recovery(component_id, &result).await;
            }
            
            // Update metrics
            if let Ok(mut metrics) = self.metrics.lock() {
                metrics.record_check(&result);
            }
            
            Some(result)
        } else {
            None
        }
    }
    
    /// Run health checks for all registered components
    pub async fn check_all(&self) -> HashMap<String, HealthCheckResult> {
        let mut results = HashMap::new();
        
        // First collect all component IDs to avoid holding locks during iteration
        let component_ids: Vec<String> = self.health_checks.keys()
            .map(|id| id.to_string())
            .collect();
        
        // Run all health checks and collect results
        for component_id in component_ids {
            if let Some(result) = self.check_component(&component_id).await {
                results.insert(component_id, result);
            }
        }
        
        results
    }
    
    /// Trigger recovery action for an unhealthy component
    async fn trigger_recovery(&self, component_id: &str, result: &HealthCheckResult) -> bool {
        if let Some(recovery) = &self.recovery {
            // Create failure info from health check result
            let severity = result.status.to_failure_severity()
                .unwrap_or(FailureSeverity::Minor);
                
            let failure_info = FailureInfo {
                message: result.message.clone(),
                severity,
                context: component_id.to_string(),
                recovery_attempts: 0,
            };
            
            // Try to recover
            let recovery_result = {
                // Acquire mutex within a block to ensure it's dropped before the await
                let mut strategy = match recovery.lock() {
                    Ok(strategy) => strategy,
                    Err(_) => return false,
                };
                
                // Record recovery attempt in metrics
                if let Ok(mut metrics) = self.metrics.lock() {
                    metrics.record_recovery();
                }
                
                // Execute recovery strategy - this cannot be awaited while holding locks
                strategy.handle_failure(failure_info.clone(), || {
                    // Simple no-op recovery action that succeeds
                    Ok::<_, Box<dyn StdError + Send + Sync + 'static>>(())
                })
            };
            
            // Now evaluate the result
            match recovery_result {
                Ok(_) => true,
                Err(_) => false,
            }
        } else {
            // No recovery strategy configured
            false
        }
    }
    
    /// Get current metrics for health monitoring
    pub fn get_metrics(&self) -> HealthMonitoringMetrics {
        self.metrics.lock().map(|m| m.clone()).unwrap_or_default()
    }
    
    /// Reset all metrics
    pub fn reset_metrics(&self) {
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.reset();
        }
    }
}

/// Helper trait for working with health checks
pub trait HealthCheckExt: HealthCheck {
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

impl<T: HealthCheck + std::any::Any> HealthCheckExt for T {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// Health check related errors
#[derive(Debug, Error)]
pub enum HealthCheckError {
    /// Health check timed out
    #[error("Health check for component '{component_id}' timed out after {duration:?}")]
    Timeout {
        /// ID of the component being checked
        component_id: String,
        /// Duration after which the check timed out
        duration: Duration,
    },
    
    /// Health check failed
    #[error("Health check for component '{component_id}' failed: {message}")]
    CheckFailed {
        /// ID of the component being checked
        component_id: String,
        /// Detailed error message
        message: String,
    },
    
    /// Component is not available for health check
    #[error("Component '{component_id}' is unavailable for health check")]
    ComponentUnavailable {
        /// ID of the component being checked
        component_id: String,
    },
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    /// Mock health check for testing
    pub struct MockHealthCheck {
        /// ID of the health check
        id: String,
        /// Configuration for the health check
        config: HealthCheckConfig,
        /// Status to return
        status: Arc<std::sync::RwLock<HealthStatus>>,
        /// Call count
        call_count: Arc<AtomicU32>,
    }

    impl MockHealthCheck {
        /// Create a new mock health check
        pub fn new(id: &str, status: HealthStatus) -> Self {
            Self {
                id: id.to_string(),
                config: HealthCheckConfig::default(),
                status: Arc::new(std::sync::RwLock::new(status)),
                call_count: Arc::new(AtomicU32::new(0)),
            }
        }
        
        /// Set the health status for the mock
        pub fn set_status(&self, status: HealthStatus) {
            let mut s = self.status.write().unwrap();
            *s = status;
        }
        
        /// Get the number of times this check has been called
        pub fn call_count(&self) -> u32 {
            self.call_count.load(Ordering::SeqCst)
        }
    }
    
    #[async_trait]
    impl HealthCheck for MockHealthCheck {
        fn id(&self) -> &str {
            &self.id
        }
        
        async fn check(&self) -> HealthCheckResult {
            self.call_count.fetch_add(1, Ordering::SeqCst);
            let status = *self.status.read().unwrap();
            HealthCheckResult::new(
                self.id.clone(),
                status,
                format!("Mock health check: {:?}", status)
            )
        }
        
        fn config(&self) -> &HealthCheckConfig {
            &self.config
        }
        
        fn config_mut(&mut self) -> &mut HealthCheckConfig {
            &mut self.config
        }
    }
    
    #[tokio::test]
    async fn test_health_check_result() {
        let result = HealthCheckResult::new(
            "test-component".to_string(),
            HealthStatus::Healthy,
            "All systems operational".to_string()
        )
        .with_metric("response_time_ms", 42.5)
        .with_metric("memory_usage_mb", 128.0);
        
        assert_eq!(result.component_id, "test-component");
        assert_eq!(result.status, HealthStatus::Healthy);
        assert_eq!(result.message, "All systems operational");
        assert_eq!(result.metrics.get("response_time_ms"), Some(&42.5));
        assert_eq!(result.metrics.get("memory_usage_mb"), Some(&128.0));
        assert!(!result.requires_recovery());
    }
    
    #[tokio::test]
    async fn test_health_monitor_basic() {
        let mut monitor = HealthMonitor::default();
        
        // Register a mock health check
        let mock_check = MockHealthCheck::new("test-component", HealthStatus::Healthy);
        monitor.register(mock_check);
        
        // Check the component
        let result = monitor.check_component("test-component").await.unwrap();
        
        assert_eq!(result.status, HealthStatus::Healthy);
        assert_eq!(monitor.component_status("test-component"), HealthStatus::Healthy);
        assert_eq!(monitor.all_statuses().get("test-component"), Some(&HealthStatus::Healthy));
    }
    
    #[tokio::test]
    async fn test_health_status_transition() {
        let mut monitor = HealthMonitor::default();
        
        // Register a mock health check with auto-recovery disabled
        let mock_check = MockHealthCheck::new("test-component", HealthStatus::Healthy);
        let mut config = HealthCheckConfig::default();
        config.auto_recovery = false;
        config.failure_threshold = 2;
        // TODO: Set the config on mock_check
        
        monitor.register(mock_check);
        
        // Initial check
        let result = monitor.check_component("test-component").await.unwrap();
        assert_eq!(result.status, HealthStatus::Healthy);
        
        // Get the mock check and change its status
        let checks = &monitor.health_checks;
        let mock = checks.get("test-component")
            .and_then(|check| check.as_any().downcast_ref::<MockHealthCheck>())
            .unwrap();
            
        // Change to unhealthy
        mock.set_status(HealthStatus::Unhealthy);
        
        // First unhealthy check
        let result = monitor.check_component("test-component").await.unwrap();
        assert_eq!(result.status, HealthStatus::Unhealthy);
        
        // Second unhealthy check - should now have consecutive_count = 2
        let result = monitor.check_component("test-component").await.unwrap();
        assert_eq!(result.status, HealthStatus::Unhealthy);
        
        // Verify metrics
        let metrics = monitor.get_metrics();
        assert_eq!(metrics.total_checks, 3);
        assert_eq!(*metrics.checks_by_status.get(&HealthStatus::Healthy).unwrap_or(&0), 1);
        assert_eq!(*metrics.checks_by_status.get(&HealthStatus::Unhealthy).unwrap_or(&0), 2);
    }
} 