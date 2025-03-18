//! Test basic security functionality

use super::*;
use crate::mcp::security::{SecurityManager, SecurityConfig};

#[test]
fn test_security_config_creation() {
    // Create a basic security configuration
    let config = create_test_security_config();
    
    // Verify the config has expected values
    assert_eq!(config.min_security_level, SecurityLevel::Medium);
    assert_eq!(config.encryption_format, EncryptionFormat::Aes256Gcm);
    assert_eq!(config.token_validity, 3600);
    assert_eq!(config.max_auth_attempts, 5);
    assert!(config.default_roles.is_empty());
}

#[test]
fn test_credentials_creation() {
    // Create test credentials
    let credentials = create_test_credentials("test-client", "test-secret");
    
    // Verify credentials properties
    assert_eq!(credentials.client_id, "test-client");
    assert_eq!(credentials.client_secret, "test-secret");
    assert_eq!(credentials.security_level, SecurityLevel::Medium);
    assert!(credentials.requested_roles.is_none());
}

#[test]
fn test_permission_creation() {
    // Create a test permission
    let permission = create_test_permission("test", "resource", Action::Read);
    
    // Verify permission properties
    assert_eq!(permission.name, "test");
    assert_eq!(permission.resource, "resource");
    assert_eq!(permission.action, Action::Read);
    assert_eq!(permission.id, "perm-resource-test");
} 