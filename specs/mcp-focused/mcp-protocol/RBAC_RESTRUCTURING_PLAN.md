# RBAC System Restructuring Plan

## Overview

The current Role-Based Access Control (RBAC) system in the MCP crate has accumulated technical debt due to its complex architecture, multiple layers of abstraction, and inconsistent trait boundaries. This document outlines a plan to restructure the RBAC system to reduce complexity while maintaining functionality and allowing for future extensions.

## Background

The current implementation includes:
- Multiple overlapping traits (`RBACManager`, `AsyncRBACManager`, `EnhancedRBACManager`)
- Complex permission models
- Advanced features like role inheritance and permission validation
- Tight coupling between the Security Manager and RBAC implementations
- Inconsistent trait boundaries and duplicate definitions

These issues contribute to compilation problems, increased maintenance burden, and challenges in extending the system.

## Goals

1. Simplify the RBAC trait hierarchy
2. Decouple the Security Manager from specific RBAC implementations
3. Implement a progressive approach to RBAC features
4. Ensure backward compatibility with existing code
5. Provide a clear migration path
6. Reduce technical debt while preserving functionality

## Non-Goals

1. Completely rewriting the security system
2. Removing advanced features entirely
3. Breaking existing security guarantees
4. Adding new security features beyond those already planned

## Design

### 1. Unified RBAC Trait

Replace the current three-tier trait system with a single comprehensive trait:

```rust
#[async_trait::async_trait]
pub trait RBACManager: Send + Sync + std::fmt::Debug {
    // Basic information
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    
    // Core async methods
    async fn has_permission(&self, user_id: &str, permission: &str, context: Option<&Context>) -> Result<bool>;
    async fn assign_role(&self, user_id: &str, role_id: &str) -> Result<()>;
    async fn revoke_role(&self, user_id: &str, role_id: &str) -> Result<()>;
    async fn get_user_roles(&self, user_id: &str) -> Result<Vec<String>>;
    async fn has_role(&self, user_id: &str, role_id: &str) -> Result<bool>;
    
    // Enhanced functionality with default implementations
    async fn get_role_details(&self, _role_id: &str) -> Result<Option<Role>> {
        Ok(None) // Default implementation returns None
    }
    
    // Other optional methods with default implementations
    async fn get_permissions_for_role(&self, _role_id: &str) -> Result<Vec<Permission>> {
        Ok(Vec::new())
    }
}
```

### 2. Decouple Security Manager

Change `SecurityManagerImpl` to use trait objects rather than generics:

```rust
pub struct SecurityManagerImpl {
    crypto_provider: Arc<dyn CryptoProvider>,
    token_manager: Arc<dyn TokenManager>,
    identity_manager: Arc<dyn IdentityManager>,
    rbac_manager: Arc<dyn RBACManager>, // Use trait object instead of generic
    audit_service: Arc<dyn AuditService>,
    version: String,
}
```

### 3. Tiered RBAC Implementation

Implement RBAC components in layers:

1. **BasicRBACManager**: Core functionality only
2. **CachedRBACManager**: Adds caching for better performance
3. **AdvancedRBACManager**: Adds inheritance and complex validation

Control availability through feature flags:

```rust
#[cfg(feature = "advanced-rbac")]
mod advanced {
    pub mod inheritance;
    pub mod validation;
    pub struct AdvancedRBACManager { /* ... */ }
}
```

### 4. Builder Pattern for Configuration

Implement a builder pattern for creating configured RBAC managers:

```rust
pub struct RBACManagerBuilder {
    with_caching: bool,
    with_inheritance: bool,
    audit_enabled: bool,
}

impl RBACManagerBuilder {
    pub fn new() -> Self { /* ... */ }
    pub fn with_caching(mut self) -> Self { /* ... */ }
    pub fn build(self) -> Arc<dyn RBACManager> { /* ... */ }
}
```

## Implementation Plan

### Phase 1: Core Restructuring

1. **Define the unified RBACManager trait** in `rbac/mod.rs`
2. **Update MockRBACManager** to implement the new trait
3. **Modify SecurityManagerImpl** to use trait objects
4. **Create BasicRBACManager** implementation with core functionality
5. **Update initialization code** in `initialize_security_manager`

### Phase 2: Advanced Features

1. **Refactor role inheritance** into a separate module
2. **Refactor permission validation** into a separate module
3. **Create AdvancedRBACManager** that extends BasicRBACManager
4. **Implement feature flags** for advanced features

### Phase 3: Performance Optimization

1. **Implement CachedRBACManager** for better performance
2. **Add LRU caching** for frequent permission checks
3. **Optimize permission matching algorithms**

### Phase 4: Testing and Documentation

1. **Create comprehensive tests** for all implementations
2. **Update documentation** to reflect new architecture
3. **Create migration guide** for existing implementations

## Migration Strategy

For existing code:

1. The new `RBACManager` trait will subsume the functionality of the previous traits
2. Existing trait implementations can be updated incrementally
3. The MockRBACManager implementation will be updated first as an example
4. Legacy traits can be deprecated but maintained for backward compatibility

## Timeline

- Phase 1: 2 days
- Phase 2: 3 days
- Phase 3: 2 days
- Phase 4: 3 days

**Total:** 10 days for complete implementation

## Risks and Mitigations

| Risk | Mitigation |
|------|------------|
| Breaking existing functionality | Comprehensive test suite before merging |
| Performance regression | Benchmark critical paths before and after changes |
| Increased code size | Use feature flags to control compilation of advanced features |
| Migration complexity | Provide clear examples and documentation |

## Success Metrics

1. Reduced compilation errors related to RBAC
2. Simplified code with fewer abstraction layers
3. Improved testability of security components
4. Maintained or improved performance
5. Preserved security guarantees

## Conclusion

This restructuring will significantly reduce technical debt in the RBAC system while preserving functionality and allowing for future extensions. By simplifying the trait hierarchy, decoupling components, and implementing a progressive approach to features, we can create a more maintainable and robust security system. 