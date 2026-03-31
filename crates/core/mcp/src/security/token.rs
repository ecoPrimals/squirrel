// SPDX-License-Identifier: AGPL-3.0-or-later
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

    /// Deterministic BLAKE3 hash of the secret (hex-encoded); never stores raw secrets.
    #[expect(
        clippy::unused_self,
        reason = "Instance method keeps parity with future keyed/salted hashing"
    )]
    fn hash_token(&self, token: &str) -> String {
        blake3::hash(token.as_bytes()).to_hex().to_string()
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn past_expiry() -> DateTime<Utc> {
        Utc::now() - Duration::hours(1)
    }

    fn future_expiry() -> DateTime<Utc> {
        Utc::now() + Duration::hours(1)
    }

    #[test]
    fn token_new_sets_fields_and_generates_id() {
        let user = Uuid::new_v4();
        let hash = "h1".to_string();
        let exp = future_expiry();
        let token = Token::new(TokenType::Access, user, hash.clone(), exp);
        assert_eq!(token.token_type, TokenType::Access);
        assert_eq!(token.user_id, user);
        assert_eq!(token.token_hash, hash);
        assert_eq!(token.expires_at, exp);
        assert!(token.last_used.is_none());
        assert!(!token.is_revoked);
        assert!(token.scopes.is_empty());
        assert!(token.metadata.is_empty());
        assert_ne!(token.id, Uuid::nil());
    }

    #[test]
    fn token_is_expired_respects_expires_at() {
        let user = Uuid::new_v4();
        let past = Token::new(TokenType::Access, user, "x".to_string(), past_expiry());
        assert!(past.is_expired());

        let future = Token::new(TokenType::Access, user, "y".to_string(), future_expiry());
        assert!(!future.is_expired());
    }

    #[test]
    fn token_is_valid_requires_not_revoked_and_not_expired() {
        let user = Uuid::new_v4();
        let mut t = Token::new(TokenType::Api, user, "z".to_string(), future_expiry());
        assert!(t.is_valid());

        t.is_revoked = true;
        assert!(!t.is_valid());

        t.is_revoked = false;
        t.expires_at = past_expiry();
        assert!(!t.is_valid());
    }

    #[test]
    fn token_config_default_matches_documented_defaults() {
        let c = TokenConfig::default();
        assert_eq!(c.access_token_expiry_minutes, 60);
        assert_eq!(c.refresh_token_expiry_days, 30);
        assert_eq!(c.api_token_expiry_days, 365);
        assert_eq!(c.session_token_expiry_hours, 24);
        assert_eq!(c.cleanup_interval_hours, 24);
    }

    #[test]
    fn default_token_manager_default_matches_new() {
        let a = DefaultTokenManager::new();
        let b = DefaultTokenManager::default();
        assert_eq!(
            a.config.access_token_expiry_minutes,
            b.config.access_token_expiry_minutes
        );
        assert_eq!(
            a.config.refresh_token_expiry_days,
            b.config.refresh_token_expiry_days
        );
    }

    #[test]
    fn debug_format_includes_discriminant_for_token_type() {
        let s = format!("{:?}", TokenType::Refresh);
        assert!(s.contains("Refresh"));
    }

    #[test]
    fn debug_format_for_token_includes_core_fields() {
        let user = Uuid::new_v4();
        let token = Token::new(
            TokenType::Session,
            user,
            "hashval".to_string(),
            future_expiry(),
        );
        let dbg = format!("{token:?}");
        assert!(dbg.contains("Session"));
        assert!(dbg.contains(&user.to_string()));
        assert!(dbg.contains("hashval"));
    }

    #[test]
    fn token_type_serde_roundtrip() {
        for tt in [
            TokenType::Access,
            TokenType::Refresh,
            TokenType::Api,
            TokenType::Session,
        ] {
            let json = serde_json::to_string(&tt).expect("serialize");
            let back: TokenType = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(back, tt);
        }
    }

    #[tokio::test]
    async fn create_token_stores_and_resolves_by_string() {
        let mgr = DefaultTokenManager::new();
        let user = Uuid::new_v4();
        let secret = "my-secret-token".to_string();
        let created = mgr
            .create_token(TokenType::Access, user, secret.clone())
            .await
            .expect("create");

        let by_id = mgr
            .get_token(&created.id)
            .await
            .expect("get")
            .expect("some");
        assert_eq!(by_id.id, created.id);

        let by_str = mgr
            .get_token_by_string(&secret)
            .await
            .expect("lookup")
            .expect("found");
        assert_eq!(by_str.id, created.id);
        assert_eq!(
            by_str.token_hash,
            blake3::hash(secret.as_bytes()).to_hex().to_string()
        );
    }

    #[tokio::test]
    async fn get_token_by_string_unknown_returns_none() {
        let mgr = DefaultTokenManager::new();
        let r = mgr.get_token_by_string("not-present").await.expect("ok");
        assert!(r.is_none());
    }

    #[tokio::test]
    async fn validate_token_updates_last_used_when_valid() {
        let mgr = DefaultTokenManager::new();
        let user = Uuid::new_v4();
        let secret = "valid-secret".to_string();
        mgr.create_token(TokenType::Api, user, secret.clone())
            .await
            .expect("create");

        let validated = mgr
            .validate_token(&secret)
            .await
            .expect("validate")
            .expect("some");
        assert!(validated.last_used.is_some());
        let stored = mgr
            .get_token(&validated.id)
            .await
            .expect("get")
            .expect("some");
        assert!(stored.last_used.is_some());
    }

    #[tokio::test]
    async fn validate_token_returns_none_when_expired() {
        let mgr = DefaultTokenManager::new();
        let user = Uuid::new_v4();
        let secret = "expiring".to_string();
        let mut t = mgr
            .create_token(TokenType::Access, user, secret.clone())
            .await
            .expect("create");
        t.expires_at = past_expiry();
        mgr.update_token(t).await.expect("update");

        let r = mgr.validate_token(&secret).await.expect("validate");
        assert!(r.is_none());
    }

    #[tokio::test]
    async fn validate_token_returns_none_when_revoked() {
        let mgr = DefaultTokenManager::new();
        let user = Uuid::new_v4();
        let secret = "revoked".to_string();
        let t = mgr
            .create_token(TokenType::Access, user, secret.clone())
            .await
            .expect("create");
        mgr.revoke_token(&t.id).await.expect("revoke");

        let r = mgr.validate_token(&secret).await.expect("validate");
        assert!(r.is_none());
    }

    #[tokio::test]
    async fn revoke_user_tokens_marks_all_for_user() {
        let mgr = DefaultTokenManager::new();
        let u1 = Uuid::new_v4();
        let u2 = Uuid::new_v4();
        mgr.create_token(TokenType::Api, u1, "a1".to_string())
            .await
            .expect("c1");
        mgr.create_token(TokenType::Api, u1, "a2".to_string())
            .await
            .expect("c2");
        mgr.create_token(TokenType::Api, u2, "b1".to_string())
            .await
            .expect("c3");

        let n = mgr.revoke_user_tokens(&u1).await.expect("revoke");
        assert_eq!(n, 2);
        let u2_active = mgr.get_active_user_tokens(&u2).await.expect("list");
        assert_eq!(u2_active.len(), 1);
        let u1_active = mgr.get_active_user_tokens(&u1).await.expect("list");
        assert!(u1_active.is_empty());
    }

    #[tokio::test]
    async fn cleanup_expired_tokens_removes_rows() {
        let mgr = DefaultTokenManager::new();
        let user = Uuid::new_v4();
        let secret = "cleanup-me".to_string();
        let mut t = mgr
            .create_token(TokenType::Access, user, secret.clone())
            .await
            .expect("create");
        t.expires_at = past_expiry();
        mgr.update_token(t).await.expect("update");

        let removed = mgr.cleanup_expired_tokens().await.expect("cleanup");
        assert_eq!(removed, 1);
        assert!(
            mgr.get_token_by_string(&secret)
                .await
                .expect("get")
                .is_none()
        );
    }

    #[tokio::test]
    async fn refresh_token_exchanges_valid_refresh_and_revokes_old() {
        let mgr = DefaultTokenManager::new();
        let user = Uuid::new_v4();
        let refresh_secret = "refresh-orig".to_string();
        let old = mgr
            .create_token(TokenType::Refresh, user, refresh_secret.clone())
            .await
            .expect("create");

        let pair = mgr
            .refresh_token(&refresh_secret)
            .await
            .expect("refresh")
            .expect("some pair");
        let (access, refresh_new) = pair;
        assert_eq!(access.token_type, TokenType::Access);
        assert_eq!(refresh_new.token_type, TokenType::Refresh);

        let old_row = mgr.get_token(&old.id).await.expect("get").expect("row");
        assert!(old_row.is_revoked);
    }

    #[tokio::test]
    async fn refresh_token_returns_none_for_non_refresh_type() {
        let mgr = DefaultTokenManager::new();
        let user = Uuid::new_v4();
        let secret = "access-only".to_string();
        mgr.create_token(TokenType::Access, user, secret.clone())
            .await
            .expect("create");

        let r = mgr.refresh_token(&secret).await.expect("refresh");
        assert!(r.is_none());
    }

    #[tokio::test]
    async fn get_token_statistics_reflects_inventory() {
        let mgr = DefaultTokenManager::new();
        let user = Uuid::new_v4();
        let mut expired = mgr
            .create_token(TokenType::Access, user, "e1".to_string())
            .await
            .expect("c1");
        expired.expires_at = past_expiry();
        mgr.update_token(expired).await.expect("upd");

        let _ = mgr
            .create_token(TokenType::Access, user, "ok".to_string())
            .await
            .expect("c2");

        let stats = mgr.get_token_statistics().await.expect("stats");
        assert_eq!(stats.total, 2);
        assert_eq!(stats.expired, 1);
        assert_eq!(stats.active, 1);
    }
}
