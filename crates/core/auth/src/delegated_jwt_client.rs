//! Delegated JWT Client - Production JWT via BearDog Ed25519 (Pure Rust!)
//!
//! TRUE ecoBin Architecture:
//! - Squirrel delegates JWT operations to BearDog (crypto specialist)
//! - Uses Ed25519 (EdDSA) instead of HMAC-SHA256
//! - Zero C dependencies (no `ring`!) → 100% Pure Rust! 🦀
//!
//! This module provides a high-level wrapper around `BearDogJwtService`
//! for easy integration into Squirrel's auth system.

#[cfg(feature = "delegated-jwt")]
use crate::beardog_jwt::{BearDogJwtConfig, BearDogJwtService, JwtClaims as BearDogJwtClaims};
use crate::{AuthError, AuthResult, JwtClaims};
use chrono::{DateTime, Utc};
use tracing::{debug, info};
use uuid::Uuid;

/// Delegated JWT Client - High-level wrapper for BearDog JWT operations
///
/// # TRUE ecoBin Architecture
///
/// - **Production**: Uses BearDog Ed25519 (Pure Rust!)
/// - **Dev/Testing**: Falls back to local JWT (feature-gated)
///
/// # Example
///
/// ```rust,ignore
/// let client = DelegatedJwtClient::new_from_env().await?;
/// let token = client.create_token(user_id, username, roles, session_id, expires_at).await?;
/// let claims = client.verify_token(&token).await?;
/// ```
pub struct DelegatedJwtClient {
    #[cfg(feature = "delegated-jwt")]
    beardog_service: BearDogJwtService,
    
    #[cfg(feature = "local-jwt")]
    _local_service: crate::jwt::JwtTokenManager,
}

impl DelegatedJwtClient {
    /// Create new delegated JWT client with custom BearDog configuration
    ///
    /// # Production Mode (delegated-jwt feature)
    ///
    /// Uses BearDog JWT service with Ed25519 signing.
    ///
    /// # Dev Mode (local-jwt feature)
    ///
    /// Uses local JWT with HMAC-SHA256 (brings `ring` dependency).
    #[cfg(feature = "delegated-jwt")]
    pub fn new(beardog_config: BearDogJwtConfig) -> AuthResult<Self> {
        info!("🌍 Initializing TRUE ecoBin JWT client (BearDog Ed25519, Pure Rust!)");
        
        let beardog_service = BearDogJwtService::new(beardog_config)
            .map_err(|e| AuthError::Internal {
                message: format!("Failed to initialize BearDog JWT service: {}", e),
            })?;
        
        Ok(Self { beardog_service })
    }
    
    /// Create new delegated JWT client from environment variables
    ///
    /// **Environment Variables**:
    /// - `BEARDOG_CRYPTO_SOCKET`: Path to BearDog crypto socket (default: `/var/run/beardog/crypto.sock`)
    /// - `JWT_KEY_ID`: BearDog key ID for JWT signing (default: `squirrel-jwt-signing-key`)
    /// - `JWT_EXPIRY_HOURS`: Token expiry in hours (default: 24)
    #[cfg(feature = "delegated-jwt")]
    pub fn new_from_env() -> AuthResult<Self> {
        use std::env;
        
        let socket_path = env::var("BEARDOG_CRYPTO_SOCKET")
            .unwrap_or_else(|_| "/var/run/beardog/crypto.sock".to_string());
        
        let key_id = env::var("JWT_KEY_ID")
            .unwrap_or_else(|_| "squirrel-jwt-signing-key".to_string());
        
        let expiry_hours = env::var("JWT_EXPIRY_HOURS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(24);
        
        info!(
            "🔧 JWT config from env: socket={}, key_id={}, expiry={}h",
            socket_path, key_id, expiry_hours
        );
        
        let beardog_config = BearDogJwtConfig {
            beardog_config: crate::beardog_client::BearDogClientConfig {
                socket_path,
                timeout_secs: 5,
                max_retries: 3,
                retry_delay_ms: 100,
            },
            key_id,
            expiry_hours,
        };
        
        Self::new(beardog_config)
    }
    
    /// Create JWT token (delegates to BearDog)
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
            "Creating JWT token via BearDog: user={}, session={}",
            username, session_id
        );
        
        let claims = BearDogJwtClaims::new(user_id, username, roles, session_id, expires_at);
        
        self.beardog_service.create_token(&claims).await
    }
    
    /// Verify JWT token (delegates to BearDog)
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
        debug!("Verifying JWT token via BearDog: length={}", token.len());
        
        let beardog_claims = self.beardog_service.verify_token(token).await?;
        
        // Convert BearDog claims to our JwtClaims type
        Ok(JwtClaims {
            sub: beardog_claims.sub,
            username: beardog_claims.username,
            roles: beardog_claims.roles,
            session_id: beardog_claims.session_id,
            iat: beardog_claims.iat,
            exp: beardog_claims.exp,
            nbf: beardog_claims.nbf,
            iss: beardog_claims.iss,
            aud: beardog_claims.aud,
            jti: beardog_claims.jti,
        })
    }
    
    /// Extract token from Authorization header
    ///
    /// Expected format: `Bearer <token>`
    #[cfg(feature = "delegated-jwt")]
    pub fn extract_token_from_header<'a>(&self, authorization_header: &'a str) -> AuthResult<&'a str> {
        self.beardog_service
            .extract_token_from_header(authorization_header)
    }
    
    // Local JWT methods (dev/testing only)
    
    /// Create new delegated JWT client (local mode)
    #[cfg(feature = "local-jwt")]
    pub fn new_local(secret: &[u8]) -> AuthResult<Self> {
        info!("⚠️ Using local JWT (dev mode, brings ring dependency)");
        
        let local_service = crate::jwt::JwtTokenManager::new(secret);
        
        Ok(Self {
            _local_service: local_service,
        })
    }
    
    #[cfg(feature = "local-jwt")]
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
    
    #[cfg(feature = "local-jwt")]
    pub async fn verify_token(&self, token: &str) -> AuthResult<JwtClaims> {
        debug!("Verifying JWT token via local service (dev mode)");
        
        self._local_service.verify_token(token)
    }
    
    #[cfg(feature = "local-jwt")]
    pub fn extract_token_from_header<'a>(&self, authorization_header: &'a str) -> AuthResult<&'a str> {
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
        let config = BearDogJwtConfig::default();
        let client = DelegatedJwtClient::new(config);
        
        assert!(client.is_ok());
    }
    
    #[test]
    fn test_delegated_client_from_env() {
        // Set test environment variables
        std::env::set_var("BEARDOG_CRYPTO_SOCKET", "/tmp/test-beardog.sock");
        std::env::set_var("JWT_KEY_ID", "test-key-id");
        std::env::set_var("JWT_EXPIRY_HOURS", "12");
        
        let client = DelegatedJwtClient::new_from_env();
        assert!(client.is_ok());
        
        // Cleanup
        std::env::remove_var("BEARDOG_CRYPTO_SOCKET");
        std::env::remove_var("JWT_KEY_ID");
        std::env::remove_var("JWT_EXPIRY_HOURS");
    }
    
    // Integration tests require BearDog running
    #[tokio::test]
    #[ignore]
    async fn test_create_and_verify_token_integration() {
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
