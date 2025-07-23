//! Context Management Module
//!
//! This module provides context management functionality for the application.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::SystemTime;

/// A snapshot of context state at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextSnapshot {
    /// Unique identifier for the snapshot
    pub id: String,
    /// Time when the snapshot was created
    pub timestamp: SystemTime,
    /// State data at the time of snapshot
    pub state: ContextState,
    /// Additional metadata about the snapshot
    pub metadata: Option<Value>,
}

/// State data for a context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextState {
    /// Unique identifier for the state
    pub id: String,
    /// Version number of the state
    pub version: u64,
    /// Timestamp of the state
    pub timestamp: u64,
    /// State data
    pub data: Value,
    /// Metadata associated with the state
    pub metadata: std::collections::HashMap<String, Value>,
    /// Whether the state is synchronized
    pub synchronized: bool,
    /// Time of last modification
    pub last_modified: SystemTime,
}

pub mod error;
pub mod learning;
pub mod manager;
pub mod plugins;
pub mod rules;
pub mod sync;
pub mod tracker;
pub mod visualization;
pub use error::{ContextError, Result};
pub use manager::ContextManager;
pub use tracker::{ContextTracker, ContextTrackerFactory};

// Re-export from rules module (only what exists)
pub use rules::Rule;

// Re-export from visualization module
pub use visualization::{
    VisualizationConfig, VisualizationManager, VisualizationRequest, VisualizationResponse,
    VisualizationSystem, VisualizationSystemConfig, VisualizationType,
};

// Re-export from learning module
pub use learning::{
    AdaptiveRuleSystem, ContextLearningManager, ExperienceReplay, LearningEngine,
    LearningIntegration, LearningMetrics, LearningState, LearningSystem, LearningSystemConfig,
    PolicyNetwork, RewardSystem,
};
