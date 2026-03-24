// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

use super::*;
use crate::ecosystem::EcosystemConfig;
use ecosystem_api::types::{PrimalContext, PrimalRequest, PrimalType, SecurityLevel};
use std::sync::Arc;

fn test_ecosystem_config() -> EcosystemConfig {
    EcosystemConfig {
        service_id: Arc::from("test-squirrel"),
        service_name: "Test Squirrel".to_string(),
        service_host: "localhost".to_string(),
        service_port: 8080,
        service_mesh_endpoint: "http://localhost:8500".to_string(),
        biome_id: Some("test-biome".to_string()),
        ..Default::default()
    }
}

fn test_primal_context() -> PrimalContext {
    PrimalContext {
        user_id: Arc::from("test-user"),
        device_id: Arc::from("test-device"),
        session_id: Arc::from(uuid::Uuid::new_v4().to_string()),
        network_location: ecosystem_api::types::NetworkLocation::default(),
        biome_id: Some(Arc::from("test-biome")),
        security_level: SecurityLevel::Internal,
        metadata: std::collections::HashMap::new(),
    }
}

fn test_primal_request(operation: &str, payload: serde_json::Value) -> PrimalRequest {
    PrimalRequest {
        id: uuid::Uuid::new_v4(),
        operation: Arc::from(operation),
        payload,
        context: test_primal_context(),
        security_context: ecosystem_api::types::SecurityContext::default(),
        metadata: std::collections::HashMap::new(),
        timestamp: chrono::Utc::now(),
    }
}

#[test]
fn test_universal_squirrel_provider_new() {
    let config = test_ecosystem_config();
    let context = test_primal_context();
    let provider = UniversalSquirrelProvider::new(config, context).expect("create provider");
    assert_eq!(provider.primal_id(), "squirrel");
    assert_eq!(provider.primal_type(), PrimalType::Squirrel);
    assert!(!provider.instance_id().is_empty());
}

#[test]
fn test_set_session_manager() {
    let config = test_ecosystem_config();
    let context = test_primal_context();
    let mut provider = UniversalSquirrelProvider::new(config, context).expect("create provider");
    let session_manager = Arc::new(RwLock::new(SessionManagerImpl::new(
        crate::session::SessionConfig::default(),
    )));
    provider.set_session_manager(session_manager);
    // No assertion - just verify it doesn't panic
}

#[tokio::test]
async fn test_register_with_ecosystem() {
    let config = test_ecosystem_config();
    let context = test_primal_context();
    let mut provider = UniversalSquirrelProvider::new(config, context).expect("create provider");
    let registration = provider
        .register_with_ecosystem()
        .await
        .expect("register with ecosystem");
    assert!(registration.service_id.starts_with("squirrel-"));
    assert_eq!(registration.primal_type, PrimalType::Squirrel);
}

#[tokio::test]
async fn test_send_heartbeat() {
    let config = test_ecosystem_config();
    let context = test_primal_context();
    let provider = UniversalSquirrelProvider::new(config, context).expect("create provider");
    provider.send_heartbeat().await.expect("send heartbeat");
}

#[tokio::test]
async fn test_deregister_from_ecosystem() {
    let config = test_ecosystem_config();
    let context = test_primal_context();
    let mut provider = UniversalSquirrelProvider::new(config, context).expect("create provider");
    provider
        .deregister_from_ecosystem()
        .await
        .expect("deregister");
}

#[tokio::test]
async fn test_handle_ai_inference() {
    let config = test_ecosystem_config();
    let context = test_primal_context();
    let provider = UniversalSquirrelProvider::new(config, context).expect("create provider");
    let payload = serde_json::json!({"prompt": "Hello", "model": "test"});
    let response = provider
        .handle_ai_inference(payload)
        .await
        .expect("handle inference");
    assert!(response.get("response").is_some());
    assert!(response.get("model").is_some());
    // Model is selected by complexity routing (e.g. squirrel-ai-fast for low complexity)
    let model = response.get("model").and_then(|v| v.as_str()).unwrap_or("");
    assert!(!model.is_empty());
}

#[tokio::test]
async fn test_handle_primal_request_ai_inference() {
    let config = test_ecosystem_config();
    let context = test_primal_context();
    let provider = UniversalSquirrelProvider::new(config, context).expect("create provider");
    let request = test_primal_request(
        "ai_inference",
        serde_json::json!({"prompt": "test prompt", "model": "squirrel-ai-v1"}),
    );
    let response = provider
        .handle_primal_request(request)
        .await
        .expect("handle request");
    assert!(matches!(response.status, ResponseStatus::Success));
}

#[tokio::test]
async fn test_handle_primal_request_missing_prompt() {
    let config = test_ecosystem_config();
    let context = test_primal_context();
    let provider = UniversalSquirrelProvider::new(config, context).expect("create provider");
    let request = test_primal_request("ai_inference", serde_json::json!({"model": "test"}));
    let result = provider.handle_primal_request(request).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_handle_primal_request_health_check() {
    let config = test_ecosystem_config();
    let context = test_primal_context();
    let provider = UniversalSquirrelProvider::new(config, context).expect("create provider");
    let request = test_primal_request("health_check", serde_json::json!({}));
    let response = provider
        .handle_primal_request(request)
        .await
        .expect("handle request");
    assert!(matches!(response.status, ResponseStatus::Success));
}

#[tokio::test]
async fn test_handle_primal_request_unknown_operation() {
    let config = test_ecosystem_config();
    let context = test_primal_context();
    let provider = UniversalSquirrelProvider::new(config, context).expect("create provider");
    let request = test_primal_request("unknown_op", serde_json::json!({}));
    let response = provider
        .handle_primal_request(request)
        .await
        .expect("handle request");
    assert!(matches!(response.status, ResponseStatus::Error { .. }));
}

#[tokio::test]
async fn test_initialize() {
    let config = test_ecosystem_config();
    let context = test_primal_context();
    let mut provider = UniversalSquirrelProvider::new(config, context).expect("create provider");
    provider
        .initialize(serde_json::json!({}))
        .await
        .expect("initialize");
}

#[tokio::test]
async fn test_shutdown() {
    let config = test_ecosystem_config();
    let context = test_primal_context();
    let mut provider = UniversalSquirrelProvider::new(config, context).expect("create provider");
    provider.shutdown().await.expect("shutdown");
}

#[test]
fn test_capabilities() {
    let config = test_ecosystem_config();
    let context = test_primal_context();
    let provider = UniversalSquirrelProvider::new(config, context).expect("create provider");
    let caps = provider.capabilities();
    assert!(!caps.is_empty());
}

#[test]
fn test_dependencies() {
    let config = test_ecosystem_config();
    let context = test_primal_context();
    let provider = UniversalSquirrelProvider::new(config, context).expect("create provider");
    let deps = provider.dependencies();
    assert!(!deps.is_empty());
}

#[tokio::test]
async fn test_health_check() {
    let config = test_ecosystem_config();
    let context = test_primal_context();
    let provider = UniversalSquirrelProvider::new(config, context).expect("create provider");
    let health = provider.health_check().await;
    assert_eq!(health.status, HealthStatus::Healthy);
}

#[test]
fn test_endpoints() {
    let config = test_ecosystem_config();
    let context = test_primal_context();
    let provider = UniversalSquirrelProvider::new(config, context).expect("create provider");
    let endpoints = provider.endpoints();
    assert!(endpoints.primary.contains("localhost"));
    assert!(endpoints.health.contains("/health"));
}

#[test]
fn test_can_serve_context() {
    let config = test_ecosystem_config();
    let context = test_primal_context();
    let context_ref = context.clone();
    let provider = UniversalSquirrelProvider::new(config, context).expect("create provider");
    assert!(provider.can_serve_context(&context_ref));
}

#[test]
fn test_dynamic_port_info() {
    let config = test_ecosystem_config();
    let context = test_primal_context();
    let provider = UniversalSquirrelProvider::new(config, context).expect("create provider");
    let info = provider.dynamic_port_info();
    assert!(info.is_some());
    let info = info.expect("should succeed");
    assert_eq!(info.port, 8080);
}

#[tokio::test]
async fn test_register_with_service_mesh() {
    let config = test_ecosystem_config();
    let context = test_primal_context();
    let provider = UniversalSquirrelProvider::new(config, context).expect("create provider");
    let service_id = provider
        .register_with_service_mesh()
        .await
        .expect("register");
    assert!(!service_id.is_empty());
}

#[tokio::test]
async fn test_deregister_from_service_mesh() {
    let config = test_ecosystem_config();
    let context = test_primal_context();
    let mut provider = UniversalSquirrelProvider::new(config, context).expect("create provider");
    provider
        .deregister_from_service_mesh()
        .await
        .expect("deregister");
}

#[test]
fn test_get_service_mesh_status() {
    let config = test_ecosystem_config();
    let context = test_primal_context();
    let provider = UniversalSquirrelProvider::new(config, context).expect("create provider");
    let status = provider.get_service_mesh_status();
    assert!(!status.connected);
}

#[tokio::test]
async fn test_handle_ecosystem_request_health() {
    let config = test_ecosystem_config();
    let context = test_primal_context();
    let provider = UniversalSquirrelProvider::new(config, context).expect("create provider");
    let request = test_ecosystem_request("health_check", serde_json::json!({}));
    let response =
        ecosystem_api::EcosystemIntegration::handle_ecosystem_request(&provider, request)
            .await
            .expect("handle request");
    assert!(matches!(response.status, ResponseStatus::Success));
}

#[tokio::test]
async fn test_ecosystem_integration_register() {
    let config = test_ecosystem_config();
    let context = test_primal_context();
    let provider = UniversalSquirrelProvider::new(config, context).expect("create provider");
    let service_id = provider
        .register_with_service_mesh()
        .await
        .expect("register");
    assert!(!service_id.is_empty());
}

fn test_ecosystem_request(operation: &str, payload: serde_json::Value) -> EcosystemRequest {
    EcosystemRequest {
        request_id: uuid::Uuid::new_v4(),
        source_service: Arc::from("test"),
        target_service: Arc::from("squirrel"),
        operation: Arc::from(operation),
        payload,
        security_context: ecosystem_api::types::SecurityContext::default(),
        metadata: std::collections::HashMap::new(),
        timestamp: chrono::Utc::now(),
    }
}

#[tokio::test]
async fn test_ecosystem_integration_health_check() {
    let config = test_ecosystem_config();
    let context = test_primal_context();
    let provider = UniversalSquirrelProvider::new(config, context).expect("create provider");
    let request = test_ecosystem_request("health_check", serde_json::json!({}));
    let response =
        ecosystem_api::EcosystemIntegration::handle_ecosystem_request(&provider, request)
            .await
            .expect("handle");
    assert!(matches!(response.status, ResponseStatus::Success));
}

#[tokio::test]
async fn test_ecosystem_integration_capabilities() {
    let config = test_ecosystem_config();
    let context = test_primal_context();
    let provider = UniversalSquirrelProvider::new(config, context).expect("create provider");
    let request = test_ecosystem_request("capabilities", serde_json::json!({}));
    let response =
        ecosystem_api::EcosystemIntegration::handle_ecosystem_request(&provider, request)
            .await
            .expect("handle");
    assert!(matches!(response.status, ResponseStatus::Success));
}

#[tokio::test]
async fn test_report_health() {
    let config = test_ecosystem_config();
    let context = test_primal_context();
    let provider = UniversalSquirrelProvider::new(config, context).expect("create provider");
    ecosystem_api::EcosystemIntegration::report_health(&provider, HealthStatus::Healthy)
        .await
        .expect("report");
}

#[tokio::test]
async fn test_update_capabilities() {
    let config = test_ecosystem_config();
    let context = test_primal_context();
    let provider = UniversalSquirrelProvider::new(config, context).expect("create provider");
    let caps = provider.capabilities();
    ecosystem_api::UniversalPrimalProvider::update_capabilities(&provider, caps)
        .await
        .expect("update");
}

#[tokio::test]
async fn test_report_primal_health() {
    let config = test_ecosystem_config();
    let context = test_primal_context();
    let provider = UniversalSquirrelProvider::new(config, context).expect("create provider");
    let health = provider.health_check().await;
    ecosystem_api::UniversalPrimalProvider::report_health(&provider, health)
        .await
        .expect("report");
}
