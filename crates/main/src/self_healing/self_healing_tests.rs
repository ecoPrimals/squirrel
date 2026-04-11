// SPDX-License-Identifier: AGPL-3.0-or-later
// ORC-Notice: Self-healing mechanics licensed under ORC
// Copyright (C) 2026 ecoPrimals Contributors

//! Unit tests for the self-healing coordinator (`SelfHealingManager` and related types).

use super::*;

fn create_test_manager() -> SelfHealingManager {
    SelfHealingManager::new(SelfHealingConfig::default())
}

#[test]
fn test_default_config() {
    let config = SelfHealingConfig::default();
    assert_eq!(config.check_interval_seconds, 30);
    assert_eq!(config.max_failures, 3);
    assert!(config.enable_auto_recovery);
}

#[test]
fn test_custom_config() {
    let config = SelfHealingConfig {
        check_interval_seconds: 60,
        max_failures: 5,
        enable_auto_recovery: false,
    };
    assert_eq!(config.check_interval_seconds, 60);
    assert_eq!(config.max_failures, 5);
    assert!(!config.enable_auto_recovery);
}

#[test]
fn test_config_serde_roundtrip() {
    let config = SelfHealingConfig::default();
    let json = serde_json::to_string(&config).expect("should succeed");
    let deserialized: SelfHealingConfig = serde_json::from_str(&json).expect("should succeed");
    assert_eq!(
        deserialized.check_interval_seconds,
        config.check_interval_seconds
    );
    assert_eq!(deserialized.max_failures, config.max_failures);
    assert_eq!(
        deserialized.enable_auto_recovery,
        config.enable_auto_recovery
    );
}

#[test]
fn test_manager_creation() {
    let manager = create_test_manager();
    assert!(manager.component_health.is_empty());
}

#[test]
fn test_register_component() {
    let mut manager = create_test_manager();
    manager.register_component("test_component");

    assert_eq!(manager.component_health.len(), 1);
    let health = manager
        .get_component_health("test_component")
        .expect("should succeed");
    assert_eq!(health.component_id, "test_component");
    assert_eq!(health.status, HealthStatus::Unknown);
    assert_eq!(health.failure_count, 0);
    assert_eq!(health.message, "Newly registered");
}

#[test]
fn test_register_multiple_components() {
    let mut manager = create_test_manager();
    manager.register_component("comp_a");
    manager.register_component("comp_b");
    manager.register_component("comp_c");

    assert_eq!(manager.component_health.len(), 3);
    assert!(manager.get_component_health("comp_a").is_some());
    assert!(manager.get_component_health("comp_b").is_some());
    assert!(manager.get_component_health("comp_c").is_some());
}

#[test]
fn test_get_nonexistent_component() {
    let manager = create_test_manager();
    assert!(manager.get_component_health("nonexistent").is_none());
}

#[test]
fn test_update_component_healthy() {
    let mut manager = create_test_manager();
    manager.register_component("test");

    manager.update_component_health("test", HealthStatus::Healthy, "All good");

    let health = manager
        .get_component_health("test")
        .expect("should succeed");
    assert_eq!(health.status, HealthStatus::Healthy);
    assert_eq!(health.message, "All good");
    assert_eq!(health.failure_count, 0);
}

#[test]
fn test_update_component_failed() {
    let mut manager = create_test_manager();
    manager.register_component("test");

    manager.update_component_health("test", HealthStatus::Failed, "Connection lost");

    let health = manager
        .get_component_health("test")
        .expect("should succeed");
    assert_eq!(health.status, HealthStatus::Failed);
    assert_eq!(health.failure_count, 1);
}

#[test]
fn test_failure_count_increments() {
    let mut manager = create_test_manager();
    manager.register_component("test");

    manager.update_component_health("test", HealthStatus::Failed, "Fail 1");
    manager.update_component_health("test", HealthStatus::Failed, "Fail 2");
    manager.update_component_health("test", HealthStatus::Failed, "Fail 3");

    let health = manager
        .get_component_health("test")
        .expect("should succeed");
    assert_eq!(health.failure_count, 3);
}

#[test]
fn test_failure_count_resets_on_healthy() {
    let mut manager = create_test_manager();
    manager.register_component("test");

    manager.update_component_health("test", HealthStatus::Failed, "Fail");
    manager.update_component_health("test", HealthStatus::Failed, "Fail");
    assert_eq!(
        manager
            .get_component_health("test")
            .expect("should succeed")
            .failure_count,
        2
    );

    manager.update_component_health("test", HealthStatus::Healthy, "Recovered");
    assert_eq!(
        manager
            .get_component_health("test")
            .expect("should succeed")
            .failure_count,
        0
    );
}

#[test]
fn test_degraded_does_not_change_failure_count() {
    let mut manager = create_test_manager();
    manager.register_component("test");

    manager.update_component_health("test", HealthStatus::Failed, "Fail");
    assert_eq!(
        manager
            .get_component_health("test")
            .expect("should succeed")
            .failure_count,
        1
    );

    manager.update_component_health("test", HealthStatus::Degraded, "Degraded");
    assert_eq!(
        manager
            .get_component_health("test")
            .expect("should succeed")
            .failure_count,
        1
    );
}

#[test]
fn test_update_nonexistent_component_is_noop() {
    let mut manager = create_test_manager();
    // Should not panic
    manager.update_component_health("nonexistent", HealthStatus::Healthy, "OK");
    assert!(manager.get_component_health("nonexistent").is_none());
}

#[test]
fn test_is_system_healthy_empty() {
    let manager = create_test_manager();
    // No components = all healthy (vacuously true)
    assert!(manager.is_system_healthy());
}

#[test]
fn test_is_system_healthy_all_healthy() {
    let mut manager = create_test_manager();
    manager.register_component("a");
    manager.register_component("b");
    manager.update_component_health("a", HealthStatus::Healthy, "OK");
    manager.update_component_health("b", HealthStatus::Healthy, "OK");

    assert!(manager.is_system_healthy());
}

#[test]
fn test_is_system_healthy_with_failed() {
    let mut manager = create_test_manager();
    manager.register_component("a");
    manager.register_component("b");
    manager.update_component_health("a", HealthStatus::Healthy, "OK");
    manager.update_component_health("b", HealthStatus::Failed, "Down");

    assert!(!manager.is_system_healthy());
}

#[test]
fn test_is_system_healthy_with_degraded() {
    let mut manager = create_test_manager();
    manager.register_component("a");
    manager.update_component_health("a", HealthStatus::Degraded, "Slow");

    assert!(!manager.is_system_healthy());
}

#[test]
fn test_system_health_summary_empty() {
    let manager = create_test_manager();
    let summary = manager.get_system_health_summary();

    // With 0 total components, degraded_count(0) < total/2(0) is false
    assert!(!summary.overall_healthy);
    assert_eq!(summary.total_components, 0);
    assert_eq!(summary.healthy_count, 0);
    assert_eq!(summary.degraded_count, 0);
    assert_eq!(summary.failed_count, 0);
}

#[test]
fn test_system_health_summary_mixed() {
    let mut manager = create_test_manager();
    manager.register_component("a");
    manager.register_component("b");
    manager.register_component("c");
    manager.register_component("d");

    manager.update_component_health("a", HealthStatus::Healthy, "OK");
    manager.update_component_health("b", HealthStatus::Healthy, "OK");
    manager.update_component_health("c", HealthStatus::Degraded, "Slow");
    manager.update_component_health("d", HealthStatus::Failed, "Down");

    let summary = manager.get_system_health_summary();
    assert_eq!(summary.total_components, 4);
    assert_eq!(summary.healthy_count, 2);
    assert_eq!(summary.degraded_count, 1);
    assert_eq!(summary.failed_count, 1);
    assert!(!summary.overall_healthy); // Has failed component
}

#[test]
fn test_system_health_summary_all_degraded() {
    let mut manager = create_test_manager();
    manager.register_component("a");
    manager.register_component("b");
    manager.update_component_health("a", HealthStatus::Degraded, "Slow");
    manager.update_component_health("b", HealthStatus::Degraded, "Slow");

    let summary = manager.get_system_health_summary();
    // All degraded (2/2 >= half) => not overall healthy
    assert!(!summary.overall_healthy);
}

#[test]
fn test_health_recommendations_all_healthy() {
    let mut manager = create_test_manager();
    manager.register_component("a");
    manager.update_component_health("a", HealthStatus::Healthy, "OK");

    let recs = manager.get_health_recommendations();
    assert_eq!(recs.len(), 1);
    assert!(recs[0].contains("All AI coordination components are healthy"));
}

#[test]
fn test_health_recommendations_failed() {
    let mut manager = create_test_manager();
    manager.register_component("a");
    manager.update_component_health("a", HealthStatus::Failed, "Down");

    let recs = manager.get_health_recommendations();
    assert!(recs.iter().any(|r| r.contains("failed")));
}

#[test]
fn test_health_recommendations_degraded() {
    let mut manager = create_test_manager();
    manager.register_component("a");
    manager.update_component_health("a", HealthStatus::Degraded, "Slow");

    let recs = manager.get_health_recommendations();
    assert!(recs.iter().any(|r| r.contains("degraded")));
}

#[test]
fn test_health_recommendations_below_80_percent() {
    let mut manager = create_test_manager();
    for i in 0..5 {
        manager.register_component(&format!("comp_{i}"));
    }
    // Only 1/5 healthy = 20%
    manager.update_component_health("comp_0", HealthStatus::Healthy, "OK");
    manager.update_component_health("comp_1", HealthStatus::Failed, "Down");
    manager.update_component_health("comp_2", HealthStatus::Failed, "Down");
    manager.update_component_health("comp_3", HealthStatus::Failed, "Down");
    manager.update_component_health("comp_4", HealthStatus::Failed, "Down");

    let recs = manager.get_health_recommendations();
    assert!(recs.iter().any(|r| r.contains("below 80%")));
}

#[test]
fn test_get_all_component_health() {
    let mut manager = create_test_manager();
    manager.register_component("a");
    manager.register_component("b");

    let all = manager.get_all_component_health();
    assert_eq!(all.len(), 2);
    assert!(all.contains_key("a"));
    assert!(all.contains_key("b"));
}

#[tokio::test]
async fn test_get_health_status() {
    let mut manager = create_test_manager();
    manager.register_component("a");

    let status = manager.get_health_status().await;
    assert_eq!(status.len(), 1);
    assert!(status.contains_key("a"));
}

#[tokio::test]
async fn test_perform_health_check() {
    let mut manager = create_test_manager();
    manager.register_component("ai_coordinator");
    manager.register_component("security_adapter");

    let result = manager.perform_health_check().await;
    assert!(result.is_ok());

    // All simulated health checks return true → healthy
    let health = manager
        .get_component_health("ai_coordinator")
        .expect("should succeed");
    assert_eq!(health.status, HealthStatus::Healthy);

    let health = manager
        .get_component_health("security_adapter")
        .expect("should succeed");
    assert_eq!(health.status, HealthStatus::Healthy);
}

#[tokio::test]
async fn test_initialize_self_healing() {
    let manager = initialize_self_healing().await.expect("should succeed");
    assert_eq!(manager.component_health.len(), 5);
    assert!(manager.get_component_health("ai_coordinator").is_some());
    assert!(manager.get_component_health("security_adapter").is_some());
    assert!(
        manager
            .get_component_health("orchestration_adapter")
            .is_some()
    );
    assert!(manager.get_component_health("storage_adapter").is_some());
    assert!(manager.get_component_health("compute_adapter").is_some());
}

#[test]
fn test_health_status_serde_roundtrip() {
    let statuses = vec![
        HealthStatus::Healthy,
        HealthStatus::Degraded,
        HealthStatus::Failed,
        HealthStatus::Unknown,
    ];
    for status in statuses {
        let json = serde_json::to_string(&status).expect("should succeed");
        let deserialized: HealthStatus = serde_json::from_str(&json).expect("should succeed");
        assert_eq!(deserialized, status);
    }
}

#[test]
fn test_component_health_serde_roundtrip() {
    let health = ComponentHealth {
        component_id: "test".to_string(),
        status: HealthStatus::Healthy,
        last_check: chrono::Utc::now(),
        message: "All good".to_string(),
        failure_count: 0,
    };
    let json = serde_json::to_string(&health).expect("should succeed");
    let deserialized: ComponentHealth = serde_json::from_str(&json).expect("should succeed");
    assert_eq!(deserialized.component_id, "test");
    assert_eq!(deserialized.status, HealthStatus::Healthy);
    assert_eq!(deserialized.failure_count, 0);
}

#[test]
fn test_system_health_summary_serde_roundtrip() {
    let summary = SystemHealthSummary {
        overall_healthy: true,
        total_components: 5,
        healthy_count: 4,
        degraded_count: 1,
        failed_count: 0,
        last_update: chrono::Utc::now(),
    };
    let json = serde_json::to_string(&summary).expect("should succeed");
    let deserialized: SystemHealthSummary = serde_json::from_str(&json).expect("should succeed");
    assert!(deserialized.overall_healthy);
    assert_eq!(deserialized.total_components, 5);
    assert_eq!(deserialized.healthy_count, 4);
}

#[test]
fn test_auto_recovery_triggered_at_max_failures() {
    let config = SelfHealingConfig {
        check_interval_seconds: 30,
        max_failures: 2,
        enable_auto_recovery: true,
    };
    let mut manager = SelfHealingManager::new(config);
    manager.register_component("test");

    // Trigger failures up to and beyond max_failures
    manager.update_component_health("test", HealthStatus::Failed, "Fail 1");
    manager.update_component_health("test", HealthStatus::Failed, "Fail 2");
    manager.update_component_health("test", HealthStatus::Failed, "Fail 3");

    // Verify the failure count is tracked
    let health = manager
        .get_component_health("test")
        .expect("should succeed");
    assert_eq!(health.failure_count, 3);
}

#[test]
fn test_auto_recovery_disabled() {
    let config = SelfHealingConfig {
        check_interval_seconds: 30,
        max_failures: 1,
        enable_auto_recovery: false,
    };
    let mut manager = SelfHealingManager::new(config);
    manager.register_component("test");

    // Should not panic even with auto_recovery disabled
    manager.update_component_health("test", HealthStatus::Failed, "Fail");
    manager.update_component_health("test", HealthStatus::Failed, "Fail again");

    let health = manager
        .get_component_health("test")
        .expect("should succeed");
    assert_eq!(health.failure_count, 2);
}
