// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Delegated JWT Client - Production JWT via Capability Discovery (TRUE PRIMAL!)
//!
//! **Evolution**: `BearDog` → Capability Discovery
//! - OLD: Hardcoded "`BearDog`" socket paths (DEV knowledge!)
//! - NEW: Discover "crypto.ed25519.sign" capability at runtime (TRUE PRIMAL!)
//!
//! **Philosophy**: Deploy like an infant - knows nothing, discovers everything!
//! - Squirrel doesn't know which primal provides crypto
//! - Squirrel discovers capability at startup
//! - Could be `BearDog`, could be any crypto primal, could change!

#[cfg(feature = "delegated-jwt")]
use crate::capability_jwt::{
    CapabilityJwtConfig, CapabilityJwtService, JwtClaims as CapabilityJwtClaims,
};
use crate::{AuthError, AuthResult, JwtClaims};
use chrono::{DateTime, Utc};
use tracing::{debug, info};
use universal_constants::identity;
use universal_constants::network::resolve_capability_unix_socket;
use uuid::Uuid;

/// Delegated JWT Client - High-level wrapper for capability-based JWT
///
/// # TRUE PRIMAL Architecture
///
/// - **Production**: Uses discovered crypto capability (Pure Rust!)
/// - **Dev/Testing**: Falls back to local JWT (feature-gated)
/// - **Zero hardcoded primal names!**
///
/// # Example
///
/// ```rust,ignore
/// // Socket path from capability discovery (NOT hardcoded!)
/// let client = DelegatedJwtClient::new_from_env().await?;
/// let token = client.create_token(user_id, username, roles, session_id, expires_at).await?;
/// let claims = client.verify_token(&token).await?;
/// ```
pub struct DelegatedJwtClient {
    #[cfg(feature = "delegated-jwt")]
    capability_service: CapabilityJwtService,

    #[cfg(all(feature = "local-jwt", not(feature = "delegated-jwt")))]
    _local_service: crate::jwt::JwtTokenManager,
}

impl DelegatedJwtClient {
    /// Create new delegated JWT client with custom crypto capability configuration
    ///
    /// # TRUE PRIMAL Mode (delegated-jwt feature)
    ///
    /// Uses capability-based crypto discovery. Socket path should come from
    /// runtime capability discovery, NOT hardcoded!
    ///
    /// # Dev Mode (local-jwt feature)
    ///
    /// Uses local JWT with HMAC-SHA256 (brings `ring` dependency).
    ///
    /// # Errors
    ///
    /// Returns [`AuthError`] if the capability-based JWT service cannot be initialized.
    #[cfg(feature = "delegated-jwt")]
    pub fn new(capability_config: CapabilityJwtConfig) -> AuthResult<Self> {
        info!("🌍 Initializing TRUE PRIMAL JWT client (capability-based discovery!)");

        let capability_service =
            CapabilityJwtService::new(capability_config).map_err(|e| AuthError::Internal {
                message: format!("Failed to initialize capability-based JWT service: {e}"),
            })?;

        Ok(Self { capability_service })
    }

    /// Create new delegated JWT client from environment variables
    ///
    /// **Environment Variables** (set by capability discovery!):
    /// - `CRYPTO_CAPABILITY_SOCKET`: Path to crypto capability socket (from discovery!)
    /// - `JWT_KEY_ID`: Key ID in crypto provider (default: `squirrel-jwt-signing-key`)
    /// - `JWT_EXPIRY_HOURS`: Token expiry in hours (default: 24)
    ///
    /// **IMPORTANT**: These should be set by capability discovery at startup!
    ///
    /// # Errors
    ///
    /// Returns [`AuthError`] if the client cannot be constructed from the resolved configuration.
    #[cfg(feature = "delegated-jwt")]
    pub fn new_from_env() -> AuthResult<Self> {
        use std::env;

        let socket_path =
            resolve_capability_unix_socket("CRYPTO_CAPABILITY_SOCKET", "crypto-provider")
                .to_string_lossy()
                .into_owned();

        let key_id =
            env::var("JWT_KEY_ID").unwrap_or_else(|_| identity::JWT_SIGNING_KEY_ID.to_string());

        let expiry_hours = env::var("JWT_EXPIRY_HOURS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(24);

        info!(
            "🔧 JWT config from env (capability discovery): socket={}, key_id={}, expiry={}h",
            socket_path, key_id, expiry_hours
        );

        let capability_config = CapabilityJwtConfig {
            crypto_config: crate::capability_crypto::CapabilityCryptoConfig {
                endpoint: Some(socket_path),
                discovery_timeout_ms: Some(5000),
            },
            key_id,
            expiry_hours,
        };

        Self::new(capability_config)
    }

    /// Create JWT token (delegates to discovered crypto capability)
    ///
    /// # Arguments
    ///
    /// * `user_id` - User UUID
    /// * `username` - Username
    /// * `roles` - User roles
    /// * `session_id` - Session UUID
    /// * `expires_at` - Token expiration time
    ///
    /// # Returns
    ///
    /// JWT token string in format: `<header>.<claims>.<signature>`
    ///
    /// # Errors
    ///
    /// Returns [`AuthError`] if token creation or delegated signing fails.
    #[cfg(feature = "delegated-jwt")]
    pub async fn create_token(
        &self,
        user_id: Uuid,
        username: String,
        roles: Vec<String>,
        session_id: Uuid,
        expires_at: DateTime<Utc>,
    ) -> AuthResult<String> {
        debug!(
            "Creating JWT token via crypto capability: user={}, session={}",
            username, session_id
        );

        let claims = CapabilityJwtClaims::new(user_id, username, roles, session_id, expires_at);

        self.capability_service.create_token(&claims).await
    }

    /// Verify JWT token (delegates to discovered crypto capability)
    ///
    /// # Arguments
    ///
    /// * `token` - JWT token string to verify
    ///
    /// # Returns
    ///
    /// Verified JWT claims
    ///
    /// # Errors
    ///
    /// Returns [`AuthError`] if the token is invalid, expired, or verification fails.
    #[cfg(feature = "delegated-jwt")]
    pub async fn verify_token(&self, token: &str) -> AuthResult<JwtClaims> {
        debug!(
            "Verifying JWT token via crypto capability: length={}",
            token.len()
        );

        let capability_claims = self.capability_service.verify_token(token).await?;

        // Convert capability claims to our JwtClaims type
        Ok(JwtClaims {
            sub: capability_claims.sub,
            username: capability_claims.username,
            roles: capability_claims.roles,
            session_id: capability_claims.session_id,
            iat: capability_claims.iat,
            exp: capability_claims.exp,
            nbf: capability_claims.nbf,
            iss: capability_claims.iss,
            aud: capability_claims.aud,
            jti: capability_claims.jti,
        })
    }

    /// Extract token from Authorization header
    ///
    /// Expected format: `Bearer <token>`
    ///
    /// # Errors
    ///
    /// Returns [`AuthError`] if the header is missing `Bearer ` or the token is empty.
    #[cfg(feature = "delegated-jwt")]
    pub fn extract_token_from_header<'a>(
        &self,
        authorization_header: &'a str,
    ) -> AuthResult<&'a str> {
        self.capability_service
            .extract_token_from_header(authorization_header)
    }

    // Local JWT methods (dev/testing only, disabled when delegated-jwt is active)

    /// Create new delegated JWT client (local mode)
    #[cfg(all(feature = "local-jwt", not(feature = "delegated-jwt")))]
    pub fn new_local(secret: &[u8]) -> AuthResult<Self> {
        info!("⚠️ Using local JWT (dev mode, brings ring dependency)");

        let local_service = crate::jwt::JwtTokenManager::new(secret);

        Ok(Self {
            _local_service: local_service,
        })
    }

    #[cfg(all(feature = "local-jwt", not(feature = "delegated-jwt")))]
    pub async fn create_token(
        &self,
        user_id: Uuid,
        username: String,
        roles: Vec<String>,
        session_id: Uuid,
        expires_at: DateTime<Utc>,
    ) -> AuthResult<String> {
        debug!("Creating JWT token via local service (dev mode)");

        let claims = crate::jwt::JwtClaims::new(user_id, username, roles, session_id, expires_at);

        self._local_service.create_token(&claims)
    }

    #[cfg(all(feature = "local-jwt", not(feature = "delegated-jwt")))]
    pub async fn verify_token(&self, token: &str) -> AuthResult<JwtClaims> {
        debug!("Verifying JWT token via local service (dev mode)");

        let local_claims = self._local_service.verify_token(token)?;
        Ok(JwtClaims {
            sub: local_claims.sub,
            username: local_claims.username,
            roles: local_claims.roles,
            session_id: local_claims.session_id,
            iat: local_claims.iat,
            exp: local_claims.exp,
            nbf: local_claims.nbf,
            iss: local_claims.iss,
            aud: local_claims.aud,
            jti: local_claims.jti,
        })
    }

    #[cfg(all(feature = "local-jwt", not(feature = "delegated-jwt")))]
    pub fn extract_token_from_header<'a>(
        &self,
        authorization_header: &'a str,
    ) -> AuthResult<&'a str> {
        self._local_service
            .extract_token_from_header(authorization_header)
    }
}

#[cfg(all(test, feature = "delegated-jwt"))]
mod tests {
    use super::*;
    use chrono::Duration;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

    #[test]
    fn test_delegated_client_creation() {
        let config = CapabilityJwtConfig::default();
        let client = DelegatedJwtClient::new(config);

        assert!(client.is_ok());
    }

    #[test]
    fn test_delegated_client_creation_with_custom_config() {
        let config = CapabilityJwtConfig {
            crypto_config: crate::capability_crypto::CapabilityCryptoConfig {
                endpoint: Some("/tmp/nonexistent-delegated.sock".to_string()),
                discovery_timeout_ms: Some(100),
            },
            key_id: "custom-jwt-key".to_string(),
            expiry_hours: 48,
        };
        let client = DelegatedJwtClient::new(config);
        assert!(client.is_ok());
    }

    #[test]
    fn test_delegated_client_from_env() {
        temp_env::with_vars(
            [
                ("CRYPTO_CAPABILITY_SOCKET", Some("/tmp/test-crypto.sock")),
                ("JWT_KEY_ID", Some("test-key-id")),
                ("JWT_EXPIRY_HOURS", Some("12")),
            ],
            || {
                let client = DelegatedJwtClient::new_from_env();
                assert!(client.is_ok());
            },
        );
    }

    #[test]
    fn test_delegated_client_from_env_defaults() {
        temp_env::with_vars(
            [
                ("CRYPTO_CAPABILITY_SOCKET", None::<&str>),
                ("JWT_KEY_ID", None::<&str>),
                ("JWT_EXPIRY_HOURS", None::<&str>),
            ],
            || {
                let client = DelegatedJwtClient::new_from_env();
                assert!(client.is_ok());
            },
        );
    }

    #[test]
    fn test_delegated_client_from_env_invalid_expiry_hours() {
        temp_env::with_vars(
            [
                ("CRYPTO_CAPABILITY_SOCKET", Some("/tmp/test.sock")),
                ("JWT_EXPIRY_HOURS", Some("not-a-number")),
            ],
            || {
                let client = DelegatedJwtClient::new_from_env();
                assert!(client.is_ok());
            },
        );
    }

    #[test]
    fn test_extract_token_from_header() {
        let config = CapabilityJwtConfig::default();
        let client = DelegatedJwtClient::new(config).expect("should succeed");

        let token = client
            .extract_token_from_header("Bearer my-jwt-token-123")
            .expect("should succeed");
        assert_eq!(token, "my-jwt-token-123");

        let err = client.extract_token_from_header("Basic credentials");
        assert!(matches!(err, Err(AuthError::InvalidToken)));

        let err = client.extract_token_from_header("Bearer ");
        assert!(matches!(err, Err(AuthError::InvalidToken)));
    }

    #[tokio::test]
    async fn test_create_and_verify_token_with_mock() {
        use tokio::io::BufReader;
        use tokio::net::UnixListener;

        let dir = tempfile::tempdir().expect("should succeed");
        let socket_path = dir.path().join("delegated-jwt-mock.sock");
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

        let config = CapabilityJwtConfig {
            crypto_config: crate::capability_crypto::CapabilityCryptoConfig {
                endpoint: Some(path_str),
                discovery_timeout_ms: Some(5000),
            },
            key_id: identity::JWT_SIGNING_KEY_ID.to_string(),
            expiry_hours: 24,
        };
        let client = DelegatedJwtClient::new(config).expect("should succeed");

        let user_id = Uuid::new_v4();
        let session_id = Uuid::new_v4();
        let expires_at = Utc::now() + Duration::hours(1);

        let token = client
            .create_token(
                user_id,
                "delegated-user".to_string(),
                vec!["user".to_string(), "editor".to_string()],
                session_id,
                expires_at,
            )
            .await
            .expect("should succeed");

        assert!(token.contains('.'));

        let claims = client.verify_token(&token).await.expect("should succeed");
        let _ = server_handle.await;
        assert_eq!(claims.username, "delegated-user");
        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.roles.len(), 2);
    }

    // Integration tests require crypto capability provider running
    #[tokio::test]
    #[ignore = "requires crypto capability provider running"]
    async fn test_create_and_verify_token_integration() {
        // Socket path from capability discovery (NOT hardcoded!)
        let client = DelegatedJwtClient::new_from_env().expect("should succeed");

        let user_id = Uuid::new_v4();
        let session_id = Uuid::new_v4();
        let expires_at = Utc::now() + Duration::hours(1);

        let token = client
            .create_token(
                user_id,
                "alice".to_string(),
                vec!["user".to_string()],
                session_id,
                expires_at,
            )
            .await
            .expect("should succeed");

        let claims = client.verify_token(&token).await.expect("should succeed");

        assert_eq!(claims.username, "alice");
        assert_eq!(claims.sub, user_id.to_string());
    }
}
