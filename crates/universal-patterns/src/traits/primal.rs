// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Core Primal trait definition.

use std::collections::HashMap;

use crate::config::PrimalConfig;

use super::{HealthStatus, MetricValue, PrimalError, PrimalInfo, PrimalState};

/// Core primal trait - foundational interface for all primals
#[expect(
    async_fn_in_trait,
    reason = "internal trait — all impls are Send + Sync"
)]
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
        primal.start().await.expect("should succeed");
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

    /// Default `restart()` propagates errors from `stop()`.
    struct MockPrimalStopFails;

    impl MockPrimalStopFails {
        fn static_info() -> &'static PrimalInfo {
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

        fn static_config() -> &'static PrimalConfig {
            static CONFIG: std::sync::OnceLock<PrimalConfig> = std::sync::OnceLock::new();
            CONFIG.get_or_init(PrimalConfig::default)
        }
    }

    impl Primal for MockPrimalStopFails {
        fn info(&self) -> &PrimalInfo {
            Self::static_info()
        }

        async fn state(&self) -> PrimalState {
            PrimalState::Running
        }

        async fn start(&mut self) -> Result<(), PrimalError> {
            Ok(())
        }

        async fn stop(&mut self) -> Result<(), PrimalError> {
            Err(PrimalError::Shutdown("stop failed".into()))
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
            Self::static_config()
        }

        async fn update_config(&mut self, _config: PrimalConfig) -> Result<(), PrimalError> {
            Ok(())
        }

        async fn metrics(&self) -> Result<HashMap<String, MetricValue>, PrimalError> {
            Ok(HashMap::new())
        }

        async fn shutdown(&mut self) -> Result<(), PrimalError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_primal_restart_propagates_stop_error() {
        let mut primal = MockPrimalStopFails;
        let err = primal.restart().await.unwrap_err();
        assert!(matches!(err, PrimalError::Shutdown(ref m) if m == "stop failed"));
    }

    /// Default `restart()` propagates errors from `start()` after `stop()` succeeds.
    struct MockPrimalStartFailsAfterStop {
        inner: MockPrimalForTraitTest,
    }

    impl MockPrimalStartFailsAfterStop {
        fn new() -> Self {
            Self {
                inner: MockPrimalForTraitTest::new(),
            }
        }
    }

    impl Primal for MockPrimalStartFailsAfterStop {
        fn info(&self) -> &PrimalInfo {
            self.inner.info()
        }

        async fn state(&self) -> PrimalState {
            self.inner.state().await
        }

        async fn start(&mut self) -> Result<(), PrimalError> {
            self.inner.start().await?;
            Err(PrimalError::Internal("start failed on restart".into()))
        }

        async fn stop(&mut self) -> Result<(), PrimalError> {
            self.inner.stop().await
        }

        async fn health_check(&self) -> Result<HealthStatus, PrimalError> {
            self.inner.health_check().await
        }

        fn config(&self) -> &PrimalConfig {
            self.inner.config()
        }

        async fn update_config(&mut self, config: PrimalConfig) -> Result<(), PrimalError> {
            self.inner.update_config(config).await
        }

        async fn metrics(&self) -> Result<HashMap<String, MetricValue>, PrimalError> {
            self.inner.metrics().await
        }

        async fn shutdown(&mut self) -> Result<(), PrimalError> {
            self.inner.shutdown().await
        }
    }

    #[tokio::test]
    async fn test_primal_restart_propagates_start_error_after_stop() {
        let mut primal = MockPrimalStartFailsAfterStop::new();
        primal.inner.state = PrimalState::Running;

        let err = primal.restart().await.unwrap_err();
        assert!(matches!(err, PrimalError::Internal(ref m) if m == "start failed on restart"));
        assert_eq!(primal.inner.stop_count, 1);
    }

    /// Exercise every `Primal` method on the mock so async-trait shims and call paths stay covered.
    #[tokio::test]
    async fn test_primal_mock_exercises_full_trait_surface() {
        let mut primal = MockPrimalForTraitTest::new();

        assert_eq!(primal.info().name, "test");
        assert_eq!(primal.state().await, PrimalState::Stopped);
        primal.start().await.expect("start");
        assert_eq!(primal.state().await, PrimalState::Running);

        let health = primal.health_check().await.expect("health");
        assert_eq!(health.status, HealthState::Healthy);

        assert_eq!(
            primal.config().network.port,
            universal_constants::network::get_service_port("http")
        );
        primal
            .update_config(PrimalConfig::default())
            .await
            .expect("update_config");

        let m = primal.metrics().await.expect("metrics");
        assert!(m.is_empty());

        primal.shutdown().await.expect("shutdown");
        assert_eq!(primal.state().await, PrimalState::Stopped);
    }

    #[tokio::test]
    async fn test_primal_stop_directly() {
        let mut primal = MockPrimalForTraitTest::new();
        primal.start().await.expect("start");
        assert_eq!(primal.state().await, PrimalState::Running);
        assert_eq!(primal.start_count, 1);

        primal.stop().await.expect("stop");
        assert_eq!(primal.state().await, PrimalState::Stopped);
        assert_eq!(primal.stop_count, 1);
    }

    #[tokio::test]
    async fn test_primal_stop_from_stopped_is_ok() {
        let mut primal = MockPrimalForTraitTest::new();
        assert_eq!(primal.state().await, PrimalState::Stopped);
        primal
            .stop()
            .await
            .expect("stop from stopped should succeed");
        assert_eq!(primal.stop_count, 1);
        assert_eq!(primal.state().await, PrimalState::Stopped);
    }

    #[tokio::test]
    async fn test_primal_multiple_start_stop_cycles() {
        let mut primal = MockPrimalForTraitTest::new();

        for i in 1..=3 {
            primal.start().await.expect("start");
            assert_eq!(primal.state().await, PrimalState::Running);
            assert_eq!(primal.start_count, i);

            primal.stop().await.expect("stop");
            assert_eq!(primal.state().await, PrimalState::Stopped);
            assert_eq!(primal.stop_count, i);
        }
    }

    /// Mock where `health_check` returns `Err`.
    struct MockPrimalHealthFails;

    impl MockPrimalHealthFails {
        fn static_info() -> &'static PrimalInfo {
            static INFO: std::sync::OnceLock<PrimalInfo> = std::sync::OnceLock::new();
            INFO.get_or_init(|| PrimalInfo {
                name: "health-fail".to_string(),
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

        fn static_config() -> &'static PrimalConfig {
            static CONFIG: std::sync::OnceLock<PrimalConfig> = std::sync::OnceLock::new();
            CONFIG.get_or_init(PrimalConfig::default)
        }
    }

    impl Primal for MockPrimalHealthFails {
        fn info(&self) -> &PrimalInfo {
            Self::static_info()
        }

        async fn state(&self) -> PrimalState {
            PrimalState::Running
        }

        async fn start(&mut self) -> Result<(), PrimalError> {
            Ok(())
        }

        async fn stop(&mut self) -> Result<(), PrimalError> {
            Ok(())
        }

        async fn health_check(&self) -> Result<HealthStatus, PrimalError> {
            Err(PrimalError::Internal("health check failed".into()))
        }

        fn config(&self) -> &PrimalConfig {
            Self::static_config()
        }

        async fn update_config(&mut self, _config: PrimalConfig) -> Result<(), PrimalError> {
            Err(PrimalError::Configuration("config update rejected".into()))
        }

        async fn metrics(&self) -> Result<HashMap<String, MetricValue>, PrimalError> {
            Err(PrimalError::Internal("metrics unavailable".into()))
        }

        async fn shutdown(&mut self) -> Result<(), PrimalError> {
            Err(PrimalError::Shutdown("shutdown failed".into()))
        }
    }

    #[tokio::test]
    async fn test_primal_health_check_error() {
        let primal = MockPrimalHealthFails;
        let err = primal.health_check().await.unwrap_err();
        assert!(matches!(err, PrimalError::Internal(ref m) if m == "health check failed"));
    }

    #[tokio::test]
    async fn test_primal_update_config_error() {
        let mut primal = MockPrimalHealthFails;
        let err = primal
            .update_config(PrimalConfig::default())
            .await
            .unwrap_err();
        assert!(matches!(err, PrimalError::Configuration(ref m) if m == "config update rejected"));
    }

    #[tokio::test]
    async fn test_primal_metrics_error() {
        let primal = MockPrimalHealthFails;
        let err = primal.metrics().await.unwrap_err();
        assert!(matches!(err, PrimalError::Internal(ref m) if m == "metrics unavailable"));
    }

    #[tokio::test]
    async fn test_primal_shutdown_error() {
        let mut primal = MockPrimalHealthFails;
        let err = primal.shutdown().await.unwrap_err();
        assert!(matches!(err, PrimalError::Shutdown(ref m) if m == "shutdown failed"));
    }

    #[tokio::test]
    async fn test_primal_restart_error_propagation_from_health_fails_mock() {
        let mut primal = MockPrimalHealthFails;
        let result = primal.restart().await;
        assert!(
            result.is_ok(),
            "restart should succeed when start/stop succeed"
        );
    }

    #[tokio::test]
    async fn test_primal_info_fields() {
        let primal = MockPrimalForTraitTest::new();
        let info = primal.info();
        assert_eq!(info.name, "test");
        assert_eq!(info.version, "1.0");
        assert_eq!(info.primal_type, PrimalType::Coordinator);
        assert_eq!(info.description, "test");
        assert!(info.tags.is_empty());
        assert!(info.capabilities.is_empty());
    }

    #[tokio::test]
    async fn test_primal_config_default_fields() {
        let primal = MockPrimalForTraitTest::new();
        let config = primal.config();
        assert_eq!(
            config.network.port,
            universal_constants::network::get_service_port("http")
        );
    }
}
