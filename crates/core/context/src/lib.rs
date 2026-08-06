// SPDX-License-Identifier: AGPL-3.0-or-later
// ORC-Notice: Context adaptation mechanics licensed under ORC
// Copyright (C) 2026 ecoPrimals Contributors

#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::float_cmp,
        reason = "test code needs direct assertions"
    )
)]
#![warn(missing_docs)]
#![expect(
    clippy::missing_errors_doc,
    clippy::unused_async,
    clippy::significant_drop_tightening,
    clippy::use_self,
    clippy::must_use_candidate,
    clippy::missing_const_for_fn,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::option_if_let_else,
    clippy::doc_markdown,
    clippy::or_fun_call,
    clippy::redundant_closure_for_method_calls,
    clippy::unnecessary_wraps,
    clippy::unused_self,
    clippy::needless_pass_by_value,
    clippy::implicit_clone,
    clippy::single_match_else,
    reason = "Large context crate; progressive lint tightening — 21 stale suppressions removed Wave 156t"
)]

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

#[cfg(test)]
impl Default for ContextState {
    fn default() -> Self {
        Self {
            id: String::new(),
            version: 0,
            timestamp: 0,
            data: Value::Null,
            metadata: std::collections::HashMap::new(),
            synchronized: false,
            last_modified: SystemTime::UNIX_EPOCH,
        }
    }
}

pub mod error;
#[cfg(feature = "context-learning")]
pub mod learning;
pub mod manager;
pub mod plugins;
pub mod rules;
/// Context synchronization and distribution
pub mod sync;
#[cfg(test)]
mod sync_tests;
mod sync_types;
pub mod tracker;
#[cfg(feature = "context-visualization")]
pub mod visualization;
pub use error::{ContextError, Result};
pub use manager::ContextManager;
pub use tracker::{ContextTracker, ContextTrackerFactory};

// Re-export from rules module (only what exists)
pub use rules::Rule;

#[cfg(feature = "context-visualization")]
pub use visualization::{
    VisualizationConfig, VisualizationManager, VisualizationRequest, VisualizationResponse,
    VisualizationSystem, VisualizationSystemConfig, VisualizationType,
};

// Re-export from learning module (feature-gated: planned but not runtime-wired)
#[cfg(feature = "context-learning")]
pub use learning::{
    AdaptiveRuleSystem, ContextLearningManager, ExperienceReplay, LearningEngine,
    LearningIntegration, LearningMetrics, LearningState, LearningSystem, LearningSystemConfig,
    PolicyNetwork, RewardSystem,
};
