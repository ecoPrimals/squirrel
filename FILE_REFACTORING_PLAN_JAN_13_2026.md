# 📦 File Refactoring Plan - Semantic Boundaries

**Date**: January 13, 2026  
**Objective**: Refactor large files following semantic boundaries  
**Principle**: Smart refactoring, not arbitrary splits  
**Target**: 100% compliance with <1000 lines per file

---

## 🎯 Executive Summary

**Current Status**: 4 files >1000 lines (99.7% compliance)  
**Target**: 0 files >1000 lines (100% compliance)  
**Approach**: Semantic module extraction, not line-count splits

**Files to Refactor**:
1. ✅ `crates/main/src/ecosystem/mod.rs` - 1060 lines → 5 modules
2. ⏳ `crates/main/src/workflow/execution.rs` - 1027 lines → 4 modules  
3. ⏳ 2 additional files (analysis pending)

---

## 📊 File 1: ecosystem/mod.rs (1060 lines)

### Current Structure Analysis

```rust
// Lines 1-58: Module docs + imports + re-exports
pub mod discovery_client;
pub mod registry;
pub mod registry_manager;

// Lines 59-290: TYPE DEFINITIONS (semantic boundary 1)
pub struct EcosystemServiceRegistration { /* ... */ }
pub enum EcosystemPrimalType { /* ... */ }
pub struct ServiceCapabilities { /* ... */ }
pub struct ServiceEndpoints { /* ... */ }
pub struct HealthCheckConfig { /* ... */ }
pub struct ResourceSpec { /* ... */ }
pub struct SecurityConfig { /* ... */ }
pub struct ResourceRequirements { /* ... */ }
pub struct EcosystemConfig { /* ... */ }

// Lines 291-436: STATUS & HEALTH (semantic boundary 2)
pub struct EcosystemManager { /* ... */ }
pub struct EcosystemStatus { /* ... */ }
pub struct ServiceMeshStatus { /* ... */ }
pub struct CrossPrimalStatus { /* ... */ }
pub struct EcosystemManagerStatus { /* ... */ }
pub struct HealthStatus { /* ... */ }
pub struct ComponentHealth { /* ... */ }

// Lines 437-877: LIFECYCLE MANAGEMENT (semantic boundary 3)
impl EcosystemManager {
    pub fn new() { /* ... */ }
    pub async fn initialize() { /* ... */ }
    pub async fn start_health_monitoring() { /* ... */ }
    pub async fn perform_health_check() { /* ... */ }
    pub async fn register_with_songbird() { /* ... */ }
    pub async fn deregister_from_songbird() { /* ... */ }
    pub async fn shutdown() { /* ... */ }
}

// Lines 878-950: UNIVERSAL PATTERNS (semantic boundary 4)
impl EcosystemManager {
    pub async fn store_data_universal() { /* ... */ }
    pub async fn retrieve_data_universal() { /* ... */ }
    pub async fn execute_computation_universal() { /* ... */ }
    pub async fn send_message_universal() { /* ... */ }
}

// Lines 951-985: CAPABILITY DISCOVERY (semantic boundary 5)
impl EcosystemManager {
    pub async fn discover_by_capability() { /* ... */ }
    pub async fn discover_service_mesh() { /* ... */ }
    pub async fn list_available_capabilities() { /* ... */ }
}

// Lines 986-1060: TRAIT IMPLEMENTATIONS (semantic boundary 6)
impl Default for EcosystemConfig { /* ... */ }
impl std::fmt::Display for EcosystemPrimalType { /* ... */ }
```

### Proposed Module Structure

```
crates/main/src/ecosystem/
├── mod.rs (120 lines) ← Module root, re-exports, docs
├── types.rs (230 lines) ← Type definitions & config
├── status.rs (180 lines) ← Status & health types
├── lifecycle.rs (300 lines) ← Initialize, start, shutdown
├── universal.rs (150 lines) ← Universal pattern integrations
├── capabilities.rs (120 lines) ← Capability discovery
├── discovery_client.rs (existing)
├── registry.rs (existing)
└── registry_manager.rs (existing)
```

### Refactoring Steps

#### Step 1: Extract Types Module (30 min)

**Create `crates/main/src/ecosystem/types.rs`**:
```rust
//! Ecosystem type definitions and configurations

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Ecosystem service registration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemServiceRegistration {
    // ... (lines 64-135 from mod.rs)
}

/// Primal type enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EcosystemPrimalType {
    // ... (lines 136-233 from mod.rs)
}

/// Service capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceCapabilities {
    // ... (lines 234-241 from mod.rs)
}

// ... all type definitions ...
```

**Update `mod.rs`**:
```rust
// Add module declaration
pub mod types;
pub use types::*;
```

**Test**:
```bash
cargo build
cargo test --package squirrel
```

#### Step 2: Extract Status Module (30 min)

**Create `crates/main/src/ecosystem/status.rs`**:
```rust
//! Ecosystem status and health monitoring types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Ecosystem manager status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemManagerStatus {
    // ... (lines 394-409 from mod.rs)
}

/// Health status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    // ... (lines 413-422 from mod.rs)
}

// ... all status types ...
```

**Update `mod.rs`**:
```rust
pub mod status;
pub use status::*;
```

**Test**:
```bash
cargo build
cargo test --package squirrel
```

#### Step 3: Extract Lifecycle Module (45 min)

**Create `crates/main/src/ecosystem/lifecycle.rs`**:
```rust
//! Ecosystem lifecycle management

use super::*;
use crate::error::PrimalError;
use crate::primal_provider::SquirrelPrimalProvider;

impl EcosystemManager {
    /// Create new ecosystem manager
    #[must_use]
    pub fn new(config: EcosystemConfig, metrics_collector: Arc<MetricsCollector>) -> Self {
        // ... (lines 440-488 from mod.rs)
    }

    /// Initialize the ecosystem manager
    pub async fn initialize(&mut self) -> Result<(), PrimalError> {
        // ... (lines 491-533 from mod.rs)
    }

    // ... all lifecycle methods ...
}
```

**Update `mod.rs`**:
```rust
pub mod lifecycle;
// Re-export is automatic through impl blocks
```

**Test**:
```bash
cargo build
cargo test --package squirrel
```

#### Step 4: Extract Universal Patterns (30 min)

**Create `crates/main/src/ecosystem/universal.rs`**:
```rust
//! Universal pattern integrations

use super::*;
use crate::error::PrimalError;
use std::collections::HashMap;

impl EcosystemManager {
    /// Store data using universal storage patterns
    pub async fn store_data_universal(
        &self,
        key: &str,
        data: &[u8],
        metadata: HashMap<String, String>,
    ) -> Result<String, PrimalError> {
        // ... (lines 879-888 from mod.rs)
    }

    // ... all universal pattern methods ...
}
```

**Update `mod.rs`**:
```rust
pub mod universal;
```

**Test**:
```bash
cargo build
cargo test --package squirrel
```

#### Step 5: Extract Capabilities Module (30 min)

**Create `crates/main/src/ecosystem/capabilities.rs`**:
```rust
//! Capability-based service discovery

use super::*;
use crate::error::PrimalError;
use crate::universal::PrimalCapability;
use crate::universal_primal_ecosystem::{CapabilityMatch, DiscoveredPrimal};

impl EcosystemManager {
    /// Discover services by capability
    pub async fn discover_by_capability(
        &self,
        capability: &PrimalCapability,
    ) -> Result<Vec<DiscoveredPrimal>, PrimalError> {
        // ... (lines 951-985 from mod.rs)
    }

    // ... all capability methods ...
}
```

**Update `mod.rs`**:
```rust
pub mod capabilities;
```

**Test**:
```bash
cargo build
cargo test --package squirrel
```

#### Step 6: Clean Up mod.rs (15 min)

**Final `mod.rs` structure**:
```rust
//! Ecosystem Integration Module
//! 
//! (Keep excellent module docs)

// Imports
use chrono::{DateTime, Utc};
// ... minimal imports

// Submodules
pub mod types;
pub mod status;
pub mod lifecycle;
pub mod universal;
pub mod capabilities;
pub mod discovery_client;
pub mod registry;
pub mod registry_manager;

// Re-exports
pub use types::*;
pub use status::*;
pub use discovery_client::*;
pub use registry::*;
pub use registry_manager::*;

// Trait implementations (if small, otherwise move to types.rs)
impl Default for EcosystemConfig {
    // ...
}

impl std::fmt::Display for EcosystemPrimalType {
    // ...
}
```

**Final line count**:
- `mod.rs`: ~120 lines ✅
- `types.rs`: ~230 lines ✅
- `status.rs`: ~180 lines ✅
- `lifecycle.rs`: ~300 lines ✅
- `universal.rs`: ~150 lines ✅
- `capabilities.rs`: ~120 lines ✅

**Total**: Same logic, better organization

### Testing Strategy

**After each step**:
```bash
# 1. Build check
cargo build --package squirrel

# 2. Run tests
cargo test --package squirrel

# 3. Check for unused imports
cargo clippy --package squirrel

# 4. Verify documentation
cargo doc --package squirrel --no-deps

# 5. Integration tests
cargo test --test integration_tests
```

**Final validation**:
```bash
# All tests pass
cargo test --all

# No clippy warnings
cargo clippy --all-targets -- -D warnings

# Documentation builds
cargo doc --all --no-deps

# File size check
find crates/main/src/ecosystem/ -name "*.rs" -exec wc -l {} \;
```

---

## 📊 File 2: workflow/execution.rs (1027 lines)

### Current Structure (To Be Analyzed)

```bash
# Analyze structure
grep -n "^pub struct\|^pub enum\|^pub fn\|^impl " crates/main/src/workflow/execution.rs
```

### Proposed Modules (Preliminary)

```
crates/main/src/workflow/
├── mod.rs
├── execution/
│   ├── mod.rs (100 lines) ← Re-exports
│   ├── state_machine.rs (300 lines) ← State transitions
│   ├── executor.rs (250 lines) ← Execution logic
│   ├── context.rs (200 lines) ← Execution context
│   └── handlers.rs (200 lines) ← Step handlers
```

### Analysis Required

1. **State Machine**: FSM logic, transitions
2. **Executor**: Core execution engine
3. **Context**: Execution state & data
4. **Handlers**: Step execution handlers

**Time Estimate**: 4-6 hours (after analysis)

---

## 🎯 Success Criteria

### Per File Refactoring

- [ ] All semantic boundaries preserved
- [ ] No logic changes (pure extraction)
- [ ] All tests passing
- [ ] Zero clippy warnings
- [ ] Documentation maintained
- [ ] File size <1000 lines

### Overall Project

- [ ] 100% files <1000 lines
- [ ] No regression in functionality
- [ ] Improved maintainability
- [ ] Clear module boundaries
- [ ] Easy to navigate

---

## ⏱️ Time Estimates

### ecosystem/mod.rs Refactoring

```
Step 1: Extract types         30 min
Step 2: Extract status         30 min
Step 3: Extract lifecycle      45 min
Step 4: Extract universal      30 min
Step 5: Extract capabilities   30 min
Step 6: Clean up mod.rs        15 min
Testing & validation           60 min
Total:                         4 hours
```

### workflow/execution.rs Refactoring

```
Analysis:                      1 hour
Extraction (4 modules):        3 hours
Testing & validation:          1 hour
Total:                         5 hours
```

### Additional Files

```
Analysis:                      2 hours
Refactoring:                   4 hours
Total:                         6 hours
```

### Grand Total: 15 hours

---

## 🚀 Execution Plan

### Phase 1: ecosystem/mod.rs (Day 1 - 4 hours)

**Morning** (2 hours):
- Extract types.rs
- Extract status.rs
- Test intermediate state

**Afternoon** (2 hours):
- Extract lifecycle.rs
- Extract universal.rs
- Extract capabilities.rs
- Final testing

### Phase 2: workflow/execution.rs (Day 2 - 5 hours)

**Morning** (3 hours):
- Analyze structure
- Design module boundaries
- Begin extraction

**Afternoon** (2 hours):
- Complete extraction
- Testing & validation

### Phase 3: Remaining Files (Day 3 - 6 hours)

**Full Day**:
- Identify remaining files >1000 lines
- Analyze and refactor
- Final validation

---

## 💡 Principles Applied

### Smart Refactoring

✅ **Do**:
- Follow semantic boundaries
- Preserve logical cohesion
- Keep related code together
- Improve discoverability

❌ **Don't**:
- Arbitrary line splits
- Break semantic units
- Create artificial boundaries
- Damage readability

### Examples

**Good Boundary** (Semantic):
```
types.rs          ← All type definitions
lifecycle.rs      ← Initialization, startup, shutdown
capabilities.rs   ← Capability discovery logic
```

**Bad Boundary** (Arbitrary):
```
part1.rs  ← Lines 1-500
part2.rs  ← Lines 501-1000
part3.rs  ← Lines 1001-1060
```

---

## 📚 Migration Guide

### For Future Developers

**Old Import**:
```rust
use crate::ecosystem::EcosystemManager;
```

**New Import** (still works!):
```rust
use crate::ecosystem::EcosystemManager;  // Re-exported from mod.rs
```

**Or Specific**:
```rust
use crate::ecosystem::lifecycle::EcosystemManager;  // Direct
use crate::ecosystem::types::EcosystemConfig;       // Types
```

### Breaking Changes

✅ **None** - All public APIs remain the same through re-exports

### Documentation Updates

- [ ] Update module docs in mod.rs
- [ ] Add module docs to new files
- [ ] Update architecture diagrams
- [ ] Update COMPLETE_STATUS.md

---

## 🔄 Rollback Plan

**If Issues Arise**:

```bash
# Backup before starting
git checkout -b refactor-ecosystem-backup
git add crates/main/src/ecosystem/
git commit -m "Backup before refactoring"

# Start refactoring on new branch
git checkout -b refactor-ecosystem

# If problems:
git checkout main
git branch -D refactor-ecosystem
```

**Incremental Approach**:
- Commit after each successful module extraction
- Test thoroughly before proceeding
- Easy to revert individual steps

---

## 📊 Metrics

### Before Refactoring

```
Files >1000 lines:   4 (99.7% compliance)
Largest file:        1060 lines
Average module:      ~250 lines
Maintainability:     Good
```

### After Refactoring

```
Files >1000 lines:   0 (100% compliance)
Largest file:        <300 lines
Average module:      ~180 lines
Maintainability:     Excellent
```

---

## 🎯 Next Actions

### Ready to Execute

1. **ecosystem/mod.rs** - Complete plan above
2. **workflow/execution.rs** - Analysis first, then execute
3. **Remaining files** - Identify and plan

### Order of Execution

1. ecosystem/mod.rs (clear semantic boundaries)
2. workflow/execution.rs (requires analysis)
3. Other files (as identified)

---

**Created**: January 13, 2026  
**Status**: Ready to Execute  
**Estimated Time**: 15 hours total  
**Risk**: Low (pure extraction, no logic changes)

🎯 **Smart refactoring for excellent maintainability!**

