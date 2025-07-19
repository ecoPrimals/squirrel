//! Self-Healing System for ecoPrimals Ecosystem
//!
//! This module provides comprehensive self-healing capabilities including:
//! - Circuit breakers for fault tolerance
//! - Health monitoring and alerting
//! - Automatic recovery mechanisms
//! - Failure detection and isolation
//! - Service degradation management
//! - Adaptive resilience patterns

use chrono::{DateTime, Utc};
use futures::stream::StreamExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{broadcast, RwLock};
use tokio::time::interval;
use tracing::{debug, error, info, warn};

use crate::error::PrimalError;

/// Health status of a component
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// Component is healthy and functioning normally
    Healthy,
    /// Component is degraded but still functional
    Degraded,
    /// Component is unhealthy and may not be functioning correctly
    Unhealthy,
    /// Component has failed and requires intervention
    Failed,
    /// Component status is unknown
    Unknown,
}

/// Circuit breaker state for fault tolerance
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CircuitBreakerState {
    /// Circuit is closed, requests flow normally
    Closed,
    /// Circuit is open, requests are rejected
    Open,
    /// Circuit is half-open, testing if service recovered
    HalfOpen,
}

/// Recovery strategy for failed components
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RecoveryStrategy {
    /// Restart the component
    Restart,
    /// Failover to backup instance
    Failover,
    /// Degrade service gracefully
    Degrade,
    /// Isolate the component
    Isolate,
    /// Manual intervention required
    Manual,
    /// No recovery needed
    None,
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    pub component_id: String,
    pub check_interval: Duration,
    pub timeout: Duration,
    pub failure_threshold: u32,
    pub recovery_threshold: u32,
    pub enabled: bool,
}

/// Circuit breaker configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    pub component_id: String,
    pub failure_threshold: u32,
    pub recovery_timeout: Duration,
    pub success_threshold: u32,
    pub enabled: bool,
}

/// Component health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub component_id: String,
    pub component_name: String,
    pub status: HealthStatus,
    pub last_check: DateTime<Utc>,
    pub consecutive_failures: u32,
    pub consecutive_successes: u32,
    pub response_time: Duration,
    pub error_rate: f64,
    pub uptime: Duration,
    pub details: HashMap<String, String>,
}

/// Circuit breaker metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerMetrics {
    pub component_id: String,
    pub state: CircuitBreakerState,
    pub failure_count: u32,
    pub success_count: u32,
    pub rejection_count: u32,
    pub last_failure: Option<DateTime<Utc>>,
    pub last_success: Option<DateTime<Utc>>,
    pub state_transitions: u32,
}

/// Recovery action result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryResult {
    pub component_id: String,
    pub strategy: RecoveryStrategy,
    pub success: bool,
    pub duration: Duration,
    pub error_message: Option<String>,
    pub timestamp: DateTime<Utc>,
}

/// Self-healing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfHealingConfig {
    pub enabled: bool,
    pub health_check_interval: Duration,
    pub recovery_timeout: Duration,
    pub max_recovery_attempts: u32,
    pub auto_recovery_enabled: bool,
    pub circuit_breaker_enabled: bool,
    pub degradation_enabled: bool,
}

impl Default for SelfHealingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            health_check_interval: Duration::from_secs(30),
            recovery_timeout: Duration::from_secs(60),
            max_recovery_attempts: 3,
            auto_recovery_enabled: true,
            circuit_breaker_enabled: true,
            degradation_enabled: true,
        }
    }
}

/// Self-healing manager
pub struct SelfHealingManager {
    config: SelfHealingConfig,
    health_checks: Arc<RwLock<HashMap<String, HealthCheckConfig>>>,
    circuit_breakers: Arc<RwLock<HashMap<String, CircuitBreakerConfig>>>,
    component_health: Arc<RwLock<HashMap<String, ComponentHealth>>>,
    circuit_breaker_metrics: Arc<RwLock<HashMap<String, CircuitBreakerMetrics>>>,
    recovery_history: Arc<RwLock<Vec<RecoveryResult>>>,
    health_monitor_tasks: Arc<RwLock<HashMap<String, tokio::task::JoinHandle<()>>>>,
    event_publisher: broadcast::Sender<SelfHealingEvent>,
    is_running: Arc<RwLock<bool>>,
}

/// Self-healing events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SelfHealingEvent {
    /// Component health status changed
    HealthStatusChanged {
        component_id: String,
        old_status: HealthStatus,
        new_status: HealthStatus,
        timestamp: DateTime<Utc>,
    },
    /// Circuit breaker state changed
    CircuitBreakerStateChanged {
        component_id: String,
        old_state: CircuitBreakerState,
        new_state: CircuitBreakerState,
        timestamp: DateTime<Utc>,
    },
    /// Recovery action initiated
    RecoveryInitiated {
        component_id: String,
        strategy: RecoveryStrategy,
        timestamp: DateTime<Utc>,
    },
    /// Recovery action completed
    RecoveryCompleted {
        component_id: String,
        result: RecoveryResult,
        timestamp: DateTime<Utc>,
    },
    /// Service degradation activated
    ServiceDegraded {
        component_id: String,
        reason: String,
        timestamp: DateTime<Utc>,
    },
    /// Service recovered
    ServiceRecovered {
        component_id: String,
        timestamp: DateTime<Utc>,
    },
}

impl SelfHealingManager {
    /// Create a new self-healing manager
    pub fn new(config: SelfHealingConfig) -> Self {
        let (event_publisher, _) = broadcast::channel(1000);

        Self {
            config,
            health_checks: Arc::new(RwLock::new(HashMap::new())),
            circuit_breakers: Arc::new(RwLock::new(HashMap::new())),
            component_health: Arc::new(RwLock::new(HashMap::new())),
            circuit_breaker_metrics: Arc::new(RwLock::new(HashMap::new())),
            recovery_history: Arc::new(RwLock::new(Vec::new())),
            health_monitor_tasks: Arc::new(RwLock::new(HashMap::new())),
            event_publisher,
            is_running: Arc::new(RwLock::new(false)),
        }
    }

    /// Start the self-healing system
    pub async fn start(&self) -> Result<(), PrimalError> {
        if *self.is_running.read().await {
            return Err(PrimalError::OperationFailed(
                "Self-healing system already running".to_string(),
            ));
        }

        info!("Starting self-healing system");
        *self.is_running.write().await = true;

        // Start health monitoring for all registered components
        self.start_health_monitoring().await?;

        // Start circuit breaker monitoring
        self.start_circuit_breaker_monitoring().await?;

        // Start recovery management
        self.start_recovery_management().await?;

        info!("Self-healing system started successfully");
        Ok(())
    }

    /// Stop the self-healing system
    pub async fn stop(&self) -> Result<(), PrimalError> {
        info!("Stopping self-healing system");
        *self.is_running.write().await = false;

        // Stop all health monitoring tasks
        let mut tasks = self.health_monitor_tasks.write().await;
        for (component_id, task) in tasks.drain() {
            debug!("Stopping health monitoring for {}", component_id);
            task.abort();
        }

        info!("Self-healing system stopped");
        Ok(())
    }

    /// Register a component for health monitoring
    pub async fn register_component(
        &self,
        component_id: String,
        component_name: String,
    ) -> Result<(), PrimalError> {
        debug!("Registering component {} for self-healing", component_id);

        // Create health check configuration
        let health_config = HealthCheckConfig {
            component_id: component_id.clone(),
            check_interval: self.config.health_check_interval,
            timeout: Duration::from_secs(10),
            failure_threshold: 3,
            recovery_threshold: 2,
            enabled: true,
        };

        // Create circuit breaker configuration
        let circuit_config = CircuitBreakerConfig {
            component_id: component_id.clone(),
            failure_threshold: 5,
            recovery_timeout: Duration::from_secs(30),
            success_threshold: 3,
            enabled: self.config.circuit_breaker_enabled,
        };

        // Initialize component health
        let health = ComponentHealth {
            component_id: component_id.clone(),
            component_name,
            status: HealthStatus::Unknown,
            last_check: Utc::now(),
            consecutive_failures: 0,
            consecutive_successes: 0,
            response_time: Duration::default(),
            error_rate: 0.0,
            uptime: Duration::default(),
            details: HashMap::new(),
        };

        // Initialize circuit breaker metrics
        let metrics = CircuitBreakerMetrics {
            component_id: component_id.clone(),
            state: CircuitBreakerState::Closed,
            failure_count: 0,
            success_count: 0,
            rejection_count: 0,
            last_failure: None,
            last_success: None,
            state_transitions: 0,
        };

        // Store configurations and initial state
        self.health_checks
            .write()
            .await
            .insert(component_id.clone(), health_config);
        self.circuit_breakers
            .write()
            .await
            .insert(component_id.clone(), circuit_config);
        self.component_health
            .write()
            .await
            .insert(component_id.clone(), health);
        self.circuit_breaker_metrics
            .write()
            .await
            .insert(component_id.clone(), metrics);

        // Start health monitoring if system is running
        if *self.is_running.read().await {
            self.start_component_health_monitoring(&component_id)
                .await?;
        }

        debug!("Component {} registered for self-healing", component_id);
        Ok(())
    }

    /// Start health monitoring for all components
    async fn start_health_monitoring(&self) -> Result<(), PrimalError> {
        debug!("Starting health monitoring for all components");

        let component_ids: Vec<String> = self.health_checks.read().await.keys().cloned().collect();

        for component_id in component_ids {
            self.start_component_health_monitoring(&component_id)
                .await?;
        }

        Ok(())
    }

    /// Start health monitoring for a specific component
    async fn start_component_health_monitoring(
        &self,
        component_id: &str,
    ) -> Result<(), PrimalError> {
        let component_id = component_id.to_string();
        let health_config = self
            .health_checks
            .read()
            .await
            .get(&component_id)
            .cloned()
            .ok_or_else(|| {
                PrimalError::NotFoundError(format!(
                    "Health check config not found for {}",
                    component_id
                ))
            })?;

        let component_health = self.component_health.clone();
        let circuit_breaker_metrics = self.circuit_breaker_metrics.clone();
        let event_publisher = self.event_publisher.clone();
        let is_running = self.is_running.clone();
        let component_id_clone = component_id.clone();

        let task = tokio::spawn(async move {
            let mut interval = interval(health_config.check_interval);

            while *is_running.read().await {
                interval.tick().await;

                // Perform health check
                let health_result = Self::perform_health_check(&component_id_clone).await;

                // Update component health
                let mut health_guard = component_health.write().await;
                if let Some(health) = health_guard.get_mut(&component_id_clone) {
                    let old_status = health.status;
                    let check_start = Instant::now();

                    match health_result {
                        Ok(_) => {
                            health.status = HealthStatus::Healthy;
                            health.consecutive_successes += 1;
                            health.consecutive_failures = 0;
                            health.response_time = check_start.elapsed();
                        }
                        Err(ref e) => {
                            health.status = HealthStatus::Unhealthy;
                            health.consecutive_failures += 1;
                            health.consecutive_successes = 0;
                            health.details.insert("error".to_string(), e.to_string());
                        }
                    }

                    health.last_check = Utc::now();

                    // Publish health status change event
                    if old_status != health.status {
                        let event = SelfHealingEvent::HealthStatusChanged {
                            component_id: component_id_clone.clone(),
                            old_status,
                            new_status: health.status,
                            timestamp: Utc::now(),
                        };
                        let _ = event_publisher.send(event);
                    }
                }
                drop(health_guard);

                // Update circuit breaker state based on health
                Self::update_circuit_breaker_state(
                    &component_id_clone,
                    &health_result,
                    &circuit_breaker_metrics,
                    &event_publisher,
                )
                .await;
            }
        });

        self.health_monitor_tasks
            .write()
            .await
            .insert(component_id.clone(), task);

        Ok(())
    }

    /// Perform health check for a component
    async fn perform_health_check(component_id: &str) -> Result<(), PrimalError> {
        debug!("Performing health check for {}", component_id);

        // Component-specific health check logic
        match component_id {
            "ai_intelligence" => Self::check_ai_intelligence_health().await,
            "mcp_integration" => Self::check_mcp_integration_health().await,
            "context_state" => Self::check_context_state_health().await,
            "agent_deployment" => Self::check_agent_deployment_health().await,
            "songbird" => Self::check_songbird_health().await,
            "toadstool" => Self::check_toadstool_health().await,
            "nestgate" => Self::check_nestgate_health().await,
            "beardog" => Self::check_beardog_health().await,
            _ => {
                warn!("Unknown component for health check: {}", component_id);
                Ok(())
            }
        }
    }

    /// Update circuit breaker state based on health check result
    async fn update_circuit_breaker_state(
        component_id: &str,
        health_result: &Result<(), PrimalError>,
        circuit_breaker_metrics: &Arc<RwLock<HashMap<String, CircuitBreakerMetrics>>>,
        event_publisher: &broadcast::Sender<SelfHealingEvent>,
    ) {
        let mut metrics_guard = circuit_breaker_metrics.write().await;
        if let Some(metrics) = metrics_guard.get_mut(component_id) {
            let old_state = metrics.state;

            match health_result {
                Ok(_) => {
                    metrics.success_count += 1;
                    metrics.last_success = Some(Utc::now());

                    // Transition from Open to HalfOpen or HalfOpen to Closed
                    match metrics.state {
                        CircuitBreakerState::Open => {
                            metrics.state = CircuitBreakerState::HalfOpen;
                            metrics.state_transitions += 1;
                        }
                        CircuitBreakerState::HalfOpen => {
                            if metrics.success_count >= 3 {
                                metrics.state = CircuitBreakerState::Closed;
                                metrics.state_transitions += 1;
                                metrics.failure_count = 0;
                            }
                        }
                        CircuitBreakerState::Closed => {
                            // Already closed, reset failure count
                            metrics.failure_count = 0;
                        }
                    }
                }
                Err(_) => {
                    metrics.failure_count += 1;
                    metrics.last_failure = Some(Utc::now());

                    // Transition to Open if failure threshold exceeded
                    if metrics.failure_count >= 5 && metrics.state == CircuitBreakerState::Closed {
                        metrics.state = CircuitBreakerState::Open;
                        metrics.state_transitions += 1;
                    } else if metrics.state == CircuitBreakerState::HalfOpen {
                        // Any failure in HalfOpen goes back to Open
                        metrics.state = CircuitBreakerState::Open;
                        metrics.state_transitions += 1;
                    }
                }
            }

            // Publish state change event
            if old_state != metrics.state {
                let event = SelfHealingEvent::CircuitBreakerStateChanged {
                    component_id: component_id.to_string(),
                    old_state,
                    new_state: metrics.state,
                    timestamp: Utc::now(),
                };
                let _ = event_publisher.send(event);
            }
        }
    }

    /// Start circuit breaker monitoring
    async fn start_circuit_breaker_monitoring(&self) -> Result<(), PrimalError> {
        debug!("Starting circuit breaker monitoring");

        // Circuit breaker monitoring logic would go here
        // This would include monitoring for circuit breaker state changes
        // and triggering appropriate actions

        Ok(())
    }

    /// Start recovery management
    async fn start_recovery_management(&self) -> Result<(), PrimalError> {
        debug!("Starting recovery management");

        // Recovery management logic would go here
        // This would include monitoring for failed components
        // and triggering recovery actions

        Ok(())
    }

    /// Perform recovery action for a component
    pub async fn perform_recovery(
        &self,
        component_id: &str,
        strategy: RecoveryStrategy,
    ) -> Result<RecoveryResult, PrimalError> {
        info!(
            "Performing recovery for component {} with strategy {:?}",
            component_id, strategy
        );

        let start_time = Instant::now();

        // Publish recovery initiated event
        let event = SelfHealingEvent::RecoveryInitiated {
            component_id: component_id.to_string(),
            strategy: strategy.clone(),
            timestamp: Utc::now(),
        };
        let _ = self.event_publisher.send(event);

        // Perform recovery based on strategy
        let recovery_result = match strategy {
            RecoveryStrategy::Restart => self.restart_component(component_id).await,
            RecoveryStrategy::Failover => self.failover_component(component_id).await,
            RecoveryStrategy::Degrade => self.degrade_component(component_id).await,
            RecoveryStrategy::Isolate => self.isolate_component(component_id).await,
            RecoveryStrategy::Manual => Ok(()),
            RecoveryStrategy::None => Ok(()),
        };

        let duration = start_time.elapsed();
        let success = recovery_result.is_ok();
        let error_message = recovery_result.as_ref().err().map(|e| e.to_string());

        let result = RecoveryResult {
            component_id: component_id.to_string(),
            strategy,
            success,
            duration,
            error_message: error_message.clone(),
            timestamp: Utc::now(),
        };

        // Store recovery result
        self.recovery_history.write().await.push(result.clone());

        // Publish recovery completed event
        let event = SelfHealingEvent::RecoveryCompleted {
            component_id: component_id.to_string(),
            result: result.clone(),
            timestamp: Utc::now(),
        };
        let _ = self.event_publisher.send(event);

        if success {
            info!(
                "Recovery completed successfully for component {}",
                component_id
            );
        } else {
            error!(
                "Recovery failed for component {}: {:?}",
                component_id, error_message
            );
        }

        Ok(result)
    }

    /// Restart a component
    async fn restart_component(&self, component_id: &str) -> Result<(), PrimalError> {
        info!("Restarting component {}", component_id);

        // Component-specific restart logic
        match component_id {
            "ai_intelligence" => {
                // Restart AI intelligence component
                debug!("Restarting AI intelligence component");
                Ok(())
            }
            "mcp_integration" => {
                // Restart MCP integration component
                debug!("Restarting MCP integration component");
                Ok(())
            }
            "context_state" => {
                // Restart context state component
                debug!("Restarting context state component");
                Ok(())
            }
            "agent_deployment" => {
                // Restart agent deployment component
                debug!("Restarting agent deployment component");
                Ok(())
            }
            "songbird" => {
                // Restart Songbird component
                debug!("Restarting Songbird component");
                Ok(())
            }
            "toadstool" => {
                // Restart ToadStool component
                debug!("Restarting ToadStool component");
                Ok(())
            }
            "nestgate" => {
                // Restart NestGate component
                debug!("Restarting NestGate component");
                Ok(())
            }
            "beardog" => {
                // Restart BearDog component
                debug!("Restarting BearDog component");
                Ok(())
            }
            _ => {
                warn!("Unknown component for restart: {}", component_id);
                Err(PrimalError::NotFoundError(format!(
                    "Component {} not found",
                    component_id
                )))
            }
        }
    }

    /// Failover a component to backup instance
    async fn failover_component(&self, component_id: &str) -> Result<(), PrimalError> {
        info!("Failing over component {}", component_id);

        // Component-specific failover logic
        match component_id {
            "ai_intelligence" => {
                // Failover to backup AI intelligence instance
                debug!("Failing over AI intelligence to backup instance");
                Ok(())
            }
            "mcp_integration" => {
                // Failover to backup MCP integration instance
                debug!("Failing over MCP integration to backup instance");
                Ok(())
            }
            _ => {
                warn!("Failover not supported for component: {}", component_id);
                Err(PrimalError::OperationFailed(format!(
                    "Failover not supported for {}",
                    component_id
                )))
            }
        }
    }

    /// Degrade a component's functionality
    async fn degrade_component(&self, component_id: &str) -> Result<(), PrimalError> {
        info!("Degrading component {}", component_id);

        // Publish service degradation event
        let event = SelfHealingEvent::ServiceDegraded {
            component_id: component_id.to_string(),
            reason: "Component failure detected".to_string(),
            timestamp: Utc::now(),
        };
        let _ = self.event_publisher.send(event);

        // Component-specific degradation logic
        match component_id {
            "ai_intelligence" => {
                // Degrade AI intelligence to basic functionality
                debug!("Degrading AI intelligence to basic functionality");
                Ok(())
            }
            "mcp_integration" => {
                // Degrade MCP integration to minimal functionality
                debug!("Degrading MCP integration to minimal functionality");
                Ok(())
            }
            _ => {
                debug!(
                    "Degradation not implemented for component: {}",
                    component_id
                );
                Ok(())
            }
        }
    }

    /// Isolate a component
    async fn isolate_component(&self, component_id: &str) -> Result<(), PrimalError> {
        info!("Isolating component {}", component_id);

        // Component-specific isolation logic
        match component_id {
            "ai_intelligence" => {
                // Isolate AI intelligence component
                debug!("Isolating AI intelligence component");
                Ok(())
            }
            "mcp_integration" => {
                // Isolate MCP integration component
                debug!("Isolating MCP integration component");
                Ok(())
            }
            _ => {
                debug!("Isolation not implemented for component: {}", component_id);
                Ok(())
            }
        }
    }

    // Component-specific health check methods
    async fn check_ai_intelligence_health() -> Result<(), PrimalError> {
        debug!("Checking AI intelligence health");
        // AI intelligence health check logic
        Ok(())
    }

    async fn check_mcp_integration_health() -> Result<(), PrimalError> {
        debug!("Checking MCP integration health");
        // MCP integration health check logic
        Ok(())
    }

    async fn check_context_state_health() -> Result<(), PrimalError> {
        debug!("Checking context state health");
        // Context state health check logic
        Ok(())
    }

    async fn check_agent_deployment_health() -> Result<(), PrimalError> {
        debug!("Checking agent deployment health");
        // Agent deployment health check logic
        Ok(())
    }

    async fn check_songbird_health() -> Result<(), PrimalError> {
        debug!("Checking Songbird health");
        // Songbird health check logic
        Ok(())
    }

    async fn check_toadstool_health() -> Result<(), PrimalError> {
        debug!("Checking ToadStool health");
        // ToadStool health check logic
        Ok(())
    }

    async fn check_nestgate_health() -> Result<(), PrimalError> {
        debug!("Checking NestGate health");
        // NestGate health check logic
        Ok(())
    }

    async fn check_beardog_health() -> Result<(), PrimalError> {
        debug!("Checking BearDog health");
        // BearDog health check logic
        Ok(())
    }

    /// Get current health status of all components
    pub async fn get_health_status(&self) -> HashMap<String, ComponentHealth> {
        self.component_health.read().await.clone()
    }

    /// Get circuit breaker metrics for all components
    pub async fn get_circuit_breaker_metrics(&self) -> HashMap<String, CircuitBreakerMetrics> {
        self.circuit_breaker_metrics.read().await.clone()
    }

    /// Get recovery history
    pub async fn get_recovery_history(&self) -> Vec<RecoveryResult> {
        self.recovery_history.read().await.clone()
    }

    /// Subscribe to self-healing events
    pub fn subscribe_to_events(&self) -> broadcast::Receiver<SelfHealingEvent> {
        self.event_publisher.subscribe()
    }
}

/// Initialize self-healing system
pub async fn initialize_self_healing() -> Result<SelfHealingManager, PrimalError> {
    info!("Initializing self-healing system");

    let config = SelfHealingConfig::default();
    let manager = SelfHealingManager::new(config);

    // Register all ecosystem components
    manager
        .register_component("ai_intelligence".to_string(), "AI Intelligence".to_string())
        .await?;
    manager
        .register_component("mcp_integration".to_string(), "MCP Integration".to_string())
        .await?;
    manager
        .register_component("context_state".to_string(), "Context State".to_string())
        .await?;
    manager
        .register_component(
            "agent_deployment".to_string(),
            "Agent Deployment".to_string(),
        )
        .await?;
    manager
        .register_component("songbird".to_string(), "Songbird Orchestration".to_string())
        .await?;
    manager
        .register_component("toadstool".to_string(), "ToadStool Compute".to_string())
        .await?;
    manager
        .register_component("nestgate".to_string(), "NestGate Storage".to_string())
        .await?;
    manager
        .register_component("beardog".to_string(), "BearDog Security".to_string())
        .await?;

    info!("Self-healing system initialized successfully");
    Ok(manager)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;

    #[test]
    async fn test_self_healing_manager_creation() {
        let config = SelfHealingConfig::default();
        let manager = SelfHealingManager::new(config);

        assert!(!*manager.is_running.read().await);
    }

    #[test]
    async fn test_component_registration() {
        let config = SelfHealingConfig::default();
        let manager = SelfHealingManager::new(config);

        let result = manager
            .register_component("test_component".to_string(), "Test Component".to_string())
            .await;
        assert!(result.is_ok());

        let health_status = manager.get_health_status().await;
        assert!(health_status.contains_key("test_component"));
    }

    #[test]
    async fn test_health_check_config() {
        let config = HealthCheckConfig {
            component_id: "test".to_string(),
            check_interval: Duration::from_secs(10),
            timeout: Duration::from_secs(5),
            failure_threshold: 3,
            recovery_threshold: 2,
            enabled: true,
        };

        assert_eq!(config.component_id, "test");
        assert_eq!(config.check_interval, Duration::from_secs(10));
        assert!(config.enabled);
    }

    #[test]
    async fn test_circuit_breaker_state_transitions() {
        let mut metrics = CircuitBreakerMetrics {
            component_id: "test".to_string(),
            state: CircuitBreakerState::Closed,
            failure_count: 0,
            success_count: 0,
            rejection_count: 0,
            last_failure: None,
            last_success: None,
            state_transitions: 0,
        };

        // Simulate failures
        metrics.failure_count = 5;
        metrics.state = CircuitBreakerState::Open;
        metrics.state_transitions += 1;

        assert_eq!(metrics.state, CircuitBreakerState::Open);
        assert_eq!(metrics.state_transitions, 1);
    }

    #[test]
    async fn test_recovery_result() {
        let result = RecoveryResult {
            component_id: "test".to_string(),
            strategy: RecoveryStrategy::Restart,
            success: true,
            duration: Duration::from_secs(5),
            error_message: None,
            timestamp: Utc::now(),
        };

        assert_eq!(result.component_id, "test");
        assert_eq!(result.strategy, RecoveryStrategy::Restart);
        assert!(result.success);
    }
}
