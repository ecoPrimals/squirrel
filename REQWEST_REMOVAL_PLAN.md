# Complete reqwest Removal Plan - TRUE PRIMAL Way

**Date**: January 19, 2026  
**Philosophy**: NO feature-gating. Complete removal and delegation to Songbird.  
**Status**: 🚧 In Progress

---

## 🎯 The TRUE PRIMAL Pattern

```
WRONG ❌: Feature-gate reqwest for optional usage
RIGHT ✅: Remove reqwest, delegate HTTP to Songbird via Unix sockets
```

**Pattern We Already Proved:**
- **Before**: squirrel-ai-tools → reqwest → OpenAI/Anthropic
- **After**: squirrel-ai-tools → Unix Socket → Songbird → HTTP APIs

**Apply Everywhere**: ANY HTTP → Songbird delegation

---

## 📊 Scope: 82 Files with reqwest References

### Category Breakdown

1. **Cargo.toml files** (~15 files)
   - Status: ✅ Already made optional
   - Next: Remove entirely from production code

2. **Test files** (~10 files)
   - Examples: `*_tests.rs`, `test_helpers.rs`
   - Action: Keep for testing, but mark clearly as test-only

3. **Example files** (2 files)
   - `dynamic_model_switching.rs`, `multi_model_discussion.rs`
   - Action: Keep but add comments about production delegation

4. **Old Provider Implementations** (~20 files)
   - `openai/mod.rs`, `anthropic/mod.rs`, `ollama.rs`, etc.
   - Status: Being replaced by `capability_ai.rs`
   - Action: Remove or mark deprecated

5. **Core Service HTTP** (~10 files)
   - `ecosystem/discovery_client.rs`, `monitoring.rs`, `federation.rs`
   - Status: 🚧 Partially cleaned
   - Action: Remove HTTP, use Unix socket discovery

6. **Auth HTTP** (~5 files)
   - `auth/providers.rs`, `auth/auth.rs`
   - Action: Delegate to BearDog (JWT already does this!)

7. **Observability/Monitoring** (~10 files)
   - `observability/*`, `monitoring/*`
   - Action: Metrics via Unix sockets or file-based

8. **Ecosystem API Client** (2 files)
   - `ecosystem-api/src/client.rs`
   - Action: Unix socket based client

9. **Plugin Marketplace** (2 files)
   - `plugins/marketplace.rs`
   - Action: Delegate to Songbird or local file-based

10. **Misc/Config/Scripts** (~6 files)
    - JSON configs, update scripts
    - Action: Documentation updates only

---

## 🔧 Systematic Removal Plan

### Phase 1: Core Services (HIGH PRIORITY)

#### 1.1 squirrel-core (STARTED ✅)
- [x] `lib.rs`: Remove `Http(#[from] reqwest::Error)` ✅
- [x] `routing/mod.rs`: Remove unused `http_client` field ✅
- [x] `ecosystem.rs`: Remove unused `http_client` field ✅
- [ ] `federation.rs`: Replace HTTP calls with Unix socket delegation
- [ ] `monitoring.rs`: Remove `reqwest::Client` field (line 481)

**Approach for federation.rs**:
```rust
// OLD (line 215):
let response = self.http_client.get(&info_url).send().await?;

// NEW:
let songbird_client = CapabilityHttpClient::discover("http.client").await?;
let response = songbird_client.get(&info_url).await?;
```

#### 1.2 Auth Services
- [ ] `auth/providers.rs`: Remove HTTP, use BearDog capability client
- [ ] `auth/auth.rs`: Same pattern as JWT migration
- [ ] `auth/errors.rs`: Remove reqwest error conversions

**Pattern**: We already did this for JWT! Replicate for other auth methods.

#### 1.3 Ecosystem Discovery
- [ ] `ecosystem/discovery_client.rs`: Unix socket based discovery
- [ ] `ecosystem/registry_manager.rs`: File or socket based registry
- [ ] `capability_discovery.rs`: Already socket-based, verify no HTTP

---

### Phase 2: AI Tools Cleanup (MEDIUM PRIORITY)

#### 2.1 Remove Old Provider Implementations
These are superseded by `capability_ai.rs`:
- [ ] `openai/mod.rs` → Mark deprecated or remove
- [ ] `anthropic/mod.rs` → Mark deprecated or remove
- [ ] `gemini/mod.rs` → Mark deprecated or remove
- [ ] `google.rs` → Remove
- [ ] `local/ollama.rs` → Keep for local use or mark dev-only
- [ ] `providers/*` → Review each

**Decision Point**: 
- Keep as `#[cfg(test)]` only? 
- Remove entirely and use capability_ai everywhere?
- **Recommendation**: Remove from lib, keep in examples/tests

#### 2.2 Error Handling
- [ ] `ai-tools/src/error.rs`: Remove reqwest error conversions (line 121)
- [ ] Already have String-based errors, just remove From impl

---

### Phase 3: Observability & Monitoring (MEDIUM PRIORITY)

#### 3.1 Metrics/Tracing
- [ ] `observability/metrics.rs`: File-based or Unix socket metrics
- [ ] `observability/tracing_utils.rs`: Similar approach
- [ ] `mcp/observability/dashboard.rs`: Static file or socket-based

**Approach**: Metrics don't need HTTP push - use:
1. File-based exports (Prometheus text format)
2. Unix socket for scraping
3. Or delegate to monitoring primal

#### 3.2 Health Checks
- [ ] `ecosystem/registry/health.rs`: Socket-based health checks
- [ ] Internal health = no HTTP needed
- [ ] External health = delegate to Songbird

---

### Phase 4: Plugins & Extensions (LOW PRIORITY)

#### 4.1 Marketplace
- [ ] `plugins/marketplace.rs`: Local manifest files
- [ ] Or delegate downloads to Songbird
- [ ] Plugin discovery via filesystem or Unix socket

#### 4.2 Connection Pools
- [ ] `universal_primal_ecosystem/connection_pool.rs`: Review usage
- [ ] If HTTP pools, replace with Unix socket pools

---

### Phase 5: Tests & Examples (KEEP)

#### 5.1 Test Files
Keep reqwest in tests but mark clearly:
```rust
#[cfg(test)]
use reqwest; // Test-only HTTP client

// OR add to Cargo.toml:
[dev-dependencies]
reqwest = "0.11"
```

#### 5.2 Examples
Update examples to show capability-based usage:
```rust
// examples/multi_model_discussion.rs
// Update to use CapabilityAIProvider instead of direct HTTP
```

---

## 🎯 Implementation Strategy

### Step-by-Step Approach

1. **Start with Core** (squirrel-core)
   - Remove unused fields ✅ (routing, ecosystem done)
   - Fix federation.rs HTTP calls
   - Fix monitoring.rs HTTP client

2. **Auth Next** (leverage JWT pattern)
   - Replicate JWT → BearDog pattern
   - Already have capability_crypto, extend it

3. **AI Tools** (already mostly done!)
   - capability_ai.rs is the future ✅
   - Mark old providers as deprecated
   - Update lib.rs to prefer capability

4. **Ecosystem/Discovery**
   - Unix socket based service discovery
   - File-based fallback
   - No HTTP registry calls needed

5. **Observability**
   - File-based metrics export
   - Unix socket based if needed
   - Internal only, no external HTTP

6. **Cleanup**
   - Remove reqwest from [dependencies]
   - Move to [dev-dependencies] for tests only
   - Validate cargo tree shows zero ring

---

## 💡 Delegation Patterns

### Pattern 1: HTTP Client Delegation (AI Tools ✅)
```rust
// OLD:
let client = reqwest::Client::new();
let response = client.post(url).json(&body).send().await?;

// NEW:
let client = CapabilityAiClient::discover("ai.chat.completion").await?;
let response = client.chat_completion(model, messages, options).await?;
```

### Pattern 2: Service Discovery (BearDog JWT ✅)
```rust
// OLD:
let response = reqwest::get("http://auth-service/token").await?;

// NEW:
let beardog = CapabilityCrypto::discover("crypto.jwt.sign").await?;
let token = beardog.sign_jwt(claims).await?;
```

### Pattern 3: Internal Communication
```rust
// OLD:
let response = http_client.get(&format!("{}/health", instance.endpoint)).await?;

// NEW:
let socket_path = format!("/var/run/primals/{}/health.sock", instance.id);
let client = UnixStream::connect(socket_path).await?;
// ... JSON-RPC or simple protocol ...
```

### Pattern 4: External HTTP (Delegate to Songbird)
```rust
// For any external HTTP needs:
let songbird = CapabilityHttpClient::discover("http.client").await?;
let response = songbird.get(url).await?;
```

---

## ✅ Success Criteria

1. **cargo tree | grep reqwest** → 0 matches in default build ✅ (already achieved!)
2. **cargo tree | grep ring** → 0 matches ✅ (already achieved!)
3. **cargo check** → Compiles without errors
4. **cargo build --target <any>** → Cross-compiles successfully
5. **No #[cfg(feature = "http")]** → No feature gates, just removal
6. **Tests still pass** → reqwest in dev-dependencies only

---

## 📊 Progress Tracker

- [x] Understand scope (82 files identified)
- [x] Create removal plan
- [ ] Phase 1: Core Services (20% complete)
  - [x] routing.rs ✅
  - [x] ecosystem.rs ✅
  - [ ] federation.rs (in progress)
  - [ ] monitoring.rs
- [ ] Phase 2: AI Tools (80% complete - have capability_ai!)
  - [ ] Mark old providers deprecated
  - [ ] Remove from default exports
- [ ] Phase 3: Observability (0%)
- [ ] Phase 4: Plugins (0%)
- [ ] Phase 5: Update tests/examples (0%)

**Estimated Time Remaining**: 4-6 hours for complete removal

---

## 🚀 Quick Wins

### Immediate Actions (30 minutes)
1. Fix federation.rs to return "unimplemented" errors
2. Fix monitoring.rs same way
3. Get cargo check to pass
4. Test one cross-compile target

### Medium Term (2 hours)
1. Implement CapabilityHttpClient for Songbird delegation
2. Replace federation HTTP calls with capability pattern
3. Update auth to use BearDog pattern throughout

### Long Term (2-4 hours)
1. Clean up all old provider implementations
2. Update observability to file/socket based
3. Full 18-target cross-compilation validation
4. Documentation updates

---

## 💭 Philosophy Reminder

**TRUE PRIMAL Way**:
- Primals are specialists
- Squirrel = AI/MCP coordinator
- Songbird = Network/HTTP specialist
- BearDog = Security/Auth specialist
- No primal does everything
- Delegate via Unix sockets
- Discover via capabilities
- NO direct HTTP in Squirrel!

**Deploy like an infant** → Use specialists for specialist work! 🍼

---

*Status: Plan complete, execution 15% done*  
*Next: Fix federation.rs and monitoring.rs to get clean compile*  
*Then: Systematic removal per phase*

