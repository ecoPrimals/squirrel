// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Capability-Based JWT Service (TRUE PRIMAL!)
//!
//! **Evolution**: BearDog JWT → Capability JWT
//! - OLD: `BearDogJwtService` (hardcoded primal name!)
//! - NEW: `CapabilityJwtService` (discovers crypto at runtime!)
//!
//! **Philosophy**: Deploy like an infant - knows nothing, discovers everything!
//! - Squirrel doesn't know which primal provides crypto
//! - Squirrel discovers "crypto.ed25519.sign" capability
//! - Could be BearDog, could be any crypto primal, could be multiple!

use crate::capability_crypto::{CapabilityCryptoConfig, CapabilityCryptoProvider};
use crate::{AuthContext, AuthError};
use anyhow::{Context, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD as BASE64_URL, Engine};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};
use uuid::Uuid;

/// JWT header for Ed25519 (EdDSA)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct JwtHeader {
    /// Algorithm: EdDSA (Ed25519)
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

/// JWT claims (compatible with existing structure)
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
            iss: "squirrel-mcp".to_string(),
            aud: "squirrel-mcp-api".to_string(),
            jti: Uuid::new_v4().to_string(),
        }
    }

    /// Convert JWT claims to AuthContext
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

/// Capability-based JWT service configuration
#[derive(Debug, Clone)]
pub struct CapabilityJwtConfig {
    /// Crypto capability client configuration
    /// (Socket path from capability discovery!)
    pub crypto_config: CapabilityCryptoConfig,

    /// Key ID in crypto provider for JWT signing
    /// (Provider-specific, we don't care which primal!)
    pub key_id: String,

    /// Token expiry duration in hours (default: 24)
    pub expiry_hours: i64,
}

impl Default for CapabilityJwtConfig {
    fn default() -> Self {
        Self {
            crypto_config: CapabilityCryptoConfig::default(),
            key_id: "squirrel-jwt-signing-key".to_string(),
            expiry_hours: 24,
        }
    }
}

/// Capability-based JWT service (TRUE PRIMAL!)
///
/// **NO hardcoded primal names!**
/// - Discovers crypto capability at runtime
/// - Uses whichever primal provides "crypto.ed25519.sign"
/// - Currently might be BearDog, future could be any crypto primal
///
/// # Examples
///
/// ```no_run
/// use squirrel_mcp_auth::capability_jwt::{CapabilityJwtService, CapabilityJwtConfig, JwtClaims};
/// use chrono::{Utc, Duration};
/// use uuid::Uuid;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     // Config from capability discovery (NOT hardcoded!)
///     let config = CapabilityJwtConfig::default();
///     let jwt_service = CapabilityJwtService::new(config)?;
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
pub struct CapabilityJwtService {
    crypto_client: CapabilityCryptoProvider,
    key_id: String,
}

impl CapabilityJwtService {
    /// Create new capability-based JWT service
    ///
    /// **IMPORTANT**: Config should come from capability discovery!
    pub fn new(config: CapabilityJwtConfig) -> Result<Self> {
        info!("🌍 Initializing TRUE PRIMAL JWT service (capability-based discovery!)");
        info!(
            "Crypto capability: endpoint={:?}, key_id={}",
            config.crypto_config.endpoint, config.key_id
        );

        let crypto_client = CapabilityCryptoProvider::from_config(config.crypto_config);

        Ok(Self {
            crypto_client,
            key_id: config.key_id,
        })
    }

    /// Create from environment (for bootstrapping)
    ///
    /// Reads configuration from environment variables set by capability discovery.
    pub fn from_env() -> Result<Self> {
        let config = CapabilityJwtConfig::default();
        Self::new(config)
    }

    /// Create JWT token (delegates signing to discovered crypto capability)
    pub async fn create_token(&self, claims: &JwtClaims) -> Result<String, AuthError> {
        debug!(
            "Creating JWT token via crypto capability: user={}, session={}",
            claims.username, claims.session_id
        );

        // 1. Create and encode header
        let header = JwtHeader::default();
        let header_json = serde_json::to_vec(&header).map_err(|e| AuthError::Internal {
            message: format!("Failed to encode JWT header: {}", e),
        })?;
        let header_b64 = BASE64_URL.encode(&header_json);

        // 2. Encode claims
        let claims_json = serde_json::to_vec(&claims).map_err(|e| AuthError::Internal {
            message: format!("Failed to encode JWT claims: {}", e),
        })?;
        let claims_b64 = BASE64_URL.encode(&claims_json);

        // 3. Create signing input
        let signing_input = format!("{}.{}", header_b64, claims_b64);

        // 4. Sign via discovered crypto capability (Pure Rust!)
        let signature = self
            .crypto_client
            .clone() // Clone for async mutable access
            .sign_ed25519(signing_input.as_bytes())
            .await
            .context("Failed to sign JWT via crypto capability")
            .map_err(|e| AuthError::Internal {
                message: e.to_string(),
            })?;

        // 5. Encode signature
        let signature_b64 = BASE64_URL.encode(&signature);

        // 6. Construct final JWT
        let token = format!("{}.{}", signing_input, signature_b64);

        debug!(
            "JWT token created via crypto capability: length={}, header={}, claims={}, sig={}",
            token.len(),
            header_b64.len(),
            claims_b64.len(),
            signature_b64.len()
        );

        Ok(token)
    }

    /// Verify JWT token (delegates verification to discovered crypto capability)
    pub async fn verify_token(&self, token: &str) -> Result<JwtClaims, AuthError> {
        debug!(
            "Verifying JWT token via crypto capability: length={}",
            token.len()
        );

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

        // 3. Verify signature via discovered crypto capability (Pure Rust!)
        let signing_input = format!("{}.{}", header_b64, claims_b64);
        let is_valid = self
            .crypto_client
            .clone() // Clone for async mutable access
            .verify_ed25519_with_key_id(signing_input.as_bytes(), &signature, &self.key_id)
            .await
            .context("Failed to verify JWT signature via crypto capability")
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
            "JWT token verified via crypto capability: user={}, session={}, exp={}",
            claims.username, claims.session_id, claims.exp
        );

        Ok(claims)
    }

    /// Extract token from Authorization header
    ///
    /// Expected format: `Bearer <token>`
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
        assert_eq!(claims.iss, "squirrel-mcp");
        assert_eq!(claims.aud, "squirrel-mcp-api");
    }

    #[test]
    fn test_jwt_header_default() {
        let header = JwtHeader::default();
        assert_eq!(header.alg, "EdDSA");
        assert_eq!(header.typ, "JWT");
    }

    #[test]
    fn test_capability_jwt_config_default() {
        let config = CapabilityJwtConfig::default();
        assert_eq!(config.key_id, "squirrel-jwt-signing-key");
        assert_eq!(config.expiry_hours, 24);
    }

    #[test]
    fn test_extract_token_from_header() {
        let config = CapabilityJwtConfig::default();
        let service = CapabilityJwtService::new(config).unwrap();

        // Valid header
        let header = "Bearer abc123def456";
        let token = service.extract_token_from_header(header).unwrap();
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

    // Integration tests (with crypto capability provider) are in tests/integration/
}
