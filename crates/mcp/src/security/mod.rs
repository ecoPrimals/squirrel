//! Centralized security management for MCP.

// Module declarations - make all modules public to avoid private type issues
pub mod audit;
pub mod auth;
pub mod crypto;
pub mod identity;
pub mod key_storage;
pub mod manager;
pub mod rbac;
pub mod token;
pub mod types;
pub mod init;
pub mod traits;

// --- Public Re-exports ---
// Re-export the key types and traits that form the public API of this security module.

// Re-export all common types from the central types module
pub use self::types::{
    Action,
    Resource,
    UserId,
    Token,
    Credentials,
    EncryptionFormat,
    EncryptionInfo,
    SecurityMetadata,
    SecurityLevel,
};

// RBAC specific types from the unified module
pub use self::rbac::{
    RBACManager,
    BasicRBACManager,
    MockRBACManager,
    RBACManagerImpl,
    RoleDefinition,
    PermissionDefinition,
    RolePermission,
    RoleDetailsResponse,
};

// User and authentication types
pub use self::token::{SessionToken, AuthToken};

// Re-export key traits and implementations
pub use self::audit::AuditService;
pub use self::audit::DefaultAuditService;

pub use self::auth::AuthManager;
pub use self::auth::DefaultAuthManager;
pub use self::auth::SecurityContext;
pub use self::auth::UserContext;

pub use self::crypto::CryptoProvider;
pub use self::crypto::DefaultCryptoProvider;
pub use self::crypto::encrypt;
pub use self::crypto::decrypt;
pub use self::crypto::generate_key;

pub use self::identity::IdentityManager;
pub use self::identity::DefaultIdentityManager;

pub use self::key_storage::KeyStorage;
pub use self::key_storage::KeyPurpose;
pub use self::key_storage::InMemoryKeyStorage;

pub use self::manager::SecurityManager;
pub use self::manager::SecurityManagerImpl;

// Re-export initialization functions
pub use self::init::{
    init_with_basic_rbac,
    init_with_mock_rbac,
    init_with_custom_rbac,
};

pub use self::traits::{ResourceTrait, ActionTrait, make_permission_string};

use std::sync::Arc;

/// Initialize the security subsystem with the provided configuration.
/// Returns a new instance of the security manager.
pub fn initialize_security_manager(config: crate::config::SecurityConfig) -> Arc<dyn SecurityManager> {
    // Create implementations
    let key_storage = Arc::new(key_storage::InMemoryKeyStorage::new());
    let identity_manager = Arc::new(identity::DefaultIdentityManager::new());
    let crypto_provider = Arc::new(crypto::DefaultCryptoProvider::new());
    let audit_service = Arc::new(audit::DefaultAuditService::new());
    
    // Use the basic RBAC manager for standard initialization
    init::init_with_basic_rbac(
        crypto_provider.clone(),
        Arc::new(token::DefaultTokenManager::new(
            key_storage.clone(),
            crypto_provider.clone(),
        )),
        identity_manager,
        audit_service,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_initialize_security_manager() {
        // Create a minimal security config for testing
        let config = crate::config::SecurityConfig {
            encryption_default_format: "AES256GCM".to_string(),
        };
        
        // Initialize the security manager
        let _security_manager = initialize_security_manager(config);
    }
}

// Documentation
#[doc = include_str!("README.md")]
pub struct SecurityDocumentation;

#[doc = include_str!("MAINTENANCE.md")]
pub struct SecurityMaintenance;
