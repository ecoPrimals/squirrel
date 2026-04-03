// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! # AI Capabilities for biomeOS Integration
//!
//! This module provides AI-powered capabilities for ecosystem intelligence,
//! optimization, prediction, and automation within the biomeOS ecosystem.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tracing::{debug, info};

use super::{IntelligenceRequest, IntelligenceResponse, Optimization, Prediction};
use crate::error::PrimalError;

/// AI Intelligence for ecosystem intelligence
#[derive(Debug, Clone)]
pub struct AiIntelligence {
    /// Core intelligence engine for analysis.
    pub intelligence_engine: IntelligenceEngine,
    /// Engine for resource and performance optimization.
    pub optimization_engine: OptimizationEngine,
    /// Engine for forecasting and anomaly detection.
    pub prediction_engine: PredictionEngine,
    /// Engine for self-healing and autonomous operations.
    pub automation_engine: AutomationEngine,
    /// Intelligence for coordinating with other biomes.
    pub federation_intelligence: FederationIntelligence,
    /// Count of active predictions.
    pub active_predictions: u32,
    /// Count of active automation tasks.
    pub automation_tasks: u32,
    /// Timestamp of last optimization run.
    pub last_optimization: Option<DateTime<Utc>>,
}

/// Core intelligence engine for ecosystem analysis
#[derive(Debug, Clone)]
pub struct IntelligenceEngine {
    /// Model identifiers used for analysis.
    pub analysis_models: Vec<String>,
    /// Learning rate for model updates.
    pub learning_rate: f64,
    /// Minimum confidence for predictions.
    pub confidence_threshold: f64,
    /// Accumulated ecosystem knowledge.
    pub ecosystem_knowledge: EcosystemKnowledge,
}

/// Optimization engine for resource and performance optimization
#[derive(Debug, Clone)]
pub struct OptimizationEngine {
    /// Strategy identifiers for optimization.
    pub optimization_strategies: Vec<String>,
    /// Target resource utilization levels.
    pub resource_targets: ResourceTargets,
    /// Target performance metrics.
    pub performance_targets: PerformanceTargets,
    /// History of optimization events.
    pub optimization_history: Vec<OptimizationEvent>,
}

/// Prediction engine for forecasting and anomaly detection
#[derive(Debug, Clone)]
pub struct PredictionEngine {
    /// Model identifiers for predictions.
    pub prediction_models: Vec<String>,
    /// Currently active predictions by ID.
    pub active_predictions: HashMap<String, Prediction>,
    /// Current prediction accuracy score.
    pub prediction_accuracy: f64,
    /// Anomaly detection configuration.
    pub anomaly_detection: AnomalyDetection,
}

/// Automation engine for self-healing and autonomous operations
#[derive(Debug, Clone, Default)]
pub struct AutomationEngine {
    /// Rules that trigger automation.
    pub automation_rules: Vec<AutomationRule>,
    /// Currently running automation tasks.
    pub active_automations: HashMap<String, AutomationTask>,
    /// History of automation events.
    pub automation_history: Vec<AutomationEvent>,
}

/// Federation intelligence for coordinating with other biomes
#[derive(Debug, Clone, Default)]
pub struct FederationIntelligence {
    /// IDs of connected biomes.
    pub connected_biomes: Vec<String>,
    /// Intelligence sharing configuration.
    pub intelligence_sharing: IntelligenceSharing,
    /// Protocols for cross-biome coordination.
    pub coordination_protocols: Vec<CoordinationProtocol>,
}

/// Ecosystem knowledge base
#[derive(Debug, Clone, Default)]
pub struct EcosystemKnowledge {
    /// Learned patterns by ID.
    pub patterns: HashMap<String, KnowledgePattern>,
    /// Generated ecosystem insights.
    pub insights: Vec<EcosystemInsight>,
    /// Accumulated learnings.
    pub learnings: Vec<EcosystemLearning>,
}

/// Resource optimization targets
#[derive(Debug, Clone, Default)]
pub struct ResourceTargets {
    /// Target CPU utilization (0.0–1.0).
    pub cpu_target: f64,
    /// Target memory utilization (0.0–1.0).
    pub memory_target: f64,
    /// Target network utilization (0.0–1.0).
    pub network_target: f64,
    /// Target storage utilization (0.0–1.0).
    pub storage_target: f64,
}

/// Performance optimization targets
#[derive(Debug, Clone, Default)]
pub struct PerformanceTargets {
    /// Target response time.
    pub response_time_target: Duration,
    /// Target throughput (requests per second).
    pub throughput_target: f64,
    /// Target error rate (0.0–1.0).
    pub error_rate_target: f64,
    /// Target availability (0.0–1.0).
    pub availability_target: f64,
}

/// Optimization event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationEvent {
    /// When the optimization occurred.
    pub timestamp: DateTime<Utc>,
    /// Type of optimization applied.
    pub optimization_type: String,
    /// Target component or metric.
    pub target: String,
    /// Value before optimization.
    pub before_value: f64,
    /// Value after optimization.
    pub after_value: f64,
    /// Improvement achieved.
    pub improvement: f64,
}

/// Anomaly detection configuration
#[derive(Debug, Clone, Default)]
pub struct AnomalyDetection {
    /// Model identifiers for detection.
    pub detection_models: Vec<String>,
    /// Threshold values by metric.
    pub threshold_settings: HashMap<String, f64>,
    /// Rules for generating alerts.
    pub alert_rules: Vec<AlertRule>,
}

/// Automation rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationRule {
    /// Unique rule identifier.
    pub rule_id: String,
    /// Human-readable rule name.
    pub name: String,
    /// Condition expression that triggers the rule.
    pub condition: String,
    /// Action to execute when triggered.
    pub action: String,
    /// Whether the rule is active.
    pub enabled: bool,
    /// Priority for rule ordering (higher = first).
    pub priority: u32,
}

/// Automation task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationTask {
    /// Unique task identifier.
    pub task_id: String,
    /// ID of the rule that triggered this task.
    pub rule_id: String,
    /// Current task status.
    pub status: String,
    /// When the task started.
    pub started_at: DateTime<Utc>,
    /// When the task completed, if finished.
    pub completed_at: Option<DateTime<Utc>>,
    /// Result or error message.
    pub result: Option<String>,
}

/// Automation event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationEvent {
    /// When the event occurred.
    pub timestamp: DateTime<Utc>,
    /// Type of automation event.
    pub event_type: String,
    /// ID of the rule that ran.
    pub rule_id: String,
    /// Action that was executed.
    pub action: String,
    /// Result of the action.
    pub result: String,
    /// How long the action took.
    pub duration: Duration,
}

/// Intelligence sharing configuration
#[derive(Debug, Clone, Default)]
pub struct IntelligenceSharing {
    /// Whether sharing is enabled.
    pub sharing_enabled: bool,
    /// Sharing level (e.g., "full", "summary").
    pub sharing_level: String,
    /// Insight IDs shared with other biomes.
    pub shared_insights: Vec<String>,
    /// Insight IDs received from other biomes.
    pub received_insights: Vec<String>,
}

/// Coordination protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinationProtocol {
    /// Unique protocol identifier.
    pub protocol_id: String,
    /// Protocol name.
    pub name: String,
    /// Protocol version.
    pub version: String,
    /// Whether the protocol is active.
    pub enabled: bool,
    /// Protocol-specific configuration.
    pub configuration: HashMap<String, String>,
}

/// Knowledge pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgePattern {
    /// Unique pattern identifier.
    pub pattern_id: String,
    /// Pattern name.
    pub name: String,
    /// Pattern description.
    pub description: String,
    /// Confidence in this pattern (0.0–1.0).
    pub confidence: f64,
    /// Number of times the pattern was used.
    pub usage_count: u32,
    /// When the pattern was last used.
    pub last_used: DateTime<Utc>,
}

/// Ecosystem insight
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemInsight {
    /// Unique insight identifier.
    pub insight_id: String,
    /// Insight title.
    pub title: String,
    /// Detailed description.
    pub description: String,
    /// Importance score (0.0–1.0).
    pub importance: f64,
    /// Whether the insight suggests an action.
    pub actionable: bool,
    /// When the insight was generated.
    pub generated_at: DateTime<Utc>,
}

/// Ecosystem learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemLearning {
    /// Unique learning identifier.
    pub learning_id: String,
    /// Topic of the learning.
    pub topic: String,
    /// Knowledge content.
    pub knowledge: String,
    /// Confidence in this learning (0.0–1.0).
    pub confidence: f64,
    /// When the learning was recorded.
    pub learned_at: DateTime<Utc>,
}

/// Alert rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    /// Unique rule identifier.
    pub rule_id: String,
    /// Rule name.
    pub name: String,
    /// Condition that triggers the alert.
    pub condition: String,
    /// Alert severity (e.g., "critical", "warning").
    pub severity: String,
    /// Whether the rule is active.
    pub enabled: bool,
}

/// Resource utilization data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUtilization {
    /// CPU utilization percentage.
    pub cpu_percent: f64,
    /// Memory utilization percentage.
    pub memory_percent: f64,
    /// Network utilization percentage.
    pub network_percent: f64,
    /// Storage utilization percentage.
    pub storage_percent: f64,
    /// When the metrics were captured.
    pub timestamp: DateTime<Utc>,
}

impl Default for AiIntelligence {
    fn default() -> Self {
        Self::new()
    }
}

impl AiIntelligence {
    /// Create a new AI intelligence instance
    #[must_use]
    pub fn new() -> Self {
        Self {
            intelligence_engine: IntelligenceEngine::default(),
            optimization_engine: OptimizationEngine::default(),
            prediction_engine: PredictionEngine::default(),
            automation_engine: AutomationEngine::default(),
            federation_intelligence: FederationIntelligence::default(),
            active_predictions: 0,
            automation_tasks: 0,
            last_optimization: None,
        }
    }

    /// Initialize AI intelligence
    pub async fn initialize(&mut self) -> Result<(), PrimalError> {
        info!("Initializing AI intelligence for ecosystem");

        // Initialize intelligence engine
        self.intelligence_engine.initialize().await?;

        // Initialize optimization engine
        self.optimization_engine.initialize().await?;

        // Initialize prediction engine
        self.prediction_engine.initialize().await?;

        // Initialize automation engine
        self.automation_engine.initialize().await?;

        // Initialize federation intelligence
        self.federation_intelligence.initialize().await?;

        info!("AI intelligence initialized successfully");
        Ok(())
    }

    /// Provide ecosystem intelligence
    pub async fn provide_ecosystem_intelligence(&self) -> Result<(), PrimalError> {
        debug!("Providing ecosystem intelligence");

        // Analyze current ecosystem state
        let _analysis = self.analyze_ecosystem_state().await?;

        // Generate optimization recommendations
        let _optimizations = self.generate_optimizations().await?;

        // Update predictions
        self.update_predictions().await?;

        // Execute automation tasks
        self.execute_automation_tasks().await?;

        Ok(())
    }

    /// Analyze ecosystem state using actual engine metrics.
    async fn analyze_ecosystem_state(&self) -> Result<EcosystemAnalysis, PrimalError> {
        debug!("Analyzing ecosystem state from engine telemetry");

        let active_predictions = self.prediction_engine.active_predictions.len() as u32;
        let active_automations = self.automation_engine.active_automations.len() as u32;
        let prediction_accuracy = self.prediction_engine.prediction_accuracy;

        let health_score = if prediction_accuracy > 0.0 {
            prediction_accuracy
        } else {
            0.5
        };

        let mut recommendations = Vec::new();
        if active_predictions == 0 {
            recommendations
                .push("No active predictions — consider seeding the prediction engine".to_string());
        }
        if active_automations == 0 && !self.automation_engine.automation_rules.is_empty() {
            recommendations.push("Automation rules defined but no active automations".to_string());
        }

        Ok(EcosystemAnalysis {
            timestamp: Utc::now(),
            health_score,
            resource_usage: ResourceUtilization {
                cpu_percent: 0.0,
                memory_percent: 0.0,
                network_percent: 0.0,
                storage_percent: 0.0,
                timestamp: Utc::now(),
            },
            active_services: active_predictions + active_automations,
            alerts: vec![],
            recommendations,
        })
    }

    /// Generate optimization recommendations derived from engine state.
    async fn generate_optimizations(&self) -> Result<Vec<Optimization>, PrimalError> {
        debug!("Generating optimization recommendations from engine state");

        let mut optimizations = Vec::new();

        if self.optimization_engine.optimization_history.is_empty() {
            optimizations.push(Optimization {
                optimization_id: format!("opt-{}", Utc::now().timestamp()),
                optimization_type: "Baseline Profiling".to_string(),
                target_component: "all".to_string(),
                improvement_potential: 0.0,
                implementation_steps: vec![
                    "Collect baseline metrics before optimizing".to_string(),
                ],
            });
        }

        for strategy in &self.optimization_engine.optimization_strategies {
            optimizations.push(Optimization {
                optimization_id: format!("opt-{strategy}-{}", Utc::now().timestamp()),
                optimization_type: strategy.clone(),
                target_component: "ecosystem".to_string(),
                improvement_potential: 10.0,
                implementation_steps: vec![format!("Apply {strategy} strategy")],
            });
        }

        Ok(optimizations)
    }

    /// Update predictions
    async fn update_predictions(&self) -> Result<(), PrimalError> {
        debug!("Updating predictions");
        // Implementation for prediction updates
        Ok(())
    }

    /// Execute automation tasks
    async fn execute_automation_tasks(&self) -> Result<(), PrimalError> {
        debug!("Executing automation tasks");
        // Implementation for automation task execution
        Ok(())
    }

    /// Analyze ecosystem
    pub async fn analyze_ecosystem(&self) -> Result<(), PrimalError> {
        debug!("Analyzing ecosystem");
        let _analysis = self.analyze_ecosystem_state().await?;
        Ok(())
    }

    /// Generate ecosystem report from current engine telemetry.
    pub async fn generate_ecosystem_report(&self) -> Result<EcosystemReport, PrimalError> {
        debug!("Generating ecosystem report from engine telemetry");

        let analysis = self.analyze_ecosystem_state().await?;
        let optimizations = self.generate_optimizations().await?;

        let recommendations: Vec<String> = optimizations
            .into_iter()
            .flat_map(|o| o.implementation_steps)
            .collect();

        Ok(EcosystemReport {
            timestamp: Utc::now(),
            ecosystem_health: analysis.health_score,
            total_services: analysis.active_services,
            active_alerts: analysis.alerts.len() as u32,
            recommendations,
            resource_summary: ResourceSummary {
                cpu_usage: analysis.resource_usage.cpu_percent,
                memory_usage: analysis.resource_usage.memory_percent,
                storage_usage: analysis.resource_usage.storage_percent,
                network_usage: analysis.resource_usage.network_percent,
            },
        })
    }

    /// Process intelligence request
    pub async fn process_intelligence_request(
        &self,
        request: IntelligenceRequest,
    ) -> Result<IntelligenceResponse, PrimalError> {
        debug!("Processing intelligence request: {}", request.request_id);

        let _recommendations = [
            "Monitor resource usage".to_string(),
            "Update security policies".to_string(),
            "Analyze ecosystem health".to_string(),
        ];
        let processing_time = 100; // Placeholder for actual processing time

        Ok(IntelligenceResponse {
            request_id: request.request_id,
            intelligence_type: "analysis".to_string(),
            result: serde_json::json!({
                "analysis": "completed",
                "confidence": 0.9
            }),
            confidence: 0.9,
            processing_time_ms: processing_time,
            metadata: std::collections::HashMap::new(),
        })
    }

    /// Health check
    pub async fn health_check(&self) -> Result<(), PrimalError> {
        debug!("Performing AI intelligence health check");

        // Check intelligence engine health
        if !self.intelligence_engine.is_healthy() {
            return Err(PrimalError::General(
                "Intelligence engine is unhealthy".to_string(),
            ));
        }

        // Check optimization engine health
        if !self.optimization_engine.is_healthy() {
            return Err(PrimalError::General(
                "Optimization engine is unhealthy".to_string(),
            ));
        }

        // Check prediction engine health
        if !self.prediction_engine.is_healthy() {
            return Err(PrimalError::General(
                "Prediction engine is unhealthy".to_string(),
            ));
        }

        Ok(())
    }
}

/// Ecosystem analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemAnalysis {
    /// When the analysis was performed.
    pub timestamp: DateTime<Utc>,
    /// Overall ecosystem health score (0.0–100.0).
    pub health_score: f64,
    /// Current resource utilization.
    pub resource_usage: ResourceUtilization,
    /// Number of active services.
    pub active_services: u32,
    /// Active alert messages.
    pub alerts: Vec<String>,
    /// Recommended actions.
    pub recommendations: Vec<String>,
}

/// Ecosystem report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemReport {
    /// When the report was generated.
    pub timestamp: DateTime<Utc>,
    /// Overall ecosystem health (0.0–100.0).
    pub ecosystem_health: f64,
    /// Total number of services.
    pub total_services: u32,
    /// Number of active alerts.
    pub active_alerts: u32,
    /// Recommended actions.
    pub recommendations: Vec<String>,
    /// Summary of resource usage.
    pub resource_summary: ResourceSummary,
}

/// Resource summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceSummary {
    /// CPU utilization (0.0–1.0).
    pub cpu_usage: f64,
    /// Memory utilization (0.0–1.0).
    pub memory_usage: f64,
    /// Storage utilization (0.0–1.0).
    pub storage_usage: f64,
    /// Network utilization (0.0–1.0).
    pub network_usage: f64,
}

impl Default for IntelligenceEngine {
    fn default() -> Self {
        Self {
            analysis_models: vec![
                "ecosystem_health".to_string(),
                "resource_optimization".to_string(),
            ],
            learning_rate: 0.01,
            confidence_threshold: 0.8,
            ecosystem_knowledge: EcosystemKnowledge::default(),
        }
    }
}

impl Default for OptimizationEngine {
    fn default() -> Self {
        Self {
            optimization_strategies: vec![
                "resource_balancing".to_string(),
                "performance_tuning".to_string(),
            ],
            resource_targets: ResourceTargets::default(),
            performance_targets: PerformanceTargets::default(),
            optimization_history: vec![],
        }
    }
}

impl Default for PredictionEngine {
    fn default() -> Self {
        Self {
            prediction_models: vec![
                "demand_forecasting".to_string(),
                "anomaly_detection".to_string(),
            ],
            active_predictions: HashMap::new(),
            prediction_accuracy: 0.85,
            anomaly_detection: AnomalyDetection::default(),
        }
    }
}

impl IntelligenceEngine {
    async fn initialize(&mut self) -> Result<(), PrimalError> {
        info!(
            models = self.analysis_models.len(),
            learning_rate = self.learning_rate,
            confidence_threshold = self.confidence_threshold,
            "Intelligence engine initialized"
        );
        Ok(())
    }

    fn is_healthy(&self) -> bool {
        !self.analysis_models.is_empty() && self.confidence_threshold > 0.0
    }
}

impl OptimizationEngine {
    async fn initialize(&mut self) -> Result<(), PrimalError> {
        info!(
            strategies = self.optimization_strategies.len(),
            "Optimization engine initialized"
        );
        self.optimization_history.clear();
        Ok(())
    }

    const fn is_healthy(&self) -> bool {
        !self.optimization_strategies.is_empty()
    }
}

impl PredictionEngine {
    async fn initialize(&mut self) -> Result<(), PrimalError> {
        info!(
            models = self.prediction_models.len(),
            accuracy = self.prediction_accuracy,
            "Prediction engine initialized"
        );
        self.active_predictions.clear();
        Ok(())
    }

    const fn is_healthy(&self) -> bool {
        !self.prediction_models.is_empty()
    }
}

impl AutomationEngine {
    async fn initialize(&mut self) -> Result<(), PrimalError> {
        info!(
            rules = self.automation_rules.len(),
            "Automation engine initialized"
        );
        self.active_automations.clear();
        self.automation_history.clear();
        Ok(())
    }
}

impl FederationIntelligence {
    async fn initialize(&mut self) -> Result<(), PrimalError> {
        info!(
            biomes = self.connected_biomes.len(),
            protocols = self.coordination_protocols.len(),
            "Federation intelligence initialized"
        );
        Ok(())
    }
}
