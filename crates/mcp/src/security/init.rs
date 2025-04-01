//! Security subsystem initialization module.
//!
//! This module provides functions to initialize the security subsystem
//! with various configurations of the RBAC manager.

use std::sync::Arc;

use crate::security::manager::{SecurityManager, SecurityManagerImpl};
use crate::security::crypto::CryptoProvider;
use crate::security::token::TokenManager;
use crate::security::identity::IdentityManager;
use crate::security::audit::AuditService;
use crate::security::rbac::{RBACManager, BasicRBACManager, MockRBACManager};

/// Initialize the security manager with a basic RBAC manager.
///
/// This function creates a new security manager with a basic RBAC implementation
/// that provides core functionality without advanced features.
///
/// # Arguments
/// * `crypto_provider` - The crypto provider to use
/// * `token_manager` - The token manager to use
/// * `identity_manager` - The identity manager to use
/// * `audit_service` - The audit service to use
///
/// # Returns
/// A new security manager with a basic RBAC implementation
pub fn init_with_basic_rbac(
    crypto_provider: Arc<dyn CryptoProvider>,
    token_manager: Arc<dyn TokenManager>,
    identity_manager: Arc<dyn IdentityManager>,
    audit_service: Arc<dyn AuditService>,
) -> Arc<dyn SecurityManager> {
    let rbac_manager = Arc::new(BasicRBACManager::new());
    
    Arc::new(SecurityManagerImpl::new(
        crypto_provider,
        token_manager,
        identity_manager,
        rbac_manager,
        audit_service,
    ))
}

/// Initialize the security manager with a mock RBAC manager for testing.
///
/// This function creates a new security manager with a mock RBAC implementation
/// that allows configuring permissions for testing purposes.
///
/// # Arguments
/// * `crypto_provider` - The crypto provider to use
/// * `token_manager` - The token manager to use
/// * `identity_manager` - The identity manager to use
/// * `audit_service` - The audit service to use
/// * `allow_all` - Whether to allow all permission checks by default
///
/// # Returns
/// A new security manager with a mock RBAC implementation
pub fn init_with_mock_rbac(
    crypto_provider: Arc<dyn CryptoProvider>,
    token_manager: Arc<dyn TokenManager>,
    identity_manager: Arc<dyn IdentityManager>,
    audit_service: Arc<dyn AuditService>,
    allow_all: bool,
) -> Arc<dyn SecurityManager> {
    let rbac_manager = Arc::new(MockRBACManager::new(allow_all));
    
    Arc::new(SecurityManagerImpl::new(
        crypto_provider,
        token_manager,
        identity_manager,
        rbac_manager,
        audit_service,
    ))
}

/// Initialize the security manager with a custom RBAC manager.
///
/// This function creates a new security manager with a custom RBAC implementation
/// provided by the caller.
///
/// # Arguments
/// * `crypto_provider` - The crypto provider to use
/// * `token_manager` - The token manager to use
/// * `identity_manager` - The identity manager to use
/// * `rbac_manager` - The custom RBAC manager to use
/// * `audit_service` - The audit service to use
///
/// # Returns
/// A new security manager with the provided RBAC implementation
pub fn init_with_custom_rbac(
    crypto_provider: Arc<dyn CryptoProvider>,
    token_manager: Arc<dyn TokenManager>,
    identity_manager: Arc<dyn IdentityManager>,
    rbac_manager: Arc<dyn RBACManager>,
    audit_service: Arc<dyn AuditService>,
) -> Arc<dyn SecurityManager> {
    Arc::new(SecurityManagerImpl::new(
        crypto_provider,
        token_manager,
        identity_manager,
        rbac_manager,
        audit_service,
    ))
} 