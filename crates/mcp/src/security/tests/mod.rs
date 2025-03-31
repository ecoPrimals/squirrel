use std::sync::Arc;
use std::collections::HashSet;

use crate::security::{
    // Remove old RBAC types
    RBACManager,
    BasicRBACManager,
    Action,
    Resource,
    SecurityManager,
    Credentials,
};
use crate::types::{SecurityLevel, EncryptionFormat};
use crate::error::Result;

// Import test modules
mod rbac_tests;
mod role_test;
mod security_test;
mod integration_test;
mod performance_benchmark;

// Integration tests for the security subsystem
mod rbac_integration;
mod auth_tests;

// Add more test modules as needed

// Helper functions for test setup

/// Creates a test RBAC manager
fn create_test_rbac_manager() -> Arc<dyn RBACManager> {
    Arc::new(BasicRBACManager::new())
}

/// Creates test credentials for authentication
fn create_test_credentials(username: &str, password: &str) -> Credentials {
    Credentials {
        username: username.to_string(),
        password: password.to_string(),
        additional_factors: None,
    }
} 