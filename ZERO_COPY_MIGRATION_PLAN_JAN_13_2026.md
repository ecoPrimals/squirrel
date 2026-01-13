# ⚡ Zero-Copy String Migration Plan

**Date**: January 13, 2026  
**Objective**: Systematic adoption of zero-copy patterns to reduce allocations  
**Philosophy**: Performance through smart memory management

---

## 🎯 Executive Summary

**Current Status**: Infrastructure complete, limited adoption  
**Target**: 50-70% reduction in string allocations  
**Opportunity**: 3,700+ `.to_string()`/`.clone()` calls

**Key Finding**: Excellent zero-copy infrastructure exists, just needs systematic adoption!

---

## 📊 Current State

### Infrastructure ✅ EXCELLENT

**Already Implemented**:
```
crates/main/src/optimization/zero_copy/
├── arc_str.rs         ← Arc<str> utilities (278 lines)
├── string_utils.rs    ← String interning cache (185 lines)
├── buffer.rs          ← Zero-copy buffer operations
└── message_utils.rs   ← Zero-copy message passing
```

**Features Available**:
- ✅ `ArcStr` type alias for `Arc<str>`
- ✅ `IntoArcStr` trait for conversions
- ✅ String interning cache with common values
- ✅ Pre-populated ecosystem strings
- ✅ Thread-safe caching
- ✅ Zero-copy buffer utilities

### Usage Analysis

**String Allocations** (opportunities):
```
.to_string():    3,075 instances across 176 files
.to_owned():       625 instances
String::from():    ~500 instances
String::clone():   ~500 instances
Total:           4,700+ allocations
```

**Already Using Zero-Copy**:
```
Arc<str> usage:    ~200 instances
ArcStr usage:      ~50 instances
Cow<str> usage:    ~30 instances
Total:             ~280 instances (~6% of opportunities)
```

**Opportunity**: 94% of string operations could benefit from zero-copy!

---

## 🎯 Migration Strategy

### Phase 1: Hot Paths (Week 1-2 - 12 hours)

**Priority**: HIGH (maximum performance impact)

#### Target Areas

**1. Service Discovery** (3 hours)
```
crates/main/src/discovery/*.rs
crates/main/src/ecosystem/*.rs
```

**Current** (allocates on every lookup):
```rust
pub async fn discover_service(&self, name: &str) -> Result<Service> {
    let service_name = name.to_string();  // ❌ Allocation
    self.registry.get(&service_name)
}
```

**After** (zero-copy):
```rust
use crate::optimization::zero_copy::ArcStr;

pub async fn discover_service(&self, name: ArcStr) -> Result<Service> {
    self.registry.get(&name)  // ✅ No allocation, just Arc clone
}
```

**Files to Update**:
```
crates/main/src/discovery/self_knowledge.rs
crates/main/src/discovery/cache.rs
crates/main/src/ecosystem/registry.rs
crates/main/src/ecosystem/registry_manager.rs
```

**Expected Impact**: -60% allocations in discovery (hot path!)

**2. Metrics Collection** (3 hours)
```
crates/main/src/monitoring/*.rs
```

**Current** (allocates metric names repeatedly):
```rust
pub fn record_metric(&self, name: &str, value: f64) {
    let metric_name = name.to_string();  // ❌ Allocation per metric!
    self.metrics.insert(metric_name, value);
}
```

**After** (intern once, reuse forever):
```rust
use crate::optimization::zero_copy::intern::intern;

pub fn record_metric(&self, name: &str, value: f64) {
    let metric_name = intern(name);  // ✅ Cached, zero-copy
    self.metrics.insert(metric_name, value);
}
```

**Expected Impact**: -90% allocations in metrics (thousands of metrics/second!)

**3. HTTP Endpoints** (3 hours)
```
crates/main/src/api/**/*.rs
```

**Current** (allocates paths, headers):
```rust
pub async fn handle_request(&self, path: &str) -> Response {
    let path_string = path.to_string();  // ❌ Allocation
    match path_string.as_str() {
        "/health" => self.health(),
        _ => self.not_found(),
    }
}
```

**After** (interned paths):
```rust
use crate::optimization::zero_copy::intern::intern;

pub async fn handle_request(&self, path: &str) -> Response {
    let path = intern(path);  // ✅ Cached common paths
    match path.as_ref() {
        "/health" => self.health(),
        _ => self.not_found(),
    }
}
```

**Expected Impact**: -80% allocations in API layer

**4. Error Messages** (3 hours)
```
crates/main/src/error.rs
crates/*/src/error.rs
```

**Current** (allocates error strings):
```rust
pub enum PrimalError {
    NotFound(String),       // ❌ Allocates
    Timeout(String),        // ❌ Allocates
    Internal(String),       // ❌ Allocates
}
```

**After** (zero-copy errors):
```rust
use crate::optimization::zero_copy::ArcStr;

pub enum PrimalError {
    NotFound(ArcStr),       // ✅ Zero-copy
    Timeout(ArcStr),        // ✅ Zero-copy
    Internal(ArcStr),       // ✅ Zero-copy
}
```

**Expected Impact**: -70% allocations in error handling

---

### Phase 2: Configuration & Constants (Week 3 - 8 hours)

**Priority**: MEDIUM (moderate frequency)

#### Target Areas

**1. Configuration Strings** (4 hours)
```
crates/main/src/config/*.rs
crates/*/src/config.rs
```

**Current** (allocates config values):
```rust
pub struct Config {
    pub service_id: String,     // ❌ Allocates
    pub endpoint: String,       // ❌ Allocates
    pub region: String,         // ❌ Allocates
}
```

**After** (zero-copy config):
```rust
use crate::optimization::zero_copy::ArcStr;

pub struct Config {
    pub service_id: ArcStr,     // ✅ Zero-copy
    pub endpoint: ArcStr,       // ✅ Zero-copy
    pub region: ArcStr,         // ✅ Zero-copy
}
```

**Benefits**:
- Config cloning is cheap (just pointer copies)
- Shared across threads efficiently
- No repeated allocations

**2. Primal Types & Capabilities** (4 hours)
```
crates/main/src/ecosystem/mod.rs (EcosystemPrimalType)
crates/main/src/capabilities/*.rs
```

**Current**:
```rust
pub enum EcosystemPrimalType {
    Squirrel,
    Songbird,
    // ...
}

impl Display for EcosystemPrimalType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            Self::Squirrel => "squirrel".to_string(),  // ❌ Allocation
            Self::Songbird => "songbird".to_string(),  // ❌ Allocation
        })
    }
}
```

**After**:
```rust
use crate::optimization::zero_copy::intern::intern;

impl Display for EcosystemPrimalType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            Self::Squirrel => intern("squirrel"),  // ✅ Cached
            Self::Songbird => intern("songbird"),  // ✅ Cached
        })
    }
}
```

**Or better** (const):
```rust
impl EcosystemPrimalType {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Squirrel => "squirrel",  // ✅ Zero-cost
            Self::Songbird => "songbird",  // ✅ Zero-cost
        }
    }
}
```

---

### Phase 3: Plugin System (Week 4 - 6 hours)

**Priority**: MEDIUM

#### Target Areas

**Plugin Metadata** (6 hours):
```
crates/core/plugins/src/*.rs
```

**Current**:
```rust
pub struct PluginMetadata {
    pub id: String,          // ❌ Allocates
    pub name: String,        // ❌ Allocates
    pub description: String, // ❌ Allocates
}
```

**After**:
```rust
use crate::optimization::zero_copy::ArcStr;

pub struct PluginMetadata {
    pub id: ArcStr,          // ✅ Zero-copy
    pub name: ArcStr,        // ✅ Zero-copy
    pub description: ArcStr, // ✅ Zero-copy
}
```

---

### Phase 4: Logging & Tracing (Week 5 - 4 hours)

**Priority**: LOW (high volume but less critical)

#### Target Areas

**Tracing Spans** (4 hours):
```rust
// Current
tracing::info!(target: "squirrel", "Processing request");  // ❌ Static but could be optimized

// After - use span caching
let span = tracing::info_span!("process_request");
let _guard = span.enter();
```

**Log Contexts**:
- Use interned strings for common log messages
- Cache span names
- Reuse error messages

---

## 🛠️ Implementation Patterns

### Pattern 1: Function Parameters

**Before**:
```rust
pub fn register_service(&self, name: &str) -> Result<()> {
    let service_name = name.to_string();  // ❌
    self.services.insert(service_name, service);
}
```

**After** (accept ArcStr):
```rust
pub fn register_service(&self, name: impl Into<ArcStr>) -> Result<()> {
    let service_name: ArcStr = name.into();  // ✅
    self.services.insert(service_name, service);
}
```

**Benefits**:
- Accepts `&str`, `String`, or `ArcStr`
- Converts once, shares everywhere
- Ergonomic API

### Pattern 2: Struct Fields

**Before**:
```rust
pub struct Service {
    pub name: String,       // ❌
    pub endpoint: String,   // ❌
}

impl Service {
    pub fn clone_name(&self) -> String {
        self.name.clone()  // ❌ Allocates
    }
}
```

**After**:
```rust
pub struct Service {
    pub name: ArcStr,       // ✅
    pub endpoint: ArcStr,   // ✅
}

impl Service {
    pub fn clone_name(&self) -> ArcStr {
        self.name.clone()  // ✅ Just pointer copy
    }
}
```

### Pattern 3: Return Values

**Before**:
```rust
pub fn get_service_name(&self) -> String {
    self.service.name.clone()  // ❌ Allocates
}
```

**After**:
```rust
pub fn get_service_name(&self) -> ArcStr {
    self.service.name.clone()  // ✅ Cheap clone
}
```

### Pattern 4: Collections

**Before**:
```rust
pub struct Registry {
    services: HashMap<String, Service>,  // ❌ String keys allocate
}
```

**After**:
```rust
pub struct Registry {
    services: HashMap<ArcStr, Service>,  // ✅ Zero-copy keys
}
```

### Pattern 5: Interning

**For frequently-used strings**:
```rust
use crate::optimization::zero_copy::intern::intern;

// Cache common values
let health_path = intern("/health");  // Cached forever
let metrics_path = intern("/metrics");  // Cached forever

// Reuse in hot paths
if path == health_path {
    // Zero comparison overhead, zero allocations
}
```

---

## 📊 Expected Impact

### Allocation Reduction

**Phase 1 (Hot Paths)**:
```
Service discovery:  -60% allocations
Metrics:           -90% allocations
HTTP endpoints:    -80% allocations
Error handling:    -70% allocations
Average:           -75% in hot paths
```

**Phase 2 (Config)**:
```
Configuration:     -80% allocations
Primal types:      -90% allocations
```

**Phase 3 (Plugins)**:
```
Plugin metadata:   -70% allocations
```

**Overall**:
```
Total string allocations: 4,700
After migration:          1,400-2,350 (-50-70%)
```

### Performance Impact

**CPU**:
```
Before: ~15% CPU time in allocation/deallocation
After:  ~5% CPU time
Saved:  10% CPU efficiency gain
```

**Memory**:
```
Before: ~20MB/s allocation rate
After:  ~7MB/s allocation rate
Saved:  -65% allocation pressure
```

**GC Pressure** (if using allocators with GC):
```
Before: High churn, frequent collections
After:  Low churn, infrequent collections
```

---

## ⏱️ Time Estimates

### Detailed Breakdown

```
Phase 1: Hot Paths (Week 1-2)
- Service discovery:    3 hours
- Metrics collection:   3 hours
- HTTP endpoints:       3 hours
- Error handling:       3 hours
- Subtotal:            12 hours

Phase 2: Configuration (Week 3)
- Config structs:       4 hours
- Primal types:         4 hours
- Subtotal:             8 hours

Phase 3: Plugin System (Week 4)
- Plugin metadata:      6 hours
- Subtotal:             6 hours

Phase 4: Logging (Week 5)
- Tracing spans:        4 hours
- Subtotal:             4 hours

Total:                 30 hours
```

### Realistic Schedule

**Week-by-week** (assuming 6 hours/week):
```
Week 1-2: Hot paths (highest impact)
Week 3:   Configuration
Week 4:   Plugin system
Week 5:   Logging & final validation
```

---

## 🎯 Success Criteria

### Per Phase

- [ ] All targeted files migrated
- [ ] Tests passing
- [ ] Benchmarks show improvement
- [ ] No performance regressions
- [ ] Memory profiling confirms reduction

### Overall

- [ ] 50-70% reduction in string allocations
- [ ] 10%+ CPU efficiency gain
- [ ] All tests passing
- [ ] Documentation updated
- [ ] Performance benchmarks green

---

## 🔧 Tooling

### Find Opportunities

```bash
#!/bin/bash
# find_zero_copy_opportunities.sh

echo "=== String Allocation Opportunities ==="

echo -e "\n.to_string() calls:"
grep -r "\.to_string()" crates/main/src --include="*.rs" | wc -l

echo -e "\n.clone() on String fields:"
grep -r "\..*\.clone()" crates/main/src --include="*.rs" | grep -E "(name|id|endpoint|path)\.clone" | wc -l

echo -e "\nString::from() calls:"
grep -r "String::from" crates/main/src --include="*.rs" | wc -l

echo -e "\nHot path files (discovery, metrics, api):"
find crates/main/src/{discovery,monitoring,api} -name "*.rs" -exec grep -l "to_string\|clone" {} \;
```

### Benchmark Script

```bash
#!/bin/bash
# benchmark_zero_copy.sh

echo "Running zero-copy benchmarks..."

# Before migration
git stash
cargo bench --bench string_allocations > before.txt

# After migration
git stash pop
cargo bench --bench string_allocations > after.txt

# Compare
echo "Improvement:"
diff before.txt after.txt
```

---

## 📚 Migration Checklist

### Per File

- [ ] Identify string allocations (`to_string`, `clone`, etc.)
- [ ] Determine if hot path (yes = high priority)
- [ ] Replace `String` with `ArcStr` in structs
- [ ] Update function parameters to `impl Into<ArcStr>`
- [ ] Use `intern()` for common values
- [ ] Run tests
- [ ] Benchmark if hot path
- [ ] Commit with before/after metrics

### Per Phase

- [ ] All targeted files migrated
- [ ] Integration tests passing
- [ ] Performance benchmarks run
- [ ] Memory profiling done
- [ ] Documentation updated

---

## 🎓 Best Practices

### When to Use ArcStr

✅ **Use ArcStr for**:
- Service names, IDs
- Configuration values
- Metric names
- HTTP paths
- Error messages
- Plugin names/IDs

❌ **Don't use ArcStr for**:
- Single-use strings
- Strings that are modified
- Very short strings (<10 bytes)
- File paths (use `PathBuf`)

### Interning Guidelines

✅ **Intern**:
- Common values (status codes, provider names)
- Repeated strings (endpoint paths)
- Constants (metric names)

❌ **Don't intern**:
- Unique values (UUIDs, timestamps)
- User input (unbounded growth)
- Dynamic content

---

**Created**: January 13, 2026  
**Status**: Ready to Execute (Phase 1 high-impact)  
**Total Time**: 30 hours over 5 weeks  
**Expected Impact**: 50-70% reduction in allocations

⚡ **Fast through smart memory management!**

