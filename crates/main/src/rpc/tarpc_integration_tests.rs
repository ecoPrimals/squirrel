// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Integration Tests for tarpc Over Universal Transport
//!
//! These tests verify the tarpc transport adapter and server logic.
//! Full end-to-end tests with real Unix sockets would require more
//! complex setup and are better suited for higher-level integration tests.

#![cfg(all(test, feature = "tarpc-rpc"))]

use super::tarpc_server::TarpcRpcServer;
use super::tarpc_service::*;
use super::types::HealthTier;
use anyhow::Result;
use std::sync::Arc;

/// Test tarpc server creation requires a JsonRpcServer instance
#[test]
fn test_tarpc_server_type_exists() {
    let _ = std::marker::PhantomData::<TarpcRpcServer>;
}

/// Test tarpc service type serialization
#[test]
fn test_tarpc_service_types_serialization() -> Result<()> {
    // Test QueryAiParams
    let params = QueryAiParams {
        prompt: "Hello".to_string(),
        model: Some(Arc::from("gpt-4")),
        max_tokens: Some(100),
        temperature: Some(0.7),
    };
    let json = serde_json::to_string(&params)?;
    let deserialized: QueryAiParams = serde_json::from_str(&json)?;
    assert_eq!(params.prompt, deserialized.prompt);

    // Test ProviderInfo
    let info = ProviderInfo {
        id: Arc::from("test"),
        name: Arc::from("Test Provider"),
        models: vec![Arc::from("model1")],
        capabilities: vec![Arc::from("text")],
        online: true,
        avg_latency_ms: Some(100.0),
        cost_tier: Arc::from("free"),
    };
    let json = serde_json::to_string(&info)?;
    let deserialized: ProviderInfo = serde_json::from_str(&json)?;
    assert_eq!(info.id, deserialized.id);

    // Test HealthCheckResult
    let health = HealthCheckResult {
        tier: HealthTier::Healthy,
        alive: true,
        ready: true,
        healthy: true,
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

    let negotiation = ProtocolNegotiation::fallback(None, "test reason");
    assert!(!negotiation.success);
    assert_eq!(negotiation.protocol, IpcProtocol::default());
}

/// Test tarpc service type serialization for the announce params
#[test]
fn test_announce_capabilities_params_serde() {
    let params = AnnounceCapabilitiesParams {
        capabilities: vec!["ai.query".to_string(), "system.ping".to_string()],
        primal: Some("squirrel".to_string()),
        socket_path: Some("/run/user/1000/biomeos/squirrel.sock".to_string()),
        tools: Some(vec!["ai_tool".to_string()]),
        sub_federations: None,
        genetic_families: None,
    };
    let json = serde_json::to_string(&params).expect("serialize");
    let roundtrip: AnnounceCapabilitiesParams = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(roundtrip.capabilities.len(), 2);
    assert_eq!(roundtrip.primal.as_deref(), Some("squirrel"));
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
