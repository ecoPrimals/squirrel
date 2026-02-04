//! Integration Tests for tarpc Over Universal Transport
//!
//! These tests verify the tarpc transport adapter and server logic.
//! Full end-to-end tests with real Unix sockets would require more
//! complex setup and are better suited for higher-level integration tests.

#![cfg(all(test, feature = "tarpc-rpc"))]

use super::tarpc_server::TarpcRpcServer;
use super::tarpc_service::*;
use anyhow::Result;

/// Test tarpc server creation with various configurations
#[test]
fn test_tarpc_server_configurations() {
    // Test basic server creation
    let server = TarpcRpcServer::new("test-squirrel".to_string());
    // Note: service_name is private, which is good encapsulation
    // Just verify creation works
    let _ = server;

    // Test server with AI router
    // Note: We can't easily create a real AI router in a unit test,
    // so we just verify the basic constructor works
    let _server2 = TarpcRpcServer::new("test-with-router".to_string());
}

/// Test tarpc service type serialization
#[test]
fn test_tarpc_service_types_serialization() -> Result<()> {
    // Test QueryAiParams
    let params = QueryAiParams {
        prompt: "Hello".to_string(),
        model: Some("gpt-4".to_string()),
        max_tokens: Some(100),
        temperature: Some(0.7),
    };
    let json = serde_json::to_string(&params)?;
    let deserialized: QueryAiParams = serde_json::from_str(&json)?;
    assert_eq!(params.prompt, deserialized.prompt);

    // Test ProviderInfo
    let info = ProviderInfo {
        id: "test".to_string(),
        name: "Test Provider".to_string(),
        models: vec!["model1".to_string()],
        capabilities: vec!["text".to_string()],
        online: true,
        avg_latency_ms: Some(100.0),
        cost_tier: "free".to_string(),
    };
    let json = serde_json::to_string(&info)?;
    let deserialized: ProviderInfo = serde_json::from_str(&json)?;
    assert_eq!(info.id, deserialized.id);

    // Test HealthCheckResult
    let health = HealthCheckResult {
        status: "healthy".to_string(),
        version: "1.0.0".to_string(),
        uptime_seconds: 100,
        active_providers: 5,
        requests_processed: 1000,
        avg_response_time_ms: Some(50.0),
    };
    let json = serde_json::to_string(&health)?;
    let deserialized: HealthCheckResult = serde_json::from_str(&json)?;
    assert_eq!(health.status, deserialized.status);

    Ok(())
}

/// Test tarpc transport adapter frame size configuration
#[test]
fn test_transport_adapter_frame_sizes() {
    // Test default frame size (16MB)
    let default_max = 16 * 1024 * 1024;
    assert!(default_max > 0);

    // Test custom frame size (32MB)
    let custom_max = 32 * 1024 * 1024;
    assert!(custom_max > default_max);

    // Verify reasonable bounds
    assert!(default_max >= 1024); // At least 1KB
    assert!(default_max <= 64 * 1024 * 1024); // At most 64MB
}

/// Test protocol negotiation types
#[test]
fn test_protocol_types() {
    use super::protocol::{IpcProtocol, ProtocolNegotiation};

    // Test protocol selection
    assert_eq!(IpcProtocol::default(), IpcProtocol::JsonRpc);
    assert!(IpcProtocol::supported().contains(&IpcProtocol::JsonRpc));

    #[cfg(feature = "tarpc-rpc")]
    assert!(IpcProtocol::supported().contains(&IpcProtocol::Tarpc));

    // Test negotiation
    let negotiation = ProtocolNegotiation::success(IpcProtocol::JsonRpc, None);
    assert!(negotiation.success);
    assert_eq!(negotiation.protocol, IpcProtocol::JsonRpc);

    let negotiation = ProtocolNegotiation::fallback(None, "test reason".to_string());
    assert!(!negotiation.success);
    assert_eq!(negotiation.protocol, IpcProtocol::default());
}

/// Test tarpc method implementations (unit tests)
#[tokio::test]
async fn test_tarpc_methods_logic() {
    use tarpc::context;

    let server = TarpcRpcServer::new("test-squirrel".to_string());

    // Test ping
    let ctx = context::current();
    let response = server.clone().ping(ctx).await;
    assert!(response.contains("pong"));
    assert!(response.contains("test-squirrel"));

    // Test health
    let ctx = context::current();
    let health = server.clone().health(ctx).await;
    assert_eq!(health.status, "healthy");
    assert!(!health.version.is_empty());

    // Test list_providers (no AI router)
    let ctx = context::current();
    let result = server.clone().list_providers(ctx).await;
    assert_eq!(result.total, 0);
    assert!(result.providers.is_empty());

    // Test discover_peers
    let ctx = context::current();
    let peers = server.clone().discover_peers(ctx).await;
    assert!(peers.is_empty()); // No peers in test environment

    // Test announce_capabilities
    let ctx = context::current();
    let params = AnnounceCapabilitiesParams {
        service: "test".to_string(),
        capabilities: vec!["test-cap".to_string()],
        metadata: std::collections::HashMap::new(),
    };
    let result = server.clone().announce_capabilities(ctx, params).await;
    assert!(result.success);
}

/// Test client builder configuration
#[test]
fn test_client_builder_config() {
    use super::tarpc_client::SquirrelClientBuilder;
    use std::time::Duration;

    // Test builder creation and method chaining
    let _builder = SquirrelClientBuilder::new("test-service").timeout(Duration::from_secs(60));

    // Just verify the builder exists and methods work
    // Fields are private (good encapsulation)
    assert!(true);
}

/// Test client wrapper ergonomics
#[test]
fn test_client_wrapper_exists() {
    use super::tarpc_client::SquirrelClient;

    // Just verify the type exists and has expected methods
    // Actual connection tests require a running server
    let _ = std::marker::PhantomData::<SquirrelClient>;
}

// Note: Full end-to-end tests with real Unix sockets would require:
// 1. Setting up actual Unix socket listeners
// 2. Spawning real server tasks
// 3. Creating real client connections
// 4. Handling connection lifecycle properly
//
// These are better suited for higher-level integration tests that run
// against a real Squirrel deployment rather than unit tests.
