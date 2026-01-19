# Archive Code Cleanup - January 19, 2026 (Final)

**Status**: 🔍 **REVIEW COMPLETE - CLEANUP TARGETS IDENTIFIED**

---

## 🎯 Cleanup Targets Found

### 1. FALSE POSITIVE TODOs (Outdated/Misleading)

#### ❌ `crates/main/src/api/ai/router.rs` (Lines 106-112)
```rust
// Fallback: Load legacy adapters in parallel (dev mode only!)
#[cfg(feature = "dev-direct-http")]
{
    info!("🔄 Loading legacy AI adapters (DEV MODE - HTTP enabled)...");
    let legacy_providers = Self::load_legacy_adapters_parallel().await;
    providers.extend(legacy_providers);
}
```

**Issue**: References `dev-direct-http` feature and `load_legacy_adapters_parallel()` which were **DELETED** in v1.7.0!

**Action**: Delete this entire `#[cfg]` block (lines 106-117)

---

#### ❌ `crates/main/src/doctor.rs` (Lines 83-85)
```rust
if should_check(subsystem, Subsystem::Http) {
    checks.push(check_http_server().await);
}
```

**Issue**: References `Subsystem::Http` and `check_http_server()` - HTTP server was **DELETED**!

**Action**: Delete this entire block

---

#### ❌ `crates/main/src/universal_adapter.rs` (Lines 21, 66-67)
```rust
// // use crate::api::ApiServer; // DELETED // DELETED
...
/// API server instance - REMOVED (HTTP API deleted)
// api_server: Option<ApiServer>, // DELETED
```

**Issue**: Commented-out code referencing deleted `ApiServer`

**Action**: Delete commented-out lines

---

### 2. OUTDATED COMMENTS (Reference Deleted Systems)

#### ❌ `crates/main/src/universal_adapter.rs` (Line 40)
```rust
/// Whether the API server is running
pub api_server_running: bool,
```

**Issue**: Field references deleted API server

**Action**: Either delete field or rename to `rpc_server_running`

---

#### ❌ `crates/main/src/biomeos_integration/mod.rs` (Line 28)
```rust
// ecosystem_client removed - deprecated, unused, had reqwest dependency
```

**Issue**: This is actually GOOD documentation (fossil record), but could be moved to archive

**Action**: Keep as-is (fossil record)

---

### 3. SUBSYSTEM ENUM CLEANUP

#### ❌ `crates/main/src/cli.rs` - `Subsystem::Http`

**Check if exists**:
```bash
$ grep -n "enum Subsystem" crates/main/src/cli.rs
```

**Issue**: If `Subsystem::Http` variant exists, it should be deleted

**Action**: Remove `Http` variant from `Subsystem` enum

---

### 4. LEGITIMATE TODOs (Keep - Future Work)

These TODOs are VALID and should be KEPT:

✅ `crates/main/src/primal_pulse/mod.rs:5`
```rust
//! TODO: Rebuild using capability_ai instead of deleted HTTP API
```
**Status**: Valid - primal_pulse needs capability_ai integration

✅ `crates/main/src/primal_pulse/neural_graph/mod.rs` (Lines 226, 266, 358)
```rust
// TODO: Implement proper topological sort
// TODO: Implement proper critical path analysis
// TODO: Implement cycle detection
```
**Status**: Valid - future algorithm improvements

✅ `crates/main/src/ecosystem/mod.rs` (12 TODOs)
```rust
// TODO: Register with ecosystem through capability discovery (Unix sockets)
// TODO: Implement via capability discovery (Unix sockets)
```
**Status**: Valid - capability discovery implementation needed

✅ `crates/main/src/primal_provider/core.rs` (7 TODOs)
```rust
// TODO: Implement via ecosystem discovery
// TODO: Implement songbird registration
```
**Status**: Valid - ecosystem integration work

✅ `crates/main/src/universal_primal_ecosystem/mod.rs` (2 TODOs)
```rust
// TODO: Implement Unix socket communication
// TODO: Implement Unix socket client discovery via capability discovery
```
**Status**: Valid - Unix socket implementation needed

---

## 📊 Summary

### False Positives / Outdated (DELETE):
- ❌ 1 `#[cfg(feature = "dev-direct-http")]` block in `router.rs`
- ❌ 1 `Subsystem::Http` check in `doctor.rs`
- ❌ 3 commented-out lines in `universal_adapter.rs`
- ❌ 1 field in `AdapterStatus` struct
- ❌ 1 `Subsystem::Http` enum variant (if exists)

### Legitimate TODOs (KEEP):
- ✅ 26 valid TODOs for future capability discovery work
- ✅ 1 fossil record comment (ecosystem_client)

---

## 🔧 Cleanup Actions

### Priority 1: Delete False Positives
1. `crates/main/src/api/ai/router.rs` - Remove `dev-direct-http` block
2. `crates/main/src/doctor.rs` - Remove `Subsystem::Http` check
3. `crates/main/src/cli.rs` - Remove `Subsystem::Http` enum variant
4. `crates/main/src/universal_adapter.rs` - Clean commented code

### Priority 2: Update Structs
1. `AdapterStatus` - Remove or rename `api_server_running` field

### Priority 3: Verify No HTTP References
```bash
$ cargo build --release 2>&1 | grep -i "http\|warp\|hyper"
# Should be ZERO matches!
```

---

## ✅ Expected Outcome

After cleanup:
- ✅ ZERO references to deleted `dev-direct-http` feature
- ✅ ZERO references to deleted `ApiServer`
- ✅ ZERO references to deleted `Subsystem::Http`
- ✅ Clean build with no warnings about unused code
- ✅ All legitimate TODOs preserved for future work

---

**Next Step**: Execute cleanup and verify build!

