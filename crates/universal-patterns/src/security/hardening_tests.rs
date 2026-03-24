// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Comprehensive tests for Security Hardening Module
//!
//! Coverage goal: 90%+
//! Strategy: Test all configuration, rate limiting, incident handling, and monitoring

use super::hardening::*;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

// ============================================================================
// Configuration Tests
// ============================================================================

#[test]
fn test_security_hardening_config_default() {
    let config = SecurityHardeningConfig::default();

    assert!(config.enable_panic_handler);
    assert!(config.enable_rate_limiting);
    assert!(config.enable_security_monitoring);
    assert!(config.enable_audit_logging);
    assert_eq!(config.max_auth_attempts_per_minute, 5);
    assert_eq!(config.account_lockout_duration_minutes, 15);
    assert_eq!(config.security_webhook_url, None);
    assert_eq!(config.environment, Environment::Production);
}

#[test]
fn test_security_hardening_config_custom() {
    let config = SecurityHardeningConfig {
        enable_panic_handler: false,
        enable_rate_limiting: true,
        enable_security_monitoring: false,
        enable_audit_logging: true,
        max_auth_attempts_per_minute: 10,
        account_lockout_duration_minutes: 30,
        security_webhook_url: Some("https://example.com/webhook".to_string()),
        environment: Environment::Development,
    };

    assert!(!config.enable_panic_handler);
    assert!(config.enable_rate_limiting);
    assert!(!config.enable_security_monitoring);
    assert!(config.enable_audit_logging);
    assert_eq!(config.max_auth_attempts_per_minute, 10);
    assert_eq!(config.account_lockout_duration_minutes, 30);
    assert!(config.security_webhook_url.is_some());
    assert_eq!(config.environment, Environment::Development);
}

#[test]
fn test_environment_variants() {
    let dev = Environment::Development;
    let test = Environment::Testing;
    let staging = Environment::Staging;
    let prod = Environment::Production;

    assert_eq!(dev, Environment::Development);
    assert_eq!(test, Environment::Testing);
    assert_eq!(staging, Environment::Staging);
    assert_eq!(prod, Environment::Production);

    // Test inequality
    assert_ne!(dev, prod);
    assert_ne!(test, staging);
}

#[test]
fn test_environment_clone() {
    let env = Environment::Production;
    let cloned = env.clone();
    assert_eq!(env, cloned);
}

// ============================================================================
// Security Incident Tests
// ============================================================================

#[test]
fn test_security_incident_application_panic() {
    let incident = SecurityIncident::ApplicationPanic {
        message: "Test panic".to_string(),
        location: Some("test.rs:123".to_string()),
        thread: "main".to_string(),
        timestamp: Utc::now(),
    };

    match incident {
        SecurityIncident::ApplicationPanic { message, .. } => {
            assert_eq!(message, "Test panic");
        }
        _ => unreachable!("Expected ApplicationPanic variant"),
    }
}

#[test]
fn test_security_incident_rate_limit_exceeded() {
    let incident = SecurityIncident::RateLimitExceeded {
        ip_address: "192.168.1.100".to_string(),
        user_agent: Some("Mozilla/5.0".to_string()),
        attempt_count: 10,
        timestamp: Utc::now(),
    };

    match incident {
        SecurityIncident::RateLimitExceeded {
            ip_address,
            attempt_count,
            ..
        } => {
            assert_eq!(ip_address, "192.168.1.100");
            assert_eq!(attempt_count, 10);
        }
        _ => unreachable!("Expected RateLimitExceeded variant"),
    }
}

#[test]
fn test_security_incident_account_locked() {
    let incident = SecurityIncident::AccountLocked {
        username: "testuser".to_string(),
        ip_address: "10.0.0.1".to_string(),
        failed_attempts: 5,
        lockout_duration: Duration::from_secs(900),
        timestamp: Utc::now(),
    };

    match incident {
        SecurityIncident::AccountLocked {
            username,
            failed_attempts,
            ..
        } => {
            assert_eq!(username, "testuser");
            assert_eq!(failed_attempts, 5);
        }
        _ => unreachable!("Expected AccountLocked variant"),
    }
}

#[test]
fn test_security_incident_suspicious_activity() {
    let mut details = HashMap::new();
    details.insert("pattern".to_string(), "sql_injection".to_string());

    let incident = SecurityIncident::SuspiciousActivity {
        activity_type: "injection_attempt".to_string(),
        details,
        risk_level: RiskLevel::Critical,
        timestamp: Utc::now(),
    };

    match incident {
        SecurityIncident::SuspiciousActivity { activity_type, .. } => {
            assert_eq!(activity_type, "injection_attempt");
            // Can't compare RiskLevel without PartialEq
        }
        _ => unreachable!("Expected SuspiciousActivity variant"),
    }
}

#[test]
fn test_security_incident_config_change() {
    let incident = SecurityIncident::SecurityConfigChange {
        changed_setting: "max_attempts".to_string(),
        old_value: "5".to_string(),
        new_value: "10".to_string(),
        changed_by: "admin".to_string(),
        timestamp: Utc::now(),
    };

    match incident {
        SecurityIncident::SecurityConfigChange {
            changed_setting,
            old_value,
            new_value,
            changed_by,
            ..
        } => {
            assert_eq!(changed_setting, "max_attempts");
            assert_eq!(old_value, "5");
            assert_eq!(new_value, "10");
            assert_eq!(changed_by, "admin");
        }
        _ => unreachable!("Expected SecurityConfigChange variant"),
    }
}

#[test]
fn test_security_incident_serialization() {
    let incident = SecurityIncident::RateLimitExceeded {
        ip_address: "1.2.3.4".to_string(),
        user_agent: None,
        attempt_count: 5,
        timestamp: Utc::now(),
    };

    let serialized = serde_json::to_string(&incident).expect("should succeed");
    let deserialized: SecurityIncident = serde_json::from_str(&serialized).expect("should succeed");

    match deserialized {
        SecurityIncident::RateLimitExceeded { ip_address, .. } => {
            assert_eq!(ip_address, "1.2.3.4");
        }
        _ => unreachable!("Deserialization failed"),
    }
}

// ============================================================================
// RiskLevel Tests
// ============================================================================

#[test]
fn test_risk_level_variants() {
    let low = RiskLevel::Low;
    let medium = RiskLevel::Medium;
    let high = RiskLevel::High;
    let critical = RiskLevel::Critical;

    // Test all variants exist (just verify they can be created)
    let _ = (low, medium, high, critical);
    assert!(true);
}

#[test]
fn test_risk_level_serialization() {
    let risk = RiskLevel::High;
    let serialized = serde_json::to_string(&risk).expect("should succeed");
    let deserialized: RiskLevel = serde_json::from_str(&serialized).expect("should succeed");

    // Can't directly compare without PartialEq, so serialize both and compare
    let re_serialized = serde_json::to_string(&deserialized).expect("should succeed");
    assert_eq!(serialized, re_serialized);
}

// ============================================================================
// SecurityHardening Tests
// ============================================================================

#[tokio::test]
async fn test_security_hardening_creation() {
    let config = SecurityHardeningConfig::default();
    let _hardening = SecurityHardening::new(config).await;

    // If we get here, creation succeeded
    assert!(true);
}

#[tokio::test]
async fn test_security_hardening_with_disabled_features() {
    let config = SecurityHardeningConfig {
        enable_panic_handler: false,
        enable_rate_limiting: false,
        enable_security_monitoring: false,
        enable_audit_logging: false,
        ..Default::default()
    };

    let _hardening = SecurityHardening::new(config).await;
    assert!(true);
}

#[tokio::test]
async fn test_check_auth_rate_limit_allows_first_attempts() {
    let config = SecurityHardeningConfig::default();
    let hardening = SecurityHardening::new(config).await;

    // First attempt should be allowed
    let result = hardening
        .check_auth_rate_limit("192.168.1.1", "user1", Some("Mozilla/5.0"))
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_check_auth_rate_limit_with_disabled_limiting() {
    let mut config = SecurityHardeningConfig::default();
    config.enable_rate_limiting = false;

    let hardening = SecurityHardening::new(config).await;

    // Should always allow when rate limiting is disabled
    let result = hardening
        .check_auth_rate_limit("192.168.1.1", "user1", None)
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_record_auth_attempt_success() {
    let config = SecurityHardeningConfig::default();
    let hardening = SecurityHardening::new(config).await;

    // Record successful attempt
    hardening
        .record_auth_attempt("192.168.1.1", "user1", true, Some("Chrome"))
        .await;

    // Should still be allowed
    let result = hardening
        .check_auth_rate_limit("192.168.1.1", "user1", Some("Chrome"))
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_record_auth_attempt_failure() {
    let config = SecurityHardeningConfig::default();
    let hardening = SecurityHardening::new(config).await;

    // Record failed attempt
    hardening
        .record_auth_attempt("192.168.1.1", "user1", false, Some("Chrome"))
        .await;

    // Should still be allowed (only 1 failure)
    let result = hardening
        .check_auth_rate_limit("192.168.1.1", "user1", Some("Chrome"))
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_rate_limit_exceeded_after_multiple_failures() {
    let mut config = SecurityHardeningConfig::default();
    config.max_auth_attempts_per_minute = 3;

    let hardening = SecurityHardening::new(config).await;

    // Record 3 failed attempts
    for _ in 0..3 {
        hardening
            .record_auth_attempt("192.168.1.1", "user1", false, Some("Chrome"))
            .await;
    }

    // Should now be rate limited
    let result = hardening
        .check_auth_rate_limit("192.168.1.1", "user1", Some("Chrome"))
        .await;

    assert!(result.is_err());
    match result {
        Err(AuthRateLimitError::RateLimitExceeded { attempts, .. }) => {
            assert!(attempts >= 3);
        }
        _ => unreachable!("Expected RateLimitExceeded error"),
    }
}

#[tokio::test]
async fn test_record_auth_with_disabled_limiting() {
    let mut config = SecurityHardeningConfig::default();
    config.enable_rate_limiting = false;

    let hardening = SecurityHardening::new(config).await;

    // Recording should be no-op when disabled
    hardening
        .record_auth_attempt("192.168.1.1", "user1", false, None)
        .await;

    // Should still be allowed
    let result = hardening
        .check_auth_rate_limit("192.168.1.1", "user1", None)
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_different_ips_independent_rate_limiting() {
    let mut config = SecurityHardeningConfig::default();
    config.max_auth_attempts_per_minute = 2;

    let hardening = SecurityHardening::new(config).await;

    // Fail from IP 1
    hardening
        .record_auth_attempt("192.168.1.1", "user1", false, None)
        .await;
    hardening
        .record_auth_attempt("192.168.1.1", "user1", false, None)
        .await;

    // IP 1 should be limited
    let result1 = hardening
        .check_auth_rate_limit("192.168.1.1", "user1", None)
        .await;
    assert!(result1.is_err());

    // IP 2 should still be allowed
    let result2 = hardening
        .check_auth_rate_limit("192.168.1.2", "user1", None)
        .await;
    assert!(result2.is_ok());
}

// ============================================================================
// Auth Rate Limit Error Tests
// ============================================================================

#[test]
fn test_auth_rate_limit_error_variants() {
    let rate_limit_err = AuthRateLimitError::RateLimitExceeded {
        attempts: 5,
        reset_time: SystemTime::now(),
    };

    match rate_limit_err {
        AuthRateLimitError::RateLimitExceeded { attempts, .. } => {
            assert_eq!(attempts, 5);
        }
        _ => unreachable!("Expected RateLimitExceeded"),
    }

    let account_locked_err = AuthRateLimitError::AccountLocked {
        remaining_time: Duration::from_secs(300),
        reason: "Too many failed attempts".to_string(),
    };

    match account_locked_err {
        AuthRateLimitError::AccountLocked { remaining_time, .. } => {
            assert_eq!(remaining_time.as_secs(), 300);
        }
        _ => unreachable!("Expected AccountLocked"),
    }
}

// ============================================================================
// Security Error Tests
// ============================================================================

#[test]
fn test_security_error_variants() {
    let config_err = SecurityError::ConfigurationError("Invalid config".to_string());
    match config_err {
        SecurityError::ConfigurationError(msg) => {
            assert_eq!(msg, "Invalid config");
        }
        _ => unreachable!("Expected ConfigurationError"),
    }

    let incident_err = SecurityError::IncidentHandlingFailed("Failed to log incident".to_string());
    match incident_err {
        SecurityError::IncidentHandlingFailed(msg) => {
            assert_eq!(msg, "Failed to log incident");
        }
        _ => unreachable!("Expected IncidentHandlingFailed"),
    }
}

// ============================================================================
// SecurityMetrics Tests
// ============================================================================

#[test]
fn test_security_metrics_creation() {
    let metrics = SecurityMetrics {
        total_ips_tracked: 100,
        total_attempts_last_hour: 500,
        failed_attempts_last_hour: 50,
        locked_accounts_count: 5,
        rate_limiting_enabled: true,
        panic_handler_enabled: true,
        security_monitoring_enabled: true,
    };

    assert_eq!(metrics.total_ips_tracked, 100);
    assert_eq!(metrics.total_attempts_last_hour, 500);
    assert_eq!(metrics.failed_attempts_last_hour, 50);
    assert_eq!(metrics.locked_accounts_count, 5);
    assert!(metrics.rate_limiting_enabled);
    assert!(metrics.panic_handler_enabled);
    assert!(metrics.security_monitoring_enabled);
}

#[test]
fn test_security_metrics_default() {
    let metrics = SecurityMetrics {
        total_ips_tracked: 0,
        total_attempts_last_hour: 0,
        failed_attempts_last_hour: 0,
        locked_accounts_count: 0,
        rate_limiting_enabled: false,
        panic_handler_enabled: false,
        security_monitoring_enabled: false,
    };

    assert_eq!(metrics.total_ips_tracked, 0);
    assert_eq!(metrics.total_attempts_last_hour, 0);
    assert!(!metrics.rate_limiting_enabled);
}

// ============================================================================
// Initialize Production Security Tests
// ============================================================================

#[tokio::test]
async fn test_initialize_production_security() {
    let result = initialize_production_security().await;

    // Should successfully create Arc<SecurityHardening>
    assert!(result.is_ok());

    let hardening = result.expect("should succeed");
    // Verify it's usable
    let check_result = hardening
        .check_auth_rate_limit("127.0.0.1", "test", None)
        .await;
    assert!(check_result.is_ok());
}

// ============================================================================
// Integration Tests
// ============================================================================

#[tokio::test]
async fn test_full_rate_limiting_workflow() {
    let mut config = SecurityHardeningConfig::default();
    config.max_auth_attempts_per_minute = 3;

    let hardening = SecurityHardening::new(config).await;
    let ip = "10.0.0.1";
    let user = "testuser";

    // Step 1: Check rate limit (should pass)
    assert!(hardening
        .check_auth_rate_limit(ip, user, None)
        .await
        .is_ok());

    // Step 2: Record failed attempt
    hardening.record_auth_attempt(ip, user, false, None).await;

    // Step 3: Check again (should still pass - only 1 failure)
    assert!(hardening
        .check_auth_rate_limit(ip, user, None)
        .await
        .is_ok());

    // Step 4: Record 2 more failures
    hardening.record_auth_attempt(ip, user, false, None).await;
    hardening.record_auth_attempt(ip, user, false, None).await;

    // Step 5: Check again (should now fail - 3 failures)
    let result = hardening.check_auth_rate_limit(ip, user, None).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_successful_auth_doesnt_contribute_to_rate_limit() {
    let mut config = SecurityHardeningConfig::default();
    config.max_auth_attempts_per_minute = 3; // Allow 3 failed attempts

    let hardening = SecurityHardening::new(config).await;

    // Mix of successful and failed attempts
    hardening
        .record_auth_attempt("192.168.1.1", "user1", true, None)
        .await;
    hardening
        .record_auth_attempt("192.168.1.1", "user1", false, None)
        .await;
    hardening
        .record_auth_attempt("192.168.1.1", "user1", true, None)
        .await;
    hardening
        .record_auth_attempt("192.168.1.1", "user1", false, None)
        .await;

    // Only 2 failures, so should still be allowed (limit is 3)
    let result = hardening
        .check_auth_rate_limit("192.168.1.1", "user1", None)
        .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_concurrent_auth_attempts() {
    use tokio::task::JoinSet;

    let config = SecurityHardeningConfig::default();
    let hardening = Arc::new(SecurityHardening::new(config).await);
    let mut set = JoinSet::new();

    // Spawn 10 concurrent checks
    for i in 0..10 {
        let hardening_clone = hardening.clone();
        set.spawn(async move {
            hardening_clone
                .check_auth_rate_limit(&format!("192.168.1.{}", i), "user", None)
                .await
        });
    }

    // All should succeed (different IPs)
    let mut success_count = 0;
    while let Some(result) = set.join_next().await {
        if result.expect("should succeed").is_ok() {
            success_count += 1;
        }
    }

    assert_eq!(success_count, 10);
}
