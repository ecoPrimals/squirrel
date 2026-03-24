// SPDX-License-Identifier: AGPL-3.0-only
// ORC-Notice: Registry mechanics licensed under ORC
// Copyright (C) 2026 ecoPrimals Contributors

//! Registry unit tests

use super::*;
use async_trait::async_trait;
use chrono::{Duration, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::traits::{
    DynamicPortInfo, NetworkLocation, PortStatus, PortType, PrimalCapability, PrimalContext,
    PrimalDependency, PrimalEndpoints, PrimalError, PrimalHealth, PrimalProvider, PrimalRequest,
    PrimalRequestType, PrimalResponse, PrimalResponseType, PrimalResult, PrimalType, SecurityLevel,
};

/// Test-only mock PrimalProvider
struct MockPrimalProvider {
    primal_id: String,
    instance_id: String,
    context: PrimalContext,
    primal_type: PrimalType,
    capabilities: Vec<PrimalCapability>,
    health: PrimalHealth,
    can_serve_user: Option<String>,
}

impl MockPrimalProvider {
    fn new(instance_id: &str, primal_type: PrimalType, user_id: &str) -> Self {
        Self {
            primal_id: "test-primal".to_string(),
            instance_id: instance_id.to_string(),
            context: PrimalContext {
                user_id: user_id.to_string(),
                device_id: "test-device".to_string(),
                session_id: Uuid::new_v4().to_string(),
                network_location: NetworkLocation {
                    ip_address: "127.0.0.1".to_string(),
                    subnet: None,
                    network_id: None,
                    geo_location: None,
                },
                security_level: SecurityLevel::Standard,
                metadata: HashMap::new(),
            },
            primal_type,
            capabilities: vec![PrimalCapability::Authentication {
                methods: vec!["password".to_string()],
            }],
            health: PrimalHealth::Healthy,
            can_serve_user: None,
        }
    }

    fn with_health(mut self, health: PrimalHealth) -> Self {
        self.health = health;
        self
    }

    fn with_capability(mut self, cap: PrimalCapability) -> Self {
        self.capabilities.push(cap);
        self
    }
}

#[async_trait]
impl PrimalProvider for MockPrimalProvider {
    fn primal_id(&self) -> &str {
        &self.primal_id
    }

    fn instance_id(&self) -> &str {
        &self.instance_id
    }

    fn context(&self) -> &PrimalContext {
        &self.context
    }

    fn primal_type(&self) -> PrimalType {
        self.primal_type.clone()
    }

    fn capabilities(&self) -> Vec<PrimalCapability> {
        self.capabilities.clone()
    }

    fn dependencies(&self) -> Vec<PrimalDependency> {
        vec![]
    }

    async fn health_check(&self) -> PrimalHealth {
        self.health.clone()
    }

    fn endpoints(&self) -> PrimalEndpoints {
        PrimalEndpoints::default()
    }

    async fn handle_primal_request(&self, request: PrimalRequest) -> PrimalResult<PrimalResponse> {
        Ok(PrimalResponse {
            request_id: request.id,
            response_type: PrimalResponseType::HealthCheck,
            payload: HashMap::new(),
            timestamp: Utc::now(),
            success: true,
            error_message: None,
            metadata: None,
        })
    }

    async fn initialize(&mut self, _config: serde_json::Value) -> PrimalResult<()> {
        Ok(())
    }

    async fn shutdown(&mut self) -> PrimalResult<()> {
        Ok(())
    }

    fn can_serve_context(&self, context: &PrimalContext) -> bool {
        if let Some(ref user) = self.can_serve_user {
            context.user_id == *user
        } else {
            self.context.user_id == context.user_id
        }
    }

    fn dynamic_port_info(&self) -> Option<DynamicPortInfo> {
        Some(DynamicPortInfo {
            assigned_port: 8080,
            port_type: PortType::Http,
            status: PortStatus::Active,
            assigned_at: Utc::now(),
            lease_duration: Duration::hours(24),
        })
    }
}

#[tokio::test]
async fn test_registry_new_and_default() {
    let registry = UniversalPrimalRegistry::new();
    let stats = registry.get_statistics().await;
    assert_eq!(stats.total_primals, 0);
    assert!(stats.primal_counts.is_empty());

    let registry2 = UniversalPrimalRegistry::default();
    let stats2 = registry2.get_statistics().await;
    assert_eq!(stats2.total_primals, 0);
}

#[tokio::test]
async fn test_register_primal_success() {
    let registry = UniversalPrimalRegistry::new();
    let provider = Arc::new(MockPrimalProvider::new(
        "inst-1",
        PrimalType::Security,
        "user1",
    ));
    let context = provider.context().clone();

    let result = registry
        .register_primal_for_context(provider, context, None)
        .await;
    assert!(result.is_ok());

    let stats = registry.get_statistics().await;
    assert_eq!(stats.total_primals, 1);
}

#[tokio::test]
async fn test_register_primal_duplicate_fails() {
    let registry = UniversalPrimalRegistry::new();
    let provider = Arc::new(MockPrimalProvider::new(
        "inst-1",
        PrimalType::Security,
        "user1",
    ));
    let context = provider.context().clone();

    registry
        .register_primal_for_context(provider.clone(), context.clone(), None)
        .await
        .expect("should succeed");

    let result = registry
        .register_primal_for_context(provider, context, None)
        .await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), PrimalError::AlreadyExists(_)));
}

#[tokio::test]
async fn test_register_unhealthy_primal_fails() {
    let registry = UniversalPrimalRegistry::new();
    let provider = Arc::new(
        MockPrimalProvider::new("inst-1", PrimalType::Security, "user1").with_health(
            PrimalHealth::Unhealthy {
                reason: "broken".to_string(),
            },
        ),
    );
    let context = provider.context().clone();

    let result = registry
        .register_primal_for_context(provider, context, None)
        .await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        PrimalError::ServiceUnavailable(_)
    ));
}

#[tokio::test]
async fn test_register_degraded_primal_succeeds() {
    let registry = UniversalPrimalRegistry::new();
    let provider = Arc::new(
        MockPrimalProvider::new("inst-1", PrimalType::Security, "user1").with_health(
            PrimalHealth::Degraded {
                issues: vec!["slow".to_string()],
            },
        ),
    );
    let context = provider.context().clone();

    let result = registry
        .register_primal_for_context(provider, context, None)
        .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_find_for_context() {
    let registry = UniversalPrimalRegistry::new();
    let provider = Arc::new(MockPrimalProvider::new(
        "inst-1",
        PrimalType::Security,
        "user1",
    ));
    let context = provider.context().clone();

    registry
        .register_primal_for_context(provider.clone(), context.clone(), None)
        .await
        .expect("should succeed");

    let found = registry.find_for_context(&context).await;
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].instance_id(), "inst-1");

    let other_context = PrimalContext {
        user_id: "other-user".to_string(),
        ..context.clone()
    };
    let found_other = registry.find_for_context(&other_context).await;
    assert!(found_other.is_empty());
}

#[tokio::test]
async fn test_find_by_capability_for_context() {
    let registry = UniversalPrimalRegistry::new();
    // MockPrimalProvider already has Authentication capability by default
    let provider = Arc::new(MockPrimalProvider::new(
        "inst-1",
        PrimalType::Security,
        "user1",
    ));
    let context = provider.context().clone();

    registry
        .register_primal_for_context(provider.clone(), context.clone(), None)
        .await
        .expect("should succeed");

    let cap = PrimalCapability::Authentication {
        methods: vec!["password".to_string()],
    };
    let found = registry
        .find_by_capability_for_context(&cap, &context)
        .await;
    assert_eq!(found.len(), 1);
}

#[tokio::test]
async fn test_find_by_capability_with_extra_capability() {
    let registry = UniversalPrimalRegistry::new();
    let provider = Arc::new(
        MockPrimalProvider::new("inst-1", PrimalType::Security, "user1").with_capability(
            PrimalCapability::Encryption {
                algorithms: vec!["AES256".to_string()],
            },
        ),
    );
    let context = provider.context().clone();

    registry
        .register_primal_for_context(provider.clone(), context.clone(), None)
        .await
        .expect("should succeed");

    let enc_cap = PrimalCapability::Encryption {
        algorithms: vec!["AES256".to_string()],
    };
    let found = registry
        .find_by_capability_for_context(&enc_cap, &context)
        .await;
    assert_eq!(found.len(), 1);
}

#[tokio::test]
async fn test_route_request_with_context() {
    let registry = UniversalPrimalRegistry::new();
    let provider = Arc::new(MockPrimalProvider::new(
        "inst-1",
        PrimalType::Security,
        "user1",
    ));
    let context = provider.context().clone();

    registry
        .register_primal_for_context(provider.clone(), context.clone(), None)
        .await
        .expect("should succeed");

    let request = PrimalRequest {
        id: Uuid::new_v4(),
        request_type: PrimalRequestType::HealthCheck,
        payload: HashMap::new(),
        timestamp: Utc::now(),
        context: None,
        priority: None,
        security_level: None,
    };

    let response = registry.route_request_with_context(request, &context).await;
    assert!(response.is_ok());
    assert!(response.expect("should succeed").success);
}

#[tokio::test]
async fn test_route_request_no_primal_fails() {
    let registry = UniversalPrimalRegistry::new();
    let context = PrimalContext::default();

    let request = PrimalRequest {
        id: Uuid::new_v4(),
        request_type: PrimalRequestType::HealthCheck,
        payload: HashMap::new(),
        timestamp: Utc::now(),
        context: None,
        priority: None,
        security_level: None,
    };

    let response = registry.route_request_with_context(request, &context).await;
    assert!(response.is_err());
    assert!(matches!(response.unwrap_err(), PrimalError::NotFound(_)));
}

#[tokio::test]
async fn test_route_request_to_instance() {
    let registry = UniversalPrimalRegistry::new();
    let provider = Arc::new(MockPrimalProvider::new(
        "inst-1",
        PrimalType::Security,
        "user1",
    ));
    let context = provider.context().clone();

    registry
        .register_primal_for_context(provider.clone(), context.clone(), None)
        .await
        .expect("should succeed");

    let request = PrimalRequest {
        id: Uuid::new_v4(),
        request_type: PrimalRequestType::HealthCheck,
        payload: HashMap::new(),
        timestamp: Utc::now(),
        context: None,
        priority: None,
        security_level: None,
    };

    let response = registry
        .route_request_to_instance(request.clone(), "inst-1")
        .await;
    assert!(response.is_ok());

    let response_missing = registry
        .route_request_to_instance(request, "nonexistent")
        .await;
    assert!(response_missing.is_err());
}

#[tokio::test]
async fn test_port_info() {
    let registry = UniversalPrimalRegistry::new();
    let provider = Arc::new(MockPrimalProvider::new(
        "inst-1",
        PrimalType::Security,
        "user1",
    ));
    let context = provider.context().clone();
    let port_info = DynamicPortInfo {
        assigned_port: 9000,
        port_type: PortType::Http,
        status: PortStatus::Active,
        assigned_at: Utc::now(),
        lease_duration: Duration::hours(1),
    };

    registry
        .register_primal_for_context(provider, context, Some(port_info.clone()))
        .await
        .expect("should succeed");

    let retrieved = registry.get_port_info("inst-1").await;
    assert!(retrieved.is_some());
    assert_eq!(retrieved.expect("should succeed").assigned_port, 9000);

    registry
        .update_port_info(
            "inst-1",
            DynamicPortInfo {
                assigned_port: 9001,
                ..port_info.clone()
            },
        )
        .await
        .expect("should succeed");
    let updated = registry.get_port_info("inst-1").await;
    assert_eq!(updated.expect("should succeed").assigned_port, 9001);
}

#[tokio::test]
async fn test_get_instances_by_type() {
    let registry = UniversalPrimalRegistry::new();
    let p1 = Arc::new(MockPrimalProvider::new(
        "inst-1",
        PrimalType::Security,
        "user1",
    ));
    let p2 = Arc::new(MockPrimalProvider::new(
        "inst-2",
        PrimalType::Security,
        "user2",
    ));

    registry
        .register_primal_for_context(p1.clone(), p1.context().clone(), None)
        .await
        .expect("should succeed");
    registry
        .register_primal_for_context(p2.clone(), p2.context().clone(), None)
        .await
        .expect("should succeed");

    let instances = registry.get_instances_by_type(PrimalType::Security).await;
    assert_eq!(instances.len(), 2);

    let instances_ai = registry.get_instances_by_type(PrimalType::AI).await;
    assert!(instances_ai.is_empty());
}

#[tokio::test]
async fn test_get_instances_for_user() {
    let registry = UniversalPrimalRegistry::new();
    let provider = Arc::new(MockPrimalProvider::new(
        "inst-1",
        PrimalType::Security,
        "alice",
    ));

    registry
        .register_primal_for_context(provider.clone(), provider.context().clone(), None)
        .await
        .expect("should succeed");

    let alice_instances = registry.get_instances_for_user("alice").await;
    assert_eq!(alice_instances.len(), 1);

    let bob_instances = registry.get_instances_for_user("bob").await;
    assert!(bob_instances.is_empty());
}

#[tokio::test]
async fn test_unregister_instance() {
    let registry = UniversalPrimalRegistry::new();
    let provider = Arc::new(MockPrimalProvider::new(
        "inst-1",
        PrimalType::Security,
        "user1",
    ));
    let context = provider.context().clone();

    registry
        .register_primal_for_context(provider.clone(), context, None)
        .await
        .expect("should succeed");

    let result = registry.unregister_instance("inst-1").await;
    assert!(result.is_ok());

    let stats = registry.get_statistics().await;
    assert_eq!(stats.total_primals, 0);

    let unreg_missing = registry.unregister_instance("nonexistent").await;
    assert!(unreg_missing.is_err());
}

#[tokio::test]
async fn test_enhanced_statistics() {
    let registry = UniversalPrimalRegistry::new();
    let p1 = Arc::new(MockPrimalProvider::new(
        "inst-1",
        PrimalType::Security,
        "user1",
    ));
    let p2 = Arc::new(MockPrimalProvider::new(
        "inst-2",
        PrimalType::Security,
        "user1",
    ));

    registry
        .register_primal_for_context(p1.clone(), p1.context().clone(), None)
        .await
        .expect("should succeed");
    registry
        .register_primal_for_context(p2.clone(), p2.context().clone(), None)
        .await
        .expect("should succeed");

    let stats = registry.get_enhanced_statistics().await;
    assert_eq!(stats.total_instances, 2);
    assert_eq!(stats.total_users, 1);
    assert_eq!(stats.instances_by_user.get("user1"), Some(&2));
}

#[tokio::test]
async fn test_health_check_all() {
    let registry = UniversalPrimalRegistry::new();
    let provider = Arc::new(MockPrimalProvider::new(
        "inst-1",
        PrimalType::Security,
        "user1",
    ));
    let context = provider.context().clone();

    registry
        .register_primal_for_context(provider.clone(), context, None)
        .await
        .expect("should succeed");

    let health_results = registry.health_check_all().await;
    assert_eq!(health_results.len(), 1);
    assert_eq!(health_results[0].0, "inst-1");
    assert!(matches!(health_results[0].1, PrimalHealth::Healthy));
}

#[tokio::test]
async fn test_discovered_primal_fields() {
    let primal = DiscoveredPrimal {
        id: "squirrel".to_string(),
        instance_id: "squirrel-1".to_string(),
        primal_type: PrimalType::Coordinator,
        capabilities: vec![],
        endpoint: "/tmp/squirrel.sock".to_string(),
        health: PrimalHealth::Healthy,
        context: PrimalContext::default(),
        port_info: None,
    };
    assert_eq!(primal.id, "squirrel");
    assert_eq!(primal.instance_id, "squirrel-1");
    assert_eq!(primal.endpoint, "/tmp/squirrel.sock");
}
