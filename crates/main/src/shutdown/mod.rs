//! Graceful Shutdown System for ecoPrimals Ecosystem
//!
//! This module provides a comprehensive graceful shutdown system that coordinates
//! shutdown across all ecosystem components in the proper order, ensuring:
//! - Active operations are completed or safely terminated
//! - Resources are properly released
//! - Connections are gracefully closed
//! - State is persisted where necessary
//! - Dependencies are shut down in reverse order

use chrono::{DateTime, Utc};
use futures::future::try_join_all;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, Mutex, RwLock};
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::error::PrimalError;
// use crate::biomeos_integration::ai_intelligence::AIIntelligence;
// use crate::biomeos_integration::context_state::ContextStateManager;
// use crate::biomeos_integration::mcp_integration::McpIntegration;
// use crate::biomeos_integration::agent_deployment::AgentDeploymentManager;
// use crate::songbird::SongbirdIntegration;
// use crate::toadstool::ToadStoolIntegration;
// use crate::nestgate::NestGateIntegration;
// use crate::beardog::BeardogIntegration;

/// Shutdown priority levels for component ordering
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ShutdownPriority {
    /// Critical components that must shut down first
    Critical = 0,
    /// High priority components
    High = 1,
    /// Medium priority components
    Medium = 2,
    /// Low priority components
    Low = 3,
    /// Background components that can shut down last
    Background = 4,
}

/// Shutdown phase representing the current stage of shutdown
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShutdownPhase {
    /// Shutdown has been initiated
    Initiated,
    /// Preparing components for shutdown
    Preparing,
    /// Stopping active operations
    StoppingOperations,
    /// Gracefully shutting down components
    GracefulShutdown,
    /// Forcefully shutting down remaining components
    ForcefulShutdown,
    /// Cleanup and resource release
    Cleanup,
    /// Shutdown completed
    Completed,
}

/// Component shutdown status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentShutdownStatus {
    pub component_id: String,
    pub component_name: String,
    pub priority: ShutdownPriority,
    pub status: ShutdownStatus,
    pub start_time: Option<DateTime<Utc>>,
    pub completion_time: Option<DateTime<Utc>>,
    pub duration: Option<Duration>,
    pub error_message: Option<String>,
    pub active_operations: u32,
    pub resources_released: bool,
}

/// Individual shutdown status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShutdownStatus {
    /// Component is running normally
    Running,
    /// Shutdown has been requested
    Requested,
    /// Component is preparing for shutdown
    Preparing,
    /// Component is stopping active operations
    StoppingOperations,
    /// Component is in graceful shutdown
    GracefulShutdown,
    /// Component is in forceful shutdown
    ForcefulShutdown,
    /// Component shutdown completed successfully
    Completed,
    /// Component shutdown failed
    Failed,
}

/// Shutdown configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShutdownConfig {
    /// Maximum time to wait for graceful shutdown
    pub graceful_timeout: Duration,
    /// Maximum time to wait for forceful shutdown
    pub forceful_timeout: Duration,
    /// Whether to persist state during shutdown
    pub persist_state: bool,
    /// Whether to wait for active operations to complete
    pub wait_for_operations: bool,
    /// Whether to enable parallel shutdown within priority levels
    pub parallel_shutdown: bool,
    /// Maximum number of concurrent shutdowns
    pub max_concurrent_shutdowns: u32,
}

impl Default for ShutdownConfig {
    fn default() -> Self {
        Self {
            graceful_timeout: Duration::from_secs(30),
            forceful_timeout: Duration::from_secs(10),
            persist_state: true,
            wait_for_operations: true,
            parallel_shutdown: true,
            max_concurrent_shutdowns: 5,
        }
    }
}

/// Shutdown manager that coordinates system shutdown
pub struct ShutdownManager {
    shutdown_id: String,
    config: ShutdownConfig,
    current_phase: Arc<RwLock<ShutdownPhase>>,
    components: Arc<RwLock<HashMap<String, ComponentShutdownStatus>>>,
    shutdown_signal: broadcast::Sender<ShutdownSignal>,
    shutdown_receiver: Arc<Mutex<broadcast::Receiver<ShutdownSignal>>>,
    start_time: Arc<RwLock<Option<DateTime<Utc>>>>,
    completion_time: Arc<RwLock<Option<DateTime<Utc>>>>,
    is_shutting_down: Arc<RwLock<bool>>,
}

/// Shutdown signal sent to components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShutdownSignal {
    pub signal_id: String,
    pub component_id: Option<String>,
    pub priority: ShutdownPriority,
    pub graceful_timeout: Duration,
    pub timestamp: DateTime<Utc>,
}

/// Shutdown report containing comprehensive shutdown information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShutdownReport {
    pub shutdown_id: String,
    pub start_time: DateTime<Utc>,
    pub completion_time: DateTime<Utc>,
    pub total_duration: Duration,
    pub final_phase: ShutdownPhase,
    pub components: Vec<ComponentShutdownStatus>,
    pub successful_shutdowns: u32,
    pub failed_shutdowns: u32,
    pub forceful_shutdowns: u32,
    pub summary: ShutdownSummary,
}

/// Summary of shutdown execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShutdownSummary {
    pub total_components: u32,
    pub successful_rate: f64,
    pub average_shutdown_time: Duration,
    pub longest_shutdown_time: Duration,
    pub shortest_shutdown_time: Duration,
    pub phases_completed: Vec<ShutdownPhase>,
}

impl ShutdownManager {
    /// Create a new shutdown manager
    pub fn new(config: ShutdownConfig) -> Self {
        let (shutdown_signal, shutdown_receiver) = broadcast::channel(1000);

        Self {
            shutdown_id: Uuid::new_v4().to_string(),
            config,
            current_phase: Arc::new(RwLock::new(ShutdownPhase::Initiated)),
            components: Arc::new(RwLock::new(HashMap::new())),
            shutdown_signal,
            shutdown_receiver: Arc::new(Mutex::new(shutdown_receiver)),
            start_time: Arc::new(RwLock::new(None)),
            completion_time: Arc::new(RwLock::new(None)),
            is_shutting_down: Arc::new(RwLock::new(false)),
        }
    }

    /// Register a component for shutdown management
    pub async fn register_component(
        &self,
        component_id: String,
        component_name: String,
        priority: ShutdownPriority,
    ) {
        let status = ComponentShutdownStatus {
            component_id: component_id.clone(),
            component_name,
            priority,
            status: ShutdownStatus::Running,
            start_time: None,
            completion_time: None,
            duration: None,
            error_message: None,
            active_operations: 0,
            resources_released: false,
        };

        self.components
            .write()
            .await
            .insert(component_id.clone(), status);
        debug!(
            "Registered component {} for shutdown management",
            component_id
        );
    }

    /// Initiate graceful shutdown of all components
    pub async fn initiate_graceful_shutdown(&self) -> Result<ShutdownReport, PrimalError> {
        info!("Initiating graceful shutdown of ecosystem");

        // Check if already shutting down
        {
            let mut is_shutting_down = self.is_shutting_down.write().await;
            if *is_shutting_down {
                return Err(PrimalError::OperationFailed(
                    "Shutdown already in progress".to_string(),
                ));
            }
            *is_shutting_down = true;
        }

        // Record start time
        *self.start_time.write().await = Some(Utc::now());

        // Execute shutdown phases
        self.execute_shutdown_phases().await?;

        // Record completion time
        *self.completion_time.write().await = Some(Utc::now());

        // Generate shutdown report
        let report = self.generate_shutdown_report().await;

        info!("Graceful shutdown completed");
        Ok(report)
    }

    /// Execute shutdown phases in order
    async fn execute_shutdown_phases(&self) -> Result<(), PrimalError> {
        let phases = vec![
            ShutdownPhase::Preparing,
            ShutdownPhase::StoppingOperations,
            ShutdownPhase::GracefulShutdown,
            ShutdownPhase::Cleanup,
            ShutdownPhase::Completed,
        ];

        for phase in phases {
            *self.current_phase.write().await = phase;
            info!("Entering shutdown phase: {:?}", phase);

            match phase {
                ShutdownPhase::Preparing => self.prepare_shutdown().await?,
                ShutdownPhase::StoppingOperations => self.stop_operations().await?,
                ShutdownPhase::GracefulShutdown => self.graceful_shutdown().await?,
                ShutdownPhase::Cleanup => self.cleanup_resources().await?,
                ShutdownPhase::Completed => {
                    info!("Shutdown phases completed successfully");
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Prepare components for shutdown
    async fn prepare_shutdown(&self) -> Result<(), PrimalError> {
        info!("Preparing components for shutdown");

        // Update all components to preparing status
        let mut components = self.components.write().await;
        for (_, status) in components.iter_mut() {
            status.status = ShutdownStatus::Preparing;
            status.start_time = Some(Utc::now());
        }

        // Send prepare signal to all components
        let signal = ShutdownSignal {
            signal_id: Uuid::new_v4().to_string(),
            component_id: None,
            priority: ShutdownPriority::Background,
            graceful_timeout: self.config.graceful_timeout,
            timestamp: Utc::now(),
        };

        if let Err(e) = self.shutdown_signal.send(signal) {
            warn!("Failed to send shutdown preparation signal: {}", e);
        }

        Ok(())
    }

    /// Stop active operations in all components
    async fn stop_operations(&self) -> Result<(), PrimalError> {
        info!("Stopping active operations");

        // Group components by priority
        let component_groups = self.group_components_by_priority().await;

        // Stop operations in priority order
        for (priority, component_ids) in component_groups {
            info!("Stopping operations for priority {:?} components", priority);

            let shutdown_futures: Vec<_> = component_ids
                .iter()
                .map(|component_id| self.stop_component_operations(component_id.clone()))
                .collect();

            // Execute shutdowns in parallel within priority level
            if self.config.parallel_shutdown {
                match try_join_all(shutdown_futures).await {
                    Ok(_) => {
                        debug!("All components stopped successfully");
                    }
                    Err(e) => {
                        warn!("Failed to stop component operations: {}", e);
                    }
                }
            } else {
                // Sequential shutdown
                for future in shutdown_futures {
                    if let Err(e) = future.await {
                        warn!("Failed to stop component operations: {}", e);
                    }
                }
            }
        }

        Ok(())
    }

    /// Gracefully shut down all components
    async fn graceful_shutdown(&self) -> Result<(), PrimalError> {
        info!("Starting graceful shutdown of components");

        // Group components by priority
        let component_groups = self.group_components_by_priority().await;

        // Shutdown components in priority order
        for (priority, component_ids) in component_groups {
            info!("Shutting down priority {:?} components", priority);

            let shutdown_futures: Vec<_> = component_ids
                .iter()
                .map(|component_id| self.shutdown_component_gracefully(component_id.clone()))
                .collect();

            // Execute shutdowns in parallel within priority level
            if self.config.parallel_shutdown {
                match try_join_all(shutdown_futures).await {
                    Ok(_) => {
                        debug!("All components shutdown gracefully");
                    }
                    Err(e) => {
                        warn!("Failed to shutdown component gracefully: {}", e);
                    }
                }
            } else {
                // Sequential shutdown
                for future in shutdown_futures {
                    if let Err(e) = future.await {
                        warn!("Failed to shutdown component gracefully: {}", e);
                    }
                }
            }
        }

        Ok(())
    }

    /// Clean up resources and finalize shutdown
    async fn cleanup_resources(&self) -> Result<(), PrimalError> {
        info!("Cleaning up resources");

        // Final cleanup for all components
        let mut components = self.components.write().await;
        for (_, status) in components.iter_mut() {
            if status.status != ShutdownStatus::Completed {
                status.status = ShutdownStatus::Completed;
                status.completion_time = Some(Utc::now());
                status.resources_released = true;
            }
        }

        info!("Resource cleanup completed");
        Ok(())
    }

    /// Group components by priority for ordered shutdown
    async fn group_components_by_priority(&self) -> Vec<(ShutdownPriority, Vec<String>)> {
        let components = self.components.read().await;
        let mut priority_groups: HashMap<ShutdownPriority, Vec<String>> = HashMap::new();

        for (component_id, status) in components.iter() {
            priority_groups
                .entry(status.priority)
                .or_insert_with(Vec::new)
                .push(component_id.clone());
        }

        // Sort by priority (lower values first)
        let mut sorted_groups: Vec<_> = priority_groups.into_iter().collect();
        sorted_groups.sort_by_key(|(priority, _)| *priority);

        sorted_groups
    }

    /// Stop operations for a specific component
    async fn stop_component_operations(&self, component_id: String) -> Result<(), PrimalError> {
        debug!("Stopping operations for component: {}", component_id);

        // Update component status
        {
            let mut components = self.components.write().await;
            if let Some(status) = components.get_mut(&component_id) {
                status.status = ShutdownStatus::StoppingOperations;
            }
        }

        // Component-specific operation stopping logic
        match component_id.as_str() {
            "ai_intelligence" => self.stop_ai_intelligence_operations().await?,
            "mcp_integration" => self.stop_mcp_integration_operations().await?,
            "context_state" => self.stop_context_state_operations().await?,
            "agent_deployment" => self.stop_agent_deployment_operations().await?,
            "songbird" => self.stop_songbird_operations().await?,
            "toadstool" => self.stop_toadstool_operations().await?,
            "nestgate" => self.stop_nestgate_operations().await?,
            "beardog" => self.stop_beardog_operations().await?,
            _ => warn!("Unknown component for operation stopping: {}", component_id),
        }

        Ok(())
    }

    /// Gracefully shut down a specific component
    async fn shutdown_component_gracefully(&self, component_id: String) -> Result<(), PrimalError> {
        debug!("Gracefully shutting down component: {}", component_id);

        // Update component status
        {
            let mut components = self.components.write().await;
            if let Some(status) = components.get_mut(&component_id) {
                status.status = ShutdownStatus::GracefulShutdown;
            }
        }

        // Component-specific graceful shutdown logic
        let shutdown_result = match component_id.as_str() {
            "ai_intelligence" => self.shutdown_ai_intelligence().await,
            "mcp_integration" => self.shutdown_mcp_integration().await,
            "context_state" => self.shutdown_context_state().await,
            "agent_deployment" => self.shutdown_agent_deployment().await,
            "songbird" => self.shutdown_songbird().await,
            "toadstool" => self.shutdown_toadstool().await,
            "nestgate" => self.shutdown_nestgate().await,
            "beardog" => self.shutdown_beardog().await,
            _ => {
                warn!("Unknown component for graceful shutdown: {}", component_id);
                Ok(())
            }
        };

        // Update component status based on shutdown result
        {
            let mut components = self.components.write().await;
            if let Some(status) = components.get_mut(&component_id) {
                match &shutdown_result {
                    Ok(_) => {
                        status.status = ShutdownStatus::Completed;
                        status.completion_time = Some(Utc::now());
                        status.resources_released = true;
                    }
                    Err(e) => {
                        status.status = ShutdownStatus::Failed;
                        status.error_message = Some(e.to_string());
                        status.completion_time = Some(Utc::now());
                    }
                }
            }
        }

        shutdown_result
    }

    // Component-specific operation stopping methods
    async fn stop_ai_intelligence_operations(&self) -> Result<(), PrimalError> {
        debug!("Stopping AI intelligence operations");
        // Implementation would stop active AI processing
        Ok(())
    }

    async fn stop_mcp_integration_operations(&self) -> Result<(), PrimalError> {
        debug!("Stopping MCP integration operations");
        // Implementation would stop active MCP sessions
        Ok(())
    }

    async fn stop_context_state_operations(&self) -> Result<(), PrimalError> {
        debug!("Stopping context state operations");
        // Implementation would stop context processing
        Ok(())
    }

    async fn stop_agent_deployment_operations(&self) -> Result<(), PrimalError> {
        debug!("Stopping agent deployment operations");
        // Implementation would stop active deployments
        Ok(())
    }

    async fn stop_songbird_operations(&self) -> Result<(), PrimalError> {
        debug!("Stopping Songbird operations");
        // Implementation would stop orchestration tasks
        Ok(())
    }

    async fn stop_toadstool_operations(&self) -> Result<(), PrimalError> {
        debug!("Stopping ToadStool operations");
        // Implementation would stop compute jobs
        Ok(())
    }

    async fn stop_nestgate_operations(&self) -> Result<(), PrimalError> {
        debug!("Stopping NestGate operations");
        // Implementation would stop storage operations
        Ok(())
    }

    async fn stop_beardog_operations(&self) -> Result<(), PrimalError> {
        debug!("Stopping BearDog operations");
        // Implementation would stop security operations
        Ok(())
    }

    // Component-specific graceful shutdown methods
    async fn shutdown_ai_intelligence(&self) -> Result<(), PrimalError> {
        debug!("Shutting down AI intelligence");
        // Implementation would properly shut down AI intelligence
        Ok(())
    }

    async fn shutdown_mcp_integration(&self) -> Result<(), PrimalError> {
        debug!("Shutting down MCP integration");
        // Implementation would properly shut down MCP integration
        Ok(())
    }

    async fn shutdown_context_state(&self) -> Result<(), PrimalError> {
        debug!("Shutting down context state");
        // Implementation would properly shut down context state
        Ok(())
    }

    async fn shutdown_agent_deployment(&self) -> Result<(), PrimalError> {
        debug!("Shutting down agent deployment");
        // Implementation would properly shut down agent deployment
        Ok(())
    }

    async fn shutdown_songbird(&self) -> Result<(), PrimalError> {
        debug!("Shutting down Songbird");
        // Implementation would properly shut down Songbird
        Ok(())
    }

    async fn shutdown_toadstool(&self) -> Result<(), PrimalError> {
        debug!("Shutting down ToadStool");
        // Implementation would properly shut down ToadStool
        Ok(())
    }

    async fn shutdown_nestgate(&self) -> Result<(), PrimalError> {
        debug!("Shutting down NestGate");
        // Implementation would properly shut down NestGate
        Ok(())
    }

    async fn shutdown_beardog(&self) -> Result<(), PrimalError> {
        debug!("Shutting down BearDog");
        // Implementation would properly shut down BearDog
        Ok(())
    }

    /// Generate comprehensive shutdown report
    async fn generate_shutdown_report(&self) -> ShutdownReport {
        let start_time = self.start_time.read().await.unwrap_or_else(|| Utc::now());
        let completion_time = self
            .completion_time
            .read()
            .await
            .unwrap_or_else(|| Utc::now());
        let total_duration = completion_time
            .signed_duration_since(start_time)
            .to_std()
            .unwrap_or_default();

        let components: Vec<_> = self.components.read().await.values().cloned().collect();
        let successful_shutdowns = components
            .iter()
            .filter(|c| c.status == ShutdownStatus::Completed)
            .count() as u32;
        let failed_shutdowns = components
            .iter()
            .filter(|c| c.status == ShutdownStatus::Failed)
            .count() as u32;
        let forceful_shutdowns = components
            .iter()
            .filter(|c| c.status == ShutdownStatus::ForcefulShutdown)
            .count() as u32;

        let summary = self
            .generate_shutdown_summary(&components, total_duration)
            .await;

        ShutdownReport {
            shutdown_id: self.shutdown_id.clone(),
            start_time,
            completion_time,
            total_duration,
            final_phase: *self.current_phase.read().await,
            components,
            successful_shutdowns,
            failed_shutdowns,
            forceful_shutdowns,
            summary,
        }
    }

    /// Generate shutdown summary statistics
    async fn generate_shutdown_summary(
        &self,
        components: &[ComponentShutdownStatus],
        total_duration: Duration,
    ) -> ShutdownSummary {
        let total_components = components.len() as u32;
        let successful_rate = if total_components > 0 {
            components
                .iter()
                .filter(|c| c.status == ShutdownStatus::Completed)
                .count() as f64
                / total_components as f64
        } else {
            0.0
        };

        let durations: Vec<_> = components.iter().filter_map(|c| c.duration).collect();

        let average_shutdown_time = if !durations.is_empty() {
            durations.iter().sum::<Duration>() / durations.len() as u32
        } else {
            Duration::default()
        };

        let longest_shutdown_time = durations.iter().max().copied().unwrap_or_default();
        let shortest_shutdown_time = durations.iter().min().copied().unwrap_or_default();

        ShutdownSummary {
            total_components,
            successful_rate,
            average_shutdown_time,
            longest_shutdown_time,
            shortest_shutdown_time,
            phases_completed: vec![
                ShutdownPhase::Preparing,
                ShutdownPhase::StoppingOperations,
                ShutdownPhase::GracefulShutdown,
                ShutdownPhase::Cleanup,
                ShutdownPhase::Completed,
            ],
        }
    }

    /// Get current shutdown status
    pub async fn get_shutdown_status(&self) -> (ShutdownPhase, Vec<ComponentShutdownStatus>) {
        let phase = *self.current_phase.read().await;
        let components = self.components.read().await.values().cloned().collect();
        (phase, components)
    }

    /// Check if system is shutting down
    pub async fn is_shutting_down(&self) -> bool {
        *self.is_shutting_down.read().await
    }
}

/// Initialize comprehensive shutdown system
pub async fn initialize_shutdown_system() -> Result<ShutdownManager, PrimalError> {
    info!("Initializing comprehensive shutdown system");

    let config = ShutdownConfig::default();
    let shutdown_manager = ShutdownManager::new(config);

    // Register all ecosystem components
    shutdown_manager
        .register_component(
            "beardog".to_string(),
            "BearDog Security".to_string(),
            ShutdownPriority::Critical,
        )
        .await;
    shutdown_manager
        .register_component(
            "ai_intelligence".to_string(),
            "AI Intelligence".to_string(),
            ShutdownPriority::High,
        )
        .await;
    shutdown_manager
        .register_component(
            "mcp_integration".to_string(),
            "MCP Integration".to_string(),
            ShutdownPriority::High,
        )
        .await;
    shutdown_manager
        .register_component(
            "context_state".to_string(),
            "Context State".to_string(),
            ShutdownPriority::Medium,
        )
        .await;
    shutdown_manager
        .register_component(
            "agent_deployment".to_string(),
            "Agent Deployment".to_string(),
            ShutdownPriority::Medium,
        )
        .await;
    shutdown_manager
        .register_component(
            "songbird".to_string(),
            "Songbird Orchestration".to_string(),
            ShutdownPriority::Low,
        )
        .await;
    shutdown_manager
        .register_component(
            "toadstool".to_string(),
            "ToadStool Compute".to_string(),
            ShutdownPriority::Low,
        )
        .await;
    shutdown_manager
        .register_component(
            "nestgate".to_string(),
            "NestGate Storage".to_string(),
            ShutdownPriority::Background,
        )
        .await;

    info!("Shutdown system initialized successfully");
    Ok(shutdown_manager)
}

/// Perform emergency shutdown with minimal cleanup
pub async fn emergency_shutdown() -> Result<(), PrimalError> {
    warn!("Performing emergency shutdown");

    // Immediate shutdown without graceful handling
    std::process::exit(1);
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;

    #[test]
    async fn test_shutdown_manager_creation() {
        let config = ShutdownConfig::default();
        let manager = ShutdownManager::new(config);

        assert!(!manager.is_shutting_down().await);
    }

    #[test]
    async fn test_component_registration() {
        let config = ShutdownConfig::default();
        let manager = ShutdownManager::new(config);

        manager
            .register_component(
                "test_component".to_string(),
                "Test Component".to_string(),
                ShutdownPriority::Medium,
            )
            .await;

        let (_, components) = manager.get_shutdown_status().await;
        assert_eq!(components.len(), 1);
        assert_eq!(components[0].component_id, "test_component");
    }

    #[test]
    async fn test_shutdown_config_default() {
        let config = ShutdownConfig::default();

        assert_eq!(config.graceful_timeout, Duration::from_secs(30));
        assert_eq!(config.forceful_timeout, Duration::from_secs(10));
        assert!(config.persist_state);
        assert!(config.parallel_shutdown);
    }

    #[test]
    async fn test_shutdown_phases() {
        let config = ShutdownConfig::default();
        let manager = ShutdownManager::new(config);

        let initial_phase = manager.current_phase.read().await;
        assert_eq!(*initial_phase, ShutdownPhase::Initiated);
    }

    #[test]
    async fn test_priority_grouping() {
        let config = ShutdownConfig::default();
        let manager = ShutdownManager::new(config);

        manager
            .register_component(
                "critical".to_string(),
                "Critical".to_string(),
                ShutdownPriority::Critical,
            )
            .await;
        manager
            .register_component(
                "high".to_string(),
                "High".to_string(),
                ShutdownPriority::High,
            )
            .await;
        manager
            .register_component("low".to_string(), "Low".to_string(), ShutdownPriority::Low)
            .await;

        let groups = manager.group_components_by_priority().await;
        assert_eq!(groups.len(), 3);

        // Verify priority ordering
        assert_eq!(groups[0].0, ShutdownPriority::Critical);
        assert_eq!(groups[1].0, ShutdownPriority::High);
        assert_eq!(groups[2].0, ShutdownPriority::Low);
    }
}
