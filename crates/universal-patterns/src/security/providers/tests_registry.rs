// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Tests for UniversalSecurityRegistry and LocalSecurityProvider.

use super::*;
use crate::config::AuthMethod;
use crate::security::context::SecurityContext;
use crate::traits::{Principal, PrincipalType};
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;

fn box_local_service(p: LocalSecurityProvider) -> Arc<UniversalSecurityProviderBox> {
    Arc::new(UniversalSecurityProviderBox::Local(Arc::new(p)))
}

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

#[tokio::test]
async fn test_registry_new() {
    let registry = UniversalSecurityRegistry::new();

    assert_eq!(
        registry.list_services().len(),
        0,
        "New registry should be empty"
    );
}

#[tokio::test]
async fn test_registry_register_service() {
    let mut registry = UniversalSecurityRegistry::new();
    let config = SecurityServiceConfig::default();
    let provider = LocalSecurityProvider::new(config)
        .await
        .expect("test: operation should succeed");
    let service = box_local_service(provider);

    let result = registry
        .register_service("test-service".to_string(), service)
        .await;

    assert!(result.is_ok(), "Should register service successfully");
    assert_eq!(
        registry.list_services().len(),
        1,
        "Registry should have 1 service"
    );
    assert!(
        registry
            .list_services()
            .contains(&"test-service".to_string())
    );
}

#[tokio::test]
async fn test_registry_find_by_capability() {
    let mut registry = UniversalSecurityRegistry::new();
    let config = SecurityServiceConfig::default();
    let provider = LocalSecurityProvider::new(config)
        .await
        .expect("test: operation should succeed");
    let service = box_local_service(provider);

    registry
        .register_service("local-service".to_string(), service)
        .await
        .expect("test: operation should succeed");

    let caps = registry
        .get_service("local-service")
        .expect("service registered")
        .get_capabilities();
    let capability = caps
        .into_iter()
        .find(|c| matches!(c, SecurityCapability::Authentication { .. }))
        .expect("provider advertises Authentication");

    let services = registry.find_by_capability(&capability);

    assert_eq!(
        services.len(),
        1,
        "Should find 1 service with authentication capability"
    );
    assert_eq!(services[0], "local-service");
}

#[tokio::test]
async fn test_registry_find_by_capability_no_match() {
    let mut registry = UniversalSecurityRegistry::new();
    let config = SecurityServiceConfig::default();
    let provider = LocalSecurityProvider::new(config)
        .await
        .expect("test: operation should succeed");
    let service = box_local_service(provider);

    registry
        .register_service("local-service".to_string(), service)
        .await
        .expect("test: operation should succeed");

    let capability = SecurityCapability::ThreatDetection {
        anomaly_detection: true,
        real_time_analysis: true,
        threat_intelligence: true,
    };

    let services = registry.find_by_capability(&capability);

    assert_eq!(
        services.len(),
        0,
        "Should find no services with threat detection capability"
    );
}

#[tokio::test]
async fn test_registry_get_service() {
    let mut registry = UniversalSecurityRegistry::new();
    let config = SecurityServiceConfig::default();
    let provider = LocalSecurityProvider::new(config)
        .await
        .expect("test: operation should succeed");
    let service = box_local_service(provider);

    registry
        .register_service("test-service".to_string(), service)
        .await
        .expect("test: operation should succeed");

    let retrieved = registry.get_service("test-service");

    assert!(retrieved.is_some(), "Should retrieve registered service");

    let non_existent = registry.get_service("non-existent");
    assert!(
        non_existent.is_none(),
        "Should not retrieve non-existent service"
    );
}

#[tokio::test]
async fn test_registry_list_services() {
    let mut registry = UniversalSecurityRegistry::new();

    let config1 = SecurityServiceConfig {
        service_id: "service-1".to_string(),
        ..Default::default()
    };
    let provider1 = LocalSecurityProvider::new(config1)
        .await
        .expect("test: operation should succeed");
    registry
        .register_service("service-1".to_string(), box_local_service(provider1))
        .await
        .expect("test: operation should succeed");

    let config2 = SecurityServiceConfig {
        service_id: "service-2".to_string(),
        ..Default::default()
    };
    let provider2 = LocalSecurityProvider::new(config2)
        .await
        .expect("test: operation should succeed");
    registry
        .register_service("service-2".to_string(), box_local_service(provider2))
        .await
        .expect("test: operation should succeed");

    let services = registry.list_services();

    assert_eq!(services.len(), 2, "Should list 2 services");
    assert!(services.contains(&"service-1".to_string()));
    assert!(services.contains(&"service-2".to_string()));
}

#[tokio::test]
async fn test_registry_find_optimal_service() {
    let mut registry = UniversalSecurityRegistry::new();

    let config = SecurityServiceConfig::default();
    let provider = LocalSecurityProvider::new(config)
        .await
        .expect("test: operation should succeed");
    registry
        .register_service("local-service".to_string(), box_local_service(provider))
        .await
        .expect("test: operation should succeed");

    let requirements = vec![SecurityCapability::Authentication {
        methods: vec![AuthMethod::None],
        multi_factor: false,
        session_management: false,
    }];

    let result = registry.find_optimal_service(requirements).await;

    assert!(result.is_ok(), "Should find optimal service");
    assert_eq!(
        result.expect("test: operation should succeed"),
        "local-service"
    );
}

#[tokio::test]
async fn test_registry_find_optimal_service_no_match() {
    let mut registry = UniversalSecurityRegistry::new();

    let config = SecurityServiceConfig::default();
    let provider = LocalSecurityProvider::new(config)
        .await
        .expect("test: operation should succeed");
    registry
        .register_service("local-service".to_string(), box_local_service(provider))
        .await
        .expect("test: operation should succeed");

    let requirements = vec![SecurityCapability::ThreatDetection {
        anomaly_detection: true,
        real_time_analysis: true,
        threat_intelligence: true,
    }];

    let result = registry.find_optimal_service(requirements).await;

    assert!(
        result.is_err(),
        "Should not find service for unmatched requirements"
    );
}

#[tokio::test]
async fn test_local_provider_new() {
    let config = SecurityServiceConfig::default();
    let result = LocalSecurityProvider::new(config).await;

    assert!(result.is_ok(), "Should create local provider successfully");
}

#[tokio::test]
async fn test_local_provider_get_capabilities() {
    let config = SecurityServiceConfig::default();
    let provider = LocalSecurityProvider::new(config)
        .await
        .expect("test: operation should succeed");

    let capabilities = provider.get_capabilities();

    assert_eq!(
        capabilities.len(),
        2,
        "Local provider should have 2 capabilities"
    );

    let has_auth = capabilities
        .iter()
        .any(|cap| matches!(cap, SecurityCapability::Authentication { .. }));
    assert!(has_auth, "Should have authentication capability");

    let has_crypto = capabilities
        .iter()
        .any(|cap| matches!(cap, SecurityCapability::Cryptography { .. }));
    assert!(has_crypto, "Should have cryptography capability");
}

#[tokio::test]
async fn test_local_provider_get_service_info() {
    let config = SecurityServiceConfig {
        service_id: "local-security".to_string(),
        ..Default::default()
    };
    let provider = LocalSecurityProvider::new(config)
        .await
        .expect("test: operation should succeed");

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
    let provider = LocalSecurityProvider::new(config)
        .await
        .expect("test: operation should succeed");

    let health = provider.health_check().await;

    assert!(health.is_ok(), "Health check should succeed");
    let health = health.expect("test: operation should succeed");
    assert!(matches!(health.status, HealthStatus::Healthy));
    assert!(health.message.contains("Local security"));
}

#[tokio::test]
async fn test_local_provider_handle_request() {
    let config = SecurityServiceConfig::default();
    let provider = LocalSecurityProvider::new(config)
        .await
        .expect("test: operation should succeed");

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
    let mut provider = LocalSecurityProvider::new(config)
        .await
        .expect("test: operation should succeed");

    let new_config = SecurityServiceConfig {
        service_id: "updated-service".to_string(),
        timeout_seconds: Some(60),
        ..Default::default()
    };

    let result = provider.initialize(new_config.clone()).await;

    assert!(result.is_ok(), "Should initialize successfully");
}
