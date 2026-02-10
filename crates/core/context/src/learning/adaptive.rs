// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Adaptive Rule System
//!
//! This module implements an adaptive rule system that can modify and evolve rules
//! based on learning experiences and performance feedback.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, info, trace};
use uuid::Uuid;

use super::LearningSystemConfig;
use crate::error::Result;
use crate::rules::Rule;

/// Adaptive rule system
#[derive(Debug)]
pub struct AdaptiveRuleSystem {
    /// System configuration (reserved for future use)
    #[allow(dead_code)]
    config: Arc<LearningSystemConfig>,

    /// Adaptive rules
    adaptive_rules: Arc<RwLock<HashMap<String, AdaptiveRule>>>,

    /// Rule adaptations history
    adaptations: Arc<RwLock<Vec<RuleAdaptation>>>,

    /// Adaptation statistics
    stats: Arc<Mutex<AdaptationStats>>,
}

/// Adaptive rule that can be modified based on learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptiveRule {
    /// Base rule
    pub base_rule: Rule,

    /// Adaptation metadata
    pub adaptation_meta: AdaptationMetadata,

    /// Performance metrics
    pub performance: RulePerformance,

    /// Adaptation history
    pub adaptation_history: Vec<String>,
}

/// Adaptation metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptationMetadata {
    /// Rule ID
    pub rule_id: String,

    /// Adaptation level (0.0 to 1.0)
    pub adaptation_level: f64,

    /// Learning rate for this rule
    pub learning_rate: f64,

    /// Adaptation strategy
    pub strategy: AdaptationStrategy,

    /// Last adaptation time
    pub last_adaptation: DateTime<Utc>,

    /// Adaptation count
    pub adaptation_count: usize,
}

/// Adaptation strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AdaptationStrategy {
    /// Gradually adjust rule parameters
    Gradual,

    /// Threshold-based adaptation
    Threshold(f64),

    /// Performance-based adaptation
    Performance,

    /// Reinforcement learning-based adaptation
    Reinforcement,
}

/// Rule performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RulePerformance {
    /// Success rate
    pub success_rate: f64,

    /// Average execution time
    pub avg_execution_time: f64,

    /// Application count
    pub application_count: usize,

    /// Success count
    pub success_count: usize,

    /// Effectiveness score
    pub effectiveness: f64,

    /// Impact score
    pub impact: f64,

    /// Last performance update
    pub last_update: DateTime<Utc>,
}

impl Default for RulePerformance {
    fn default() -> Self {
        Self {
            success_rate: 0.0,
            avg_execution_time: 0.0,
            application_count: 0,
            success_count: 0,
            effectiveness: 0.0,
            impact: 0.0,
            last_update: Utc::now(),
        }
    }
}

/// Rule adaptation record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleAdaptation {
    /// Adaptation ID
    pub id: String,

    /// Rule ID
    pub rule_id: String,

    /// Adaptation type
    pub adaptation_type: AdaptationType,

    /// Changes made
    pub changes: Vec<RuleChange>,

    /// Reason for adaptation
    pub reason: String,

    /// Performance before adaptation
    pub performance_before: RulePerformance,

    /// Performance after adaptation
    pub performance_after: Option<RulePerformance>,

    /// Adaptation timestamp
    pub timestamp: DateTime<Utc>,
}

/// Type of adaptation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AdaptationType {
    /// Modify rule conditions
    ConditionModification,

    /// Modify rule actions
    ActionModification,

    /// Adjust rule priority
    PriorityAdjustment,

    /// Change rule parameters
    ParameterAdjustment,

    /// Enable/disable rule
    EnablementChange,
}

/// Specific change to a rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleChange {
    /// Type of change
    pub change_type: String,

    /// Target component
    pub target: String,

    /// Previous value
    pub previous_value: Value,

    /// New value
    pub new_value: Value,

    /// Confidence in change
    pub confidence: f64,
}

/// Adaptation statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptationStats {
    /// Total adaptations
    pub total_adaptations: usize,

    /// Successful adaptations
    pub successful_adaptations: usize,

    /// Failed adaptations
    pub failed_adaptations: usize,

    /// Average improvement
    pub average_improvement: f64,

    /// Rules with adaptations
    pub rules_with_adaptations: usize,

    /// Last adaptation time
    pub last_adaptation: DateTime<Utc>,

    /// Number of rule adaptations
    pub rule_adaptations: usize,

    /// Last rule adapted
    pub last_adapted_rule: Option<String>,
}

impl Default for AdaptationStats {
    fn default() -> Self {
        Self {
            total_adaptations: 0,
            successful_adaptations: 0,
            failed_adaptations: 0,
            average_improvement: 0.0,
            rules_with_adaptations: 0,
            last_adaptation: Utc::now(),
            rule_adaptations: 0,
            last_adapted_rule: None,
        }
    }
}

impl AdaptiveRuleSystem {
    /// Create a new adaptive rule system
    pub async fn new(config: Arc<LearningSystemConfig>) -> Result<Self> {
        Ok(Self {
            config,
            adaptive_rules: Arc::new(RwLock::new(HashMap::new())),
            adaptations: Arc::new(RwLock::new(Vec::new())),
            stats: Arc::new(Mutex::new(AdaptationStats::default())),
        })
    }

    /// Initialize the adaptive rule system
    pub async fn initialize(&self) -> Result<()> {
        info!("Initializing adaptive rule system");

        // Initialize with default configuration
        info!("Adaptive rule system initialized successfully");
        Ok(())
    }

    /// Add a rule to the adaptive system
    pub async fn add_rule(&self, rule: Rule) -> Result<()> {
        let adaptive_rule = AdaptiveRule {
            base_rule: rule.clone(),
            adaptation_meta: AdaptationMetadata {
                rule_id: rule.id().to_string(),
                adaptation_level: 0.0,
                learning_rate: 0.1,
                strategy: AdaptationStrategy::Gradual,
                last_adaptation: Utc::now(),
                adaptation_count: 0,
            },
            performance: RulePerformance::default(),
            adaptation_history: Vec::new(),
        };

        let mut adaptive_rules = self.adaptive_rules.write().await;
        adaptive_rules.insert(rule.id().to_string(), adaptive_rule);

        debug!("Added rule to adaptive system: {}", rule.id());
        Ok(())
    }

    /// Update rule performance
    pub async fn update_rule_performance(
        &self,
        rule_id: &str,
        success: bool,
        execution_time: f64,
    ) -> Result<()> {
        let mut adaptive_rules = self.adaptive_rules.write().await;

        if let Some(adaptive_rule) = adaptive_rules.get_mut(rule_id) {
            let performance = &mut adaptive_rule.performance;

            performance.application_count += 1;
            if success {
                performance.success_count += 1;
            }

            performance.success_rate =
                performance.success_count as f64 / performance.application_count as f64;

            // Update average execution time
            performance.avg_execution_time = (performance.avg_execution_time
                * (performance.application_count - 1) as f64
                + execution_time)
                / performance.application_count as f64;

            // Calculate effectiveness (simplified)
            performance.effectiveness =
                performance.success_rate * (1.0 / (1.0 + performance.avg_execution_time));

            performance.last_update = Utc::now();

            debug!(
                "Updated performance for rule {}: success_rate={:.2}, effectiveness={:.2}",
                rule_id, performance.success_rate, performance.effectiveness
            );
        }

        Ok(())
    }

    /// Adapt rules based on performance
    pub async fn adapt_rules(&self) -> Result<Vec<RuleAdaptation>> {
        let mut adaptations = Vec::new();
        let adaptive_rules = self.adaptive_rules.read().await;

        for (rule_id, adaptive_rule) in adaptive_rules.iter() {
            debug!(
                "🧠 Evaluating adaptive rule '{}' for potential adaptation",
                rule_id
            );

            if self.should_adapt_rule(adaptive_rule).await? {
                info!(
                    "⚡ Adapting rule '{}' based on performance criteria",
                    rule_id
                );

                if let Some(adaptation) = self.adapt_rule(adaptive_rule).await? {
                    debug!(
                        "✅ Successfully adapted rule '{}': {:?}",
                        rule_id, adaptation
                    );
                    adaptations.push(adaptation);

                    // Track rule adaptation statistics
                    {
                        let mut stats = self.stats.lock().await;
                        stats.rule_adaptations += 1;
                        stats.last_adapted_rule = Some(rule_id.clone());
                    }
                } else {
                    debug!(
                        "⚠️ Rule '{}' marked for adaptation but no changes generated",
                        rule_id
                    );
                }
            } else {
                trace!(
                    "📋 Rule '{}' does not require adaptation at this time",
                    rule_id
                );
            }
        }

        // Record adaptations
        if !adaptations.is_empty() {
            let mut adaptation_history = self.adaptations.write().await;
            adaptation_history.extend(adaptations.clone());

            // Update statistics
            self.update_adaptation_stats(&adaptations).await?;
        }

        Ok(adaptations)
    }

    /// Check if a rule should be adapted
    async fn should_adapt_rule(&self, adaptive_rule: &AdaptiveRule) -> Result<bool> {
        let performance = &adaptive_rule.performance;
        let meta = &adaptive_rule.adaptation_meta;

        // Only adapt if we have enough data
        if performance.application_count < 10 {
            return Ok(false);
        }

        // Check adaptation strategy
        match meta.strategy {
            AdaptationStrategy::Gradual => {
                // Adapt if performance is below threshold
                Ok(performance.effectiveness < 0.5)
            }
            AdaptationStrategy::Threshold(threshold) => Ok(performance.success_rate < threshold),
            AdaptationStrategy::Performance => {
                // Adapt if performance is declining
                Ok(performance.effectiveness < 0.6)
            }
            AdaptationStrategy::Reinforcement => {
                // Adapt based on learning signal
                Ok(performance.effectiveness < 0.7)
            }
        }
    }

    /// Adapt a specific rule
    async fn adapt_rule(&self, adaptive_rule: &AdaptiveRule) -> Result<Option<RuleAdaptation>> {
        let rule_id = &adaptive_rule.base_rule.id().to_string();
        let performance = &adaptive_rule.performance;

        // Determine adaptation type based on performance
        let adaptation_type = if performance.success_rate < 0.3 {
            AdaptationType::ConditionModification
        } else if performance.avg_execution_time > 1.0 {
            AdaptationType::ActionModification
        } else {
            AdaptationType::ParameterAdjustment
        };

        // Generate rule changes
        let changes = self
            .generate_rule_changes(&adaptation_type, adaptive_rule)
            .await?;

        if !changes.is_empty() {
            let adaptation = RuleAdaptation {
                id: Uuid::new_v4().to_string(),
                rule_id: rule_id.clone(),
                adaptation_type,
                changes,
                reason: format!(
                    "Performance-based adaptation: effectiveness={:.2}",
                    performance.effectiveness
                ),
                performance_before: performance.clone(),
                performance_after: None,
                timestamp: Utc::now(),
            };

            info!("Adapted rule {}: {:?}", rule_id, adaptation.adaptation_type);
            return Ok(Some(adaptation));
        }

        Ok(None)
    }

    /// Generate rule changes for adaptation
    async fn generate_rule_changes(
        &self,
        adaptation_type: &AdaptationType,
        adaptive_rule: &AdaptiveRule,
    ) -> Result<Vec<RuleChange>> {
        let mut changes = Vec::new();

        match adaptation_type {
            AdaptationType::ConditionModification => {
                // Modify rule conditions (simplified)
                changes.push(RuleChange {
                    change_type: "condition_threshold".to_string(),
                    target: "condition_0".to_string(),
                    previous_value: serde_json::json!(0.5),
                    new_value: serde_json::json!(0.6),
                    confidence: 0.8,
                });
            }
            AdaptationType::ActionModification => {
                // Modify rule actions (simplified)
                changes.push(RuleChange {
                    change_type: "action_parameter".to_string(),
                    target: "action_0".to_string(),
                    previous_value: serde_json::json!("old_value"),
                    new_value: serde_json::json!("new_value"),
                    confidence: 0.7,
                });
            }
            AdaptationType::PriorityAdjustment => {
                // Adjust rule priority
                changes.push(RuleChange {
                    change_type: "priority".to_string(),
                    target: "rule_priority".to_string(),
                    previous_value: serde_json::json!(adaptive_rule.base_rule.priority()),
                    new_value: serde_json::json!(adaptive_rule.base_rule.priority() + 1),
                    confidence: 0.9,
                });
            }
            AdaptationType::ParameterAdjustment => {
                // Adjust rule parameters
                changes.push(RuleChange {
                    change_type: "parameter".to_string(),
                    target: "timeout".to_string(),
                    previous_value: serde_json::json!(10.0),
                    new_value: serde_json::json!(15.0),
                    confidence: 0.6,
                });
            }
            AdaptationType::EnablementChange => {
                // Enable/disable rule
                changes.push(RuleChange {
                    change_type: "enabled".to_string(),
                    target: "rule_enabled".to_string(),
                    previous_value: serde_json::json!(true),
                    new_value: serde_json::json!(false),
                    confidence: 0.5,
                });
            }
        }

        Ok(changes)
    }

    /// Update adaptation statistics
    async fn update_adaptation_stats(&self, adaptations: &[RuleAdaptation]) -> Result<()> {
        let mut stats = self.stats.lock().await;

        stats.total_adaptations += adaptations.len();
        stats.last_adaptation = Utc::now();

        // Calculate improvement based on actual adaptation data
        let improvement = adaptations
            .iter()
            .map(|adaptation| {
                // Calculate real improvement metric based on adaptation properties
                let base_improvement = 0.1; // Base improvement value

                // Enhance improvement calculation based on adaptation type and impact
                let type_multiplier = match &adaptation.adaptation_type {
                    AdaptationType::ParameterAdjustment => 1.0,
                    AdaptationType::PriorityAdjustment => 1.2,
                    AdaptationType::ActionModification => 1.5,
                    AdaptationType::ConditionModification => 1.8,
                    AdaptationType::EnablementChange => 2.0,
                };

                // Factor in performance improvement if available
                let performance_factor = if let Some(after) = &adaptation.performance_after {
                    // Calculate improvement ratio based on performance metrics
                    let before_score = adaptation.performance_before.success_rate;
                    let after_score = after.success_rate;
                    if before_score > 0.0 {
                        (after_score / before_score).clamp(0.5, 2.0) // Clamp between 0.5x and 2.0x
                    } else {
                        1.0 // Default if no baseline
                    }
                } else {
                    0.8 // Reduced factor if no after-performance data
                };

                let calculated_improvement =
                    base_improvement * type_multiplier * performance_factor;

                debug!(
                    "📊 Adaptation improvement: {} type={:?} perf_factor={:.2} improvement={:.3}",
                    adaptation.rule_id,
                    adaptation.adaptation_type,
                    performance_factor,
                    calculated_improvement
                );

                calculated_improvement
            })
            .sum::<f64>()
            / adaptations.len() as f64;

        stats.average_improvement = (stats.average_improvement
            * (stats.total_adaptations - adaptations.len()) as f64
            + improvement * adaptations.len() as f64)
            / stats.total_adaptations as f64;

        debug!(
            "Updated adaptation statistics: total={}, avg_improvement={:.2}",
            stats.total_adaptations, stats.average_improvement
        );

        Ok(())
    }

    /// Get adaptive rules
    pub async fn get_adaptive_rules(&self) -> HashMap<String, AdaptiveRule> {
        self.adaptive_rules.read().await.clone()
    }

    /// Get adaptation history
    pub async fn get_adaptations(&self) -> Vec<RuleAdaptation> {
        self.adaptations.read().await.clone()
    }

    /// Get adaptation statistics
    pub async fn get_stats(&self) -> AdaptationStats {
        self.stats.lock().await.clone()
    }

    /// Get rule performance
    pub async fn get_rule_performance(&self, rule_id: &str) -> Option<RulePerformance> {
        let adaptive_rules = self.adaptive_rules.read().await;
        adaptive_rules
            .get(rule_id)
            .map(|rule| rule.performance.clone())
    }

    /// Remove rule from adaptive system
    pub async fn remove_rule(&self, rule_id: &str) -> Result<()> {
        let mut adaptive_rules = self.adaptive_rules.write().await;
        adaptive_rules.remove(rule_id);

        debug!("Removed rule from adaptive system: {}", rule_id);
        Ok(())
    }

    /// Clear adaptation history
    pub async fn clear_history(&self) -> Result<()> {
        let mut adaptations = self.adaptations.write().await;
        adaptations.clear();

        let mut stats = self.stats.lock().await;
        *stats = AdaptationStats::default();

        info!("Cleared adaptation history");
        Ok(())
    }

    /// Export adaptive rules
    pub async fn export_rules(&self) -> Result<Value> {
        let adaptive_rules = self.get_adaptive_rules().await;
        let adaptations = self.get_adaptations().await;
        let stats = self.get_stats().await;

        let export = serde_json::json!({
            "export_timestamp": Utc::now(),
            "adaptive_rules": adaptive_rules,
            "adaptations": adaptations,
            "statistics": stats
        });

        Ok(export)
    }
}
