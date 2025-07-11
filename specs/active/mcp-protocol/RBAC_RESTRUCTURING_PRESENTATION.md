# RBAC Restructuring Plan

## Current Challenges

- The `SecurityManagerImpl<R>` is tightly coupled to specific RBAC implementations through generics, making it hard to swap implementations.
- We have duplicate definitions of the `EnhancedRBACManager` trait with overlapping functionality.
- The architecture is too complex, with advanced features being implemented before basic functionality is stable.
- Frequent compilation issues due to the complex architecture.
- Type mismatches between different parts of the codebase (UUID vs strings for user IDs).
- Inconsistent error handling and credential types.

## Proposed Solution

We need to restructure the RBAC system to:

1. Simplify the trait hierarchy with a single, comprehensive `RBACManager` trait
2. Decouple the Security Manager from specific RBAC implementations
3. Implement a progressive feature adoption approach (basic → advanced)
4. Improve maintainability with better abstractions
5. Create clear interfaces for extending the security system
6. Establish consistent type handling throughout the system

## Key Design Changes

### 1. Unified `RBACManager` Trait

- **Status: ✅ COMPLETED**
- Consolidates all RBAC functionality into a single trait
- Provides default implementations for convenience methods
- Ensures backward compatibility with existing code

### 2. Decoupled Security Manager

- **Status: ✅ COMPLETED**
- Uses trait objects (`dyn RBACManager`) instead of generics
- Allows runtime swapping of RBAC implementations
- Simplifies initialization and configuration

### 3. Tiered RBAC Managers

- **Status: ✅ COMPLETED for `BasicRBACManager` and `MockRBACManager`**
- Implement different RBAC managers for different needs:
  - `BasicRBACManager`: Core functionality for simple applications
  - `EnhancedRBACManager`: Extends basic with role hierarchies and conditions
  - `MockRBACManager`: For testing, with deterministic behavior

### 4. Builder Pattern for Configuration

- **Status: 🔄 PLANNED**
- Consistent configuration approach across managers
- Type-safe builder pattern for initialization
- Clear separation of configuration from behavior

### 5. Type Consistency

- **Status: 🔄 IN PROGRESS**
- Standardized user ID type handling (string vs UUID)
- Consistent credential type usage
- Clear error types with detailed information

## Implementation Plan

### Phase 1: Core Restructuring - MOSTLY COMPLETED

- [x] Define the unified `RBACManager` trait
- [x] Implement `BasicRBACManager` with core functionality
- [x] Create backward-compatible adapter for existing code
- [x] Update `SecurityManagerImpl` to use trait objects
- [x] Migrate `MockRBACManager` to the new system
- [x] Create standalone tests to validate the implementations

### Phase 2: Integration and Compatibility - STARTED

- [ ] Update the authentication manager to work with the new RBAC system
- [ ] Address type mismatches (UUID vs string) for user IDs
- [ ] Fix initialization code to use the new implementations
- [ ] Create comprehensive integration tests
- [ ] Update documentation to reflect the changes

### Phase 3: Advanced Features (PLANNED)

- [ ] Implement `EnhancedRBACManager` with role hierarchies
- [ ] Add context-aware permission checks
- [ ] Include condition-based access control
- [ ] Support dynamic policy evaluation

### Phase 4: Performance Optimization (PLANNED)

- [ ] Implement caching for frequently accessed permissions
- [ ] Optimize permission checks for large role sets
- [ ] Add benchmarks for performance comparison
- [ ] Document performance characteristics

### Phase 5: Testing and Documentation (PLANNED)

- [ ] Create comprehensive test suite
- [ ] Add code examples for common use cases
- [ ] Document integration patterns
- [ ] Create tutorials for extending the system

## Current Status and Challenges

We've completed the core restructuring of the RBAC system, but integration with the rest of the codebase presents significant challenges:

1. **Compilation Issues**: The codebase has numerous interdependent compilation errors that extend beyond the RBAC implementation.

2. **Type Mismatches**: There are inconsistencies in how user IDs are represented (UUID vs strings) across different components.

3. **Credential Types**: Different credential structures are used in various parts of the security subsystem.

4. **Integration Complexity**: The security subsystem has tight coupling between components, making isolated changes difficult.

To address these challenges, we're taking a multi-pronged approach:

1. **Standalone Testing**: Continue to validate the implementations independently to ensure core functionality works.

2. **Incremental Integration**: Focus on one component at a time, starting with the authentication manager.

3. **Documentation**: Track progress and challenges to inform future refactoring efforts.

## Benefits

1. **Reduced Complexity**: Simpler trait hierarchy and clearer responsibilities
2. **Better Extensibility**: Easy to implement custom RBAC managers for specific needs
3. **Improved Testability**: Better mocking capabilities for isolated testing
4. **Reduced Coupling**: Security Manager works with any RBACManager implementation
5. **Progressive Approach**: Basic functionality first, then advanced features
6. **Type Consistency**: Clear and consistent type usage across the system

## Example Usage

```rust
// Initialize a basic RBAC manager
let rbac_manager = Arc::new(BasicRBACManager::new());

// Initialize the security manager with the RBAC manager
let security_manager = SecurityManagerBuilder::new()
    .with_rbac_manager(rbac_manager)
    .with_crypto_provider(crypto_provider)
    .with_token_manager(token_manager)
    .build();

// Use the security manager for permission checks
if security_manager.has_permission(user_id, "resource:action", context).await? {
    // Perform the action
}
```

## Backward Compatibility

The new `RBACManager` trait will provide all functionality from the previous traits, allowing for a gradual transition:

- `EnhancedRBACManager` → `RBACManager`
- `UserRoleManager` → `RBACManager`
- `PermissionManager` → `RBACManager`

## Next Steps

1. ✅ Review and approve the restructuring plan
2. ✅ Implement core trait and basic RBAC manager
3. ✅ Update mock RBAC manager for testing
4. ✅ Create standalone tests for RBAC implementations
5. 🚧 Fix authentication manager to work with the new system
6. 🚧 Update initialization code for the new implementations
7. ⏳ Phase out old RBAC implementations
8. ⏳ Update documentation to reflect the changes

## Questions and Discussion

- Should we consider a more comprehensive refactoring of the security subsystem, beyond just the RBAC components?
- Should we prioritize fixing the authentication manager or focus on phasing out the old RBAC implementations first?
- How should we handle the conversion between UUID and string for user IDs? Add conversion methods or standardize on one type? 