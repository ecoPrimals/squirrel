// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

use super::*;
use std::collections::HashMap;
use uuid::Uuid;

#[tokio::test]
async fn test_security_monitoring_system_new() {
    let config = SecurityMonitoringConfig::default();
    let system = SecurityMonitoringSystem::new(config);

    let stats = system.get_statistics().await;
    assert_eq!(stats.total_events, 0);
    assert_eq!(stats.alerts_generated, 0);
}

#[tokio::test]
async fn test_security_monitoring_system_record_event() {
    let config = SecurityMonitoringConfig::default();
    let system = SecurityMonitoringSystem::new(config);

    let correlation_id = CorrelationId::new();
    let event = SecurityEvent::new(
        SecurityEventType::Authentication {
            success: true,
            user_id: Some("user123".to_string()),
            method: "password".to_string(),
        },
        "192.168.1.1".to_string(),
        EventSeverity::Info,
        "auth_service".to_string(),
        correlation_id,
    );

    system.record_event(event).await;

    let buffer_len = system.test_get_buffer_len().await;
    assert_eq!(buffer_len, 1);
}

#[tokio::test]
async fn test_security_monitoring_system_get_active_alerts() {
    let config = SecurityMonitoringConfig::default();
    let system = SecurityMonitoringSystem::new(config);

    let alerts = system.get_active_alerts().await;
    assert_eq!(alerts.len(), 0);
}

#[tokio::test]
async fn test_monitoring_with_disabled_real_time() {
    let config = SecurityMonitoringConfig {
        enable_real_time_monitoring: false,
        ..Default::default()
    };
    let system = SecurityMonitoringSystem::new(config);

    let correlation_id = CorrelationId::new();
    let event = SecurityEvent::new(
        SecurityEventType::RateLimitViolation {
            client_ip: "10.0.0.1".to_string(),
            endpoint: "/api".to_string(),
            violation_count: 5,
        },
        "10.0.0.1".to_string(),
        EventSeverity::Warning,
        "rate_limiter".to_string(),
        correlation_id,
    );

    system.record_event(event).await;
    let buffer_len = system.test_get_buffer_len().await;
    assert_eq!(buffer_len, 1);
}

#[tokio::test]
async fn test_monitoring_event_with_nil_id() {
    let config = SecurityMonitoringConfig::default();
    let system = SecurityMonitoringSystem::new(config);

    let mut event = SecurityEvent::new(
        SecurityEventType::SuspiciousActivity {
            client_ip: "192.168.1.1".to_string(),
            activity_type: "scan".to_string(),
            details: HashMap::new(),
        },
        "192.168.1.1".to_string(),
        EventSeverity::High,
        "detector".to_string(),
        CorrelationId::new(),
    );
    event.event_id = Uuid::nil();

    system.record_event(event).await;
    let buffer_len = system.test_get_buffer_len().await;
    assert_eq!(buffer_len, 1);
}

#[tokio::test]
async fn test_monitoring_buffer_flush_on_size() {
    let config = SecurityMonitoringConfig {
        enable_real_time_monitoring: false,
        event_buffer_size: 3,
        ..Default::default()
    };
    let system = SecurityMonitoringSystem::new(config);

    for i in 0..5 {
        let event = SecurityEvent::new(
            SecurityEventType::Authentication {
                success: true,
                user_id: Some(format!("user{i}")),
                method: "password".to_string(),
            },
            "192.168.1.1".to_string(),
            EventSeverity::Info,
            "auth".to_string(),
            CorrelationId::new(),
        );
        system.record_event(event).await;
    }

    let buffer_len = system.test_get_buffer_len().await;
    assert!(buffer_len <= 3);
}

#[tokio::test]
async fn test_monitoring_alert_thresholds_config() {
    let config = SecurityMonitoringConfig::default();
    assert_eq!(config.alert_thresholds.failed_auth_per_hour, 10);
    assert!(config.alert_thresholds.max_failed_requests_ratio > 0.0);
}

#[tokio::test]
async fn test_monitoring_shutdown_phases() {
    let config = SecurityMonitoringConfig {
        enable_real_time_monitoring: false,
        ..Default::default()
    };
    let system = SecurityMonitoringSystem::new(config);

    assert!(system.shutdown(ShutdownPhase::StopAccepting).await.is_ok());
    assert!(system.shutdown(ShutdownPhase::DrainRequests).await.is_ok());
    assert!(
        system
            .shutdown(ShutdownPhase::CloseConnections)
            .await
            .is_ok()
    );
    assert!(
        system
            .shutdown(ShutdownPhase::CleanupResources)
            .await
            .is_ok()
    );
    assert!(system.shutdown(ShutdownPhase::ShutdownTasks).await.is_ok());
    assert!(system.shutdown(ShutdownPhase::FinalCleanup).await.is_ok());
}

#[tokio::test]
async fn test_monitoring_component_name() {
    let config = SecurityMonitoringConfig::default();
    let system = SecurityMonitoringSystem::new(config);
    assert_eq!(system.component_name(), "security_monitoring");
}

#[tokio::test]
async fn test_monitoring_estimated_shutdown_time() {
    let config = SecurityMonitoringConfig::default();
    let system = SecurityMonitoringSystem::new(config);
    assert_eq!(system.estimated_shutdown_time(), Duration::from_secs(10));
}

#[tokio::test]
async fn test_start_noop_when_real_time_disabled() {
    let config = SecurityMonitoringConfig {
        enable_real_time_monitoring: false,
        ..Default::default()
    };
    let system = SecurityMonitoringSystem::new(config);
    assert!(system.start().await.is_ok());
}

#[tokio::test]
async fn test_failed_auth_event_generates_alert_after_start() {
    let config = SecurityMonitoringConfig {
        enable_behavioral_analysis: false,
        ..Default::default()
    };
    let system = SecurityMonitoringSystem::new(config);
    system.start().await.expect("start");

    let event = SecurityEvent::new(
        SecurityEventType::Authentication {
            success: false,
            user_id: None,
            method: "password".to_string(),
        },
        "10.0.0.1".to_string(),
        EventSeverity::Warning,
        "auth".to_string(),
        CorrelationId::new(),
    );
    system.record_event(event).await;
    tokio::time::sleep(Duration::from_millis(250)).await;
    let alerts = system.get_active_alerts().await;
    assert_eq!(alerts.len(), 1);
    assert_eq!(alerts[0].alert_type, AlertType::BruteForceAttempt);
}

#[tokio::test]
async fn test_critical_input_validation_event_generates_alert() {
    let config = SecurityMonitoringConfig {
        enable_behavioral_analysis: false,
        ..Default::default()
    };
    let system = SecurityMonitoringSystem::new(config);
    system.start().await.expect("start");

    let event = SecurityEvent::new(
        SecurityEventType::InputValidationViolation {
            client_ip: "192.168.0.1".to_string(),
            violation_type: "injection".to_string(),
            risk_level: "Critical".to_string(),
        },
        "192.168.0.1".to_string(),
        EventSeverity::High,
        "validator".to_string(),
        CorrelationId::new(),
    );
    system.record_event(event).await;
    tokio::time::sleep(Duration::from_millis(250)).await;
    let alerts = system.get_active_alerts().await;
    assert_eq!(alerts.len(), 1);
    assert_eq!(alerts[0].alert_type, AlertType::InputValidationAbuse);
}

#[tokio::test]
async fn test_is_shutdown_complete_tracks_flag() {
    let config = SecurityMonitoringConfig::default();
    let system = SecurityMonitoringSystem::new(config);
    assert!(!system.is_shutdown_complete().await);
    system
        .shutdown(ShutdownPhase::CleanupResources)
        .await
        .expect("should succeed");
    assert!(system.is_shutdown_complete().await);
}
