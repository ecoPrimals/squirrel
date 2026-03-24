// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

use crate::{AuthContext, AuthError, Permission};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Debug, Clone, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
pub struct BearerToken {
    pub id: Uuid,
    #[zeroize(drop)]
    pub token: String,
    pub user_id: Uuid,
    pub username: String,
    pub name: String,
    pub permissions: Vec<Permission>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub last_used: Option<DateTime<Utc>>,
    pub active: bool,
}

impl BearerToken {
    pub fn new(
        user_id: Uuid,
        username: String,
        name: String,
        permissions: Vec<Permission>,
        expires_at: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            token: Self::generate_token(),
            user_id,
            username,
            name,
            permissions,
            expires_at,
            created_at: Utc::now(),
            last_used: None,
            active: true,
        }
    }

    fn generate_token() -> String {
        // Generate a secure random token
        let random_bytes: [u8; 32] = rand::random();
        format!("smc_{}", STANDARD.encode(random_bytes))
    }

    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }

    pub fn is_valid(&self) -> bool {
        self.active && !self.is_expired()
    }

    pub fn touch(&mut self) {
        self.last_used = Some(Utc::now());
    }

    pub fn revoke(&mut self) {
        self.active = false;
    }

    pub fn to_auth_context(&self) -> AuthContext {
        AuthContext {
            user_id: self.user_id,
            username: self.username.clone(),
            permissions: self.permissions.clone(),
            session_id: self.id, // Use token ID as session ID for API tokens
            expires_at: self
                .expires_at
                .unwrap_or_else(|| Utc::now() + chrono::Duration::days(365)),
            issued_at: self.created_at,
            roles: vec!["api".to_string()], // API tokens have a special "api" role
        }
    }
}

pub struct BearerTokenValidator {
    tokens: Arc<RwLock<HashMap<String, BearerToken>>>,
    user_tokens: Arc<RwLock<HashMap<Uuid, Vec<String>>>>,
}

impl BearerTokenValidator {
    pub fn new() -> Self {
        Self {
            tokens: Arc::new(RwLock::new(HashMap::new())),
            user_tokens: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn create_token(
        &self,
        user_id: Uuid,
        username: String,
        name: String,
        permissions: Vec<Permission>,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<BearerToken, AuthError> {
        let token = BearerToken::new(user_id, username, name, permissions, expires_at);
        let token_value = token.token.clone();

        // Store token
        {
            let mut tokens = self.tokens.write().await;
            tokens.insert(token_value.clone(), token.clone());
        }

        // Track user tokens
        {
            let mut user_tokens = self.user_tokens.write().await;
            user_tokens.entry(user_id).or_default().push(token_value);
        }

        Ok(token)
    }

    pub async fn validate_token(&self, token: &str) -> Result<AuthContext, AuthError> {
        let mut tokens = self.tokens.write().await;

        if let Some(bearer_token) = tokens.get_mut(token) {
            if !bearer_token.is_valid() {
                return Err(AuthError::TokenExpired);
            }

            // Update last used timestamp
            bearer_token.touch();

            Ok(bearer_token.to_auth_context())
        } else {
            Err(AuthError::InvalidToken)
        }
    }

    pub async fn revoke_token(&self, token: &str) -> Result<(), AuthError> {
        let mut tokens = self.tokens.write().await;

        if let Some(bearer_token) = tokens.get_mut(token) {
            bearer_token.revoke();
            Ok(())
        } else {
            Err(AuthError::InvalidToken)
        }
    }

    pub async fn revoke_user_tokens(&self, user_id: &Uuid) -> Result<u32, AuthError> {
        let mut revoked_count = 0;
        let user_tokens = self.user_tokens.read().await;

        if let Some(token_values) = user_tokens.get(user_id) {
            let mut tokens = self.tokens.write().await;

            for token_value in token_values {
                if let Some(bearer_token) = tokens.get_mut(token_value) {
                    if bearer_token.active {
                        bearer_token.revoke();
                        revoked_count += 1;
                    }
                }
            }
        }

        Ok(revoked_count)
    }

    pub async fn list_user_tokens(&self, user_id: &Uuid) -> Result<Vec<BearerToken>, AuthError> {
        let user_tokens = self.user_tokens.read().await;
        let tokens = self.tokens.read().await;

        let mut result = Vec::new();
        if let Some(token_values) = user_tokens.get(user_id) {
            for token_value in token_values {
                if let Some(bearer_token) = tokens.get(token_value) {
                    result.push(bearer_token.clone());
                }
            }
        }

        Ok(result)
    }

    pub async fn cleanup_expired_tokens(&self) -> Result<u32, AuthError> {
        let mut removed_count = 0;

        // Clean up expired tokens
        {
            let mut tokens = self.tokens.write().await;
            tokens.retain(|_, token| {
                if token.is_expired() {
                    removed_count += 1;
                    false
                } else {
                    true
                }
            });
        }

        // Clean up user token references
        {
            let mut user_tokens = self.user_tokens.write().await;
            let tokens = self.tokens.read().await;

            for (_, token_values) in user_tokens.iter_mut() {
                token_values.retain(|token_value| tokens.contains_key(token_value));
            }

            // Remove empty user token lists
            user_tokens.retain(|_, token_values| !token_values.is_empty());
        }

        Ok(removed_count)
    }

    pub async fn get_token_stats(&self) -> Result<TokenStats, AuthError> {
        let tokens = self.tokens.read().await;

        let total_tokens = tokens.len();
        let active_tokens = tokens.values().filter(|t| t.is_valid()).count();
        let expired_tokens = tokens.values().filter(|t| t.is_expired()).count();
        let revoked_tokens = tokens.values().filter(|t| !t.active).count();

        Ok(TokenStats {
            total_tokens,
            active_tokens,
            expired_tokens,
            revoked_tokens,
            timestamp: Utc::now(),
        })
    }

    pub fn extract_bearer_token<'a>(
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenStats {
    pub total_tokens: usize,
    pub active_tokens: usize,
    pub expired_tokens: usize,
    pub revoked_tokens: usize,
    pub timestamp: DateTime<Utc>,
}

impl Default for BearerTokenValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[tokio::test]
    async fn test_token_creation() {
        let validator = BearerTokenValidator::new();
        let user_id = Uuid::new_v4();
        let expires_at = Some(Utc::now() + Duration::hours(1));

        let token = validator
            .create_token(
                user_id,
                "test_user".to_string(),
                "API Token".to_string(),
                vec![],
                expires_at,
            )
            .await
            .expect("should succeed");

        assert_eq!(token.user_id, user_id);
        assert_eq!(token.username, "test_user");
        assert!(token.is_valid());
        assert!(token.token.starts_with("smc_"));
    }

    #[tokio::test]
    async fn test_token_validation() {
        let validator = BearerTokenValidator::new();
        let user_id = Uuid::new_v4();

        let token = validator
            .create_token(
                user_id,
                "test_user".to_string(),
                "API Token".to_string(),
                vec![],
                None,
            )
            .await
            .expect("should succeed");

        let auth_context = validator.validate_token(&token.token).await.expect("should succeed");
        assert_eq!(auth_context.user_id, user_id);
        assert_eq!(auth_context.username, "test_user");
    }

    #[tokio::test]
    async fn test_token_revocation() {
        let validator = BearerTokenValidator::new();
        let user_id = Uuid::new_v4();

        let token = validator
            .create_token(
                user_id,
                "test_user".to_string(),
                "API Token".to_string(),
                vec![],
                None,
            )
            .await
            .expect("should succeed");

        // Token should be valid initially
        assert!(validator.validate_token(&token.token).await.is_ok());

        // Revoke the token
        validator.revoke_token(&token.token).await.expect("should succeed");

        // Token should now be invalid
        let result = validator.validate_token(&token.token).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AuthError::TokenExpired));
    }

    #[tokio::test]
    async fn test_expired_token() {
        let validator = BearerTokenValidator::new();
        let user_id = Uuid::new_v4();
        let expires_at = Some(Utc::now() - Duration::hours(1)); // Already expired

        let token = validator
            .create_token(
                user_id,
                "test_user".to_string(),
                "API Token".to_string(),
                vec![],
                expires_at,
            )
            .await
            .expect("should succeed");

        let result = validator.validate_token(&token.token).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AuthError::TokenExpired));
    }

    #[tokio::test]
    async fn test_extract_bearer_token() {
        let validator = BearerTokenValidator::new();

        let header = "Bearer smc_abc123def456";
        let token = validator.extract_bearer_token(header).expect("should succeed");
        assert_eq!(token, "smc_abc123def456");

        let invalid_header = "smc_abc123def456";
        let result = validator.extract_bearer_token(invalid_header);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AuthError::InvalidToken));
    }

    #[tokio::test]
    async fn test_user_token_revocation() {
        let validator = BearerTokenValidator::new();
        let user_id = Uuid::new_v4();

        // Create multiple tokens for the user
        let _token1 = validator
            .create_token(
                user_id,
                "test_user".to_string(),
                "Token 1".to_string(),
                vec![],
                None,
            )
            .await
            .expect("should succeed");

        let _token2 = validator
            .create_token(
                user_id,
                "test_user".to_string(),
                "Token 2".to_string(),
                vec![],
                None,
            )
            .await
            .expect("should succeed");

        // Revoke all user tokens
        let revoked_count = validator.revoke_user_tokens(&user_id).await.expect("should succeed");
        assert_eq!(revoked_count, 2);
    }
}
