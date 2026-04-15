// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

use super::*;
use chrono::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;

#[test]
fn test_jwt_claims_creation() {
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

    assert_eq!(claims.sub, user_id.to_string());
    assert_eq!(claims.username, "alice");
    assert_eq!(claims.roles.len(), 2);
    assert_eq!(claims.session_id, session_id.to_string());
    assert_eq!(claims.iss, identity::JWT_ISSUER);
    assert_eq!(claims.aud, identity::JWT_AUDIENCE);
}

#[test]
fn test_jwt_claims_to_auth_context() {
    let user_id = Uuid::new_v4();
    let session_id = Uuid::new_v4();
    let expires_at = Utc::now() + Duration::hours(1);

    let claims = JwtClaims::new(
        user_id,
        "alice".to_string(),
        vec!["user".to_string()],
        session_id,
        expires_at,
    );

    let context = claims.to_auth_context().expect("should succeed");

    assert_eq!(context.user_id, user_id);
    assert_eq!(context.username, "alice");
    assert_eq!(context.session_id, session_id);
    assert_eq!(context.roles.len(), 1);
}

#[test]
fn test_jwt_header_default() {
    let header = JwtHeader::default();
    assert_eq!(header.alg, "EdDSA");
    assert_eq!(header.typ, "JWT");
}

#[test]
fn test_beardog_jwt_config_default() {
    let config = SecurityProviderJwtConfig::default();
    assert_eq!(config.key_id, identity::JWT_SIGNING_KEY_ID);
    assert_eq!(config.expiry_hours, 24);
    assert_eq!(config.crypto_config.discovery_timeout_ms, Some(500));
}

#[test]
fn test_extract_token_from_header() {
    let config = SecurityProviderJwtConfig::default();
    let service = SecurityProviderJwtService::new(config).expect("should succeed");

    // Valid header
    let header = "Bearer abc123def456";
    let token = service
        .extract_token_from_header(header)
        .expect("should succeed");
    assert_eq!(token, "abc123def456");

    // Invalid header (no Bearer prefix)
    let invalid_header = "abc123def456";
    let result = service.extract_token_from_header(invalid_header);
    assert!(matches!(result, Err(AuthError::InvalidToken)));

    // Invalid header (empty token)
    let empty_header = "Bearer ";
    let result = service.extract_token_from_header(empty_header);
    assert!(matches!(result, Err(AuthError::InvalidToken)));
}

#[test]
fn test_jwt_claims_to_auth_context_invalid_sub() {
    let user_id = Uuid::new_v4();
    let session_id = Uuid::new_v4();
    let expires_at = Utc::now() + Duration::hours(1);

    let mut claims = JwtClaims::new(
        user_id,
        "alice".to_string(),
        vec!["user".to_string()],
        session_id,
        expires_at,
    );
    claims.sub = "not-a-valid-uuid".to_string();

    let result = claims.to_auth_context();
    assert!(matches!(result, Err(AuthError::InvalidToken)));
}

#[test]
fn test_jwt_claims_to_auth_context_invalid_session_id() {
    let user_id = Uuid::new_v4();
    let session_id = Uuid::new_v4();
    let expires_at = Utc::now() + Duration::hours(1);

    let mut claims = JwtClaims::new(
        user_id,
        "alice".to_string(),
        vec!["user".to_string()],
        session_id,
        expires_at,
    );
    claims.session_id = "invalid-session-uuid".to_string();

    let result = claims.to_auth_context();
    assert!(matches!(result, Err(AuthError::InvalidToken)));
}

#[test]
fn test_beardog_jwt_service_new_with_custom_config() {
    let config = SecurityProviderJwtConfig {
        crypto_config: CapabilityCryptoConfig {
            endpoint: Some("/tmp/nonexistent.sock".to_string()),
            discovery_timeout_ms: Some(100),
        },
        key_id: "custom-key".to_string(),
        expiry_hours: 12,
    };
    let service = SecurityProviderJwtService::new(config).expect("should succeed");
    // Service creation succeeds; crypto calls would fail at runtime
    assert!(service.extract_token_from_header("Bearer x").is_ok());
}

#[tokio::test]
async fn test_verify_token_invalid_format_too_few_parts() {
    let config = SecurityProviderJwtConfig {
        crypto_config: CapabilityCryptoConfig {
            endpoint: Some("/tmp/nonexistent.sock".to_string()),
            discovery_timeout_ms: Some(100),
        },
        key_id: identity::JWT_SIGNING_KEY_ID.to_string(),
        expiry_hours: 24,
    };
    let service = SecurityProviderJwtService::new(config).expect("should succeed");

    let result = service.verify_token("only.two").await;
    assert!(matches!(result, Err(AuthError::InvalidToken)));
}

#[tokio::test]
async fn test_verify_token_invalid_format_too_many_parts() {
    let config = SecurityProviderJwtConfig {
        crypto_config: CapabilityCryptoConfig {
            endpoint: Some("/tmp/nonexistent.sock".to_string()),
            discovery_timeout_ms: Some(100),
        },
        key_id: identity::JWT_SIGNING_KEY_ID.to_string(),
        expiry_hours: 24,
    };
    let service = SecurityProviderJwtService::new(config).expect("should succeed");

    let result = service.verify_token("one.two.three.four").await;
    assert!(matches!(result, Err(AuthError::InvalidToken)));
}

#[tokio::test]
async fn test_verify_token_invalid_signature_base64() {
    let config = SecurityProviderJwtConfig {
        crypto_config: CapabilityCryptoConfig {
            endpoint: Some("/tmp/nonexistent.sock".to_string()),
            discovery_timeout_ms: Some(100),
        },
        key_id: identity::JWT_SIGNING_KEY_ID.to_string(),
        expiry_hours: 24,
    };
    let service = SecurityProviderJwtService::new(config).expect("should succeed");

    let header_b64 = BASE64_URL.encode(r#"{"alg":"EdDSA","typ":"JWT"}"#);
    let claims_b64 =
        BASE64_URL.encode(r#"{"sub":"00000000-0000-0000-0000-000000000001","exp":9999999999}"#);
    let invalid_sig = "!!!invalid-base64!!!";
    let token = format!("{header_b64}.{claims_b64}.{invalid_sig}");

    let result = service.verify_token(&token).await;
    assert!(matches!(result, Err(AuthError::InvalidToken)));
}

#[tokio::test]
async fn test_verify_token_expired() {
    let dir = tempfile::tempdir().expect("should succeed");
    let socket_path = dir.path().join("ecosystem-jwt-expired.sock");
    let path_str = socket_path.to_string_lossy().to_string();

    let listener = UnixListener::bind(&socket_path).expect("should succeed");

    let server_handle = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.expect("should succeed");
        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        reader.read_line(&mut line).await.expect("should succeed");
        let req: serde_json::Value = serde_json::from_str(&line).expect("should succeed");
        assert_eq!(req["method"], "crypto.verify");
        let response = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": { "valid": true }
        });
        let mut stream = reader.into_inner();
        stream
            .write_all(response.to_string().as_bytes())
            .await
            .expect("should succeed");
        stream.write_all(b"\n").await.expect("should succeed");
    });

    let config = SecurityProviderJwtConfig {
        crypto_config: CapabilityCryptoConfig {
            endpoint: Some(path_str),
            discovery_timeout_ms: Some(5000),
        },
        key_id: identity::JWT_SIGNING_KEY_ID.to_string(),
        expiry_hours: 24,
    };
    let service = SecurityProviderJwtService::new(config).expect("should succeed");

    let header_b64 = BASE64_URL.encode(r#"{"alg":"EdDSA","typ":"JWT"}"#);
    let claims = serde_json::json!({
        "sub": "550e8400-e29b-41d4-a716-446655440000",
        "username": "alice",
        "roles": ["user"],
        "session_id": "550e8400-e29b-41d4-a716-446655440001",
        "iat": 0,
        "exp": 1,
        "nbf": 0,
        "iss": identity::JWT_ISSUER,
        "aud": identity::JWT_AUDIENCE,
        "jti": "550e8400-e29b-41d4-a716-446655440002"
    });
    let claims_b64 = BASE64_URL.encode(claims.to_string());
    let sig = BASE64_URL.encode([0u8; 64]);
    let token = format!("{header_b64}.{claims_b64}.{sig}");

    let verify_result = service.verify_token(&token).await;
    let _ = server_handle.await;
    assert!(matches!(verify_result, Err(AuthError::TokenExpired)));
}

#[tokio::test]
async fn test_verify_token_nbf_future() {
    let dir = tempfile::tempdir().expect("should succeed");
    let socket_path = dir.path().join("ecosystem-jwt-nbf.sock");
    let path_str = socket_path.to_string_lossy().to_string();

    let listener = UnixListener::bind(&socket_path).expect("should succeed");

    let server_handle = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.expect("should succeed");
        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        reader.read_line(&mut line).await.expect("should succeed");
        let req: serde_json::Value = serde_json::from_str(&line).expect("should succeed");
        assert_eq!(req["method"], "crypto.verify");
        let response = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": { "valid": true }
        });
        let mut stream = reader.into_inner();
        stream
            .write_all(response.to_string().as_bytes())
            .await
            .expect("should succeed");
        stream.write_all(b"\n").await.expect("should succeed");
    });

    let config = SecurityProviderJwtConfig {
        crypto_config: CapabilityCryptoConfig {
            endpoint: Some(path_str),
            discovery_timeout_ms: Some(5000),
        },
        key_id: identity::JWT_SIGNING_KEY_ID.to_string(),
        expiry_hours: 24,
    };
    let service = SecurityProviderJwtService::new(config).expect("should succeed");

    let header_b64 = BASE64_URL.encode(r#"{"alg":"EdDSA","typ":"JWT"}"#);
    let future_nbf = Utc::now().timestamp() + 3600;
    let future_exp = future_nbf + 86400;
    let claims = serde_json::json!({
        "sub": "550e8400-e29b-41d4-a716-446655440000",
        "username": "alice",
        "roles": ["user"],
        "session_id": "550e8400-e29b-41d4-a716-446655440001",
        "iat": 0,
        "exp": future_exp,
        "nbf": future_nbf,
        "iss": identity::JWT_ISSUER,
        "aud": identity::JWT_AUDIENCE,
        "jti": "550e8400-e29b-41d4-a716-446655440002"
    });
    let claims_b64 = BASE64_URL.encode(claims.to_string());
    let sig = BASE64_URL.encode([0u8; 64]);
    let token = format!("{header_b64}.{claims_b64}.{sig}");

    let verify_result = service.verify_token(&token).await;
    let _ = server_handle.await;
    assert!(matches!(verify_result, Err(AuthError::InvalidToken)));
}

#[tokio::test]
async fn test_verify_token_invalid_claims_json() {
    let dir = tempfile::tempdir().expect("should succeed");
    let socket_path = dir.path().join("ecosystem-jwt-bad-json.sock");
    let path_str = socket_path.to_string_lossy().to_string();

    let listener = UnixListener::bind(&socket_path).expect("should succeed");

    let server_handle = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.expect("should succeed");
        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        reader.read_line(&mut line).await.expect("should succeed");
        let response = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": { "valid": true }
        });
        let mut stream = reader.into_inner();
        stream
            .write_all(response.to_string().as_bytes())
            .await
            .expect("should succeed");
        stream.write_all(b"\n").await.expect("should succeed");
    });

    let config = SecurityProviderJwtConfig {
        crypto_config: CapabilityCryptoConfig {
            endpoint: Some(path_str),
            discovery_timeout_ms: Some(5000),
        },
        key_id: identity::JWT_SIGNING_KEY_ID.to_string(),
        expiry_hours: 24,
    };
    let service = SecurityProviderJwtService::new(config).expect("should succeed");

    let header_b64 = BASE64_URL.encode(r#"{"alg":"EdDSA","typ":"JWT"}"#);
    let claims_b64 = BASE64_URL.encode("{ invalid json }");
    let sig = BASE64_URL.encode([0u8; 64]);
    let token = format!("{header_b64}.{claims_b64}.{sig}");

    let verify_result = service.verify_token(&token).await;
    let _ = server_handle.await;
    assert!(matches!(verify_result, Err(AuthError::InvalidToken)));
}

#[tokio::test]
async fn test_create_and_verify_token_roundtrip() {
    let dir = tempfile::tempdir().expect("should succeed");
    let socket_path = dir.path().join("ecosystem-jwt-roundtrip.sock");
    let path_str = socket_path.to_string_lossy().to_string();

    let listener = UnixListener::bind(&socket_path).expect("should succeed");

    let server_handle = tokio::spawn(async move {
        let (stream1, _) = listener.accept().await.expect("should succeed");
        let mut reader = BufReader::new(stream1);
        let mut line = String::new();
        reader.read_line(&mut line).await.expect("should succeed");
        let req: serde_json::Value = serde_json::from_str(&line).expect("should succeed");
        assert_eq!(req["method"], "crypto.sign");
        let sig_b64 =
            base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &[0u8; 64][..]);
        let response = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": { "signature": sig_b64 }
        });
        let mut stream = reader.into_inner();
        stream
            .write_all(response.to_string().as_bytes())
            .await
            .expect("should succeed");
        stream.write_all(b"\n").await.expect("should succeed");

        let (stream2, _) = listener.accept().await.expect("should succeed");
        let mut reader2 = BufReader::new(stream2);
        let mut line2 = String::new();
        reader2.read_line(&mut line2).await.expect("should succeed");
        let req2: serde_json::Value = serde_json::from_str(&line2).expect("should succeed");
        assert_eq!(req2["method"], "crypto.verify");
        let response2 = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": { "valid": true }
        });
        let mut stream2 = reader2.into_inner();
        stream2
            .write_all(response2.to_string().as_bytes())
            .await
            .expect("should succeed");
        stream2.write_all(b"\n").await.expect("should succeed");
    });

    let config = SecurityProviderJwtConfig {
        crypto_config: CapabilityCryptoConfig {
            endpoint: Some(path_str.clone()),
            discovery_timeout_ms: Some(5000),
        },
        key_id: identity::JWT_SIGNING_KEY_ID.to_string(),
        expiry_hours: 24,
    };
    let service = SecurityProviderJwtService::new(config).expect("should succeed");

    let claims = JwtClaims::new(
        Uuid::new_v4(),
        "alice".to_string(),
        vec!["user".to_string(), "admin".to_string()],
        Uuid::new_v4(),
        Utc::now() + Duration::hours(1),
    );

    let token = service.create_token(&claims).await.expect("should succeed");
    assert!(token.contains('.'));
    assert_eq!(token.split('.').count(), 3);

    let verified = service.verify_token(&token).await.expect("should succeed");
    let _ = server_handle.await;
    assert_eq!(verified.username, "alice");
    assert_eq!(verified.roles.len(), 2);
}

// Integration tests (with security provider) are in tests/integration/
