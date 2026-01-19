# Phase 3 Remaining Work - TRUE ecoBin Evolution

**Date**: January 19, 2026  
**Status**: Phase 3 ~60% complete (3/8 crates done)  
**Blocker**: Workspace-wide `reqwest` dependency

---

## ✅ Completed (3/8 crates)

1. ✅ **squirrel-ai-tools** - reqwest optional, capability-ai default
2. ✅ **squirrel-mcp-config** - reqwest optional, http-config feature
3. ✅ **squirrel-mcp** - reqwest optional, direct-http feature

---

## ⏳ Remaining (5/8 crates)

All use `reqwest` from workspace or directly:

1. **ecosystem-api** - Uses `reqwest.workspace = true`
2. **squirrel-core** - Uses reqwest directly
3. **squirrel-mcp-auth** - Uses reqwest directly  
4. **universal-patterns** - Uses `reqwest.workspace = true`
5. **main (squirrel)** - Uses reqwest directly

---

## 🎯 Root Cause

**Workspace Cargo.toml** (line 65):
```toml
reqwest = { version = "0.11", default-features = false, features = ["json", "rustls-tls"] }
```

This makes reqwest available to all workspace members via `reqwest.workspace = true`.

---

## 🔧 Solution Approaches

### Option A: Remove from Workspace (Recommended)

**Strategy**: Remove `reqwest` from `[workspace.dependencies]`, make each crate declare it optionally.

**Impact**:
- Clean separation: capability-based crates don't pull reqwest
- Each crate controls its own features
- More explicit dependencies

**Work**:
1. Remove reqwest from workspace Cargo.toml
2. Update 5 remaining crates to declare reqwest as `optional = true`
3. Add feature flags in each crate
4. Test builds with/without features

**Time**: ~2-3 hours

### Option B: Conditional Workspace Features

**Strategy**: Make workspace reqwest conditional on a workspace-level feature.

**Problem**: Workspace dependencies can't be optional in Rust (as of 2026).

**Status**: Not possible with current Cargo.

### Option C: Hybrid Approach (Pragmatic)

**Strategy**: 
1. Keep workspace reqwest for crates that truly need HTTP (ecosystem-api)
2. Make it optional in individual crates that can work without it
3. Document which crates are "Pure Rust capable"

**Time**: ~1 hour

---

## 📋 Detailed Plan (Option A - Recommended)

### Step 1: Remove from Workspace (5 min)

```toml
# In Cargo.toml, REMOVE this line:
# reqwest = { version = "0.11", default-features = false, features = ["json", "rustls-tls"] }
```

### Step 2: Update ecosystem-api (15 min)

```toml
# crates/ecosystem-api/Cargo.toml
[dependencies]
reqwest = { version = "0.11", default-features = false, features = ["json", "rustls-tls"], optional = true }

[features]
default = []
http-api = ["dep:reqwest"]
```

### Step 3: Update squirrel-core (20 min)

```toml
# crates/core/core/Cargo.toml
[dependencies]
reqwest = { version = "0.11", default-features = false, features = ["json", "stream", "rustls-tls"], optional = true }

[features]
default = []
http-client = ["dep:reqwest"]
```

### Step 4: Update squirrel-mcp-auth (15 min)

Already has local-jwt feature. Add http feature:

```toml
[dependencies]
reqwest = { version = "0.11", default-features = false, features = ["json", "rustls-tls"], optional = true }

[features]
default = []
http-auth = ["dep:reqwest"]
local-jwt = ["dep:jsonwebtoken"]  # existing
```

### Step 5: Update universal-patterns (15 min)

```toml
# crates/universal-patterns/Cargo.toml
[dependencies]
reqwest = { version = "0.11", default-features = false, features = ["json", "rustls-tls"], optional = true }

[features]
default = []
http-patterns = ["dep:reqwest"]
```

### Step 6: Update main crate (20 min)

```toml
# crates/main/Cargo.toml
[dependencies]
reqwest = { version = "0.11", default-features = false, features = ["json", "rustls-tls"], optional = true }

[features]
default = ["capability-ai"]
capability-ai = []  # Pure Rust!
dev-http = ["dep:reqwest"]  # For development
```

### Step 7: Feature-gate code (30-60 min)

For each crate, wrap reqwest usage:

```rust
#[cfg(feature = "http-client")]
use reqwest;

#[cfg(feature = "http-client")]
pub async fn fetch_from_http(url: &str) -> Result<String> {
    // ... reqwest code ...
}

#[cfg(not(feature = "http-client"))]
pub async fn fetch_from_http(_url: &str) -> Result<String> {
    Err(Error::NotSupported("HTTP client not enabled. Use capability-based client or enable 'http-client' feature.".into()))
}
```

### Step 8: Test & Validate (30 min)

```bash
# Should have NO ring!
cargo tree --no-default-features | grep ring

# Should have ring (dev mode)
cargo tree --features dev-http | grep ring

# All tests should pass
cargo test --workspace --no-default-features
```

---

## 🎯 Expected Outcome

After Option A:

**Default build** (Pure Rust!):
```bash
cargo build
cargo tree | grep ring  # EMPTY! ✅
```

**Dev build** (with HTTP):
```bash
cargo build --features dev-http
cargo tree | grep ring  # Shows ring (acceptable for dev) ⚠️
```

**Production build** (Pure Rust!):
```bash
cargo build --release
# NO ring, uses Unix sockets for all external communication ✅
```

---

## 📊 Complexity Analysis

**Option A**:
- Complexity: Medium
- Time: 2-3 hours
- Result: TRUE ecoBin achievable
- Maintainability: High (explicit dependencies)

**Option C**:
- Complexity: Low
- Time: 1 hour  
- Result: Partial (some crates still have ring)
- Maintainability: Medium (mixed approach)

---

## 🚀 Recommendation

**Go with Option A**: Remove reqwest from workspace, make optional in each crate.

**Rationale**:
1. Clean architecture (capability-based = Pure Rust)
2. Explicit about HTTP dependencies
3. TRUE ecoBin achievable
4. Time investment worth it (2-3 hours for TRUE certification)

---

## 📝 Next Session Checklist

- [ ] Remove reqwest from workspace Cargo.toml
- [ ] Update ecosystem-api Cargo.toml + feature
- [ ] Update squirrel-core Cargo.toml + feature
- [ ] Update squirrel-mcp-auth Cargo.toml + feature
- [ ] Update universal-patterns Cargo.toml + feature
- [ ] Update main crate Cargo.toml + feature
- [ ] Feature-gate reqwest usage in code
- [ ] Test: `cargo tree --no-default-features | grep ring` → EMPTY
- [ ] Test: All tests pass
- [ ] Proceed to Phase 5 (Validation)

---

*Estimated time to TRUE ecoBin: 2-3 hours*  
*Current progress: 60% of Phase 3 complete*

🌍🦀 The ecological way: remove HTTP, use Unix sockets! 🦀🌍

