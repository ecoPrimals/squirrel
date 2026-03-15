// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Context Management Module
//!
//! This module provides context management functionality for the application.

#![forbid(unsafe_code)]
#![allow(
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::missing_docs_in_private_items,
    clippy::unused_async,
    clippy::significant_drop_tightening,
    clippy::use_self,
    clippy::must_use_candidate,
    clippy::missing_const_for_fn,
    clippy::struct_excessive_bools,
    clippy::derive_partial_eq_without_eq,
    clippy::format_push_string,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::option_if_let_else,
    clippy::suboptimal_flops,
    clippy::cast_possible_truncation,
    clippy::doc_markdown,
    clippy::branches_sharing_code,
    clippy::redundant_clone,
    clippy::return_self_not_must_use,
    clippy::ignored_unit_patterns,
    clippy::cast_lossless,
    clippy::manual_let_else,
    clippy::single_char_pattern,
    clippy::uninlined_format_args,
    clippy::or_fun_call,
    clippy::redundant_closure_for_method_calls,
    clippy::unnecessary_wraps,
    clippy::unused_self,
    clippy::manual_string_new,
    clippy::needless_pass_by_value,
    clippy::map_unwrap_or,
    clippy::if_not_else,
    clippy::imprecise_flops,
    clippy::used_underscore_binding,
    clippy::single_match_else,
    clippy::implicit_clone,
    clippy::match_same_arms,
    clippy::manual_midpoint,
    clippy::too_many_lines,
    clippy::default_trait_access,
    clippy::significant_drop_in_scrutinee
)]

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
pub mod learning;
pub mod manager;
pub mod plugins;
pub mod rules;
/// Context synchronization and distribution
pub mod sync;
#[cfg(test)]
mod sync_tests;
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
