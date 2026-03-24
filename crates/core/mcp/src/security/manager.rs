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
    #[must_use]
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
    #[must_use]
    pub fn audit_service(&self) -> Arc<DefaultAuditService> {
        self.audit_service.clone()
    }

    /// Get crypto provider
    #[must_use]
    pub fn crypto_provider(&self) -> Arc<DefaultCryptoProvider> {
        self.crypto_provider.clone()
    }

    /// Get identity manager
    #[must_use]
    pub fn identity_manager(&self) -> Arc<DefaultIdentityManager> {
        self.identity_manager.clone()
    }

    /// Get key storage
    #[must_use]
    pub fn key_storage(&self) -> Arc<InMemoryKeyStorage> {
        self.key_storage.clone()
    }

    /// Get RBAC manager
    #[must_use]
    pub fn rbac_manager(&self) -> Arc<BasicRBACManager> {
        self.rbac_manager.clone()
    }

    /// Get token manager
    #[must_use]
    pub fn token_manager(&self) -> Arc<DefaultTokenManager> {
        self.token_manager.clone()
    }

    /// Initialize security manager (stub — will delegate to BearDog)
    #[allow(
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
    #[must_use]
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::MCPError;

    fn quiet_config() -> SecurityConfig {
        SecurityConfig {
            enable_audit: false,
            enable_rbac: false,
            enable_encryption: false,
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn new_default_clone_and_accessors() {
        let m = SecurityManagerImpl::new(quiet_config());
        let dbg = format!("{m:?}");
        assert!(dbg.contains("SecurityManagerImpl"));

        assert!(m.audit_service().get_events().await.is_empty());
        let _ = m.crypto_provider();
        let _ = m.identity_manager();
        let _ = m.key_storage();
        let _ = m.rbac_manager();
        let _ = m.token_manager();

        let c = SecurityManagerImpl::default();
        assert!(c.get_config().enable_audit);

        let cloned = m.clone();
        assert_eq!(
            cloned.get_config().enable_audit,
            m.get_config().enable_audit
        );
    }

    #[test]
    fn initialize_and_update_config() {
        let mut m = SecurityManagerImpl::new(quiet_config());
        assert!(m.initialize().is_ok());
        let mut next = quiet_config();
        next.enable_audit = true;
        assert!(m.update_config(next).is_ok());
        assert!(m.get_config().enable_audit);
    }

    #[tokio::test]
    async fn authenticate_audit_paths() {
        let mut cfg = quiet_config();
        cfg.enable_audit = true;
        let m = SecurityManagerImpl::new(cfg);
        let id = m
            .identity_manager()
            .create_identity("bob".to_string(), None)
            .await
            .expect("create_identity");

        let ok = m.authenticate("bob", "pw").await.expect("authenticate");
        assert_eq!(ok.expect("bob session").id, id.id);
        let events = m.audit_service().get_events().await;
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].status, "success");

        m.audit_service().clear_events().await;
        let fail = m.authenticate("nobody", "pw").await.expect("authenticate");
        assert!(fail.is_none());
        let events = m.audit_service().get_events().await;
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].status, "failure");
    }

    #[tokio::test]
    async fn authenticate_skips_audit_when_disabled() {
        let m = SecurityManagerImpl::new(quiet_config());
        m.identity_manager()
            .create_identity("c".to_string(), None)
            .await
            .expect("create_identity");
        m.authenticate("c", "x")
            .await
            .expect("authenticate")
            .expect("c session");
        assert!(m.audit_service().get_events().await.is_empty());
    }

    #[tokio::test]
    async fn check_permission_rbac_disabled_always_ok() {
        let m = SecurityManagerImpl::new(quiet_config());
        let p = Permission::new("r".to_string(), "a".to_string());
        assert!(m.check_permission("not-a-uuid", &p).await.is_ok());
        assert!(
            m.check_permission_by_id(&Uuid::new_v4(), "r", "a")
                .await
                .expect("check_permission_by_id")
        );
    }

    #[tokio::test]
    async fn check_permission_rbac_denied_and_granted() {
        let mut cfg = quiet_config();
        cfg.enable_rbac = true;
        cfg.enable_audit = false;
        let m = SecurityManagerImpl::new(cfg);

        let user = Uuid::new_v4();
        let role = m
            .rbac_manager()
            .create_role("role".to_string(), String::new())
            .await
            .expect("create_role");
        m.rbac_manager()
            .add_permission_to_role(
                &role.id,
                Permission::new("res".to_string(), "act".to_string()),
            )
            .await
            .expect("add_permission_to_role");
        m.rbac_manager()
            .assign_role_to_user(&user, &role.id, &Uuid::new_v4())
            .await
            .expect("assign_role_to_user");

        assert!(
            m.check_permission_by_id(&user, "res", "act")
                .await
                .expect("check_permission_by_id")
        );

        let err = m
            .check_permission(
                &user.to_string(),
                &Permission::new("x".to_string(), "y".to_string()),
            )
            .await
            .unwrap_err();
        assert!(matches!(err, MCPError::Authorization(_)));

        assert!(
            m.check_permission(
                &user.to_string(),
                &Permission::new("res".to_string(), "act".to_string())
            )
            .await
            .is_ok()
        );
    }

    #[tokio::test]
    async fn check_permission_by_id_audit_when_enabled() {
        let mut cfg = quiet_config();
        cfg.enable_rbac = true;
        cfg.enable_audit = true;
        let m = SecurityManagerImpl::new(cfg);
        let id = Uuid::new_v4();
        m.check_permission_by_id(&id, "any", "op")
            .await
            .expect("check_permission_by_id");
        let events = m.audit_service().get_events().await;
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].status, "denied");
    }

    #[test]
    fn encrypt_decrypt_pass_through_when_disabled() {
        let m = SecurityManagerImpl::new(quiet_config());
        let data = [7_u8, 8, 9];
        assert_eq!(m.encrypt(&data).expect("encrypt"), data);
        assert_eq!(m.decrypt(&data).expect("decrypt"), data);
    }

    #[test]
    fn encrypt_decrypt_round_trip_when_enabled() {
        let mut cfg = quiet_config();
        cfg.enable_encryption = true;
        let m = SecurityManagerImpl::new(cfg);
        let plain = b"secret payload";
        let ct = m.encrypt(plain).expect("encrypt");
        assert_ne!(ct.as_slice(), plain);
        assert_eq!(m.decrypt(&ct).expect("decrypt"), plain);
    }
}
