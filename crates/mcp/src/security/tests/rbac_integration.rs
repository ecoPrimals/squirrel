use std::sync::Arc;
use tokio::test;

use crate::error::Result;
use crate::security::{
    initialize_security_manager,
    init_with_basic_rbac,
    init_with_mock_rbac,
    SecurityManager,
    BasicRBACManager,
    MockRBACManager,
    Credentials,
    Action,
    Resource,
};
use crate::config::SecurityConfig;
use crate::security::audit::DefaultAuditService;
use crate::security::crypto::DefaultCryptoProvider;
use crate::security::identity::DefaultIdentityManager;
use crate::security::token::DefaultTokenManager;
use crate::security::key_storage::InMemoryKeyStorage;

#[test]
async fn test_security_manager_with_basic_rbac() -> Result<()> {
    // Create components
    let key_storage = Arc::new(InMemoryKeyStorage::new());
    let identity_manager = Arc::new(DefaultIdentityManager::new());
    let crypto_provider = Arc::new(DefaultCryptoProvider::new());
    let token_manager = Arc::new(DefaultTokenManager::new(
        key_storage.clone(),
        crypto_provider.clone(),
    ));
    let audit_service = Arc::new(DefaultAuditService::new());
    
    // Create a basic RBAC manager and add a role with permissions
    let rbac_manager = Arc::new(BasicRBACManager::new());
    
    // Create the security manager
    let security_manager = init_with_basic_rbac(
        crypto_provider,
        token_manager,
        identity_manager,
        audit_service,
    );
    
    // Test authentication
    let credentials = Credentials {
        username: "test_user".to_string(),
        password: "password123".to_string(),
        additional_factors: None,
    };
    
    let token = security_manager.authenticate(&credentials).await?;
    
    // Test authorization (should fail without permissions)
    let resource = Resource {
        id: "document123".to_string(),
        security_level: Default::default(),
    };
    let action = Action::new("read");
    
    let auth_result = security_manager.authorize(&token, &resource, &action, None).await;
    assert!(auth_result.is_err(), "Authorization should fail without permissions");
    
    Ok(())
}

#[test]
async fn test_security_manager_with_mock_rbac() -> Result<()> {
    // Create components
    let key_storage = Arc::new(InMemoryKeyStorage::new());
    let identity_manager = Arc::new(DefaultIdentityManager::new());
    let crypto_provider = Arc::new(DefaultCryptoProvider::new());
    let token_manager = Arc::new(DefaultTokenManager::new(
        key_storage.clone(),
        crypto_provider.clone(),
    ));
    let audit_service = Arc::new(DefaultAuditService::new());
    
    // Create the security manager with mock RBAC that allows all permissions
    let security_manager = init_with_mock_rbac(
        crypto_provider,
        token_manager,
        identity_manager,
        audit_service,
        true, // allow_all = true
    );
    
    // Test authentication
    let credentials = Credentials {
        username: "test_user".to_string(),
        password: "password123".to_string(),
        additional_factors: None,
    };
    
    let token = security_manager.authenticate(&credentials).await?;
    
    // Test authorization (should succeed with mock RBAC)
    let resource = Resource {
        id: "document123".to_string(),
        security_level: Default::default(),
    };
    let action = Action::new("read");
    
    let auth_result = security_manager.authorize(&token, &resource, &action, None).await;
    assert!(auth_result.is_ok(), "Authorization should succeed with mock RBAC");
    
    Ok(())
}

#[test]
async fn test_initialization_methods() -> Result<()> {
    // Test the standard initialization method
    let config = SecurityConfig {
        encryption_default_format: "AES256GCM".to_string(),
    };
    
    let security_manager = initialize_security_manager(config);
    
    // Test that the security manager was created successfully
    assert_eq!(security_manager.version(), env!("CARGO_PKG_VERSION"));
    
    Ok(())
} 