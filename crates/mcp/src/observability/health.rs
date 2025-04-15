//! # Health Checking
//! 
//! This module provides health checking capabilities for MCP components,
//! enabling monitoring and automated recovery.
//!
//! ## Key Components
//!
//! - **HealthStatus**: Represents the health status of a component
//! - **HealthCheck**: A specific check for a component 
//! - **HealthChecker**: Manages and executes health checks
//! - **HealthRegistry**: Registers components for health checking

use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};
use tokio::sync::broadcast;
use crate::observability::{ObservabilityError, ObservabilityResult};

/// Health status of a component
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthStatus {
    /// Component is healthy and working correctly
    Healthy,
    /// Component is operational but showing signs of issues
    Degraded,
    /// Component is not functioning correctly
    Unhealthy,
    /// Component status is unknown
    Unknown,
}

impl std::fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Healthy => write!(f, "HEALTHY"),
            Self::Degraded => write!(f, "DEGRADED"),
            Self::Unhealthy => write!(f, "UNHEALTHY"),
            Self::Unknown => write!(f, "UNKNOWN"),
        }
    }
}

/// Detailed health check result
#[derive(Debug, Clone)]
pub struct HealthCheckResult {
    /// Status of the check
    status: HealthStatus,
    /// Detailed message about the check result
    message: String,
    /// Additional context or metadata
    details: HashMap<String, String>,
    /// When the check was performed
    timestamp: Instant,
}

impl HealthCheckResult {
    /// Create a new health check result
    pub fn new(status: HealthStatus, message: impl Into<String>) -> Self {
        Self {
            status,
            message: message.into(),
            details: HashMap::new(),
            timestamp: Instant::now(),
        }
    }

    /// Add a detail to the health check result
    pub fn with_detail(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.details.insert(key.into(), value.into());
        self
    }

    /// Get the status
    pub fn status(&self) -> HealthStatus {
        self.status
    }

    /// Get the message
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Get the details
    pub fn details(&self) -> &HashMap<String, String> {
        &self.details
    }

    /// Get the timestamp
    pub fn timestamp(&self) -> Instant {
        self.timestamp
    }

    /// Create a new healthy result
    pub fn healthy(message: impl Into<String>) -> Self {
        Self::new(HealthStatus::Healthy, message)
    }

    /// Create a new degraded result
    pub fn degraded(message: impl Into<String>) -> Self {
        Self::new(HealthStatus::Degraded, message)
    }

    /// Create a new unhealthy result
    pub fn unhealthy(message: impl Into<String>) -> Self {
        Self::new(HealthStatus::Unhealthy, message)
    }

    /// Create a new unknown result
    pub fn unknown(message: impl Into<String>) -> Self {
        Self::new(HealthStatus::Unknown, message)
    }
}

/// Health check function signature
pub type HealthCheckFn = Box<dyn Fn() -> HealthCheckResult + Send + Sync>;

/// A health check for a component
pub struct HealthCheck {
    /// Name of the health check
    name: String,
    /// Component this check is for
    component: String,
    /// Description of what this check verifies
    description: String,
    /// Function that performs the check
    check_fn: HealthCheckFn,
    /// Last result of this check
    last_result: Mutex<Option<HealthCheckResult>>,
}

impl std::fmt::Debug for HealthCheck {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HealthCheck")
            .field("name", &self.name)
            .field("component", &self.component)
            .field("description", &self.description)
            .field("last_result", &self.last_result)
            .finish_non_exhaustive() // Skip check_fn field as it doesn't implement Debug
    }
}

impl HealthCheck {
    /// Create a new health check
    pub fn new(
        name: impl Into<String>,
        component: impl Into<String>,
        description: impl Into<String>,
        check_fn: HealthCheckFn,
    ) -> Self {
        Self {
            name: name.into(),
            component: component.into(),
            description: description.into(),
            check_fn,
            last_result: Mutex::new(None),
        }
    }

    /// Execute the health check
    pub fn execute(&self) -> ObservabilityResult<HealthCheckResult> {
        let result = (self.check_fn)();
        
        let mut last_result = self.last_result.lock().map_err(|e| 
            ObservabilityError::HealthError(format!("Failed to acquire last_result lock: {}", e)))?;
        *last_result = Some(result.clone());
        
        Ok(result)
    }

    /// Get the name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the component
    pub fn component(&self) -> &str {
        &self.component
    }

    /// Get the description
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Get the last result
    pub fn last_result(&self) -> ObservabilityResult<Option<HealthCheckResult>> {
        let last_result = self.last_result.lock().map_err(|e| 
            ObservabilityError::HealthError(format!("Failed to acquire last_result lock: {}", e)))?;
        Ok(last_result.clone())
    }
}

/// Component health status notification
#[derive(Debug, Clone)]
pub struct HealthStatusChange {
    /// Component that changed status
    pub component: String,
    /// Previous health status
    pub previous: HealthStatus,
    /// New health status
    pub current: HealthStatus,
    /// Time of the status change
    pub timestamp: Instant,
    /// Details about the change
    pub details: HashMap<String, String>,
}

/// Configuration for the health checker
#[derive(Debug, Clone)]
pub struct HealthCheckerConfig {
    /// Default check interval
    pub check_interval: Duration,
    /// Whether to auto-start checking
    pub auto_start: bool,
    /// Number of status change notifications to buffer
    pub notification_buffer: usize,
}

impl Default for HealthCheckerConfig {
    fn default() -> Self {
        Self {
            check_interval: Duration::from_secs(30),
            auto_start: true,
            notification_buffer: 100,
        }
    }
}

/// The health checker manages and executes health checks
#[derive(Debug)]
pub struct HealthChecker {
    /// Health checker configuration
    config: RwLock<HealthCheckerConfig>,
    /// All registered health checks
    checks: RwLock<HashMap<String, Arc<HealthCheck>>>,
    /// Components and their last reported status
    component_status: RwLock<HashMap<String, HealthStatus>>,
    /// Channel for status change notifications
    status_change_tx: broadcast::Sender<HealthStatusChange>,
}

impl HealthChecker {
    /// Create a new health checker
    pub fn new() -> Self {
        let (status_change_tx, _) = broadcast::channel(100);
        
        Self {
            config: RwLock::new(HealthCheckerConfig::default()),
            checks: RwLock::new(HashMap::new()),
            component_status: RwLock::new(HashMap::new()),
            status_change_tx,
        }
    }

    /// Initialize the health checker
    pub fn initialize(&self) -> ObservabilityResult<()> {
        // Initialize with default configuration
        let config = self.config.read().map_err(|e| 
            ObservabilityError::HealthError(format!("Failed to acquire config read lock: {}", e)))?;
            
        if config.auto_start {
            // In a real implementation, we would start a background task for regular health checks
        }
        
        Ok(())
    }

    /// Set the health checker configuration
    pub fn set_config(&self, config: HealthCheckerConfig) -> ObservabilityResult<()> {
        let mut current_config = self.config.write().map_err(|e| 
            ObservabilityError::HealthError(format!("Failed to acquire config write lock: {}", e)))?;
        *current_config = config;
        Ok(())
    }

    /// Register a health check
    pub fn register_check(&self, check: HealthCheck) -> ObservabilityResult<Arc<HealthCheck>> {
        let check_arc = Arc::new(check);
        let name = check_arc.name().to_string();
        let component = check_arc.component().to_string();
        
        let mut checks = self.checks.write().map_err(|e| 
            ObservabilityError::HealthError(format!("Failed to acquire checks write lock: {}", e)))?;
        
        checks.insert(name, check_arc.clone());
        
        // Initialize component status as unknown
        let mut component_status = self.component_status.write().map_err(|e| 
            ObservabilityError::HealthError(format!("Failed to acquire component_status write lock: {}", e)))?;
        
        if !component_status.contains_key(&component) {
            component_status.insert(component, HealthStatus::Unknown);
        }
        
        Ok(check_arc)
    }

    /// Register a simple health check with a function
    pub fn register_check_fn(
        &self,
        name: impl Into<String>,
        component: impl Into<String>,
        description: impl Into<String>,
        check_fn: impl Fn() -> HealthCheckResult + Send + Sync + 'static,
    ) -> ObservabilityResult<Arc<HealthCheck>> {
        let check = HealthCheck::new(
            name,
            component,
            description,
            Box::new(check_fn),
        );
        
        self.register_check(check)
    }

    /// Execute all health checks for a component
    pub fn check_component(&self, component: &str) -> ObservabilityResult<Vec<HealthCheckResult>> {
        let checks = self.checks.read().map_err(|e| 
            ObservabilityError::HealthError(format!("Failed to acquire checks read lock: {}", e)))?;
        
        let mut results = Vec::new();
        let mut has_unhealthy = false;
        let mut has_degraded = false;
        let mut has_healthy = false;
        
        // Execute all checks for this component
        for check in checks.values() {
            if check.component() == component {
                let result = check.execute()?;
                match result.status() {
                    HealthStatus::Unhealthy => has_unhealthy = true,
                    HealthStatus::Degraded => has_degraded = true,
                    HealthStatus::Healthy => has_healthy = true,
                    _ => {}
                }
                results.push(result);
            }
        }
        
        // Update component status based on check results
        let mut component_status = self.component_status.write().map_err(|e| 
            ObservabilityError::HealthError(format!("Failed to acquire component_status write lock: {}", e)))?;
        
        let previous = component_status.get(component).copied().unwrap_or(HealthStatus::Unknown);
        
        // Determine new status (worst status wins)
        let new_status = if has_unhealthy {
            HealthStatus::Unhealthy
        } else if has_degraded {
            HealthStatus::Degraded
        } else if has_healthy {
            HealthStatus::Healthy
        } else {
            HealthStatus::Unknown
        };
        
        // Update status and emit notification if changed
        if previous != new_status {
            component_status.insert(component.to_string(), new_status);
            
            let status_change = HealthStatusChange {
                component: component.to_string(),
                previous,
                current: new_status,
                timestamp: Instant::now(),
                details: HashMap::new(),
            };
            
            let _ = self.status_change_tx.send(status_change);
        }
        
        Ok(results)
    }

    /// Execute a specific health check
    pub fn execute_check(&self, check_name: &str) -> ObservabilityResult<HealthCheckResult> {
        let checks = self.checks.read().map_err(|e| 
            ObservabilityError::HealthError(format!("Failed to acquire checks read lock: {}", e)))?;
        
        let check = checks.get(check_name).ok_or_else(|| 
            ObservabilityError::HealthError(format!("Health check not found: {}", check_name)))?;
        
        check.execute()
    }

    /// Execute all registered health checks
    pub fn check_all(&self) -> ObservabilityResult<HashMap<String, Vec<HealthCheckResult>>> {
        let checks = self.checks.read().map_err(|e| 
            ObservabilityError::HealthError(format!("Failed to acquire checks read lock: {}", e)))?;
        
        let mut components = std::collections::HashSet::new();
        for check in checks.values() {
            components.insert(check.component().to_string());
        }
        
        let mut results = HashMap::new();
        for component in components {
            results.insert(component.clone(), self.check_component(&component)?);
        }
        
        Ok(results)
    }

    /// Get the current status of a component
    pub fn component_status(&self, component: &str) -> ObservabilityResult<HealthStatus> {
        let component_status = self.component_status.read().map_err(|e| 
            ObservabilityError::HealthError(format!("Failed to acquire component_status read lock: {}", e)))?;
        
        Ok(component_status.get(component).copied().unwrap_or(HealthStatus::Unknown))
    }

    /// Get status of all components
    pub fn all_component_status(&self) -> ObservabilityResult<HashMap<String, HealthStatus>> {
        let component_status = self.component_status.read().map_err(|e| 
            ObservabilityError::HealthError(format!("Failed to acquire component_status read lock: {}", e)))?;
        
        Ok(component_status.clone())
    }

    /// Get a specific health check
    pub fn get_check(&self, name: &str) -> ObservabilityResult<Option<Arc<HealthCheck>>> {
        let checks = self.checks.read().map_err(|e| 
            ObservabilityError::HealthError(format!("Failed to acquire checks read lock: {}", e)))?;
        
        Ok(checks.get(name).cloned())
    }

    /// Get all health checks for a component
    pub fn get_component_checks(&self, component: &str) -> ObservabilityResult<Vec<Arc<HealthCheck>>> {
        let checks = self.checks.read().map_err(|e| 
            ObservabilityError::HealthError(format!("Failed to acquire checks read lock: {}", e)))?;
        
        let mut result = Vec::new();
        for check in checks.values() {
            if check.component() == component {
                result.push(check.clone());
            }
        }
        
        Ok(result)
    }

    /// Subscribe to health status changes
    pub fn subscribe(&self) -> broadcast::Receiver<HealthStatusChange> {
        self.status_change_tx.subscribe()
    }

    /// Start automatic health checking
    pub async fn start_checking(&self) -> ObservabilityResult<()> {
        // In a real implementation, this would start a background task that runs health checks
        // at the configured interval. For this example, we'll just check everything once.
        self.check_all()?;
        Ok(())
    }
}

impl Default for HealthChecker {
    fn default() -> Self {
        Self::new()
    }
} 