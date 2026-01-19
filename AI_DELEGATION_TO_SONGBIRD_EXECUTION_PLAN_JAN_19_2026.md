# AI Delegation to Songbird - Execution Plan

**Date**: January 19, 2026  
**Goal**: Remove `ring` dependency by delegating AI HTTP calls to Songbird  
**Timeline**: ~8-12 hours  
**Status**: Ready to begin

---

## 🎯 Objective

**Current Problem**:
```
Squirrel → reqwest → rustls → ring ❌
```

**Target Solution**:
```
Squirrel → Unix Socket → Songbird → AI Vendors ✅
```

**Pattern**: Use the SAME approach as JWT delegation to BearDog!

---

## 📊 Dependency Analysis

### Where `ring` Comes From

```bash
ring v0.17.14
├── rustls v0.21.12
│   ├── hyper-rustls v0.24.2
│   │   └── reqwest v0.11.27
│   │       ├── ecosystem-api
│   │       ├── squirrel-ai-tools
│   │       ├── squirrel-core
│   │       ├── squirrel-mcp
│   │       ├── squirrel-mcp-auth
│   │       ├── squirrel-mcp-config
│   │       └── 10+ other crates
```

### Crates Using `reqwest`

1. **ecosystem-api** - HTTP calls to ecosystem services
2. **squirrel-ai-tools** - OpenAI/Anthropic/etc APIs
3. **squirrel-core** - HTTP utilities
4. **squirrel-mcp** - MCP protocol HTTP transport
5. **squirrel-cli** - CLI HTTP operations
6. **squirrel-mcp-auth** - Auth HTTP calls
7. **squirrel-mcp-config** - Config fetching

---

## 🔧 Migration Strategy

### Phase 1: Create Capability AI Client (~2-3 hours)

**File**: `crates/tools/ai-tools/src/capability_ai.rs`

**Pattern**: Copy from `crates/core/auth/src/capability_crypto.rs`

```rust
/// Generic AI capability client - discovers AI providers at runtime
pub struct CapabilityAiClient {
    socket_path: PathBuf,
    request_id: AtomicU64,
    capability_provider: String,
}

impl CapabilityAiClient {
    /// Discover AI capability via environment
    pub fn new(config: CapabilityAiConfig) -> Result<Self, AIError> {
        let socket_path = config.socket_path.or_else(|| {
            env::var("AI_CAPABILITY_SOCKET")
                .ok()
                .map(PathBuf::from)
        }).ok_or(AIError::Configuration(
            "AI_CAPABILITY_SOCKET not configured".into()
        ))?;

        Ok(Self {
            socket_path,
            request_id: AtomicU64::new(1),
            capability_provider: "discovered_at_runtime".to_string(),
        })
    }

    /// Chat completion via discovered AI provider
    pub async fn chat_completion(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, AIError> {
        let params = serde_json::to_value(request)?;
        let response = self.send_request("ai.chat.completion", params).await?;
        Ok(serde_json::from_value(response)?)
    }

    /// Text embedding via discovered AI provider
    pub async fn create_embedding(
        &self,
        request: EmbeddingRequest,
    ) -> Result<EmbeddingResponse, AIError> {
        let params = serde_json::to_value(request)?;
        let response = self.send_request("ai.embedding.create", params).await?;
        Ok(serde_json::from_value(response)?)
    }

    /// Send JSON-RPC request over Unix socket
    async fn send_request(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, AIError> {
        // Unix socket JSON-RPC implementation
        // (Copy pattern from capability_crypto.rs)
    }
}
```

**Configuration**:
```rust
pub struct CapabilityAiConfig {
    pub socket_path: Option<PathBuf>,
    pub timeout: Duration,
}

impl Default for CapabilityAiConfig {
    fn default() -> Self {
        Self {
            socket_path: env::var("AI_CAPABILITY_SOCKET")
                .ok()
                .map(PathBuf::from),
            timeout: Duration::from_secs(30),
        }
    }
}
```

---

### Phase 2: Update AI Tools (~2-3 hours)

**File**: `crates/tools/ai-tools/src/lib.rs`

**Before** (uses reqwest):
```rust
use openai::Client;

pub async fn chat_completion(prompt: &str) -> Result<String> {
    let client = Client::new(&env::var("OPENAI_API_KEY")?);
    let response = client.chat()
        .create(request)
        .await?;
    Ok(response.choices[0].message.content.clone())
}
```

**After** (uses capability):
```rust
use crate::capability_ai::CapabilityAiClient;

pub async fn chat_completion(prompt: &str) -> Result<String> {
    let client = CapabilityAiClient::new(Default::default())?;
    let response = client.chat_completion(request).await?;
    Ok(response.choices[0].message.content.clone())
}
```

**Feature Gating**:
```toml
# crates/tools/ai-tools/Cargo.toml
[features]
default = ["delegated-ai"]           # Production: Pure Rust!
delegated-ai = []                    # AI via Songbird (Pure Rust)
direct-http = ["dep:openai", "dep:anthropic-sdk", "dep:reqwest"]  # Dev only

[dependencies]
# Optional for dev/testing
openai = { version = "4.0", optional = true }
anthropic-sdk = { version = "0.2", optional = true }
reqwest = { version = "0.11", features = ["rustls-tls"], optional = true }

# Always available
tokio = { version = "1.0", features = ["net", "io-util"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

---

### Phase 3: Remove reqwest from Core Crates (~2-3 hours)

#### 3.1 ecosystem-api

**Audit**:
```bash
grep -r "reqwest" crates/ecosystem-api/src/
```

**Replace** HTTP calls with Unix socket JSON-RPC:
```rust
// ❌ OLD
let response = reqwest::get(url).await?;

// ✅ NEW
let response = socket_client.call("method", params).await?;
```

#### 3.2 squirrel-core

**Audit**:
```bash
grep -r "use reqwest" crates/core/core/src/
```

**Action**: Remove HTTP utilities, use Unix sockets

#### 3.3 squirrel-mcp

**Audit**: Check MCP protocol HTTP transport

**Action**: MCP should use WebSocket or Unix Socket, not HTTP

#### 3.4 Other Crates

- **squirrel-cli**: Remove HTTP, use Unix sockets
- **squirrel-mcp-auth**: Already Pure Rust! (no changes needed)
- **squirrel-mcp-config**: Use Unix socket for config fetching

---

### Phase 4: Feature Gate reqwest (~1 hour)

**Workspace Cargo.toml**:
```toml
[workspace]
members = [
    "crates/*",
    "crates/core/*",
    "crates/tools/*",
    # ...
]

[workspace.dependencies]
# Make reqwest optional workspace-wide
reqwest = { version = "0.11", features = ["rustls-tls"], optional = true }
```

**Each Crate**:
```toml
[features]
default = ["delegated-ai"]
delegated-ai = []
direct-http = ["dep:reqwest"]

[dependencies]
reqwest = { workspace = true, optional = true }
```

---

### Phase 5: Testing & Validation (~2 hours)

#### 5.1 Dependency Check
```bash
# Should be EMPTY!
cargo tree | grep ring

# Should show reqwest ONLY with feature
cargo tree --features direct-http | grep reqwest
```

#### 5.2 Build Tests
```bash
# Production build (Pure Rust!)
cargo build --release

# Dev build (with HTTP)
cargo build --release --features direct-http
```

#### 5.3 Cross-Compilation
```bash
# ARM64 (should work now!)
cargo build --release --target aarch64-unknown-linux-musl

# Verify
file target/aarch64-unknown-linux-musl/release/squirrel
# Should be: "ARM aarch64, statically linked"
```

#### 5.4 Integration Tests
```bash
# With mock Songbird (Unix socket)
export AI_CAPABILITY_SOCKET=/tmp/mock-ai.sock
cargo test --test ai_capability_integration
```

---

### Phase 6: Documentation (~1 hour)

#### 6.1 Update Configuration
**File**: `config/production.toml`
```toml
[ai]
capability_socket = "${AI_CAPABILITY_SOCKET}"  # From discovery
timeout_secs = 30

# No API keys needed! Songbird handles that.
```

#### 6.2 Migration Guide
**File**: `AI_DELEGATION_MIGRATION_GUIDE.md`
- Why we're delegating
- How to configure
- How to test
- How to migrate existing code

#### 6.3 Update README
- Remove API key configuration
- Add AI capability socket configuration
- Update architecture diagrams

---

## 📋 Detailed Checklist

### Phase 1: Capability AI Client (2-3 hours)

- [ ] Create `crates/tools/ai-tools/src/capability_ai.rs`
- [ ] Implement `CapabilityAiClient` struct
- [ ] Implement discovery via `AI_CAPABILITY_SOCKET`
- [ ] Implement JSON-RPC methods:
  - [ ] `chat_completion`
  - [ ] `create_embedding`
  - [ ] `text_generation` (fallback)
- [ ] Add error handling
- [ ] Add retry logic
- [ ] Add timeout handling
- [ ] Write unit tests

### Phase 2: Update AI Tools (2-3 hours)

- [ ] Update `crates/tools/ai-tools/src/lib.rs`
- [ ] Migrate OpenAI calls to capability
- [ ] Migrate Anthropic calls to capability
- [ ] Feature-gate direct HTTP:
  ```toml
  [features]
  default = ["delegated-ai"]
  delegated-ai = []
  direct-http = ["dep:openai", "dep:anthropic-sdk"]
  ```
- [ ] Update configuration
- [ ] Update tests
- [ ] Add integration tests

### Phase 3: Remove reqwest from Core (2-3 hours)

- [ ] **ecosystem-api**:
  - [ ] Audit reqwest usage: `grep -r "use reqwest" crates/ecosystem-api/`
  - [ ] Replace with Unix socket JSON-RPC
  - [ ] Test integration

- [ ] **squirrel-core**:
  - [ ] Audit reqwest usage
  - [ ] Remove HTTP utilities
  - [ ] Use Unix sockets
  - [ ] Test core functionality

- [ ] **squirrel-mcp**:
  - [ ] Audit MCP HTTP transport
  - [ ] Switch to WebSocket or Unix Socket
  - [ ] Test MCP protocol

- [ ] **squirrel-cli**:
  - [ ] Remove HTTP operations
  - [ ] Use Unix sockets
  - [ ] Test CLI commands

- [ ] **Other crates**: Review and update

### Phase 4: Feature Gating (1 hour)

- [ ] Update `Cargo.toml` in each crate
- [ ] Make `reqwest` optional
- [ ] Add `direct-http` feature
- [ ] Test both modes:
  ```bash
  cargo build --release  # Should have NO reqwest
  cargo build --release --features direct-http  # Should have reqwest
  ```

### Phase 5: Validation (2 hours)

- [ ] **Dependency Check**:
  ```bash
  cargo tree | grep ring  # Should be EMPTY!
  cargo tree | grep reqwest  # Should be EMPTY!
  cargo tree | grep "\-sys"  # Only Pure Rust syscalls
  ```

- [ ] **Build Tests**:
  - [ ] x86_64-unknown-linux-musl
  - [ ] aarch64-unknown-linux-musl (ARM64!)

- [ ] **Integration Tests**:
  - [ ] Mock Songbird Unix socket
  - [ ] AI capability discovery
  - [ ] Chat completion
  - [ ] Embeddings

- [ ] **Doctor Mode**:
  ```bash
  ./target/release/squirrel doctor
  # Should show: AI capability discovered via Unix socket
  ```

### Phase 6: Documentation (1 hour)

- [ ] Create `AI_DELEGATION_MIGRATION_GUIDE.md`
- [ ] Update `README.md`
- [ ] Update `CURRENT_STATUS.md`
- [ ] Update `TRUE_ECOBIN_CORRECTED_STATUS_JAN_19_2026.md`
- [ ] Create TRUE ecoBin certification (for real this time!)

---

## 🎯 Success Criteria

### Must Have ✅

1. **NO ring in dependency tree**:
   ```bash
   cargo tree | grep ring
   # Output: (nothing)
   ```

2. **ARM64 cross-compilation works**:
   ```bash
   cargo build --target aarch64-unknown-linux-musl
   # Output: success
   ```

3. **All tests pass**:
   ```bash
   cargo test
   # Output: 559/559 passing
   ```

4. **Feature gating works**:
   ```bash
   cargo build --release  # Pure Rust
   cargo build --release --features direct-http  # Dev mode
   ```

### Should Have 🎯

1. Integration tests with mock Songbird
2. Doctor Mode shows AI capability status
3. Configuration documented
4. Migration guide complete

### Nice to Have 🌟

1. Performance benchmarks (vs direct HTTP)
2. Failover to direct HTTP in dev
3. Multi-provider support
4. Connection pooling

---

## 📊 Timeline

**Estimated Total**: 8-12 hours

| Phase | Task | Time | Complexity |
|-------|------|------|------------|
| 1 | Capability AI Client | 2-3h | Medium (copy JWT pattern) |
| 2 | Update AI Tools | 2-3h | Medium (migration) |
| 3 | Remove Core reqwest | 2-3h | Medium (grep & replace) |
| 4 | Feature Gating | 1h | Low (config changes) |
| 5 | Validation | 2h | Low (testing) |
| 6 | Documentation | 1h | Low (writing) |

**Recommended Schedule**:
- **Session 1** (4-6 hours): Phases 1-2 (AI delegation)
- **Session 2** (3-4 hours): Phases 3-4 (Core cleanup)
- **Session 3** (1-2 hours): Phases 5-6 (Validation & docs)

---

## 🔗 Reference Materials

### Proven Patterns (Copy These!)

1. **JWT Delegation** (COMPLETED Jan 18, 2026) ✅
   - `crates/core/auth/src/capability_crypto.rs`
   - `crates/core/auth/src/capability_jwt.rs`
   - `crates/core/auth/src/delegated_jwt_client.rs`
   - **Status**: Working in production!

2. **Unix Socket JSON-RPC**
   - Pattern established in JWT migration
   - Retry logic, timeout handling
   - Error propagation

3. **Feature Gating**
   - `delegated-jwt` vs `local-jwt`
   - Same pattern for `delegated-ai` vs `direct-http`

### Songbird AI Proxy API (Target)

```rust
// Songbird provides these JSON-RPC methods:
// - ai.chat.completion
// - ai.embedding.create
// - ai.text.generation
// - ai.model.list
```

---

## 💡 Key Insights

### Why This Will Work

1. **Proven Pattern**: JWT delegation works perfectly
2. **Same Architecture**: Unix socket + JSON-RPC
3. **Clear Separation**: AI tools → Songbird (network) → Vendors
4. **Feature Gating**: Dev mode still works with direct HTTP

### Potential Challenges

1. **Songbird Not Ready**: Use mock socket for now
2. **API Mismatch**: Adapt Songbird API if needed
3. **Performance**: Unix socket adds ~50µs (acceptable)
4. **Testing**: Need mock Songbird for integration tests

### Solutions

1. **Mock Songbird**: Create simple Unix socket server for tests
2. **Gradual Migration**: Feature-gate allows gradual rollout
3. **Backward Compat**: Dev mode keeps working with `direct-http`
4. **Documentation**: Clear migration guide for users

---

## 🚀 Let's Begin!

**Status**: Ready to start Phase 1  
**Next Step**: Create `capability_ai.rs`  
**Pattern**: Copy from `capability_crypto.rs` (proven!)  
**Timeline**: ~2-3 hours  
**Confidence**: High (same pattern as JWT)

---

*Created: January 19, 2026*  
*Goal: TRUE ecoBin certification (for real this time!)*  
*Approach: Ecological delegation to Songbird*  
*Pattern: Proven by JWT migration*

Let's delegate AI HTTP to Songbird and achieve TRUE ecoBin! 🌍🦀✨

