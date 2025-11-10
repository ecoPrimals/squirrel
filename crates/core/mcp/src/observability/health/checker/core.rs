//! Core Health Checker Implementation
//!
//! This module contains the main HealthChecker struct and its core
//! functionality for managing health checks and component status.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use tokio::sync::broadcast;

use crate::observability::{ObservabilityResult, ObservabilityError};
use crate::observability::health::types::HealthStatus;
use crate::observability::health::component::ComponentHealth;
use crate::observability::health::result::HealthCheckResult;
use crate::observability::health::event::HealthStatusEvent;
use crate::observability::health::subscription::{
// Removed: use squirrel_mcp_config::get_service_endpoints;
    HealthStatusSubscriber, HealthStatusSubscriberNonBlocking
};
use crate::observability::health::HealthReport;

use super::types::{HealthCheck, HealthCheckFn, HealthCheckerConfig, HealthCheckType};
use super::execution::{run_health_check, execute_health_check_sync, update_component_health_with_result, calculate_system_status};
use super::scheduler;
use super::reporting;

/// Main health checker implementation
///
/// Manages health checks for multiple components and provides
/// status monitoring, subscription, and reporting capabilities.
pub struct HealthChecker {
    /// Map of component health
    component_health: RwLock<HashMap<String, ComponentHealth>>,
    /// Map of health checks by ID
    health_checks: RwLock<HashMap<String, Arc<HealthCheck>>>,
    /// Health check scheduler task handle
    _scheduler_task: Mutex<Option<tokio::task::JoinHandle<()>>>,
    /// Status change broadcaster
    status_change_tx: broadcast::Sender<HealthStatusEvent>,
    /// Configuration
    config: RwLock<HealthCheckerConfig>,
}

impl HealthChecker {
    /// Create a new health checker
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(100);
        
        Self {
            component_health: RwLock::new(HashMap::new()),
            health_checks: RwLock::new(HashMap::new()),
            _scheduler_task: Mutex::new(None),
            status_change_tx: tx,
            config: RwLock::new(HealthCheckerConfig::default()),
        }
    }
    
    /// Initialize the health checker
    pub fn initialize(&self) -> ObservabilityResult<()> {
        // Initialize any synchronous components here
        Ok(())
    }
    
    /// Initialize the health checker asynchronously
    pub async fn initialize_async(&self) -> ObservabilityResult<()> {
        self.start_scheduler().await
    }
    
    /// Start the health check scheduler
    pub async fn start_scheduler(&self) -> ObservabilityResult<()> {
        // Clone the necessary data for the scheduler
        let health_checks = Arc::new(RwLock::new(
            match self.health_checks.read() {
                Ok(checks) => checks.clone(),
                Err(_) => HashMap::new(),
            }
        ));
        
        let component_health = Arc::new(RwLock::new(
            match self.component_health.read() {
                Ok(health) => health.clone(),
                Err(_) => HashMap::new(),
            }
        ));
        
        // Start the scheduler task
        let scheduler_handle = tokio::spawn(async move {
            scheduler::start_scheduler(health_checks, component_health).await
        });
        
        // Store the handle
        if let Ok(mut task) = self._scheduler_task.lock() {
            *task = Some(scheduler_handle);
        }
        
        Ok(())
    }
    
    /// Execute health checks that are due to run
    pub async fn execute_due_health_checks(&self) -> ObservabilityResult<()> {
        scheduler::execute_due_health_checks(&self.health_checks, &self.component_health).await
    }
    
    /// Register a health check
    pub fn register_health_check(
        &self,
        component_id: &str,
        check_id: &str,
        description: &str,
        check_fn: HealthCheckFn,
    ) -> ObservabilityResult<()> {
        self.register_health_check_internal(
            check_id,
            component_id,
            description,
            HealthCheckType::Basic,
            check_fn,
            None,
        )
    }
    
    /// Register a health check (internal implementation)
    pub fn register_health_check_internal(
        &self,
        id: impl Into<String>,
        component_id: impl Into<String>,
        name: impl Into<String>,
        check_type: HealthCheckType,
        check_fn: HealthCheckFn,
        interval: Option<u64>,
    ) -> ObservabilityResult<()> {
        let check = HealthCheck::new(
            id.into(),
            component_id,
            name,
            check_type,
            check_fn,
            interval,
        );
        
        if let Ok(mut health_checks) = self.health_checks.write() {
            health_checks.insert(check.id.clone(), Arc::new(check));
            Ok(())
        } else {
            Err(ObservabilityError::HealthError(
                "Failed to write health checks".to_string(),
            ))
        }
    }
    
    /// Get a health check by ID
    pub fn get_check(&self, check_id: &str) -> ObservabilityResult<Option<Arc<HealthCheck>>> {
        match self.health_checks.read() {
            Ok(health_checks) => Ok(health_checks.get(check_id).cloned()),
            Err(_) => Err(ObservabilityError::HealthError(
                "Failed to read health checks".to_string(),
            )),
        }
    }
    
    /// Remove a health check
    pub fn remove_check(&self, check_id: &str) -> ObservabilityResult<Arc<HealthCheck>> {
        if let Ok(mut health_checks) = self.health_checks.write() {
            health_checks.remove(check_id).ok_or_else(|| {
                ObservabilityError::HealthError(format!("Health check '{}' not found", check_id))
            })
        } else {
            Err(ObservabilityError::HealthError(
                "Failed to write health checks".to_string(),
            ))
        }
    }
    
    /// Execute a specific health check by ID
    pub async fn execute_health_check(&self, id: &str) -> ObservabilityResult<HealthCheckResult> {
        let check = match self.health_checks.read() {
            Ok(health_checks) => {
                health_checks.get(id).cloned().ok_or_else(|| {
                    ObservabilityError::HealthError(format!("Health check '{}' not found", id))
                })?
            }
            Err(_) => {
                return Err(ObservabilityError::HealthError(
                    "Failed to read health checks".to_string(),
                ))
            }
        };
        
        let result = run_health_check(&check).await;
        
        // Update component status based on check result
        let _ = update_component_health_with_result(&self.component_health, &check.component_id, &result);
        
        Ok(result)
    }
    
    /// Get all health checks for a component
    pub async fn get_component_health_checks(&self, component_id: &str) -> ObservabilityResult<Vec<Arc<HealthCheck>>> {
        match self.health_checks.read() {
            Ok(health_checks) => {
                let checks = health_checks
                    .values()
                    .filter(|check| check.component_id == component_id)
                    .cloned()
                    .collect();
                Ok(checks)
            }
            Err(_) => Err(ObservabilityError::HealthError(
                "Failed to read health checks".to_string(),
            )),
        }
    }
    
    /// Execute all health checks for a component
    pub async fn execute_component_health_checks(&self, component_id: &str) -> ObservabilityResult<Vec<(String, HealthCheckResult)>> {
        let checks = self.get_component_health_checks(component_id).await?;
        let mut results = Vec::new();
        
        for check in checks {
            let result = run_health_check(&check).await;
            results.push((check.id.clone(), result.clone()));
            
            // Update component status based on check result
            let _ = update_component_health_with_result(&self.component_health, component_id, &result);
        }
        
        Ok(results)
    }
    
    /// Execute all health checks
    pub fn execute_all(&self) -> ObservabilityResult<HashMap<String, HealthCheckResult>> {
        if let Ok(health_checks) = self.health_checks.read() {
            let mut results = HashMap::new();
            
            for (id, check) in health_checks.iter() {
                match execute_health_check_sync(check) {
                    Ok(result) => {
                        results.insert(id.clone(), result.clone());
                        
                        // Update component status based on check result
                        let _ = update_component_health_with_result(&self.component_health, &check.component_id, &result);
                    }
                    Err(e) => {
                        // Create an error result
                        let error_result = HealthCheckResult::unhealthy(&format!("Check failed: {}", e));
                        results.insert(id.clone(), error_result.clone());
                        
                        // Update component status based on error result
                        let _ = update_component_health_with_result(&self.component_health, &check.component_id, &error_result);
                    }
                }
            }
            
            Ok(results)
        } else {
            Err(ObservabilityError::HealthError(
                "Failed to read health checks".to_string(),
            ))
        }
    }
    
    /// Check all components without modifying state
    pub fn check_all(&self) -> ObservabilityResult<HashMap<String, HealthCheckResult>> {
        self.execute_all()
    }
    
    /// Get all component health asynchronously
    pub async fn get_all_component_health(&self) -> ObservabilityResult<HashMap<String, ComponentHealth>> {
        match self.component_health.read() {
            Ok(health) => Ok(health.clone()),
            Err(_) => Err(ObservabilityError::HealthError(
                "Failed to read component health".to_string(),
            )),
        }
    }
    
    /// Subscribe to health status changes
    pub fn subscribe(&self) -> HealthStatusSubscriber {
        HealthStatusSubscriber::new(self.status_change_tx.subscribe())
    }
    
    /// Subscribe to status changes (returns receiver directly)
    pub fn subscribe_to_status_changes(&self) -> broadcast::Receiver<HealthStatusEvent> {
        self.status_change_tx.subscribe()
    }
    
    /// Subscribe to health status changes (non-blocking)
    pub fn subscribe_non_blocking(&self) -> HealthStatusSubscriberNonBlocking {
        HealthStatusSubscriberNonBlocking::new(self.status_change_tx.subscribe())
    }
    
    /// Shutdown the health checker
    pub async fn shutdown(&self) -> ObservabilityResult<()> {
        // Stop the scheduler if it's running
        if let Ok(mut task) = self._scheduler_task.lock() {
            if let Some(handle) = task.take() {
                handle.abort();
            }
        }
        Ok(())
    }
    
    /// Shutdown the health checker synchronously
    pub fn shutdown_sync(&self) -> ObservabilityResult<()> {
        // Check if we're already in a tokio runtime
        if tokio::runtime::Handle::try_current().is_ok() {
            // We're in a runtime, use block_in_place to run the async code
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    self.shutdown().await
                })
            })
        } else {
            // Not in a runtime, create a minimal one for this operation
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|e| 
                    ObservabilityError::HealthError(format!("Failed to create runtime: {}", e)))?;
            
            rt.block_on(async {
                self.shutdown().await
            })
        }
    }
    
    /// Calculate system status from component statuses
    pub fn calculate_system_status(component_statuses: &HashMap<String, ComponentHealth>) -> HealthStatus {
        calculate_system_status(component_statuses)
    }
    
    /// Register a component synchronously
    pub fn register_component_sync(
        &self,
        component_id: impl Into<String>,
        name: impl Into<String>,
        status: HealthStatus,
    ) -> ObservabilityResult<()> {
        let component_id_str = component_id.into();
        let name_str = name.into();
        
        if let Ok(mut component_health) = self.component_health.write() {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            
            component_health.insert(
                component_id_str.clone(),
                ComponentHealth {
                    component_id: component_id_str,
                    name: name_str,
                    status,
                    details: None,
                    last_checked: now,
                    last_status_change: now,
                    metadata: HashMap::new(),
                    tags: std::collections::HashSet::new(),
                },
            );
            
            Ok(())
        } else {
            Err(ObservabilityError::HealthError(
                "Failed to write component health".to_string(),
            ))
        }
    }
    
    /// Register a component
    pub async fn register_component(
        &self,
        component_id: impl Into<String>,
        name: impl Into<String>,
        status: HealthStatus,
    ) -> ObservabilityResult<()> {
        self.register_component_sync(component_id, name, status)
    }
    
    /// Get component health
    pub fn get_component_health(&self, component_id: &str) -> ObservabilityResult<Option<ComponentHealth>> {
        match self.component_health.read() {
            Ok(health) => Ok(health.get(component_id).cloned()),
            Err(_) => Err(ObservabilityError::HealthError(
                "Failed to read component health".to_string(),
            )),
        }
    }
    
    /// Get component status
    pub fn component_status(&self, component_id: &str) -> ObservabilityResult<HealthStatus> {
        match self.component_health.read() {
            Ok(health) => {
                health
                    .get(component_id)
                    .map(|c| c.status)
                    .ok_or_else(|| {
                        ObservabilityError::HealthError(format!("Component '{}' not found", component_id))
                    })
            }
            Err(_) => Err(ObservabilityError::HealthError(
                "Failed to read component health".to_string(),
            )),
        }
    }
    
    /// Get all component statuses
    pub fn all_component_status(&self) -> ObservabilityResult<HashMap<String, HealthStatus>> {
        match self.component_health.read() {
            Ok(health) => {
                let statuses = health
                    .iter()
                    .map(|(id, component)| (id.clone(), component.status))
                    .collect();
                Ok(statuses)
            }
            Err(_) => Err(ObservabilityError::HealthError(
                "Failed to read component health".to_string(),
            )),
        }
    }
    
    /// Update component status
    pub fn update_component_status(
        &self,
        component_id: &str,
        status: HealthStatus,
        details: Option<String>,
    ) -> ObservabilityResult<()> {
        if let Ok(mut component_health) = self.component_health.write() {
            if let Some(health) = component_health.get_mut(component_id) {
                let status_changed = health.status != status;
                health.status = status;
                health.details = details;
                
                if status_changed {
                    health.last_status_change = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();
                }
                
                Ok(())
            } else {
                Err(ObservabilityError::HealthError(format!("Component '{}' not found", component_id)))
            }
        } else {
            Err(ObservabilityError::HealthError(
                "Failed to write component health".to_string(),
            ))
        }
    }
    
    /// Update component status and notify subscribers
    pub fn update_component_status_and_notify(
        &self,
        component_id: &str,
        status: HealthStatus,
        details: Option<String>,
    ) -> ObservabilityResult<()> {
        self.update_component_status(component_id, status, details)?;
        
        // Create and send status event
        let event = self.create_status_event_sync()?;
        let _ = self.status_change_tx.send(event);
        
        Ok(())
    }
    
    /// Get overall system status
    pub fn overall_status(&self) -> ObservabilityResult<HealthStatus> {
        let component_health = match self.component_health.read() {
            Ok(health) => health,
            Err(_) => return Err(ObservabilityError::HealthError("Failed to read component health".to_string())),
        };
        
        let status = Self::calculate_system_status(&component_health);
        Ok(status)
    }
    
    /// Register a health check function
    pub fn register_check_fn(
        &self,
        component_id: &str,
        check_id: &str,
        description: &str,
        check_fn: impl Fn() -> HealthCheckResult + Send + Sync + 'static,
    ) -> ObservabilityResult<()> {
        self.register_health_check_internal(
            check_id,
            component_id,
            description,
            HealthCheckType::Basic,
            Box::new(check_fn),
            None,
        )
    }
    
    /// Register a health check object
    pub fn register_check(
        &self,
        health_check: HealthCheck,
    ) -> ObservabilityResult<Arc<HealthCheck>> {
        if let Ok(mut health_checks) = self.health_checks.write() {
            let check_arc = Arc::new(health_check);
            health_checks.insert(check_arc.id.clone(), check_arc.clone());
            Ok(check_arc)
        } else {
            Err(ObservabilityError::HealthError(
                "Failed to write health checks".to_string(),
            ))
        }
    }
    
    /// Execute a specific check by ID
    pub fn execute_check(&self, check_id: &str) -> ObservabilityResult<HealthCheckResult> {
        if let Ok(health_checks) = self.health_checks.read() {
            if let Some(check) = health_checks.get(check_id) {
                let result = execute_health_check_sync(check)?;
                
                // Update component status based on check result
                let _ = update_component_health_with_result(&self.component_health, &check.component_id, &result);
                
                Ok(result)
            } else {
                Err(ObservabilityError::HealthError(format!("Health check '{}' not found", check_id)))
            }
        } else {
            Err(ObservabilityError::HealthError("Failed to read health checks".to_string()))
        }
    }
    
    /// Check all health checks for a component
    pub fn check_component(&self, component_id: &str) -> ObservabilityResult<HashMap<String, HealthCheckResult>> {
        if let Ok(health_checks) = self.health_checks.read() {
            let mut results = HashMap::new();
            
            for (id, check) in health_checks.iter() {
                if check.component_id == component_id {
                    match execute_health_check_sync(check) {
                        Ok(result) => {
                            results.insert(id.clone(), result.clone());
                            
                            // Update component status based on check result
                            let _ = update_component_health_with_result(&self.component_health, component_id, &result);
                        }
                        Err(e) => {
                            // Create an error result
                            let error_result = HealthCheckResult::unhealthy(&format!("Check failed: {}", e));
                            results.insert(id.clone(), error_result.clone());
                            
                            // Update component status based on error result
                            let _ = update_component_health_with_result(&self.component_health, component_id, &error_result);
                        }
                    }
                }
            }
            
            Ok(results)
        } else {
            Err(ObservabilityError::HealthError("Failed to read health checks".to_string()))
        }
    }
    
    /// Get system health synchronously
    pub fn get_system_health_sync(&self) -> ObservabilityResult<HashMap<String, ComponentHealth>> {
        match self.component_health.read() {
            Ok(health) => Ok(health.clone()),
            Err(e) => Err(ObservabilityError::HealthError(format!("Failed to read component health: {}", e))),
        }
    }
    
    /// Create a status event synchronously
    pub fn create_status_event_sync(&self) -> ObservabilityResult<HealthStatusEvent> {
        reporting::create_status_event_sync(&self.component_health)
    }
    
    /// Get system health status
    pub fn get_system_health_status(&self) -> ObservabilityResult<HealthStatus> {
        // Get all component statuses
        let component_health = match self.component_health.read() {
            Ok(health) => health,
            Err(_) => return Err(ObservabilityError::HealthError("Failed to read component health".to_string())),
        };
        
        // Calculate system status based on component health
        let status = Self::calculate_system_status(&component_health);
        Ok(status)
    }
    
    /// Update a health check (compatibility method)
    pub fn update_check(
        &self,
        check_id: &str,
        check_fn: impl Fn() -> HealthCheckResult + Send + Sync + 'static,
    ) -> ObservabilityResult<()> {
        if let Ok(mut health_checks) = self.health_checks.write() {
            if let Some(check) = health_checks.get_mut(check_id) {
                // Update the check function
                let new_check = Arc::new(HealthCheck::new(
                    check_id,
                    &check.component_id,
                    &check.description,
                    check.check_type.clone(),
                    Box::new(check_fn),
                    check.interval,
                ));
                health_checks.insert(check_id.to_string(), new_check);
                Ok(())
            } else {
                Err(ObservabilityError::HealthError(format!("Health check '{}' not found", check_id)))
            }
        } else {
            Err(ObservabilityError::HealthError("Failed to write health checks".to_string()))
        }
    }

    /// Get health report
    pub fn get_health_report(&self) -> ObservabilityResult<HealthReport> {
        let overall_status = self.overall_status()?;
        let components = self.get_system_health_sync()?;
        
        Ok(HealthReport {
            overall_status,
            component_statuses: components,
            timestamp: std::time::SystemTime::now(),
        })
    }

    /// Get all checks for a component
    pub fn get_component_checks(&self, component_id: &str) -> ObservabilityResult<HashMap<String, HealthCheckResult>> {
        if let Ok(health_checks) = self.health_checks.read() {
            let mut results = HashMap::new();
            
            for (id, check) in health_checks.iter() {
                if check.component_id == component_id {
                    let result = (check.check_fn)();
                    results.insert(id.clone(), result);
                }
            }
            
            Ok(results)
        } else {
            Err(ObservabilityError::HealthError("Failed to read health checks".to_string()))
        }
    }
    
    /// Get a JSON report of the current health status
    pub fn get_json_report(&self) -> ObservabilityResult<String> {
        let report = self.get_health_report()?;
        serde_json::to_string_pretty(&report)
            .map_err(|e| ObservabilityError::HealthError(format!("Failed to serialize health report: {}", e)))
    }

    /// Get system health report (async version)
    pub async fn get_system_health(&self) -> ObservabilityResult<super::super::types::SystemHealthReport> {
        use super::super::types::{SystemHealthReport, ComponentHealthInfo, SystemHealthMetrics, SystemInfo};
        
        let component_health = self.component_health.read()
            .map_err(|_| ObservabilityError::HealthError("Failed to read component health".to_string()))?;
        
        let mut components = HashMap::new();
        let mut healthy = 0;
        let mut degraded = 0;
        let mut unhealthy = 0;
        let mut unknown = 0;
        
        for (id, health) in component_health.iter() {
            match health.status {
                HealthStatus::Healthy => healthy += 1,
                HealthStatus::Degraded => degraded += 1,
                HealthStatus::Unhealthy => unhealthy += 1,
                HealthStatus::Unknown => unknown += 1,
            }
            
            components.insert(id.clone(), ComponentHealthInfo {
                status: health.status,
                name: health.name.clone(),
                last_check: Some(std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(health.last_checked)),
                details: health.details.clone(),
                metrics: HashMap::new(), // Would populate with actual metrics
            });
        }
        
        let overall_status = if unhealthy > 0 {
            HealthStatus::Unhealthy
        } else if degraded > 0 {
            HealthStatus::Degraded
        } else if healthy > 0 {
            HealthStatus::Healthy
        } else {
            HealthStatus::Unknown
        };
        
        Ok(SystemHealthReport {
            overall_status,
            components,
            metrics: SystemHealthMetrics {
                healthy_components: healthy,
                degraded_components: degraded,
                unhealthy_components: unhealthy,
                unknown_components: unknown,
                uptime_seconds: 0, // Would get actual uptime
                memory_usage_percent: 0.0, // Would get actual memory usage
                cpu_usage_percent: 0.0, // Would get actual CPU usage
            },
            timestamp: std::time::SystemTime::now(),
            system_info: SystemInfo {
                service_name: "mcp".to_string(),
                version: "1.0.0".to_string(),
                environment: std::env::var("SQUIRREL_ENV").unwrap_or_else(|_| "development".to_string()),
                host: std::env::var("MCP_HOST")
                    .unwrap_or_else(|_| "localhost".to_string()),
            },
        })
    }

    /// Remove duplicate update_component_status method
    /// (Keep only the sync version and rename the async one)
    pub async fn update_component_status_async(
        &self,
        component_id: &str,
        status: HealthStatus,
        details: Option<String>,
    ) -> ObservabilityResult<()> {
        self.update_component_status_and_notify(component_id, status, details)
    }

    /// Add dependency between components
    pub async fn add_dependency(&self, component_id: &str, dependency_id: &str) -> ObservabilityResult<()> {
        // In a full implementation, this would track dependencies
        // For now, we'll just register the dependency component if it doesn't exist
        {
            let component_health = self.component_health.read()
                .map_err(|_| ObservabilityError::HealthError("Failed to read component health".to_string()))?;
            
            if !component_health.contains_key(dependency_id) {
                drop(component_health);
                self.register_component(dependency_id, dependency_id, HealthStatus::Unknown).await?;
            }
        }
        
        // Log the dependency relationship
        tracing::info!("Added dependency: {} depends on {}", component_id, dependency_id);
        Ok(())
    }
}

impl Default for HealthChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for HealthChecker {
    fn clone(&self) -> Self {
        // Create a new sender with the same capacity
        let (tx, _) = broadcast::channel(100);
        
        // Create a new instance with cloned data
        Self {
            component_health: RwLock::new(match self.component_health.read() {
                Ok(health) => health.clone(),
                Err(_) => HashMap::new(),
            }),
            health_checks: RwLock::new(match self.health_checks.read() {
                Ok(checks) => checks.clone(),
                Err(_) => HashMap::new(),
            }),
            _scheduler_task: Mutex::new(None),
            status_change_tx: tx,
            config: RwLock::new(match self.config.read() {
                Ok(config) => config.clone(),
                Err(_) => HealthCheckerConfig::default(),
            }),
        }
    }
} 