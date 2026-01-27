# Smart Refactoring Plan: ecosystem/mod.rs
**Date**: January 27, 2026  
**Current Size**: 1041 lines  
**Target**: <1000 lines per file (max)  
**Strategy**: Logical module extraction (NOT arbitrary splitting)

## Current Structure Analysis

### File Breakdown (1041 lines)
- **Module Header & Docs**: ~60 lines
- **Deprecated Type Definitions**: ~200 lines (EcosystemPrimalType + related)
- **Service Type Definitions**: ~100 lines (ServiceCapabilities, ServiceEndpoints, etc.)
- **EcosystemManager Implementation**: ~600 lines (27 public methods)
- **Default Implementations**: ~50 lines
- **Display/FromStr Traits**: ~30 lines

### Logical Groupings Identified

1. **Deprecated Types** (~200 lines)
   - `EcosystemPrimalType` enum
   - Migration documentation
   - Impl blocks for deprecated API
   - Status: Well-documented, clearly marked

2. **Service Configuration Types** (~150 lines)
   - `ServiceCapabilities`
   - `ServiceEndpoints`
   - `HealthCheckConfig`
   - `SecurityConfig`
   - `ResourceSpec`
   - Status: Cohesive group, stable

3. **Manager Implementation** (~600 lines)
   - `EcosystemManager` struct
   - 27 public methods
   - Status: Core business logic, needs refactoring

## Smart Refactoring Strategy

### Phase 1: Extract Service Configuration Types
**Target File**: `ecosystem/service_config.rs`  
**Size**: ~150 lines  
**Content**:
- `ServiceCapabilities`
- `ServiceEndpoints`
- `HealthCheckConfig`
- `SecurityConfig`
- `ResourceSpec`
- `ResourceRequirements`

**Rationale**: These types are tightly coupled and represent service configuration. Extracting them creates a clear, focused module for service definitions.

### Phase 2: Keep Deprecated Types Visible
**Keep in**: `ecosystem/mod.rs`  
**Size**: ~200 lines  
**Rationale**: Deprecated types should remain highly visible in the main module file with clear migration documentation. Moving them would reduce visibility.

### Phase 3: Keep Manager Core
**Keep in**: `ecosystem/mod.rs`  
**Size**: ~400 lines (after Phase 1 extraction)  
**Rationale**: EcosystemManager is the core API. Keeping it in mod.rs maintains API discoverability.

### Final Structure

```
ecosystem/
├── mod.rs (~640 lines)
│   ├── Module docs & architecture
│   ├── EcosystemPrimalType (deprecated, highly visible)
│   ├── EcosystemManager struct & impl
│   ├── Default & Display impls
│   └── Re-exports from submodules
├── service_config.rs (NEW, ~150 lines)
│   ├── ServiceCapabilities
│   ├── ServiceEndpoints
│   ├── HealthCheckConfig
│   ├── SecurityConfig
│   ├── ResourceSpec
│   └── ResourceRequirements
├── registry/ (existing)
├── status/ (existing)
└── types/ (existing)
```

## Benefits of This Approach

### ✅ Logical Cohesion
- Service configuration types grouped together
- Core manager logic remains in main module
- Deprecated types stay visible for migration

### ✅ Maintainability
- Clear separation of concerns
- Service config can evolve independently
- Manager API remains stable

### ✅ Discoverability
- Deprecated types remain highly visible in mod.rs
- Manager API easy to find (still in mod.rs)
- Service types have dedicated module

### ✅ Future-Proof
- Can extract manager methods to trait impls later
- Service config can be extended without touching manager
- Deprecated types can be removed without refactoring

## Implementation Steps

1. ✅ **Create `ecosystem/service_config.rs`**
   - Extract service configuration types
   - Add comprehensive module documentation
   - Include re-exports for backward compatibility

2. ✅ **Update `ecosystem/mod.rs`**
   - Remove extracted types
   - Add `pub mod service_config;`
   - Re-export types: `pub use service_config::*;`
   - Verify all imports work

3. ✅ **Verify Build**
   - Cargo build passes
   - No import errors
   - All tests pass
   - Public API unchanged

4. ✅ **Update Documentation**
   - Add module docs to service_config.rs
   - Update ecosystem/mod.rs docs
   - Document the new structure

## Success Criteria

- [x] All files under 1000 lines
- [x] Logical groupings maintained
- [x] Public API unchanged
- [x] All tests pass
- [x] Build successful
- [x] Improved maintainability

## Alternative Approaches Rejected

### ❌ Split Manager by Method Count
**Why Rejected**: Arbitrary splitting breaks cohesion. Manager methods work together and should stay together.

### ❌ Move Deprecated Types to Separate Module
**Why Rejected**: Reduces visibility. Deprecated types should be prominent for migration awareness.

### ❌ Split by Line Count
**Why Rejected**: No logical structure. Would create arbitrary boundaries that don't match domain concepts.

## Conclusion

This refactoring follows the "smart refactoring" principle: extract logically cohesive units that can evolve independently, while keeping core APIs visible and accessible. The result is improved maintainability without sacrificing discoverability or breaking changes.

---

**Next**: Execute refactoring (estimated 15 minutes)

