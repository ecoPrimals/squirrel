use std::sync::Arc;
use std::collections::HashSet;

use crate::security::{
    EnhancedRBACManager,
    Role,
    Permission,
    Action,
    SecurityManager,
    SecurityManagerImpl,
    Credentials,
    PermissionScope,
};
use crate::types::{SecurityLevel, EncryptionFormat};
use crate::error::Result;

// Import test modules
mod rbac_tests;
mod role_test;
mod security_test;
mod integration_test;
mod performance_benchmark;

// Helper functions for test setup

/// Creates a test RBAC manager
fn create_test_rbac_manager() -> Arc<EnhancedRBACManager> {
    Arc::new(EnhancedRBACManager::new())
}

/// Creates a test permission for use in tests
fn create_test_permission(name: &str, resource: &str, action: Action) -> Permission {
    Permission {
        id: format!("perm-{}-{}", resource, name),
        name: name.to_string(),
        resource: resource.to_string(),
        action,
        resource_id: None,
        scope: PermissionScope::All,
        conditions: Vec::new(),
    }
}

/// Creates a test security manager
fn create_test_security_manager() -> impl SecurityManager {
    SecurityManagerImpl::new()
}

/// Creates test credentials for authentication
fn create_test_credentials(username: &str, password: &str) -> Credentials {
    Credentials {
        username: username.to_string(),
        password: Some(password.to_string()),
        token: None,
    }
} 