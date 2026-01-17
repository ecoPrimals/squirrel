# Deep Evolution Plan - January 17, 2026

**Purpose**: Evolve codebase to modern idiomatic Rust with deep debt solutions  
**Principles**:
1. **Primal Self-Knowledge**: Only encode what Squirrel knows about itself
2. **Capability Discovery**: Discover other primals at runtime via capability adapter
3. **No Hardcoding**: No primal names (Songbird, BearDog, ToadStool, NestGate)
4. **Mocks → Real**: Replace mocks in production with complete implementations
5. **Unsafe → Safe**: Evolve unsafe code to fast AND safe Rust
6. **Smart Refactoring**: Large files refactored intelligently, not just split

---

## 🎯 Critical Violations Found

### 1. **HARDCODING VIOLATIONS** (❌ CRITICAL)

**Files with Primal Name Hardcoding** (993 instances!):

#### Worst Offenders:
1. **`crates/main/src/ecosystem/mod.rs`** (983 lines)
   - Hardcodes: `Songbird`, `BearDog`, `ToadStool`, `NestGate`
   - Has enum: `EcosystemPrimalType::Songbird, ::BearDog, etc.`
   - **VIOLATION**: Squirrel shouldn't know these exist!

2. **`crates/main/src/doctor.rs`** (checks)
   - `check_songbird_connectivity()` - hardcoded function name
   - `check_beardog_connectivity()` - hardcoded function name
   - **VIOLATION**: Knows about specific primals

3. **`crates/main/src/api/server.rs`**
   - `SongbirdAiIntegration` - hardcoded struct
   - `songbird::handle_songbird_register` - hardcoded module
   - **VIOLATION**: Direct Songbird knowledge

4. **`crates/main/src/api/ai/router.rs`**
   - Comments: "With Songbird", "Without Songbird"
   - Parameter: `_songbird_client`
   - **VIOLATION**: Hardcoded primal name in API

5. **`crates/main/src/songbird/mod.rs`** (753 lines!)
   - **VIOLATION**: Entire module dedicated to one primal!
   - Should be: capability discovery, not hardcoded integration

6. **`crates/main/src/beardog.rs`**
   - **VIOLATION**: Entire file for one primal
   - Should be: security capability discovery

7. **`crates/main/src/toadstool.rs`**
   - **VIOLATION**: Entire file for one primal
   - Should be: compute capability discovery

**Examples of Violations**:
```rust
// ❌ BAD: Hardcoded primal names
enum EcosystemPrimalType {
    Songbird,
    BearDog,
    ToadStool,
    NestGate,
}

// ❌ BAD: Songbird-specific code
impl SongbirdAiIntegration {
    pub async fn register_capabilities() -> Result<()> { ... }
}

// ❌ BAD: Hardcoded primal checks
async fn check_songbird_connectivity() -> HealthCheck { ... }
async fn check_beardog_connectivity() -> HealthCheck { ... }
```

**✅ CORRECT APPROACH**:
```rust
// ✅ GOOD: Capability-based discovery
let capabilities = capability_registry.discover("ai.text-generation").await?;

// ✅ GOOD: Generic health checks
async fn check_capability_connectivity(capability: &str) -> HealthCheck {
    let providers = discover_capability(capability).await?;
    // Check each discovered provider
}

// ✅ GOOD: No primal names in code
let security_provider = discover_capability("security.auth").await?;
let compute_provider = discover_capability("compute.gpu").await?;
```

---

### 2. **MOCKS IN PRODUCTION CODE** (⚠️ HIGH)

**Production Files with Mocks**:
1. `crates/main/src/testing/mock_providers.rs` (in `src/`, not `tests/`)
2. `crates/main/src/discovery/mechanisms/registry_trait.rs` (has `MockRegistryProvider`)
3. `crates/main/src/compute_client/provider_trait.rs` (has `MockComputeProvider`)
4. `crates/ecosystem-api/src/client.rs` (has `MockServiceMeshClient` - 286 lines!)
5. `crates/core/mcp/src/enhanced/server.rs` (has `MockPluginManager`)

**Examples**:
```rust
// ❌ BAD: Mock in production code
// crates/ecosystem-api/src/client.rs
pub struct MockServiceMeshClient {
    registered_services: Arc<Mutex<HashMap<String, ServiceRegistration>>>,
}

// ❌ BAD: Mock in main crate
// crates/main/src/testing/mock_providers.rs
pub struct MockEcosystemManager { ... }
pub struct MockProvider { ... }
```

**✅ SOLUTION**: 
- Move all `Mock*` to `tests/` directory
- In production: Use real implementations or `Result<T, E>` for unavailable features
- Testing module in `src/testing/` should provide test utilities, not mocks

---

### 3. **UNSAFE CODE** (✅ MOSTLY GOOD)

**Status**: ✅ **Excellent!** - Zero unsafe blocks in main code
- `#![deny(unsafe_code)]` in main crates ✅
- Only unsafe references in old Windows clippy errors (not current code)
- Plugin system explicitly avoids unsafe

**No Action Needed**: Unsafe code is already eliminated!

---

### 4. **INCOMPLETE IMPLEMENTATIONS** (⚠️ MEDIUM)

**Placeholder Implementations** (from earlier audit):

1. **Songbird Capability Discovery** (router.rs line 78)
   ```rust
   // TODO: Implement actual Songbird capability discovery
   ```
   **Status**: Placeholder comment, no implementation

2. **Daemon Mode** (main.rs line 74)
   ```rust
   _daemon: bool, // TODO: Implement daemon mode
   ```
   **Status**: Flag exists but unused

3. **MCP Adapter** (router/mcp_adapter.rs line 279)
   ```rust
   // TODO: Complete MCP adapter implementation
   ```
   **Status**: Stub types only

4. **Plugin Sandboxing** (plugins/security.rs line 136)
   ```rust
   // TODO: Implement proper sandboxed plugin loading through WebAssembly
   ```
   **Status**: Secure stub instead of WASM

5. **Streaming Inference** (multiple files)
   - `local/native.rs` line 483
   - `universal_provider.rs` line 384
   **Status**: Not implemented

---

### 5. **LARGE FILES FOR SMART REFACTORING** (📊 INFO)

**Files > 800 lines** (candidates for refactoring):

1. **`monitoring/metrics/collector.rs`** (992 lines)
   - **Reason**: Metrics collection is complex domain
   - **Refactor**: Split by metric type (system, app, custom)

2. **`ecosystem/mod.rs`** (983 lines) ❌ **PRIORITY**
   - **Reason**: Contains hardcoded primal types
   - **Refactor**: Remove hardcoding, split discovery/registry/types

3. **`universal_primal_ecosystem/mod.rs`** (974 lines)
   - **Reason**: Large ecosystem coordination
   - **Refactor**: Split into coordinator, registry, discovery

4. **`error_handling/safe_operations.rs`** (888 lines)
   - **Reason**: Comprehensive error handling
   - **Refactor**: Group by error category (IO, network, parse, etc.)

5. **`biomeos_integration/*.rs`** (multiple 800+ line files)
   - **Reason**: Complex biomeOS integration
   - **Refactor**: Split by concern (deployment, context, manifest)

6. **`songbird/mod.rs`** (753 lines) ❌ **PRIORITY**
   - **Reason**: Hardcoded Songbird integration
   - **Refactor**: DELETE and replace with capability discovery

---

## 🚀 EVOLUTION PLAN

### Phase 1: Remove Hardcoding (CRITICAL) ⚡

**Priority**: P0 - Violates TRUE PRIMAL architecture

#### Step 1.1: Analyze Songbird Module Dependencies
- Check what `songbird/mod.rs` exports
- Identify capabilities needed
- Map to capability discovery patterns

#### Step 1.2: Evolve Ecosystem Types
- Remove `EcosystemPrimalType` enum (hardcoded names)
- Replace with capability-based discovery
- Update all call sites

#### Step 1.3: Remove Primal-Specific Modules
- **DELETE**: `crates/main/src/songbird/mod.rs` (753 lines)
- **DELETE**: `crates/main/src/beardog.rs`
- **DELETE**: `crates/main/src/toadstool.rs`
- **REPLACE**: With capability adapter integration

#### Step 1.4: Evolve Doctor Health Checks
- Remove `check_songbird_connectivity()`
- Remove `check_beardog_connectivity()`
- Add `check_capability_connectivity(capability: &str)`

#### Step 1.5: Evolve AI Router
- Remove `_songbird_client` parameter
- Use `capability_registry` instead
- Discover AI providers via capability, not Songbird

#### Step 1.6: Evolve API Server
- Remove `SongbirdAiIntegration`
- Replace with `CapabilityIntegration`
- Generic registration, not Songbird-specific

---

### Phase 2: Move Mocks to Tests (HIGH) ⚡

**Priority**: P1 - Production code should have zero mocks

#### Step 2.1: Move Testing Mocks
```bash
# Move mock_providers.rs to tests
mv crates/main/src/testing/mock_providers.rs crates/main/tests/common/mock_providers.rs
```

#### Step 2.2: Evolve Ecosystem API
- Remove `MockServiceMeshClient` from `ecosystem-api/src/client.rs`
- Move to test fixtures

#### Step 2.3: Evolve Registry Mocks
- Remove `MockRegistryProvider` from production
- Move to test utilities

#### Step 2.4: Verify No Mocks in Src
```bash
grep -r "struct Mock" crates/*/src --include="*.rs" | grep -v "tests/"
# Should return ZERO results
```

---

### Phase 3: Complete Implementations (MEDIUM) ⚡

**Priority**: P2 - Fill in TODOs with real code

#### Step 3.1: Complete Songbird → Capability Discovery
**File**: `crates/main/src/api/ai/router.rs`

**Current**:
```rust
// TODO: Implement actual Songbird capability discovery
```

**Evolution**:
```rust
// Discover AI text generation providers via capability registry
let text_gen_providers = capability_registry
    .discover("ai.text-generation")
    .await?;

// Discover AI image generation providers
let image_gen_providers = capability_registry
    .discover("ai.image-generation")
    .await?;

// Create adapters for each discovered provider
for provider in text_gen_providers {
    let adapter = UniversalAiAdapter::from_capability(provider).await?;
    providers.push(Arc::new(adapter));
}
```

#### Step 3.2: Daemon Mode Decision
**Options**:
1. Implement full daemon mode (fork, PID file, etc.)
2. Remove flag (use systemd/supervisor instead)

**Recommendation**: Option 2 (remove flag) - modern services use systemd

#### Step 3.3: Complete MCP Adapter
- Implement full JSON-RPC over Unix socket
- Add streaming support
- Add error handling

---

### Phase 4: Smart Refactoring (LOW) 📊

**Priority**: P3 - Improve maintainability

#### Files to Refactor:
1. **`ecosystem/mod.rs`** → Split into:
   - `ecosystem/types.rs` (pure types)
   - `ecosystem/discovery.rs` (capability discovery)
   - `ecosystem/manager.rs` (ecosystem coordination)
   - `ecosystem/config.rs` (configuration)

2. **`monitoring/metrics/collector.rs`** → Split into:
   - `metrics/system.rs` (system metrics)
   - `metrics/application.rs` (app metrics)
   - `metrics/custom.rs` (custom metrics)
   - `metrics/collector.rs` (coordination)

3. **`error_handling/safe_operations.rs`** → Split into:
   - `errors/io.rs` (IO errors)
   - `errors/network.rs` (network errors)
   - `errors/parse.rs` (parsing errors)
   - `errors/operations.rs` (safe operation wrappers)

---

## 📋 ACTIONABLE TASKS (Priority Order)

### Immediate (Today)

1. **✅ Analyze Songbird Module**
   - Read `songbird/mod.rs` to understand current usage
   - Identify what capabilities it provides
   - Map to capability discovery patterns
   - **Estimated**: 30 minutes

2. **✅ Create Capability Discovery Plan**
   - Design agnostic capability interface
   - Plan migration from hardcoded to discovered
   - **Estimated**: 30 minutes

### Short-Term (This Week)

3. **⚡ Remove Hardcoded Primal Names** (Phase 1)
   - Evolve `EcosystemPrimalType` enum
   - Remove Songbird/BearDog/ToadStool modules
   - Update all call sites
   - **Estimated**: 4-6 hours
   - **Impact**: HIGH - fixes critical architecture violation

4. **⚡ Move Mocks to Tests** (Phase 2)
   - Move all `Mock*` from `src/` to `tests/`
   - **Estimated**: 1-2 hours
   - **Impact**: MEDIUM - cleaner production code

5. **⚡ Complete Capability Discovery** (Phase 3.1)
   - Implement real capability discovery in router
   - Remove TODO placeholder
   - **Estimated**: 2-3 hours
   - **Impact**: HIGH - enables TRUE PRIMAL architecture

### Long-Term (This Month)

6. **📊 Smart Refactoring** (Phase 4)
   - Refactor large files intelligently
   - **Estimated**: 2-3 days total
   - **Impact**: MEDIUM - improved maintainability

---

## 🎯 SUCCESS CRITERIA

### Phase 1 Complete When:
- ✅ ZERO hardcoded primal names in production code
- ✅ No `Songbird`, `BearDog`, `ToadStool`, `NestGate` strings
- ✅ All discovery via capability registry
- ✅ Enum `EcosystemPrimalType` deleted or genericized

### Phase 2 Complete When:
- ✅ ZERO `Mock*` structs in `src/` directories
- ✅ All mocks in `tests/` only
- ✅ Production code uses real implementations or `Result<T, E>`

### Phase 3 Complete When:
- ✅ All TODO placeholders replaced with real code
- ✅ Capability discovery fully implemented
- ✅ No incomplete implementations in critical paths

### Phase 4 Complete When:
- ✅ No files > 1000 lines
- ✅ Files split by logical domains
- ✅ Clear module boundaries

---

## 🔍 BEFORE & AFTER

### BEFORE (Current Violations):
```rust
// ❌ Hardcoded primal names
enum EcosystemPrimalType {
    Songbird,
    BearDog,
    ToadStool,
    NestGate,
}

// ❌ Songbird-specific code
pub mod songbird;
impl SongbirdAiIntegration { ... }

// ❌ Mocks in production
pub struct MockServiceMeshClient { ... }

// ❌ Hardcoded checks
async fn check_songbird_connectivity() { ... }
```

### AFTER (Evolved):
```rust
// ✅ Capability-based discovery (no hardcoding)
let capabilities = capability_registry
    .discover("ai.text-generation")
    .await?;

// ✅ Generic integration (works with ANY primal)
pub struct CapabilityIntegration {
    capability: String,
    providers: Vec<CapabilityProvider>,
}

// ✅ No mocks in production (moved to tests/)
// All production code uses real implementations

// ✅ Generic health checks
async fn check_capability(capability: &str) -> HealthCheck {
    let providers = discover_capability(capability).await?;
    // Health check for discovered providers
}
```

---

**Status**: Ready to execute  
**Next Action**: Analyze Songbird module to plan evolution  
**Estimated Total Effort**: 8-12 hours for Phases 1-3  
**Impact**: ✅ **TRUE PRIMAL ARCHITECTURE ACHIEVED**

🦀 **Let's evolve to self-aware, capability-based Squirrel!** 🐿️

