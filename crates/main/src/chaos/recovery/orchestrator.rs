//! Main recovery orchestrator for chaos experiments

use super::{
    ActiveRecovery, CompletedRecoveryMetrics, HealthValidator, RecoveryConfig, RecoveryMetrics,
    RecoveryMetricsCollector, RecoveryStatus, RecoveryStep, RecoveryStrategy,
    RecoveryStrategyExecutor, StepStatus, ValidationResult,
};
use crate::error::PrimalError;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tokio::time::timeout;
use uuid::Uuid;

/// Recovery orchestrator for chaos experiments
#[derive(Debug)]
pub struct RecoveryOrchestrator {
    /// Active recovery operations
    active_recoveries: Arc<RwLock<HashMap<String, ActiveRecovery>>>,
    /// Recovery strategy implementations
    strategies: Arc<RecoveryStrategyExecutor>,
    /// Health validation system
    validator: Arc<HealthValidator>,
    /// Recovery metrics collector
    metrics: Arc<RecoveryMetricsCollector>,
}

impl RecoveryOrchestrator {
    /// Create a new recovery orchestrator
    pub fn new() -> Self {
        Self {
            active_recoveries: Arc::new(RwLock::new(HashMap::new())),
            strategies: Arc::new(RecoveryStrategyExecutor::new()),
            validator: Arc::new(HealthValidator::new()),
            metrics: Arc::new(RecoveryMetricsCollector::new()),
        }
    }

    /// Start a recovery operation
    pub async fn start_recovery(
        &self,
        experiment_id: String,
        config: RecoveryConfig,
    ) -> Result<String, PrimalError> {
        let recovery_id = Uuid::new_v4().to_string();
        let start_time = Instant::now();

        let recovery = ActiveRecovery {
            recovery_id: recovery_id.clone(),
            experiment_id: experiment_id.clone(),
            config: config.clone(),
            status: RecoveryStatus::Starting,
            start_time,
            executed_steps: Vec::new(),
            validation_results: Vec::new(),
            metrics: RecoveryMetrics::new(),
        };

        // Store active recovery
        {
            let mut recoveries = self.active_recoveries.write().await;
            recoveries.insert(recovery_id.clone(), recovery);
        }

        // Start metrics tracking
        self.metrics
            .start_tracking(recovery_id.clone(), RecoveryMetrics::new())
            .await;

        // Execute recovery in background
        let orchestrator = self.clone();
        tokio::spawn(async move {
            if let Err(e) = orchestrator.execute_recovery(&recovery_id).await {
                let _ = orchestrator
                    .mark_recovery_failed(&recovery_id, e.to_string())
                    .await;
            }
        });

        Ok(recovery_id)
    }

    /// Execute the complete recovery process
    async fn execute_recovery(&self, recovery_id: &str) -> Result<(), PrimalError> {
        // Get recovery configuration
        let config = {
            let recoveries = self.active_recoveries.read().await;
            recoveries
                .get(recovery_id)
                .ok_or_else(|| {
                    PrimalError::NotFound(format!("Recovery not found: {}", recovery_id))
                })?
                .config
                .clone()
        };

        // Execute with timeout
        let result = timeout(config.timeout, async {
            self.update_recovery_status(recovery_id, RecoveryStatus::ExecutingStrategies)
                .await;

            // Execute recovery strategies
            self.execute_recovery_strategies(recovery_id, &config.strategies)
                .await?;

            // Perform validation
            self.update_recovery_status(recovery_id, RecoveryStatus::Validating)
                .await;

            self.perform_recovery_validation(recovery_id, &config.validation_steps)
                .await?;

            // Mark as completed
            self.update_recovery_status(recovery_id, RecoveryStatus::Completed)
                .await;

            Ok::<(), PrimalError>(())
        })
        .await;

        match result {
            Ok(Ok(())) => {
                // Finalize recovery metrics
                self.finalize_recovery_metrics(recovery_id).await;
                Ok(())
            }
            Ok(Err(e)) => Err(e),
            Err(_) => {
                // Timeout occurred
                self.update_recovery_status(recovery_id, RecoveryStatus::TimedOut)
                    .await;
                Err(PrimalError::Timeout(
                    "Recovery operation timed out".to_string(),
                ))
            }
        }
    }

    /// Execute recovery strategies
    async fn execute_recovery_strategies(
        &self,
        recovery_id: &str,
        strategies: &[RecoveryStrategy],
    ) -> Result<(), PrimalError> {
        for strategy in strategies {
            let step_id = Uuid::new_v4().to_string();
            let step = RecoveryStep {
                step_id: step_id.clone(),
                strategy: strategy.clone(),
                status: StepStatus::Running,
                start_time: std::time::SystemTime::now(),
                end_time: None,
                duration: None,
                output: String::new(),
                error: None,
            };

            // Add step to tracking
            self.add_recovery_step(recovery_id, step).await;

            // Execute strategy
            let result = self.strategies.execute_strategy(strategy).await;

            // Update step completion
            self.complete_recovery_step(recovery_id, &step_id, result)
                .await;
        }

        Ok(())
    }

    /// Perform recovery validation
    async fn perform_recovery_validation(
        &self,
        recovery_id: &str,
        validation_steps: &[super::ValidationStep],
    ) -> Result<(), PrimalError> {
        for step in validation_steps {
            let result = self.validator.perform_validation(step).await?;
            self.add_validation_result(recovery_id, result).await;

            // Check if we should fail fast
            let config = {
                let recoveries = self.active_recoveries.read().await;
                recoveries.get(recovery_id).map(|r| r.config.clone())
            };

            if let Some(config) = config {
                if config.fail_fast && !self.check_validation_success(recovery_id).await {
                    return Err(PrimalError::ValidationFailed(
                        "Validation failed with fail_fast enabled".to_string(),
                    ));
                }
            }
        }

        // Check overall validation success
        if !self.check_validation_success(recovery_id).await {
            return Err(PrimalError::ValidationFailed(
                "Overall validation failed".to_string(),
            ));
        }

        Ok(())
    }

    /// Update recovery status
    async fn update_recovery_status(&self, recovery_id: &str, status: RecoveryStatus) {
        let mut recoveries = self.active_recoveries.write().await;
        if let Some(recovery) = recoveries.get_mut(recovery_id) {
            recovery.status = status;
        }
    }

    /// Add recovery step to tracking
    async fn add_recovery_step(&self, recovery_id: &str, step: RecoveryStep) {
        let mut recoveries = self.active_recoveries.write().await;
        if let Some(recovery) = recoveries.get_mut(recovery_id) {
            recovery.executed_steps.push(step);
        }
    }

    /// Complete a recovery step with results
    async fn complete_recovery_step(
        &self,
        recovery_id: &str,
        step_id: &str,
        result: Result<String, PrimalError>,
    ) {
        let mut recoveries = self.active_recoveries.write().await;
        if let Some(recovery) = recoveries.get_mut(recovery_id) {
            if let Some(step) = recovery
                .executed_steps
                .iter_mut()
                .find(|s| s.step_id == step_id)
            {
                let end_time = std::time::SystemTime::now();
                step.end_time = Some(end_time);
                step.duration = end_time.duration_since(step.start_time).ok();

                match result {
                    Ok(output) => {
                        step.status = StepStatus::Completed;
                        step.output = output;
                        recovery.metrics.strategies_succeeded += 1;
                    }
                    Err(e) => {
                        step.status = StepStatus::Failed;
                        step.error = Some(e.to_string());
                    }
                }
                recovery.metrics.strategies_attempted += 1;
            }
        }
    }

    /// Add validation result
    async fn add_validation_result(&self, recovery_id: &str, result: ValidationResult) {
        let mut recoveries = self.active_recoveries.write().await;
        if let Some(recovery) = recoveries.get_mut(recovery_id) {
            recovery.metrics.validations_performed += 1;
            if result.passed {
                recovery.metrics.validations_passed += 1;
            }
            recovery.validation_results.push(result);
        }
    }

    /// Check if all validations passed
    async fn check_validation_success(&self, recovery_id: &str) -> bool {
        let recoveries = self.active_recoveries.read().await;
        if let Some(recovery) = recoveries.get(recovery_id) {
            recovery.validation_results.iter().all(|v| v.passed)
        } else {
            false
        }
    }

    /// Mark recovery as failed
    async fn mark_recovery_failed(&self, recovery_id: &str, reason: String) {
        self.update_recovery_status(recovery_id, RecoveryStatus::Failed { reason })
            .await;
    }

    /// Finalize recovery metrics
    async fn finalize_recovery_metrics(&self, recovery_id: &str) {
        let recovery = {
            let recoveries = self.active_recoveries.read().await;
            recoveries.get(recovery_id).cloned()
        };

        if let Some(recovery) = recovery {
            let total_time = recovery.start_time.elapsed();

            let mut final_metrics = recovery.metrics.clone();
            final_metrics.total_recovery_time = Some(total_time);

            // Calculate health score improvement
            final_metrics.health_score_after =
                Some(self.calculate_health_score_improvement(&recovery).await);

            // Store historical metrics
            let completed_metrics = CompletedRecoveryMetrics {
                recovery_id: recovery.recovery_id.clone(),
                experiment_id: recovery.experiment_id.clone(),
                recovery_duration: total_time,
                strategies_used: recovery.config.strategies.clone(),
                success_rate: final_metrics.validations_passed as f64
                    / final_metrics.validations_performed.max(1) as f64,
                health_improvement: final_metrics.health_score_after.unwrap_or(0.0),
                timestamp: std::time::SystemTime::now(),
            };

            self.metrics
                .store_completed_recovery(completed_metrics)
                .await;
        }
    }

    /// Calculate health score improvement after recovery
    async fn calculate_health_score_improvement(&self, recovery: &ActiveRecovery) -> f64 {
        // Calculate health score based on validation results
        if recovery.validation_results.is_empty() {
            return 0.0;
        }

        let passed_validations = recovery
            .validation_results
            .iter()
            .filter(|v| v.passed)
            .count() as f64;

        passed_validations / recovery.validation_results.len() as f64
    }

    /// Get recovery status
    pub async fn get_recovery_status(&self, recovery_id: &str) -> Option<RecoveryStatus> {
        let recoveries = self.active_recoveries.read().await;
        recoveries.get(recovery_id).map(|r| r.status.clone())
    }

    /// Get detailed recovery information
    pub async fn get_recovery_details(&self, recovery_id: &str) -> Option<ActiveRecovery> {
        let recoveries = self.active_recoveries.read().await;
        recoveries.get(recovery_id).cloned()
    }

    /// Get recovery metrics
    pub async fn get_recovery_metrics(&self) -> Vec<CompletedRecoveryMetrics> {
        self.metrics.get_historical_metrics().await
    }

    /// Stop active recovery
    pub async fn stop_recovery(&self, recovery_id: &str) -> Result<(), PrimalError> {
        let mut recoveries = self.active_recoveries.write().await;
        if let Some(_recovery) = recoveries.remove(recovery_id) {
            // TODO: Implement recovery cancellation logic
            Ok(())
        } else {
            Err(PrimalError::NotFound(format!(
                "Recovery not found: {}",
                recovery_id
            )))
        }
    }
}

impl Clone for RecoveryOrchestrator {
    fn clone(&self) -> Self {
        Self {
            active_recoveries: self.active_recoveries.clone(),
            strategies: self.strategies.clone(),
            validator: self.validator.clone(),
            metrics: self.metrics.clone(),
        }
    }
}

impl Default for RecoveryOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}
