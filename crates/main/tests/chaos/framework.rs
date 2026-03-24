// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

#![expect(clippy::unwrap_used, clippy::expect_used, reason = "Test code: explicit unwrap/expect and local lint noise")]
//! Core chaos engineering framework
//!
//! Provides the infrastructure for injecting failures and validating recovery.

use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

/// Chaos engineering configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChaosConfig {
    /// Enable chaos testing
    pub enabled: bool,
    /// Failure injection rate (0.0 - 1.0)
    pub failure_rate: f64,
    /// Maximum duration for chaos scenarios
    pub max_duration: Duration,
    /// Recovery validation timeout
    pub recovery_timeout: Duration,
    /// Metrics collection interval
    pub metrics_interval: Duration,
}

impl Default for ChaosConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            failure_rate: 0.1,  // 10% failure rate
            max_duration: Duration::from_secs(300),  // 5 minutes max
            recovery_timeout: Duration::from_secs(60),
            metrics_interval: Duration::from_secs(1),
        }
    }
}

/// Result of a chaos scenario execution
#[derive(Debug, Clone)]
pub struct ChaosResult {
    pub scenario_name: String,
    pub success: bool,
    pub duration: Duration,
    pub failures_injected: u32,
    pub recovery_time: Option<Duration>,
    pub metrics: ChaosMetrics,
    pub error: Option<String>,
}

/// Metrics collected during chaos testing
#[derive(Debug, Clone, Default)]
pub struct ChaosMetrics {
    pub requests_attempted: u64,
    pub requests_succeeded: u64,
    pub requests_failed: u64,
    pub average_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub error_rate: f64,
}

/// Chaos scenario trait
#[async_trait]
pub trait ChaosScenario: Send + Sync {
    /// Get scenario name
    fn name(&self) -> &str;
    
    /// Get scenario description
    fn description(&self) -> &str;
    
    /// Execute the chaos scenario
    async fn execute(&self, engine: &ChaosEngine) -> Result<ChaosResult, Box<dyn std::error::Error + Send + Sync>>;
    
    /// Validate recovery after scenario
    async fn validate_recovery(&self) -> Result<bool, Box<dyn std::error::Error + Send + Sync>>;
}

/// Main chaos engineering engine
pub struct ChaosEngine {
    config: ChaosConfig,
    metrics: Arc<RwLock<ChaosMetrics>>,
    active_scenarios: Arc<RwLock<Vec<String>>>,
}

impl ChaosEngine {
    /// Create new chaos engine
    pub fn new(config: ChaosConfig) -> Self {
        Self {
            config,
            metrics: Arc::new(RwLock::new(ChaosMetrics::default())),
            active_scenarios: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Execute a chaos scenario
    pub async fn run_scenario<S: ChaosScenario>(
        &self,
        scenario: S,
    ) -> Result<ChaosResult, Box<dyn std::error::Error + Send + Sync>> {
        if !self.config.enabled {
            return Ok(ChaosResult {
                scenario_name: scenario.name().to_string(),
                success: true,
                duration: Duration::from_secs(0),
                failures_injected: 0,
                recovery_time: None,
                metrics: ChaosMetrics::default(),
                error: Some("Chaos testing disabled".to_string()),
            });
        }

        // Register active scenario
        self.active_scenarios.write().await.push(scenario.name().to_string());

        // Execute scenario
        let start = std::time::Instant::now();
        let result = scenario.execute(self).await?;

        // Validate recovery
        let recovery_start = std::time::Instant::now();
        let recovered = tokio::time::timeout(
            self.config.recovery_timeout,
            scenario.validate_recovery(),
        )
        .await
        .map_err(|_| "Recovery timeout")?;

        let recovery_time = if recovered.unwrap_or(false) {
            Some(recovery_start.elapsed())
        } else {
            None
        };

        // Remove from active scenarios
        self.active_scenarios.write().await.retain(|s| s != scenario.name());

        Ok(ChaosResult {
            recovery_time,
            ..result
        })
    }

    /// Record request metrics
    pub async fn record_request(&self, success: bool, latency: Duration) {
        let mut metrics = self.metrics.write().await;
        metrics.requests_attempted += 1;
        
        if success {
            metrics.requests_succeeded += 1;
        } else {
            metrics.requests_failed += 1;
        }

        // Update error rate
        metrics.error_rate = metrics.requests_failed as f64 / metrics.requests_attempted as f64;
        
        // Update latencies (simplified - would use proper percentile calculation in production)
        let latency_ms = latency.as_millis() as f64;
        metrics.average_latency_ms = 
            (metrics.average_latency_ms * (metrics.requests_attempted - 1) as f64 + latency_ms) 
            / metrics.requests_attempted as f64;
    }

    /// Get current metrics
    pub async fn metrics(&self) -> ChaosMetrics {
        self.metrics.read().await.clone()
    }

    /// Reset metrics
    pub async fn reset_metrics(&self) {
        *self.metrics.write().await = ChaosMetrics::default();
    }

    /// Check if scenario should inject failure
    pub fn should_inject_failure(&self) -> bool {
        rand::random::<f64>() < self.config.failure_rate
    }

    /// Get active scenarios
    pub async fn active_scenarios(&self) -> Vec<String> {
        self.active_scenarios.read().await.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chaos_config_default() {
        let config = ChaosConfig::default();
        assert!(config.enabled);
        assert_eq!(config.failure_rate, 0.1);
        assert_eq!(config.max_duration, Duration::from_secs(300));
    }

    #[test]
    fn test_chaos_engine_creation() {
        let config = ChaosConfig::default();
        let engine = ChaosEngine::new(config);
        // Engine created successfully
    }

    #[tokio::test]
    async fn test_chaos_metrics_recording() {
        let engine = ChaosEngine::new(ChaosConfig::default());
        
        engine.record_request(true, Duration::from_millis(100)).await;
        engine.record_request(true, Duration::from_millis(200)).await;
        engine.record_request(false, Duration::from_millis(150)).await;

        let metrics = engine.metrics().await;
        assert_eq!(metrics.requests_attempted, 3);
        assert_eq!(metrics.requests_succeeded, 2);
        assert_eq!(metrics.requests_failed, 1);
        assert!(metrics.error_rate > 0.3 && metrics.error_rate < 0.4);
    }

    #[tokio::test]
    async fn test_chaos_metrics_reset() {
        let engine = ChaosEngine::new(ChaosConfig::default());
        
        engine.record_request(true, Duration::from_millis(100)).await;
        engine.reset_metrics().await;

        let metrics = engine.metrics().await;
        assert_eq!(metrics.requests_attempted, 0);
    }

    #[test]
    fn test_failure_injection_probability() {
        let config = ChaosConfig {
            enabled: true,
            failure_rate: 0.0,
            ..Default::default()
        };
        let engine = ChaosEngine::new(config);

        // With 0% failure rate, should never inject failures
        let mut injected = 0;
        for _ in 0..100 {
            if engine.should_inject_failure() {
                injected += 1;
            }
        }
        assert_eq!(injected, 0);
    }
}

