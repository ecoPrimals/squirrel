// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Data types for the learning integration layer.
//!
//! Configuration, state, statistics, and error types used by
//! [`super::integration::LearningIntegration`].
//!
//! Fields are self-documenting DTO structs; see the parent module for usage docs.
#![expect(missing_docs, reason = "DTO fields — documented at usage site")]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use super::{
    AdaptiveRuleSystem, ContextLearningManager, LearningEngine, LearningMetrics, PolicyNetwork,
    RewardSystem,
};
use crate::manager::ContextManager;
use crate::rules::RuleManager;

/// Snapshot from a context-monitoring tick (session counts and sync health).
#[derive(Debug, Clone)]
pub struct ContextMonitoringResults {
    /// Active context session keys (excluding internal recovery snapshots).
    pub total_contexts: usize,
    /// Sessions whose state is not yet synchronized (may need intervention).
    pub contexts_needing_intervention: usize,
    #[allow(dead_code)] // Carried for downstream telemetry correlation
    pub monitoring_timestamp: chrono::DateTime<chrono::Utc>,
}

/// Learning integration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningIntegrationConfig {
    pub enable_context_manager: bool,
    pub enable_rule_manager: bool,
    pub enable_visualization: bool,
    pub update_interval: std::time::Duration,
    pub enable_auto_triggers: bool,
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
    pub min_context_changes: usize,
    pub min_rule_applications: usize,
    pub error_rate_threshold: f64,
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

/// Integration state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationState {
    pub status: IntegrationStatus,
    pub last_update: DateTime<Utc>,
    pub active_integrations: Vec<String>,
    pub errors: Vec<IntegrationError>,
}

/// Integration status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IntegrationStatus {
    Initializing,
    Active,
    Paused,
    Stopped,
    Error,
}

/// Integration error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationError {
    pub id: String,
    pub error_type: String,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub component: String,
}

/// Integration statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationStats {
    pub total_operations: usize,
    pub successful_operations: usize,
    pub failed_operations: usize,
    pub context_syncs: usize,
    pub rule_adaptations: usize,
    pub learning_episodes: usize,
    pub average_operation_time: f64,
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

/// References to integration components
#[derive(Debug, Clone)]
#[allow(dead_code)] // Held for wiring as integration phases land
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
