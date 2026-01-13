# 🔧 Quick Fix Checklist - Squirrel Audit

**Date**: January 13, 2026  
**Purpose**: Immediate action items to unblock development  
**Time Estimate**: 2-4 hours

---

## 🔴 **CRITICAL BLOCKERS** (Fix First)

### 1. Fix Workspace `nix` Dependency ⏱️ 5 min

**Problem**:
```
error: `dependency.nix` was not found in `workspace.dependencies`
```

**Location**: `crates/main/Cargo.toml` line 39

**Fix**:
Add to `crates/Cargo.toml` in `[workspace.dependencies]` section:
```toml
nix = { version = "0.27", features = ["process", "signal"] }
```

**Verify**:
```bash
cargo check --workspace
```

---

### 2. Complete Plugin Metadata Migration ⏱️ 30-60 min

**Problem**: 30+ deprecation warnings
```
warning: use of deprecated struct `plugin::PluginMetadata`: 
  Use squirrel_interfaces::plugins::PluginMetadata instead
```

**Affected Files**:
- `crates/core/plugins/src/plugin.rs`
- `crates/core/plugins/src/web/adapter.rs`
- `crates/core/plugins/src/web/example.rs`
- `crates/core/plugins/src/web/api.rs`

**Fix Strategy**:
1. Replace all `use crate::plugin::PluginMetadata` with:
   ```rust
   use squirrel_interfaces::plugins::PluginMetadata;
   ```

2. Remove deprecated struct from `plugin.rs`

3. Update all usages to use the interface version

**Verify**:
```bash
cargo build --all-targets 2>&1 | grep -c "deprecated"
# Should be 0
```

---

### 3. Fix Integration Test Compilation ⏱️ 1-2 hours

**Problem**: 26 compilation errors in `crates/main/tests/integration_tests.rs`

**Sample Errors**:
```
error[E0061]: this method takes 2 arguments but 7 were supplied
error[E0308]: mismatched types
error[E0599]: no method named `expect` found
```

**Fix Strategy**:
1. Update test signatures to match current API
2. Fix `SquirrelPrimalProvider::new()` calls
3. Update method expectations
4. Fix type mismatches

**Verify**:
```bash
cargo test --test integration_tests --no-run
```

---

## 🟡 **HIGH PRIORITY** (Enable Development)

### 4. Enable Clippy ⏱️ After fixes 1-3

**Command**:
```bash
cargo clippy --all-targets --all-features -- -D warnings
```

**Expected**: Should pass after above fixes

---

### 5. Enable Formatting ⏱️ After fix 1

**Command**:
```bash
cargo fmt --all -- --check
```

**Expected**: Should pass after workspace fix

---

### 6. Measure Test Coverage ⏱️ After fix 3

**Command**:
```bash
cargo llvm-cov --all-features --workspace --summary-only
```

**Expected**: Get baseline coverage number

---

## 🟢 **QUICK WINS** (Optional but Recommended)

### 7. Implement TLS for HTTPS Fallback ⏱️ 30-45 min

**File**: `crates/main/src/rpc/https_fallback.rs`

**TODOs**:
```rust
// TODO: Implement TLS configuration
// TODO: Add certificate management
```

**Strategy**: Use `rustls` (already in dependencies)

---

### 8. Add Uptime Tracking ⏱️ 15-20 min

**Files**:
- `crates/main/src/rpc/https_fallback.rs`
- `crates/main/src/rpc/protocol_router.rs`

**Current Issue**: Health endpoint returns `uptime: 0`

**Fix**:
```rust
static START_TIME: OnceCell<Instant> = OnceCell::new();

fn init() {
    START_TIME.get_or_init(|| Instant::now());
}

fn get_uptime() -> Duration {
    START_TIME.get().map(|t| t.elapsed()).unwrap_or_default()
}
```

---

### 9. Document High-Traffic APIs ⏱️ Ongoing

**Priority APIs** (from TODO):
- `ai-tools/src/lib.rs` - 324 items need docs
- `ai-tools/src/router/*.rs` - Core routing logic
- `ai-tools/src/common/*.rs` - Common utilities

**Template**:
```rust
/// Brief description of what this function does.
///
/// # Arguments
///
/// * `param1` - Description
/// * `param2` - Description
///
/// # Returns
///
/// Description of return value
///
/// # Errors
///
/// When this function returns an error
///
/// # Examples
///
/// ```
/// use crate::example;
/// let result = example::function();
/// ```
```

---

## 📋 Execution Checklist

**Session 1: Unblock Builds** (30-90 min)
- [ ] 1. Fix `nix` workspace dependency
- [ ] 4. Run `cargo clippy` (should pass)
- [ ] 5. Run `cargo fmt --check` (should pass)
- [ ] Test: `cargo build --workspace` (clean build)

**Session 2: Fix Tests** (1-2 hours)
- [ ] 2. Complete plugin metadata migration
- [ ] 3. Fix integration test compilation
- [ ] 6. Measure test coverage baseline
- [ ] Test: `cargo test --workspace` (measure pass rate)

**Session 3: Quick Wins** (1-2 hours)
- [ ] 7. Implement TLS for HTTPS
- [ ] 8. Add uptime tracking
- [ ] 9. Document 10-20 high-priority APIs
- [ ] Test: Full audit re-run

---

## 🎯 Success Criteria

### After Session 1:
- ✅ `cargo build --workspace` passes
- ✅ `cargo clippy --all-targets` passes
- ✅ `cargo fmt --all --check` passes
- ✅ Zero workspace errors

### After Session 2:
- ✅ Plugin metadata migration complete (0 deprecation warnings)
- ✅ Integration tests compile
- ✅ Test coverage measured (baseline established)
- ✅ Test compilation error count reduced to 0

### After Session 3:
- ✅ TLS implemented and configured
- ✅ Uptime tracking working
- ✅ 10-20 critical APIs documented
- ✅ Ready for systematic TODO reduction

---

## 🚀 Expected Outcomes

**Before**:
- Grade: B+ (83/100)
- Clippy: FAILING
- Tests: BLOCKED
- Warnings: 30+ deprecations

**After Quick Fixes**:
- Grade: B+ → A- (88/100)
- Clippy: PASSING
- Tests: COMPILING + MEASURABLE
- Warnings: 0

**Time Investment**: 2-4 hours  
**Grade Improvement**: +5 points  
**Value**: Unblocks all development workflows

---

## 📝 Notes

### Workspace Structure
The project has multiple Cargo workspaces:
- Root workspace: `Cargo.toml`
- Crates workspace: `crates/Cargo.toml`
- Sub-crate workspaces: `core/mcp`, `core/core`, `core/plugins`, `core/auth`

When adding dependencies, ensure they're in the correct workspace.

### Testing Strategy
After fixes:
1. Run unit tests: `cargo test --lib`
2. Run integration tests: `cargo test --test '*'`
3. Run all tests: `cargo test --workspace`
4. Measure coverage: `cargo llvm-cov`

### Plugin Metadata Migration
This is a planned migration to consolidate plugin metadata definitions. The interfaces crate has the canonical version. All other uses should reference it.

---

**Created**: January 13, 2026  
**Estimated Completion**: Same day (2-4 hours)  
**Priority**: 🔴 **CRITICAL** - Blocks all development

🔧 **Let's unblock development and get back to A+ evolution!** 🚀

