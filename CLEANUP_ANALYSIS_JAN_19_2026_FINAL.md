# Archive Code Cleanup Analysis - January 19, 2026 (Final)

**Context**: After v1.6.0 HTTP debt cleanup, review for remaining orphaned code.

---

## 🎯 Cleanup Targets Identified

### 1. ORPHANED HTTP API TESTS (8 files, ~55KB) ⚠️ HIGH PRIORITY

**Location**: `crates/main/src/api/*_tests.rs`

**Files**:
- `api/ecosystem_tests.rs` (9.1K)
- `api/ecosystem_edge_case_tests.rs` (12K)
- `api/health_tests.rs` (6.7K)
- `api/health_edge_case_tests.rs` (9.0K)
- `api/metrics_tests.rs` (4.1K)
- `api/metrics_edge_case_tests.rs` (9.5K)
- `api/management_tests.rs` (2.5K)
- `api/service_mesh_tests.rs` (8.1K)

**Reason for Deletion**:
- These tests validate HTTP endpoints that were **DELETED** in v1.6.0
- Test modules like `health`, `metrics`, `ecosystem`, `service_mesh` no longer exist
- Squirrel now uses Unix sockets + JSON-RPC + tarpc (NO HTTP!)

**Impact**: **ZERO** - These tests cannot run (missing imports!)

---

### 2. LEGACY API MODULE (Partial Cleanup) ⚠️ MEDIUM PRIORITY

**Location**: `crates/main/src/api/`

**Current State**:
- `mod.rs`: Stale docs referencing deleted `ApiServer`, but exports `types`
- `types.rs`: Kept for "backward compat" but may be unused
- `ai/` subdirectory: **STILL IN USE by tarpc_server!**

**Investigation**:
```bash
$ grep -r "use crate::api::ai" crates/
crates/main/src/rpc/tarpc_server.rs:use crate::api::ai::AiRouter;
```

**Decision**: **KEEP** `api/ai/` for now (used by tarpc!)
- `crates/main/src/rpc/tarpc_server.rs` depends on `AiRouter`
- This is a valid use case (tarpc RPC AI routing)

**Action**:
1. ✅ **KEEP**: `api/ai/` (actively used by tarpc_server)
2. ⚠️ **DELETE**: `api/*_tests.rs` (orphaned HTTP tests)
3. 🤔 **INVESTIGATE**: `api/types.rs` and `api/mod.rs` - are they used?

---

### 3. PRIMAL_PULSE MODULE (~1,149 lines) ⚠️ LOW PRIORITY

**Location**: `crates/main/src/primal_pulse/`

**Current State**:
```rust
//! TODO: Rebuild using capability_ai instead of deleted HTTP API

// Legacy modules REMOVED - used deleted HTTP API (api::ai)
// pub(crate) mod handlers; // DELETED
// mod tools;                // DELETED  
// pub use tools::register_primal_pulse_tools; // DELETED

// Remaining modules (may need updates)
pub mod neural_graph;  // ~1,000 lines
mod schemas;
```

**Used By**:
- `crates/main/src/lib.rs` - references `primal_pulse` module
- `crates/main/src/main.rs` - references `primal_pulse` module

**Analysis**:
- `neural_graph/` appears to be a working module (dependency graph analysis)
- No active imports of `primal_pulse` found (unused?)
- **TODO comment** indicates it needs rebuilding for capability_ai

**Decision**: **INVESTIGATE FURTHER** - May be intentionally disabled for future work

---

### 4. OUTDATED TODOs (~26 instances)

**Categories**:

**A. Capability Discovery (Likely Already Done?)**:
```rust
// crates/main/src/ecosystem/mod.rs
// TODO: Register with ecosystem through capability discovery (Unix sockets)
// TODO: Deregister through capability discovery (Unix sockets)
// TODO: Implement via capability discovery (Unix sockets)
```

**Impact**: These may be **FALSE POSITIVES** - Unix socket capability discovery is **ALREADY IMPLEMENTED** via `capability_http` and `capability_crypto`!

**B. Unix Socket Implementation**:
```rust
// crates/main/src/universal_primal_ecosystem/mod.rs
// TODO: Implement Unix socket communication
// TODO: Implement Unix socket client discovery via capability discovery
```

**Impact**: May be **FALSE POSITIVES** if Unix sockets are already working.

**C. Service Discovery Stubs**:
```rust
// crates/main/src/primal_provider/core.rs
let available_primals: Vec<serde_json::Value> = Vec::new(); // TODO: Implement via ecosystem discovery
```

**Impact**: Legitimate TODOs for future capability discovery implementation.

---

## 📊 Summary

### Recommended Actions

| Item | Lines | Action | Confidence |
|------|-------|--------|------------|
| HTTP API tests | ~2,000+ | **DELETE** | 100% |
| `api/types.rs` | ~400 | **INVESTIGATE** | 50% |
| `api/mod.rs` | ~55 | **UPDATE DOCS** | 100% |
| `primal_pulse/` | ~1,149 | **KEEP** (future work) | 70% |
| Outdated TODOs | N/A | **AUDIT & UPDATE** | 80% |

### Estimated Impact

**Files to Delete**: 8 (HTTP test files)  
**Lines to Remove**: ~2,000+  
**Build Impact**: ZERO (tests are already failing/skipped)  
**Functionality Impact**: ZERO (tests orphaned HTTP endpoints)

---

## 🚀 Execution Plan

### Phase 1: Safe Deletions (HIGH CONFIDENCE)
1. ✅ Delete 8 orphaned HTTP API test files
2. ✅ Update `api/mod.rs` docs (remove ApiServer references)

### Phase 2: Investigation (MEDIUM CONFIDENCE)
3. 🔍 Check if `api/types.rs` is used anywhere
4. 🔍 Audit TODO comments for false positives

### Phase 3: Future Work (LOW PRIORITY)
5. 📋 Document `primal_pulse` status (intentionally disabled? needs rebuild?)
6. 📋 Create issues for legitimate TODOs

---

**Next Step**: Execute Phase 1 (safe deletions)

