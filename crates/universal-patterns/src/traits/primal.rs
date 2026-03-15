// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Core Primal trait definition.

use async_trait::async_trait;
use std::collections::HashMap;

use crate::config::PrimalConfig;

use super::{HealthStatus, MetricValue, PrimalError, PrimalInfo, PrimalState};

/// Core primal trait - foundational interface for all primals
#[async_trait]
pub trait Primal: Send + Sync {
    /// Get primal information
    fn info(&self) -> &PrimalInfo;

    /// Get current state of the primal
    async fn state(&self) -> PrimalState;

    /// Start the primal
    async fn start(&mut self) -> Result<(), PrimalError>;

    /// Stop the primal
    async fn stop(&mut self) -> Result<(), PrimalError>;

    /// Restart the primal
    async fn restart(&mut self) -> Result<(), PrimalError> {
        self.stop().await?;
        self.start().await?;
        Ok(())
    }

    /// Check if the primal is healthy
    async fn health_check(&self) -> Result<HealthStatus, PrimalError>;

    /// Get configuration
    fn config(&self) -> &PrimalConfig;

    /// Update configuration
    async fn update_config(&mut self, config: PrimalConfig) -> Result<(), PrimalError>;

    /// Get metrics
    async fn metrics(&self) -> Result<HashMap<String, MetricValue>, PrimalError>;

    /// Handle shutdown signal
    async fn shutdown(&mut self) -> Result<(), PrimalError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use chrono::Utc;
    use std::collections::HashMap;
    use uuid::Uuid;

    use crate::config::PrimalConfig;
    use crate::traits::{HealthState, MetricValue, PrimalInfo, PrimalState, PrimalType};

    /// Minimal mock for testing Primal trait default implementations
    struct MockPrimalForTraitTest {
        state: PrimalState,
        start_count: u32,
        stop_count: u32,
    }

    impl MockPrimalForTraitTest {
        fn new() -> Self {
            Self {
                state: PrimalState::Stopped,
                start_count: 0,
                stop_count: 0,
            }
        }
    }

    #[async_trait]
    impl Primal for MockPrimalForTraitTest {
        fn info(&self) -> &PrimalInfo {
            static INFO: std::sync::OnceLock<PrimalInfo> = std::sync::OnceLock::new();
            INFO.get_or_init(|| PrimalInfo {
                name: "test".to_string(),
                version: "1.0".to_string(),
                instance_id: Uuid::new_v4(),
                primal_type: PrimalType::Coordinator,
                description: "test".to_string(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
                tags: vec![],
                capabilities: vec![],
            })
        }

        async fn state(&self) -> PrimalState {
            self.state.clone()
        }

        async fn start(&mut self) -> Result<(), PrimalError> {
            self.start_count += 1;
            self.state = PrimalState::Running;
            Ok(())
        }

        async fn stop(&mut self) -> Result<(), PrimalError> {
            self.stop_count += 1;
            self.state = PrimalState::Stopped;
            Ok(())
        }

        async fn health_check(&self) -> Result<HealthStatus, PrimalError> {
            Ok(HealthStatus {
                status: HealthState::Healthy,
                details: HashMap::new(),
                timestamp: Utc::now(),
                duration: chrono::Duration::zero(),
            })
        }

        fn config(&self) -> &PrimalConfig {
            static CONFIG: std::sync::OnceLock<PrimalConfig> = std::sync::OnceLock::new();
            CONFIG.get_or_init(PrimalConfig::default)
        }

        async fn update_config(&mut self, _config: PrimalConfig) -> Result<(), PrimalError> {
            Ok(())
        }

        async fn metrics(&self) -> Result<HashMap<String, MetricValue>, PrimalError> {
            Ok(HashMap::new())
        }

        async fn shutdown(&mut self) -> Result<(), PrimalError> {
            self.state = PrimalState::Stopped;
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_primal_default_restart_implementation() {
        let mut primal = MockPrimalForTraitTest::new();
        primal.start().await.unwrap();
        assert_eq!(primal.state().await, PrimalState::Running);

        // restart() uses default impl: stop then start
        let result = primal.restart().await;
        assert!(result.is_ok());
        assert_eq!(primal.state().await, PrimalState::Running);
        assert_eq!(primal.start_count, 2);
        assert_eq!(primal.stop_count, 1);
    }

    #[tokio::test]
    async fn test_primal_restart_from_stopped_state() {
        let mut primal = MockPrimalForTraitTest::new();
        assert_eq!(primal.state().await, PrimalState::Stopped);

        let result = primal.restart().await;
        assert!(result.is_ok());
        assert_eq!(primal.state().await, PrimalState::Running);
        assert_eq!(primal.start_count, 1);
        assert_eq!(primal.stop_count, 1);
    }
}
