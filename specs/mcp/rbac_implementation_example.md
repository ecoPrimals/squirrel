# RBAC Implementation Example

This document provides example code for the restructured RBAC system as described in the RBAC_RESTRUCTURING_PLAN.md. This is intended to demonstrate the practical implementation of the proposed changes.

## 1. Unified RBAC Trait

```rust
use async_trait::async_trait;
use std::fmt::Debug;
use std::sync::Arc;
use crate::error::Result;
use crate::context_manager::Context;

/// Unified RBAC Manager trait that combines core and enhanced functionality
#[async_trait]
pub trait RBACManager: Send + Sync + Debug {
    /// Get the name of the RBAC Manager implementation
    fn name(&self) -> &str;
    
    /// Get the version of the RBAC Manager implementation
    fn version(&self) -> &str;
    
    /// Check if a user has a specific permission
    async fn has_permission(&self, user_id: &str, permission: &str, context: Option<&Context>) -> Result<bool>;
    
    /// Assign a role to a user
    async fn assign_role(&self, user_id: &str, role_id: &str) -> Result<()>;
    
    /// Revoke a role from a user
    async fn revoke_role(&self, user_id: &str, role_id: &str) -> Result<()>;
    
    /// Get all roles assigned to a user
    async fn get_user_roles(&self, user_id: &str) -> Result<Vec<String>>;
    
    /// Check if a user has a specific role
    async fn has_role(&self, user_id: &str, role_id: &str) -> Result<bool>;
    
    /// Get detailed information about a role (optional)
    async fn get_role_details(&self, role_id: &str) -> Result<Option<Role>> {
        // Default implementation returns None
        Ok(None)
    }
    
    /// Get permissions for a role (optional)
    async fn get_permissions_for_role(&self, _role_id: &str) -> Result<Vec<Permission>> {
        // Default implementation returns empty list
        Ok(Vec::new())
    }
}
```

## 2. Basic RBAC Manager Implementation

```rust
use std::collections::{HashMap, HashSet};
use tokio::sync::RwLock;
use async_trait::async_trait;
use uuid::Uuid;
use chrono::Utc;

/// Basic RBAC Manager implementation with core functionality
#[derive(Debug)]
pub struct BasicRBACManager {
    /// Roles managed by this RBAC manager
    roles: RwLock<HashMap<String, Role>>,
    /// User-to-role mappings
    user_roles: RwLock<HashMap<String, HashSet<String>>>,
}

impl BasicRBACManager {
    /// Create a new RBAC manager
    pub fn new() -> Self {
        Self {
            roles: RwLock::new(HashMap::new()),
            user_roles: RwLock::new(HashMap::new()),
        }
    }
    
    /// Internal helper to get user roles as a HashSet
    async fn get_user_roles_set(&self, user_id: &str) -> HashSet<String> {
        let user_roles = self.user_roles.read().await;
        user_roles.get(user_id).cloned().unwrap_or_default()
    }
    
    /// Get a role by ID
    pub async fn get_role(&self, role_id: &str) -> Result<Role> {
        let roles = self.roles.read().await;
        roles.get(role_id)
            .cloned()
            .ok_or_else(|| MCPError::Security(SecurityError::RBACError(
                format!("Role not found: {role_id}")
            )))
    }
    
    /// Create a new role
    pub async fn create_role(&self, name: &str, description: Option<&str>) -> Result<Role> {
        let role_id = Uuid::new_v4().to_string();
        let now = Utc::now();
        
        let role = Role {
            id: role_id.clone(),
            name: name.to_string(),
            description: description.map(ToString::to_string),
            permissions: HashSet::new(),
            created_at: now,
            updated_at: now,
            // Other fields simplified for this example
        };
        
        {
            let mut roles = self.roles.write().await;
            // Check if role with this name already exists
            if roles.values().any(|r| r.name == name) {
                return Err(MCPError::Security(SecurityError::RBACError(
                    format!("Role exists: {name}")
                )));
            }
            
            roles.insert(role_id.clone(), role.clone());
        }
        
        Ok(role)
    }
}

#[async_trait]
impl RBACManager for BasicRBACManager {
    fn name(&self) -> &str {
        "BasicRBACManager"
    }
    
    fn version(&self) -> &str {
        "1.0.0"
    }
    
    async fn has_permission(&self, user_id: &str, permission: &str, _context: Option<&Context>) -> Result<bool> {
        let user_roles = self.get_user_roles_set(user_id).await;
        
        // If the user has no roles, they have no permissions
        if user_roles.is_empty() {
            return Ok(false);
        }
        
        // Check each role for the permission
        for role_id in user_roles {
            if let Ok(role) = self.get_role(&role_id).await {
                if role.permissions.iter().any(|p| p.to_string() == permission) {
                    return Ok(true);
                }
            }
        }
        
        Ok(false)
    }
    
    async fn assign_role(&self, user_id: &str, role_id: &str) -> Result<()> {
        // Verify role exists
        {
            let roles = self.roles.read().await;
            if !roles.contains_key(role_id) {
                return Err(MCPError::Security(SecurityError::RBACError(
                    format!("Role not found: {role_id}")
                )));
            }
        }
        
        // Add role to user
        {
            self.user_roles.write().await
                .entry(user_id.to_string())
                .or_insert_with(HashSet::new)
                .insert(role_id.to_string());
        }
        
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
        let roles = self.get_user_roles_set(user_id).await;
        Ok(roles.into_iter().collect())
    }
    
    async fn has_role(&self, user_id: &str, role_id: &str) -> Result<bool> {
        let user_roles = self.get_user_roles_set(user_id).await;
        Ok(user_roles.contains(role_id))
    }
    
    // Override the default implementation of get_role_details
    async fn get_role_details(&self, role_id: &str) -> Result<Option<Role>> {
        match self.get_role(role_id).await {
            Ok(role) => Ok(Some(role)),
            Err(_) => Ok(None),
        }
    }
}
```

## 3. Cached RBAC Manager Implementation

```rust
use lru::LruCache;
use std::num::NonZeroUsize;
use std::sync::Mutex;

/// RBAC Manager with caching for better performance
#[derive(Debug)]
pub struct CachedRBACManager {
    /// Inner basic implementation
    inner: BasicRBACManager,
    /// Permission check cache
    permission_cache: Mutex<LruCache<String, bool>>,
    /// Role check cache
    role_cache: Mutex<LruCache<String, bool>>,
}

impl CachedRBACManager {
    /// Create a new cached RBAC manager
    pub fn new() -> Self {
        Self {
            inner: BasicRBACManager::new(),
            permission_cache: Mutex::new(LruCache::new(NonZeroUsize::new(1000).unwrap())),
            role_cache: Mutex::new(LruCache::new(NonZeroUsize::new(500).unwrap())),
        }
    }
    
    /// Generate cache key for permission checks
    fn permission_cache_key(user_id: &str, permission: &str) -> String {
        format!("perm:{}:{}", user_id, permission)
    }
    
    /// Generate cache key for role checks
    fn role_cache_key(user_id: &str, role_id: &str) -> String {
        format!("role:{}:{}", user_id, role_id)
    }
}

#[async_trait]
impl RBACManager for CachedRBACManager {
    fn name(&self) -> &str {
        "CachedRBACManager"
    }
    
    fn version(&self) -> &str {
        "1.0.0"
    }
    
    async fn has_permission(&self, user_id: &str, permission: &str, context: Option<&Context>) -> Result<bool> {
        // For simplicity, only cache when context is None
        if context.is_none() {
            let cache_key = Self::permission_cache_key(user_id, permission);
            
            // Check cache first
            {
                let mut cache = self.permission_cache.lock().unwrap();
                if let Some(&result) = cache.get(&cache_key) {
                    return Ok(result);
                }
            }
            
            // If not in cache, ask the inner manager
            let result = self.inner.has_permission(user_id, permission, context).await?;
            
            // Cache the result
            {
                let mut cache = self.permission_cache.lock().unwrap();
                cache.put(cache_key, result);
            }
            
            return Ok(result);
        }
        
        // If context is provided, just delegate to inner without caching
        self.inner.has_permission(user_id, permission, context).await
    }
    
    // Implement other methods by delegating to inner with appropriate caching...
    // (Other method implementations would follow the same pattern)
    
    async fn assign_role(&self, user_id: &str, role_id: &str) -> Result<()> {
        // Invalidate caches when roles change
        {
            let mut perm_cache = self.permission_cache.lock().unwrap();
            perm_cache.clear(); // Simple strategy - clear all for now
            
            let mut role_cache = self.role_cache.lock().unwrap();
            role_cache.clear();
        }
        
        self.inner.assign_role(user_id, role_id).await
    }
    
    // Implement remaining methods...
}
```

## 4. Advanced RBAC Manager with Inheritance

```rust
#[cfg(feature = "advanced-rbac")]
pub mod advanced {
    use super::*;
    use std::collections::{HashMap, HashSet};
    
    /// Advanced RBAC Manager with role inheritance and advanced validation
    #[derive(Debug)]
    pub struct AdvancedRBACManager {
        /// Basic implementation for core functionality
        inner: BasicRBACManager,
        /// Role inheritance relationships
        inheritance: RwLock<HashMap<String, HashSet<String>>>,
    }
    
    impl AdvancedRBACManager {
        /// Create a new advanced RBAC manager
        pub fn new() -> Self {
            Self {
                inner: BasicRBACManager::new(),
                inheritance: RwLock::new(HashMap::new()),
            }
        }
        
        /// Add inheritance relationship between roles
        pub async fn add_inheritance(&self, child_role: &str, parent_role: &str) -> Result<()> {
            // Verify both roles exist
            {
                let roles = self.inner.roles.read().await;
                if !roles.contains_key(child_role) {
                    return Err(MCPError::Security(SecurityError::RBACError(
                        format!("Child role not found: {child_role}")
                    )));
                }
                
                if !roles.contains_key(parent_role) {
                    return Err(MCPError::Security(SecurityError::RBACError(
                        format!("Parent role not found: {parent_role}")
                    )));
                }
            }
            
            // Add inheritance relationship
            {
                let mut inheritance = self.inheritance.write().await;
                inheritance
                    .entry(child_role.to_string())
                    .or_insert_with(HashSet::new)
                    .insert(parent_role.to_string());
            }
            
            Ok(())
        }
        
        /// Get all parent roles (direct and inherited)
        async fn get_all_parent_roles(&self, role_id: &str) -> HashSet<String> {
            let mut result = HashSet::new();
            let mut to_process = vec![role_id.to_string()];
            
            // Simple breadth-first traversal to find all parent roles
            while let Some(current) = to_process.pop() {
                let inheritance = self.inheritance.read().await;
                
                if let Some(parents) = inheritance.get(&current) {
                    for parent in parents {
                        if result.insert(parent.clone()) {
                            to_process.push(parent.clone());
                        }
                    }
                }
            }
            
            result
        }
    }
    
    #[async_trait]
    impl RBACManager for AdvancedRBACManager {
        fn name(&self) -> &str {
            "AdvancedRBACManager"
        }
        
        fn version(&self) -> &str {
            "1.0.0"
        }
        
        // Override has_permission to include inherited permissions
        async fn has_permission(&self, user_id: &str, permission: &str, context: Option<&Context>) -> Result<bool> {
            // First check direct permissions
            if self.inner.has_permission(user_id, permission, context).await? {
                return Ok(true);
            }
            
            // If not found, check permissions from inherited roles
            let user_roles = self.inner.get_user_roles_set(user_id).await;
            
            // For each role, get all parent roles and check their permissions
            for role_id in user_roles {
                let parent_roles = self.get_all_parent_roles(&role_id).await;
                
                for parent_role in parent_roles {
                    if let Ok(role) = self.inner.get_role(&parent_role).await {
                        if role.permissions.iter().any(|p| p.to_string() == permission) {
                            return Ok(true);
                        }
                    }
                }
            }
            
            Ok(false)
        }
        
        // Implement other methods by delegating to inner...
        // Ensure that inheritance is considered where appropriate
    }
}
```

## 5. Builder Pattern for RBAC Manager Creation

```rust
/// Builder for creating configured RBAC managers
pub struct RBACManagerBuilder {
    with_caching: bool,
    with_inheritance: bool,
    cache_size: usize,
}

impl RBACManagerBuilder {
    /// Create a new builder with default settings
    pub fn new() -> Self {
        Self {
            with_caching: false,
            with_inheritance: false,
            cache_size: 1000,
        }
    }
    
    /// Enable caching for better performance
    pub fn with_caching(mut self) -> Self {
        self.with_caching = true;
        self
    }
    
    /// Set the cache size
    pub fn with_cache_size(mut self, size: usize) -> Self {
        self.cache_size = size;
        self
    }
    
    /// Enable role inheritance
    #[cfg(feature = "advanced-rbac")]
    pub fn with_inheritance(mut self) -> Self {
        self.with_inheritance = true;
        self
    }
    
    /// Build the configured RBAC manager
    pub fn build(self) -> Arc<dyn RBACManager> {
        #[cfg(feature = "advanced-rbac")]
        {
            if self.with_inheritance {
                return Arc::new(advanced::AdvancedRBACManager::new());
            }
        }
        
        if self.with_caching {
            Arc::new(CachedRBACManager::new())
        } else {
            Arc::new(BasicRBACManager::new())
        }
    }
}
```

## 6. Updated Security Manager Implementation

```rust
/// Security Manager implementation using trait objects
pub struct SecurityManagerImpl {
    crypto_provider: Arc<dyn CryptoProvider>,
    token_manager: Arc<dyn TokenManager>,
    identity_manager: Arc<dyn IdentityManager>,
    rbac_manager: Arc<dyn RBACManager>, // Now uses trait object
    audit_service: Arc<dyn AuditService>,
    version: String,
}

impl SecurityManagerImpl {
    /// Create a new security manager with the specified components
    pub fn new(
        crypto_provider: Arc<dyn CryptoProvider>,
        token_manager: Arc<dyn TokenManager>,
        identity_manager: Arc<dyn IdentityManager>,
        rbac_manager: Arc<dyn RBACManager>, // Accept any RBACManager implementation
        audit_service: Arc<dyn AuditService>,
    ) -> Self {
        Self {
            crypto_provider,
            token_manager,
            identity_manager,
            rbac_manager,
            audit_service,
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
    
    // Implementation details...
}
```

## 7. Updated MockRBACManager for Testing

```rust
/// Mock RBAC Manager for testing
#[derive(Debug, Clone)]
pub struct MockRBACManager {
    allow_all: bool,
}

impl MockRBACManager {
    /// Create a new mock RBAC manager
    pub fn new(allow_all: bool) -> Self {
        Self {
            allow_all,
        }
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
    
    async fn assign_role(&self, _user_id: &str, _role_id: &str) -> Result<()> {
        Ok(())
    }
    
    async fn revoke_role(&self, _user_id: &str, _role_id: &str) -> Result<()> {
        Ok(())
    }
    
    async fn get_user_roles(&self, _user_id: &str) -> Result<Vec<String>> {
        Ok(vec!["mock_role".to_string()])
    }
    
    async fn has_role(&self, _user_id: &str, _role_id: &str) -> Result<bool> {
        Ok(self.allow_all)
    }
}
```

## 8. Initialization in MCP

```rust
/// Initialize the security subsystem with the provided configuration
pub fn initialize_security_manager(config: crate::config::SecurityConfig) -> Arc<dyn SecurityManager> {
    // Create implementations
    let key_storage = Arc::new(key_storage::InMemoryKeyStorage::new());
    let identity_manager = Arc::new(identity::DefaultIdentityManager::new());
    let crypto_provider = Arc::new(crypto::DefaultCryptoProvider::new());
    let audit_service = Arc::new(audit::DefaultAuditService::new());
    
    // Create appropriate RBAC manager based on configuration
    let rbac_manager = RBACManagerBuilder::new()
        .with_caching(config.enable_rbac_caching)
        .with_cache_size(config.rbac_cache_size.unwrap_or(1000))
        // Conditionally enable advanced features
        .build();
    
    // Create token manager
    let token_manager = Arc::new(token::DefaultTokenManager::new(
        key_storage.clone(),
        crypto_provider.clone(),
    ));
    
    // Create and return the security manager
    Arc::new(manager::SecurityManagerImpl::new(
        crypto_provider.clone(),
        token_manager.clone(),
        identity_manager,
        rbac_manager,
        audit_service,
    ))
}
```

## Summary

This implementation demonstrates:

1. A unified RBAC trait that combines the functionality of the current traits
2. Basic, Cached, and Advanced implementations that build on each other
3. Separation of concerns with progressive feature adoption
4. Use of feature flags to control advanced functionality
5. A builder pattern for configuration
6. Decoupling of the Security Manager from specific RBAC implementations

This approach significantly reduces technical debt while maintaining functionality and providing a clear path for future enhancements. 