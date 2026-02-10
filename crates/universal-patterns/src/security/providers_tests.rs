// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Tests for security providers module
//!
//! Comprehensive test suite for security services, registry, and capability matching

use super::*;
use crate::security::context::SecurityContext;
use crate::config::AuthMethod;
use crate::traits::{Principal, PrincipalType};
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;

// ============================================================================
// Test Helpers
// ============================================================================

fn create_test_security_context() -> SecurityContext {
    let principal = Principal {
        id: "test-user".to_string(),
        name: "Test User".to_string(),
        principal_type: PrincipalType::User,
        roles: vec!["user".to_string()],
        permissions: vec!["read".to_string()],
        metadata: HashMap::new(),
    };
    SecurityContext::from_principal(&principal)
}

// ============================================================================
// SecurityHealth Tests
// ============================================================================

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

// ============================================================================
// SecurityServiceConfig Tests
// ============================================================================

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

// ============================================================================
// SecurityResponse Tests
// ============================================================================

#[test]
fn test_security_response_success() {
    let request_id = "test-request-123".to_string();
    let message = "Operation completed successfully".to_string();
    
    let response = SecurityResponse::success(request_id.clone(), message.clone())
        .expect("Should create successful response");
    
    assert_eq!(response.request_id, request_id);
    assert!(matches!(response.status, SecurityResponseStatus::Success));
    assert_eq!(response.data, serde_json::json!({"message": message}));
    assert!(response.metadata.is_empty());
}

#[test]
fn test_security_response_failed() {
    let request_id = "test-request-456".to_string();
    let reason = "Authentication failed".to_string();
    
    let response = SecurityResponse::failed(request_id.clone(), reason.clone())
        .expect("Should create failed response");
    
    assert_eq!(response.request_id, request_id);
    assert!(matches!(response.status, SecurityResponseStatus::Failed { .. }));
    
    if let SecurityResponseStatus::Failed { reason: r } = response.status {
        assert_eq!(r, reason);
    } else {
        panic!("Expected Failed status");
    }
    
    assert_eq!(response.data, serde_json::Value::Null);
}

// ============================================================================
// Capability Matching Tests
// ============================================================================

#[test]
fn test_capabilities_match_authentication() {
    let required = SecurityCapability::Authentication {
        methods: vec![AuthMethod::None],
        multi_factor: false,
        session_management: false,
    };
    
    let provided = SecurityCapability::Authentication {
        methods: vec![AuthMethod::None, AuthMethod::Token {
            token_file: std::path::PathBuf::from("/tmp/token"),
        }],
        multi_factor: true,
        session_management: true,
    };
    
    assert!(capabilities_match(&required, &provided), 
        "Should match when provided methods include required methods");
}

#[test]
fn test_capabilities_match_authentication_no_match() {
    let required = SecurityCapability::Authentication {
        methods: vec![AuthMethod::Beardog {
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
    
    assert!(!capabilities_match(&required, &provided), 
        "Should not match when required methods are not provided");
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
    
    assert!(capabilities_match(&required, &provided), 
        "Should match when provided capabilities meet requirements");
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
    
    assert!(!capabilities_match(&required, &provided), 
        "Should not match when required ABAC is not provided");
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
    
    assert!(capabilities_match(&required, &provided), 
        "Should match when required algorithms are available");
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
    
    assert!(!capabilities_match(&required, &provided), 
        "Should not match different capability types");
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
    
    assert!(capabilities_match(&required, &provided), 
        "Threat detection capabilities should match by type");
}

// ============================================================================
// UniversalSecurityRegistry Tests
// ============================================================================

#[tokio::test]
async fn test_registry_new() {
    let registry = UniversalSecurityRegistry::new();
    
    assert_eq!(registry.list_services().len(), 0, "New registry should be empty");
}

#[tokio::test]
async fn test_registry_register_service() {
    let mut registry = UniversalSecurityRegistry::new();
    let config = SecurityServiceConfig::default();
    let provider = LocalSecurityProvider::new(config).await.expect("test: operation should succeed");
    let service: Arc<dyn UniversalSecurityService> = Arc::new(provider);
    
    let result = registry.register_service("test-service".to_string(), service).await;
    
    assert!(result.is_ok(), "Should register service successfully");
    assert_eq!(registry.list_services().len(), 1, "Registry should have 1 service");
    assert!(registry.list_services().contains(&"test-service".to_string()));
}

#[tokio::test]
async fn test_registry_find_by_capability() {
    let mut registry = UniversalSecurityRegistry::new();
    let config = SecurityServiceConfig::default();
    let provider = LocalSecurityProvider::new(config).await.expect("test: operation should succeed");
    let service: Arc<dyn UniversalSecurityService> = Arc::new(provider);
    
    registry.register_service("local-service".to_string(), service).await.expect("test: operation should succeed");
    
    let capability = SecurityCapability::Authentication {
        methods: vec![AuthMethod::None],
        multi_factor: false,
        session_management: false,
    };
    
    let services = registry.find_by_capability(&capability);
    
    assert_eq!(services.len(), 1, "Should find 1 service with authentication capability");
    assert_eq!(services[0], "local-service");
}

#[tokio::test]
async fn test_registry_find_by_capability_no_match() {
    let mut registry = UniversalSecurityRegistry::new();
    let config = SecurityServiceConfig::default();
    let provider = LocalSecurityProvider::new(config).await.expect("test: operation should succeed");
    let service: Arc<dyn UniversalSecurityService> = Arc::new(provider);
    
    registry.register_service("local-service".to_string(), service).await.expect("test: operation should succeed");
    
    // LocalSecurityProvider doesn't have ThreatDetection capability
    let capability = SecurityCapability::ThreatDetection {
        anomaly_detection: true,
        real_time_analysis: true,
        threat_intelligence: true,
    };
    
    let services = registry.find_by_capability(&capability);
    
    assert_eq!(services.len(), 0, "Should find no services with threat detection capability");
}

#[tokio::test]
async fn test_registry_get_service() {
    let mut registry = UniversalSecurityRegistry::new();
    let config = SecurityServiceConfig::default();
    let provider = LocalSecurityProvider::new(config).await.expect("test: operation should succeed");
    let service: Arc<dyn UniversalSecurityService> = Arc::new(provider);
    
    registry.register_service("test-service".to_string(), service).await.expect("test: operation should succeed");
    
    let retrieved = registry.get_service("test-service");
    
    assert!(retrieved.is_some(), "Should retrieve registered service");
    
    let non_existent = registry.get_service("non-existent");
    assert!(non_existent.is_none(), "Should not retrieve non-existent service");
}

#[tokio::test]
async fn test_registry_list_services() {
    let mut registry = UniversalSecurityRegistry::new();
    
    let config1 = SecurityServiceConfig {
        service_id: "service-1".to_string(),
        ..Default::default()
    };
    let provider1 = LocalSecurityProvider::new(config1).await.expect("test: operation should succeed");
    registry.register_service("service-1".to_string(), Arc::new(provider1)).await.expect("test: operation should succeed");
    
    let config2 = SecurityServiceConfig {
        service_id: "service-2".to_string(),
        ..Default::default()
    };
    let provider2 = LocalSecurityProvider::new(config2).await.expect("test: operation should succeed");
    registry.register_service("service-2".to_string(), Arc::new(provider2)).await.expect("test: operation should succeed");
    
    let services = registry.list_services();
    
    assert_eq!(services.len(), 2, "Should list 2 services");
    assert!(services.contains(&"service-1".to_string()));
    assert!(services.contains(&"service-2".to_string()));
}

#[tokio::test]
async fn test_registry_find_optimal_service() {
    let mut registry = UniversalSecurityRegistry::new();
    
    // Register local provider
    let config = SecurityServiceConfig::default();
    let provider = LocalSecurityProvider::new(config).await.expect("test: operation should succeed");
    registry.register_service("local-service".to_string(), Arc::new(provider)).await.expect("test: operation should succeed");
    
    // Find service with authentication capability
    let requirements = vec![SecurityCapability::Authentication {
        methods: vec![AuthMethod::None],
        multi_factor: false,
        session_management: false,
    }];
    
    let result = registry.find_optimal_service(requirements).await;
    
    assert!(result.is_ok(), "Should find optimal service");
    assert_eq!(result.expect("test: operation should succeed"), "local-service");
}

#[tokio::test]
async fn test_registry_find_optimal_service_no_match() {
    let mut registry = UniversalSecurityRegistry::new();
    
    // Register local provider (no threat detection)
    let config = SecurityServiceConfig::default();
    let provider = LocalSecurityProvider::new(config).await.expect("test: operation should succeed");
    registry.register_service("local-service".to_string(), Arc::new(provider)).await.expect("test: operation should succeed");
    
    // Request capability that local provider doesn't have
    let requirements = vec![SecurityCapability::ThreatDetection {
        anomaly_detection: true,
        real_time_analysis: true,
        threat_intelligence: true,
    }];
    
    let result = registry.find_optimal_service(requirements).await;
    
    assert!(result.is_err(), "Should not find service for unmatched requirements");
}

// ============================================================================
// LocalSecurityProvider Tests
// ============================================================================

#[tokio::test]
async fn test_local_provider_new() {
    let config = SecurityServiceConfig::default();
    let result = LocalSecurityProvider::new(config).await;
    
    assert!(result.is_ok(), "Should create local provider successfully");
}

#[tokio::test]
async fn test_local_provider_get_capabilities() {
    let config = SecurityServiceConfig::default();
    let provider = LocalSecurityProvider::new(config).await.expect("test: operation should succeed");
    
    let capabilities = provider.get_capabilities();
    
    assert_eq!(capabilities.len(), 2, "Local provider should have 2 capabilities");
    
    // Check for Authentication capability
    let has_auth = capabilities.iter().any(|cap| {
        matches!(cap, SecurityCapability::Authentication { .. })
    });
    assert!(has_auth, "Should have authentication capability");
    
    // Check for Cryptography capability
    let has_crypto = capabilities.iter().any(|cap| {
        matches!(cap, SecurityCapability::Cryptography { .. })
    });
    assert!(has_crypto, "Should have cryptography capability");
}

#[tokio::test]
async fn test_local_provider_get_service_info() {
    let config = SecurityServiceConfig {
        service_id: "local-security".to_string(),
        ..Default::default()
    };
    let provider = LocalSecurityProvider::new(config).await.expect("test: operation should succeed");
    
    let info = provider.get_service_info();
    
    assert_eq!(info.service_id, "local-security");
    assert_eq!(info.name, "Local Security Service");
    assert_eq!(info.version, "1.0.0");
    assert!(matches!(info.trust_level, TrustLevel::Medium));
    assert_eq!(info.supported_protocols, vec!["Local".to_string()]);
    assert!(info.compliance_certifications.is_empty());
}

#[tokio::test]
async fn test_local_provider_health_check() {
    let config = SecurityServiceConfig::default();
    let provider = LocalSecurityProvider::new(config).await.expect("test: operation should succeed");
    
    let health = provider.health_check().await;
    
    assert!(health.is_ok(), "Health check should succeed");
    let health = health.expect("test: operation should succeed");
    assert!(matches!(health.status, HealthStatus::Healthy));
    assert!(health.message.contains("Local security"));
}

#[tokio::test]
async fn test_local_provider_handle_request() {
    let config = SecurityServiceConfig::default();
    let provider = LocalSecurityProvider::new(config).await.expect("test: operation should succeed");
    
    let request = SecurityRequest {
        request_id: "test-123".to_string(),
        operation: SecurityOperation::Authenticate,
        parameters: HashMap::new(),
        context: create_test_security_context(),
        requester: "test-user".to_string(),
        timestamp: Utc::now(),
        priority: Priority::Normal,
    };
    
    let response = provider.handle_security_request(request).await;
    
    assert!(response.is_ok(), "Should handle request successfully");
    let response = response.expect("test: operation should succeed");
    assert_eq!(response.request_id, "test-123");
    assert!(matches!(response.status, SecurityResponseStatus::Success));
}

#[tokio::test]
async fn test_local_provider_initialize() {
    let config = SecurityServiceConfig::default();
    let mut provider = LocalSecurityProvider::new(config).await.expect("test: operation should succeed");
    
    let new_config = SecurityServiceConfig {
        service_id: "updated-service".to_string(),
        timeout_seconds: Some(60),
        ..Default::default()
    };
    
    let result = provider.initialize(new_config.clone()).await;
    
    assert!(result.is_ok(), "Should initialize successfully");
}

// ============================================================================
// BeardogSecurityProvider Tests
// ============================================================================

#[tokio::test]
async fn test_beardog_provider_new() {
    let config = SecurityServiceConfig::default();
    let result = BeardogSecurityProvider::new(config).await;
    
    assert!(result.is_ok(), "Should create beardog provider successfully");
}

#[tokio::test]
async fn test_beardog_provider_get_capabilities() {
    let config = SecurityServiceConfig::default();
    let provider = BeardogSecurityProvider::new(config).await.expect("test: operation should succeed");
    
    let capabilities = provider.get_capabilities();
    
    assert_eq!(capabilities.len(), 4, "Beardog provider should have 4 capabilities");
    
    // Check for Authentication capability
    let has_auth = capabilities.iter().any(|cap| {
        matches!(cap, SecurityCapability::Authentication { .. })
    });
    assert!(has_auth, "Should have authentication capability");
    
    // Check for Authorization capability
    let has_authz = capabilities.iter().any(|cap| {
        matches!(cap, SecurityCapability::Authorization { .. })
    });
    assert!(has_authz, "Should have authorization capability");
    
    // Check for Cryptography capability
    let has_crypto = capabilities.iter().any(|cap| {
        matches!(cap, SecurityCapability::Cryptography { .. })
    });
    assert!(has_crypto, "Should have cryptography capability");
    
    // Check for Compliance capability
    let has_compliance = capabilities.iter().any(|cap| {
        matches!(cap, SecurityCapability::Compliance { .. })
    });
    assert!(has_compliance, "Should have compliance capability");
}

#[tokio::test]
async fn test_beardog_provider_get_service_info() {
    let config = SecurityServiceConfig {
        service_id: "beardog-security".to_string(),
        ..Default::default()
    };
    let provider = BeardogSecurityProvider::new(config).await.expect("test: operation should succeed");
    
    let info = provider.get_service_info();
    
    assert_eq!(info.service_id, "beardog-security");
    assert_eq!(info.name, "Beardog Security Service");
    assert_eq!(info.version, "1.0.0");
    assert!(matches!(info.trust_level, TrustLevel::High));
    assert!(info.supported_protocols.contains(&"HTTPS".to_string()));
    assert!(info.supported_protocols.contains(&"gRPC".to_string()));
    assert!(!info.compliance_certifications.is_empty());
}

#[tokio::test]
async fn test_beardog_provider_health_check() {
    let config = SecurityServiceConfig::default();
    let provider = BeardogSecurityProvider::new(config).await.expect("test: operation should succeed");
    
    let health = provider.health_check().await;
    
    assert!(health.is_ok(), "Health check should succeed");
    let health = health.expect("test: operation should succeed");
    assert!(matches!(health.status, HealthStatus::Healthy));
    assert!(health.message.contains("Beardog"));
}

#[tokio::test]
async fn test_beardog_provider_handle_request() {
    let config = SecurityServiceConfig::default();
    let provider = BeardogSecurityProvider::new(config).await.expect("test: operation should succeed");
    
    let request = SecurityRequest {
        request_id: "beardog-test-123".to_string(),
        operation: SecurityOperation::Authenticate,
        parameters: HashMap::new(),
        context: create_test_security_context(),
        requester: "test-user".to_string(),
        timestamp: Utc::now(),
        priority: Priority::High,
    };
    
    let response = provider.handle_security_request(request).await;
    
    assert!(response.is_ok(), "Should handle request successfully");
    let response = response.expect("test: operation should succeed");
    assert_eq!(response.request_id, "beardog-test-123");
    assert!(matches!(response.status, SecurityResponseStatus::Success));
}

#[tokio::test]
async fn test_beardog_provider_initialize() {
    let config = SecurityServiceConfig::default();
    let mut provider = BeardogSecurityProvider::new(config).await.expect("test: operation should succeed");
    
    let new_config = SecurityServiceConfig {
        service_id: "beardog-updated".to_string(),
        endpoint: Some("https://beardog.example.com".to_string()),
        timeout_seconds: Some(120),
        ..Default::default()
    };
    
    let result = provider.initialize(new_config).await;
    
    assert!(result.is_ok(), "Should initialize successfully");
}

// ============================================================================
// Integration Tests
// ============================================================================

#[tokio::test]
async fn test_register_security_service_function() {
    let mut registry = UniversalSecurityRegistry::new();
    let config = SecurityServiceConfig::default();
    let provider = LocalSecurityProvider::new(config).await.expect("test: operation should succeed");
    let service: Arc<dyn UniversalSecurityService> = Arc::new(provider);
    
    let result = register_security_service(&mut registry, service).await;
    
    assert!(result.is_ok(), "Should register service successfully");
    assert!(registry.list_services().contains(&"local-security".to_string()));
}

#[tokio::test]
async fn test_multi_provider_registry() {
    let mut registry = UniversalSecurityRegistry::new();
    
    // Register local provider
    let local_config = SecurityServiceConfig {
        service_id: "local".to_string(),
        ..Default::default()
    };
    let local_provider = LocalSecurityProvider::new(local_config).await.expect("test: operation should succeed");
    registry.register_service("local-service".to_string(), Arc::new(local_provider)).await.expect("test: operation should succeed");
    
    // Register beardog provider
    let beardog_config = SecurityServiceConfig {
        service_id: "beardog".to_string(),
        ..Default::default()
    };
    let beardog_provider = BeardogSecurityProvider::new(beardog_config).await.expect("test: operation should succeed");
    registry.register_service("beardog-service".to_string(), Arc::new(beardog_provider)).await.expect("test: operation should succeed");
    
    // Verify both services are registered
    let services = registry.list_services();
    assert_eq!(services.len(), 2, "Should have 2 services registered");
    
    // Find authentication capability (both have it)
    let auth_cap = SecurityCapability::Authentication {
        methods: vec![AuthMethod::None],
        multi_factor: false,
        session_management: false,
    };
    let auth_services = registry.find_by_capability(&auth_cap);
    assert!(auth_services.len() >= 1, "Should find at least 1 service with authentication");
    
    // Find compliance capability (only beardog has it)
    let compliance_cap = SecurityCapability::Compliance {
        standards: vec!["SOX".to_string()],
        audit_logging: false,
        real_time_monitoring: false,
    };
    let compliance_services = registry.find_by_capability(&compliance_cap);
    assert_eq!(compliance_services.len(), 1, "Only beardog should have compliance capability");
    assert_eq!(compliance_services[0], "beardog-service");
}

#[tokio::test]
async fn test_end_to_end_security_workflow() {
    // Create registry
    let mut registry = UniversalSecurityRegistry::new();
    
    // Register beardog provider
    let config = SecurityServiceConfig {
        service_id: "beardog-primary".to_string(),
        endpoint: Some("https://beardog.local".to_string()),
        ..Default::default()
    };
    let provider = BeardogSecurityProvider::new(config).await.expect("test: operation should succeed");
    registry.register_service("beardog-primary".to_string(), Arc::new(provider)).await.expect("test: operation should succeed");
    
    // Find service by capability
    let requirements = vec![
        SecurityCapability::Authentication {
            methods: vec![AuthMethod::Beardog {
                service_id: "beardog-primary".to_string(),
            }],
            multi_factor: true,
            session_management: true,
        },
        SecurityCapability::Compliance {
            standards: vec!["SOX".to_string()],
            audit_logging: true,
            real_time_monitoring: true,
        },
    ];
    
    let service_id = registry.find_optimal_service(requirements).await;
    assert!(service_id.is_ok(), "Should find suitable service");
    
    // Get the service
    let service = registry.get_service(&service_id.expect("test: operation should succeed"));
    assert!(service.is_some(), "Should retrieve service");
    
    // Check health
    let health = service.expect("test: operation should succeed").health_check().await;
    assert!(health.is_ok(), "Health check should pass");
    assert!(health.expect("test: operation should succeed").is_healthy(), "Service should be healthy");
}

