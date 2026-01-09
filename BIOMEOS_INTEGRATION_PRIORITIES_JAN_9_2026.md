# 🎯 biomeOS Integration Priorities - January 9, 2026
**Date**: January 9, 2026  
**Status**: 🚨 **ACTION REQUIRED** - Build Broken + Protocol Evolution Needed  
**Priority**: 🔴 **CRITICAL** - Blocking biomeOS Integration

---

## 🚨 CRITICAL: Multiple Build Errors Found

### Audit Found (Compilation Errors)
1. ❌ `ecosystem-api/src/client.rs` - Missing 5 imports
2. ❌ `universal-patterns/src/security/hardening.rs` - Missing panic imports
3. ❌ `config/src/constants.rs` - 4 deprecation errors
4. ❌ `ai-tools` tests - 39 API mismatch errors

### biomeOS Team Found (Additional Errors)
5. ❌ `crates/core/workflow/src/engine.rs` - **ActionResult type missing**

```
error[E0412]: cannot find type `ActionResult` in this scope
  --> crates/core/workflow/src/engine.rs:36:13
   |
36 |     result: ActionResult,
   |             ^^^^^^^^^^^^ not found in this scope
```

**Reality Check**: Build is MORE broken than initially assessed.

---

## 🎯 Updated Mission: Match Songbird & BearDog

### What biomeOS Needs (From Handoff Document)

Squirrel must evolve to match the proven patterns from:
- ✅ **Songbird** (v3.6 - Production, UDP + JSON-RPC + tarpc)
- ✅ **BearDog** (v0.15.0 - Production, HTTP + JSON-RPC + tarpc)

### Protocol Evolution Required

```
Current State:
  Squirrel → HTTP REST only
  
Target State (Like Songbird/BearDog):
  Squirrel → HTTP REST + JSON-RPC + tarpc + Unix Sockets
```

---

## 🔴 PHASE 0: Fix ALL Build Errors (BLOCKING)

**Time**: 4-6 hours  
**Priority**: 🔴 **IMMEDIATE**  
**Blocks**: Everything else

### Error Set 1: ecosystem-api imports (5 min)
```rust
// File: crates/ecosystem-api/src/client.rs
// Add at top:
use crate::{
    EcosystemServiceRegistration,
    PrimalType,
    ServiceCapabilities,
    ServiceEndpoints,
    ResourceSpec,
};
```

### Error Set 2: panic imports (2 min)
```rust
// File: crates/universal-patterns/src/security/hardening.rs
use std::panic::{self, PanicHookInfo};
```

### Error Set 3: Deprecation allowances (2 min)
```rust
// File: crates/config/src/constants.rs
#[cfg(test)]
#[allow(deprecated)]
mod tests { ... }
```

### Error Set 4: ActionResult Missing (30 min - INVESTIGATE)
```rust
// File: crates/core/workflow/src/engine.rs:36
// Issue: ActionResult type not found

// Likely fixes:
// 1. Check if ActionResult should be imported from another module
// 2. Check if it was renamed/removed in a refactor
// 3. Check if it needs to be defined in this module
// 4. Search codebase: rg "ActionResult" --type rust
```

**Action**: Find where `ActionResult` should come from or if it needs to be defined.

### Error Set 5: Test API mismatches (2-3 hours)
```rust
// File: crates/tools/ai-tools/tests/router_dispatch_comprehensive_tests.rs
// Update 39 test errors to match current API
```

**Verification**:
```bash
cargo build --release
cargo test --workspace
cargo clippy --all-targets --all-features
```

---

## 🔴 PHASE 1: JSON-RPC + tarpc Evolution (CRITICAL)

**Time**: 8-12 hours  
**Priority**: 🔴 **HIGH**  
**Enables**: biomeOS NUCLEUS integration

### Why This Matters

From biomeOS handoff:
> "we need to evolve to be json-rpc and tarpc first like ../songbird/ and ../beardog/"

**Current Reality**:
- Songbird: ✅ Production with JSON-RPC + tarpc
- BearDog: ✅ Production with JSON-RPC + tarpc
- Squirrel: ❌ HTTP REST only

**biomeOS Expectation**: Squirrel matches the proven pattern.

### Implementation Plan

#### 1. Study Reference Implementations (1 hour)
```bash
# Examine Songbird's JSON-RPC implementation
cd ../songbird
rg "json-rpc|tarpc" --type rust -A 5

# Examine BearDog's JSON-RPC implementation
cd ../beardog
rg "json-rpc|tarpc" --type rust -A 5

# Document patterns
```

#### 2. Add Dependencies (15 min)
```toml
# Cargo.toml
[dependencies]
jsonrpc-core = "18.0"
jsonrpc-derive = "18.0"
jsonrpc-ipc-server = "18.0"
tarpc = { version = "0.34", features = ["full"] }
tokio-serde = { version = "0.9", features = ["json"] }
```

#### 3. Implement JSON-RPC Server (4-6 hours)

**File**: `crates/main/src/rpc/mod.rs` (NEW)
```rust
use jsonrpc_core::{IoHandler, Result as RpcResult};
use jsonrpc_derive::rpc;

#[rpc]
pub trait SquirrelRpc {
    /// List available AI providers
    #[rpc(name = "list_providers")]
    fn list_providers(&self) -> RpcResult<Vec<String>>;
    
    /// Query AI with prompt
    #[rpc(name = "query_ai")]
    fn query_ai(&self, prompt: String, provider: Option<String>) -> RpcResult<String>;
    
    /// Get capabilities
    #[rpc(name = "get_capabilities")]
    fn get_capabilities(&self) -> RpcResult<serde_json::Value>;
    
    /// Health check
    #[rpc(name = "health_check")]
    fn health_check(&self) -> RpcResult<String>;
}

pub struct SquirrelRpcImpl {
    // Internal state
}

impl SquirrelRpc for SquirrelRpcImpl {
    fn list_providers(&self) -> RpcResult<Vec<String>> {
        // Implementation
        Ok(vec!["openai".into(), "claude".into(), "ollama".into()])
    }
    
    fn query_ai(&self, prompt: String, provider: Option<String>) -> RpcResult<String> {
        // Implementation
        Ok("AI response".into())
    }
    
    fn get_capabilities(&self) -> RpcResult<serde_json::Value> {
        // Return capability JSON (reuse existing capability code)
        Ok(serde_json::json!({
            "ai": {
                "mcp-server": true,
                "multi-provider": true,
                "local-inference": true
            }
        }))
    }
    
    fn health_check(&self) -> RpcResult<String> {
        Ok("healthy".into())
    }
}
```

#### 4. Add Unix Socket Support (2-3 hours)

**File**: `crates/main/src/rpc/socket.rs` (NEW)
```rust
use jsonrpc_ipc_server::ServerBuilder;
use std::path::PathBuf;

pub async fn start_unix_socket_server(
    socket_path: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut io = IoHandler::new();
    let rpc = SquirrelRpcImpl::new();
    io.extend_with(rpc.to_delegate());
    
    // Start server on Unix socket
    let server = ServerBuilder::new(io)
        .start(&socket_path)?;
        
    println!("JSON-RPC server listening on {:?}", socket_path);
    server.wait();
    
    Ok(())
}
```

#### 5. Add tarpc Support (2-3 hours)

**File**: `crates/main/src/rpc/tarpc_server.rs` (NEW)
```rust
use tarpc::{context, server};

#[tarpc::service]
pub trait SquirrelService {
    async fn list_providers() -> Vec<String>;
    async fn query_ai(prompt: String, provider: Option<String>) -> String;
    async fn get_capabilities() -> serde_json::Value;
    async fn health_check() -> String;
}

#[derive(Clone)]
pub struct SquirrelServer;

impl SquirrelService for SquirrelServer {
    async fn list_providers(self, _: context::Context) -> Vec<String> {
        vec!["openai".into(), "claude".into(), "ollama".into()]
    }
    
    async fn query_ai(
        self,
        _: context::Context,
        prompt: String,
        provider: Option<String>,
    ) -> String {
        // Implementation
        "AI response".into()
    }
    
    async fn get_capabilities(self, _: context::Context) -> serde_json::Value {
        serde_json::json!({
            "ai": {
                "mcp-server": true,
                "multi-provider": true
            }
        })
    }
    
    async fn health_check(self, _: context::Context) -> String {
        "healthy".into()
    }
}
```

#### 6. Update Main to Start All Servers (1 hour)

**File**: `crates/main/src/main.rs`
```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Existing HTTP REST server
    let http_handle = tokio::spawn(async {
        start_http_server().await
    });
    
    // NEW: JSON-RPC Unix socket server
    let socket_path = PathBuf::from("/tmp/squirrel.sock");
    let rpc_handle = tokio::spawn(async move {
        start_unix_socket_server(socket_path).await
    });
    
    // NEW: tarpc server
    let tarpc_handle = tokio::spawn(async {
        start_tarpc_server().await
    });
    
    // Wait for all servers
    tokio::try_join!(http_handle, rpc_handle, tarpc_handle)?;
    
    Ok(())
}
```

### Testing JSON-RPC

```bash
# Test Unix socket JSON-RPC
echo '{"jsonrpc":"2.0","method":"list_providers","params":{},"id":1}' | \
  nc -U /tmp/squirrel.sock

# Expected response:
# {"jsonrpc":"2.0","result":["openai","claude","ollama"],"id":1}

# Test capabilities
echo '{"jsonrpc":"2.0","method":"get_capabilities","params":{},"id":2}' | \
  nc -U /tmp/squirrel.sock

# Test health check
echo '{"jsonrpc":"2.0","method":"health_check","params":{},"id":3}' | \
  nc -U /tmp/squirrel.sock
```

---

## 🟡 PHASE 2: NUCLEUS Protocol Compatibility (HIGH)

**Time**: 4-6 hours  
**Priority**: 🟡 **HIGH**  
**Enables**: biomeOS 5-layer verification

### biomeOS NUCLEUS Discovery (5 Layers)

From biomeOS handoff:
1. **Physical Discovery**: UDP multicast / Unix socket scanning
2. **Identity Verification**: BearDog cryptographic verification
3. **Capability Verification**: Query `--capability` endpoint
4. **Trust Evaluation**: Genetic lineage verification
5. **Registration**: Add to biomeOS primal registry

### Required Implementations

#### 1. UDP Multicast Announcement (2 hours)
```rust
// File: crates/main/src/discovery/multicast.rs (NEW)

use tokio::net::UdpSocket;
use std::net::Ipv4Addr;

const MULTICAST_ADDR: &str = "239.255.0.1:4200"; // Same as Songbird

pub async fn start_multicast_announcer() -> Result<(), Box<dyn std::error::Error>> {
    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    socket.join_multicast_v4(
        Ipv4Addr::new(239, 255, 0, 1),
        Ipv4Addr::UNSPECIFIED,
    )?;
    
    let announcement = serde_json::json!({
        "primal": "squirrel",
        "version": env!("CARGO_PKG_VERSION"),
        "capabilities": ["ai.mcp-server", "ai.multi-provider"],
        "endpoints": {
            "http": "http://localhost:9010",
            "rpc": "/tmp/squirrel.sock"
        }
    });
    
    loop {
        socket.send_to(
            announcement.to_string().as_bytes(),
            MULTICAST_ADDR,
        ).await?;
        
        tokio::time::sleep(Duration::from_secs(30)).await;
    }
}
```

#### 2. Identity Verification Endpoint (1 hour)
```rust
// File: crates/main/src/api/identity.rs (NEW)

#[derive(Serialize)]
pub struct IdentityProof {
    pub primal_id: String,
    pub instance_id: String,
    pub family_id: Option<String>, // For BearDog lineage
    pub signature: Option<String>, // Cryptographic proof
    pub capabilities: Vec<String>,
}

pub async fn get_identity_proof() -> Json<IdentityProof> {
    // Integrate with BearDog if available
    let family_id = discover_beardog_family().await;
    
    Json(IdentityProof {
        primal_id: "squirrel".into(),
        instance_id: get_instance_id(),
        family_id,
        signature: None, // TODO: Sign with BearDog
        capabilities: get_capabilities_list(),
    })
}
```

#### 3. Enhanced Capability Endpoint (30 min)
```rust
// File: crates/main/src/api/capabilities.rs (ENHANCE)

#[derive(Serialize)]
pub struct EnhancedCapabilities {
    pub primal: PrimalInfo,
    pub capabilities: HashMap<String, CapabilityDetail>,
    pub dependencies: Vec<DependencyInfo>,
    pub health: HealthStatus,
}

pub async fn get_enhanced_capabilities() -> Json<EnhancedCapabilities> {
    Json(EnhancedCapabilities {
        primal: PrimalInfo {
            id: "squirrel".into(),
            name: "Squirrel AI".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            primal_type: "ai-coordinator".into(),
        },
        capabilities: get_detailed_capabilities(),
        dependencies: vec![
            DependencyInfo {
                primal_id: "biomeos".into(),
                capability: "discovery.primal-registry".into(),
                required: true,
            },
            DependencyInfo {
                primal_id: "beardog".into(),
                capability: "security.encryption".into(),
                required: false,
            },
            DependencyInfo {
                primal_id: "songbird".into(),
                capability: "comms.p2p-discovery".into(),
                required: false,
            },
        ],
        health: check_health().await,
    })
}
```

#### 4. Trust Evaluation Support (30 min)
```rust
// File: crates/main/src/security/trust.rs (NEW)

#[derive(Serialize)]
pub struct TrustInfo {
    pub family_id: Option<String>,
    pub genetic_lineage: Option<String>,
    pub trust_level: TrustLevel,
    pub verification_timestamp: DateTime<Utc>,
}

pub async fn get_trust_info() -> TrustInfo {
    // Query BearDog for genetic lineage
    let family = discover_beardog_family().await;
    
    TrustInfo {
        family_id: family.clone(),
        genetic_lineage: family,
        trust_level: TrustLevel::Verified,
        verification_timestamp: Utc::now(),
    }
}
```

---

## 🟢 PHASE 3: Testing & Validation (MEDIUM)

**Time**: 4-6 hours  
**Priority**: 🟢 **MEDIUM**  
**Validates**: Everything works

### Integration Tests

```bash
# 1. Start Squirrel
./target/release/squirrel &

# 2. Verify HTTP REST
curl http://localhost:9010/api/v1/health

# 3. Verify JSON-RPC
echo '{"jsonrpc":"2.0","method":"health_check","params":{},"id":1}' | \
  nc -U /tmp/squirrel.sock

# 4. Verify UDP multicast
tcpdump -i any -n udp port 4200

# 5. Verify capabilities
curl http://localhost:9010/api/v1/capabilities | jq .

# 6. Verify identity
curl http://localhost:9010/api/v1/identity | jq .
```

### biomeOS Discovery Test

```bash
# Start biomeOS
cd ../biomeOS
cargo run --package biomeos-api &

# Wait for discovery
sleep 5

# Check if Squirrel appears in topology
curl http://localhost:3000/api/v1/topology | \
  jq '.primals[] | select(.name == "Squirrel")'

# Should see:
# {
#   "id": "squirrel",
#   "name": "Squirrel",
#   "status": "healthy",
#   "capabilities": [...],
#   "endpoints": {...}
# }
```

---

## 📊 Priority Matrix

| Task | Priority | Time | Blocks | ROI |
|------|----------|------|--------|-----|
| Fix Build Errors | 🔴 CRITICAL | 4-6h | Everything | ♾️ |
| JSON-RPC + tarpc | 🔴 HIGH | 8-12h | biomeOS integration | High |
| NUCLEUS Protocol | 🟡 HIGH | 4-6h | Discovery | High |
| Testing | 🟢 MEDIUM | 4-6h | Deployment | Medium |
| **TOTAL** | - | **20-30h** | - | - |

---

## 🎯 Success Criteria

### Phase 0: Build Success ✅
- [ ] `cargo build --release` completes with NO errors
- [ ] `cargo test --workspace` passes
- [ ] `cargo clippy` passes
- [ ] Binary runs: `./target/release/squirrel --version`

### Phase 1: Protocol Evolution ✅
- [ ] JSON-RPC server responds on Unix socket
- [ ] tarpc server responds
- [ ] HTTP REST still works (backward compat)
- [ ] All protocols tested

### Phase 2: NUCLEUS Compatibility ✅
- [ ] UDP multicast announcing
- [ ] Identity endpoint returns valid proof
- [ ] Capability endpoint enhanced
- [ ] Trust evaluation working

### Phase 3: Integration Success ✅
- [ ] biomeOS discovers Squirrel
- [ ] NUCLEUS 5-layer verification passes
- [ ] Topology API includes Squirrel
- [ ] AI queries work via all protocols

---

## 📋 Checklist for biomeOS Team

When complete, verify:

### Discovery
- [ ] Squirrel appears in `GET /api/v1/topology`
- [ ] UDP multicast packets received
- [ ] Unix socket discoverable

### Identity
- [ ] `GET /api/v1/identity` returns valid proof
- [ ] Family ID matches if BearDog available
- [ ] Signature verification passes

### Capabilities
- [ ] `GET /api/v1/capabilities` returns enhanced format
- [ ] Dependencies listed correctly
- [ ] Health status accurate

### Functionality
- [ ] JSON-RPC methods work: `list_providers`, `query_ai`, `get_capabilities`, `health_check`
- [ ] HTTP REST endpoints still work
- [ ] Can query AI via biomeOS

---

## 🚀 Timeline

### This Week (Jan 9-15)
- [ ] Fix ALL build errors (Day 1-2)
- [ ] Implement JSON-RPC server (Day 2-3)
- [ ] Add Unix socket support (Day 3-4)
- [ ] Test basic RPC (Day 4)

### Next Week (Jan 16-22)
- [ ] Add tarpc support (Day 1-2)
- [ ] Implement UDP multicast (Day 2-3)
- [ ] Add NUCLEUS endpoints (Day 3-4)
- [ ] Integration testing (Day 4-5)

### Week 3 (Jan 23-29)
- [ ] Polish and documentation
- [ ] biomeOS integration validation
- [ ] Production readiness checks
- [ ] Showcase creation

---

## 💬 Communication with biomeOS

### What to Report

**After Phase 0** (Build Fixed):
```
✅ Build errors fixed
✅ Binary ready: target/release/squirrel
✅ Basic functionality verified
⏳ JSON-RPC evolution in progress
```

**After Phase 1** (JSON-RPC Ready):
```
✅ JSON-RPC server implemented
✅ Unix socket: /tmp/squirrel.sock
✅ tarpc support added
⏳ NUCLEUS integration in progress
```

**After Phase 2** (NUCLEUS Ready):
```
✅ UDP multicast announcing
✅ Identity verification endpoint
✅ Enhanced capabilities endpoint
✅ Ready for biomeOS integration testing
```

---

## 📚 Reference Implementations

### Study These First

1. **Songbird v3.6** (Production JSON-RPC + tarpc):
   - `../songbird/crates/songbird-core/src/rpc/`
   - UDP multicast pattern
   - Unix socket setup

2. **BearDog v0.15.0** (Production JSON-RPC):
   - `../beardog/crates/beardog-server/src/rpc/`
   - Identity verification
   - Cryptographic signing

3. **biomeOS NUCLEUS Protocol**:
   - `../biomeOS/specs/NUCLEUS_SECURE_DISCOVERY_PROTOCOL.md`
   - 5-layer verification process
   - Integration patterns

---

## 🎯 Bottom Line

**Current Reality**: Squirrel is architecturally excellent but:
1. ❌ Build is broken (more errors than initially found)
2. ❌ Uses HTTP REST only (needs JSON-RPC + tarpc)
3. ❌ No Unix socket support
4. ❌ No NUCLEUS protocol compatibility

**Target State**: Match Songbird & BearDog pattern:
1. ✅ Build succeeds
2. ✅ HTTP REST + JSON-RPC + tarpc + Unix sockets
3. ✅ UDP multicast discovery
4. ✅ NUCLEUS 5-layer verification

**Investment**: 20-30 hours over 2-3 weeks

**Return**: Full biomeOS integration + ecosystem intelligence

---

**Start with Phase 0: Fix ALL build errors**  
**Including the ActionResult issue biomeOS found**

🐿️ **Let's make Squirrel match the proven pattern!** 🦀

