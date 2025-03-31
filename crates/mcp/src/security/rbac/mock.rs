//! Mock RBAC Manager implementation for testing.
//!
//! This module provides a simple mock implementation of the `RBACManager` trait
//! that can be used for testing. It allows configuring the behavior of the mock
//! to return specific values for permission checks, role assignments, etc.

use tokio::sync::RwLock;
use std::collections::{HashMap, HashSet};
use async_trait::async_trait;

use crate::error::Result;
use crate::context_manager::Context;
use super::unified::RBACManager;

/// Mock RBAC Manager for testing.
///
/// This implementation provides a configurable mock that can be used for
/// testing security-related components that depend on the RBAC system.
#[derive(Debug)]
pub struct MockRBACManager {
    /// Whether to allow all permission checks
    allow_all: bool,
    /// Optional user roles to return for specific users
    user_roles: RwLock<HashMap<String, HashSet<String>>>,
}

impl MockRBACManager {
    /// Create a new mock RBAC manager.
    ///
    /// # Arguments
    /// * `allow_all` - Whether to allow all permission checks by default
    ///
    /// # Returns
    /// A new `MockRBACManager` instance with the specified configuration.
    pub fn new(allow_all: bool) -> Self {
        Self {
            allow_all,
            user_roles: RwLock::new(HashMap::new()),
        }
    }
    
    /// Configure the mock to return specific roles for a user.
    ///
    /// # Arguments
    /// * `user_id` - The ID of the user to configure roles for
    /// * `roles` - The roles to assign to the user
    ///
    /// # Returns
    /// A reference to the updated mock for method chaining.
    pub async fn with_user_roles(mut self, user_id: &str, roles: Vec<String>) -> Self {
        {
            let mut user_roles = self.user_roles.write().await;
            let role_set = HashSet::from_iter(roles.into_iter());
            user_roles.insert(user_id.to_string(), role_set);
        } // ensure the write guard is dropped here
        
        self
    }
}

// Implement Clone for MockRBACManager
impl Clone for MockRBACManager {
    fn clone(&self) -> Self {
        // Create a new instance with the same allow_all setting
        let cloned = Self {
            allow_all: self.allow_all,
            user_roles: RwLock::new(HashMap::new()),
        };
        
        // We'll sync the user_roles in an async context where this is used
        cloned
    }
}

#[async_trait]
impl RBACManager for MockRBACManager {
    fn name(&self) -> &str {
        "MockRBACManager"
    }
    
    fn version(&self) -> &str {
        "1.0.0"
    }
    
    async fn has_permission(&self, _user_id: &str, _permission: &str, _context: Option<&Context>) -> Result<bool> {
        Ok(self.allow_all)
    }
    
    async fn assign_role(&self, user_id: &str, role_id: &str) -> Result<()> {
        let mut user_roles = self.user_roles.write().await;
        user_roles
            .entry(user_id.to_string())
            .or_insert_with(HashSet::new)
            .insert(role_id.to_string());
        Ok(())
    }
    
    async fn revoke_role(&self, user_id: &str, role_id: &str) -> Result<()> {
        let mut user_roles = self.user_roles.write().await;
        if let Some(roles) = user_roles.get_mut(user_id) {
            roles.remove(role_id);
        }
        Ok(())
    }
    
    async fn get_user_roles(&self, user_id: &str) -> Result<Vec<String>> {
        let user_roles = self.user_roles.read().await;
        let roles = user_roles
            .get(user_id)
            .cloned()
            .unwrap_or_else(|| {
                if self.allow_all {
                    // Return a default role if allow_all is true
                    let mut set = HashSet::new();
                    set.insert("mock_role".to_string());
                    set
                } else {
                    HashSet::new()
                }
            });
        
        Ok(roles.into_iter().collect())
    }
    
    async fn has_role(&self, user_id: &str, role_id: &str) -> Result<bool> {
        let user_roles = self.user_roles.read().await;
        if let Some(roles) = user_roles.get(user_id) {
            Ok(roles.contains(role_id))
        } else {
            Ok(self.allow_all)
        }
    }
}

impl Default for MockRBACManager {
    fn default() -> Self {
        Self::new(true)
    }
} 