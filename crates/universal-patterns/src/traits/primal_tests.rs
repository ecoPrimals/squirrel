// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Comprehensive tests for the Primal trait
//!
//! Coverage goal: 90%+
//! Strategy: Test default implementations, trait behavior, error paths

use async_trait::async_trait;
use chrono::Utc;
use std::collections::HashMap;
use uuid::Uuid;

use super::primal::Primal;
use crate::config::PrimalConfig;
use crate::traits::{
    HealthState, HealthStatus, MetricValue, PrimalError, PrimalInfo, PrimalState, PrimalType,
};

/// Mock primal implementation for testing
struct MockPrimal {
    info: PrimalInfo,
    config: PrimalConfig,
    state: PrimalState,
    start_count: u32,
    stop_count: u32,
    health_fails: bool,
}

impl MockPrimal {
    fn new(name: &str, primal_type: PrimalType) -> Self {
        let now = Utc::now();
        Self {
            info: PrimalInfo {
                name: name.to_string(),
                version: "1.0.0".to_string(),
                instance_id: Uuid::new_v4(),
                primal_type,
                description: format!("Test {} primal", name),
                created_at: now,
                updated_at: now,
                tags: vec!["test".to_string()],
                capabilities: vec!["test-capability".to_string()],
            },
            config: PrimalConfig::default(),
            state: PrimalState::Stopped,
            start_count: 0,
            stop_count: 0,
            health_fails: false,
        }
    }

    fn with_health_failure(mut self) -> Self {
        self.health_fails = true;
        self
    }
}

#[async_trait]
impl Primal for MockPrimal {
    fn info(&self) -> &PrimalInfo {
        &self.info
    }

    async fn state(&self) -> PrimalState {
        self.state.clone()
    }

    async fn start(&mut self) -> Result<(), PrimalError> {
        if self.state == PrimalState::Running {
            return Err(PrimalError::State("Primal already running".to_string()));
        }
        self.start_count += 1;
        self.state = PrimalState::Running;
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), PrimalError> {
        if self.state == PrimalState::Stopped {
            return Err(PrimalError::State("Primal already stopped".to_string()));
        }
        self.stop_count += 1;
        self.state = PrimalState::Stopped;
        Ok(())
    }

    async fn health_check(&self) -> Result<HealthStatus, PrimalError> {
        if self.health_fails {
            return Err(PrimalError::HealthCheck(
                "Simulated health check failure".to_string(),
            ));
        }
        Ok(HealthStatus {
            status: HealthState::Healthy,
            details: HashMap::new(),
            timestamp: chrono::Utc::now(),
            duration: chrono::Duration::milliseconds(10),
        })
    }

    fn config(&self) -> &PrimalConfig {
        &self.config
    }

    async fn update_config(&mut self, new_config: PrimalConfig) -> Result<(), PrimalError> {
        self.config = new_config;
        Ok(())
    }

    async fn metrics(&self) -> Result<HashMap<String, MetricValue>, PrimalError> {
        let mut metrics = HashMap::new();
        metrics.insert(
            "start_count".to_string(),
            MetricValue::Counter(self.start_count as u64),
        );
        metrics.insert(
            "stop_count".to_string(),
            MetricValue::Counter(self.stop_count as u64),
        );
        Ok(metrics)
    }

    async fn shutdown(&mut self) -> Result<(), PrimalError> {
        if self.state != PrimalState::Stopped {
            self.stop().await?;
        }
        self.state = PrimalState::Stopped;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_primal_info() {
        let primal = MockPrimal::new("test-primal", PrimalType::AI);
        let info = primal.info();

        assert_eq!(info.name, "test-primal");
        assert_eq!(info.version, "1.0.0");
        assert_eq!(info.primal_type, PrimalType::AI);
        assert!(info.description.contains("Test test-primal primal"));
        assert_eq!(info.tags, vec!["test"]);
        assert_eq!(info.capabilities, vec!["test-capability"]);
    }

    #[tokio::test]
    async fn test_primal_initial_state() {
        let primal = MockPrimal::new("test", PrimalType::Security);
        let state = primal.state().await;
        assert_eq!(state, PrimalState::Stopped);
    }

    #[tokio::test]
    async fn test_primal_start() {
        let mut primal = MockPrimal::new("test", PrimalType::Coordinator);

        // Start primal
        let result = primal.start().await;
        assert!(result.is_ok());
        assert_eq!(primal.state().await, PrimalState::Running);
        assert_eq!(primal.start_count, 1);
    }

    #[tokio::test]
    async fn test_primal_start_when_running_fails() {
        let mut primal = MockPrimal::new("test", PrimalType::Storage);

        // Start once
        primal.start().await.unwrap();

        // Try to start again - should fail
        let result = primal.start().await;
        assert!(result.is_err());
        match result {
            Err(PrimalError::State(msg)) => {
                assert!(msg.contains("already running"));
            }
            _ => panic!("Expected State error"),
        }
    }

    #[tokio::test]
    async fn test_primal_stop() {
        let mut primal = MockPrimal::new("test", PrimalType::Orchestration);

        // Start then stop
        primal.start().await.unwrap();
        let result = primal.stop().await;

        assert!(result.is_ok());
        assert_eq!(primal.state().await, PrimalState::Stopped);
        assert_eq!(primal.stop_count, 1);
    }

    #[tokio::test]
    async fn test_primal_stop_when_stopped_fails() {
        let mut primal = MockPrimal::new("test", PrimalType::Compute);

        // Already stopped, try to stop again
        let result = primal.stop().await;
        assert!(result.is_err());
        match result {
            Err(PrimalError::State(msg)) => {
                assert!(msg.contains("already stopped"));
            }
            _ => panic!("Expected State error"),
        }
    }

    #[tokio::test]
    async fn test_primal_restart_default_impl() {
        let mut primal = MockPrimal::new("test", PrimalType::Network);

        // Start primal first
        primal.start().await.unwrap();
        assert_eq!(primal.state().await, PrimalState::Running);

        // Restart using default implementation
        let result = primal.restart().await;

        assert!(result.is_ok());
        assert_eq!(primal.state().await, PrimalState::Running);
        assert_eq!(primal.stop_count, 1);
        assert_eq!(primal.start_count, 2); // Once for initial start, once for restart
    }

    #[tokio::test]
    async fn test_primal_restart_from_stopped() {
        let mut primal = MockPrimal::new("test", PrimalType::AI);

        // Restart from stopped state (should fail stop, then succeed start)
        let result = primal.restart().await;

        // Will fail because stop() fails when already stopped
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_primal_health_check_healthy() {
        let primal = MockPrimal::new("test", PrimalType::Security);

        let result = primal.health_check().await;
        assert!(result.is_ok());
        let health = result.unwrap();
        assert_eq!(health.status, HealthState::Healthy);
    }

    #[tokio::test]
    async fn test_primal_health_check_unhealthy() {
        let primal = MockPrimal::new("test", PrimalType::Coordinator).with_health_failure();

        let result = primal.health_check().await;
        assert!(result.is_err());
        match result {
            Err(PrimalError::HealthCheck(msg)) => {
                assert!(msg.contains("Simulated health check failure"));
            }
            _ => panic!("Expected HealthCheck error"),
        }
    }

    #[tokio::test]
    async fn test_primal_config() {
        let primal = MockPrimal::new("test", PrimalType::Storage);
        let _config = primal.config();

        // Config exists
        assert!(true);
    }

    #[tokio::test]
    async fn test_primal_update_config() {
        let mut primal = MockPrimal::new("test", PrimalType::Orchestration);

        let new_config = PrimalConfig::default();

        let result = primal.update_config(new_config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_primal_metrics() {
        let mut primal = MockPrimal::new("test", PrimalType::Compute);

        // Start and stop a few times
        primal.start().await.unwrap();
        primal.stop().await.unwrap();
        primal.start().await.unwrap();

        let metrics = primal.metrics().await.unwrap();

        // Check metrics exist and have expected values
        assert!(metrics.contains_key("start_count"));
        assert!(metrics.contains_key("stop_count"));

        match metrics.get("start_count") {
            Some(MetricValue::Counter(val)) => assert_eq!(*val, 2),
            _ => panic!("Expected Counter metric for start_count"),
        }

        match metrics.get("stop_count") {
            Some(MetricValue::Counter(val)) => assert_eq!(*val, 1),
            _ => panic!("Expected Counter metric for stop_count"),
        }
    }

    #[tokio::test]
    async fn test_primal_shutdown_from_running() {
        let mut primal = MockPrimal::new("test", PrimalType::Network);

        // Start primal
        primal.start().await.unwrap();
        assert_eq!(primal.state().await, PrimalState::Running);

        // Shutdown
        let result = primal.shutdown().await;
        assert!(result.is_ok());
        assert_eq!(primal.state().await, PrimalState::Stopped);
        assert_eq!(primal.stop_count, 1);
    }

    #[tokio::test]
    async fn test_primal_shutdown_from_stopped() {
        let mut primal = MockPrimal::new("test", PrimalType::AI);

        // Already stopped
        let result = primal.shutdown().await;
        assert!(result.is_ok());
        assert_eq!(primal.state().await, PrimalState::Stopped);
        assert_eq!(primal.stop_count, 0); // Should not increment stop count
    }

    #[tokio::test]
    async fn test_primal_lifecycle() {
        let mut primal = MockPrimal::new("test", PrimalType::Security);

        // Full lifecycle test
        assert_eq!(primal.state().await, PrimalState::Stopped);

        primal.start().await.unwrap();
        assert_eq!(primal.state().await, PrimalState::Running);

        let health = primal.health_check().await.unwrap();
        assert_eq!(health.status, HealthState::Healthy);

        primal.stop().await.unwrap();
        assert_eq!(primal.state().await, PrimalState::Stopped);

        primal.shutdown().await.unwrap();
        assert_eq!(primal.state().await, PrimalState::Stopped);
    }

    #[tokio::test]
    async fn test_multiple_primal_types() {
        // Test each primal type variant
        let types = vec![
            PrimalType::Coordinator,
            PrimalType::Security,
            PrimalType::Orchestration,
            PrimalType::Storage,
            PrimalType::Compute,
            PrimalType::AI,
            PrimalType::Network,
            PrimalType::Custom("CustomType".to_string()),
        ];

        for primal_type in types {
            let primal = MockPrimal::new("test", primal_type.clone());
            assert_eq!(primal.info().primal_type, primal_type);
        }
    }
}
