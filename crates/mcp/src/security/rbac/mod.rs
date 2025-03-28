//! RBAC Module
//!
//! This module provides a comprehensive Role-Based Access Control system with:
//! - Fine-grained permission control
//! - Role inheritance (direct, filtered, conditional, and delegated)
//! - Permission validation with audit logging
//! - Complex authorization rules
//! - High-performance permission caching
//! - Optimized handling of large role hierarchies

use tracing::info;
use std::sync::Arc;
use std::collections::{HashMap, HashSet};
use chrono::{DateTime, Utc, Timelike};
use serde::{Serialize, Deserialize};
use std::time::Instant;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use lru::LruCache;
use tokio::sync::Mutex;
use std::num::NonZeroUsize;

use crate::error::{MCPError, Result, SecurityError};

// Re-export types from security::types
use crate::security::types::{
    Action, Permission, PermissionCondition, PermissionContext, 
    PermissionScope, Role,
};

// Private modules
/// Role inheritance graph implementation
mod role_inheritance;
/// Permission validation and condition checking
mod permission_validation;
/// Core RBAC manager implementation
mod manager;

#[cfg(test)]
mod tests;

// Re-export components for use throughout the application
pub use self::role_inheritance::{InheritanceManager, InheritanceType};
pub use self::permission_validation::{
    AsyncPermissionValidator, ValidationAuditRecord, ValidationResult, ValidationRule,
};
pub use self::manager::RBACManager;

/// RBAC-specific error types
#[derive(Debug, Clone, thiserror::Error)]
pub enum RBACError {
    /// Role already exists
    #[error("Role already exists: {0}")]
    RoleExists(String),
    
    /// Role not found
    #[error("Role not found: {0}")]
    RoleNotFound(String),
    
    /// Role dependency cycle detected
    #[error("Role dependency cycle detected: {0}")]
    DependencyCycle(String),
    
    /// Inheritance already exists
    #[error("Inheritance already exists: {0} -> {1}")]
    InheritanceExists(String, String),
    
    /// Validation rule already exists
    #[error("Validation rule already exists: {0}")]
    ValidationRuleExists(String),
    
    /// Validation rule not found
    #[error("Validation rule not found: {0}")]
    ValidationRuleNotFound(String),
    
    /// Validation error
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    /// Expression error
    #[error("Expression error: {0}")]
    ExpressionError(String),
    
    /// Inheritance error
    #[error("Inheritance error: {0}")]
    InheritanceError(String),
    
    /// General error
    #[error("General error: {0}")]
    General(String),
}

/// Permission audit event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionAuditEvent {
    /// Audit event ID
    pub id: String,
    
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    
    /// User ID
    pub user_id: String,
    
    /// Action performed
    pub action: String,
    
    /// Target resource
    pub resource: String,
    
    /// Result of the permission check
    pub result: String,
    
    /// Roles used
    pub roles: Vec<String>,
    
    /// Permissions evaluated
    pub permissions: Vec<String>,
    
    /// Context information
    pub context: HashMap<String, String>,
}

/// Enhanced RBAC Manager that supports role inheritance, permission validation, and audit logging.
///
/// Key features:
/// - Advanced role inheritance (direct, filtered, conditional, delegated)
/// - Permission validation with rules and context
/// - Comprehensive audit logging
/// - High-performance permission caching
/// - Efficient handling of large role hierarchies
/// - Thread-safe async operations
///
/// The permission caching system drastically improves performance for repeated permission checks
/// by storing results in an LRU cache. This is particularly beneficial when the same permissions
/// are checked frequently across the application.
///
/// Performance optimizations include:
/// - Cached permission checks for frequent patterns
/// - Parallel processing for large role hierarchies
/// - Efficient batch permission resolution
/// - Smart cache key generation based on context
pub struct EnhancedRBACManager {
    /// RBAC Manager
    rbac_manager: Arc<RBACManager>,
    
    /// Inheritance manager
    inheritance_manager: Arc<InheritanceManager>,
    
    /// Permission validator
    permission_validator: Arc<AsyncPermissionValidator>,
    
    /// Whether audit logging is enabled
    audit_enabled: bool,
    
    /// Permission check cache
    permission_cache: Arc<Mutex<LruCache<u64, ValidationResult>>>,
    
    /// Cache hit metrics
    cache_hits: Arc<Mutex<usize>>,
    
    /// Cache miss metrics
    cache_misses: Arc<Mutex<usize>>,
}

// Implement Clone for EnhancedRBACManager
impl Clone for EnhancedRBACManager {
    fn clone(&self) -> Self {
        Self {
            rbac_manager: self.rbac_manager.clone(),
            inheritance_manager: self.inheritance_manager.clone(),
            permission_validator: self.permission_validator.clone(),
            audit_enabled: self.audit_enabled,
            permission_cache: self.permission_cache.clone(),
            cache_hits: self.cache_hits.clone(),
            cache_misses: self.cache_misses.clone(),
        }
    }
}

impl Default for EnhancedRBACManager {
    fn default() -> Self {
        Self::new()
    }
}

impl EnhancedRBACManager {
    /// Create a new enhanced RBAC manager
    pub fn new() -> Self {
        Self {
            rbac_manager: Arc::new(RBACManager::new()),
            inheritance_manager: Arc::new(InheritanceManager::new()),
            permission_validator: Arc::new(AsyncPermissionValidator::new()),
            audit_enabled: true,
            permission_cache: Arc::new(Mutex::new(LruCache::new(NonZeroUsize::new(1000).unwrap()))), // Cache up to 1000 permission checks
            cache_hits: Arc::new(Mutex::new(0)),
            cache_misses: Arc::new(Mutex::new(0)),
        }
    }
    
    /// Create from existing RBAC manager
    pub fn from_existing(rbac_manager: RBACManager) -> Self {
        Self {
            rbac_manager: Arc::new(rbac_manager),
            inheritance_manager: Arc::new(InheritanceManager::new()),
            permission_validator: Arc::new(AsyncPermissionValidator::new()),
            audit_enabled: true,
            permission_cache: Arc::new(Mutex::new(LruCache::new(NonZeroUsize::new(1000).unwrap()))), // Cache up to 1000 permission checks
            cache_hits: Arc::new(Mutex::new(0)),
            cache_misses: Arc::new(Mutex::new(0)),
        }
    }
    
    /// Create a new role with enhanced inheritance
    pub async fn create_role(
        &self,
        name: String,
        description: Option<String>,
        permissions: HashSet<Permission>,
        parent_roles: HashSet<String>,
    ) -> Result<Role> {
        // First, create the role through the RBAC manager to get a valid ID
        let rbac_role = self.rbac_manager.create_role(&name, description.as_deref()).await?;
        
        // Create our enhanced Role struct with the same ID
        let role = Role {
            id: rbac_role.id.clone(),
            name: name.clone(),
            description: description.clone(),
            permissions: permissions.clone(),
            parent_roles: parent_roles.clone(),
            security_level: crate::types::SecurityLevel::Standard,
            can_delegate: false,
            managed_roles: HashSet::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        // Add permissions to the role
        for permission in &permissions {
            self.rbac_manager.add_permission_to_role(&role.id, permission.clone()).await?;
        }
        
        // Add role to inheritance manager
        self.inheritance_manager.add_role(&role.id).await?;
        
        // Add direct inheritance relationships
        for parent_id in parent_roles {
            self.inheritance_manager
                .add_direct_inheritance(&parent_id, &role.id)
                .await?;
        }
        
        Ok(role)
    }
    
    /// Create filtered inheritance relationship
    pub async fn create_filtered_inheritance(
        &self,
        parent_id: &str,
        child_id: &str,
        included_permissions: HashSet<String>,
        excluded_permissions: HashSet<String>,
    ) -> Result<()> {
        // Verify roles exist
        if self.rbac_manager.get_role(parent_id).await.is_err() {
            return Err(MCPError::Security(SecurityError::RBACError(
                format!("Role not found: {}", parent_id)
            )));
        }
        
        if self.rbac_manager.get_role(child_id).await.is_err() {
            return Err(MCPError::Security(SecurityError::RBACError(
                format!("Role not found: {}", child_id)
            )));
        }
        
        // Create filtered inheritance
        self.inheritance_manager
            .add_filtered_inheritance(
                parent_id,
                child_id,
                included_permissions,
                excluded_permissions,
            )
            .await
    }
    
    /// Create conditional inheritance relationship
    pub async fn create_conditional_inheritance(
        &self,
        parent_id: &str,
        child_id: &str,
        condition: String,
    ) -> Result<()> {
        // Verify roles exist
        if self.rbac_manager.get_role(parent_id).await.is_err() {
            return Err(MCPError::Security(SecurityError::RBACError(
                format!("Role not found: {}", parent_id)
            )));
        }
        
        if self.rbac_manager.get_role(child_id).await.is_err() {
            return Err(MCPError::Security(SecurityError::RBACError(
                format!("Role not found: {}", child_id)
            )));
        }
        
        // Create conditional inheritance
        self.inheritance_manager
            .add_conditional_inheritance(parent_id, child_id, condition)
            .await
    }
    
    /// Create delegated inheritance relationship
    pub async fn create_delegated_inheritance(
        &self,
        parent_id: &str,
        child_id: &str,
        delegator_id: String,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<()> {
        // Verify roles exist
        if self.rbac_manager.get_role(parent_id).await.is_err() {
            return Err(MCPError::Security(SecurityError::RBACError(
                format!("Role not found: {}", parent_id)
            )));
        }
        
        if self.rbac_manager.get_role(child_id).await.is_err() {
            return Err(MCPError::Security(SecurityError::RBACError(
                format!("Role not found: {}", child_id)
            )));
        }
        
        // Create delegated inheritance
        self.inheritance_manager
            .add_delegated_inheritance(parent_id, child_id, delegator_id, expires_at)
            .await
    }
    
    /// Assign role to user
    pub async fn assign_role(&self, user_id: String, role_id: String) -> Result<()> {
        self.rbac_manager.assign_role_to_user(&user_id, &role_id).await
    }
    
    /// Add validation rule
    pub async fn add_validation_rule(&self, rule: ValidationRule) -> Result<()> {
        self.permission_validator.add_rule(rule.clone()).await?;
        
        info!("Added validation rule: {}", rule.id);
        Ok(())
    }
    
    /// Check if user has permission (optimized for performance)
    pub async fn has_permission(
        &self,
        user_id: &str,
        resource: &str,
        action: Action,
        context: &PermissionContext,
    ) -> Result<ValidationResult> {
        let start_time = Instant::now();
        
        // Create a cache key by hashing relevant inputs
        let cache_key = self.create_permission_cache_key(user_id, resource, &action, context);
        
        // Check cache first
        {
            let mut cache = self.permission_cache.lock().await;
            if let Some(result) = cache.get(&cache_key) {
                // Update cache hit metrics
                let mut hits = self.cache_hits.lock().await;
                *hits += 1;
                
                // Skip audit logging for cached results to improve performance
                let elapsed = start_time.elapsed();
                info!("Permission check for {}/{}/{} (cached): {:?} in {:?}", 
                     user_id, resource, action, result, elapsed);
                
                return Ok(result.clone());
            }
        }
        
        // Update cache miss metrics
        {
            let mut misses = self.cache_misses.lock().await;
            *misses += 1;
        }
        
        // Batch processing: Get all roles and permissions in one go
        
        // 1. Get roles for user (optimize by storing a mapping of user->roles)
        let user_role_ids = self.rbac_manager.get_user_roles(user_id).await;
        
        // 2. Use parallel processing for roles if there are many
        let mut roles = Vec::new();
        
        if user_role_ids.len() > 10 {
            // Use parallel processing for many roles
            let rbac_manager = self.rbac_manager.clone();
            let role_ids: Vec<String> = user_role_ids.into_iter().collect();
            
            let role_futures: Vec<_> = role_ids.iter()
                .map(|role_id| {
                    let rbac = rbac_manager.clone();
                    let id = role_id.clone();
                    async move {
                        rbac.get_role(&id).await
                    }
                })
                .collect();
                
            let results = futures::future::join_all(role_futures).await;
            
            for result in results {
                if let Ok(role) = result {
                    roles.push(role);
                }
            }
        } else {
            // Use sequential processing for few roles
            for role_id in &user_role_ids {
                if let Ok(role) = self.rbac_manager.get_role(role_id).await {
                    roles.push(role);
                }
            }
        }
        
        // 3. Create role map once
        let role_map: HashMap<String, Role> = roles
            .iter()
            .map(|r| (r.id.clone(), r.clone()))
            .collect();
            
        // 4. Get all permissions (direct and inherited) efficiently
        let mut all_permissions = HashSet::new();
        
        // Add direct permissions
        for role in &roles {
            all_permissions.extend(role.permissions.clone());
        }
        
        // Add inherited permissions for each role (can be parallelized for many roles)
        if roles.len() > 10 {
            let inheritance_manager = self.inheritance_manager.clone();
            let inheritance_futures: Vec<_> = roles.iter()
                .map(|role| {
                    let im = inheritance_manager.clone();
                    let role_id = role.id.clone();
                    let rm = role_map.clone();
                    let ctx = context.clone();
                    async move {
                        im.get_inherited_permissions(&role_id, &rm, Some(&ctx)).await
                    }
                })
                .collect();
                
            let results = futures::future::join_all(inheritance_futures).await;
            
            for result in results {
                if let Ok(permissions) = result {
                    all_permissions.extend(permissions);
                }
            }
        } else {
            for role in &roles {
                if let Ok(inherited) = self.inheritance_manager
                    .get_inherited_permissions(&role.id, &role_map, Some(context))
                    .await {
                    all_permissions.extend(inherited);
                }
            }
        }
        
        // 5. Validate permission
        let result = self.permission_validator
            .validate(user_id, resource, action, &roles, &all_permissions, context)
            .await;
            
        // 6. Cache the result
        {
            let mut cache = self.permission_cache.lock().await;
            cache.put(cache_key, result.clone());
        }
        
        let elapsed = start_time.elapsed();
        info!("Permission check for {}/{}/{}: {:?} in {:?}", 
             user_id, resource, action, result, elapsed);
             
        Ok(result)
    }
    
    /// Create a cache key for permission checks
    fn create_permission_cache_key(
        &self, 
        user_id: &str, 
        resource: &str, 
        action: &Action, 
        context: &PermissionContext
    ) -> u64 {
        let mut hasher = DefaultHasher::new();
        
        // Hash critical elements
        user_id.hash(&mut hasher);
        resource.hash(&mut hasher);
        
        // Convert action to a string representation for hashing
        let action_str = format!("{:?}", action);
        action_str.hash(&mut hasher);
        
        // Hash important context elements that might affect permission
        context.security_level.hash(&mut hasher);
        
        // Only hash time to the nearest hour to avoid excessive cache misses
        if let Some(time) = &context.current_time {
            let hour = time.hour();
            hour.hash(&mut hasher);
        }
        
        // Hash resource owner if present (affects Own scope permissions)
        if let Some(owner) = &context.resource_owner_id {
            owner.hash(&mut hasher);
        }
        
        // Hash resource group if present (affects Group scope permissions)
        if let Some(group) = &context.resource_group_id {
            group.hash(&mut hasher);
        }
        
        // Return the computed hash
        hasher.finish()
    }
    
    /// Get cache statistics
    pub async fn get_cache_stats(&self) -> (usize, usize) {
        let hits = self.cache_hits.lock().await;
        let misses = self.cache_misses.lock().await;
        (*hits, *misses)
    }
    
    /// Clear the permission cache
    pub async fn clear_cache(&self) {
        let mut cache = self.permission_cache.lock().await;
        cache.clear();
    }
    
    /// Set cache capacity
    pub async fn set_cache_capacity(&self, capacity: usize) {
        let mut cache = self.permission_cache.lock().await;
        cache.resize(NonZeroUsize::new(capacity).unwrap_or(NonZeroUsize::new(1).unwrap()));
    }
    
    /// Enable audit logging
    pub fn enable_audit(&mut self, enabled: bool) {
        self.audit_enabled = enabled;
    }
    
    /// Get audit records for a user
    pub async fn get_user_audit(&self, user_id: &str) -> Vec<ValidationAuditRecord> {
        self.permission_validator.get_user_audit(user_id).await
    }
    
    /// Get audit records for a resource
    pub async fn get_resource_audit(&self, resource: &str) -> Vec<ValidationAuditRecord> {
        self.permission_validator.get_resource_audit(resource).await
    }
    
    /// Get all audit records
    pub async fn get_all_audit(&self) -> Vec<ValidationAuditRecord> {
        self.permission_validator.get_all_audit().await
    }
    
    /// Clear audit records
    pub async fn clear_audit(&self) {
        self.permission_validator.clear_audit().await;
    }
    
    /// Set maximum audit log size
    pub async fn set_max_audit_size(&self, size: usize) {
        self.permission_validator.set_max_audit_size(size).await;
    }
    
    /// Get role inheritance diagram as DOT format
    pub async fn get_inheritance_diagram(&self) -> String {
        self.inheritance_manager.to_dot().await
    }
    
    /// Get roles for a user
    pub async fn get_user_roles(&self, user_id: &str) -> HashSet<String> {
        self.rbac_manager.get_user_roles(user_id).await
    }
    
    /// Get a role by ID
    pub async fn get_role(&self, role_id: &str) -> Result<Role> {
        self.rbac_manager.get_role(role_id).await
    }
} 