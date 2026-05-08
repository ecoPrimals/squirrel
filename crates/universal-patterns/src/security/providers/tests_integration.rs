// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Integration tests for SecurityProviderIntegration (capability-based security delegation)
//! and multi-provider registry workflows.

use super::*;
use crate::config::AuthMethod;
use crate::security::context::SecurityContext;
use crate::traits::{Credentials, Principal, PrincipalType};
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;

fn box_local_service(p: LocalSecurityProvider) -> Arc<UniversalSecurityProviderBox> {
    Arc::new(UniversalSecurityProviderBox::Local(Arc::new(p)))
}

fn box_security_provider_service(
    p: SecurityProviderIntegration,
) -> Arc<UniversalSecurityProviderBox> {
    Arc::new(UniversalSecurityProviderBox::SecurityProvider(Arc::new(p)))
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
async fn test_beardog_provider_new() {
    let config = SecurityServiceConfig::default();
    let result = SecurityProviderIntegration::new(config).await;

    assert!(
        result.is_ok(),
        "Should create beardog provider successfully"
    );
}

#[tokio::test]
async fn test_beardog_provider_get_capabilities() {
    let config = SecurityServiceConfig::default();
    let provider = SecurityProviderIntegration::new(config)
        .await
        .expect("test: operation should succeed");

    let capabilities = provider.get_capabilities();

    assert_eq!(
        capabilities.len(),
        4,
        "Security provider should have 4 capabilities"
    );

    let has_auth = capabilities
        .iter()
        .any(|cap| matches!(cap, SecurityCapability::Authentication { .. }));
    assert!(has_auth, "Should have authentication capability");

    let has_authz = capabilities
        .iter()
        .any(|cap| matches!(cap, SecurityCapability::Authorization { .. }));
    assert!(has_authz, "Should have authorization capability");

    let has_crypto = capabilities
        .iter()
        .any(|cap| matches!(cap, SecurityCapability::Cryptography { .. }));
    assert!(has_crypto, "Should have cryptography capability");

    let has_compliance = capabilities
        .iter()
        .any(|cap| matches!(cap, SecurityCapability::Compliance { .. }));
    assert!(has_compliance, "Should have compliance capability");
}

#[tokio::test]
async fn test_beardog_provider_get_service_info() {
    #[expect(
        deprecated,
        reason = "test uses BEARDOG_SECURITY_SERVICE_ID legacy wire id"
    )]
    let config = SecurityServiceConfig {
        service_id: BEARDOG_SECURITY_SERVICE_ID.to_string(),
        ..Default::default()
    };
    let provider = SecurityProviderIntegration::new(config)
        .await
        .expect("test: operation should succeed");

    let info = provider.get_service_info();

    assert_eq!(info.service_id, SECURITY_SERVICE_ID);
    assert_eq!(info.name, "Security Provider Service");
    assert_eq!(info.version, "1.0.0");
    assert!(matches!(info.trust_level, TrustLevel::High));
    assert!(
        info.supported_protocols
            .contains(&"json-rpc-2.0".to_string())
    );
    assert!(
        info.supported_protocols
            .contains(&"unix-socket".to_string())
    );
    assert!(!info.compliance_certifications.is_empty());
}

#[tokio::test]
async fn test_beardog_provider_health_check() {
    let config = SecurityServiceConfig::default();
    let provider = SecurityProviderIntegration::new(config)
        .await
        .expect("test: operation should succeed");

    let health = provider.health_check().await;

    assert!(health.is_ok(), "Health check should succeed");
    let health = health.expect("test: operation should succeed");
    assert!(matches!(health.status, HealthStatus::Healthy));
    assert!(health.message.contains("Security provider"));
}

#[tokio::test]
async fn test_beardog_provider_handle_request() {
    let config = SecurityServiceConfig::default();
    let provider = SecurityProviderIntegration::new(config)
        .await
        .expect("test: operation should succeed");

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
    let mut provider = SecurityProviderIntegration::new(config)
        .await
        .expect("test: operation should succeed");

    let new_config = SecurityServiceConfig {
        service_id: "beardog-updated".to_string(),
        endpoint: Some("https://beardog.example.com".to_string()),
        timeout_seconds: Some(120),
        ..Default::default()
    };

    let result = provider.initialize(new_config).await;

    assert!(result.is_ok(), "Should initialize successfully");
}

#[tokio::test]
async fn beardog_integration_factory_delegates_to_new() {
    let config = SecurityServiceConfig::default();
    let p = SecurityProviderFactory::new(config)
        .await
        .expect("integration new");
    assert_eq!(p.get_service_info().service_id, SECURITY_SERVICE_ID);
}

#[tokio::test]
async fn beardog_universal_trait_authenticate_variants_cover_branches() {
    let config = SecurityServiceConfig {
        service_id: "other-security".to_string(),
        ..Default::default()
    };
    let provider = SecurityProviderIntegration::new(config)
        .await
        .expect("provider");
    assert!(matches!(
        provider.get_service_info().trust_level,
        TrustLevel::Medium
    ));

    let _ = crate::security::traits::UniversalSecurityProvider::authenticate(
        &provider,
        &Credentials::Password {
            username: "u".to_string(),
            password: "p".to_string(),
        },
    )
    .await
    .expect("password");
    let _ = crate::security::traits::UniversalSecurityProvider::authenticate(
        &provider,
        &Credentials::ApiKey {
            key: "k".to_string(),
            service_id: "api-svc".to_string(),
        },
    )
    .await
    .expect("api key");
    let _ = crate::security::traits::UniversalSecurityProvider::authenticate(
        &provider,
        &Credentials::Bearer {
            token: "tokentokentoken".to_string(),
        },
    )
    .await
    .expect("bearer");
    let _ = crate::security::traits::UniversalSecurityProvider::authenticate(
        &provider,
        &Credentials::Token {
            token: "tokentokentoken".to_string(),
        },
    )
    .await
    .expect("jwt");
    let _ = crate::security::traits::UniversalSecurityProvider::authenticate(
        &provider,
        &Credentials::ServiceAccount {
            service_id: "sa".to_string(),
            api_key: "k".to_string(),
        },
    )
    .await
    .expect("svc acct");
    let _ = crate::security::traits::UniversalSecurityProvider::authenticate(
        &provider,
        &Credentials::Bootstrap {
            service_id: "boot".to_string(),
        },
    )
    .await
    .expect("bootstrap");
    let _ = crate::security::traits::UniversalSecurityProvider::authenticate(
        &provider,
        &Credentials::Test {
            service_id: "t".to_string(),
        },
    )
    .await
    .expect("test cred");
    let _ = crate::security::traits::UniversalSecurityProvider::authenticate(
        &provider,
        &Credentials::Custom(HashMap::new()),
    )
    .await
    .expect("custom -> anonymous branch");
}

#[tokio::test]
async fn beardog_universal_authorize_encrypt_sign_verify_audit_health() {
    let config = SecurityServiceConfig {
        service_id: "svc-1".to_string(),
        ..Default::default()
    };
    let provider = SecurityProviderIntegration::new(config)
        .await
        .expect("provider");

    let mut principal = Principal {
        id: "p1".to_string(),
        name: "P".to_string(),
        principal_type: PrincipalType::User,
        roles: vec![],
        permissions: vec!["read".to_string()],
        metadata: HashMap::new(),
    };
    assert!(
        crate::security::traits::UniversalSecurityProvider::authorize(
            &provider, &principal, "read", "r1"
        )
        .await
        .expect("authorize")
    );
    assert!(
        !crate::security::traits::UniversalSecurityProvider::authorize(
            &provider, &principal, "delete", "r1"
        )
        .await
        .expect("deny")
    );
    principal.permissions = vec!["*".to_string()];
    assert!(
        crate::security::traits::UniversalSecurityProvider::authorize(
            &provider, &principal, "any", "r"
        )
        .await
        .expect("star")
    );

    let plain = b"hello world";
    let enc = crate::security::traits::UniversalSecurityProvider::encrypt(&provider, plain)
        .await
        .expect("enc");
    let dec = crate::security::traits::UniversalSecurityProvider::decrypt(&provider, &enc)
        .await
        .expect("dec");
    assert_eq!(dec, plain);

    let sig = crate::security::traits::UniversalSecurityProvider::sign(&provider, plain)
        .await
        .expect("sign");
    assert!(
        crate::security::traits::UniversalSecurityProvider::verify(&provider, plain, &sig)
            .await
            .expect("verify ok")
    );
    assert!(
        !crate::security::traits::UniversalSecurityProvider::verify(&provider, plain, &[0u8])
            .await
            .expect("verify bad")
    );

    crate::security::traits::UniversalSecurityProvider::audit_log(
        &provider,
        "op",
        &create_test_security_context(),
    )
    .await
    .expect("audit");

    let h = crate::security::traits::UniversalSecurityProvider::health_check(&provider)
        .await
        .expect("health trait");
    assert!(h.is_healthy());
}

#[tokio::test]
async fn test_register_security_service_function() {
    let mut registry = UniversalSecurityRegistry::new();
    let config = SecurityServiceConfig::default();
    let provider = LocalSecurityProvider::new(config)
        .await
        .expect("test: operation should succeed");
    let service = box_local_service(provider);

    let result = register_security_service(&mut registry, service).await;

    assert!(result.is_ok(), "Should register service successfully");
    assert!(
        registry
            .list_services()
            .contains(&"local-security".to_string())
    );
}

#[tokio::test]
async fn test_multi_provider_registry() {
    let mut registry = UniversalSecurityRegistry::new();

    let local_config = SecurityServiceConfig {
        service_id: "local".to_string(),
        ..Default::default()
    };
    let local_provider = LocalSecurityProvider::new(local_config)
        .await
        .expect("test: operation should succeed");
    registry
        .register_service(
            "local-service".to_string(),
            box_local_service(local_provider),
        )
        .await
        .expect("test: operation should succeed");

    let beardog_config = SecurityServiceConfig {
        service_id: "beardog".to_string(),
        ..Default::default()
    };
    let beardog_provider = SecurityProviderIntegration::new(beardog_config)
        .await
        .expect("test: operation should succeed");
    registry
        .register_service(
            "beardog-service".to_string(),
            box_security_provider_service(beardog_provider),
        )
        .await
        .expect("test: operation should succeed");

    let services = registry.list_services();
    assert_eq!(services.len(), 2, "Should have 2 services registered");

    let requirements = vec![SecurityCapability::Authentication {
        methods: vec![AuthMethod::None],
        multi_factor: false,
        session_management: false,
    }];
    let result = registry.find_optimal_service(requirements).await;
    assert!(
        result.is_ok(),
        "Should find at least 1 service with authentication"
    );

    let compliance_cap = SecurityCapability::Compliance {
        standards: vec!["SOX".to_string(), "GDPR".to_string()],
        audit_logging: true,
        real_time_monitoring: true,
    };
    let compliance_services = registry.find_by_capability(&compliance_cap);
    assert_eq!(
        compliance_services.len(),
        1,
        "Only beardog should have compliance capability"
    );
    assert_eq!(compliance_services[0], "beardog-service");
}

#[tokio::test]
async fn test_end_to_end_security_workflow() {
    let mut registry = UniversalSecurityRegistry::new();

    let config = SecurityServiceConfig {
        service_id: SECURITY_SERVICE_ID.to_string(),
        endpoint: Some("https://security.local".to_string()),
        ..Default::default()
    };
    let provider = SecurityProviderIntegration::new(config)
        .await
        .expect("test: operation should succeed");
    registry
        .register_service(
            SECURITY_SERVICE_ID.to_string(),
            box_security_provider_service(provider),
        )
        .await
        .expect("test: operation should succeed");

    let requirements = vec![
        SecurityCapability::Authentication {
            methods: vec![AuthMethod::SecurityProvider {
                service_id: SECURITY_PRIMARY_SERVICE_ID.to_string(),
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

    let service = registry.get_service(&service_id.expect("test: operation should succeed"));
    assert!(service.is_some(), "Should retrieve service");

    let health = service
        .expect("test: operation should succeed")
        .health_check()
        .await;
    assert!(health.is_ok(), "Health check should pass");
    assert!(
        health.expect("test: operation should succeed").is_healthy(),
        "Service should be healthy"
    );
}
