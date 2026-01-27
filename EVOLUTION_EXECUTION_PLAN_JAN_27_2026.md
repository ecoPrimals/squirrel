# 🚀 Evolution Execution Plan - Squirrel (Jan 27, 2026)

**Date**: January 27, 2026, 23:00 UTC  
**Scope**: Comprehensive technical debt elimination and modern Rust evolution  
**Timeline**: 6-8 weeks  
**Priority**: Production readiness

---

## 🎯 EXECUTION PHILOSOPHY

### Core Principles

1. **Deep Debt Solutions** - Fix root causes, not symptoms
2. **Modern Idiomatic Rust** - Follow 2024/2025 best practices
3. **Capability-Based Discovery** - Zero hardcoded primal knowledge
4. **Smart Refactoring** - Maintain cohesion, not just split files
5. **Safe & Fast Rust** - Eliminate unsafe where possible
6. **Complete Implementations** - No production mocks

### Standards Compliance

- **TRUE PRIMAL**: Self-knowledge only, runtime discovery
- **ecoBin**: Pure Rust, zero C dependencies
- **JSON-RPC First**: All IPC via JSON-RPC 2.0
- **90% Test Coverage**: Comprehensive testing
- **Idiomatic Rust**: 2024+ patterns

---

## 📋 EXECUTION ORDER (Priority-Based)

### Phase 1: CRITICAL BLOCKERS (Week 1)

#### ✅ 1.1 Fix Test Compilation [COMPLETED]
**Status**: ✅ ChatMessage API fixed, ChatOptions.top_p added

**Remaining**:
- Fix `total_tokens` field access in tests
- Verify all tests compile

**Action**:
```bash
# Fix remaining test error
cd crates/tools/ai-tools/examples
# Update capability_ai_demo.rs line 71:
response.usage.map(|u| u.total_tokens)
```

**Effort**: 30 minutes  
**Priority**: P0

#### 1.2 Format All Code

**Command**:
```bash
cd /home/eastgate/Development/ecoPrimals/phase1/squirrel
cargo fmt
```

**Affected**: 4 files  
**Effort**: 1 minute  
**Priority**: P0

#### 1.3 Measure Actual Test Coverage

**Commands**:
```bash
# After fixing compilation:
cargo test
cargo llvm-cov --html
firefox target/llvm-cov/html/index.html
```

**Purpose**: Establish baseline for coverage improvements  
**Effort**: 30 minutes  
**Priority**: P0

---

### Phase 2: HARDCODED PRIMAL REFERENCES (Weeks 1-2)

**Total**: 667 occurrences across 82 files

#### 2.1 Evolution Strategy

**Pattern**: Replace `EcosystemPrimalType` with capability-based discovery

**Before** (Hardcoded):
```rust
use crate::ecosystem::types::EcosystemPrimalType;

// ❌ BAD: Compile-time coupling
match primal_type {
    EcosystemPrimalType::Songbird => endpoint = "/primal/songbird",
    EcosystemPrimalType::BearDog => endpoint = "/primal/beardog",
}
```

**After** (Capability-Based):
```rust
use crate::discovery::capability_resolver::CapabilityResolver;

// ✅ GOOD: Runtime discovery
let resolver = CapabilityResolver::new()?;
let endpoint = resolver
    .discover_capability("http.proxy")  // What we need
    .await?
    .endpoint;  // Who provides it (discovered!)
```

#### 2.2 Priority Files (High Impact)

**Top 10 Files to Evolve**:

1. **`crates/main/src/ecosystem/mod.rs`** - 42 references
   - Core ecosystem integration
   - Remove `EcosystemPrimalType` usage
   - Implement capability discovery

2. **`crates/main/src/biomeos_integration/mod.rs`** - 46 references
   - BiomeOS coordination
   - Use capability-based service location
   - Remove hardcoded primal names

3. **`crates/main/src/ecosystem/types.rs`** - 25 references
   - Type definitions
   - Already deprecated ✅
   - Remove usage in other modules

4. **`crates/main/src/security/mod.rs`** - 14 references
   - Security coordination
   - Already has `beardog_coordinator.rs` (refactor this)
   - Use capability discovery for crypto

5. **`crates/main/src/ai/model_splitting/mod.rs`** - 39 references
   - AI routing
   - Use capability discovery for AI providers
   - Remove hardcoded service names

#### 2.3 Implementation Pattern

**Step-by-Step**:

1. **Identify hardcoded reference**:
```rust
// Find this pattern:
EcosystemPrimalType::Songbird
// or
"songbird"
"beardog"
"nestgate"
```

2. **Determine capability needed**:
```rust
// What does this code actually need?
// - HTTP proxy? → "http.proxy"
// - Crypto service? → "crypto.sign"  
// - Storage? → "storage.put"
// - Discovery? → "discovery.announce"
```

3. **Replace with discovery**:
```rust
// Add at module level:
use crate::discovery::capability_resolver::CapabilityResolver;

// Replace usage:
async fn get_service() -> Result<Endpoint> {
    let resolver = CapabilityResolver::new()?;
    let service = resolver
        .discover_capability("http.proxy")
        .await?;
    Ok(service.endpoint)
}
```

4. **Update tests**:
```rust
#[tokio::test]
async fn test_capability_discovery() {
    // Mock capability resolver
    let resolver = CapabilityResolver::mock(vec![
        MockCapability::new("http.proxy", "/tmp/proxy.sock"),
    ]);
    // Test with mocked discovery
}
```

#### 2.4 Execution Plan

**Week 1** (20 hours):
- Day 1-2: `ecosystem/mod.rs` + `biomeos_integration/mod.rs`
- Day 3: `security/` modules
- Day 4: `ai/model_splitting/mod.rs`
- Day 5: Review and test

**Week 2** (20 hours):
- Day 1-3: Remaining 78 files (lower density)
- Day 4: Integration testing
- Day 5: Documentation update

**Target**: Zero hardcoded primal names ✅

---

### Phase 3: PRODUCTION MOCKS ELIMINATION (Week 3)

**Total**: ~300 production mocks (excluding test files)

#### 3.1 Audit Strategy

**Step 1**: Identify all mocks in production code
```bash
# Find production mocks
rg -i "mock|stub|fake" crates/main/src --type rust | \
  grep -v test | grep -v "comment" > production_mocks.txt
```

**Step 2**: Categorize by purpose
- **HTTP mocks** → Evolve to Unix socket + capability discovery
- **Storage mocks** → Implement real file/memory storage
- **Crypto mocks** → Delegate to capability-based provider
- **AI mocks** → Delegate to capability-based provider

**Step 3**: Create evolution plan per category

#### 3.2 Common Patterns

**Pattern 1: HTTP Mock → Capability Delegation**

Before:
```rust
// Mock HTTP client
pub struct MockHttpClient;

impl HttpClient for MockHttpClient {
    async fn get(&self, url: &str) -> Result<String> {
        Ok("mock response".to_string())  // ❌ Mock
    }
}
```

After:
```rust
// Real capability-based client
pub struct CapabilityHttpClient {
    resolver: CapabilityResolver,
}

impl HttpClient for CapabilityHttpClient {
    async fn get(&self, url: &str) -> Result<String> {
        let http_provider = self.resolver
            .discover_capability("http.proxy")
            .await?;
        
        // Send JSON-RPC request to actual provider
        send_json_rpc_request(
            &http_provider.endpoint,
            "http.get",
            json!({"url": url})
        ).await
    }
}
```

**Pattern 2: Storage Mock → Real Implementation**

Before:
```rust
pub struct MockStorage {
    data: HashMap<String, Vec<u8>>,  // ❌ In-memory mock
}
```

After:
```rust
pub enum Storage {
    Memory(MemoryStorage),     // ✅ Real in-memory (for tests/cache)
    File(FileStorage),         // ✅ Real file storage
    Capability(CapabilityStorage),  // ✅ Delegated to NestGate
}

impl Storage {
    pub fn from_config(config: &StorageConfig) -> Result<Self> {
        match config.backend {
            StorageBackend::Memory => Ok(Self::Memory(MemoryStorage::new())),
            StorageBackend::File => Ok(Self::File(FileStorage::new(&config.path)?)),
            StorageBackend::Capability => {
                // Discover storage capability at runtime
                Ok(Self::Capability(CapabilityStorage::discover().await?))
            }
        }
    }
}
```

#### 3.3 Execution

**Week 3** (40 hours):
- Day 1: Audit and categorize all production mocks
- Day 2-3: Evolve HTTP mocks to capability delegation
- Day 4: Evolve storage/crypto mocks
- Day 5: Testing and validation

**Target**: Zero production mocks ✅

---

### Phase 4: ERROR HANDLING EVOLUTION (Week 4)

**Total**: 494 `unwrap()`/`expect()` calls across 69 files

#### 4.1 Prioritization

**Critical** (High-traffic code):
- `monitoring/metrics/collector.rs` - 38 unwraps
- `ecosystem/` modules - ~50 unwraps
- `biomeos_integration/` - ~40 unwraps

**Medium** (Tests, but used in examples):
- Test files - ~200 unwraps (acceptable but improve)

**Low** (Tests only):
- Pure test code - ~166 unwraps (acceptable)

#### 4.2 Evolution Pattern

**Before** (Panic risk):
```rust
pub fn process_data(&self, key: &str) -> Data {
    let data = self.map.get(key).unwrap();  // ❌ Will panic!
    data.clone()
}
```

**After** (Proper error handling):
```rust
pub fn process_data(&self, key: &str) -> Result<Data> {
    let data = self.map.get(key)
        .ok_or_else(|| anyhow!("Missing data for key: {}", key))?;  // ✅ Error
    Ok(data.clone())
}

// Or with context:
pub fn process_data(&self, key: &str) -> Result<Data> {
    self.map.get(key)
        .cloned()
        .context(format!("Missing data for key: {}", key))  // ✅ Rich error
}
```

#### 4.3 Execution

**Week 4** (30 hours):
- Day 1: Fix `monitoring/metrics/collector.rs` (38 unwraps)
- Day 2: Fix `ecosystem/` modules
- Day 3: Fix `biomeos_integration/` modules
- Day 4-5: Review and update error types

**Target**: <10 unwraps in production code ✅

---

### Phase 5: UNSAFE CODE EVOLUTION (Week 5, Part 1)

**Total**: 28 `unsafe` blocks across 10 files

#### 5.1 Analysis

**Justified Use Cases** (Keep):
- Plugin FFI (8 in `test_dynamic_plugin.rs`) - Required for dynamic loading
- Dynamic loading (2 in `dynamic_example.rs`) - Required for plugins

**Potentially Eliminable**:
- `enhanced/serialization/codecs.rs` (6 blocks) - Review for safe alternatives
- `cli/plugins/` (7 blocks) - May have safe alternatives

#### 5.2 Evolution Pattern

**Pattern 1: Transmute → Type-Safe Conversion**

Before:
```rust
unsafe {
    std::mem::transmute::<T, U>(value)  // ❌ Dangerous
}
```

After:
```rust
// Use proper serialization
let bytes = bincode::serialize(&value)?;  // ✅ Safe
bincode::deserialize::<U>(&bytes)?
```

**Pattern 2: Raw Pointers → Safe Abstractions**

Before:
```rust
unsafe {
    let ptr = data.as_ptr();  // ❌ Raw pointer
    *ptr
}
```

After:
```rust
data.get(0).copied()  // ✅ Safe with bounds check
```

#### 5.3 Execution

**Week 5, Days 1-2** (16 hours):
- Audit all 28 unsafe blocks
- Document justification for each
- Evolve where possible
- Add safety comments to remaining

**Target**: <15 unsafe blocks, all documented ✅

---

### Phase 6: LARGE FILE REFACTORING (Week 5, Part 2)

**Files >1000 Lines**:
1. `enhanced/workflow/execution.rs` - 1027 lines
2. `context/src/rules/evaluator_tests.rs` - 1017 lines
3. `adapter-pattern-tests/src/lib.rs` - 1012 lines

#### 6.1 Smart Refactoring Strategy

**NOT**: Just split into arbitrary chunks  
**YES**: Logical domain separation

**Example: `workflow/execution.rs` (1027 lines)**

**Analysis**:
```rust
// Current structure (single file):
- WorkflowExecutor (main struct)
- ExecutionState (state machine)
- StepRunner (step execution)
- ErrorRecovery (error handling)
- ProgressTracking (progress updates)
```

**Smart Refactoring**:
```
workflow/
  ├── execution.rs (250 lines)     - Main executor coordination
  ├── state.rs (200 lines)         - State machine
  ├── step_runner.rs (200 lines)   - Step execution
  ├── recovery.rs (200 lines)      - Error recovery
  ├── progress.rs (177 lines)      - Progress tracking
  └── mod.rs (50 lines)            - Public API
```

**Principles**:
- ✅ Each module has single responsibility
- ✅ Clear boundaries between concerns
- ✅ Maintains logical cohesion
- ✅ Easy to test independently
- ✅ Reduces cognitive load

#### 6.2 Execution

**Week 5, Days 3-5** (24 hours):
- Day 3: Refactor `workflow/execution.rs`
- Day 4: Refactor test files (if needed)
- Day 5: Testing and documentation

**Target**: All files <1000 lines ✅

---

### Phase 7: TEST COVERAGE EXPANSION (Weeks 6-7)

**Current**: <50%  
**Target**: 90%

#### 7.1 Coverage Strategy

**Priority 1 - Core Modules** (Week 6):
- `ecosystem/` - Critical for primal coordination
- `discovery/` - Critical for capability discovery
- `security/` - Critical for auth/crypto
- `biomeos_integration/` - Critical for orchestration

**Priority 2 - Integration** (Week 7, Part 1):
- End-to-end scenarios
- Cross-module integration
- Error path coverage

**Priority 3 - Chaos/Fault** (Week 7, Part 2):
- Service failure scenarios
- Network interruption
- Resource exhaustion
- Recovery mechanisms

#### 7.2 Test Types

**1. Unit Tests**
```rust
#[test]
fn test_capability_discovery_success() {
    // Test successful discovery
}

#[test]
fn test_capability_discovery_not_found() {
    // Test error case
}

#[test]
fn test_capability_discovery_timeout() {
    // Test timeout handling
}
```

**2. Integration Tests**
```rust
#[tokio::test]
async fn test_end_to_end_ai_request() {
    // Start mock capability provider
    // Send request through Squirrel
    // Verify response
    // Check metrics
}
```

**3. Chaos Tests** (Directory already exists!)
```rust
#[tokio::test]
async fn test_service_failure_recovery() {
    // Start service
    // Kill service mid-request
    // Verify graceful handling
    // Verify retry logic
}
```

**4. Property-Based Tests**
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_capability_parsing_never_panics(s in "\\PC*") {
        // Test with random inputs
        let _ = parse_capability(&s);  // Should never panic
    }
}
```

#### 7.3 Execution

**Week 6** (40 hours):
- Core module unit tests
- Target: 70% coverage

**Week 7** (40 hours):
- Integration tests
- Chaos/fault tests
- Target: 90% coverage ✅

---

### Phase 8: EXTERNAL DEPENDENCY ANALYSIS (Week 8)

#### 8.1 Current Dependencies

**Already Pure Rust** ✅:
- `tokio` - Async runtime
- `serde`/`serde_json` - Serialization
- `anyhow`/`thiserror` - Error handling
- `tracing` - Logging
- `clap` - CLI

**Requires Analysis**:
- None! We're already ecoBin compliant ✅

**Feature-Gated** (Optional):
- `reqwest` - Only in optional features

#### 8.2 Dependency Health Check

**Commands**:
```bash
# Check for outdated dependencies
cargo outdated

# Check for vulnerabilities
cargo audit

# Check for unmaintained deps
cargo tree --depth 1 | grep -E "yanked|deprecated"
```

#### 8.3 Execution

**Week 8, Day 1** (8 hours):
- Run dependency analysis
- Update outdated dependencies
- Document any issues
- Create upgrade plan if needed

**Target**: All deps current and secure ✅

---

## 📊 PROGRESS TRACKING

### Daily Checklist

```markdown
## [Date]

### Completed
- [ ] Fixed [X] test compilation errors
- [ ] Removed [X] hardcoded references
- [ ] Evolved [X] production mocks
- [ ] Fixed [X] unwrap/expect calls
- [ ] Reviewed [X] unsafe blocks
- [ ] Refactored [X] large files
- [ ] Added [X] tests (coverage: [X]%)

### Blockers
- None / [Describe blocker]

### Next Steps
- [Specific next task]

### Metrics
- Build: ✅/❌
- Tests: [X]/[Y] passing
- Coverage: [X]%
- Hardcoded refs: [X] remaining
```

### Weekly Review

**End of each week**:
1. Run full test suite
2. Measure coverage
3. Check all metrics
4. Update TODO list
5. Document learnings

---

## 🎯 SUCCESS CRITERIA

### Week 1
- ✅ Tests compile
- ✅ Code formatted
- ✅ Baseline coverage measured
- 🔄 Started hardcoded ref removal

### Week 2
- ✅ All hardcoded refs removed
- ✅ Capability discovery working
- 🔄 Started mock elimination

### Week 3
- ✅ Production mocks eliminated
- ✅ Real implementations complete
- 🔄 Started error handling fixes

### Week 4
- ✅ <10 unwrap/expect in production
- ✅ Proper error propagation
- 🔄 Started unsafe review

### Week 5
- ✅ Unsafe code reviewed/evolved
- ✅ Large files refactored
- 🔄 Started test expansion

### Week 6-7
- ✅ 90% test coverage
- ✅ e2e tests complete
- ✅ Chaos tests working

### Week 8
- ✅ Dependencies analyzed
- ✅ All metrics green
- ✅ **PRODUCTION READY** 🎉

---

## 🛠️ TOOLS & COMMANDS

### Essential Commands

```bash
# Format code
cargo fmt

# Check compilation
cargo check --all-targets

# Run tests
cargo test

# Measure coverage
cargo llvm-cov --html

# Find hardcoded refs
rg -i "beardog|songbird|nestgate" crates/main/src

# Find unwraps
rg "\.unwrap\(\)|\.expect\(" crates/main/src

# Find unsafe
rg "unsafe" crates/main/src

# Find large files
find crates -name "*.rs" -exec wc -l {} \; | awk '$1 > 1000'

# Check dependencies
cargo tree --depth 1
cargo outdated
cargo audit
```

### Automation Scripts

**Create `scripts/evolution-check.sh`**:
```bash
#!/bin/bash
# Evolution progress checker

echo "🔍 Evolution Progress Check"
echo ""

echo "📊 Hardcoded Refs:"
rg -i "beardog|songbird|nestgate|toadstool" crates/main/src --type rust -c | \
    awk '{sum+=$1} END {print "  Remaining: " sum}'

echo ""
echo "🔧 unwrap/expect:"
rg "\.unwrap\(\)|\.expect\(" crates/main/src --type rust -c | \
    awk '{sum+=$1} END {print "  Remaining: " sum}'

echo ""
echo "🧪 Test Coverage:"
cargo llvm-cov --quiet 2>&1 | grep "TOTAL" | awk '{print "  Coverage: " $10}'

echo ""
echo "🏗️ Build Status:"
cargo check --all-targets > /dev/null 2>&1 && echo "  ✅ Pass" || echo "  ❌ Fail"

echo ""
echo "✅ Evolution check complete!"
```

---

## 📚 REFERENCE PATTERNS

### Capability Discovery Pattern

```rust
use crate::discovery::capability_resolver::CapabilityResolver;

pub struct ServiceClient {
    resolver: CapabilityResolver,
}

impl ServiceClient {
    pub fn new() -> Result<Self> {
        Ok(Self {
            resolver: CapabilityResolver::new()?,
        })
    }

    pub async fn call_service(&self, capability: &str, method: &str, params: serde_json::Value) -> Result<serde_json::Value> {
        // Discover service at runtime
        let service = self.resolver
            .discover_capability(capability)
            .await?;

        // Connect to service
        let mut stream = tokio::net::UnixStream::connect(&service.endpoint).await?;

        // Send JSON-RPC request
        send_json_rpc_request(&mut stream, method, params).await
    }
}
```

### Error Handling Pattern

```rust
use anyhow::{Context, Result};

pub fn process_config(path: &str) -> Result<Config> {
    let content = std::fs::read_to_string(path)
        .context(format!("Failed to read config file: {}", path))?;

    let config: Config = serde_json::from_str(&content)
        .context("Failed to parse config JSON")?;

    Ok(config)
}
```

### Mock Elimination Pattern

```rust
// Feature-gate test utilities
#[cfg(test)]
pub mod mock {
    pub struct MockService {
        // Mock implementation for tests only
    }
}

// Production uses real implementation
pub struct RealService {
    resolver: CapabilityResolver,
}

#[cfg(not(test))]
pub type Service = RealService;

#[cfg(test)]
pub type Service = mock::MockService;
```

---

## 🎓 LESSONS LEARNED (Update Weekly)

### Week 1
- [Document learnings here]

### Week 2
- [Document learnings here]

---

## 📞 SUPPORT & RESOURCES

### Documentation
- `COMPREHENSIVE_AUDIT_JAN_27_2026_EVENING.md` - Full audit
- `MIGRATION_GUIDE_HARDCODED_TO_CAPABILITY.md` - Migration patterns
- `wateringHole/PRIMAL_IPC_PROTOCOL.md` - IPC standards

### Questions
- Post in wateringHole for inter-primal coordination
- Check existing evolution examples in BearDog/Songbird

---

**Execution Plan**: v1.0  
**Created**: January 27, 2026, 23:00 UTC  
**Status**: **READY TO EXECUTE** ✅

🐿️🦀✨ **From Technical Debt to Production Excellence!** ✨🦀🐿️

