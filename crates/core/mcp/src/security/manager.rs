// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Security manager for MCP security
//!
//! This module provides security management functionality for the MCP system.
//! Actual security operations are delegated to the BearDog framework.
//!
//! **Nov 9, 2025 Update**: SecurityConfig consolidated into unified config system.
//! Re-exported from `squirrel-mcp-config` for consistency.

use crate::error::Result;
use std::sync::Arc;
use uuid::Uuid;

use super::audit::{AuditEvent, DefaultAuditService};
use super::crypto::DefaultCryptoProvider;
use super::identity::{DefaultIdentityManager, UserIdentity};
use super::key_storage::InMemoryKeyStorage;
use super::rbac::{BasicRBACManager, Permission};
use super::token::DefaultTokenManager;

// Re-export SecurityConfig from unified config (Nov 9, 2025 consolidation)
// This provides enable_audit, enable_encryption, enable_rbac, token_expiry_minutes
// plus all other unified security configuration fields
pub use squirrel_mcp_config::SecurityConfig;

/// Security manager implementation
///
/// This provides comprehensive security management that can be extended
/// or replaced with BearDog integration in the future.
#[derive(Debug)]
pub struct SecurityManagerImpl {
    config: SecurityConfig,
    audit_service: Arc<DefaultAuditService>,
    crypto_provider: Arc<DefaultCryptoProvider>,
    identity_manager: Arc<DefaultIdentityManager>,
    key_storage: Arc<InMemoryKeyStorage>,
    rbac_manager: Arc<BasicRBACManager>,
    token_manager: Arc<DefaultTokenManager>,
}

impl SecurityManagerImpl {
    /// Create a new security manager
    pub fn new(config: SecurityConfig) -> Self {
        Self {
            config,
            audit_service: Arc::new(DefaultAuditService::new()),
            crypto_provider: Arc::new(DefaultCryptoProvider::new()),
            identity_manager: Arc::new(DefaultIdentityManager::new()),
            key_storage: Arc::new(InMemoryKeyStorage::new()),
            rbac_manager: Arc::new(BasicRBACManager::new()),
            token_manager: Arc::new(DefaultTokenManager::new()),
        }
    }

    /// Get audit service
    pub fn audit_service(&self) -> Arc<DefaultAuditService> {
        self.audit_service.clone()
    }

    /// Get crypto provider
    pub fn crypto_provider(&self) -> Arc<DefaultCryptoProvider> {
        self.crypto_provider.clone()
    }

    /// Get identity manager
    pub fn identity_manager(&self) -> Arc<DefaultIdentityManager> {
        self.identity_manager.clone()
    }

    /// Get key storage
    pub fn key_storage(&self) -> Arc<InMemoryKeyStorage> {
        self.key_storage.clone()
    }

    /// Get RBAC manager
    pub fn rbac_manager(&self) -> Arc<BasicRBACManager> {
        self.rbac_manager.clone()
    }

    /// Get token manager
    pub fn token_manager(&self) -> Arc<DefaultTokenManager> {
        self.token_manager.clone()
    }

    /// Initialize security manager (stub — will delegate to BearDog)
    #[expect(
        clippy::missing_const_for_fn,
        reason = "will be non-const when BearDog init is integrated"
    )]
    pub fn initialize(&self) -> Result<()> {
        Ok(())
    }

    /// Authenticate user
    pub async fn authenticate(
        &self,
        username: &str,
        password: &str,
    ) -> Result<Option<UserIdentity>> {
        let result = self
            .identity_manager
            .authenticate(username, password)
            .await?;

        if self.config.enable_audit {
            let event = AuditEvent {
                id: Uuid::new_v4(),
                timestamp: chrono::Utc::now(),
                event_type: "authentication".to_string(),
                user_id: Some(username.to_string()),
                resource_id: None,
                action: "login".to_string(),
                status: if result.is_some() {
                    "success"
                } else {
                    "failure"
                }
                .to_string(),
                message: format!("Authentication attempt for user: {username}"),
            };
            self.audit_service.log_event(event).await;
        }

        Ok(result)
    }

    /// Check permission using token and Permission struct.
    /// When RBAC is disabled, always allows. Otherwise parses token as user ID.
    pub async fn check_permission(&self, token: &str, permission: &Permission) -> Result<()> {
        if !self.config.enable_rbac {
            return Ok(());
        }
        let user_id = Uuid::parse_str(token).unwrap_or(Uuid::nil());
        let allowed = self
            .check_permission_by_id(&user_id, &permission.resource, &permission.action)
            .await?;
        if allowed {
            Ok(())
        } else {
            Err(crate::error::MCPError::Authorization(format!(
                "Permission denied: {}:{}",
                permission.resource, permission.action
            )))
        }
    }

    /// Check permission by user ID (internal)
    pub async fn check_permission_by_id(
        &self,
        user_id: &Uuid,
        resource: &str,
        action: &str,
    ) -> Result<bool> {
        if !self.config.enable_rbac {
            return Ok(true);
        }

        let result = self
            .rbac_manager
            .check_permission(user_id, resource, action)
            .await?;

        if self.config.enable_audit {
            let event = AuditEvent {
                id: Uuid::new_v4(),
                timestamp: chrono::Utc::now(),
                event_type: "authorization".to_string(),
                user_id: Some(user_id.to_string()),
                resource_id: Some(resource.to_string()),
                action: action.to_string(),
                status: if result { "granted" } else { "denied" }.to_string(),
                message: format!(
                    "Permission check for user {user_id} on resource {resource} action {action}"
                ),
            };
            self.audit_service.log_event(event).await;
        }

        Ok(result)
    }

    /// Encrypt data
    pub fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        if !self.config.enable_encryption {
            return Ok(data.to_vec());
        }

        self.crypto_provider.encrypt(data)
    }

    /// Decrypt data
    pub fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        if !self.config.enable_encryption {
            return Ok(data.to_vec());
        }

        self.crypto_provider.decrypt(data)
    }

    /// Get security configuration
    pub const fn get_config(&self) -> &SecurityConfig {
        &self.config
    }

    /// Update security configuration
    pub fn update_config(&mut self, config: SecurityConfig) -> Result<()> {
        self.config = config;
        Ok(())
    }
}

impl Default for SecurityManagerImpl {
    fn default() -> Self {
        Self::new(SecurityConfig::default())
    }
}

impl Clone for SecurityManagerImpl {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            audit_service: self.audit_service.clone(),
            crypto_provider: self.crypto_provider.clone(),
            identity_manager: self.identity_manager.clone(),
            key_storage: self.key_storage.clone(),
            rbac_manager: self.rbac_manager.clone(),
            token_manager: self.token_manager.clone(),
        }
    }
}
