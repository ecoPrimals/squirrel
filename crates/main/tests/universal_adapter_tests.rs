// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Comprehensive integration tests for UniversalAdapterV2
//!
//! Tests the complete universal adapter flow including protocol negotiation,
//! connection pooling, capability execution, and error handling.

use serial_test::serial;
use squirrel::universal_adapter_v2::{Protocol, UniversalAdapterV2};
use std::sync::Arc;

#[tokio::test]
async fn test_universal_adapter_awaken() {
    let adapter = UniversalAdapterV2::awaken()
        .await
        .expect("Should awaken successfully");

    let identity = adapter.identity();

    // Should have discovered self-identity
    assert!(!identity.identity().name.is_empty());
    assert!(!identity.identity().capabilities.is_empty());
}

#[tokio::test]
async fn test_connect_capability_from_environment() {
    // Set up test environment
    std::env::set_var("TEST_SERVICE_ENDPOINT", "http://localhost:8888");

    let adapter = UniversalAdapterV2::awaken().await.expect("Should awaken");

    let client = adapter
        .connect_capability("test.service")
        .await
        .expect("Should connect to test service");

    assert_eq!(client.endpoint(), "http://localhost:8888");
    assert_eq!(client.service_name(), "test.service-provider");

    std::env::remove_var("TEST_SERVICE_ENDPOINT");
}

#[tokio::test]
async fn test_connect_capability_not_found() {
    std::env::remove_var("NONEXISTENT_SERVICE_ENDPOINT");

    let adapter = UniversalAdapterV2::awaken().await.expect("Should awaken");

    let result = adapter.connect_capability("nonexistent.service").await;

    assert!(result.is_err());
    // Error should indicate capability not found
    match result {
        Err(e) => {
            let err_msg = format!("{:?}", e);
            assert!(err_msg.contains("not found") || err_msg.contains("Capability"));
        }
        Ok(_) => panic!("Expected error for non-existent capability"),
    }
}

#[tokio::test]
async fn test_connection_pooling_reuse() {
    std::env::set_var("POOLING_TEST_ENDPOINT", "http://localhost:7890");

    let adapter = UniversalAdapterV2::awaken().await.expect("Should awaken");

    // First connection
    let client1 = adapter
        .connect_capability("pooling.test")
        .await
        .expect("Should connect first time");

    // Second connection should reuse from pool
    let client2 = adapter
        .connect_capability("pooling.test")
        .await
        .expect("Should connect second time");

    // Both should have same endpoint
    assert_eq!(client1.endpoint(), client2.endpoint());
    assert_eq!(client1.endpoint(), "http://localhost:7890");

    std::env::remove_var("POOLING_TEST_ENDPOINT");
}

#[tokio::test]
async fn test_protocol_detection_https() {
    std::env::set_var("HTTPS_SERVICE_ENDPOINT", "https://secure.example.com:443");

    let adapter = UniversalAdapterV2::awaken().await.expect("Should awaken");

    let client = adapter
        .connect_capability("https.service")
        .await
        .expect("Should connect to HTTPS service");

    // Should detect HTTPS protocol
    assert_eq!(client.protocol(), Protocol::Https);
    assert_eq!(client.endpoint(), "https://secure.example.com:443");

    std::env::remove_var("HTTPS_SERVICE_ENDPOINT");
}

#[tokio::test]
async fn test_protocol_detection_unix_socket() {
    std::env::set_var("UNIX_SERVICE_ENDPOINT", "unix:///var/run/service.sock");

    let adapter = UniversalAdapterV2::awaken().await.expect("Should awaken");

    let client = adapter
        .connect_capability("unix.service")
        .await
        .expect("Should connect to Unix socket service");

    // Should detect JSON-RPC protocol (for Unix sockets)
    assert_eq!(client.protocol(), Protocol::JsonRpc);
    assert_eq!(client.endpoint(), "unix:///var/run/service.sock");

    std::env::remove_var("UNIX_SERVICE_ENDPOINT");
}

#[tokio::test]
async fn test_protocol_detection_localhost() {
    std::env::set_var("LOCAL_SERVICE_ENDPOINT", "http://localhost:9999");

    let adapter = UniversalAdapterV2::awaken().await.expect("Should awaken");

    let client = adapter
        .connect_capability("local.service")
        .await
        .expect("Should connect to localhost service");

    // http:// endpoints default to HTTP protocol
    assert_eq!(client.protocol(), Protocol::Http);
    assert_eq!(client.endpoint(), "http://localhost:9999");

    std::env::remove_var("LOCAL_SERVICE_ENDPOINT");
}

#[tokio::test]
async fn test_multiple_capability_connections() {
    // Set up multiple services
    std::env::set_var("AI_ENDPOINT", "http://localhost:8001");
    std::env::set_var("STORAGE_ENDPOINT", "http://localhost:8002");
    std::env::set_var("COMPUTE_ENDPOINT", "http://localhost:8003");

    let adapter = UniversalAdapterV2::awaken().await.expect("Should awaken");

    // Connect to all three
    let ai = adapter
        .connect_capability("ai")
        .await
        .expect("AI connection");
    let storage = adapter
        .connect_capability("storage")
        .await
        .expect("Storage connection");
    let compute = adapter
        .connect_capability("compute")
        .await
        .expect("Compute connection");

    assert_eq!(ai.endpoint(), "http://localhost:8001");
    assert_eq!(storage.endpoint(), "http://localhost:8002");
    assert_eq!(compute.endpoint(), "http://localhost:8003");

    // Cleanup
    std::env::remove_var("AI_ENDPOINT");
    std::env::remove_var("STORAGE_ENDPOINT");
    std::env::remove_var("COMPUTE_ENDPOINT");
}

#[tokio::test]
#[serial] // Serialize env var tests to prevent pollution
async fn test_adapter_with_complex_capability_names() {
    // Test various capability naming patterns
    std::env::set_var("AI_INFERENCE_ENDPOINT", "http://localhost:8001");
    std::env::set_var("SECURITY_AUTHENTICATION_ENDPOINT", "http://localhost:8002");
    std::env::set_var("STORAGE_OBJECT_STORE_ENDPOINT", "http://localhost:8003");

    let adapter = UniversalAdapterV2::awaken().await.expect("Should awaken");

    // Test dotted names
    let ai = adapter
        .connect_capability("ai.inference")
        .await
        .expect("AI inference");
    assert_eq!(ai.endpoint(), "http://localhost:8001");

    let security = adapter
        .connect_capability("security.authentication")
        .await
        .expect("Security auth");
    assert_eq!(security.endpoint(), "http://localhost:8002");

    let storage = adapter
        .connect_capability("storage.object.store")
        .await
        .expect("Storage");
    assert_eq!(storage.endpoint(), "http://localhost:8003");

    // Cleanup
    std::env::remove_var("AI_INFERENCE_ENDPOINT");
    std::env::remove_var("SECURITY_AUTHENTICATION_ENDPOINT");
    std::env::remove_var("STORAGE_OBJECT_STORE_ENDPOINT");
}

#[tokio::test]
async fn test_adapter_concurrent_connections() {
    std::env::set_var("CONCURRENT1_ENDPOINT", "http://localhost:9001");
    std::env::set_var("CONCURRENT2_ENDPOINT", "http://localhost:9002");
    std::env::set_var("CONCURRENT3_ENDPOINT", "http://localhost:9003");

    let adapter = Arc::new(UniversalAdapterV2::awaken().await.expect("Should awaken"));

    // Connect concurrently
    let adapter1 = adapter.clone();
    let adapter2 = adapter.clone();
    let adapter3 = adapter.clone();

    let (result1, result2, result3) = tokio::join!(
        async move { adapter1.connect_capability("concurrent1").await },
        async move { adapter2.connect_capability("concurrent2").await },
        async move { adapter3.connect_capability("concurrent3").await },
    );

    assert!(result1.is_ok());
    assert!(result2.is_ok());
    assert!(result3.is_ok());

    assert_eq!(result1.unwrap().endpoint(), "http://localhost:9001");
    assert_eq!(result2.unwrap().endpoint(), "http://localhost:9002");
    assert_eq!(result3.unwrap().endpoint(), "http://localhost:9003");

    // Cleanup
    std::env::remove_var("CONCURRENT1_ENDPOINT");
    std::env::remove_var("CONCURRENT2_ENDPOINT");
    std::env::remove_var("CONCURRENT3_ENDPOINT");
}

#[tokio::test]
async fn test_adapter_identity_persistence() {
    let adapter1 = UniversalAdapterV2::awaken().await.expect("Should awaken");
    let identity1 = adapter1.identity().identity().name.clone();

    let adapter2 = UniversalAdapterV2::awaken().await.expect("Should awaken");
    let identity2 = adapter2.identity().identity().name.clone();

    // Both should have same hostname-based identity
    assert_eq!(identity1, identity2);
}

#[tokio::test]
async fn test_connection_metadata() {
    std::env::set_var("METADATA_TEST_ENDPOINT", "http://localhost:7777");

    let adapter = UniversalAdapterV2::awaken().await.expect("Should awaken");

    let client = adapter
        .connect_capability("metadata.test")
        .await
        .expect("Should connect");

    // Verify connection metadata
    assert!(!client.service_name().is_empty());
    assert!(client.endpoint().starts_with("http://"));

    std::env::remove_var("METADATA_TEST_ENDPOINT");
}

#[tokio::test]
async fn test_adapter_with_ipv4_endpoint() {
    std::env::set_var("IPV4_ENDPOINT", "http://192.168.1.100:8080");

    let adapter = UniversalAdapterV2::awaken().await.expect("Should awaken");

    let client = adapter
        .connect_capability("ipv4")
        .await
        .expect("Should connect to IPv4 endpoint");

    assert_eq!(client.endpoint(), "http://192.168.1.100:8080");

    std::env::remove_var("IPV4_ENDPOINT");
}

#[tokio::test]
async fn test_adapter_with_ipv6_endpoint() {
    std::env::set_var("IPV6_ENDPOINT", "http://[::1]:8080");

    let adapter = UniversalAdapterV2::awaken().await.expect("Should awaken");

    let client = adapter
        .connect_capability("ipv6")
        .await
        .expect("Should connect to IPv6 endpoint");

    assert_eq!(client.endpoint(), "http://[::1]:8080");

    std::env::remove_var("IPV6_ENDPOINT");
}
