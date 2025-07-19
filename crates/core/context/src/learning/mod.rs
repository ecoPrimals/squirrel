//! Context Management System Learning System
//!
//! This module provides reinforcement learning capabilities for the Context Management System.
//! It enables the system to learn from context changes, adapt rules and actions based on experience,
//! and optimize context management based on outcomes.
//!
//! ## Features
//!
//! - **Reinforcement Learning Engine**: Core RL algorithm implementation
//! - **Experience Replay**: Stores and manages experiences for learning
//! - **Reward System**: Calculates rewards based on context outcomes
//! - **Policy Network**: Neural network for decision making
//! - **Adaptive Rules**: Rules that can be modified based on learning
//! - **Learning Metrics**: Tracks learning progress and performance
//! - **Integration Layer**: Connects with existing context management components
//!
//! ## Architecture
//!
//! The learning system is built around a reinforcement learning framework that:
//! 1. Observes context state changes
//! 2. Takes actions based on learned policies
//! 3. Receives rewards based on outcomes
//! 4. Updates policies to improve future decisions
//!
//! ## Learning Process
//!
//! ```
//! Context State → Learning Agent → Action → Context Update → Reward → Policy Update
//! ```

mod adaptive;
mod engine;
mod experience;
mod integration;
mod manager;
mod metrics;
mod policy;
mod reward;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, Mutex, RwLock};

use crate::error::Result;

// Re-export public types
pub use adaptive::{AdaptiveRule, AdaptiveRuleSystem, RuleAdaptation};
pub use engine::{LearningEngine, LearningEngineConfig};
pub use experience::{Experience, ExperienceBuffer, ExperienceReplay};
pub use integration::{LearningIntegration, LearningIntegrationConfig};
pub use manager::{ContextLearningManager, ContextLearningManagerConfig};
pub use metrics::{LearningMetrics, LearningPerformance, LearningStats};
pub use policy::{PolicyAction, PolicyNetwork, PolicyState};
pub use reward::{RewardCalculator, RewardMetrics, RewardSystem};

/// Learning system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningSystemConfig {
    /// Enable reinforcement learning
    pub enable_reinforcement_learning: bool,

    /// Enable experience replay
    pub enable_experience_replay: bool,

    /// Enable adaptive rules
    pub enable_adaptive_rules: bool,

    /// Enable learning metrics
    pub enable_learning_metrics: bool,

    /// Learning rate for the RL algorithm
    pub learning_rate: f64,

    /// Discount factor for future rewards
    pub discount_factor: f64,

    /// Exploration rate for epsilon-greedy strategy
    pub exploration_rate: f64,

    /// Experience replay buffer size
    pub experience_buffer_size: usize,

    /// Batch size for learning updates
    pub batch_size: usize,

    /// Target network update frequency
    pub target_update_frequency: u64,

    /// Minimum experiences before learning starts
    pub min_experiences_for_learning: usize,

    /// Maximum episode length
    pub max_episode_length: usize,

    /// Reward calculation method
    pub reward_calculation: RewardCalculationType,

    /// Policy network architecture
    pub policy_network_config: PolicyNetworkConfig,

    /// Update interval for learning
    pub learning_update_interval: Duration,

    /// Enable debug logging
    pub enable_debug_logging: bool,
}

impl Default for LearningSystemConfig {
    fn default() -> Self {
        Self {
            enable_reinforcement_learning: true,
            enable_experience_replay: true,
            enable_adaptive_rules: true,
            enable_learning_metrics: true,
            learning_rate: 0.001,
            discount_factor: 0.95,
            exploration_rate: 0.1,
            experience_buffer_size: 10000,
            batch_size: 32,
            target_update_frequency: 1000,
            min_experiences_for_learning: 1000,
            max_episode_length: 1000,
            reward_calculation: RewardCalculationType::Composite,
            policy_network_config: PolicyNetworkConfig::default(),
            learning_update_interval: Duration::from_secs(10),
            enable_debug_logging: false,
        }
    }
}

/// Reward calculation type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RewardCalculationType {
    /// Simple reward based on success/failure
    Simple,
    /// Composite reward based on multiple factors
    Composite,
    /// Custom reward function
    Custom(String),
}

/// Policy network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyNetworkConfig {
    /// Input layer size
    pub input_size: usize,

    /// Hidden layer sizes
    pub hidden_layers: Vec<usize>,

    /// Output layer size
    pub output_size: usize,

    /// Activation function
    pub activation_function: String,

    /// Optimizer configuration
    pub optimizer: String,

    /// Dropout rate
    pub dropout_rate: f64,
}

impl Default for PolicyNetworkConfig {
    fn default() -> Self {
        Self {
            input_size: 128,
            hidden_layers: vec![256, 128, 64],
            output_size: 32,
            activation_function: "relu".to_string(),
            optimizer: "adam".to_string(),
            dropout_rate: 0.2,
        }
    }
}

/// Learning system state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LearningState {
    /// System is initializing
    Initializing,
    /// System is learning
    Learning,
    /// System is evaluating
    Evaluating,
    /// System is adapting
    Adapting,
    /// System is paused
    Paused,
    /// System is stopped
    Stopped,
}

/// Learning action type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LearningActionType {
    /// Modify context state
    ModifyContext,
    /// Apply rule
    ApplyRule,
    /// Update policy
    UpdatePolicy,
    /// Adapt rule
    AdaptRule,
    /// Create snapshot
    CreateSnapshot,
    /// Trigger synchronization
    TriggerSync,
    /// Custom action
    Custom(String),
}

/// Learning event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningEvent {
    /// Event ID
    pub id: String,

    /// Event timestamp
    pub timestamp: DateTime<Utc>,

    /// Event type
    pub event_type: LearningEventType,

    /// Context ID
    pub context_id: String,

    /// Event data
    pub data: Value,

    /// Event metadata
    pub metadata: Option<Value>,
}

/// Learning event type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LearningEventType {
    /// Episode started
    EpisodeStarted,
    /// Action taken
    ActionTaken,
    /// Reward received
    RewardReceived,
    /// Episode ended
    EpisodeEnded,
    /// Policy updated
    PolicyUpdated,
    /// Rule adapted
    RuleAdapted,
    /// Performance measured
    PerformanceMeasured,
    /// Error occurred
    ErrorOccurred,
}

/// Central learning system that coordinates all learning components
#[derive(Debug)]
pub struct LearningSystem {
    /// Configuration
    config: Arc<LearningSystemConfig>,

    /// Learning engine
    engine: Arc<LearningEngine>,

    /// Context learning manager
    manager: Arc<ContextLearningManager>,

    /// Experience replay buffer
    experience_replay: Arc<ExperienceReplay>,

    /// Reward system
    reward_system: Arc<RewardSystem>,

    /// Policy network
    policy_network: Arc<PolicyNetwork>,

    /// Learning metrics
    metrics: Arc<LearningMetrics>,

    /// Adaptive rule system
    adaptive_rules: Arc<AdaptiveRuleSystem>,

    /// Integration layer
    integration: Arc<LearningIntegration>,

    /// Current learning state
    state: Arc<RwLock<LearningState>>,

    /// Event broadcaster
    event_broadcaster: Arc<broadcast::Sender<LearningEvent>>,

    /// System statistics
    stats: Arc<Mutex<LearningSystemStats>>,
}

/// Learning system statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningSystemStats {
    /// Total episodes
    pub total_episodes: u64,

    /// Total actions taken
    pub total_actions: u64,

    /// Total rewards received
    pub total_rewards: f64,

    /// Average reward per episode
    pub average_reward_per_episode: f64,

    /// Success rate
    pub success_rate: f64,

    /// Learning accuracy
    pub learning_accuracy: f64,

    /// Policy update count
    pub policy_updates: u64,

    /// Rule adaptations
    pub rule_adaptations: u64,

    /// System uptime
    pub uptime: Duration,

    /// Last performance measurement
    pub last_performance: Option<DateTime<Utc>>,
}

impl Default for LearningSystemStats {
    fn default() -> Self {
        Self {
            total_episodes: 0,
            total_actions: 0,
            total_rewards: 0.0,
            average_reward_per_episode: 0.0,
            success_rate: 0.0,
            learning_accuracy: 0.0,
            policy_updates: 0,
            rule_adaptations: 0,
            uptime: Duration::from_secs(0),
            last_performance: None,
        }
    }
}

impl LearningSystem {
    /// Create a new learning system
    pub async fn new(config: LearningSystemConfig) -> Result<Self> {
        let config = Arc::new(config);

        // Create event broadcaster
        let (event_sender, _) = broadcast::channel(1000);
        let event_broadcaster = Arc::new(event_sender);

        // Create components
        let engine = Arc::new(LearningEngine::new(config.clone()).await?);
        let manager = Arc::new(ContextLearningManager::new(config.clone()).await?);
        let experience_replay = Arc::new(ExperienceReplay::new(config.experience_buffer_size));
        let reward_system = Arc::new(RewardSystem::new(config.clone()).await?);
        let policy_network =
            Arc::new(PolicyNetwork::new(config.policy_network_config.clone()).await?);
        let metrics = Arc::new(LearningMetrics::new(config.clone()).await?);
        let adaptive_rules = Arc::new(AdaptiveRuleSystem::new(config.clone()).await?);
        let integration = Arc::new(LearningIntegration::new(config.clone()).await?);

        Ok(Self {
            config,
            engine,
            manager,
            experience_replay,
            reward_system,
            policy_network,
            metrics,
            adaptive_rules,
            integration,
            state: Arc::new(RwLock::new(LearningState::Initializing)),
            event_broadcaster,
            stats: Arc::new(Mutex::new(LearningSystemStats::default())),
        })
    }

    /// Initialize the learning system
    pub async fn initialize(&self) -> Result<()> {
        // Update state
        *self.state.write().await = LearningState::Initializing;

        // Initialize components
        self.engine.initialize().await?;
        self.manager.initialize().await?;
        self.reward_system.initialize().await?;
        self.policy_network.initialize().await?;
        self.metrics.initialize().await?;
        self.adaptive_rules.initialize().await?;
        self.integration.initialize().await?;

        // Update state
        *self.state.write().await = LearningState::Learning;

        // Send initialization event
        self.send_event(LearningEvent {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            event_type: LearningEventType::EpisodeStarted,
            context_id: "system".to_string(),
            data: serde_json::json!({
                "action": "system_initialized",
                "config": (*self.config).clone()
            }),
            metadata: None,
        })
        .await?;

        Ok(())
    }

    /// Start the learning system
    pub async fn start(&self) -> Result<()> {
        // Update state
        *self.state.write().await = LearningState::Learning;

        // Start components
        self.engine.start().await?;
        self.manager.start().await?;
        self.metrics.start().await?;
        self.integration.start().await?;

        Ok(())
    }

    /// Stop the learning system
    pub async fn stop(&self) -> Result<()> {
        // Update state
        *self.state.write().await = LearningState::Stopped;

        // Stop components
        self.engine.stop().await?;
        self.manager.stop().await?;
        self.metrics.stop().await?;
        self.integration.stop().await?;

        Ok(())
    }

    /// Get current learning state
    pub async fn get_state(&self) -> LearningState {
        self.state.read().await.clone()
    }

    /// Get system statistics
    pub async fn get_stats(&self) -> LearningSystemStats {
        self.stats.lock().await.clone()
    }

    /// Send learning event
    async fn send_event(&self, event: LearningEvent) -> Result<()> {
        let _ = self.event_broadcaster.send(event);
        Ok(())
    }

    /// Subscribe to learning events
    pub fn subscribe_to_events(&self) -> broadcast::Receiver<LearningEvent> {
        self.event_broadcaster.subscribe()
    }

    /// Get learning engine
    pub fn get_engine(&self) -> Arc<LearningEngine> {
        Arc::clone(&self.engine)
    }

    /// Get context learning manager
    pub fn get_manager(&self) -> Arc<ContextLearningManager> {
        Arc::clone(&self.manager)
    }

    /// Get experience replay
    pub fn get_experience_replay(&self) -> Arc<ExperienceReplay> {
        Arc::clone(&self.experience_replay)
    }

    /// Get reward system
    pub fn get_reward_system(&self) -> Arc<RewardSystem> {
        Arc::clone(&self.reward_system)
    }

    /// Get policy network
    pub fn get_policy_network(&self) -> Arc<PolicyNetwork> {
        Arc::clone(&self.policy_network)
    }

    /// Get learning metrics
    pub fn get_metrics(&self) -> Arc<LearningMetrics> {
        Arc::clone(&self.metrics)
    }

    /// Get adaptive rule system
    pub fn get_adaptive_rules(&self) -> Arc<AdaptiveRuleSystem> {
        Arc::clone(&self.adaptive_rules)
    }

    /// Get integration layer
    pub fn get_integration(&self) -> Arc<LearningIntegration> {
        Arc::clone(&self.integration)
    }
}
