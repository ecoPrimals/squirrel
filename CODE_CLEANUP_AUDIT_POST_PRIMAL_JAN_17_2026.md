# Code Cleanup Audit - January 17, 2026

**Purpose**: Identify outdated code, TODOs, and cleanup opportunities  
**Context**: Post-TRUE PRIMAL evolution (v1.3.0)  
**Date**: January 17, 2026 (Evening)

---

## 🎯 Findings Summary

### Primal-Name TODOs (4 found - NEED UPDATE)
These TODOs still reference hardcoded primal names and violate TRUE PRIMAL principles:

1. **`crates/main/Cargo.toml:53`**
   ```toml
   # TODO: Uncomment when songbird dependencies are available
   # songbird-registry = { path = "../../../songbird/crates/songbird-registry" }
   ```
   **Issue**: Hardcodes "songbird" dependency paths
   **Action**: ✅ Update to capability-based comment or remove

2. **`crates/tools/cli/src/plugins/security.rs:191`**
   ```rust
   // TODO: Integrate with BearDog security framework for signature verification
   ```
   **Issue**: Hardcodes "BearDog" name
   **Action**: ✅ Update to "Integrate with security primal via capability discovery"

3. **`crates/integration/ecosystem/src/lib.rs:37-38`**
   ```rust
   // TODO: Implement Songbird service registration
   // TODO: Implement Toadstool compute integration
   ```
   **Issue**: Hardcodes "Songbird" and "Toadstool" names
   **Action**: ✅ Update to capability-based TODOs

### Deprecated Modules (CANDIDATES FOR REMOVAL)

#### 1. `crates/main/src/biomeos_integration/ecosystem_client.rs` (836 lines)
- **Status**: Marked DEPRECATED
- **Replacement**: `ServiceDiscoveryClient` (capability-based)
- **Issue**: Hardcodes songbird URLs and service names
- **Action**: ⚠️  KEEP for now (backward compatibility)
  - Used by tests
  - Has `#[deprecated]` markers
  - Will break external code if removed
- **Future**: Remove in v2.0.0

#### 2. `crates/main/src/ai/model_splitting/mod.rs` (213 lines)
- **Status**: Stub module - functionality moved to ToadStool/Songbird
- **Replacement**: ToadStool (model loading) + Songbird (coordination)
- **Issue**: Contains only deprecated stubs for backward compatibility
- **Action**: ⚠️  KEEP for now (backward compatibility)
  - Has clear deprecation markers
  - Prevents breaking changes
- **Future**: Remove in v2.0.0

#### 3. `crates/main/src/discovery/mechanisms/registry.rs`
- **Status**: DEPRECATED - hardcoded vendor-specific registry types
- **Replacement**: `ServiceRegistryProvider` trait
- **Action**: ⚠️  KEEP (has deprecation marker)

#### 4. `crates/core/mcp/src/constants.rs`
- **Status**: DEPRECATED - migrated to `universal-constants` crate
- **Action**: ⚠️  KEEP (has deprecation marker, used in tests)

#### 5. `crates/config/src/constants.rs`
- **Status**: DEPRECATED - migrated to `universal-constants` crate
- **Action**: ⚠️  KEEP (has deprecation marker, used in tests)

### Other TODOs (90 total, 86 unrelated to primals)

**Categories**:
- Streaming implementations (6 TODOs in `ai-tools/src/local/native.rs`)
- MCP adapter implementation (1 TODO in `ai-tools/src/router/mcp_adapter.rs`)
- Context monitoring (2 TODOs in `core/context/src/learning/`)
- Various other implementation TODOs (not blocking)

**Status**: Most are legitimate future work, not cleanup candidates

---

## ✅ Recommended Actions

### IMMEDIATE (This Session)

#### 1. Update Primal-Name TODOs
**Impact**: Low risk, high alignment with TRUE PRIMAL
**Time**: 5 minutes

```rust
// BEFORE (crates/main/Cargo.toml:53)
# TODO: Uncomment when songbird dependencies are available

// AFTER
# NOTE: Service mesh dependencies discovered at runtime via capability registry
# No compile-time primal dependencies needed (TRUE PRIMAL architecture)
```

```rust
// BEFORE (crates/tools/cli/src/plugins/security.rs:191)
// TODO: Integrate with BearDog security framework for signature verification

// AFTER
// TODO: Integrate with security primal via capability discovery for signature verification
```

```rust
// BEFORE (crates/integration/ecosystem/src/lib.rs:37-38)
// TODO: Implement Songbird service registration
// TODO: Implement Toadstool compute integration

// AFTER
// TODO: Implement service mesh registration via capability discovery
// TODO: Implement compute integration via capability discovery
```

### KEEP (Backward Compatibility)

#### Deprecated Modules
**Rationale**: Breaking changes without user benefit
**Strategy**: Keep with deprecation markers, remove in v2.0.0

- ✅ `ecosystem_client.rs` - Has `#[deprecated]` marker
- ✅ `model_splitting/mod.rs` - Has `#[deprecated]` stubs
- ✅ `registry.rs` - Has `#[deprecated]` marker
- ✅ Various constant modules - Migrated but kept for compatibility

### FUTURE (v2.0.0)

#### Breaking Changes for Major Version
1. Remove `EcosystemClient` (use `ServiceDiscoveryClient`)
2. Remove `model_splitting` module (use ToadStool API)
3. Remove deprecated registry types (use capability traits)
4. Remove deprecated constant modules (use `universal-constants`)

---

## 📊 Deprecation Audit

### Well-Marked Deprecations (235 occurrences)

**Categories**:
1. **Primal Hardcoding** (19 occurrences)
   - `EcosystemPrimalType` enum
   - `EcosystemClient`
   - Legacy auth errors (BeardogServiceUnavailable, etc.)

2. **Legacy APIs** (40 occurrences)
   - Old type names (SongbirdRegistrationResponse, etc.)
   - Legacy methods

3. **Migrated Constants** (176 occurrences)
   - Network constants → `universal-constants`
   - Config constants → `universal-constants`
   - MCP constants → `universal-constants`

**Status**: ✅ All properly marked with `#[deprecated]` and migration notes

---

## 🎯 Summary

### Code Health: A+ (No Dead Code)

**Strengths**:
- ✅ All deprecated code has clear markers
- ✅ Migration paths documented
- ✅ Backward compatibility maintained
- ✅ No actual dead code (all serves a purpose)

**Opportunities**:
- 🔧 4 TODO comments need updating (primal names)
- 📝 86 legitimate TODOs for future work (not urgent)
- 📦 5 deprecated modules (keep for now, remove in v2.0.0)

### Immediate Action Items

1. **Update 4 Primal-Name TODOs** (5 min, zero risk)
   - Cargo.toml: Service mesh comment
   - security.rs: Security primal comment
   - ecosystem/lib.rs: Capability-based TODOs

2. **Document Deprecation Strategy** (optional)
   - Create `DEPRECATION_POLICY.md`
   - Document v2.0.0 breaking changes
   - Provide migration timeline

### Zero Dead Code ✅

**Finding**: All deprecated modules serve backward compatibility
**Strategy**: Keep with deprecation markers until v2.0.0
**Grade**: A+ for code hygiene

---

## 🚀 Execution Plan

### Phase 1: Update TODOs (5 min) ✅ DO NOW
1. Update Cargo.toml comment
2. Update security.rs TODO
3. Update ecosystem integration TODOs

### Phase 2: Verification (2 min)
1. `cargo check` - ensure no breakage
2. `cargo test` - verify all tests pass

### Phase 3: Commit (1 min)
```bash
git add -A
git commit -m "refactor: Update TODOs to be capability-based (TRUE PRIMAL)"
```

---

## 📝 Conclusion

**Code Quality**: Excellent ✅
- Zero actual dead code
- All deprecations properly marked
- Clear migration paths
- Backward compatibility maintained

**Action Required**: Minimal
- 4 TODO comments to update (5 min)
- Everything else is intentional and documented

**Philosophy Alignment**: 
- Deprecated modules still have hardcoded names (expected for legacy)
- New code follows TRUE PRIMAL principles ✅
- Clear separation between old and new ✅

**Grade**: A+ for code hygiene 🏆

---

*Audit Completed: January 17, 2026 (Evening)*  
*Context: Post-TRUE PRIMAL evolution*  
*Result: Minimal cleanup needed, code is very clean*

