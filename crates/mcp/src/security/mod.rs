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

// Import the Arc type for the initialize_security_manager function
use std::sync::Arc;

// --- Public Re-exports ---
// Re-export the key types and traits that form the public API of this security module.

// Re-export all common types from the central types module
pub use self::types::{
    Action,
    Resource,
    UserId,
    Token,
    AuthCredentials,
    EncryptionFormat,
    EncryptionInfo,
    SecurityMetadata,
    SecurityLevel,
    RoleId,
    //EntityId,  // Comment out these types that are causing issues
    //UserRole,  // Comment out these types that are causing issues
};

// RBAC specific types from the unified module
pub use self::rbac::{
    RBACManager,
    BasicRBACManager,
    MockRBACManager,
    RBACManagerImpl,
    Permission,
    // Add re-exports of the needed types
    RoleDefinition, 
    PermissionDefinition,
    RolePermission,
    RoleDetailsResponse,
};

// Explicitly re-export Permission validation types from the rbac module
pub use self::rbac::permission_validation::{
    ValidationResult,
    ValidationRule,
    ValidationAuditRecord,
};

// Also export the module for future imports
pub mod interfaces {
    pub use super::rbac::unified::*;
}

// Make the unified module's contents public 
// pub use self::rbac::unified;

// Create a module to hold UnifiedSecurity implementations with public visibility
pub mod unified {
    // Re-export security implementation for external use
    pub use super::rbac::manager::RBACManagerImpl as UnifiedSecurity;
}

// User and authentication types
pub use self::token::{SessionToken, AuthToken};

// Re-export key traits and implementations
pub use self::audit::AuditService;
pub use self::audit::DefaultAuditService;

pub use self::auth::AuthManager;
pub use self::auth::DefaultAuthManager;
pub use self::auth::SecurityContext;
pub use self::auth::UserContext;

pub use self::crypto::{CryptoProvider, DefaultCryptoProvider, KeyPurpose, KeyStorage};
pub use self::crypto::encrypt;
pub use self::crypto::decrypt;
pub use self::crypto::generate_key;

pub use self::identity::IdentityManager;
pub use self::identity::DefaultIdentityManager;

// Make the manager re-exports explicit
pub use self::manager::{SecurityManager, SecurityManagerImpl, TypedSecurityManager, CombinedSecurityManager};

// Re-export keysystem components
pub use self::key_storage::{KeyStorage, InMemoryKeyStorage};
pub use self::token::{TokenManager, DefaultTokenManager};

// Ensure we're re-exporting RBACError for correct error handling
pub use crate::error::RBACError;

// Re-export initialization functions
pub use self::init::{
    init_with_basic_rbac,
    init_with_mock_rbac,
    init_with_custom_rbac,
};

pub use self::traits::{ResourceTrait, ActionTrait, make_permission_string};

/// Initialize the security subsystem with the provided configuration.
/// Returns a new instance of the security manager.
pub fn initialize_security_manager(_config: crate::config::SecurityConfig) -> Arc<dyn SecurityManager> {
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

// Module documentation in separate files
/// Documentation from the README.md file
#[doc = include_str!("README.md")]
pub struct SecurityDocumentation;

/// Maintenance documentation
#[doc = include_str!("MAINTENANCE.md")]
pub struct SecurityMaintenance;

// The following re-exports are duplicates of those at the top of the file
// They have been commented out to avoid E0252 errors (duplicated exports)
/*
pub use self::auth::{SecurityContext};
pub use self::identity::{UserId};
pub use self::rbac::{Permission, Action, Resource};
pub use self::token::{Token, AuthCredentials, SessionToken, AuthToken};
pub use self::crypto::{CryptoProvider, DefaultCryptoProvider, KeyPurpose, KeyStorage, DefaultKeyStorage};
pub use self::manager::{SecurityManager, SecurityManagerImpl, TypedSecurityManager, CombinedSecurityManager};
pub use self::types::{SecurityLevel, EncryptionFormat};
pub use self::identity::{IdentityManager, DefaultIdentityManager};
*/
