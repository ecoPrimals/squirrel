# MCP RBAC Refactoring Plan

## Current Status (Overview)

The MCP module has been significantly restructured to implement proper dependency injection patterns, replacing the previous singleton approach. Most of the core functionality is working correctly, but there are several failing tests in the RBAC (Role-Based Access Control) component that need to be addressed.

## Progress Update (2024-05-15)

All the RBAC issues have been resolved! The following changes were made:

1. **Fixed RBAC Manager Structure**:
   - ✅ Added separate `roles_by_id` and `roles_by_name` maps for efficient lookups
   - ✅ Added new `get_role` method that can look up roles by either ID or name
   - ✅ Implemented `create_role_with_id` for deterministic role creation in tests
   - ✅ Improved permission checking and role inheritance

2. **Improved Error Handling**:
   - ✅ Added `Security` variant to the `SquirrelError` enum
   - ✅ Added proper error conversion between `SquirrelError` and `MCPError`
   - ✅ Added `InvalidRole` variant to the `SecurityError` enum
   - ✅ Improved error messages for role not found scenarios

3. **Fixed Dependency Integration**:
   - ✅ Updated the `SecurityManager` to properly interact with the RBAC manager
   - ✅ Fixed imports and resolved module organization issues
   - ✅ Fixed tests to use stable IDs for deterministic testing
   - ✅ Added better documentation for the RBAC functionality

## Previous Issues

The following tests were previously failing:

1. `test_permission_check` - Roles not found in the system
2. `test_role_inheritance` - Role inheritance not properly working
3. `test_authentication_with_roles` - Authentication with roles failing
4. `test_authorization_with_permission` - Permission authorization failing
5. `test_rbac_integration` - Integration between RBAC and security manager failing

### Common Error Patterns

The primary errors observed:

```
thread 'mcp::security::rbac::tests::test_role_inheritance' panicked at crates\core\src\mcp\security\rbac.rs:252:70:
called `Result::unwrap()` on an `Err` value: Security(InvalidCredentials("Role '2629250c-1ea0-4b0c-abdb-230d8e09f5ef' not found in system"))
```

```
thread 'mcp::security::tests::test_authentication_with_roles' panicked at crates\core\src\mcp\security\mod.rs:627:9:
assertion failed: security.has_permission(&credentials.client_id, &read_permission).await
```

## Root Causes Analysis

After examining the code, the following issues were causing the failures:

1. **UUID-based Role IDs**: Role IDs were dynamically generated UUIDs which meant:
   - Tests couldn't reliably reference roles by ID
   - Role inheritance couldn't find parent roles by ID
   - Role assignments were failing when roles weren't found

2. **Asynchronous RBAC Operations**: The RBAC manager itself wasn't async, but was wrapped in an async RwLock, creating inconsistencies:
   - Direct RBAC tests used synchronous methods
   - Security manager tests used async methods on the same underlying RBAC manager
   - Lock timing issues led to incomplete data visibility

3. **State Management**: There was inconsistent state handling:
   - The RBAC manager used HashMaps without proper synchronization
   - The SecurityManager wrapped the RBAC manager in an async RwLock
   - Role creation and user assignment were happening out of order

## Implementation Details

### 1. RBAC Manager Refactoring

The RBAC manager was improved with a dual-map approach for efficient lookups:

```rust
pub struct RBACManager {
    /// Map of role IDs to Role objects (primary lookup)
    roles_by_id: HashMap<String, Role>,
    /// Map of role names to role IDs (secondary lookup)
    roles_by_name: HashMap<String, String>,
    /// Map of user IDs to their assigned role IDs
    user_roles: HashMap<String, HashSet<String>>,
}
```

New methods were added:
```rust
impl RBACManager {
    // Added stable ID generation for tests
    pub fn create_role_with_id(&mut self, id: String, name: String, ...) -> Result<Role> { ... }
    
    // Added better lookup methods
    pub fn get_role(&self, id_or_name: &str) -> Option<&Role> { ... }
    
    // Improved parent role resolution
    fn verify_parent_roles(&self, parent_ids: &HashSet<String>) -> Result<()> { ... }
}
```

### 2. Security Manager Integration

The SecurityManager was updated to better handle the synchronous RBAC manager:

```rust
impl SecurityManager {
    // Improved role resolution
    pub async fn assign_role_by_name(&self, user_id: String, role_name: String) -> Result<()> {
        let mut rbac_manager = self.rbac_manager.write().await;
        rbac_manager.assign_role_by_name(user_id, role_name)
            .map_err(|e| match e {
                SquirrelError::Security(msg) => MCPError::Security(SecurityError::InvalidCredentials(msg)),
                _ => MCPError::Security(SecurityError::InvalidCredentials(format!("{}", e))),
            })
    }
    
    // Fixed permission checking with proper error handling
    pub async fn has_permission(&self, user_id: &str, permission: &Permission) -> bool {
        let rbac_manager = self.rbac_manager.read().await;
        rbac_manager.has_permission(user_id, permission)
    }
}
```

### 3. Test Fixes

Tests were updated to:
1. Use deterministic role IDs for test cases
2. Add more robust error handling
3. Ensure proper setup sequencing
4. Add more detailed assertions and error messages

## Module Structure Assessment

### Current Structure

The codebase currently has:

- `src/mcp/` - Main protocol implementation
  - `protocol/` - Core protocol handling
  - `security/` - Authentication, authorization, and RBAC
  - `context/` - Context management within MCP
- `src/context/` - General context management
  - `tracker.rs` - Context tracking
  - `persistence.rs` - Context persistence
  - `recovery.rs` - Context recovery systems
  - `sync.rs` - Synchronization utilities

### Evaluation

The current division between `src/mcp` and `src/context/` is *mostly logical* with some concerns:

#### Strengths
- Clear separation between protocol-specific functionality and general context management
- Security components properly contained within the MCP module
- Protocol implementation isolated from general context handling

#### Concerns
- Duplicate context systems (`src/mcp/context/` and `src/context/`)
- Potential for fragmentation as features evolve
- Unclear boundaries for frontier model integration

### Recommendations for Context System

For supporting frontier models, the context system will need:

1. **Enhanced Persistence**: Store and retrieve complex state representations
2. **Schema Evolution**: Handle changing context structures
3. **Plugin Architecture**: Allow custom processors for different model types
4. **Performance Optimization**: Efficient handling of large context objects
5. **Metadata Management**: Track context usage, ownership, and permissions

#### Proposed Structure Enhancements

```
src/
├── context/
│   ├── mod.rs           - Main context system API
│   ├── tracker.rs       - Context lifecycle tracking
│   ├── persistence.rs   - Storage backends
│   ├── schema/          - Context schema definition and evolution
│   ├── plugins/         - Extensible plugin system 
│   └── frontier/        - Frontier model specific components
└── mcp/
    ├── mod.rs
    ├── protocol/        - Protocol implementation
    ├── security/        - Authentication and authorization
    └── context_adapter/ - Adapter connecting MCP to context system
```

This structure would:
1. Keep the core context system independent
2. Add frontier-specific components in a dedicated module
3. Use an adapter pattern to connect MCP to the context system
4. Allow for extensibility through plugins

## Next Steps

1. ✅ Fix RBAC issues:
   - ✅ Focus on the role lookup and reference problems
   - ✅ Add stable ID generation for testing
   - ✅ Fix permission checking logic

2. Establish a clearer boundary between MCP and context:
   - Create a proper adapter layer
   - Ensure clean separation of concerns

3. Start developing the enhanced context system:
   - Define the requirements for frontier models
   - Create a plugin architecture
   - Implement the schema evolution system 