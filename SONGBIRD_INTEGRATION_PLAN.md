# Songbird Integration Plan - Unix Socket Delegation

**Date**: Jan 19, 2026  
**Status**: Ready to implement!  
**Goal**: Turn stubbed AI delegation into working Unix socket communication with Songbird

---

## 🎯 Current Status

### What We Have

**Binaries Available**:
- ✅ `/home/eastgate/Development/ecoPrimals/phase2/biomeOS/plasmidBin/primals/songbird`
- ✅ `/home/eastgate/Development/ecoPrimals/phase2/biomeOS/plasmidBin/primals/beardog`
- ✅ `/home/eastgate/Development/ecoPrimals/phase2/biomeOS/plasmidBin/songbird-orchestrator`

**Songbird Status** (from MANIFEST.md):
- ✅ Songbird v0.1.0 - 95% ecoBin
- ✅ UniBin compliant (3 modes: server, doctor, config)
- ✅ BearDog crypto API integration COMPLETE
- ✅ 594+ tests passing
- ✅ Capability-based CryptoProvider implemented
- ✅ UnixSocketCryptoProvider working
- ✅ Week 1 complete (Jan 18, 2026)

**Our Implementation**:
- ✅ `capability_crypto.rs` - Working BearDog client
  - JSON-RPC over Unix sockets
  - Ed25519 signing/verification
  - Retry logic, timeouts
- ✅ `capability_ai.rs` - Stubbed AI client
  - Pattern established
  - Needs Unix socket implementation

**Songbird Source Files Found**:
```
songbird/crates/songbird-orchestrator/src/ipc/unix_socket.rs
songbird/crates/songbird-orchestrator/src/ipc/server_pure_rust.rs
songbird/crates/songbird-orchestrator/src/crypto/beardog_crypto_client.rs
songbird/crates/songbird-orchestrator/src/crypto/provider.rs
songbird/crates/songbird-orchestrator/src/crypto/discovery.rs
```

---

## 📋 Investigation Steps

### Step 1: Examine Songbird's Unix Socket Implementation
- [x] Find socket paths in Songbird codebase
- [ ] Understand Songbird's JSON-RPC API format
- [ ] Identify what capabilities Songbird exposes
- [ ] Find default socket paths

### Step 2: Understand AI Delegation
- [ ] What API does Songbird expose for AI calls?
- [ ] How does Songbird proxy to OpenAI/Anthropic?
- [ ] What JSON-RPC methods are available?
- [ ] How are AI responses formatted?

### Step 3: Start Songbird Binary
- [ ] Run Songbird from plasmidBin
- [ ] Find where Unix socket is created
- [ ] Test basic connectivity

---

## 🚀 Implementation Plan

### Phase 1: Basic Songbird Client (1-2 hours)

**Create** `crates/tools/ai-tools/src/capability_songbird.rs`:
```rust
//! Songbird Unix Socket Client
//!
//! Delegates network operations to Songbird primal via Unix sockets.
//! Pattern: Same as capability_crypto.rs but for AI/network calls.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::net::UnixStream;

#[derive(Debug, Clone)]
pub struct SongbirdClientConfig {
    pub socket_path: PathBuf,
    pub timeout_secs: u64,
    pub max_retries: usize,
}

impl Default for SongbirdClientConfig {
    fn default() -> Self {
        Self {
            socket_path: std::env::var("SONGBIRD_SOCKET")
                .unwrap_or_else(|_| "/var/run/songbird/provider.sock".to_string())
                .into(),
            timeout_secs: 30,  // AI calls can be slow
            max_retries: 3,
        }
    }
}

pub struct SongbirdClient {
    config: SongbirdClientConfig,
    request_id: std::sync::atomic::AtomicU64,
}

impl SongbirdClient {
    pub fn new(config: SongbirdClientConfig) -> Result<Self> {
        // Same pattern as CryptoClient
    }
    
    pub async fn ai_complete(
        &self,
        provider: &str,  // "openai", "anthropic", etc.
        request: AiCompletionRequest,
    ) -> Result<AiCompletionResponse> {
        // JSON-RPC call to Songbird
        // Songbird makes actual HTTP call to AI provider
        // Returns response via Unix socket
    }
}
```

### Phase 2: Integration (30 min)

**Update** `crates/tools/ai-tools/src/capability_ai.rs`:
- Replace `unimplemented!()` stubs
- Use `SongbirdClient` for actual calls
- Keep same external API

### Phase 3: Testing (1 hour)

1. **Unit Tests**: Mock Unix socket communication
2. **Integration Tests**: Real Songbird binary
3. **End-to-End**: Actual OpenAI/Anthropic calls via Songbird

---

## 🔍 Discovery Tasks

### Find Socket Paths

Check Songbird codebase:
```bash
cd /home/eastgate/Development/ecoPrimals/phase1/songbird
grep -r "socket_path\|\.sock" crates/songbird-orchestrator/src/ipc/
grep -r "default.*socket" crates/songbird-config/
```

### Find AI API Methods

Check what JSON-RPC methods Songbird exposes:
```bash
grep -r "ai\|openai\|anthropic" crates/songbird-orchestrator/src/
grep -r "rpc.*method" crates/songbird-orchestrator/src/
```

### Start Songbird for Testing

```bash
# Run Songbird from plasmidBin
cd /home/eastgate/Development/ecoPrimals/phase2/biomeOS/plasmidBin
./primals/songbird server --help  # Check usage
./primals/songbird server  # Start server

# Or use the development version
cd /home/eastgate/Development/ecoPrimals/phase1/songbird
cargo run --bin songbird-orchestrator -- --help
```

---

## 📝 Implementation Pattern

### Proven Pattern (from capability_crypto.rs)

```rust
// 1. Configuration with socket path discovery
struct ClientConfig {
    socket_path: PathBuf,  // From env or discovery
    timeout_secs: u64,
    max_retries: usize,
}

// 2. JSON-RPC request/response
#[derive(Serialize)]
struct JsonRpcRequest {
    jsonrpc: String,  // "2.0"
    method: String,
    params: JsonValue,
    id: u64,
}

// 3. Unix socket communication
async fn send_request(&self, request: JsonRpcRequest) -> Result<JsonRpcResponse> {
    let stream = UnixStream::connect(&self.config.socket_path).await?;
    // Write request, read response
}

// 4. Retry logic with exponential backoff
// 5. Timeout handling
// 6. Error mapping
```

**This pattern is PROVEN** - it works for BearDog crypto!

---

## 🎯 Success Criteria

### Phase 1: Basic Client (DONE when)
- [x] SongbirdClient struct created
- [ ] Unix socket connection working
- [ ] JSON-RPC request/response working
- [ ] Basic error handling

### Phase 2: Integration (DONE when)
- [ ] `capability_ai.rs` uses SongbirdClient
- [ ] No more `unimplemented!()`
- [ ] Tests pass

### Phase 3: End-to-End (DONE when)
- [ ] Can call OpenAI via Songbird via Unix socket
- [ ] Can call Anthropic via Songbird via Unix socket
- [ ] Response format matches original
- [ ] Performance acceptable (< 1s overhead)

---

## 🌍 The Ecological Way

**Before** (What we deleted):
```
Squirrel → HTTP → OpenAI API
         (reqwest, rustls, ring)
```

**After** (What we're implementing):
```
Squirrel → Unix Socket → Songbird → HTTP → OpenAI API
           (Pure Rust)   (handles HTTP/TLS)
```

**Benefits**:
- ✅ Squirrel: 100% Pure Rust (zero C dependencies!)
- ✅ Separation of concerns (network primal handles network)
- ✅ Security (Songbird can enforce policies, rate limits, etc.)
- ✅ Testability (mock Unix socket, don't need real AI APIs)
- ✅ TRUE PRIMAL (knows nothing, discovers everything)

---

## 📚 References

- **Working Pattern**: `crates/core/auth/src/capability_crypto.rs`
- **Stub to Implement**: `crates/tools/ai-tools/src/capability_ai.rs`
- **Songbird Source**: `/home/eastgate/Development/ecoPrimals/phase1/songbird`
- **Binaries**: `/home/eastgate/Development/ecoPrimals/phase2/biomeOS/plasmidBin/primals/`

---

## ⏱️ Estimated Timeline

- **Phase 1** (Basic Client): 1-2 hours
- **Phase 2** (Integration): 30 minutes
- **Phase 3** (Testing): 1 hour
- **Total**: 2.5-3.5 hours

---

**Status**: Ready to begin implementation!  
**Next Step**: Examine Songbird's Unix socket code to understand the API

The ecological way - delegate to specialists! 🌍🦀✨

