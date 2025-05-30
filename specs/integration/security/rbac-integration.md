# RBAC Integration Specification

## Overview
This document specifies the RBAC (Role-Based Access Control) integration requirements for the Squirrel MCP project, focusing on authorization, role management, and permission validation.

## Integration Status
- Current Progress: 40%
- Target Completion: Q2 2024
- Priority: High

## Component Integration

### 1. RBAC Manager Interface
```rust
/// Unified RBAC Manager trait that consolidates core and enhanced functionality
#[async_trait]
pub trait RBACManager: Send + Sync + std::fmt::Debug {
    /// Get the manager's name
    fn name(&self) -> &str;
    
    /// Get the version of the manager
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
}
```

### 2. Mock RBAC Manager
```rust
/// Mock RBAC Manager for testing
#[derive(Debug)]
pub struct MockRBACManager {
    /// Whether to allow all permission checks
    allow_all: bool,
    /// Optional user roles to return for specific users
    user_roles: RwLock<HashMap<String, HashSet<String>>>,
}

impl MockRBACManager {
    /// Create a new mock RBAC manager
    pub fn new(allow_all: bool) -> Self {
        Self {
            allow_all,
            user_roles: RwLock::new(HashMap::new()),
        }
    }
    
    /// Configure the mock to return specific roles for a user
    pub async fn with_user_roles(&self, user_id: &str, roles: Vec<String>) -> Result<()> {
        let mut user_roles = self.user_roles.write().await;
        let role_set = HashSet::from_iter(roles.into_iter());
        user_roles.insert(user_id.to_string(), role_set);
        
        Ok(())
    }
}
```

### 3. MCP Integration
```rust
/// Core MCP Adapter that integrates with the RBAC system
pub struct CoreMCPAdapter {
    /// RBAC Manager for authorization
    rbac_manager: Arc<dyn RBACManager>,
    /// Other components...
}

impl CoreMCPAdapter {
    /// Create a new Core MCP Adapter
    pub fn new(rbac_manager: Arc<dyn RBACManager>) -> Self {
        Self {
            rbac_manager,
            // Initialize other components...
        }
    }
    
    /// Check if a user has a specific permission
    pub async fn check_permission(&self, user_id: &str, permission: &str) -> Result<bool> {
        self.rbac_manager.has_permission(user_id, permission, None).await
    }
}
```

## Integration Requirements

### 1. RBAC Manager Integration
- Must support multiple implementations (Basic, Mock, Custom)
- Must integrate with the MCP security system
- Must provide thread-safe access to roles and permissions
- Must support context-aware permission checks
- Must provide proper error handling

### 2. Permission Checking Integration
- Must support resource-based permissions
- Must support action-based permissions
- Must support wildcard permissions
- Must support permission inheritance
- Must provide context-aware permission validation

### 3. Role Management Integration
- Must support dynamic role assignment
- Must support role revocation
- Must support role hierarchies
- Must provide role query capabilities
- Must support bulk role operations

## Integration Tests

### 1. RBAC Manager Integration Tests
```rust
#[tokio::test]
async fn test_rbac_integration() {
    // Create mock RBAC manager
    let rbac_manager = Arc::new(MockRBACManager::new(false));
    
    // Configure roles
    rbac_manager.with_user_roles("user1", vec!["admin".to_string()]).await?;
    rbac_manager.with_user_roles("user2", vec!["user".to_string()]).await?;
    
    // Create MCP adapter with RBAC manager
    let adapter = CoreMCPAdapter::new(rbac_manager);
    
    // Test permissions
    assert!(adapter.check_permission("user1", "admin:action").await?);
    assert!(!adapter.check_permission("user2", "admin:action").await?);
}
```

### 2. Mock RBAC Manager Tests
```rust
#[tokio::test]
async fn test_mock_rbac_manager() {
    // Create mock RBAC manager
    let rbac = MockRBACManager::new(false);
    
    // Verify initial state
    assert_eq!(rbac.name(), "MockRBACManager");
    assert_eq!(rbac.version(), "1.0.0");
    
    // Configure roles
    rbac.with_user_roles("user1", vec!["admin".to_string()]).await?;
    
    // Test role assignment
    assert!(rbac.has_role("user1", "admin").await?);
    assert!(!rbac.has_role("user2", "admin").await?);
    
    // Test permission checks
    assert!(!rbac.has_permission("user2", "admin:read", None).await?);
    
    // Create a mock that allows all permissions
    let rbac_allow_all = MockRBACManager::new(true);
    assert!(rbac_allow_all.has_permission("user2", "admin:read", None).await?);
}
```

## Implementation Guidelines

### 1. RBAC Manager Implementation
```rust
#[async_trait]
impl RBACManager for MockRBACManager {
    fn name(&self) -> &str {
        "MockRBACManager"
    }
    
    fn version(&self) -> &str {
        "1.0.0"
    }
    
    async fn has_permission(&self, user_id: &str, permission: &str, _context: Option<&Context>) -> Result<bool> {
        if self.allow_all {
            return Ok(true);
        }
        
        // Check if any of the user's roles have the permission
        // In a real implementation, we would check against a permission database
        let roles = self.get_user_roles(user_id).await?;
        
        // For mock purposes, we'll just check if the user has admin role
        Ok(roles.contains(&"admin".to_string()))
    }
    
    async fn assign_role(&self, user_id: &str, role_id: &str) -> Result<()> {
        let mut user_roles = self.user_roles.write().await;
        
        user_roles.entry(user_id.to_string())
            .or_insert_with(HashSet::new)
            .insert(role_id.to_string());
        
        Ok(())
    }
    
    // Other implementation methods...
}
```

### 2. MCP Adapter Integration
```rust
impl CoreMCPAdapter {
    pub async fn initialize(&self) -> Result<()> {
        // Initialize RBAC system
        self.init_rbac().await?;
        
        // Initialize other components
        // ...
        
        Ok(())
    }
    
    async fn init_rbac(&self) -> Result<()> {
        // Log initialization
        info!("Initializing RBAC system with {}", self.rbac_manager.name());
        
        // Perform any setup required for the RBAC manager
        // ...
        
        Ok(())
    }
    
    pub async fn authorize_action(&self, user_id: &str, action: &str, resource: &str) -> Result<bool> {
        let permission = format!("{}:{}", action, resource);
        self.rbac_manager.has_permission(user_id, &permission, None).await
    }
}
```

## Error Handling

### 1. RBAC Error Types
```rust
#[derive(Debug, Error)]
pub enum RBACError {
    #[error("Role not found: {0}")]
    RoleNotFound(String),
    
    #[error("User not found: {0}")]
    UserNotFound(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("Invalid role format: {0}")]
    InvalidRoleFormat(String),
    
    #[error("Invalid permission format: {0}")]
    InvalidPermissionFormat(String),
}
```

### 2. Error Recovery Strategies
```rust
impl CoreMCPAdapter {
    pub async fn handle_rbac_error(&self, error: RBACError) -> Result<()> {
        match error {
            RBACError::PermissionDenied(msg) => {
                // Log the permission denial
                warn!("Permission denied: {}", msg);
                // Increment metrics
                self.metrics.increment_counter("rbac.permission_denied");
            }
            RBACError::RoleNotFound(role_id) => {
                // Log the error
                warn!("Role not found: {}", role_id);
                // Create the role if it's a standard role
                if self.is_standard_role(&role_id) {
                    self.create_standard_role(&role_id).await?;
                }
            }
            _ => {
                // Log other errors
                error!("RBAC error: {}", error);
            }
        }
        
        Ok(())
    }
}
```

## Monitoring and Metrics

### 1. RBAC Metrics
- Permission check count
- Permission denial count
- Role assignment count
- Role revocation count
- Error count by type
- Average permission check time

### 2. Metrics Implementation
```rust
impl CoreMCPAdapter {
    pub fn record_permission_check(&self, user_id: &str, permission: &str, result: bool) {
        // Record metrics for permission checks
        self.metrics.increment_counter("rbac.permission_check");
        
        if !result {
            self.metrics.increment_counter("rbac.permission_denied");
        }
        
        // Record per-permission metrics
        let permission_metric = format!("rbac.permission.{}", permission.replace(':', "_"));
        self.metrics.increment_counter(&permission_metric);
    }
    
    pub fn record_role_operation(&self, operation: &str, user_id: &str, role_id: &str) {
        // Record metrics for role operations
        let operation_metric = format!("rbac.role_operation.{}", operation);
        self.metrics.increment_counter(&operation_metric);
        
        // Record per-role metrics
        let role_metric = format!("rbac.role.{}.{}", role_id, operation);
        self.metrics.increment_counter(&role_metric);
    }
}
```

## Integration Scenarios

### 1. Basic Authentication Flow
1. User authenticates with the system
2. System retrieves user roles from RBAC manager
3. User attempts to access a protected resource
4. System checks permission using RBAC manager
5. Access is granted or denied based on permission check

### 2. Role Management Flow
1. Administrator assigns a role to a user
2. RBAC manager updates role assignments
3. User's permissions are immediately updated
4. User attempts to access a previously restricted resource
5. Access is granted based on new role assignment

### 3. Permission Inheritance Flow
1. Administrator creates a role hierarchy
2. Child roles inherit permissions from parent roles
3. User is assigned a child role
4. User attempts to access a resource protected by a parent role permission
5. Access is granted based on inherited permissions

## Migration Strategy

### 1. From Legacy Authorization
- Map legacy permissions to new RBAC permissions
- Create roles based on legacy user groups
- Assign roles to users based on legacy permissions
- Run parallel authorization checks during migration
- Monitor permission check failures

### 2. To Enhanced RBAC
- Extend context-aware permission checks
- Implement attribute-based access control (ABAC)
- Support dynamic permission evaluation
- Integrate with external identity providers
- Support fine-grained audit logging

<version>1.0.0</version> 