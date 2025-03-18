use std::collections::HashSet;
use std::sync::Arc;

use tokio::test;
use serde_json::json;

use crate::mcp::security::{
    SecurityManager, 
    SecurityConfig, 
    Credentials,
    rbac::{Role, Permission, Action}
};
use crate::mcp::types::SecurityLevel;
use crate::test_utils::TestFactory;

#[test]
async fn test_authentication() {
    let security = SecurityManager::new(SecurityConfig::default()).await.unwrap();
    let credentials = Credentials {
        client_id: "test_user".to_string(),
        client_secret: "secret".to_string(),
        security_level: SecurityLevel::Standard,
        requested_roles: None,
    };

    let token = security.authenticate(&credentials).await.unwrap();
    assert!(!token.is_empty());
}

#[test]
async fn test_authorization() {
    let security = SecurityManager::new(SecurityConfig::default()).await.unwrap();
    let credentials = Credentials {
        client_id: "test_user".to_string(),
        client_secret: "secret".to_string(),
        security_level: SecurityLevel::High,
        requested_roles: None,
    };

    let token = security.authenticate(&credentials).await.unwrap();
    assert!(security.authorize(&token, SecurityLevel::Standard, None).await.is_ok());
    assert!(security.authorize(&token, SecurityLevel::High, None).await.is_ok());
    assert!(security.authorize(&token, SecurityLevel::Maximum, None).await.is_err());
}

#[test]
async fn test_encryption() {
    let security = SecurityManager::new(SecurityConfig::default()).await.unwrap();
    let credentials = Credentials {
        client_id: "test_user".to_string(),
        client_secret: "secret".to_string(),
        security_level: SecurityLevel::Standard,
        requested_roles: None,
    };

    let token = security.authenticate(&credentials).await.unwrap();
    let session = security.authorize(&token, SecurityLevel::Standard, None).await.unwrap();

    let data = b"test data";
    let encrypted = security.encrypt(&session.id, data).await.unwrap();
    let decrypted = security.decrypt(&session.id, &encrypted).await.unwrap();
    assert_eq!(data.to_vec(), decrypted);
}

#[test]
async fn test_rbac_integration() {
    let mut config = SecurityConfig::default();
    
    // Create default user role with read permission
    let mut user_permissions = HashSet::new();
    let read_permission = Permission {
        id: "read-doc-1".to_string(),
        name: "read".to_string(),
        resource: "document".to_string(),
        action: Action::Read,
    };
    user_permissions.insert(read_permission.clone());
    
    let user_role = Role {
        id: "user-role-1".to_string(),
        name: "user".to_string(),
        description: Some("Basic user".to_string()),
        permissions: user_permissions,
        parent_roles: HashSet::new(),
    };
    
    config.default_roles.push(user_role.clone());
    let security = SecurityManager::new(config).await.unwrap();
    
    // Authenticate with role request
    let credentials = Credentials {
        client_id: "test_user".to_string(),
        client_secret: "secret".to_string(),
        security_level: SecurityLevel::Standard,
        requested_roles: Some(vec![user_role.id.clone()]),
    };
    
    // Use the token to ensure it's not marked as unused
    let _token = security.authenticate(&credentials).await.unwrap();
    
    // Check permissions
    assert!(security.has_permission(&credentials.client_id, &read_permission).await);
}

#[test]
async fn test_authentication_with_roles() {
    let mut config = SecurityConfig::default();
    
    // Create roles with specific permissions
    let mut read_permissions = HashSet::new();
    read_permissions.insert(Permission {
        id: "read-1".to_string(),
        name: "read".to_string(),
        resource: "data".to_string(),
        action: Action::Read,
    });
    
    let reader_role = Role {
        id: "reader-1".to_string(),
        name: "reader".to_string(),
        description: Some("Can read data".to_string()),
        permissions: read_permissions,
        parent_roles: HashSet::new(),
    };
    
    let mut write_permissions = HashSet::new();
    write_permissions.insert(Permission {
        id: "write-1".to_string(),
        name: "write".to_string(),
        resource: "data".to_string(),
        action: Action::Write,
    });
    
    let writer_role = Role {
        id: "writer-1".to_string(),
        name: "writer".to_string(),
        description: Some("Can write data".to_string()),
        permissions: write_permissions,
        parent_roles: HashSet::new(),
    };
    
    config.default_roles.push(reader_role.clone());
    config.default_roles.push(writer_role.clone());
    
    let security = Arc::new(SecurityManager::new(config).await.unwrap());
    
    // Authenticate with multiple roles
    let credentials = Credentials {
        client_id: "test_user".to_string(),
        client_secret: "secret".to_string(),
        security_level: SecurityLevel::Standard,
        requested_roles: Some(vec![reader_role.id.clone(), writer_role.id.clone()]),
    };
    
    security.authenticate(&credentials).await.unwrap();
    
    // Check for read permission
    assert!(security.has_permission(
        &credentials.client_id,
        &Permission {
            id: "read-1".to_string(),
            name: "read".to_string(),
            resource: "data".to_string(),
            action: Action::Read,
        }
    ).await);
    
    // Check for write permission
    assert!(security.has_permission(
        &credentials.client_id,
        &Permission {
            id: "write-1".to_string(),
            name: "write".to_string(),
            resource: "data".to_string(),
            action: Action::Write,
        }
    ).await);
} 