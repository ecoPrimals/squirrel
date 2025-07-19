//! Context Management Module
//!
//! This module provides context management functionality for the application.

pub mod error;
pub mod learning;
pub mod manager;
pub mod plugins;
pub mod rules;
pub mod tracker;
pub mod visualization;

// Add the additional module with ContextState definition
#[path = "mod.rs"]
pub mod context_types;

// Re-export commonly used types
pub use context_types::ContextState;
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
    AdaptiveRuleSystem, ContextLearningManager, ExperienceReplay, LearningEngine, LearningEvent,
    LearningEventType, LearningIntegration, LearningMetrics, LearningState, LearningSystem,
    LearningSystemConfig, PolicyNetwork, RewardSystem,
};
