// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! JWT Token Management (Feature-Gated for Dev Mode)
//!
//! This module provides LOCAL JWT validation using jsonwebtoken crate.
//! It brings `ring` v0.17 C dependency, so it's feature-gated for dev/testing only.
//!
//! **Production Mode**: Use `BearDog` JWT delegation (Pure Rust!)
//! **Dev Mode**: Use this module (fast iteration, with ring C dep)
//!
//! Enable with: `--features local-jwt`

use crate::{AuthContext, AuthError};
use chrono::{DateTime, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use universal_constants::identity;
use uuid::Uuid;

/// JWT claims payload for local (dev-mode) token validation.
///
/// Maps standard IETF RFC 7519 registered claims to Squirrel's auth context.
/// Production mode delegates JWT validation to a capability-discovered provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    /// Subject — user ID as UUID string
    pub sub: String,
    /// Human-readable username
    pub username: String,
    /// Authorization roles (e.g. `["admin", "user"]`)
    pub roles: Vec<String>,
    /// Session identifier as UUID string
    pub session_id: String,
    /// Issued-at timestamp (Unix epoch seconds)
    pub iat: i64,
    /// Expiration timestamp (Unix epoch seconds)
    pub exp: i64,
    /// Not-before timestamp (Unix epoch seconds)
    pub nbf: i64,
    /// Issuer identifier
    pub iss: String,
    /// Audience identifier
    pub aud: String,
    /// Unique JWT identifier (jti)
    pub jti: String,
}

impl JwtClaims {
    /// Build claims for a new token with the given identity and expiration.
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

    /// Convert validated claims into an [`AuthContext`] for downstream authorization.
    pub fn to_auth_context(&self) -> Result<AuthContext, AuthError> {
        let user_id = Uuid::parse_str(&self.sub).map_err(|_| AuthError::InvalidToken)?;

        let session_id = Uuid::parse_str(&self.session_id).map_err(|_| AuthError::InvalidToken)?;

        let created_at = DateTime::from_timestamp(self.iat, 0).ok_or(AuthError::InvalidToken)?;

        let expires_at = DateTime::from_timestamp(self.exp, 0).ok_or(AuthError::InvalidToken)?;

        Ok(AuthContext {
            user_id,
            username: self.username.clone(),
            permissions: vec![],
            session_id,
            expires_at,
            created_at,
            roles: self.roles.clone(),
            auth_provider: crate::types::AuthProvider::Standalone,
        })
    }
}

/// HMAC-HS256 token manager for local (dev-mode) JWT creation and verification.
///
/// Uses `jsonwebtoken` crate which pulls the `ring` C dependency.
/// Production deployments should use [`DelegatedJwtClient`](crate::DelegatedJwtClient) instead.
pub struct JwtTokenManager {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    validation: Validation,
}

impl JwtTokenManager {
    /// Create a token manager with the given HMAC secret.
    pub fn new(secret: &[u8]) -> Self {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_issuer(&[identity::JWT_ISSUER]);
        validation.set_audience(&[identity::JWT_AUDIENCE]);
        validation.validate_exp = true;
        validation.validate_nbf = true;

        Self {
            encoding_key: EncodingKey::from_secret(secret),
            decoding_key: DecodingKey::from_secret(secret),
            validation,
        }
    }

    /// Encode claims into a signed JWT string.
    pub fn create_token(&self, claims: &JwtClaims) -> Result<String, AuthError> {
        let header = Header::new(Algorithm::HS256);

        encode(&header, claims, &self.encoding_key).map_err(|e| AuthError::Internal {
            message: e.to_string(),
        })
    }

    /// Verify a JWT string and return the decoded claims.
    pub fn verify_token(&self, token: &str) -> Result<JwtClaims, AuthError> {
        let token_data = decode::<JwtClaims>(token, &self.decoding_key, &self.validation).map_err(
            |e| match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::TokenExpired,
                jsonwebtoken::errors::ErrorKind::InvalidToken => AuthError::InvalidToken,
                _ => AuthError::Internal {
                    message: e.to_string(),
                },
            },
        )?;

        Ok(token_data.claims)
    }

    /// Extract the bearer token from an `Authorization` header value.
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
    fn test_jwt_token_creation_and_verification() {
        let secret = b"test-secret-key";
        let token_manager = JwtTokenManager::new(secret);

        let user_id = Uuid::new_v4();
        let session_id = Uuid::new_v4();
        let expires_at = Utc::now() + Duration::hours(1);

        let claims = JwtClaims::new(
            user_id,
            "test_user".to_string(),
            vec!["user".to_string()],
            session_id,
            expires_at,
        );

        let token = token_manager.create_token(&claims).unwrap();
        let verified_claims = token_manager.verify_token(&token).unwrap();

        assert_eq!(claims.sub, verified_claims.sub);
        assert_eq!(claims.username, verified_claims.username);
        assert_eq!(claims.roles, verified_claims.roles);
        assert_eq!(claims.session_id, verified_claims.session_id);
    }

    #[test]
    fn test_expired_token() {
        let secret = b"test-secret-key";
        let token_manager = JwtTokenManager::new(secret);

        let user_id = Uuid::new_v4();
        let session_id = Uuid::new_v4();
        let expires_at = Utc::now() - Duration::hours(1); // Already expired

        let claims = JwtClaims::new(
            user_id,
            "test_user".to_string(),
            vec!["user".to_string()],
            session_id,
            expires_at,
        );

        let token = token_manager.create_token(&claims).unwrap();
        let result = token_manager.verify_token(&token);

        assert!(matches!(result, Err(AuthError::TokenExpired)));
    }

    #[test]
    fn test_extract_token_from_header() {
        let secret = b"test-secret-key";
        let token_manager = JwtTokenManager::new(secret);

        let header = "Bearer abc123def456";
        let token = token_manager.extract_token_from_header(header).unwrap();
        assert_eq!(token, "abc123def456");

        let invalid_header = "abc123def456";
        let result = token_manager.extract_token_from_header(invalid_header);
        assert!(matches!(result, Err(AuthError::InvalidToken)));
    }
}
