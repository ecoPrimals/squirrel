//! Standalone security tests that don't rely on other modules
use crate::mcp::security::{
    RBACManager,
    Role,
    Permission,
    Action,
    SecurityManager,
    SecurityConfig,
    Credentials,
};
use crate::mcp::types::{SecurityLevel, EncryptionFormat};
use std::collections::HashSet;

#[test]
fn test_rbac_manager_basic() {
    // Create a basic RBAC manager
    let rbac = RBACManager::new();
    
    // Verify it starts empty
    assert!(rbac.roles.is_empty());
}

#[test]
fn test_permission_creation() {
    // Create a simple permission
    let permission = Permission {
        id: "test-perm".to_string(),
        name: "Test".to_string(),
        resource: "Document".to_string(),
        action: Action::Read,
    };
    
    // Verify permission properties
    assert_eq!(permission.id, "test-perm");
    assert_eq!(permission.name, "Test");
    assert_eq!(permission.resource, "Document");
    assert_eq!(permission.action, Action::Read);
}

#[test]
fn test_security_config() {
    // Create a security configuration
    let config = SecurityConfig {
        min_security_level: SecurityLevel::Medium,
        encryption_format: EncryptionFormat::Aes256Gcm,
        token_validity: 3600,
        max_auth_attempts: 5,
        default_roles: Vec::new(),
    };
    
    // Verify config properties
    assert_eq!(config.min_security_level, SecurityLevel::Medium);
    assert_eq!(config.encryption_format, EncryptionFormat::Aes256Gcm);
    assert_eq!(config.token_validity, 3600);
    assert_eq!(config.max_auth_attempts, 5);
}

#[test]
fn test_credentials_validation() {
    // Create credentials
    let credentials = Credentials {
        client_id: "test-client".to_string(),
        client_secret: "test-secret".to_string(),
        security_level: SecurityLevel::Medium,
        requested_roles: None,
    };
    
    // Verify credentials properties
    assert_eq!(credentials.client_id, "test-client");
    assert_eq!(credentials.client_secret, "test-secret");
    assert_eq!(credentials.security_level, SecurityLevel::Medium);
    assert!(credentials.requested_roles.is_none());
} 