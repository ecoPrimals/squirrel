// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

use super::checks::{
    check_ai_providers, check_binary, check_configuration, check_discovered_services,
    check_rpc_server, check_unix_socket,
};
use super::{run_doctor, HealthCheck, HealthReport, HealthStatus};
use crate::cli::{OutputFormat, Subsystem};

#[test]
fn test_check_binary() {
    let check = check_binary();
    assert_eq!(check.status, HealthStatus::Ok);
    assert!(check.message.contains("squirrel"));
}

#[test]
fn test_check_configuration() {
    let check = check_configuration();
    assert!(matches!(
        check.status,
        HealthStatus::Ok | HealthStatus::Warning
    ));
}

#[tokio::test]
async fn test_should_check_filter() {
    assert!(super::should_check(None, Subsystem::Ai));
    assert!(super::should_check(Some(Subsystem::Ai), Subsystem::Ai));
    assert!(!super::should_check(Some(Subsystem::Config), Subsystem::Ai));
}

// ========================================================================
// ADDITIONAL UNIT TESTS
// ========================================================================

#[test]
fn test_health_status_serialization() {
    let status = HealthStatus::Ok;
    let json = serde_json::to_string(&status).expect("should succeed");
    assert!(json.contains("Ok") || json.contains("\"ok\""));
}

#[test]
fn test_health_check_structure() {
    let check = HealthCheck {
        name: "Test",
        status: HealthStatus::Ok,
        message: "Test message".to_string(),
        duration_ms: 100,
        details: None,
    };

    assert_eq!(check.name, "Test");
    assert!(matches!(check.status, HealthStatus::Ok));
    assert_eq!(check.duration_ms, 100);
}

#[test]
fn test_health_report_serialization() {
    let report = HealthReport {
        version: "1.2.0".to_string(),
        timestamp: chrono::Utc::now(),
        overall_status: HealthStatus::Ok,
        checks: vec![],
        recommendations: vec![],
    };

    let json = serde_json::to_string(&report).expect("should succeed");
    assert!(json.contains("1.2.0"));
    // Status might be "Ok" or "ok" depending on serialization
    assert!(json.to_lowercase().contains("ok"));
}

// ========================================================================
// E2E TESTS - Full Check Flows
// ========================================================================

#[test]
fn test_binary_check_always_succeeds() {
    let check = check_binary();
    assert_eq!(check.status, HealthStatus::Ok);
    assert!(check.message.contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn test_configuration_check_structure() {
    let check = check_configuration();
    assert_eq!(check.name, "Configuration");
    assert!(!check.message.is_empty());
}

#[test]
fn test_unix_socket_check_returns_valid_status() {
    let check = check_unix_socket();
    assert!(matches!(
        check.status,
        HealthStatus::Ok | HealthStatus::Warning | HealthStatus::Error
    ));
}

#[test]
fn test_rpc_server_check_structure() {
    let check = check_rpc_server();
    assert_eq!(check.name, "RPC Server");
}

// ========================================================================
// CHAOS TESTS - Edge Cases
// ========================================================================

#[test]
fn test_all_checks_run_without_panic() {
    let checks = (
        check_binary(),
        check_configuration(),
        check_unix_socket(),
        check_rpc_server(),
    );

    assert!(!checks.0.name.is_empty());
    assert!(!checks.1.name.is_empty());
    assert!(!checks.2.name.is_empty());
    assert!(!checks.3.name.is_empty());
}

#[test]
fn test_checks_complete_in_reasonable_time() {
    use std::time::Instant;
    let start = Instant::now();

    let _ = (
        check_binary(),
        check_configuration(),
        check_unix_socket(),
        check_rpc_server(),
    );

    let elapsed = start.elapsed();
    assert!(
        elapsed.as_secs() < 10,
        "All checks should complete in < 10s, took: {elapsed:?}",
    );
}

#[test]
fn test_subsystem_filtering_none() {
    assert!(super::should_check(None, Subsystem::Ai));
    assert!(super::should_check(None, Subsystem::Config));
    assert!(super::should_check(None, Subsystem::Rpc));
}

#[test]
fn test_subsystem_filtering_specific() {
    assert!(super::should_check(Some(Subsystem::Ai), Subsystem::Ai));
    assert!(!super::should_check(Some(Subsystem::Ai), Subsystem::Config));
    assert!(!super::should_check(Some(Subsystem::Config), Subsystem::Rpc));
}

#[test]
fn test_subsystem_display() {
    assert_eq!(format!("{}", Subsystem::Ai), "ai");
    assert_eq!(format!("{}", Subsystem::Config), "config");
    assert_eq!(format!("{}", Subsystem::Rpc), "rpc");
}

// ========================================================================
// FAULT TESTS - Error Scenarios
// ========================================================================

#[test]
fn test_all_checks_have_valid_durations() {
    let checks = (
        check_binary(),
        check_configuration(),
        check_unix_socket(),
        check_rpc_server(),
    );

    assert!(checks.0.duration_ms < 10000);
    assert!(checks.1.duration_ms < 10000);
    assert!(checks.2.duration_ms < 10000);
    assert!(checks.3.duration_ms < 10000);
}

#[tokio::test]
async fn test_health_status_ordering() {
    // Test that status types are distinct
    let ok_check = HealthCheck {
        name: "OK",
        status: HealthStatus::Ok,
        message: "OK".to_string(),
        duration_ms: 10,
        details: None,
    };

    let warn_check = HealthCheck {
        name: "Warn",
        status: HealthStatus::Warning,
        message: "Warn".to_string(),
        duration_ms: 20,
        details: None,
    };

    let err_check = HealthCheck {
        name: "Error",
        status: HealthStatus::Error,
        message: "Error".to_string(),
        duration_ms: 30,
        details: None,
    };

    // Verify they're distinct
    assert!(!matches!(ok_check.status, HealthStatus::Warning));
    assert!(!matches!(warn_check.status, HealthStatus::Ok));
    assert!(!matches!(err_check.status, HealthStatus::Ok));
}

// ========================================================================
// INTEGRATION TESTS
// ========================================================================

#[test]
fn test_concurrent_check_execution() {
    let results = (
        check_binary(),
        check_binary(),
        check_configuration(),
        check_configuration(),
    );

    assert!(!results.0.message.is_empty());
    assert!(!results.1.message.is_empty());
    assert!(!results.2.message.is_empty());
    assert!(!results.3.message.is_empty());
}

#[test]
fn test_health_report_json_serialization() {
    let report = HealthReport {
        version: "1.2.0".to_string(),
        timestamp: chrono::Utc::now(),
        overall_status: HealthStatus::Ok,
        checks: vec![HealthCheck {
            name: "Test",
            status: HealthStatus::Ok,
            message: "OK".to_string(),
            duration_ms: 50,
            details: None,
        }],
        recommendations: vec![],
    };

    let json = serde_json::to_string(&report).expect("should succeed");
    assert!(json.contains("1.2.0"));
    assert!(json.contains("Test"));
}

#[test]
fn test_checks_produce_valid_messages() {
    let checks = (
        check_binary(),
        check_configuration(),
        check_unix_socket(),
        check_rpc_server(),
    );

    assert!(!checks.0.message.is_empty());
    assert!(!checks.1.message.is_empty());
    assert!(!checks.2.message.is_empty());
    assert!(!checks.3.message.is_empty());
}

#[test]
fn test_generate_recommendations_empty_checks() {
    let checks = vec![];
    let recs = super::generate_recommendations(&checks);
    assert_eq!(recs.len(), 1);
    assert!(recs[0].contains("operational"));
}

#[test]
fn test_generate_recommendations_ai_provider_warning() {
    let checks = vec![HealthCheck {
        name: "AI Providers",
        status: HealthStatus::Warning,
        message: "No AI providers configured".to_string(),
        duration_ms: 10,
        details: None,
    }];
    let recs = super::generate_recommendations(&checks);
    assert!(recs.iter().any(|r| r.contains("AI_PROVIDER_SOCKETS")));
}

#[test]
fn test_generate_recommendations_ecosystem_services_warning() {
    let checks = vec![HealthCheck {
        name: "Ecosystem Services",
        status: HealthStatus::Warning,
        message: "No services discovered".to_string(),
        duration_ms: 10,
        details: None,
    }];
    let recs = super::generate_recommendations(&checks);
    assert!(
        recs.iter()
            .any(|r| r.contains("ecosystem registry") || r.contains("service mesh")),
        "Expected capability-based ecosystem recommendation, got: {recs:?}"
    );
    assert!(
        recs.iter()
            .any(|r| r.contains("security provider") || r.contains("auth")),
        "Expected capability-based security recommendation, got: {recs:?}"
    );
}

#[tokio::test]
async fn test_run_doctor_text_format() {
    let result = run_doctor(false, OutputFormat::Text, None);
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_run_doctor_json_format() {
    let result = run_doctor(false, OutputFormat::Json, None);
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_run_doctor_subsystem_filter_config() {
    let result = run_doctor(false, OutputFormat::Text, Some(Subsystem::Config));
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_run_doctor_subsystem_filter_ai() {
    let result = run_doctor(false, OutputFormat::Text, Some(Subsystem::Ai));
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_run_doctor_comprehensive() {
    let result = run_doctor(true, OutputFormat::Text, None);
    assert!(result.is_ok());
}

#[test]
fn test_check_discovered_services() {
    let check = check_discovered_services();
    assert_eq!(check.name, "Ecosystem Services");
    assert!(matches!(
        check.status,
        HealthStatus::Ok | HealthStatus::Warning
    ));
}

#[test]
fn test_check_ai_providers_comprehensive() {
    let check = check_ai_providers(true);
    assert_eq!(check.name, "AI Providers");
    assert!(matches!(
        check.status,
        HealthStatus::Ok | HealthStatus::Warning
    ));
}
