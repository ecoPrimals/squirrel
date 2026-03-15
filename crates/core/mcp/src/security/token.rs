// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Token management for MCP security
//!
//! This module provides token management functionality for the MCP system.
//! Actual token operations are delegated to the BearDog framework.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration};
use crate::error::Result;

/// Token type enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TokenType {
    Access,
    Refresh,
    Api,
    Session,
}

/// Token information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    pub id: Uuid,
    pub token_type: TokenType,
    pub user_id: Uuid,
    pub token_hash: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub last_used: Option<DateTime<Utc>>,
    pub is_revoked: bool,
    pub scopes: Vec<String>,
    pub metadata: HashMap<String, String>,
}

impl Token {
    pub fn new(token_type: TokenType, user_id: Uuid, token_hash: String, expires_at: DateTime<Utc>) -> Self {
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

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    pub fn is_valid(&self) -> bool {
        !self.is_revoked && !self.is_expired()
    }
}

/// Token configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenConfig {
    pub access_token_expiry_minutes: i64,
    pub refresh_token_expiry_days: i64,
    pub api_token_expiry_days: i64,
    pub session_token_expiry_hours: i64,
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
    /// Create a new token manager
    pub fn new() -> Self {
        Self {
            tokens: Arc::new(RwLock::new(HashMap::new())),
            token_hashes: Arc::new(RwLock::new(HashMap::new())),
            config: TokenConfig::default(),
        }
    }

    /// Create a new token manager with custom configuration
    pub fn new_with_config(config: TokenConfig) -> Self {
        Self {
            tokens: Arc::new(RwLock::new(HashMap::new())),
            token_hashes: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Create a new token
    pub async fn create_token(&self, token_type: TokenType, user_id: Uuid, token_string: String) -> Result<Token> {
        let token_hash = self.hash_token(&token_string);
        
        let expires_at = match token_type {
            TokenType::Access => Utc::now() + Duration::minutes(self.config.access_token_expiry_minutes),
            TokenType::Refresh => Utc::now() + Duration::days(self.config.refresh_token_expiry_days),
            TokenType::Api => Utc::now() + Duration::days(self.config.api_token_expiry_days),
            TokenType::Session => Utc::now() + Duration::hours(self.config.session_token_expiry_hours),
        };

        let token = Token::new(token_type, user_id, token_hash.clone(), expires_at);
        
        let mut tokens = self.tokens.write().await;
        let mut token_hashes = self.token_hashes.write().await;
        
        tokens.insert(token.id, token.clone());
        token_hashes.insert(token_hash, token.id);
        
        Ok(token)
    }

    /// Get token by ID
    pub async fn get_token(&self, id: &Uuid) -> Result<Option<Token>> {
        let tokens = self.tokens.read().await;
        Ok(tokens.get(id).cloned())
    }

    /// Get token by token string
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

    /// Validate token
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

    /// Update token
    pub async fn update_token(&self, token: Token) -> Result<()> {
        let mut tokens = self.tokens.write().await;
        tokens.insert(token.id, token);
        Ok(())
    }

    /// Revoke token
    pub async fn revoke_token(&self, id: &Uuid) -> Result<()> {
        let mut tokens = self.tokens.write().await;
        if let Some(token) = tokens.get_mut(id) {
            token.is_revoked = true;
        }
        Ok(())
    }

    /// Revoke token by string
    pub async fn revoke_token_by_string(&self, token_string: &str) -> Result<()> {
        if let Some(token) = self.get_token_by_string(token_string).await? {
            self.revoke_token(&token.id).await?;
        }
        Ok(())
    }

    /// Revoke all tokens for user
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

    /// Get user tokens
    pub async fn get_user_tokens(&self, user_id: &Uuid) -> Result<Vec<Token>> {
        let tokens = self.tokens.read().await;
        Ok(tokens.values().filter(|t| t.user_id == *user_id).cloned().collect())
    }

    /// Get active user tokens
    pub async fn get_active_user_tokens(&self, user_id: &Uuid) -> Result<Vec<Token>> {
        let tokens = self.tokens.read().await;
        Ok(tokens.values().filter(|t| t.user_id == *user_id && t.is_valid()).cloned().collect())
    }

    /// Cleanup expired tokens
    pub async fn cleanup_expired_tokens(&self) -> Result<usize> {
        let mut tokens = self.tokens.write().await;
        let mut token_hashes = self.token_hashes.write().await;
        let mut count = 0;
        
        let expired_tokens: Vec<_> = tokens.iter()
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

    /// Refresh token
    pub async fn refresh_token(&self, refresh_token_string: &str) -> Result<Option<(Token, Token)>> {
        if let Some(refresh_token) = self.get_token_by_string(refresh_token_string).await? {
            if refresh_token.token_type == TokenType::Refresh && refresh_token.is_valid() {
                // Create new access token
                let new_access_token = self.create_token(
                    TokenType::Access,
                    refresh_token.user_id,
                    format!("access_{}", Uuid::new_v4()),
                ).await?;
                
                // Create new refresh token
                let new_refresh_token = self.create_token(
                    TokenType::Refresh,
                    refresh_token.user_id,
                    format!("refresh_{}", Uuid::new_v4()),
                ).await?;
                
                // Revoke old refresh token
                self.revoke_token(&refresh_token.id).await?;
                
                return Ok(Some((new_access_token, new_refresh_token)));
            }
        }
        
        Ok(None)
    }

    /// Hash token string (placeholder implementation)
    fn hash_token(&self, token: &str) -> String {
        // Placeholder implementation - in production, use proper hashing
        format!("hash_{}", token)
    }

    /// Get token statistics
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

/// Token statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenStatistics {
    pub total: usize,
    pub active: usize,
    pub expired: usize,
    pub revoked: usize,
}

impl Default for DefaultTokenManager {
    fn default() -> Self {
        Self::new()
    }
} 