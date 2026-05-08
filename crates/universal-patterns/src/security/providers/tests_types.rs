// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Tests for security types: SecurityHealth, SecurityServiceConfig, SecurityResponse,
//! and capability matching logic.

use super::*;
use crate::config::AuthMethod;
use std::collections::HashMap;

use chrono::Utc;

#[test]
fn test_security_health_is_healthy() {
    let health = SecurityHealth {
        status: HealthStatus::Healthy,
        message: "All systems operational".to_string(),
        last_check: Utc::now(),
        metrics: HashMap::new(),
    };

    assert!(health.is_healthy(), "Healthy status should return true");
}

#[test]
fn test_security_health_is_not_healthy_degraded() {
    let health = SecurityHealth {
        status: HealthStatus::Degraded,
        message: "Service degraded".to_string(),
        last_check: Utc::now(),
        metrics: HashMap::new(),
    };

    assert!(!health.is_healthy(), "Degraded status should return false");
}

#[test]
fn test_security_health_is_not_healthy_unhealthy() {
    let health = SecurityHealth {
        status: HealthStatus::Unhealthy,
        message: "Service down".to_string(),
        last_check: Utc::now(),
        metrics: HashMap::new(),
    };

    assert!(!health.is_healthy(), "Unhealthy status should return false");
}

#[test]
fn test_security_service_config_default() {
    let config = SecurityServiceConfig::default();

    assert_eq!(config.service_id, "default");
    assert_eq!(config.endpoint, None);
    assert_eq!(config.timeout_seconds, Some(30));
    assert_eq!(config.max_retries, Some(3));
    assert_eq!(config.auth_config, None);
}

#[test]
fn test_security_service_config_custom() {
    let mut auth_config = HashMap::new();
    auth_config.insert("api_key".to_string(), "test123".to_string());

    let config = SecurityServiceConfig {
        service_id: "custom-service".to_string(),
        endpoint: Some("https://api.example.com".to_string()),
        timeout_seconds: Some(60),
        max_retries: Some(5),
        auth_config: Some(auth_config.clone()),
    };

    assert_eq!(config.service_id, "custom-service");
    assert_eq!(config.endpoint, Some("https://api.example.com".to_string()));
    assert_eq!(config.timeout_seconds, Some(60));
    assert_eq!(config.max_retries, Some(5));
    assert_eq!(config.auth_config, Some(auth_config));
}

#[test]
fn test_security_response_success() {
    let request_id = "test-request-123".to_string();
    let message = "Operation completed successfully".to_string();

    let response = SecurityResponse::success(request_id.clone(), message.clone());

    assert_eq!(response.request_id, request_id);
    assert!(matches!(response.status, SecurityResponseStatus::Success));
    assert_eq!(response.data, serde_json::json!({"message": message}));
    assert!(response.metadata.is_empty());
}

#[test]
fn test_security_response_failed() {
    let request_id = "test-request-456".to_string();
    let reason = "Authentication failed".to_string();

    let response = SecurityResponse::failed(request_id.clone(), reason.clone());

    assert_eq!(response.request_id, request_id);
    assert!(matches!(
        response.status,
        SecurityResponseStatus::Failed { .. }
    ));

    if let SecurityResponseStatus::Failed { reason: r } = response.status {
        assert_eq!(r, reason);
    } else {
        unreachable!("Expected Failed status");
    }

    assert_eq!(response.data, serde_json::Value::Null);
}

#[test]
fn test_capabilities_match_authentication() {
    let required = SecurityCapability::Authentication {
        methods: vec![AuthMethod::None],
        multi_factor: false,
        session_management: false,
    };

    let provided = SecurityCapability::Authentication {
        methods: vec![
            AuthMethod::None,
            AuthMethod::Token {
                token_file: std::path::PathBuf::from("/tmp/token"),
            },
        ],
        multi_factor: true,
        session_management: true,
    };

    assert!(
        capabilities_match(&required, &provided),
        "Should match when provided methods include required methods"
    );
}

#[test]
fn test_capabilities_match_authentication_no_match() {
    let required = SecurityCapability::Authentication {
        methods: vec![AuthMethod::SecurityProvider {
            service_id: "beardog-1".to_string(),
        }],
        multi_factor: false,
        session_management: false,
    };

    let provided = SecurityCapability::Authentication {
        methods: vec![AuthMethod::None],
        multi_factor: false,
        session_management: false,
    };

    assert!(
        !capabilities_match(&required, &provided),
        "Should not match when required methods are not provided"
    );
}

#[test]
fn test_capabilities_match_authorization() {
    let required = SecurityCapability::Authorization {
        rbac: true,
        abac: false,
        policy_engine: false,
    };

    let provided = SecurityCapability::Authorization {
        rbac: true,
        abac: true,
        policy_engine: true,
    };

    assert!(
        capabilities_match(&required, &provided),
        "Should match when provided capabilities meet requirements"
    );
}

#[test]
fn test_capabilities_match_authorization_no_match() {
    let required = SecurityCapability::Authorization {
        rbac: true,
        abac: true,
        policy_engine: false,
    };

    let provided = SecurityCapability::Authorization {
        rbac: true,
        abac: false,
        policy_engine: true,
    };

    assert!(
        !capabilities_match(&required, &provided),
        "Should not match when required ABAC is not provided"
    );
}

#[test]
fn test_capabilities_match_cryptography() {
    let required = SecurityCapability::Cryptography {
        algorithms: vec!["AES-256".to_string()],
        key_management: false,
        hardware_security: false,
    };

    let provided = SecurityCapability::Cryptography {
        algorithms: vec!["AES-256".to_string(), "RSA-4096".to_string()],
        key_management: true,
        hardware_security: true,
    };

    assert!(
        capabilities_match(&required, &provided),
        "Should match when required algorithms are available"
    );
}

#[test]
fn test_capabilities_match_different_types() {
    let required = SecurityCapability::Authentication {
        methods: vec![AuthMethod::None],
        multi_factor: false,
        session_management: false,
    };

    let provided = SecurityCapability::Authorization {
        rbac: true,
        abac: false,
        policy_engine: false,
    };

    assert!(
        !capabilities_match(&required, &provided),
        "Should not match different capability types"
    );
}

#[test]
fn test_capabilities_match_threat_detection() {
    let required = SecurityCapability::ThreatDetection {
        anomaly_detection: true,
        real_time_analysis: false,
        threat_intelligence: false,
    };

    let provided = SecurityCapability::ThreatDetection {
        anomaly_detection: false,
        real_time_analysis: true,
        threat_intelligence: true,
    };

    assert!(
        capabilities_match(&required, &provided),
        "Threat detection capabilities should match by type"
    );
}
