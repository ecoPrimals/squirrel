# jsonrpsee Removed - Following BearDog's Pattern

**Date**: January 19, 2026  
**Discovery**: BearDog uses manual JSON-RPC (~150 lines) instead of jsonrpsee  
**Action**: Removed jsonrpsee from Squirrel  
**Result**: One less potential C dependency path!

---

## 🎯 What Was Done

### Removed from `crates/main/Cargo.toml`

**Before**:
```toml
# JSON-RPC server for Unix socket IPC (OPTIONAL)
jsonrpsee = { version = "0.24", features = ["server"], optional = true }

[features]
jsonrpc-server = ["dep:jsonrpsee"]
```

**After**:
```toml
# JSON-RPC: Using manual implementation (BearDog pattern) - ZERO C dependencies!
# jsonrpsee removed - it pulls ring via rustls

[features]
# jsonrpc-server feature removed
```

---

## ✅ Validation

```bash
# Check dependency tree
$ cargo tree | grep jsonrpsee
# Result: ZERO! ✅

# Code usage
$ grep -r "use jsonrpsee" crates/main/src
# Result: ZERO! ✅
```

---

## 💡 Why This Matters

### The Problem

**jsonrpsee dependency chain**:
```
jsonrpsee → rustls → ring (C dependency!)
```

Even though it was optional and feature-gated, it was a potential path to C dependencies.

### BearDog's Solution

**Manual JSON-RPC (~150 lines)**:
```rust
// Just serde_json!
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: Option<serde_json::Value>,
    pub id: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub result: Option<serde_json::Value>,
    pub error: Option<JsonRpcError>,
    pub id: serde_json::Value,
}

// ~150 lines total!
```

**Benefits**:
- ✅ 100% Pure Rust
- ✅ Zero C dependencies
- ✅ Full control
- ✅ Simpler
- ✅ Faster compile
- ✅ Smaller binary

---

## 📊 Impact

### Squirrel's Status

**Before Removal**:
- jsonrpsee: Optional dependency
- Feature: `jsonrpc-server`
- Usage: ZERO (never used!)
- Impact: Potential C dependency if enabled

**After Removal**:
- jsonrpsee: GONE ✅
- Feature: REMOVED ✅
- Usage: Still ZERO ✅
- Impact: One less C dependency path!

---

## 🎯 What's Left for 100% Pure Rust

### Current Status

✅ **Dependencies Removed**:
- jsonwebtoken → Delegated to BearDog (capability_crypto)
- AI providers (10,251 lines) → Delegated to Songbird (capability_ai)
- ecosystem_client (835 lines) → Deleted
- jsonrpsee → Removed (this change!)

🚧 **Remaining** (test harness code):
- reqwest usage in 17 files (test/integration utilities)
- Estimated: 2-3 hours of cleanup

### The Path Forward

1. ✅ **jsonrpsee**: DONE! (5 minutes)
2. 🚧 **reqwest test harness**: Audit and clean (2-3 hours)
3. ⏳ **Validation**: Build, test, cross-compile (15 min)
4. 🎉 **100% Pure Rust**: Victory!

---

## 💬 Key Learnings

### BearDog Showed the Way

**Why manual JSON-RPC is better**:
1. Simpler - JSON-RPC spec is straightforward
2. Faster - No heavy dependencies
3. Pure Rust - Zero C dependencies
4. Full control - Custom error handling
5. Proven - BearDog uses it in production!

### Squirrel's Implementation

We don't even need manual JSON-RPC yet! We use:
- Unix sockets for IPC ✅
- capability_ai for AI delegation ✅
- Direct Unix socket communication ✅

If we ever need JSON-RPC, we'll copy BearDog's ~150 line implementation.

---

## 🏆 Bottom Line

**What was accomplished**:
- ✅ Removed jsonrpsee dependency
- ✅ Removed jsonrpc-server feature
- ✅ Validated: ZERO jsonrpsee in tree
- ✅ Following BearDog's proven pattern

**Time spent**: 5 minutes

**Impact**: Removed potential C dependency path

**Remaining**: Test harness cleanup (2-3 hours)

---

**The ecological way - remove what we don't use, follow proven patterns!** 🌍🦀✨

