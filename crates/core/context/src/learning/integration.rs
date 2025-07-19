//! Learning Integration Layer
//!
//! This module provides the integration layer that connects the learning system
//! with the existing Context Management System components.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, error, info, warn};

use super::{
    AdaptiveRuleSystem, ContextLearningManager, LearningEngine, LearningMetrics,
    LearningSystemConfig, PolicyNetwork, RewardSystem,
};
use crate::error::Result;
use crate::manager::ContextManager;
use crate::rules::RuleManager;
use crate::visualization::VisualizationSystem;
use crate::ContextTracker;

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
        let config = Arc::new(LearningIntegrationConfig::default());

        Ok(Self {
            config,
            context_manager: None,
            rule_manager: None,
            visualization_system: None,
            learning_engine: None,
            context_learning_manager: None,
            reward_system: None,
            policy_network: None,
            learning_metrics: None,
            adaptive_rule_system: None,
            state: Arc::new(RwLock::new(IntegrationState {
                status: IntegrationStatus::Initializing,
                last_update: Utc::now(),
                active_integrations: Vec::new(),
                errors: Vec::new(),
            })),
            stats: Arc::new(Mutex::new(IntegrationStats::default())),
            task_handles: Arc::new(Mutex::new(Vec::new())),
        })
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
        if let Some(context_manager) = &refs.context_manager {
            // Check for context changes (simplified implementation)
            debug!("Monitoring context changes");
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
#[derive(Debug, Clone)]
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
