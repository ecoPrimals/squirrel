// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

#![allow(clippy::unwrap_used, clippy::expect_used)]
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

    assert!(!identity.identity().name.is_empty());
    assert!(!identity.identity().capabilities.is_empty());
}

#[test]
fn test_connect_capability_from_environment() {
    temp_env::with_var(
        "TEST_SERVICE_ENDPOINT",
        Some("http://localhost:8888"),
        || {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let adapter = UniversalAdapterV2::awaken().await.expect("Should awaken");

                let client = adapter
                    .connect_capability("test.service")
                    .await
                    .expect("Should connect to test service");

                assert_eq!(client.endpoint(), "http://localhost:8888");
                assert_eq!(client.service_name(), "test.service-provider");
            })
        },
    );
}

#[test]
fn test_connect_capability_not_found() {
    temp_env::with_var("NONEXISTENT_SERVICE_ENDPOINT", None::<&str>, || {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let adapter = UniversalAdapterV2::awaken().await.expect("Should awaken");

            let result = adapter.connect_capability("nonexistent.service").await;

            assert!(result.is_err());
            match result {
                Err(e) => {
                    let err_msg = format!("{:?}", e);
                    assert!(err_msg.contains("not found") || err_msg.contains("Capability"));
                }
                Ok(_) => panic!("Expected error for non-existent capability"),
            }
        })
    });
}

#[test]
fn test_connection_pooling_reuse() {
    temp_env::with_var(
        "POOLING_TEST_ENDPOINT",
        Some("http://localhost:7890"),
        || {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let adapter = UniversalAdapterV2::awaken().await.expect("Should awaken");

                let client1 = adapter
                    .connect_capability("pooling.test")
                    .await
                    .expect("Should connect first time");

                let client2 = adapter
                    .connect_capability("pooling.test")
                    .await
                    .expect("Should connect second time");

                assert_eq!(client1.endpoint(), client2.endpoint());
                assert_eq!(client1.endpoint(), "http://localhost:7890");
            })
        },
    );
}

#[test]
fn test_protocol_detection_https() {
    temp_env::with_var(
        "HTTPS_SERVICE_ENDPOINT",
        Some("https://secure.example.com:443"),
        || {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let adapter = UniversalAdapterV2::awaken().await.expect("Should awaken");

                let client = adapter
                    .connect_capability("https.service")
                    .await
                    .expect("Should connect to HTTPS service");

                assert_eq!(client.protocol(), Protocol::Https);
                assert_eq!(client.endpoint(), "https://secure.example.com:443");
            })
        },
    );
}

#[test]
fn test_protocol_detection_unix_socket() {
    temp_env::with_var(
        "UNIX_SERVICE_ENDPOINT",
        Some("unix:///var/run/service.sock"),
        || {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let adapter = UniversalAdapterV2::awaken().await.expect("Should awaken");

                let client = adapter
                    .connect_capability("unix.service")
                    .await
                    .expect("Should connect to Unix socket service");

                assert_eq!(client.protocol(), Protocol::JsonRpc);
                assert_eq!(client.endpoint(), "unix:///var/run/service.sock");
            })
        },
    );
}

#[test]
fn test_protocol_detection_localhost() {
    temp_env::with_var(
        "LOCAL_SERVICE_ENDPOINT",
        Some("http://localhost:9999"),
        || {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let adapter = UniversalAdapterV2::awaken().await.expect("Should awaken");

                let client = adapter
                    .connect_capability("local.service")
                    .await
                    .expect("Should connect to localhost service");

                assert_eq!(client.protocol(), Protocol::Http);
                assert_eq!(client.endpoint(), "http://localhost:9999");
            })
        },
    );
}

#[test]
fn test_multiple_capability_connections() {
    temp_env::with_vars(
        [
            ("AI_ENDPOINT", Some("http://localhost:8001")),
            ("STORAGE_ENDPOINT", Some("http://localhost:8002")),
            ("COMPUTE_ENDPOINT", Some("http://localhost:8003")),
        ],
        || {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let adapter = UniversalAdapterV2::awaken().await.expect("Should awaken");

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
            })
        },
    );
}

#[test]
#[serial]
fn test_adapter_with_complex_capability_names() {
    temp_env::with_vars(
        [
            ("AI_INFERENCE_ENDPOINT", Some("http://localhost:8001")),
            (
                "SECURITY_AUTHENTICATION_ENDPOINT",
                Some("http://localhost:8002"),
            ),
            (
                "STORAGE_OBJECT_STORE_ENDPOINT",
                Some("http://localhost:8003"),
            ),
        ],
        || {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let adapter = UniversalAdapterV2::awaken().await.expect("Should awaken");

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
            })
        },
    );
}

#[test]
fn test_adapter_concurrent_connections() {
    temp_env::with_vars(
        [
            ("CONCURRENT1_ENDPOINT", Some("http://localhost:9001")),
            ("CONCURRENT2_ENDPOINT", Some("http://localhost:9002")),
            ("CONCURRENT3_ENDPOINT", Some("http://localhost:9003")),
        ],
        || {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let adapter = Arc::new(UniversalAdapterV2::awaken().await.expect("Should awaken"));

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
            })
        },
    );
}

#[tokio::test]
async fn test_adapter_identity_persistence() {
    let adapter1 = UniversalAdapterV2::awaken().await.expect("Should awaken");
    let identity1 = adapter1.identity().identity().name.clone();

    let adapter2 = UniversalAdapterV2::awaken().await.expect("Should awaken");
    let identity2 = adapter2.identity().identity().name.clone();

    assert_eq!(identity1, identity2);
}

#[test]
fn test_connection_metadata() {
    temp_env::with_var(
        "METADATA_TEST_ENDPOINT",
        Some("http://localhost:7777"),
        || {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let adapter = UniversalAdapterV2::awaken().await.expect("Should awaken");

                let client = adapter
                    .connect_capability("metadata.test")
                    .await
                    .expect("Should connect");

                assert!(!client.service_name().is_empty());
                assert!(client.endpoint().starts_with("http://"));
            })
        },
    );
}

#[test]
fn test_adapter_with_ipv4_endpoint() {
    temp_env::with_var("IPV4_ENDPOINT", Some("http://192.168.1.100:8080"), || {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let adapter = UniversalAdapterV2::awaken().await.expect("Should awaken");

            let client = adapter
                .connect_capability("ipv4")
                .await
                .expect("Should connect to IPv4 endpoint");

            assert_eq!(client.endpoint(), "http://192.168.1.100:8080");
        })
    });
}

#[test]
fn test_adapter_with_ipv6_endpoint() {
    temp_env::with_var("IPV6_ENDPOINT", Some("http://[::1]:8080"), || {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let adapter = UniversalAdapterV2::awaken().await.expect("Should awaken");

            let client = adapter
                .connect_capability("ipv6")
                .await
                .expect("Should connect to IPv6 endpoint");

            assert_eq!(client.endpoint(), "http://[::1]:8080");
        })
    });
}
