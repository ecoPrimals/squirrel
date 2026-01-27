# Test Migration Strategy - Refined
**Date**: January 28, 2026, 04:00 UTC  
**Insight**: Not all `EcosystemPrimalType` refs should be removed

---

## 🎯 Key Insight

**Tests of deprecated APIs should stay** - they ensure backward compatibility.  
**Tests using deprecated APIs should migrate** - they demonstrate new patterns.

---

## 📊 Test Categories

### Category 1: Testing Deprecated API (KEEP AS-IS)
These tests verify the deprecated `EcosystemPrimalType` enum works correctly:
- `test_primal_type_variants()` - Tests enum variants
- `test_primal_type_as_str()` - Tests string conversion
- `test_primal_type_from_str()` - Tests parsing
- `test_primal_type_serialization()` - Tests serde

**Action**: ✅ **KEEP** - Add `#[allow(deprecated)]` attribute

**Rationale**: We maintain backward compatibility by keeping the deprecated API functional and tested.

### Category 2: Using Deprecated API (MIGRATE)
These tests use `EcosystemPrimalType` to test other functionality:
- Discovery tests using `find_services_by_type()`
- Coordination tests using `start_coordination()`
- Integration tests assuming specific primal types

**Action**: 🔄 **MIGRATE** - Convert to capability-based

**Pattern**:
```rust
// OLD:
#[test]
async fn test_discover_songbird() {
    let services = manager
        .find_services_by_type(EcosystemPrimalType::Songbird)
        .await?;
    assert!(!services.is_empty());
}

// NEW:
#[test]
async fn test_discover_service_mesh() {
    let services = manager
        .find_services_by_capability("service_mesh")
        .await?;
    assert!(!services.is_empty());
}
```

---

## 🎯 Revised Approach

### Step 1: Add Deprecation Allows
Add `#[allow(deprecated)]` to tests that TEST the deprecated API:
- `ecosystem_types_tests.rs` - Keep as backward compat tests
- `ecosystem_primal_type_*` tests - Keep for enum testing

### Step 2: Migrate Usage Tests
Convert tests that USE deprecated API to discover services:
- Replace `find_services_by_type()` with `find_services_by_capability()`
- Replace `EcosystemPrimalType::X` with capability strings
- Update assertions from type checks to capability checks

### Step 3: Add New Capability Tests
Create comprehensive capability-based test suites:
- `capability_discovery_tests.rs` - New discovery patterns
- `capability_integration_tests.rs` - E2E testing
- `capability_edge_cases_tests.rs` - Error handling

---

## 📈 Impact Assessment

### Original Estimate
- 324 test refs to migrate

### Refined Estimate
- Tests of deprecated API: ~100 refs (KEEP with `#[allow(deprecated)]`)
- Tests using deprecated API: ~150 refs (MIGRATE to capability-based)
- Already migrated: ~74 refs (DONE)

**Actual Migration Needed**: ~150 refs (not 324)

---

## 🚀 Execution Plan

### Phase 1: Quick Wins (30 min)
1. Add `#[allow(deprecated)]` to backward compat tests
2. Verify all tests still pass
3. Document which tests are intentionally using deprecated API

### Phase 2: Strategic Migration (60 min)
1. Migrate discovery usage tests
2. Migrate coordination usage tests
3. Add new capability-based tests

### Phase 3: Validation (15 min)
1. Run full test suite
2. Verify coverage maintained/improved
3. Update documentation

---

## ✅ Success Criteria

- [ ] Backward compat tests kept with `#[allow(deprecated)]`
- [ ] Discovery usage migrated to capability-based
- [ ] New capability tests added
- [ ] All 191+ tests passing
- [ ] Documentation updated

---

**Status**: Strategy refined - ready to execute

🐿️🦀✨ **Smart Migration** ✨🦀🐿️

