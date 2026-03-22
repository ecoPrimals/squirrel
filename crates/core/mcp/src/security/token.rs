// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Token management for MCP security
//!
//! This module provides token management functionality for the MCP system.
//! Actual token operations are delegated to the BearDog framework.

use crate::error::Result;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Kind of token issued or validated by the manager.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TokenType {
    /// Short-lived API access token.
    Access,
    /// Long-lived token used to obtain new access tokens.
    Refresh,
    /// Long-lived automation or integration token.
    Api,
    /// Interactive browser or session token.
    Session,
}

/// Stored token record with hash and lifecycle fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    /// Unique record id for this token.
    pub id: Uuid,
    /// Discriminated token kind.
    pub token_type: TokenType,
    /// Owning user id.
    pub user_id: Uuid,
    /// Opaque hash of the secret material (not the raw secret).
    pub token_hash: String,
    /// Creation timestamp (UTC).
    pub created_at: DateTime<Utc>,
    /// Instant after which the token must not be accepted.
    pub expires_at: DateTime<Utc>,
    /// Last successful validation time, if any.
    pub last_used: Option<DateTime<Utc>>,
    /// Whether the token was explicitly invalidated.
    pub is_revoked: bool,
    /// OAuth-style scope strings, if used.
    pub scopes: Vec<String>,
    /// Extension metadata for callers or policy.
    pub metadata: HashMap<String, String>,
}

impl Token {
    /// Creates a new token row with empty scopes and metadata.
    #[must_use]
    pub fn new(
        token_type: TokenType,
        user_id: Uuid,
        token_hash: String,
        expires_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            token_type,
            user_id,
            token_hash,
            created_at: Utc::now(),
            expires_at,
            last_used: None,
            is_revoked: false,
            scopes: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Returns true if the current time is past `expires_at`.
    #[must_use]
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Returns true if the token is neither revoked nor expired.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        !self.is_revoked && !self.is_expired()
    }
}

/// Expiry durations used when minting tokens of each type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenConfig {
    /// Access token lifetime in minutes.
    pub access_token_expiry_minutes: i64,
    /// Refresh token lifetime in days.
    pub refresh_token_expiry_days: i64,
    /// API token lifetime in days.
    pub api_token_expiry_days: i64,
    /// Session token lifetime in hours.
    pub session_token_expiry_hours: i64,
    /// Suggested background cleanup interval for expired rows.
    pub cleanup_interval_hours: i64,
}

impl Default for TokenConfig {
    fn default() -> Self {
        Self {
            access_token_expiry_minutes: 60,
            refresh_token_expiry_days: 30,
            api_token_expiry_days: 365,
            session_token_expiry_hours: 24,
            cleanup_interval_hours: 24,
        }
    }
}

/// Default token manager implementation
///
/// This provides basic token management that can be extended
/// or replaced with BearDog integration in the future.
#[derive(Debug, Clone)]
pub struct DefaultTokenManager {
    tokens: Arc<RwLock<HashMap<Uuid, Token>>>,
    token_hashes: Arc<RwLock<HashMap<String, Uuid>>>,
    config: TokenConfig,
}

impl DefaultTokenManager {
    /// Creates a manager with default expiry configuration.
    #[must_use]
    pub fn new() -> Self {
        Self {
            tokens: Arc::new(RwLock::new(HashMap::new())),
            token_hashes: Arc::new(RwLock::new(HashMap::new())),
            config: TokenConfig::default(),
        }
    }

    /// Creates a manager using the supplied expiry configuration.
    #[must_use]
    pub fn new_with_config(config: TokenConfig) -> Self {
        Self {
            tokens: Arc::new(RwLock::new(HashMap::new())),
            token_hashes: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Hashes the secret string, assigns expiry from config, and stores the record.
    pub async fn create_token(
        &self,
        token_type: TokenType,
        user_id: Uuid,
        token_string: String,
    ) -> Result<Token> {
        let token_hash = self.hash_token(&token_string);

        let expires_at = match token_type {
            TokenType::Access => {
                Utc::now() + Duration::minutes(self.config.access_token_expiry_minutes)
            }
            TokenType::Refresh => {
                Utc::now() + Duration::days(self.config.refresh_token_expiry_days)
            }
            TokenType::Api => Utc::now() + Duration::days(self.config.api_token_expiry_days),
            TokenType::Session => {
                Utc::now() + Duration::hours(self.config.session_token_expiry_hours)
            }
        };

        let token = Token::new(token_type, user_id, token_hash.clone(), expires_at);

        let mut tokens = self.tokens.write().await;
        let mut token_hashes = self.token_hashes.write().await;

        tokens.insert(token.id, token.clone());
        token_hashes.insert(token_hash, token.id);

        Ok(token)
    }

    /// Returns the token record by id, if present.
    pub async fn get_token(&self, id: &Uuid) -> Result<Option<Token>> {
        let tokens = self.tokens.read().await;
        Ok(tokens.get(id).cloned())
    }

    /// Resolves a token from its secret string via the hash index.
    pub async fn get_token_by_string(&self, token_string: &str) -> Result<Option<Token>> {
        let token_hash = self.hash_token(token_string);
        let token_hashes = self.token_hashes.read().await;

        if let Some(token_id) = token_hashes.get(&token_hash) {
            let tokens = self.tokens.read().await;
            Ok(tokens.get(token_id).cloned())
        } else {
            Ok(None)
        }
    }

    /// Returns the token after validation and updates `last_used` when valid.
    pub async fn validate_token(&self, token_string: &str) -> Result<Option<Token>> {
        if let Some(mut token) = self.get_token_by_string(token_string).await? {
            if token.is_valid() {
                // Update last used timestamp
                token.last_used = Some(Utc::now());
                self.update_token(token.clone()).await?;
                Ok(Some(token))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    /// Inserts or replaces a token record by id.
    pub async fn update_token(&self, token: Token) -> Result<()> {
        let mut tokens = self.tokens.write().await;
        tokens.insert(token.id, token);
        Ok(())
    }

    /// Marks a token as revoked by id.
    pub async fn revoke_token(&self, id: &Uuid) -> Result<()> {
        let mut tokens = self.tokens.write().await;
        if let Some(token) = tokens.get_mut(id) {
            token.is_revoked = true;
        }
        Ok(())
    }

    /// Resolves the token from its secret string and revokes it.
    pub async fn revoke_token_by_string(&self, token_string: &str) -> Result<()> {
        if let Some(token) = self.get_token_by_string(token_string).await? {
            self.revoke_token(&token.id).await?;
        }
        Ok(())
    }

    /// Revokes every non-revoked token belonging to the user and returns count.
    pub async fn revoke_user_tokens(&self, user_id: &Uuid) -> Result<usize> {
        let mut tokens = self.tokens.write().await;
        let mut count = 0;

        for token in tokens.values_mut() {
            if token.user_id == *user_id && !token.is_revoked {
                token.is_revoked = true;
                count += 1;
            }
        }

        Ok(count)
    }

    /// Lists all tokens issued for the user.
    pub async fn get_user_tokens(&self, user_id: &Uuid) -> Result<Vec<Token>> {
        let tokens = self.tokens.read().await;
        Ok(tokens
            .values()
            .filter(|t| t.user_id == *user_id)
            .cloned()
            .collect())
    }

    /// Lists tokens for the user that pass `is_valid`.
    pub async fn get_active_user_tokens(&self, user_id: &Uuid) -> Result<Vec<Token>> {
        let tokens = self.tokens.read().await;
        Ok(tokens
            .values()
            .filter(|t| t.user_id == *user_id && t.is_valid())
            .cloned()
            .collect())
    }

    /// Deletes expired tokens from both maps and returns how many were removed.
    pub async fn cleanup_expired_tokens(&self) -> Result<usize> {
        let mut tokens = self.tokens.write().await;
        let mut token_hashes = self.token_hashes.write().await;
        let mut count = 0;

        let expired_tokens: Vec<_> = tokens
            .iter()
            .filter(|(_, token)| token.is_expired())
            .map(|(id, token)| (*id, token.token_hash.clone()))
            .collect();

        for (id, hash) in expired_tokens {
            tokens.remove(&id);
            token_hashes.remove(&hash);
            count += 1;
        }

        Ok(count)
    }

    /// Exchanges a valid refresh token for a new access and refresh pair.
    pub async fn refresh_token(
        &self,
        refresh_token_string: &str,
    ) -> Result<Option<(Token, Token)>> {
        if let Some(refresh_token) = self.get_token_by_string(refresh_token_string).await?
            && refresh_token.token_type == TokenType::Refresh
            && refresh_token.is_valid()
        {
            // Create new access token
            let new_access_token = self
                .create_token(
                    TokenType::Access,
                    refresh_token.user_id,
                    format!("access_{}", Uuid::new_v4()),
                )
                .await?;

            // Create new refresh token
            let new_refresh_token = self
                .create_token(
                    TokenType::Refresh,
                    refresh_token.user_id,
                    format!("refresh_{}", Uuid::new_v4()),
                )
                .await?;

            // Revoke old refresh token
            self.revoke_token(&refresh_token.id).await?;

            return Ok(Some((new_access_token, new_refresh_token)));
        }

        Ok(None)
    }

    /// Hash token string (stub — will use proper hashing via BearDog)
    #[expect(
        clippy::unused_self,
        reason = "will use internal hash config when BearDog is integrated"
    )]
    fn hash_token(&self, token: &str) -> String {
        format!("hash_{token}")
    }

    /// Returns aggregate counts for stored tokens.
    pub async fn get_token_statistics(&self) -> Result<TokenStatistics> {
        let tokens = self.tokens.read().await;
        let total = tokens.len();
        let active = tokens.values().filter(|t| t.is_valid()).count();
        let expired = tokens.values().filter(|t| t.is_expired()).count();
        let revoked = tokens.values().filter(|t| t.is_revoked).count();

        Ok(TokenStatistics {
            total,
            active,
            expired,
            revoked,
        })
    }
}

/// Aggregate counts for token inventory reporting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenStatistics {
    /// Total token rows in storage.
    pub total: usize,
    /// Tokens that are valid (not revoked, not expired).
    pub active: usize,
    /// Tokens past expiry (may still be present until cleanup).
    pub expired: usize,
    /// Tokens that were explicitly revoked.
    pub revoked: usize,
}

impl Default for DefaultTokenManager {
    fn default() -> Self {
        Self::new()
    }
}
