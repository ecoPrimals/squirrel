// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Real Primal-to-Primal Integration Tests
//!
//! These tests verify actual communication between primal services without mocks.
//! They test the full integration stack including:
//! - Network communication
//! - Service discovery
//! - Authentication flows
//! - MCP protocol implementation
//! - Error handling and resilience

mod integration;

use integration::*;
use integration::framework::TestEnvironmentBuilder;
use integration::fixtures::{TestUser, messages, configs};
use integration::assertions::*;
use std::time::Duration;

/// Test 1: Basic Squirrel Health Check
///
/// Verifies that Squirrel can start and respond to health checks.
/// This is the foundation for all other integration tests.
#[tokio::test]
#[ignore] // Requires actual service deployment
async fn test_01_squirrel_startup_and_health() {
    let env = TestEnvironmentBuilder::new("squirrel_health")
        .with_timeout(Duration::from_secs(30))
        .with_service(ServiceType::Squirrel)
        .build()
        .await
        .expect("Failed to build test environment");
    
    // Get Squirrel service
    let services = env.services.read().await;
    let squirrel = services.values()
        .find(|s| s.service_type == ServiceType::Squirrel)
        .expect("Squirrel service not found");
    
    // Verify health endpoint responds
    assert_service_healthy(&squirrel.health_endpoint)
        .await
        .expect("Squirrel health check failed");
    
    env.cleanup().await.expect("Cleanup failed");
}

/// Test 2: Squirrel to Songbird Communication
///
/// Tests service discovery and basic message exchange between Squirrel and Songbird.
/// Verifies capability-based discovery and MCP message routing.
#[tokio::test]
#[ignore] // Requires actual service deployment
async fn test_02_squirrel_to_songbird_communication() {
    let env = TestEnvironmentBuilder::new("squirrel_songbird")
        .with_timeout(Duration::from_secs(60))
        .with_service(ServiceType::Squirrel)
        .with_service(ServiceType::Songbird)
        .build()
        .await
        .expect("Failed to build test environment");
    
    let services = env.services.read().await;
    let squirrel = services.values()
        .find(|s| s.service_type == ServiceType::Squirrel)
        .expect("Squirrel not found");
    let songbird = services.values()
        .find(|s| s.service_type == ServiceType::Songbird)
        .expect("Songbird not found");
    
    // Register Songbird's orchestration capability
    let registration = messages::service_registration("songbird");
    assert_mcp_exchange_succeeds(&squirrel.base_url, &songbird.base_url, registration)
        .await
        .expect("Service registration failed");
    
    // Discover orchestration capability
    let services_found = assert_capability_discovered(
        &squirrel.base_url,
        "orchestration"
    ).await.expect("Capability discovery failed");
    
    assert!(!services_found.is_empty(), "No orchestration services found");
    
    // Send test message via MCP
    let ping = messages::ping_message();
    let response = assert_mcp_exchange_succeeds(&squirrel.base_url, &songbird.base_url, ping)
        .await
        .expect("MCP message exchange failed");
    
    assert!(response.get("pong").is_some(), "Expected pong response");
    
    env.cleanup().await.expect("Cleanup failed");
}

/// Test 3: Authentication Flow with BearDog
///
/// Tests end-to-end authentication flow between Squirrel and BearDog.
/// Verifies:
/// - JWT token generation
/// - Token validation
/// - Role-based access control
#[tokio::test]
#[ignore] // Requires actual service deployment
async fn test_03_authentication_flow_with_beardog() {
    let env = TestEnvironmentBuilder::new("squirrel_beardog_auth")
        .with_timeout(Duration::from_secs(60))
        .with_service(ServiceType::Squirrel)
        .with_service(ServiceType::BearDog)
        .build()
        .await
        .expect("Failed to build test environment");
    
    let services = env.services.read().await;
    let squirrel = services.values()
        .find(|s| s.service_type == ServiceType::Squirrel)
        .expect("Squirrel not found");
    let beardog = services.values()
        .find(|s| s.service_type == ServiceType::BearDog)
        .expect("BearDog not found");
    
    // Test user authentication
    let test_user = TestUser::alice();
    let client = reqwest::Client::new();
    
    // Request authentication token
    let auth_response = client
        .post(format!("{}/auth/login", beardog.base_url))
        .json(&serde_json::json!({
            "username": test_user.username,
            "password": "test_password_123",
        }))
        .send()
        .await
        .expect("Auth request failed");
    
    assert!(auth_response.status().is_success(), "Authentication failed");
    
    let auth_data: serde_json::Value = auth_response.json()
        .await
        .expect("Failed to parse auth response");
    
    let token = auth_data.get("token")
        .and_then(|t| t.as_str())
        .expect("No token in response");
    
    // Use token to access Squirrel
    let protected_response = client
        .get(format!("{}/api/user/profile", squirrel.base_url))
        .bearer_auth(token)
        .send()
        .await
        .expect("Protected request failed");
    
    assert!(protected_response.status().is_success(), "Token validation failed");
    
    env.cleanup().await.expect("Cleanup failed");
}

/// Test 4: Multi-Primal Service Discovery
///
/// Tests complex service discovery scenarios with multiple primals.
/// Verifies:
/// - Dynamic service registration
/// - Capability aggregation
/// - Service health monitoring
#[tokio::test]
#[ignore] // Requires actual service deployment
async fn test_04_multi_primal_service_discovery() {
    let env = TestEnvironmentBuilder::new("multi_primal_discovery")
        .with_timeout(Duration::from_secs(90))
        .with_service(ServiceType::Squirrel)
        .with_service(ServiceType::Songbird)
        .with_service(ServiceType::BearDog)
        .with_service(ServiceType::ToadStool)
        .build()
        .await
        .expect("Failed to build test environment");
    
    let services = env.services.read().await;
    let squirrel = services.values()
        .find(|s| s.service_type == ServiceType::Squirrel)
        .expect("Squirrel not found");
    
    // Discover all capabilities
    let client = reqwest::Client::new();
    let discovery_response = client
        .get(format!("{}/discover/all", squirrel.base_url))
        .send()
        .await
        .expect("Discovery request failed");
    
    assert!(discovery_response.status().is_success(), "Discovery failed");
    
    let discovered: serde_json::Value = discovery_response.json()
        .await
        .expect("Failed to parse discovery response");
    
    // Verify we found multiple services
    let service_count = discovered.get("services")
        .and_then(|s| s.as_array())
        .map(|a| a.len())
        .unwrap_or(0);
    
    assert!(service_count >= 4, "Expected at least 4 services, found {}", service_count);
    
    // Verify each service has capabilities
    for capability in ["orchestration", "security", "storage", "monitoring"] {
        let services_with_cap = assert_capability_discovered(&squirrel.base_url, capability)
            .await
            .expect(&format!("Failed to discover {}", capability));
        
        assert!(!services_with_cap.is_empty(), 
            "No services found with capability: {}", capability);
    }
    
    env.cleanup().await.expect("Cleanup failed");
}

/// Test 5: Error Recovery and Resilience
///
/// Tests error handling and recovery in distributed scenarios.
/// Verifies:
/// - Circuit breaker behavior
/// - Automatic retry logic
/// - Graceful degradation
/// - Error propagation
#[tokio::test]
#[ignore] // Requires actual service deployment
async fn test_05_error_recovery_and_resilience() {
    let env = TestEnvironmentBuilder::new("error_recovery")
        .with_timeout(Duration::from_secs(120))
        .with_service(ServiceType::Squirrel)
        .with_service(ServiceType::Songbird)
        .with_chaos() // Enable chaos testing
        .build()
        .await
        .expect("Failed to build test environment");
    
    let services = env.services.read().await;
    let squirrel = services.values()
        .find(|s| s.service_type == ServiceType::Squirrel)
        .expect("Squirrel not found");
    let songbird = services.values()
        .find(|s| s.service_type == ServiceType::Songbird)
        .expect("Songbird not found");
    
    let client = reqwest::Client::new();
    
    // Test 1: Service temporarily unavailable
    // Simulate Songbird downtime
    env.stop_service(&songbird.name).await.expect("Failed to stop Songbird");
    
    // Squirrel should handle gracefully
    let response = client
        .post(format!("{}/mcp/send", squirrel.base_url))
        .json(&serde_json::json!({
            "target": songbird.base_url,
            "message": messages::ping_message(),
        }))
        .timeout(Duration::from_secs(10))
        .send()
        .await;
    
    // Should get error but not panic
    assert!(response.is_ok(), "Request should complete even if target is down");
    let response = response.unwrap();
    
    // Should indicate service unavailable
    assert!(
        response.status().is_server_error() || response.status().is_client_error(),
        "Should return error status when service is down"
    );
    
    // Restart Songbird
    let songbird_id = env.start_service(ServiceType::Songbird).await
        .expect("Failed to restart Songbird");
    env.wait_for_service(&songbird_id).await
        .expect("Songbird failed to become healthy");
    
    // Test 2: Recovery after restart
    let services = env.services.read().await;
    let songbird_new = services.get(&songbird_id).expect("Songbird not found");
    
    // Should successfully communicate after recovery
    let recovery_response = assert_mcp_exchange_succeeds(
        &squirrel.base_url,
        &songbird_new.base_url,
        messages::ping_message()
    ).await;
    
    assert!(recovery_response.is_ok(), "Should recover after service restart");
    
    env.cleanup().await.expect("Cleanup failed");
}

#[tokio::test]
async fn test_integration_framework_works() {
    // This test verifies the framework itself works
    let env = IntegrationTestEnvironment::new("framework_test").await;
    assert_eq!(env.test_name, "framework_test");
    assert!(env.data_dir.exists());
    
    env.cleanup().await.expect("Cleanup failed");
    assert!(!env.data_dir.exists());
}

/// Test 6: Squirrel to ToadStool Integration
///
/// Tests integration with ToadStool (knowledge graph/semantic store).
/// Verifies semantic query routing and knowledge retrieval.
#[tokio::test]
#[ignore] // Requires actual service deployment
async fn test_06_squirrel_toadstool_knowledge_integration() {
    let env = TestEnvironmentBuilder::new("squirrel_toadstool")
        .with_timeout(Duration::from_secs(60))
        .with_service(ServiceType::Squirrel)
        .with_service(ServiceType::ToadStool)
        .build()
        .await
        .expect("Failed to build test environment");
    
    let client = reqwest::Client::new();
    let services = env.services.read().await;
    let squirrel = services.values()
        .find(|s| s.service_type == ServiceType::Squirrel)
        .expect("Squirrel not found");
    let toadstool = services.values()
        .find(|s| s.service_type == ServiceType::ToadStool)
        .expect("ToadStool not found");
    
    // Test 1: Store knowledge via Squirrel
    let store_response = client
        .post(format!("{}/mcp/send", squirrel.base_url))
        .json(&serde_json::json!({
            "target": toadstool.base_url,
            "message": {
                "type": "knowledge_store",
                "payload": {
                    "entity": "test_entity",
                    "relationships": ["relates_to:other_entity"],
                    "attributes": {"test": "value"}
                }
            }
        }))
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .expect("Failed to send store request");
    
    assert_response_successful(&store_response);
    
    // Test 2: Query knowledge via Squirrel
    let query_response = client
        .post(format!("{}/mcp/send", squirrel.base_url))
        .json(&serde_json::json!({
            "target": toadstool.base_url,
            "message": {
                "type": "knowledge_query",
                "payload": {
                    "query": "test_entity"
                }
            }
        }))
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .expect("Failed to send query request");
    
    assert_response_successful(&query_response);
    let query_data: serde_json::Value = query_response
        .json()
        .await
        .expect("Failed to parse query response");
    
    // Verify knowledge was retrieved
    assert!(
        query_data.get("results").is_some(),
        "Query should return results"
    );
    
    env.cleanup().await.expect("Cleanup failed");
}

/// Test 7: Squirrel to BiomeOS Integration
///
/// Tests integration with BiomeOS (orchestration layer).
/// Verifies deployment, scaling, and orchestration coordination.
#[tokio::test]
#[ignore] // Requires actual service deployment
async fn test_07_squirrel_biomeos_orchestration() {
    let env = TestEnvironmentBuilder::new("squirrel_biomeos")
        .with_timeout(Duration::from_secs(60))
        .with_service(ServiceType::Squirrel)
        .with_service(ServiceType::BiomeOS)
        .build()
        .await
        .expect("Failed to build test environment");
    
    let client = reqwest::Client::new();
    let services = env.services.read().await;
    let squirrel = services.values()
        .find(|s| s.service_type == ServiceType::Squirrel)
        .expect("Squirrel not found");
    let biomeos = services.values()
        .find(|s| s.service_type == ServiceType::BiomeOS)
        .expect("BiomeOS not found");
    
    // Test 1: Register Squirrel with BiomeOS orchestration
    let register_response = client
        .post(format!("{}/mcp/send", squirrel.base_url))
        .json(&serde_json::json!({
            "target": biomeos.base_url,
            "message": {
                "type": "service_register",
                "payload": {
                    "service_name": "squirrel",
                    "capabilities": ["ai_routing", "mcp_coordination"],
                    "health_endpoint": squirrel.health_endpoint
                }
            }
        }))
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .expect("Failed to send register request");
    
    assert_response_successful(&register_response);
    
    // Test 2: Query orchestration status
    let status_response = client
        .get(format!("{}/orchestration/status", biomeos.base_url))
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .expect("Failed to get orchestration status");
    
    assert_response_successful(&status_response);
    let status_data: serde_json::Value = status_response
        .json()
        .await
        .expect("Failed to parse status response");
    
    // Verify Squirrel is registered
    assert!(
        status_data.get("registered_services").is_some(),
        "Should have registered services"
    );
    
    env.cleanup().await.expect("Cleanup failed");
}

/// Test 8: Squirrel with NestGate Security
///
/// Tests integration with NestGate (security gateway).
/// Verifies authentication, authorization, and security policy enforcement.
#[tokio::test]
#[ignore] // Requires actual service deployment
async fn test_08_squirrel_nestgate_security() {
    let env = TestEnvironmentBuilder::new("squirrel_nestgate")
        .with_timeout(Duration::from_secs(60))
        .with_service(ServiceType::Squirrel)
        .with_service(ServiceType::NestGate)
        .with_user(TestUser::admin())
        .build()
        .await
        .expect("Failed to build test environment");
    
    let client = reqwest::Client::new();
    let services = env.services.read().await;
    let squirrel = services.values()
        .find(|s| s.service_type == ServiceType::Squirrel)
        .expect("Squirrel not found");
    let nestgate = services.values()
        .find(|s| s.service_type == ServiceType::NestGate)
        .expect("NestGate not found");
    
    // Test 1: Authentication via NestGate
    let auth_response = client
        .post(format!("{}/auth/login", nestgate.base_url))
        .json(&serde_json::json!({
            "username": "test_admin",
            "password": "test_password"
        }))
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .expect("Failed to send auth request");
    
    assert_response_successful(&auth_response);
    let auth_data: serde_json::Value = auth_response
        .json()
        .await
        .expect("Failed to parse auth response");
    
    let token = auth_data.get("token")
        .and_then(|t| t.as_str())
        .expect("Auth response should contain token");
    
    // Test 2: Authorized request through Squirrel
    let authed_response = client
        .post(format!("{}/mcp/send", squirrel.base_url))
        .header("Authorization", format!("Bearer {}", token))
        .json(&serde_json::json!({
            "target": nestgate.base_url,
            "message": messages::ping_message()
        }))
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .expect("Failed to send authorized request");
    
    assert_response_successful(&authed_response);
    
    // Test 3: Unauthorized request should fail
    let unauthed_response = client
        .post(format!("{}/mcp/send", squirrel.base_url))
        .json(&serde_json::json!({
            "target": nestgate.base_url,
            "message": messages::ping_message()
        }))
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .expect("Failed to send unauthorized request");
    
    assert!(
        unauthed_response.status().is_client_error(),
        "Unauthorized request should be rejected"
    );
    
    env.cleanup().await.expect("Cleanup failed");
}

/// Test 9: Multi-Primal Workflow
///
/// Tests a complex workflow involving multiple primals:
/// Squirrel → NestGate (auth) → ToadStool (knowledge) → BiomeOS (orchestration)
#[tokio::test]
#[ignore] // Requires actual service deployment
async fn test_09_multi_primal_workflow() {
    let env = TestEnvironmentBuilder::new("multi_primal_workflow")
        .with_timeout(Duration::from_secs(120))
        .with_service(ServiceType::Squirrel)
        .with_service(ServiceType::NestGate)
        .with_service(ServiceType::ToadStool)
        .with_service(ServiceType::BiomeOS)
        .with_user(TestUser::admin())
        .build()
        .await
        .expect("Failed to build test environment");
    
    let client = reqwest::Client::new();
    let services = env.services.read().await;
    let squirrel = services.values()
        .find(|s| s.service_type == ServiceType::Squirrel)
        .expect("Squirrel not found");
    let nestgate = services.values()
        .find(|s| s.service_type == ServiceType::NestGate)
        .expect("NestGate not found");
    let toadstool = services.values()
        .find(|s| s.service_type == ServiceType::ToadStool)
        .expect("ToadStool not found");
    let biomeos = services.values()
        .find(|s| s.service_type == ServiceType::BiomeOS)
        .expect("BiomeOS not found");
    
    // Step 1: Authenticate with NestGate
    let auth_response = client
        .post(format!("{}/auth/login", nestgate.base_url))
        .json(&serde_json::json!({
            "username": "test_admin",
            "password": "test_password"
        }))
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .expect("Failed to authenticate");
    
    assert_response_successful(&auth_response);
    let auth_data: serde_json::Value = auth_response
        .json()
        .await
        .expect("Failed to parse auth response");
    let token = auth_data.get("token")
        .and_then(|t| t.as_str())
        .expect("Auth response should contain token");
    
    // Step 2: Query knowledge from ToadStool (authenticated)
    let knowledge_response = client
        .post(format!("{}/mcp/send", squirrel.base_url))
        .header("Authorization", format!("Bearer {}", token))
        .json(&serde_json::json!({
            "target": toadstool.base_url,
            "message": {
                "type": "knowledge_query",
                "payload": {"query": "workflow_test"}
            }
        }))
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .expect("Failed to query knowledge");
    
    assert_response_successful(&knowledge_response);
    
    // Step 3: Trigger orchestration in BiomeOS
    let orchestration_response = client
        .post(format!("{}/mcp/send", squirrel.base_url))
        .header("Authorization", format!("Bearer {}", token))
        .json(&serde_json::json!({
            "target": biomeos.base_url,
            "message": {
                "type": "orchestrate_workflow",
                "payload": {
                    "workflow_id": "multi_primal_test",
                    "steps": ["query_knowledge", "process_data", "return_result"]
                }
            }
        }))
        .timeout(Duration::from_secs(20))
        .send()
        .await
        .expect("Failed to trigger orchestration");
    
    assert_response_successful(&orchestration_response);
    
    // Step 4: Verify all services remain healthy
    for service in services.values() {
        assert_service_healthy(&service.health_endpoint)
            .await
            .expect(&format!("Service {} became unhealthy", service.name));
    }
    
    env.cleanup().await.expect("Cleanup failed");
}

/// Test 10: Load Testing - Concurrent Requests
///
/// Tests Squirrel's ability to handle concurrent requests across multiple primals.
#[tokio::test]
#[ignore] // Requires actual service deployment
async fn test_10_concurrent_load() {
    let env = TestEnvironmentBuilder::new("concurrent_load")
        .with_timeout(Duration::from_secs(120))
        .with_service(ServiceType::Squirrel)
        .with_service(ServiceType::Songbird)
        .with_service(ServiceType::BearDog)
        .build()
        .await
        .expect("Failed to build test environment");
    
    let client = reqwest::Client::new();
    let services = env.services.read().await;
    let squirrel = services.values()
        .find(|s| s.service_type == ServiceType::Squirrel)
        .expect("Squirrel not found");
    
    // Launch 50 concurrent requests
    let mut handles = Vec::new();
    for i in 0..50 {
        let client = client.clone();
        let squirrel_url = squirrel.base_url.clone();
        let songbird_url = services.values()
            .find(|s| s.service_type == ServiceType::Songbird)
            .map(|s| s.base_url.clone())
            .expect("Songbird not found");
        
        let handle = tokio::spawn(async move {
            let response = client
                .post(format!("{}/mcp/send", squirrel_url))
                .json(&serde_json::json!({
                    "target": songbird_url,
                    "message": {
                        "type": "ping",
                        "payload": {"request_id": i}
                    }
                }))
                .timeout(Duration::from_secs(10))
                .send()
                .await;
            
            response.is_ok() && response.unwrap().status().is_success()
        });
        
        handles.push(handle);
    }
    
    // Wait for all requests to complete
    let results: Vec<bool> = futures::future::join_all(handles)
        .await
        .into_iter()
        .filter_map(|r| r.ok())
        .collect();
    
    // At least 95% should succeed
    let success_rate = results.iter().filter(|&&r| r).count() as f64 / results.len() as f64;
    assert!(
        success_rate >= 0.95,
        "Success rate too low: {:.2}%",
        success_rate * 100.0
    );
    
    env.cleanup().await.expect("Cleanup failed");
}

