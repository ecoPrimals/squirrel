// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Ecosystem JWT Implementation Using Capability-Based Crypto (Pure Rust!)
//!
//! This module provides JWT token creation and verification by delegating
//! cryptographic operations to a capability-discovered crypto provider.
//!
//! **Architecture**: TRUE PRIMAL + TRUE ecoBin
//! - No `jsonwebtoken` crate → No `ring` → No C dependencies!
//! - Discovers crypto.signing capability at runtime (no hardcoded primal)
//! - Uses Ed25519 (`EdDSA`) instead of HMAC-SHA256
//! - 100% Pure Rust!
//!
//! **JWT Format**:
//! - Algorithm: `EdDSA` (Ed25519)
//! - Header: `{"alg":"EdDSA","typ":"JWT"}`
//! - Claims: Same as before (sub, exp, iat, etc.)
//! - Signature: Ed25519 (64 bytes, base64url-encoded)

use crate::capability_crypto::{CapabilityCryptoConfig, CapabilityCryptoProvider};
use crate::{AuthContext, AuthError};
use anyhow::{Context, Result};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD as BASE64_URL};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};
use universal_constants::identity;
use uuid::Uuid;

/// JWT header for Ed25519 (`EdDSA`)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct JwtHeader {
    /// Algorithm: `EdDSA` (Ed25519)
    alg: String,
    /// Token type: JWT
    typ: String,
}

impl Default for JwtHeader {
    fn default() -> Self {
        Self {
            alg: "EdDSA".to_string(),
            typ: "JWT".to_string(),
        }
    }
}

/// JWT claims (same structure as before for compatibility)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    /// Subject (user ID)
    pub sub: String,
    /// Username
    pub username: String,
    /// User roles
    pub roles: Vec<String>,
    /// Session ID
    pub session_id: String,
    /// Issued at (Unix timestamp)
    pub iat: i64,
    /// Expiration time (Unix timestamp)
    pub exp: i64,
    /// Not before (Unix timestamp)
    pub nbf: i64,
    /// Issuer
    pub iss: String,
    /// Audience
    pub aud: String,
    /// JWT ID (unique identifier)
    pub jti: String,
}

impl JwtClaims {
    /// Create new JWT claims
    #[must_use]
    pub fn new(
        user_id: Uuid,
        username: String,
        roles: Vec<String>,
        session_id: Uuid,
        expires_at: DateTime<Utc>,
    ) -> Self {
        let now = Utc::now();

        Self {
            sub: user_id.to_string(),
            username,
            roles,
            session_id: session_id.to_string(),
            iat: now.timestamp(),
            exp: expires_at.timestamp(),
            nbf: now.timestamp(),
            iss: identity::JWT_ISSUER.to_string(),
            aud: identity::JWT_AUDIENCE.to_string(),
            jti: Uuid::new_v4().to_string(),
        }
    }

    /// Convert JWT claims to `AuthContext`
    ///
    /// # Errors
    ///
    /// Returns [`AuthError`] if claim fields cannot be parsed into IDs or timestamps.
    pub fn to_auth_context(&self) -> Result<AuthContext, AuthError> {
        let user_id = Uuid::parse_str(&self.sub).map_err(|_| AuthError::InvalidToken)?;

        let session_id = Uuid::parse_str(&self.session_id).map_err(|_| AuthError::InvalidToken)?;

        let created_at = DateTime::from_timestamp(self.iat, 0).ok_or(AuthError::InvalidToken)?;

        let expires_at = DateTime::from_timestamp(self.exp, 0).ok_or(AuthError::InvalidToken)?;

        Ok(AuthContext {
            user_id,
            username: self.username.clone(),
            permissions: vec![], // Permissions fetched separately
            session_id,
            expires_at,
            created_at,
            roles: self.roles.clone(),
            auth_provider: crate::types::AuthProvider::Standalone,
        })
    }
}

/// Ecosystem JWT service configuration (capability-based)
#[derive(Debug, Clone)]
pub struct BearDogJwtConfig {
    /// Crypto provider configuration (capability-based)
    pub crypto_config: CapabilityCryptoConfig,

    /// Key ID for JWT signing/verification (optional, provider-specific)
    pub key_id: String,

    /// Token expiry duration in hours (default: 24)
    pub expiry_hours: i64,
}

impl Default for BearDogJwtConfig {
    fn default() -> Self {
        Self {
            crypto_config: CapabilityCryptoConfig::default(),
            key_id: identity::JWT_SIGNING_KEY_ID.to_string(),
            expiry_hours: 24,
        }
    }
}

/// Ecosystem JWT token manager using capability-based crypto (Pure Rust!)
///
/// # Examples
///
/// ```no_run
/// use squirrel_mcp_auth::ecosystem_jwt::{BearDogJwtService, BearDogJwtConfig, JwtClaims};
/// use chrono::{Utc, Duration};
/// use uuid::Uuid;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let config = BearDogJwtConfig::default();
///     let jwt_service = BearDogJwtService::new(config)?;
///
///     let claims = JwtClaims::new(
///         Uuid::new_v4(),
///         "alice".to_string(),
///         vec!["user".to_string()],
///         Uuid::new_v4(),
///         Utc::now() + Duration::hours(24),
///     );
///
///     let token = jwt_service.create_token(&claims).await?;
///     let verified_claims = jwt_service.verify_token(&token).await?;
///
///     assert_eq!(claims.username, verified_claims.username);
///     Ok(())
/// }
/// ```
pub struct BearDogJwtService {
    crypto: CapabilityCryptoProvider,
    key_id: String,
}

impl BearDogJwtService {
    /// Create new capability-based JWT service
    ///
    /// # Errors
    ///
    /// Returns [`anyhow::Error`] if the service cannot be initialized.
    pub fn new(config: BearDogJwtConfig) -> Result<Self> {
        info!(
            "Initializing ecosystem JWT service: key_id={}, endpoint={:?}",
            config.key_id, config.crypto_config.endpoint
        );

        let crypto = CapabilityCryptoProvider::from_config(config.crypto_config);

        Ok(Self {
            crypto,
            key_id: config.key_id,
        })
    }

    /// Create JWT token (delegates signing to security provider primal)
    ///
    /// # Arguments
    /// * `claims` - JWT claims to encode
    ///
    /// # Returns
    /// JWT token string in format: `<header>.<claims>.<signature>`
    ///
    /// # Process
    /// 1. Encode header (`EdDSA`)
    /// 2. Encode claims (base64url)
    /// 3. Create signing input: `<header>.<claims>`
    /// 4. Sign via security provider Ed25519
    /// 5. Encode signature (base64url)
    /// 6. Return complete JWT
    ///
    /// # Errors
    ///
    /// Returns [`AuthError`] if encoding fails or delegated signing returns an error.
    pub async fn create_token(&self, claims: &JwtClaims) -> Result<String, AuthError> {
        debug!(
            "Creating JWT token: user={}, session={}",
            claims.username, claims.session_id
        );

        // 1. Create and encode header
        let header = JwtHeader::default();
        let header_json = serde_json::to_vec(&header).map_err(|e| AuthError::Internal {
            message: format!("Failed to encode JWT header: {e}"),
        })?;
        let header_b64 = BASE64_URL.encode(&header_json);

        // 2. Encode claims
        let claims_json = serde_json::to_vec(&claims).map_err(|e| AuthError::Internal {
            message: format!("Failed to encode JWT claims: {e}"),
        })?;
        let claims_b64 = BASE64_URL.encode(&claims_json);

        // 3. Create signing input
        let signing_input = format!("{header_b64}.{claims_b64}");

        // 4. Sign via discovered crypto provider (Pure Rust!)
        let signature = self
            .crypto
            .clone() // Clone for async mutable access
            .sign_ed25519(signing_input.as_bytes())
            .await
            .context("Failed to sign JWT via capability crypto")
            .map_err(|e| AuthError::Internal {
                message: e.to_string(),
            })?;

        // 5. Encode signature
        let signature_b64 = BASE64_URL.encode(&signature);

        // 6. Construct final JWT
        let token = format!("{signing_input}.{signature_b64}");

        debug!(
            "JWT token created: length={}, header={}, claims={}, sig={}",
            token.len(),
            header_b64.len(),
            claims_b64.len(),
            signature_b64.len()
        );

        Ok(token)
    }

    /// Verify JWT token (delegates verification to security provider primal)
    ///
    /// # Arguments
    /// * `token` - JWT token string to verify
    ///
    /// # Returns
    /// Verified JWT claims
    ///
    /// # Process
    /// 1. Parse token (split on '.')
    /// 2. Decode signature
    /// 3. Verify via security provider Ed25519
    /// 4. Decode and parse claims
    /// 5. Validate expiration
    /// 6. Return claims
    ///
    /// # Errors
    ///
    /// Returns [`AuthError`] if the token is malformed, expired, or verification fails.
    pub async fn verify_token(&self, token: &str) -> Result<JwtClaims, AuthError> {
        debug!("Verifying JWT token: length={}", token.len());

        // 1. Split token into parts
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            error!("Invalid JWT format: expected 3 parts, got {}", parts.len());
            return Err(AuthError::InvalidToken);
        }

        let (header_b64, claims_b64, signature_b64) = (parts[0], parts[1], parts[2]);

        // 2. Decode signature
        let signature = BASE64_URL.decode(signature_b64).map_err(|e| {
            error!("Failed to decode JWT signature: {}", e);
            AuthError::InvalidToken
        })?;

        // 3. Verify signature via discovered crypto provider (Pure Rust!)
        let signing_input = format!("{header_b64}.{claims_b64}");
        let is_valid = self
            .crypto
            .clone() // Clone for async mutable access
            .verify_ed25519_with_key_id(signing_input.as_bytes(), &signature, &self.key_id)
            .await
            .context("Failed to verify JWT signature via capability crypto")
            .map_err(|e| AuthError::Internal {
                message: e.to_string(),
            })?;

        if !is_valid {
            error!("JWT signature verification failed");
            return Err(AuthError::InvalidToken);
        }

        // 4. Decode and parse claims
        let claims_json = BASE64_URL.decode(claims_b64).map_err(|e| {
            error!("Failed to decode JWT claims: {}", e);
            AuthError::InvalidToken
        })?;

        let claims: JwtClaims = serde_json::from_slice(&claims_json).map_err(|e| {
            error!("Failed to parse JWT claims: {}", e);
            AuthError::InvalidToken
        })?;

        // 5. Validate expiration
        let now = Utc::now().timestamp();
        if claims.exp < now {
            error!(
                "JWT token expired: exp={}, now={}, diff={}s",
                claims.exp,
                now,
                now - claims.exp
            );
            return Err(AuthError::TokenExpired);
        }

        // 6. Validate not-before
        if claims.nbf > now {
            error!(
                "JWT token not yet valid: nbf={}, now={}, diff={}s",
                claims.nbf,
                now,
                claims.nbf - now
            );
            return Err(AuthError::InvalidToken);
        }

        debug!(
            "JWT token verified: user={}, session={}, exp={}",
            claims.username, claims.session_id, claims.exp
        );

        Ok(claims)
    }

    /// Extract token from Authorization header
    ///
    /// Expected format: `Bearer <token>`
    ///
    /// # Errors
    ///
    /// Returns [`AuthError`] if the header is missing `Bearer ` or the token is empty.
    pub fn extract_token_from_header<'a>(
        &self,
        authorization_header: &'a str,
    ) -> Result<&'a str, AuthError> {
        if !authorization_header.starts_with("Bearer ") {
            return Err(AuthError::InvalidToken);
        }

        let token = &authorization_header[7..]; // Remove "Bearer " prefix
        if token.is_empty() {
            return Err(AuthError::InvalidToken);
        }

        Ok(token)
    }
}

#[cfg(test)]
mod tests {
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
        let config = BearDogJwtConfig::default();
        assert_eq!(config.key_id, identity::JWT_SIGNING_KEY_ID);
        assert_eq!(config.expiry_hours, 24);
        assert_eq!(config.crypto_config.discovery_timeout_ms, Some(500));
    }

    #[test]
    fn test_extract_token_from_header() {
        let config = BearDogJwtConfig::default();
        let service = BearDogJwtService::new(config).expect("should succeed");

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
        let config = BearDogJwtConfig {
            crypto_config: CapabilityCryptoConfig {
                endpoint: Some("/tmp/nonexistent.sock".to_string()),
                discovery_timeout_ms: Some(100),
            },
            key_id: "custom-key".to_string(),
            expiry_hours: 12,
        };
        let service = BearDogJwtService::new(config).expect("should succeed");
        // Service creation succeeds; crypto calls would fail at runtime
        assert!(service.extract_token_from_header("Bearer x").is_ok());
    }

    #[tokio::test]
    async fn test_verify_token_invalid_format_too_few_parts() {
        let config = BearDogJwtConfig {
            crypto_config: CapabilityCryptoConfig {
                endpoint: Some("/tmp/nonexistent.sock".to_string()),
                discovery_timeout_ms: Some(100),
            },
            key_id: identity::JWT_SIGNING_KEY_ID.to_string(),
            expiry_hours: 24,
        };
        let service = BearDogJwtService::new(config).expect("should succeed");

        let result = service.verify_token("only.two").await;
        assert!(matches!(result, Err(AuthError::InvalidToken)));
    }

    #[tokio::test]
    async fn test_verify_token_invalid_format_too_many_parts() {
        let config = BearDogJwtConfig {
            crypto_config: CapabilityCryptoConfig {
                endpoint: Some("/tmp/nonexistent.sock".to_string()),
                discovery_timeout_ms: Some(100),
            },
            key_id: identity::JWT_SIGNING_KEY_ID.to_string(),
            expiry_hours: 24,
        };
        let service = BearDogJwtService::new(config).expect("should succeed");

        let result = service.verify_token("one.two.three.four").await;
        assert!(matches!(result, Err(AuthError::InvalidToken)));
    }

    #[tokio::test]
    async fn test_verify_token_invalid_signature_base64() {
        let config = BearDogJwtConfig {
            crypto_config: CapabilityCryptoConfig {
                endpoint: Some("/tmp/nonexistent.sock".to_string()),
                discovery_timeout_ms: Some(100),
            },
            key_id: identity::JWT_SIGNING_KEY_ID.to_string(),
            expiry_hours: 24,
        };
        let service = BearDogJwtService::new(config).expect("should succeed");

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

        let config = BearDogJwtConfig {
            crypto_config: CapabilityCryptoConfig {
                endpoint: Some(path_str),
                discovery_timeout_ms: Some(5000),
            },
            key_id: identity::JWT_SIGNING_KEY_ID.to_string(),
            expiry_hours: 24,
        };
        let service = BearDogJwtService::new(config).expect("should succeed");

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

        let config = BearDogJwtConfig {
            crypto_config: CapabilityCryptoConfig {
                endpoint: Some(path_str),
                discovery_timeout_ms: Some(5000),
            },
            key_id: identity::JWT_SIGNING_KEY_ID.to_string(),
            expiry_hours: 24,
        };
        let service = BearDogJwtService::new(config).expect("should succeed");

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

        let config = BearDogJwtConfig {
            crypto_config: CapabilityCryptoConfig {
                endpoint: Some(path_str),
                discovery_timeout_ms: Some(5000),
            },
            key_id: identity::JWT_SIGNING_KEY_ID.to_string(),
            expiry_hours: 24,
        };
        let service = BearDogJwtService::new(config).expect("should succeed");

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

        let config = BearDogJwtConfig {
            crypto_config: CapabilityCryptoConfig {
                endpoint: Some(path_str.clone()),
                discovery_timeout_ms: Some(5000),
            },
            key_id: identity::JWT_SIGNING_KEY_ID.to_string(),
            expiry_hours: 24,
        };
        let service = BearDogJwtService::new(config).expect("should succeed");

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
}
