# Phase 1.5: Zero-Hardcoding Evolution Plan

**Mission**: Deploy like an infant - zero knowledge, discover everything at runtime

---

## 🎯 SCOPE

### Hardcoding Audit
- **Vendor Names**: 136 references (OpenAI, HuggingFace, Ollama, Anthropic)
- **Port Numbers**: 145 references (8080, 9010, 9090, etc.)
- **Durations**: 327 references (hardcoded timeouts, sleeps)
- **Remaining Primal Refs**: Various in tests/configs

---

## 🚀 EXECUTION STRATEGY

### Phase A: Vendor Abstraction (30 min)
**Goal**: Replace vendor names with capability types

**Before**:
```rust
match provider_type {
    "openai" => OpenAIAdapter::new(),
    "huggingface" => HuggingFaceAdapter::new(),
    ...
}
```

**After**:
```rust
// Discovered via capability registry
let provider = registry.discover_by_capability("text.generation").await?;
```

**Actions**:
1. AI adapters already behind `#[cfg(feature = "dev-direct-http")]` ✅
2. Update router to use capability types only
3. Configuration uses capability names, not vendor names

### Phase B: Port Configuration (20 min)
**Goal**: All ports from config/env, zero hardcoding

**Before**:
```rust
let port = 9010; // Hardcoded
```

**After**:
```rust
let port = config.server.port.unwrap_or_else(|| {
    env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(9010) // Only default fallback
});
```

**Actions**:
1. Audit all `:8080`, `:9010` patterns
2. Move to config structs
3. Document env var overrides

### Phase C: Duration Configuration (20 min)
**Goal**: All timeouts configurable

**Before**:
```rust
timeout(Duration::from_secs(30), operation).await
```

**After**:
```rust
let timeout_secs = config.timeouts.operation.unwrap_or(30);
timeout(Duration::from_secs(timeout_secs), operation).await
```

**Actions**:
1. Create `TimeoutConfig` struct
2. Replace hardcoded `Duration::from_secs()` in critical paths
3. Keep test hardcoding (it's intentional)

### Phase D: Primal Self-Knowledge (30 min)
**Goal**: Complete the transformation

**Actions**:
1. Audit doctor.rs warning messages
2. Remove any "Songbird" references in user-facing strings
3. Update comments to be primal-agnostic
4. Verify zero primal hardcoding in production code

---

## 🎯 PRINCIPLES

### 1. Self-Knowledge Only
```rust
// ❌ BAD: Hardcoded other primal
if service_name == "songbird" { ... }

// ✅ GOOD: Capability-based
if service.has_capability("service_mesh") { ... }
```

### 2. Zero Vendor Lock-in
```rust
// ❌ BAD: Vendor-specific
let client = OpenAIClient::new();

// ✅ GOOD: Capability-based
let client = registry.get_provider("text.generation").await?;
```

### 3. Configurable Defaults
```rust
// ❌ BAD: Magic number
timeout(Duration::from_secs(30), ...)

// ✅ GOOD: Configurable with reasonable default
timeout(Duration::from_secs(config.timeout.unwrap_or(30)), ...)
```

### 4. Runtime Discovery
```rust
// ❌ BAD: Compile-time knowledge
const SONGBIRD_PORT: u16 = 8081;

// ✅ GOOD: Runtime discovery
let services = discover_services_with_capability("service_mesh").await?;
let endpoint = &services[0].endpoint; // Port in endpoint
```

---

## 📋 SCOPE BOUNDARIES

### What to Evolve
- ✅ Production code vendor names
- ✅ User-facing primal references
- ✅ Critical path ports/timeouts
- ✅ Configuration defaults

### What to Keep (Intentional)
- ✅ Test hardcoding (tests are explicit scenarios)
- ✅ Example configs (documentation)
- ✅ Feature-gated dev adapters (dev-direct-http)
- ✅ Deprecation notices (guide migration)

---

## 🎯 SUCCESS CRITERIA

- [ ] Zero vendor names in production routing logic
- [ ] All ports configurable via config/env
- [ ] Critical timeouts configurable
- [ ] No primal names in user-facing strings
- [ ] Doctor command uses generic discovery
- [ ] Tests still pass (187 tests)
- [ ] Binary still functional

---

## ⏰ ESTIMATED TIME: 2 hours

**Total Evolution**: Phases 1, 2, 1.5 = ~4.5 hours of 8-12 hour plan

---

**Philosophy**: "Deploy like an infant - knows nothing, discovers everything" 🐿️
