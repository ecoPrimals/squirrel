// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

use super::*;
use chrono::Utc;
// Note: sleep reserved for async test delays

#[tokio::test]
async fn test_rate_limiting() {
    let config = SecurityHardeningConfig {
        max_auth_attempts_per_minute: 3,
        ..Default::default()
    };

    let hardening = SecurityHardening::new(config).await;

    // Should allow first few attempts
    for i in 0..3 {
        assert!(
            hardening
                .check_auth_rate_limit("192.168.1.1", &format!("user{}", i), None)
                .await
                .is_ok()
        );
        hardening
            .record_auth_attempt("192.168.1.1", &format!("user{}", i), false, None)
            .await;
    }

    // Should block after rate limit exceeded
    let result = hardening
        .check_auth_rate_limit("192.168.1.1", "user4", None)
        .await;
    assert!(result.is_err());

    if let Err(AuthRateLimitError::RateLimitExceeded { attempts, .. }) = result {
        assert_eq!(attempts, 3);
    } else {
        unreachable!("Expected RateLimitExceeded error");
    }
}

#[tokio::test]
async fn test_account_lockout() {
    let config = SecurityHardeningConfig {
        max_auth_attempts_per_minute: 2,
        account_lockout_duration_minutes: 1,
        ..Default::default()
    };

    let hardening = SecurityHardening::new(config).await;

    // Generate enough failures to trigger lockout
    for _ in 0..6 {
        let _ = hardening
            .check_auth_rate_limit("192.168.1.2", "testuser", None)
            .await;
        hardening
            .record_auth_attempt("192.168.1.2", "testuser", false, None)
            .await;
    }

    // Account should be locked
    let result = hardening
        .check_auth_rate_limit("192.168.1.3", "testuser", None)
        .await;
    assert!(matches!(
        result,
        Err(AuthRateLimitError::AccountLocked { .. })
    ));
}

#[tokio::test]
async fn test_security_metrics() {
    let hardening = SecurityHardening::new(SecurityHardeningConfig::default()).await;

    // Record some attempts
    hardening
        .record_auth_attempt("192.168.1.1", "user1", true, None)
        .await;
    hardening
        .record_auth_attempt("192.168.1.1", "user1", false, None)
        .await;
    hardening
        .record_auth_attempt("192.168.1.2", "user2", false, None)
        .await;

    let metrics = hardening.get_security_metrics().await;
    assert_eq!(metrics.total_ips_tracked, 2);
    assert_eq!(metrics.total_attempts_last_hour, 3);
    assert_eq!(metrics.failed_attempts_last_hour, 2);
    assert!(metrics.rate_limiting_enabled);
}

#[tokio::test]
async fn test_rate_limiting_disabled_short_circuits() {
    let config = SecurityHardeningConfig {
        enable_rate_limiting: false,
        ..Default::default()
    };
    let hardening = SecurityHardening::new(config).await;
    assert!(
        hardening
            .check_auth_rate_limit("10.0.0.1", "user", None)
            .await
            .is_ok()
    );
    hardening
        .record_auth_attempt("10.0.0.1", "user", false, None)
        .await;
    let metrics = hardening.get_security_metrics().await;
    assert!(!metrics.rate_limiting_enabled);
    assert_eq!(metrics.total_ips_tracked, 0);
}

#[tokio::test]
async fn test_report_incident_security_config_change_and_webhook_path() {
    let config = SecurityHardeningConfig {
        security_webhook_url: Some("https://example.com/security-hook".to_string()),
        ..Default::default()
    };
    let hardening = SecurityHardening::new(config).await;

    let incident = SecurityIncident::SecurityConfigChange {
        changed_setting: "max_auth".to_string(),
        old_value: "5".to_string(),
        new_value: "10".to_string(),
        changed_by: "admin".to_string(),
        timestamp: Utc::now(),
    };
    hardening
        .report_incident(incident)
        .await
        .expect("report should succeed");
}

#[tokio::test]
async fn test_report_incident_suspicious_activity_risk_levels() {
    let hardening = SecurityHardening::new(SecurityHardeningConfig::default()).await;

    let low = SecurityIncident::SuspiciousActivity {
        activity_type: "scan".to_string(),
        details: HashMap::new(),
        risk_level: RiskLevel::Low,
        timestamp: Utc::now(),
    };
    hardening.report_incident(low).await.expect("ok");

    let critical = SecurityIncident::SuspiciousActivity {
        activity_type: "exploit".to_string(),
        details: HashMap::new(),
        risk_level: RiskLevel::Critical,
        timestamp: Utc::now(),
    };
    hardening.report_incident(critical).await.expect("ok");
}

#[tokio::test]
async fn test_initialize_production_security_returns_arc() {
    let arc = initialize_production_security().await.expect("init");
    let m = arc.get_security_metrics().await;
    assert!(m.panic_handler_enabled);
}
