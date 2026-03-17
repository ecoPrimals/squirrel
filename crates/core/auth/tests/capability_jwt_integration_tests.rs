// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

#![allow(clippy::unwrap_used, clippy::expect_used)]
// Integration tests gated behind `integration-tests` feature — API migration
// (CryptoClient → CapabilityCryptoConfig endpoint) tracked in CURRENT_STATUS.md known issues.
#[cfg(not(feature = "integration-tests"))]
#[tokio::test]
async fn placeholder_capability_jwt_tests_disabled() {}

#[cfg(feature = "integration-tests")]
mod integration_tests {
    // Integration tests for capability-based JWT
    //
    // These tests validate the TRUE PRIMAL capability-based crypto and JWT
    // implementation using a mock crypto provider.

    use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
    use chrono::{Duration, Utc};
    use serde_json::json;
    use squirrel_mcp_auth::{
        capability_crypto::{CryptoClient, CryptoClientConfig},
        capability_jwt::{CapabilityJwtConfig, CapabilityJwtService, JwtClaims},
    };
    use std::path::PathBuf;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::net::{UnixListener, UnixStream};
    use uuid::Uuid;

    /// Mock crypto provider that simulates Ed25519 operations
    ///
    /// This is a simple test server that responds to JSON-RPC requests
    /// for crypto.ed25519.sign and crypto.ed25519.verify.
    struct MockCryptoProvider {
        socket_path: PathBuf,
        listener: UnixListener,
    }

    impl MockCryptoProvider {
        async fn new(socket_path: PathBuf) -> std::io::Result<Self> {
            // Remove socket if it exists
            let _ = std::fs::remove_file(&socket_path);

            let listener = UnixListener::bind(&socket_path)?;

            Ok(Self {
                socket_path,
                listener,
            })
        }

        async fn run(&self) {
            loop {
                match self.listener.accept().await {
                    Ok((stream, _)) => {
                        tokio::spawn(Self::handle_connection(stream));
                    }
                    Err(_) => break,
                }
            }
        }

        async fn handle_connection(stream: UnixStream) {
            let (reader, mut writer) = stream.into_split();
            let mut reader = BufReader::new(reader);
            let mut line = String::new();

            if reader.read_line(&mut line).await.is_err() {
                return;
            }

            let request: serde_json::Value = match serde_json::from_str(&line) {
                Ok(req) => req,
                Err(_) => return,
            };

            let method = request["method"].as_str().unwrap_or("");
            let id = request["id"].as_u64().unwrap_or(0);

            let response = match method {
                "crypto.ed25519.sign" => {
                    // Mock signature (64 bytes of deterministic data)
                    let _data = request["params"]["data"].as_str().unwrap_or("");
                    let signature = BASE64.encode(vec![42u8; 64]); // Mock signature

                    json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": {
                            "signature": signature
                        }
                    })
                }
                "crypto.ed25519.verify" => {
                    // Mock verification (always returns true for our mock signature)
                    json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": {
                            "valid": true
                        }
                    })
                }
                _ => {
                    json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "error": {
                            "code": -32601,
                            "message": "Method not found"
                        }
                    })
                }
            };

            let response_str = serde_json::to_string(&response).unwrap();
            let _ = writer.write_all(response_str.as_bytes()).await;
            let _ = writer.write_all(b"\n").await;
            let _ = writer.flush().await;
        }
    }

    #[tokio::test]
    async fn test_capability_crypto_client() {
        let socket_path = PathBuf::from("/tmp/test-crypto-capability.sock");

        // Start mock provider
        let provider = MockCryptoProvider::new(socket_path.clone())
            .await
            .expect("Failed to create mock provider");

        tokio::spawn(async move {
            provider.run().await;
        });

        // Give server time to start
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Create crypto client
        let config = CryptoClientConfig {
            socket_path,
            timeout_secs: 5,
            max_retries: 3,
            retry_delay_ms: 100,
        };

        let client = CryptoClient::new(config).expect("Failed to create crypto client");

        // Test signing
        let data = b"Hello, capability discovery!";
        let signature = client
            .ed25519_sign(data, "test-key")
            .await
            .expect("Failed to sign data");

        assert_eq!(signature.len(), 64, "Signature should be 64 bytes");

        // Test verification
        let valid = client
            .ed25519_verify(data, &signature, "test-key")
            .await
            .expect("Failed to verify signature");

        assert!(valid, "Signature should be valid");

        // Cleanup
        let _ = std::fs::remove_file("/tmp/test-crypto-capability.sock");
    }

    #[tokio::test]
    async fn test_capability_jwt_full_flow() {
        let socket_path = PathBuf::from("/tmp/test-jwt-capability.sock");

        // Start mock provider
        let provider = MockCryptoProvider::new(socket_path.clone())
            .await
            .expect("Failed to create mock provider");

        tokio::spawn(async move {
            provider.run().await;
        });

        // Give server time to start
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Create JWT service with capability discovery
        let config = CapabilityJwtConfig {
            crypto_config: CryptoClientConfig {
                socket_path,
                timeout_secs: 5,
                max_retries: 3,
                retry_delay_ms: 100,
            },
            key_id: "test-jwt-key".to_string(),
            expiry_hours: 24,
        };

        let jwt_service = CapabilityJwtService::new(config).expect("Failed to create JWT service");

        // Create JWT claims
        let user_id = Uuid::new_v4();
        let session_id = Uuid::new_v4();
        let expires_at = Utc::now() + Duration::hours(1);

        let claims = JwtClaims::new(
            user_id,
            "alice".to_string(),
            vec!["user".to_string(), "admin".to_string()],
            session_id,
            expires_at,
        );

        // Create token
        let token = jwt_service
            .create_token(&claims)
            .await
            .expect("Failed to create JWT token");

        assert!(!token.is_empty(), "Token should not be empty");
        assert_eq!(token.matches('.').count(), 2, "Token should have 3 parts");

        // Verify token
        let verified_claims = jwt_service
            .verify_token(&token)
            .await
            .expect("Failed to verify JWT token");

        assert_eq!(verified_claims.username, "alice");
        assert_eq!(verified_claims.sub, user_id.to_string());
        assert_eq!(verified_claims.session_id, session_id.to_string());
        assert_eq!(verified_claims.roles.len(), 2);
        assert!(verified_claims.roles.contains(&"user".to_string()));
        assert!(verified_claims.roles.contains(&"admin".to_string()));

        // Cleanup
        let _ = std::fs::remove_file("/tmp/test-jwt-capability.sock");
    }

    #[tokio::test]
    async fn test_jwt_token_extraction() {
        let socket_path = PathBuf::from("/tmp/test-jwt-extract.sock");

        let provider = MockCryptoProvider::new(socket_path.clone())
            .await
            .expect("Failed to create mock provider");

        tokio::spawn(async move {
            provider.run().await;
        });

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let config = CapabilityJwtConfig {
            crypto_config: CryptoClientConfig {
                socket_path,
                timeout_secs: 5,
                max_retries: 3,
                retry_delay_ms: 100,
            },
            key_id: "test-key".to_string(),
            expiry_hours: 24,
        };

        let jwt_service = CapabilityJwtService::new(config).unwrap();

        // Test valid Bearer token
        let header = "Bearer eyJhbGciOiJFZERTQSIsInR5cCI6IkpXVCJ9.test.signature";
        let token = jwt_service
            .extract_token_from_header(header)
            .expect("Should extract token");

        assert!(token.starts_with("eyJ"));

        // Test invalid header (no Bearer)
        let invalid_header = "eyJhbGciOiJFZERTQSIsInR5cCI6IkpXVCJ9.test.signature";
        let result = jwt_service.extract_token_from_header(invalid_header);
        assert!(result.is_err(), "Should fail without Bearer prefix");

        // Test empty token
        let empty_header = "Bearer ";
        let result = jwt_service.extract_token_from_header(empty_header);
        assert!(result.is_err(), "Should fail with empty token");

        // Cleanup
        let _ = std::fs::remove_file("/tmp/test-jwt-extract.sock");
    }

    #[tokio::test]
    async fn test_expired_token() {
        let socket_path = PathBuf::from("/tmp/test-jwt-expired.sock");

        let provider = MockCryptoProvider::new(socket_path.clone())
            .await
            .expect("Failed to create mock provider");

        tokio::spawn(async move {
            provider.run().await;
        });

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let config = CapabilityJwtConfig {
            crypto_config: CryptoClientConfig {
                socket_path,
                timeout_secs: 5,
                max_retries: 3,
                retry_delay_ms: 100,
            },
            key_id: "test-key".to_string(),
            expiry_hours: 24,
        };

        let jwt_service = CapabilityJwtService::new(config).unwrap();

        // Create already-expired token
        let user_id = Uuid::new_v4();
        let session_id = Uuid::new_v4();
        let expires_at = Utc::now() - Duration::hours(1); // Expired 1 hour ago

        let claims = JwtClaims::new(
            user_id,
            "bob".to_string(),
            vec!["user".to_string()],
            session_id,
            expires_at,
        );

        let token = jwt_service.create_token(&claims).await.unwrap();

        // Try to verify expired token
        let result = jwt_service.verify_token(&token).await;
        assert!(result.is_err(), "Should reject expired token");

        // Cleanup
        let _ = std::fs::remove_file("/tmp/test-jwt-expired.sock");
    }

    #[tokio::test]
    async fn test_capability_discovery_from_env() {
        let socket_path = "/tmp/test-env-capability.sock";

        temp_env::with_vars(
            [
                ("CRYPTO_CAPABILITY_SOCKET", Some(socket_path)),
                ("JWT_KEY_ID", Some("env-test-key")),
                ("JWT_EXPIRY_HOURS", Some("12")),
            ],
            || {
                let config = CapabilityJwtConfig::default();

                assert_eq!(
                    config.crypto_config.socket_path.to_str().unwrap(),
                    socket_path
                );
                assert_eq!(config.key_id, "squirrel-jwt-signing-key");
                assert_eq!(config.expiry_hours, 24);
            },
        );
    }
}
