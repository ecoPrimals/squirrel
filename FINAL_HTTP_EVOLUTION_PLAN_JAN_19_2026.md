# Final HTTP Evolution Plan - January 19, 2026

**Mission**: Complete socket migration - eliminate ALL remaining HTTP/feature-gated code

---

## 🔍 Current Feature Gate Inventory

### 1. `monitoring` Feature (MetricsCollector)
**Files**: `main.rs`, `lib.rs`  
**Status**: ⚠️ Feature-gated  
**Usage**: Optional metrics collection

**Current State**:
```rust
#[cfg(feature = "monitoring")]
use squirrel::monitoring::metrics::MetricsCollector;
```

**Evolution Path**:
- **Option A**: Remove entirely (monitoring via external tools)
- **Option B**: Evolve to RPC metrics endpoint (tarpc `get_metrics()`)
- **Option C**: Keep as-is (already optional, no HTTP dependency)

**Recommendation**: **Option C** - Already clean! No HTTP, just optional metrics.

---

### 2. `tarpc-rpc` Feature (tarpc Server/Client)
**Files**: `rpc/mod.rs`  
**Status**: ⚠️ Feature-gated  
**Usage**: High-performance binary RPC

**Current State**:
```rust
#[cfg(feature = "tarpc-rpc")]
pub mod tarpc_server;
```

**Evolution Path**:
- **Remove feature gate** - Make tarpc the DEFAULT!
- tarpc is ecoPrimals standard (NOT HTTP!)
- Should be core, not optional

**Recommendation**: **REMOVE FEATURE GATE** - Make tarpc core functionality!

---

### 3. `dev-direct-http` Feature (HTTP Adapters)
**Files**: `api/ai/router.rs`  
**Status**: ⚠️ Feature-gated for DEV MODE ONLY  
**Usage**: Direct HTTP to AI vendors (OpenAI, Ollama, HuggingFace)

**Current State**:
```rust
#[cfg(feature = "dev-direct-http")]
use super::adapters::{HuggingFaceAdapter, OllamaAdapter, OpenAIAdapter};
```

**Evolution Path**:
- **DELETE** - These are legacy HTTP adapters!
- Production uses `capability_ai` (delegates to Songbird via Unix sockets)
- Dev mode can also use Unix socket delegation

**Recommendation**: **DELETE** - Evolve dev mode to Unix sockets!

---

### 4. `nvml` Feature (NVIDIA GPU Monitoring)
**Files**: `hardware/gpu.rs`  
**Status**: ⚠️ Feature-gated  
**Usage**: Optional NVIDIA GPU metrics

**Current State**:
```rust
#[cfg(feature = "nvml")]
use nvml_wrapper::Nvml;
```

**Evolution Path**:
- **Keep as-is** - Hardware-specific, no HTTP dependency
- Optional for non-NVIDIA systems

**Recommendation**: **KEEP** - Already clean, hardware-specific!

---

### 5. `ecosystem` Feature
**Files**: `lib.rs`  
**Status**: ⚠️ Feature-gated  
**Usage**: Ecosystem manager

**Current State**:
```rust
#[cfg(feature = "ecosystem")]
pub use ecosystem::EcosystemManager;
```

**Evolution Path**:
- **Remove feature gate** - Core functionality!
- Ecosystem integration is essential for capability discovery

**Recommendation**: **REMOVE FEATURE GATE** - Make ecosystem core!

---

### 6. `reqwest` Dependency (⚠️ CRITICAL!)
**Files**: `biomeos_integration/mod.rs`  
**Status**: ⚠️ Still in Cargo.toml (optional)  
**Usage**: Legacy HTTP client

**Current State**:
```toml
reqwest = { version = "0.11", default-features = false, features = ["json", "rustls-tls"], optional = true }
```

**Evolution Path**:
- **DELETE ENTIRELY** - Use `capability_http` instead!
- All HTTP should go through Songbird via Unix sockets

**Recommendation**: **DELETE** - Complete the Pure Rust evolution!

---

## 🎯 Execution Plan

### Phase 1: Make Core Features DEFAULT (30 min)
1. ✅ Remove `tarpc-rpc` feature gate → Make tarpc core
2. ✅ Remove `ecosystem` feature gate → Make ecosystem core
3. ✅ Update `Cargo.toml` default features

### Phase 2: Delete Legacy HTTP Code (45 min)
4. ✅ Delete `dev-direct-http` feature and HTTP adapters
5. ✅ Delete `reqwest` dependency
6. ✅ Update `api/ai/router.rs` to use only `capability_ai`
7. ✅ Clean up `biomeos_integration/mod.rs` reqwest references

### Phase 3: Clean Build Validation (15 min)
8. ✅ Build without features: `cargo build --no-default-features`
9. ✅ Build with all features: `cargo build --all-features`
10. ✅ Validate 0 HTTP symbols: `nm target/release/squirrel | grep -iE "(hyper|warp|reqwest)"`

### Phase 4: Documentation (15 min)
11. ✅ Update `Cargo.toml` feature documentation
12. ✅ Update `README.md` with new defaults
13. ✅ Archive this plan

---

## 📊 Expected Impact

| Item | Before | After | Change |
|------|--------|-------|--------|
| Feature gates | 6 | 2 (nvml, monitoring) | -4 |
| HTTP dependencies | 1 (reqwest) | 0 | -1 |
| Default features | `capability-ai`, `ecosystem` | Add `tarpc-rpc`, `ecosystem` | +1 |
| HTTP adapters | 3 (dev mode) | 0 | -3 |
| Build config | Complex | Simple | ✅ |

---

## ✅ Success Criteria

1. ✅ tarpc RPC is DEFAULT (not optional)
2. ✅ Ecosystem is DEFAULT (not optional)
3. ✅ ZERO `reqwest` dependency
4. ✅ ZERO HTTP adapters (even in dev mode)
5. ✅ ZERO HTTP symbols in binary
6. ✅ Clean build with `--no-default-features`
7. ✅ Only 2 optional features: `monitoring`, `nvml` (both hardware-specific)

---

## 🚀 Benefits

1. **Simpler Configuration**
   - Fewer feature flags to manage
   - Clear defaults

2. **100% Socket-Based**
   - tarpc for high-performance RPC
   - Unix sockets for biomeOS integration
   - NO HTTP anywhere!

3. **ecoPrimals Compliant**
   - JSON-RPC + tarpc (standard!)
   - Capability-based discovery
   - NO HTTP frameworks

4. **Better Developer Experience**
   - Default build "just works"
   - No confusion about which features to enable

---

**Ready to execute!** 🎯

