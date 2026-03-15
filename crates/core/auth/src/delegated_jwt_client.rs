// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Delegated JWT Client - Production JWT via Capability Discovery (TRUE PRIMAL!)
//!
//! **Evolution**: BearDog → Capability Discovery
//! - OLD: Hardcoded "BearDog" socket paths (DEV knowledge!)
//! - NEW: Discover "crypto.ed25519.sign" capability at runtime (TRUE PRIMAL!)
//!
//! **Philosophy**: Deploy like an infant - knows nothing, discovers everything!
//! - Squirrel doesn't know which primal provides crypto
//! - Squirrel discovers capability at startup
//! - Could be BearDog, could be any crypto primal, could change!

#[cfg(feature = "delegated-jwt")]
use crate::capability_jwt::{
    CapabilityJwtConfig, CapabilityJwtService, JwtClaims as CapabilityJwtClaims,
};
use crate::{AuthError, AuthResult, JwtClaims};
use chrono::{DateTime, Utc};
use tracing::{debug, info};
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
    #[cfg(feature = "delegated-jwt")]
    pub fn new(capability_config: CapabilityJwtConfig) -> AuthResult<Self> {
        info!("🌍 Initializing TRUE PRIMAL JWT client (capability-based discovery!)");

        let capability_service =
            CapabilityJwtService::new(capability_config).map_err(|e| AuthError::Internal {
                message: format!("Failed to initialize capability-based JWT service: {}", e),
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
    #[cfg(feature = "delegated-jwt")]
    pub fn new_from_env() -> AuthResult<Self> {
        use std::env;

        let socket_path = env::var("CRYPTO_CAPABILITY_SOCKET")
            .unwrap_or_else(|_| "/var/run/crypto/provider.sock".to_string());

        let key_id =
            env::var("JWT_KEY_ID").unwrap_or_else(|_| "squirrel-jwt-signing-key".to_string());

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

    #[test]
    fn test_delegated_client_creation() {
        let config = CapabilityJwtConfig::default();
        let client = DelegatedJwtClient::new(config);

        assert!(client.is_ok());
    }

    #[test]
    fn test_delegated_client_from_env() {
        // Set test environment variables (as capability discovery would)
        unsafe { std::env::set_var("CRYPTO_CAPABILITY_SOCKET", "/tmp/test-crypto.sock") };
        unsafe { std::env::set_var("JWT_KEY_ID", "test-key-id") };
        unsafe { std::env::set_var("JWT_EXPIRY_HOURS", "12") };

        let client = DelegatedJwtClient::new_from_env();
        assert!(client.is_ok());

        // Cleanup
        unsafe { std::env::remove_var("CRYPTO_CAPABILITY_SOCKET") };
        unsafe { std::env::remove_var("JWT_KEY_ID") };
        unsafe { std::env::remove_var("JWT_EXPIRY_HOURS") };
    }

    // Integration tests require crypto capability provider running
    #[tokio::test]
    #[ignore]
    async fn test_create_and_verify_token_integration() {
        // Socket path from capability discovery (NOT hardcoded!)
        let client = DelegatedJwtClient::new_from_env().unwrap();

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
            .unwrap();

        let claims = client.verify_token(&token).await.unwrap();

        assert_eq!(claims.username, "alice");
        assert_eq!(claims.sub, user_id.to_string());
    }
}
