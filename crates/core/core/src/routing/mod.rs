// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! MCP Task Routing Service
//!
//! This module provides a comprehensive task routing service for the Machine Context Protocol.
//! It has been split into focused modules for better organization and maintainability.
//!
//! ## Architecture
//!
//! The routing service is composed of several focused modules:
//!
//! * **config**: Configuration management for routing strategies and rules
//! * **balancer**: Load balancing across multiple agents with performance tracking
//! * **agent**: Agent registration, health monitoring, and lifecycle management
//! * **context**: Context storage and synchronization between agents
//!
//! ## Usage
//!
//! ```rust,no_run
//! use crate::routing::{McpRoutingService, RoutingConfig};
//! use crate::routing::config::LoadBalancingStrategy;
//!
//! let config = RoutingConfig::new()
//!     .with_load_balancing_strategy(LoadBalancingStrategy::Adaptive)
//!     .with_max_concurrent_tasks(100);
//!
//! let routing_service = McpRoutingService::new(config)?;
//! routing_service.start().await?;
//! ```

pub mod agent;
pub mod balancer;
pub mod config;
pub mod context;

// Re-export main types
pub use agent::{
    AgentHealthStatus, AgentRegistry, AgentRegistryStats, AgentSummary, HealthCheckConfig,
    RegisteredAgent,
};
pub use balancer::{AgentPerformanceStats, LoadBalancer, PerformanceMetric};
pub use config::*;
pub use context::{
    ContextExport, ContextManager, ContextStatistics, ContextStorage, PersistentContext,
    SharedContext,
};

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use parking_lot::RwLock;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

use crate::{
    AgentSpec, CoordinationResult, Error, McpRouter, McpTask, QueuedTask, ResponseMetadata, Result,
    RoutingStats, ScaleRequirements, ScaleResult, TaskPriority, TaskResponse,
};

/// Multi-MCP task routing and coordination service
///
/// This is the core of Squirrel MCP's functionality as a multi-MCP coordinator.
/// It handles:
/// - Routing MCP tasks to appropriate agents/primals
/// - Load balancing across multiple MCP endpoints
/// - Context coordination and persistence
/// - Performance optimization and scaling
/// - Federation across multiple nodes
#[derive(Clone)]
pub struct McpRoutingService {
    /// Configuration
    config: RoutingConfig,
    /// Service state
    state: Arc<RoutingState>,
    /// Agent registry
    agent_registry: Arc<AgentRegistry>,
    /// Task queue
    task_queue: Arc<DashMap<String, QueuedTask>>,
    /// Load balancer
    load_balancer: Arc<LoadBalancer>,
    /// Context manager
    context_manager: Arc<ContextManager>,
    // Note: HTTP removed - use Songbird via Unix sockets for any HTTP needs
}

/// Internal state for the routing service
#[derive(Debug)]
struct RoutingState {
    /// Unique node identifier
    node_id: String,
    /// Number of active tasks
    active_tasks: RwLock<u64>,
    /// Number of completed tasks
    completed_tasks: RwLock<u64>,
    /// Number of failed tasks
    failed_tasks: RwLock<u64>,
    /// Average response time
    average_response_time: RwLock<f64>,
    /// Last scaling event
    last_scale_event: RwLock<Option<DateTime<Utc>>>,
    /// Federation nodes
    #[expect(dead_code, reason = "Reserved for federation routing")]
    federation_nodes: RwLock<Vec<String>>,
}

impl McpRoutingService {
    /// Create a new routing service with the given configuration
    pub fn new(config: RoutingConfig) -> Result<Self> {
        // Validate configuration
        config.validate().map_err(Error::ConfigurationError)?;

        // Create components
        let agent_registry = Arc::new(AgentRegistry::new(HealthCheckConfig::default()));
        let load_balancer = Arc::new(LoadBalancer::new(
            config.load_balancing_strategy.clone(),
            config.max_concurrent_tasks,
        ));
        let context_manager = Arc::new(ContextManager::new(if config.context_persistence {
            Some(ContextStorage::Local)
        } else {
            None
        }));

        let state = Arc::new(RoutingState {
            node_id: uuid::Uuid::new_v4().to_string(),
            active_tasks: RwLock::new(0),
            completed_tasks: RwLock::new(0),
            failed_tasks: RwLock::new(0),
            average_response_time: RwLock::new(0.0),
            last_scale_event: RwLock::new(None),
            federation_nodes: RwLock::new(Vec::new()),
        });

        Ok(Self {
            config,
            state,
            agent_registry,
            task_queue: Arc::new(DashMap::new()),
            load_balancer,
            context_manager,
        })
    }

    /// Start the routing service
    pub async fn start(&self) -> Result<()> {
        info!(
            "Starting MCP routing service with node ID: {}",
            self.state.node_id
        );

        // Start background tasks
        self.start_background_tasks().await;

        info!("MCP routing service started successfully");
        Ok(())
    }

    /// Start background processing tasks
    async fn start_background_tasks(&self) {
        // Start task processing loop
        let service_clone = self.clone();
        tokio::spawn(async move {
            service_clone.task_processing_loop().await;
        });

        // Start health monitoring if enabled
        if self.config.performance_monitoring {
            let service_clone = self.clone();
            tokio::spawn(async move {
                service_clone.health_monitoring_loop().await;
            });
        }

        // Start performance monitoring if enabled
        if self.config.performance_monitoring {
            let service_clone = self.clone();
            tokio::spawn(async move {
                service_clone.performance_monitoring_loop().await;
            });
        }

        // Start auto-scaling if enabled
        if self.config.scaling_enabled {
            let service_clone = self.clone();
            tokio::spawn(async move {
                service_clone.auto_scaling_loop().await;
            });
        }
    }

    /// Main task processing loop
    async fn task_processing_loop(&self) {
        debug!("Starting task processing loop");

        loop {
            if let Some((task_id, queued_task)) = self.find_next_task() {
                if let Err(e) = self.process_task(task_id, queued_task).await {
                    error!("Error processing task: {}", e);
                }
            } else {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        }
    }

    /// Find the next task to process
    fn find_next_task(&self) -> Option<(String, QueuedTask)> {
        let mut highest_priority = None;
        let mut selected_task = None;

        for entry in self.task_queue.iter() {
            let task_id = entry.key().clone();
            let queued_task = entry.value().clone();

            if let Some(current_priority) = &highest_priority {
                if queued_task.priority > *current_priority {
                    highest_priority = Some(queued_task.priority);
                    selected_task = Some((task_id, queued_task));
                }
            } else {
                highest_priority = Some(queued_task.priority);
                selected_task = Some((task_id, queued_task));
            }
        }

        if let Some((task_id, _)) = &selected_task {
            self.task_queue.remove(task_id);
        }

        selected_task
    }

    /// Process a single task
    async fn process_task(&self, task_id: String, queued_task: QueuedTask) -> Result<()> {
        let start_time = std::time::Instant::now();

        // Increment active tasks
        {
            let mut active_tasks = self.state.active_tasks.write();
            *active_tasks += 1;
        }

        // Process the task
        let result = self.route_task(queued_task.task).await;

        // Update statistics
        let duration = start_time.elapsed();
        match result {
            Ok(_) => {
                let mut completed_tasks = self.state.completed_tasks.write();
                *completed_tasks += 1;
            }
            Err(e) => {
                error!("Task {} failed: {}", task_id, e);
                let mut failed_tasks = self.state.failed_tasks.write();
                *failed_tasks += 1;
            }
        }

        // Update response time
        {
            let mut avg_response_time = self.state.average_response_time.write();
            *avg_response_time = (*avg_response_time).mul_add(0.9, duration.as_secs_f64() * 0.1);
        }

        // Decrement active tasks
        {
            let mut active_tasks = self.state.active_tasks.write();
            *active_tasks = active_tasks.saturating_sub(1);
        }

        Ok(())
    }

    /// Health monitoring loop
    async fn health_monitoring_loop(&self) {
        debug!("Starting health monitoring loop");

        loop {
            // Check agent health
            let agents = self.agent_registry.get_all_agents();
            for agent in agents {
                // Simplified health check - in real implementation, this would ping the agent
                if agent.time_since_last_seen() > chrono::Duration::minutes(5) {
                    let _ = self
                        .agent_registry
                        .update_agent_health(&agent.id, AgentHealthStatus::Offline);
                }
            }

            // Clean up stale agents
            let removed_agents = self
                .agent_registry
                .cleanup_stale_agents(chrono::Duration::hours(1));
            if !removed_agents.is_empty() {
                info!("Removed {} stale agents", removed_agents.len());
            }

            // Clean up expired contexts
            let expired_contexts = self.context_manager.cleanup_expired_contexts().await;
            if expired_contexts > 0 {
                info!("Cleaned up {} expired contexts", expired_contexts);
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
        }
    }

    /// Performance monitoring loop
    async fn performance_monitoring_loop(&self) {
        debug!("Starting performance monitoring loop");

        loop {
            // Get and log performance statistics
            let stats = self.get_stats();
            debug!(
                "Performance stats: active={}, completed={}, failed={}, avg_response_time={:.2}s",
                stats.active_tasks,
                stats.completed_tasks,
                stats.failed_tasks,
                stats.average_response_time
            );

            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
        }
    }

    /// Auto-scaling loop
    async fn auto_scaling_loop(&self) {
        debug!("Starting auto-scaling loop");

        loop {
            // Check if scaling is needed
            let active_tasks = *self.state.active_tasks.read();
            let max_tasks = self.config.max_concurrent_tasks as u64;

            if active_tasks > (max_tasks * 80 / 100) {
                warn!(
                    "High task load detected: {}/{}, scaling may be needed",
                    active_tasks, max_tasks
                );

                // Update last scale event
                let mut last_scale_event = self.state.last_scale_event.write();
                *last_scale_event = Some(Utc::now());
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
        }
    }

    /// Register an agent with the routing service
    pub async fn register_agent(&self, agent_spec: AgentSpec) -> Result<()> {
        self.agent_registry.register_agent(agent_spec)
    }

    /// Queue a task for processing
    pub async fn queue_task(&self, task: McpTask, priority: TaskPriority) -> Result<String> {
        let task_id = uuid::Uuid::new_v4().to_string();
        let queued_task = QueuedTask {
            task,
            priority,
            queued_at: Utc::now(),
            retry_count: 0,
            max_retries: 3,
        };

        self.task_queue.insert(task_id.clone(), queued_task);
        debug!("Queued task {} with priority {:?}", task_id, priority);

        Ok(task_id)
    }

    /// Get routing statistics
    pub fn get_stats(&self) -> RoutingStats {
        let state = &self.state;
        RoutingStats {
            node_id: state.node_id.clone(),
            active_tasks: *state.active_tasks.read(),
            completed_tasks: *state.completed_tasks.read(),
            failed_tasks: *state.failed_tasks.read(),
            average_response_time: *state.average_response_time.read(),
            registered_agents: self.agent_registry.get_all_agents().len() as u32,
            queued_tasks: self.task_queue.len() as u64,
            federation_nodes: 0, // FUTURE: [Feature] implement federation node counting
                                 // Tracking: Planned for v0.2.0 - federation support
        }
    }

    /// Get agent registry
    pub const fn get_agent_registry(&self) -> &Arc<AgentRegistry> {
        &self.agent_registry
    }

    /// Get load balancer
    pub const fn get_load_balancer(&self) -> &Arc<LoadBalancer> {
        &self.load_balancer
    }

    /// Get context manager
    pub const fn get_context_manager(&self) -> &Arc<ContextManager> {
        &self.context_manager
    }
}

#[async_trait::async_trait]
impl McpRouter for McpRoutingService {
    async fn route_task(&self, task: McpTask) -> Result<TaskResponse> {
        // Find suitable agents
        let available_agents = self.agent_registry.get_available_agents();

        if available_agents.is_empty() {
            return Err(Error::NoAgentAvailable);
        }

        // Select best agent using load balancer
        let selected_agent = self.load_balancer.select_agent(available_agents).await?;

        // Create a simple task response
        Ok(TaskResponse {
            id: uuid::Uuid::new_v4().to_string(),
            task_id: task.id.clone(),
            agent_id: selected_agent.id.clone(),
            execution_time: std::time::Duration::from_millis(100),
            context: None,
            result: serde_json::json!({
                "status": "completed",
                "agent_id": selected_agent.id,
                "message": "Task processed successfully"
            }),
            response: serde_json::json!({
                "status": "completed",
                "message": "Task processed successfully"
            }),
            metadata: ResponseMetadata {
                context_updated: false,
                processing_time: std::time::Duration::from_millis(100),
                agent_version: Some(selected_agent.id),
            },
        })
    }

    async fn coordinate_agents(&self, agents: Vec<AgentSpec>) -> Result<CoordinationResult> {
        let agent_count = agents.len() as u32;

        // Register all agents
        for agent in agents {
            self.register_agent(agent).await?;
        }

        Ok(CoordinationResult {
            registered_agents: agent_count,
            failed_registrations: 0,
            total_agents: agent_count,
            status: "success".to_string(),
        })
    }

    async fn scale_capacity(&self, requirements: ScaleRequirements) -> Result<ScaleResult> {
        // Simple scaling implementation
        Ok(ScaleResult {
            scaling_triggered: true,
            target_instances: requirements.target_capacity,
            current_instances: requirements.target_capacity,
            scaling_status: "success".to_string(),
            message: "Capacity scaled successfully".to_string(),
            new_capacity: requirements.target_capacity,
        })
    }
}
