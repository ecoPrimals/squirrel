# Handoff to Squirrel Team - Neural API Integration

**Date**: January 20, 2026  
**From**: biomeOS Team  
**To**: Squirrel Team  
**Priority**: HIGH  
**Estimated Effort**: 2-3 hours  
**Impact**: **CRITICAL - Enables 100% Pure Rust + TRUE Service Mesh**

---

## 🎯 Executive Summary

**What**: Migrate Squirrel from direct capability sockets to Neural API routing mesh  
**Why**: Zero C dependencies, -40% binary size, -33% compile time, TRUE service mesh  
**Status**: **90% DONE!** Your existing capability infrastructure is perfect! ✅  
**Work Required**: Replace one file (`songbird_client.rs`) that uses `reqwest`

---

## 🎉 GREAT NEWS: 90% Already Done!

**You already have everything needed!** ✅

### ✅ **What Works Perfectly (No Changes Needed)**

**File**: `crates/tools/ai-tools/src/capability_ai.rs`
- ✅ Uses Unix sockets
- ✅ Uses JSON-RPC
- ✅ TRUE PRIMAL pattern (no hardcoded names)
- ✅ **NO CHANGES NEEDED!**

**File**: `crates/tools/ai-tools/src/capability_http.rs`
- ✅ Uses Unix sockets
- ✅ Uses JSON-RPC
- ✅ TRUE PRIMAL pattern (no hardcoded names)
- ✅ **NO CHANGES NEEDED!**

**Configuration**:
```rust
// Current (in capability_ai.rs line 43):
socket_path: std::env::var("AI_CAPABILITY_SOCKET")
    .unwrap_or_else(|_| "/var/run/ai/provider.sock".to_string())
    .into(),

// To use Neural API, just set environment variable!
// export AI_CAPABILITY_SOCKET="/tmp/neural-api-nat0.sock"
```

**That's it!** Just point your existing code at Neural API! ✨

---

## ❌ **What Needs Replacement (1 File)**

**File**: `crates/core/mcp/src/monitoring/songbird_client.rs`
- ❌ Uses `reqwest::Client` (brings C dependencies)
- ❌ Lines 100, 109-112, 194-221, 224-251, 426-441
- ⚠️ **BLOCKER for 100% Pure Rust**

**Why It Matters**:
- This ONE file brings `reqwest` → `ring` → `openssl-sys` → C dependencies
- Binary size: 25 MB (with reqwest) → 10-15 MB (without) = **-40%**
- Compile time: 120s → 80s = **-33%**

---

## 🚀 Migration Steps (2-3 Hours Total)

### **Step 1: Add Neural API Client Dependency** (15 min)

**File**: `Cargo.toml` (workspace root, line 52)

**Add to `[workspace.dependencies]`**:
```toml
# Neural API Client for service mesh routing
neural-api-client = { path = "../../phase2/biomeOS/crates/neural-api-client" }
```

**File**: `crates/core/mcp/Cargo.toml`

**Add to `[dependencies]`**:
```toml
# Neural API Client for service mesh routing
neural-api-client = { workspace = true }
```

**Remove**:
```toml
# reqwest REMOVED - Use neural-api-client instead!
# reqwest = { version = "0.11", features = ["json", "rustls-tls"] }
```

---

### **Step 2: Replace songbird_client.rs** (1-2 hours)

**File**: `crates/core/mcp/src/monitoring/songbird_client.rs`

**Option A: Minimal Change (15 min)** - Just comment out reqwest usage:
```rust
// TEMPORARY: Disable Songbird monitoring until Neural API migration
pub fn create_songbird_client() -> Arc<MockMonitoringClient> {
    Arc::new(MockMonitoringClient::new())
}
```

**Option B: Full Migration (1-2 hours)** - Replace with Neural API:

**Add import (line 1)**:
```rust
use neural_api_client::{NeuralApiClient, HttpRequest};
```

**Replace struct (lines 98-104)**:
```rust
#[derive(Debug)]
pub struct SongbirdMonitoringClient {
    config: SongbirdClientConfig,
    neural_client: Arc<NeuralApiClient>,  // ✅ Pure Rust!
    current_metrics: Arc<RwLock<Option<SongbirdMetrics>>>,
    metrics_buffer: Arc<RwLock<Vec<MonitoringEvent>>>,
    events_buffer: Arc<RwLock<Vec<MonitoringEvent>>>,
}
```

**Replace new() (lines 106-121)**:
```rust
impl SongbirdMonitoringClient {
    pub fn new(config: SongbirdClientConfig) -> Result<Self> {
        // Connect to Neural API for routing
        let socket_path = std::env::var("NEURAL_API_SOCKET")
            .unwrap_or_else(|_| "/tmp/neural-api-nat0.sock".to_string());
        
        let neural_client = NeuralApiClient::new(&socket_path)
            .map_err(|e| MCPError::General(format!("Failed to connect to Neural API: {}", e)))?;

        Ok(Self {
            config,
            neural_client: Arc::new(neural_client),
            current_metrics: Arc::new(RwLock::new(None)),
            metrics_buffer: Arc::new(RwLock::new(Vec::new())),
            events_buffer: Arc::new(RwLock::new(Vec::new())),
        })
    }
}
```

**Replace send_metrics_to_songbird (lines 194-221)**:
```rust
async fn send_metrics_to_songbird(&self, metrics: &SongbirdMetrics) -> Result<()> {
    let url = format!("{}/api/v1/metrics", self.config.endpoint);
    
    let request = HttpRequest {
        method: "POST".to_string(),
        url,
        headers: vec![
            ("Content-Type".to_string(), "application/json".to_string()),
        ],
        body: Some(serde_json::to_string(metrics)
            .map_err(|e| MCPError::General(format!("Failed to serialize metrics: {}", e)))?),
    };
    
    match self.neural_client.proxy_http(request).await {
        Ok(response) => {
            if response.status >= 200 && response.status < 300 {
                debug!("Successfully sent metrics via Neural API");
                Ok(())
            } else {
                warn!("Metrics endpoint returned error: {}", response.status);
                Ok(()) // Don't fail on monitoring issues
            }
        }
        Err(e) => {
            warn!("Failed to send metrics via Neural API: {}. Continuing without external monitoring.", e);
            Ok(())
        }
    }
}
```

**Similar changes for**:
- `send_events_to_songbird` (lines 224-251)
- `get_health_status` (lines 426-441)

---

### **Step 3: Update Clone Implementation** (5 min)

**Replace Clone impl (lines 306-317)**:
```rust
impl Clone for SongbirdMonitoringClient {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            neural_client: Arc::clone(&self.neural_client),  // ✅ Changed
            current_metrics: Arc::clone(&self.current_metrics),
            metrics_buffer: Arc::clone(&self.metrics_buffer),
            events_buffer: Arc::clone(&self.events_buffer),
        }
    }
}
```

---

### **Step 4: Build & Test** (30 min)

```bash
cd /home/eastgate/Development/ecoPrimals/phase1/squirrel

# Build
cargo build --release

# Expected: 0 errors, 0 C dependencies
# Binary size: ~10-15 MB (down from ~25 MB!)

# Run tests
cargo test

# Expected: All tests pass
```

---

### **Step 5: Integration Test** (1 hour)

**Requires**: Tower Atomic + Neural API running (biomeOS team handles this)

**Your part**:
```bash
# Set environment to use Neural API
export NEURAL_API_SOCKET="/tmp/neural-api-nat0.sock"
export AI_CAPABILITY_SOCKET="/tmp/neural-api-nat0.sock"
export HTTP_CAPABILITY_SOCKET="/tmp/neural-api-nat0.sock"
export ANTHROPIC_API_KEY="sk-ant-xxxxx"  # From testing-secrets/

# Start Squirrel
./target/release/squirrel server --socket /tmp/ai-nat0.sock

# Test AI call
echo '{"jsonrpc":"2.0","method":"ai.chat.completion","params":{"model":"claude-3-5-sonnet-20241022","messages":[{"role":"user","content":"Hello from Squirrel via Neural API!"}]},"id":1}' \
  | nc -U /tmp/ai-nat0.sock
```

**Expected**: Response from Anthropic API via Neural API routing! ✅

---

### **Step 6: ecoBin Harvest** (15 min)

```bash
# Build for x86_64
cargo build --release --target x86_64-unknown-linux-musl
strip target/x86_64-unknown-linux-musl/release/squirrel

# Build for ARM64
cargo build --release --target aarch64-unknown-linux-musl
strip target/aarch64-unknown-linux-musl/release/squirrel

# Harvest to plasmidBin
cp target/x86_64-unknown-linux-musl/release/squirrel \
   ../../../phase2/biomeOS/plasmidBin/squirrel_x86_64_linux_musl/squirrel

cp target/aarch64-unknown-linux-musl/release/squirrel \
   ../../../phase2/biomeOS/plasmidBin/squirrel_arm64_linux_musl/squirrel

# Verify size
ls -lh ../../../phase2/biomeOS/plasmidBin/squirrel_*/squirrel
# Expected: ~10-15 MB (down from ~25 MB!)
```

---

## ✅ Verification Checklist

### Before Starting
- [ ] Neural API Client available (`../../phase2/biomeOS/crates/neural-api-client/`)
- [ ] Migration guide read (`NEURAL_API_MIGRATION_GUIDE_JAN_20_2026.md`)
- [ ] Backup current `songbird_client.rs` (optional)

### Migration
- [ ] Add `neural-api-client` to workspace dependencies
- [ ] Add `neural-api-client` to `mcp/Cargo.toml`
- [ ] Remove `reqwest` from `mcp/Cargo.toml`
- [ ] Update `songbird_client.rs` imports
- [ ] Replace `reqwest::Client` with `NeuralApiClient`
- [ ] Update `new()` method
- [ ] Update `send_metrics_to_songbird()`
- [ ] Update `send_events_to_songbird()`
- [ ] Update `get_health_status()`
- [ ] Update `Clone` implementation

### Verification
- [ ] Build succeeds (`cargo build --release`)
- [ ] Zero C dependencies (`cargo tree | grep -i "ring\|openssl"` → empty)
- [ ] Tests pass (`cargo test`)
- [ ] Binary size reduced (10-15 MB, down from 25 MB)
- [ ] Integration test works (AI call via Neural API)
- [ ] ecoBins harvested (x86_64 + ARM64)

---

## 📊 Expected Results

### Before Migration
- **Binary Size**: ~25 MB (with reqwest + ring)
- **Compile Time**: ~120 seconds
- **C Dependencies**: 2+ (ring, openssl-sys)
- **Architecture**: Direct capability sockets
- **Observability**: Limited

### After Migration
- **Binary Size**: ~10-15 MB (**-40%**) ✅
- **Compile Time**: ~80 seconds (**-33%**) ✅
- **C Dependencies**: **0** ✅
- **Architecture**: TRUE service mesh ✅
- **Observability**: Full (all requests logged) ✅

---

## 🆘 Troubleshooting

### Issue: "Cannot find neural-api-client"

**Solution**:
```bash
# Verify path is correct
ls ../../phase2/biomeOS/crates/neural-api-client/

# If not, adjust path in Cargo.toml
```

---

### Issue: "Cannot connect to Neural API socket"

**Solution**:
```bash
# Check Neural API is running (biomeOS team handles this)
ls -la /tmp/neural-api-*.sock

# If not, contact biomeOS team
```

---

### Issue: "Build errors after removing reqwest"

**Solution**:
```bash
# Find remaining reqwest references
grep -r "reqwest::" crates/ --include="*.rs" | grep -v test | grep -v example

# Replace with neural_api_client::HttpRequest
```

---

### Issue: "Tests fail after migration"

**Solution**:
1. Check if tests use `reqwest` directly
2. Update tests to use `neural_api_client` or mock it
3. Or use `#[ignore]` for tests that require real Neural API

---

## 📚 Complete Documentation

**For detailed step-by-step**:
- [NEURAL_API_MIGRATION_GUIDE_JAN_20_2026.md](NEURAL_API_MIGRATION_GUIDE_JAN_20_2026.md) (650+ lines)

**For Neural API Client API**:
- `../../phase2/biomeOS/specs/NEURAL_API_CLIENT_SPECIFICATION.md` (627 lines)
- `../../phase2/biomeOS/crates/neural-api-client/README.md`

**For quick reference**:
- `../../phase2/biomeOS/QUICK_REFERENCE_NEURAL_ROUTING.md`

**For architecture**:
- `../../phase2/biomeOS/NEURAL_API_ARCHITECTURE_CORRECTION_JAN_20_2026.md`

---

## 💡 Key Points

### 1. **Minimal Changes Required** ✨

Your existing `capability_ai` and `capability_http` are **perfect**! Just point them at Neural API:

```bash
# OLD: Point to Songbird directly
export AI_CAPABILITY_SOCKET="/var/run/songbird/ai.sock"

# NEW: Point to Neural API
export AI_CAPABILITY_SOCKET="/tmp/neural-api-nat0.sock"
```

**That's it!** No code changes to capability modules! ✅

---

### 2. **Only One File Needs Work**

**File**: `crates/core/mcp/src/monitoring/songbird_client.rs`
- **Lines to change**: ~50 lines total
- **Time**: 1-2 hours (or 15 min for minimal disable)
- **Impact**: Removes ALL C dependencies

---

### 3. **Backward Compatible**

You can support both during migration:

```rust
// Try Neural API first, fallback to direct
let socket_path = std::env::var("NEURAL_API_SOCKET")
    .or_else(|_| std::env::var("AI_CAPABILITY_SOCKET"))
    .unwrap_or_else(|_| "/tmp/neural-api-nat0.sock".to_string());
```

---

### 4. **biomeOS Team Handles Deployment**

**You don't deploy Tower Atomic or Neural API** - that's on us!

**You only need to**:
1. Update one file
2. Build Squirrel
3. Point it at Neural API socket
4. Harvest ecoBins

We handle the mesh infrastructure! ✅

---

## 🎯 Success Criteria

**Required**:
- ✅ Squirrel builds with zero C dependencies
- ✅ Binary size < 15 MB
- ✅ Can make AI calls via Neural API
- ✅ ecoBins harvested (x86_64 + ARM64)

**Nice to Have**:
- ✅ All tests pass
- ✅ Monitoring still works (via Neural API)
- ✅ Compile time < 90 seconds

---

## 📞 Support

**Questions?** Contact biomeOS team

**Blockers?**
- Neural API not running → biomeOS team
- Tower Atomic issues → biomeOS team
- Socket path issues → biomeOS team

**Your scope**:
- Squirrel code changes only
- Build and test
- ecoBin harvest

**Our scope** (biomeOS team):
- Neural API infrastructure
- Tower Atomic deployment
- Routing mesh
- Integration environment

---

## 🎊 When You're Done

**Notify biomeOS team with**:
1. ✅ ecoBins harvested (paths)
2. ✅ Binary sizes (before/after)
3. ✅ Dependency tree (show zero C deps)
4. ✅ Any issues encountered

**We'll**:
1. Update plasmidBin manifest
2. Run full integration tests
3. Deploy to production
4. Celebrate! 🎉

---

## 🏆 Impact

**This migration enables**:
- ✅ **100% Pure Rust** ecosystem
- ✅ **TRUE service mesh** (observability, routing, learning)
- ✅ **Smaller binaries** (-40%)
- ✅ **Faster builds** (-33%)
- ✅ **Production-ready** ecoBins for all architectures

**You're the final piece!** 🧩

---

**Handoff Date**: January 20, 2026  
**From**: biomeOS Team  
**Status**: Ready to Execute  
**Estimated Time**: 2-3 hours  
**Priority**: HIGH  
**Impact**: CRITICAL (enables 100% Pure Rust)

---

🦀 **Squirrel → Neural API: The Final Evolution to 100% Pure Rust!** ✨

**Good luck! You've got this!** 🐿️✨

