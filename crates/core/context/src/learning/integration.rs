// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Learning Integration Layer
//!
//! This module provides the integration layer that connects the learning system
//! with the existing Context Management System components.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, error, info};

use super::{
    AdaptiveRuleSystem, ContextLearningManager, LearningEngine, LearningMetrics,
    LearningSystemConfig, PolicyNetwork, RewardSystem,
};
use crate::error::Result;
use crate::manager::ContextManager;
use crate::rules::RuleManager;
use crate::visualization::VisualizationSystem;

// Re-export planned feature types (available for downstream consumers)
#[expect(unused_imports, reason = "re-export for planned consumer")]
pub use super::integration_types::{
    ContextUsagePattern, LearningRequest, LearningRequestType, StateChange,
    StateChangePatternAnalysis, analyze_state_change_patterns,
};

/// Context monitoring results for tracking
///
/// Note: Planned feature for context monitoring - implementation in progress
#[derive(Debug, Clone)]
#[expect(dead_code, reason = "planned feature not yet wired")]
pub struct ContextMonitoringResults {
    pub total_contexts: usize,
    pub contexts_needing_intervention: usize,
    pub monitoring_timestamp: chrono::DateTime<chrono::Utc>,
}

/// Learning integration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningIntegrationConfig {
    /// Enable context manager integration
    pub enable_context_manager: bool,

    /// Enable rule manager integration
    pub enable_rule_manager: bool,

    /// Enable visualization integration
    pub enable_visualization: bool,

    /// Integration update interval
    pub update_interval: std::time::Duration,

    /// Enable automatic learning triggers
    pub enable_auto_triggers: bool,

    /// Learning trigger thresholds
    pub trigger_thresholds: TriggerThresholds,
}

impl Default for LearningIntegrationConfig {
    fn default() -> Self {
        Self {
            enable_context_manager: true,
            enable_rule_manager: true,
            enable_visualization: true,
            update_interval: std::time::Duration::from_secs(30),
            enable_auto_triggers: true,
            trigger_thresholds: TriggerThresholds::default(),
        }
    }
}

/// Trigger thresholds for automatic learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerThresholds {
    /// Minimum context changes to trigger learning
    pub min_context_changes: usize,

    /// Minimum rule applications to trigger adaptation
    pub min_rule_applications: usize,

    /// Error rate threshold for learning trigger
    pub error_rate_threshold: f64,

    /// Performance degradation threshold
    pub performance_threshold: f64,
}

impl Default for TriggerThresholds {
    fn default() -> Self {
        Self {
            min_context_changes: 10,
            min_rule_applications: 5,
            error_rate_threshold: 0.2,
            performance_threshold: 0.7,
        }
    }
}

/// Learning integration layer
#[derive(Debug)]
pub struct LearningIntegration {
    /// System configuration
    config: Arc<LearningIntegrationConfig>,

    /// Context manager reference
    context_manager: Option<Arc<ContextManager>>,

    /// Rule manager reference
    rule_manager: Option<Arc<RuleManager>>,

    /// Visualization system reference
    visualization_system: Option<Arc<VisualizationSystem>>,

    /// Learning engine reference
    learning_engine: Option<Arc<LearningEngine>>,

    /// Context learning manager reference
    context_learning_manager: Option<Arc<ContextLearningManager>>,

    /// Reward system reference
    reward_system: Option<Arc<RewardSystem>>,

    /// Policy network reference
    policy_network: Option<Arc<PolicyNetwork>>,

    /// Learning metrics reference
    learning_metrics: Option<Arc<LearningMetrics>>,

    /// Adaptive rule system reference
    adaptive_rule_system: Option<Arc<AdaptiveRuleSystem>>,

    /// Integration state
    state: Arc<RwLock<IntegrationState>>,

    /// Integration statistics
    stats: Arc<Mutex<IntegrationStats>>,

    /// Background task handles
    task_handles: Arc<Mutex<Vec<tokio::task::JoinHandle<()>>>>,
}

/// Integration state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationState {
    /// Integration status
    pub status: IntegrationStatus,

    /// Last integration update
    pub last_update: DateTime<Utc>,

    /// Active integrations
    pub active_integrations: Vec<String>,

    /// Integration errors
    pub errors: Vec<IntegrationError>,
}

/// Integration status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IntegrationStatus {
    /// Integration is initializing
    Initializing,

    /// Integration is active
    Active,

    /// Integration is paused
    Paused,

    /// Integration is stopped
    Stopped,

    /// Integration has errors
    Error,
}

/// Integration error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationError {
    /// Error ID
    pub id: String,

    /// Error type
    pub error_type: String,

    /// Error message
    pub message: String,

    /// Error timestamp
    pub timestamp: DateTime<Utc>,

    /// Component that caused the error
    pub component: String,
}

/// Integration statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationStats {
    /// Total integration operations
    pub total_operations: usize,

    /// Successful operations
    pub successful_operations: usize,

    /// Failed operations
    pub failed_operations: usize,

    /// Context synchronizations
    pub context_syncs: usize,

    /// Rule adaptations triggered
    pub rule_adaptations: usize,

    /// Learning episodes triggered
    pub learning_episodes: usize,

    /// Average operation time
    pub average_operation_time: f64,

    /// Last operation time
    pub last_operation: DateTime<Utc>,
}

impl Default for IntegrationStats {
    fn default() -> Self {
        Self {
            total_operations: 0,
            successful_operations: 0,
            failed_operations: 0,
            context_syncs: 0,
            rule_adaptations: 0,
            learning_episodes: 0,
            average_operation_time: 0.0,
            last_operation: Utc::now(),
        }
    }
}

impl LearningIntegration {
    /// Create a new learning integration layer
    pub async fn new(system_config: Arc<LearningSystemConfig>) -> Result<Self> {
        // Create enhanced configuration based on system configuration
        let integration_config = Self::build_integration_config(&system_config).await?;
        let config = Arc::new(integration_config);

        // Initialize state based on system configuration
        let initial_state = Self::determine_initial_state(&system_config);

        // Pre-configure component managers based on system settings
        let (context_manager, rule_manager) = Self::initialize_managers(&system_config).await?;

        // Setup visualization system if enabled in configuration
        let visualization_system = if system_config.enable_learning_metrics {
            let vis_config = crate::visualization::VisualizationSystemConfig::default();
            Some(Arc::new(
                VisualizationSystem::new(vis_config).await.map_err(|e| {
                    crate::error::ContextError::InitializationFailed(format!(
                        "Visualization setup failed: {e}"
                    ))
                })?,
            ))
        } else {
            debug!("Visualization disabled in system configuration");
            None
        };

        // Configure learning engine with system-specific parameters
        let learning_engine = if system_config.enable_reinforcement_learning {
            let engine_config = system_config.clone();
            Some(Arc::new(LearningEngine::new(engine_config).await.map_err(
                |e| {
                    crate::error::ContextError::InitializationFailed(format!(
                        "Learning engine setup failed: {e}"
                    ))
                },
            )?))
        } else {
            debug!("Learning engine disabled in system configuration");
            None
        };

        // Initialize metrics collection based on configuration
        let learning_metrics = if system_config.enable_learning_metrics {
            let metrics_config = system_config.clone();
            Some(Arc::new(
                LearningMetrics::new(metrics_config).await.map_err(|e| {
                    crate::error::ContextError::InitializationFailed(format!(
                        "Learning metrics setup failed: {e}"
                    ))
                })?,
            ))
        } else {
            debug!("Learning metrics disabled in system configuration");
            None
        };

        // Configure adaptive rule system with system parameters
        let adaptive_rule_system = if system_config.enable_adaptive_rules {
            let rule_config = system_config.clone();
            Some(Arc::new(
                AdaptiveRuleSystem::new(rule_config).await.map_err(|e| {
                    crate::error::ContextError::InitializationFailed(format!(
                        "Adaptive rule system setup failed: {e}"
                    ))
                })?,
            ))
        } else {
            debug!("Adaptive rule system disabled in system configuration");
            None
        };

        // Setup reward system based on configuration
        let reward_system = if system_config.enable_reinforcement_learning {
            let reward_config = system_config.clone();
            Some(Arc::new(RewardSystem::new(reward_config).await.map_err(
                |e| {
                    crate::error::ContextError::InitializationFailed(format!(
                        "Reward system setup failed: {e}"
                    ))
                },
            )?))
        } else {
            debug!("Reward system disabled in system configuration");
            None
        };

        // Configure policy network with system-derived parameters
        let policy_network = if system_config.enable_reinforcement_learning {
            let policy_config = system_config.policy_network_config.clone();
            Some(Arc::new(PolicyNetwork::new(policy_config).await.map_err(
                |e| {
                    crate::error::ContextError::InitializationFailed(format!(
                        "Policy network setup failed: {e}"
                    ))
                },
            )?))
        } else {
            debug!("Policy network disabled in system configuration");
            None
        };

        // Initialize context learning manager with system configuration
        let context_learning_manager = if system_config.enable_reinforcement_learning {
            Some(Arc::new(
                ContextLearningManager::new(system_config.clone())
                    .await
                    .map_err(|e| {
                        crate::error::ContextError::InitializationFailed(format!(
                            "Context learning manager setup failed: {e}"
                        ))
                    })?,
            ))
        } else {
            debug!("Context learning manager disabled in system configuration");
            None
        };

        info!(
            "Learning integration initialized with configuration: learning_rate={}, exploration_rate={}, components_enabled={}",
            system_config.learning_rate,
            system_config.exploration_rate,
            Self::count_enabled_components(&system_config)
        );

        Ok(Self {
            config,
            context_manager: Some(context_manager),
            rule_manager: Some(rule_manager),
            visualization_system,
            learning_engine,
            context_learning_manager,
            reward_system,
            policy_network,
            learning_metrics,
            adaptive_rule_system,
            state: Arc::new(RwLock::new(initial_state)),
            stats: Arc::new(Mutex::new(IntegrationStats::default())),
            task_handles: Arc::new(Mutex::new(Vec::new())),
        })
    }

    /// Build integration configuration from system configuration
    async fn build_integration_config(
        system_config: &LearningSystemConfig,
    ) -> Result<LearningIntegrationConfig> {
        let integration_config = LearningIntegrationConfig {
            enable_context_manager: system_config.enable_reinforcement_learning,
            enable_rule_manager: system_config.enable_adaptive_rules,
            enable_visualization: system_config.enable_learning_metrics,
            update_interval: system_config.learning_update_interval,
            enable_auto_triggers: false,
            trigger_thresholds: Default::default(),
        };

        // Determine initial status (removing field access since field doesn't exist)
        // integration_config.status = status;  // Field doesn't exist
        // integration_config.active_integrations = Vec::new(); // Field doesn't exist

        Ok(integration_config)
    }

    /// Determine initial state based on system configuration
    fn determine_initial_state(system_config: &LearningSystemConfig) -> IntegrationState {
        let status = if system_config.enable_reinforcement_learning {
            // Use existing field instead of auto_start
            IntegrationStatus::Active
        } else {
            IntegrationStatus::Initializing
        };

        IntegrationState {
            status,
            last_update: Utc::now(),
            active_integrations: Vec::new(), // Fix type: should be Vec<String> not integer
            errors: Vec::new(),
        }
    }

    /// Initialize managers based on system configuration
    async fn initialize_managers(
        system_config: &LearningSystemConfig,
    ) -> Result<(Arc<ContextManager>, Arc<RuleManager>)> {
        // Initialize context manager (always enabled)
        let context_manager = Arc::new(ContextManager::new());

        // Initialize rule manager with default rules directory
        let rule_manager = if system_config.enable_adaptive_rules {
            Some(Arc::new(RuleManager::new("./rules"))) // Provide default rules directory
        } else {
            None
        };

        let rule_manager = rule_manager.unwrap_or_else(|| Arc::new(RuleManager::new("./rules")));

        Ok((context_manager, rule_manager))
    }

    /// Count enabled components in system configuration
    fn count_enabled_components(system_config: &LearningSystemConfig) -> usize {
        let mut count = 0;
        if system_config.enable_reinforcement_learning {
            count += 1;
        }
        if system_config.enable_reinforcement_learning {
            count += 1;
        } // context manager
        if system_config.enable_learning_metrics {
            count += 1;
        }
        if system_config.enable_learning_metrics {
            count += 1;
        } // visualization
        if system_config.enable_adaptive_rules {
            count += 1;
        }
        if system_config.enable_reinforcement_learning {
            count += 1;
        } // reward system
        if system_config.enable_reinforcement_learning {
            count += 1;
        } // policy network
        count
    }

    /// Initialize the integration layer
    pub async fn initialize(&self) -> Result<()> {
        info!("Initializing learning integration layer");

        // Update state
        let mut state = self.state.write().await;
        state.status = IntegrationStatus::Initializing;
        state.last_update = Utc::now();

        info!("Learning integration layer initialized successfully");
        Ok(())
    }

    /// Start the integration layer
    pub async fn start(&self) -> Result<()> {
        info!("Starting learning integration layer");

        // Start background tasks
        self.start_background_tasks().await?;

        // Update state
        let mut state = self.state.write().await;
        state.status = IntegrationStatus::Active;
        state.last_update = Utc::now();

        info!("Learning integration layer started successfully");
        Ok(())
    }

    /// Stop the integration layer
    pub async fn stop(&self) -> Result<()> {
        info!("Stopping learning integration layer");

        // Stop background tasks
        self.stop_background_tasks().await?;

        // Update state
        let mut state = self.state.write().await;
        state.status = IntegrationStatus::Stopped;
        state.last_update = Utc::now();

        info!("Learning integration layer stopped successfully");
        Ok(())
    }

    /// Set context manager
    pub fn set_context_manager(&mut self, manager: Arc<ContextManager>) {
        self.context_manager = Some(manager);
    }

    /// Set rule manager
    pub fn set_rule_manager(&mut self, manager: Arc<RuleManager>) {
        self.rule_manager = Some(manager);
    }

    /// Set visualization system
    pub fn set_visualization_system(&mut self, system: Arc<VisualizationSystem>) {
        self.visualization_system = Some(system);
    }

    /// Set learning engine
    pub fn set_learning_engine(&mut self, engine: Arc<LearningEngine>) {
        self.learning_engine = Some(engine);
    }

    /// Set context learning manager
    pub fn set_context_learning_manager(&mut self, manager: Arc<ContextLearningManager>) {
        self.context_learning_manager = Some(manager);
    }

    /// Set reward system
    pub fn set_reward_system(&mut self, system: Arc<RewardSystem>) {
        self.reward_system = Some(system);
    }

    /// Set policy network
    pub fn set_policy_network(&mut self, network: Arc<PolicyNetwork>) {
        self.policy_network = Some(network);
    }

    /// Set learning metrics
    pub fn set_learning_metrics(&mut self, metrics: Arc<LearningMetrics>) {
        self.learning_metrics = Some(metrics);
    }

    /// Set adaptive rule system
    pub fn set_adaptive_rule_system(&mut self, system: Arc<AdaptiveRuleSystem>) {
        self.adaptive_rule_system = Some(system);
    }

    /// Start background tasks
    async fn start_background_tasks(&self) -> Result<()> {
        let mut handles = self.task_handles.lock().await;

        // Context monitoring task
        if self.config.enable_context_manager {
            let context_task = {
                let integration = self.clone_refs();
                let config = Arc::clone(&self.config);

                tokio::spawn(async move {
                    let mut interval = tokio::time::interval(config.update_interval);

                    loop {
                        interval.tick().await;

                        if let Err(e) = Self::monitor_context_changes(&integration).await {
                            error!("Context monitoring error: {}", e);
                        }
                    }
                })
            };

            handles.push(context_task);
        }

        // Rule monitoring task
        if self.config.enable_rule_manager {
            let rule_task = {
                let integration = self.clone_refs();
                let config = Arc::clone(&self.config);

                tokio::spawn(async move {
                    let mut interval = tokio::time::interval(config.update_interval);

                    loop {
                        interval.tick().await;

                        if let Err(e) = Self::monitor_rule_performance(&integration).await {
                            error!("Rule monitoring error: {}", e);
                        }
                    }
                })
            };

            handles.push(rule_task);
        }

        // Learning synchronization task
        let learning_task = {
            let integration = self.clone_refs();
            let config = Arc::clone(&self.config);

            tokio::spawn(async move {
                let mut interval = tokio::time::interval(config.update_interval);

                loop {
                    interval.tick().await;

                    if let Err(e) = Self::synchronize_learning(&integration).await {
                        error!("Learning synchronization error: {}", e);
                    }
                }
            })
        };

        handles.push(learning_task);

        Ok(())
    }

    /// Stop background tasks
    async fn stop_background_tasks(&self) -> Result<()> {
        let mut handles = self.task_handles.lock().await;

        for handle in handles.drain(..) {
            handle.abort();
        }

        Ok(())
    }

    /// Clone references for background tasks
    fn clone_refs(&self) -> IntegrationRefs {
        IntegrationRefs {
            context_manager: self.context_manager.clone(),
            rule_manager: self.rule_manager.clone(),
            learning_engine: self.learning_engine.clone(),
            context_learning_manager: self.context_learning_manager.clone(),
            reward_system: self.reward_system.clone(),
            policy_network: self.policy_network.clone(),
            learning_metrics: self.learning_metrics.clone(),
            adaptive_rule_system: self.adaptive_rule_system.clone(),
        }
    }

    /// Monitor context changes
    async fn monitor_context_changes(refs: &IntegrationRefs) -> Result<()> {
        // Monitor context changes and trigger learning if needed
        if let Some(_context_manager) = &refs.context_manager {
            debug!("Starting simplified context change monitoring");

            // Use simplified monitoring since specific methods don't exist yet
            // FUTURE: [API-Enhancement] Implement proper context monitoring when ContextManager API is enhanced
            // Tracking: Planned for v0.2.0 - ContextManager API enhancement

            // Placeholder for future context monitoring implementation
            debug!("Context monitoring placeholder - actual implementation pending");

            // Simple monitoring results
            let _monitoring_results = ContextMonitoringResults {
                total_contexts: 0,                // Placeholder
                contexts_needing_intervention: 0, // Placeholder
                monitoring_timestamp: chrono::Utc::now(),
            };

            debug!("Context change monitoring completed (simplified version)");
        } else {
            debug!("No context manager available for monitoring");
        }

        Ok(())
    }

    /// Monitor rule performance
    async fn monitor_rule_performance(refs: &IntegrationRefs) -> Result<()> {
        // Monitor rule performance and trigger adaptations if needed
        if let Some(adaptive_rules) = &refs.adaptive_rule_system {
            let adaptations = adaptive_rules.adapt_rules().await?;

            if !adaptations.is_empty() {
                info!("Triggered {} rule adaptations", adaptations.len());
            }
        }

        Ok(())
    }

    /// Synchronize learning components
    async fn synchronize_learning(refs: &IntegrationRefs) -> Result<()> {
        // Synchronize learning components
        if let Some(learning_metrics) = &refs.learning_metrics {
            learning_metrics.take_snapshot().await?;
        }

        Ok(())
    }

    /// Trigger learning episode
    pub async fn trigger_learning_episode(&self, context_id: &str) -> Result<String> {
        if let Some(learning_manager) = &self.context_learning_manager {
            let episode_id = learning_manager.start_episode(context_id).await?;

            // Update statistics
            let mut stats = self.stats.lock().await;
            stats.learning_episodes += 1;
            stats.last_operation = Utc::now();

            info!("Triggered learning episode: {}", episode_id);
            Ok(episode_id)
        } else {
            Err(crate::error::ContextError::NotInitialized)
        }
    }

    /// Calculate reward for context action
    pub async fn calculate_reward(&self, context_id: &str, action_data: Value) -> Result<f64> {
        if let Some(reward_system) = &self.reward_system {
            // Create reward context (simplified)
            let reward_context = super::reward::RewardContext {
                action: super::engine::RLAction {
                    id: uuid::Uuid::new_v4().to_string(),
                    action_type: "context_action".to_string(),
                    parameters: action_data,
                    confidence: 1.0,
                    expected_reward: 0.0,
                },
                previous_state: super::engine::RLState {
                    id: uuid::Uuid::new_v4().to_string(),
                    features: vec![0.0; 10],
                    context_id: context_id.to_string(),
                    timestamp: Utc::now(),
                    metadata: None,
                },
                current_state: super::engine::RLState {
                    id: uuid::Uuid::new_v4().to_string(),
                    features: vec![0.5; 10],
                    context_id: context_id.to_string(),
                    timestamp: Utc::now(),
                    metadata: None,
                },
                performance_metrics: super::reward::PerformanceMetrics {
                    sync_status: true,
                    version: 1,
                    active_contexts: 1,
                    memory_usage: 0.5,
                    processing_time: 0.1,
                    success_rate: 0.8,
                    error_rate: 0.1,
                    throughput: 10.0,
                },
                rule_results: None,
                error_info: None,
                timestamp: Utc::now(),
            };

            let reward = reward_system.calculate_reward(reward_context).await?;

            // Update statistics
            let mut stats = self.stats.lock().await;
            stats.total_operations += 1;
            stats.successful_operations += 1;
            stats.last_operation = Utc::now();

            Ok(reward)
        } else {
            Err(crate::error::ContextError::NotInitialized)
        }
    }

    /// Get integration state
    pub async fn get_state(&self) -> IntegrationState {
        self.state.read().await.clone()
    }

    /// Get integration statistics
    pub async fn get_stats(&self) -> IntegrationStats {
        self.stats.lock().await.clone()
    }

    /// Update integration statistics
    ///
    /// Note: Internal method for future integration statistics - implementation in progress
    #[expect(dead_code, reason = "planned feature not yet wired")]
    async fn update_stats(&self, operation_success: bool, operation_time: f64) -> Result<()> {
        let mut stats = self.stats.lock().await;

        stats.total_operations += 1;
        if operation_success {
            stats.successful_operations += 1;
        } else {
            stats.failed_operations += 1;
        }

        // Update average operation time
        stats.average_operation_time =
            (stats.average_operation_time * (stats.total_operations - 1) as f64 + operation_time)
                / stats.total_operations as f64;

        stats.last_operation = Utc::now();

        Ok(())
    }

    /// Record integration error
    ///
    /// Note: Internal method for future error tracking - implementation in progress
    #[expect(dead_code, reason = "planned feature not yet wired")]
    async fn record_error(&self, error_type: &str, message: &str, component: &str) -> Result<()> {
        let error = IntegrationError {
            id: uuid::Uuid::new_v4().to_string(),
            error_type: error_type.to_string(),
            message: message.to_string(),
            timestamp: Utc::now(),
            component: component.to_string(),
        };

        let mut state = self.state.write().await;
        state.errors.push(error);

        // Keep error history manageable
        if state.errors.len() > 100 {
            state.errors.remove(0);
        }

        Ok(())
    }

    /// Get learning system status
    pub async fn get_learning_status(&self) -> Result<Value> {
        let mut status = serde_json::json!({
            "integration_state": self.get_state().await,
            "integration_stats": self.get_stats().await,
            "components": {}
        });

        // Add component status
        if let Some(learning_engine) = &self.learning_engine {
            status["components"]["learning_engine"] = serde_json::json!({
                "state": learning_engine.get_state().await,
                "metrics": learning_engine.get_metrics().await
            });
        }

        if let Some(learning_metrics) = &self.learning_metrics {
            status["components"]["learning_metrics"] = serde_json::json!({
                "performance": learning_metrics.get_performance().await,
                "stats": learning_metrics.get_stats().await
            });
        }

        Ok(status)
    }
}

/// References to integration components
///
/// Note: Component references for future integration - some components not yet wired up
#[derive(Debug, Clone)]
#[expect(dead_code, reason = "planned feature not yet wired")]
pub struct IntegrationRefs {
    pub context_manager: Option<Arc<ContextManager>>,
    pub rule_manager: Option<Arc<RuleManager>>,
    pub learning_engine: Option<Arc<LearningEngine>>,
    pub context_learning_manager: Option<Arc<ContextLearningManager>>,
    pub reward_system: Option<Arc<RewardSystem>>,
    pub policy_network: Option<Arc<PolicyNetwork>>,
    pub learning_metrics: Option<Arc<LearningMetrics>>,
    pub adaptive_rule_system: Option<Arc<AdaptiveRuleSystem>>,
}
