use std::sync::Arc;
use std::collections::HashSet;

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
use crate::mcp::Result;
use crate::test_utils::TestData;
use crate::test_utils::security as security_utils;

// Import test modules
mod rbac_tests;
mod role_test;
mod security_test;

// Helper functions for test setup

/// Creates a test RBAC manager
fn create_test_rbac_manager() -> RBACManager {
    security_utils::create_test_rbac_manager()
}

/// Creates a test permission for use in tests
fn create_test_permission(name: &str, resource: &str, action: Action) -> Permission {
    Permission {
        id: format!("perm-{}-{}", resource, name),
        name: name.to_string(),
        resource: resource.to_string(),
        action,
    }
}

/// Creates a test security configuration
fn create_test_security_config() -> SecurityConfig {
    security_utils::create_test_security_config()
}

/// Creates test credentials for authentication
fn create_test_credentials(client_id: &str, secret: &str) -> Credentials {
    security_utils::create_test_credentials(client_id, secret)
} 