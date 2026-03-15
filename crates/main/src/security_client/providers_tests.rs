// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Tests for security provider management

use super::providers::*;
use crate::universal_primal_ecosystem::DiscoveredPrimal;
use chrono::Utc;
use universal_patterns::{PrimalContext, PrimalHealth, PrimalType};

#[test]
fn test_security_provider_from_discovered_primal() {
    let primal = DiscoveredPrimal {
        id: "test-security".to_string(),
        instance_id: "inst-security-123".to_string(),
        primal_type: PrimalType::Security,
        endpoint: "http://localhost:9090".to_string(),
        capabilities: vec![],
        health: PrimalHealth::Healthy,
        context: PrimalContext::default(),
        port_info: None,
    };

    let provider = SecurityProvider::from_discovered_primal(&primal);

    assert_eq!(provider.provider_id, "inst-security-123");
    assert_eq!(provider.metadata.name, "test-security");
    assert_eq!(provider.metadata.version, "unknown");
    assert!(provider.metadata.standards.contains(&"oauth2".to_string()));
    assert!(provider.metadata.standards.contains(&"openid".to_string()));
    assert!(
        provider
            .metadata
            .certifications
            .contains(&"soc2".to_string())
    );
    assert_eq!(provider.metadata.regions, vec!["local"]);
    assert_eq!(provider.health.health_score, 1.0);
    assert_eq!(provider.health.response_time_ms, 50.0);
    assert_eq!(provider.health.availability_percent, 99.9);
    assert_eq!(provider.health.incident_count, 0);
    assert_eq!(provider.routing_score, 0.9);
    assert!(matches!(
        provider.trust_level,
        super::types::TrustLevel::High
    ));
}

#[test]
fn test_security_provider_metadata_creation() {
    let metadata = SecurityProviderMetadata {
        name: "Auth Provider".to_string(),
        version: "2.1.0".to_string(),
        standards: vec!["oauth2".to_string(), "saml".to_string()],
        certifications: vec!["soc2".to_string(), "iso27001".to_string()],
        regions: vec!["us-east-1".to_string(), "eu-west-1".to_string()],
        compliance: vec![
            super::types::ComplianceFramework::Soc2,
            super::types::ComplianceFramework::Gdpr,
        ],
    };

    assert_eq!(metadata.name, "Auth Provider");
    assert_eq!(metadata.version, "2.1.0");
    assert_eq!(metadata.standards.len(), 2);
    assert_eq!(metadata.certifications.len(), 2);
    assert_eq!(metadata.regions.len(), 2);
    assert_eq!(metadata.compliance.len(), 2);
}

#[test]
fn test_security_provider_health_creation() {
    let health = SecurityProviderHealth {
        health_score: 0.95,
        response_time_ms: 25.0,
        availability_percent: 99.99,
        incident_count: 0,
        last_assessment: Utc::now(),
        last_check: Utc::now(),
    };

    assert_eq!(health.health_score, 0.95);
    assert_eq!(health.response_time_ms, 25.0);
    assert_eq!(health.availability_percent, 99.99);
    assert_eq!(health.incident_count, 0);
}

#[test]
fn test_security_provider_serialization() {
    let primal = DiscoveredPrimal {
        id: "serialize-sec".to_string(),
        instance_id: "inst-serialize-sec".to_string(),
        primal_type: PrimalType::Security,
        endpoint: "http://test:9090".to_string(),
        capabilities: vec![],
        health: PrimalHealth::Healthy,
        context: PrimalContext::default(),
        port_info: None,
    };

    let provider = SecurityProvider::from_discovered_primal(&primal);
    let json = serde_json::to_string(&provider).unwrap();
    let deserialized: SecurityProvider = serde_json::from_str(&json).unwrap();

    assert_eq!(provider.provider_id, deserialized.provider_id);
    assert_eq!(provider.metadata.name, deserialized.metadata.name);
    assert_eq!(provider.routing_score, deserialized.routing_score);
}

#[test]
fn test_security_provider_health_degraded() {
    let health = SecurityProviderHealth {
        health_score: 0.45,
        response_time_ms: 2000.0,
        availability_percent: 85.0,
        incident_count: 3,
        last_assessment: Utc::now(),
        last_check: Utc::now(),
    };

    assert!(health.health_score < 0.5);
    assert!(health.response_time_ms > 1000.0);
    assert!(health.incident_count > 0);
}
