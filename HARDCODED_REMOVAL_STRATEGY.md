# Hardcoded Reference Removal Strategy

**Date**: January 27, 2026  
**Phase**: 2 - Remove Hardcoded Primal References  
**Target**: 667 references → 0 references

---

## 🎯 Strategy Overview

### Principle
Replace ALL hardcoded primal names with capability-based discovery at runtime.

### Core Pattern
```rust
// ❌ OLD (Hardcoded):
use crate::ecosystem::types::EcosystemPrimalType;
let primal = EcosystemPrimalType::Songbird;
let endpoint = format!("http://localhost:8001/{}", primal.service_name());

// ✅ NEW (Capability Discovery):
use crate::discovery::capability_resolver::CapabilityResolver;
let resolver = CapabilityResolver::new();
let service = resolver.discover_provider(
    CapabilityRequest::new("service_mesh.coordination")
).await?;
let endpoint = service.endpoint; // Discovered!
```

---

## 📋 Capability Mapping Matrix

### Primal → Capability Translation

| Hardcoded Primal | Primary Capability | Secondary Capabilities | Environment Variable |
|------------------|-------------------|------------------------|---------------------|
| `Songbird` | `service_mesh` | `service_mesh.coordination`, `service_mesh.discovery` | `SERVICE_MESH_ENDPOINT` |
| `BearDog` | `security.auth` | `security.crypto`, `security.tls` | `SECURITY_AUTH_ENDPOINT` |
| `NestGate` | `storage.object` | `storage.file`, `storage.kv` | `STORAGE_OBJECT_ENDPOINT` |
| `ToadStool` | `compute.container` | `compute.orchestration`, `compute.batch` | `COMPUTE_CONTAINER_ENDPOINT` |
| `BiomeOS` | `platform.orchestration` | `platform.lifecycle`, `platform.config` | `PLATFORM_ORCHESTRATION_ENDPOINT` |
| `Squirrel` | `ai.orchestration` | `ai.chat`, `ai.embedding` | `AI_ORCHESTRATION_ENDPOINT` |

---

## 🔍 Reference Inventory

### Found References (101 total in ecosystem/)

#### 1. `crates/main/src/ecosystem/mod.rs` (42 refs)
**Priority**: 🔴 HIGH (Core module)

**Lines with hardcoded refs**:
- Line 157: `Songbird => "songbird"`
- Line 158: `BearDog => "beardog"`
- Line 159: `NestGate => "nestgate"`
- Line 187: `Songbird => "SONGBIRD"`
- Line 188: `BearDog => "BEARDOG"`
- Line 189: `NestGate => "NESTGATE"`
- Line 225: `"songbird" => Ok(Songbird)`
- Line 226: `"beardog" => Ok(BearDog)`
- Line 227: `"nestgate" => Ok(NestGate)`
- Line 457: `primal_type: EcosystemPrimalType::Squirrel` (self-knowledge - OK!)
- Line 524: `fn find_services_by_type(primal_type: EcosystemPrimalType)`
- Line 549: `fn start_coordination(participants: Vec<EcosystemPrimalType>)`
- Line 866: `Songbird => write!(f, "songbird")`
- Line 867: `BearDog => write!(f, "beardog")`
- Line 868: `NestGate => write!(f, "nestgate")`

**Action**: Deprecate `EcosystemPrimalType` enum entirely, replace usage with capability discovery

#### 2. `crates/main/src/ecosystem/registry/discovery.rs` (9 refs)
**Priority**: 🔴 HIGH (Discovery module)

**Lines**:
- Line 152-155: Port mapping by primal type
- Line 203-219: Capability mapping by primal type

**Action**: Replace with generic capability-based port assignment

#### 3. Test Files (50+ refs)
**Priority**: 🟡 MEDIUM (Tests can use test data)

**Files**:
- `ecosystem_manager_test.rs`
- `ecosystem_types_tests.rs`
- `registry/metrics_tests.rs`
- `registry/discovery_*_tests.rs`

**Action**: Update tests to use capability-based discovery, keep deprecated enum in test utilities

---

## 🔧 Implementation Plan

### Phase 2.1: Deprecate EcosystemPrimalType (Week 1)

**Step 1**: Mark enum as deprecated (DONE ✅)
```rust
#[deprecated(since = "0.1.0", note = "Use CapabilityRegistry")]
pub enum EcosystemPrimalType { /* ... */ }
```

**Step 2**: Create capability-based replacement traits

```rust
/// Capability-based service identification
pub trait ServiceCapability {
    fn primary_capability(&self) -> &str;
    fn secondary_capabilities(&self) -> Vec<&str>;
}
```

**Step 3**: Update `EcosystemManager` methods to use capabilities

```rust
// OLD API (deprecated):
pub async fn find_services_by_type(
    &self,
    primal_type: EcosystemPrimalType,
) -> Result<Vec<DiscoveredService>, PrimalError>

// NEW API:
pub async fn find_services_by_capability(
    &self,
    capability: &str,
) -> Result<Vec<DiscoveredService>, PrimalError>
```

### Phase 2.2: Replace Core Usage (Week 1)

**Target Files** (in priority order):

1. ✅ `crates/main/src/universal_provider.rs` (FIXED)
   - Removed `songbird_endpoint` field usage
   - Use generic `service_mesh_endpoint`

2. 🔄 `crates/main/src/ecosystem/mod.rs` (IN PROGRESS)
   - Replace `find_services_by_type()` with `find_services_by_capability()`
   - Replace `start_coordination(Vec<EcosystemPrimalType>)` with capability list
   - Update `create_service_registration()` to not use primal type

3. `crates/main/src/ecosystem/registry/discovery.rs`
   - Remove primal-type-based port mapping
   - Use capability-based port assignment from config

4. `crates/main/src/primal_provider.rs`
   - Update service registration to not include primal type
   - Use capability list instead

### Phase 2.3: Update Tests (Week 1-2)

**Strategy**: Tests can continue using deprecated enum for now, but should migrate to capabilities

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    // Helper for backward compatibility in tests
    fn primal_type_to_capability(primal_type: EcosystemPrimalType) -> &'static str {
        match primal_type {
            EcosystemPrimalType::Songbird => "service_mesh",
            EcosystemPrimalType::BearDog => "security.auth",
            EcosystemPrimalType::NestGate => "storage.object",
            // ...
        }
    }
    
    #[test]
    fn test_discovery() {
        // Use capability instead of primal type
        let capability = "service_mesh";
        // ...
    }
}
```

### Phase 2.4: Remove Enum (Week 2)

**Step 1**: Verify all production code uses capabilities
```bash
# Should return 0 results in production code
rg "EcosystemPrimalType::(Songbird|BearDog|NestGate|ToadStool)" \
    crates/main/src \
    --type rust \
    --glob '!**/*test*.rs'
```

**Step 2**: Move enum to test utilities
```rust
// crates/main/src/ecosystem/types.rs → Remove enum

// tests/test_utilities/mod.rs → Add for testing only
#[cfg(test)]
pub enum EcosystemPrimalType { /* ... */ }
```

**Step 3**: Update all remaining test usage

**Step 4**: Final cleanup - remove all references

---

## 🎨 Patterns for Each Scenario

### Pattern 1: Service Discovery

```rust
// ❌ OLD:
let services = ecosystem_manager
    .find_services_by_type(EcosystemPrimalType::Songbird)
    .await?;

// ✅ NEW:
let services = ecosystem_manager
    .find_services_by_capability("service_mesh")
    .await?;
```

### Pattern 2: Direct Connection

```rust
// ❌ OLD:
let endpoint = EcosystemPrimalType::Songbird.default_endpoint();
let client = HttpClient::connect(&endpoint)?;

// ✅ NEW:
let resolver = CapabilityResolver::new();
let service = resolver.discover_provider(
    CapabilityRequest::new("service_mesh")
).await?;
let client = HttpClient::connect(&service.endpoint)?;
```

### Pattern 3: Coordination

```rust
// ❌ OLD:
let participants = vec![
    EcosystemPrimalType::Songbird,
    EcosystemPrimalType::BearDog,
    EcosystemPrimalType::NestGate,
];
ecosystem_manager.start_coordination(participants, context).await?;

// ✅ NEW:
let required_capabilities = vec![
    "service_mesh",
    "security.auth",
    "storage.object",
];
ecosystem_manager.start_coordination_by_capabilities(
    required_capabilities,
    context
).await?;
```

### Pattern 4: Service Registration

```rust
// ❌ OLD:
let registration = EcosystemServiceRegistration {
    service_id: "squirrel-001",
    primal_type: EcosystemPrimalType::Squirrel,  // ❌ Hardcoded
    name: "Squirrel AI",
    // ...
};

// ✅ NEW:
let registration = EcosystemServiceRegistration {
    service_id: "squirrel-001",
    capabilities: vec![
        "ai.orchestration",     // ✅ What we DO
        "ai.chat",
        "ai.embedding",
    ],
    name: "Squirrel AI",
    // ...
};
```

### Pattern 5: Port Assignment

```rust
// ❌ OLD:
fn get_default_port(primal_type: EcosystemPrimalType) -> u16 {
    match primal_type {
        EcosystemPrimalType::Songbird => 8001,
        EcosystemPrimalType::BearDog => 8002,
        // ...
    }
}

// ✅ NEW:
fn get_default_port(capability: &str) -> u16 {
    match capability {
        "service_mesh" => 8001,
        "security.auth" => 8002,
        "storage.object" => 8003,
        _ => 8080, // Generic default
    }
}

// Or better - from config/environment:
fn get_port_from_capability(capability: &str) -> u16 {
    std::env::var(format!("{}_PORT", capability.to_uppercase().replace('.', "_")))
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080)
}
```

---

## 🧪 Testing Strategy

### Unit Tests
```rust
#[test]
fn test_capability_discovery() {
    let resolver = CapabilityResolver::new();
    
    // Set environment for test
    std::env::set_var("SERVICE_MESH_ENDPOINT", "http://test:8001");
    
    let service = resolver.discover_provider(
        CapabilityRequest::new("service_mesh")
    ).await.unwrap();
    
    assert_eq!(service.endpoint, "http://test:8001");
    assert!(service.capabilities.contains(&"service_mesh".to_string()));
}
```

### Integration Tests
```rust
#[tokio::test]
async fn test_cross_primal_coordination() {
    // Start mock services for capabilities
    let _mock_mesh = start_mock_service("service_mesh", 8001).await;
    let _mock_auth = start_mock_service("security.auth", 8002).await;
    
    let ecosystem = EcosystemManager::new(config, metrics);
    
    let result = ecosystem.start_coordination_by_capabilities(
        vec!["service_mesh", "security.auth"],
        context
    ).await;
    
    assert!(result.is_ok());
}
```

### E2E Tests
```bash
# Start real services
./scripts/start-test-environment.sh

# Run squirrel with discovery
RUST_LOG=debug cargo run -- server

# Verify it discovers services
curl http://localhost:8080/health | jq '.discovered_services'
```

---

## 📊 Progress Tracking

### Metrics

| Metric | Start | Week 1 | Week 2 | Target |
|--------|-------|--------|--------|--------|
| **Hardcoded Refs** | 667 | 600 (-10%) | 0 | 0 |
| **Capability APIs** | Few | Many | Universal | All |
| **Tests Passing** | ✅ | ✅ | ✅ | ✅ |
| **TRUE PRIMAL Score** | 60% | 80% | 100% | 100% |

### Verification Commands

```bash
# Count remaining hardcoded references
rg "EcosystemPrimalType::(Songbird|BearDog|NestGate|ToadStool)" \
    crates/main/src \
    --type rust \
    --glob '!**/*test*.rs' \
    | wc -l

# Count capability-based usage
rg "CapabilityResolver|CapabilityRequest|discover_provider" \
    crates/main/src \
    --type rust \
    | wc -l

# Verify tests still pass
cargo test --lib

# Check for deprecation warnings
cargo build --lib 2>&1 | grep "deprecated"
```

---

## ✅ Success Criteria

### Week 1 Complete When:
- [ ] Core `EcosystemManager` methods use capabilities
- [ ] `universal_provider.rs` uses capability discovery
- [ ] New capability-based APIs implemented
- [ ] 10% reduction in hardcoded refs (67 removed)
- [ ] All tests passing

### Week 2 Complete When:
- [ ] Zero hardcoded primal refs in production code
- [ ] `EcosystemPrimalType` moved to test utilities only
- [ ] All production code uses capability discovery
- [ ] Full test coverage of capability discovery
- [ ] Documentation updated
- [ ] TRUE PRIMAL compliance: 100%

---

## 🚀 Quick Start

### Day 1: Core Methods (2-4 hours)

```bash
# 1. Open ecosystem/mod.rs
code crates/main/src/ecosystem/mod.rs

# 2. Find usages
rg "EcosystemPrimalType" crates/main/src/ecosystem/mod.rs

# 3. Replace method signatures
# OLD: find_services_by_type(primal_type: EcosystemPrimalType)
# NEW: find_services_by_capability(capability: &str)

# 4. Run tests
cargo test --lib

# 5. Track progress
./scripts/evolution-check.sh
```

### Day 2: Registry Module (2-4 hours)

```bash
# 1. Update discovery.rs
code crates/main/src/ecosystem/registry/discovery.rs

# 2. Replace port mapping
# Remove primal-type-based, add capability-based

# 3. Run tests
cargo test --lib
```

### Day 3-5: Update Tests (4-8 hours)

```bash
# Update test files to use capabilities
for file in crates/main/src/ecosystem/**/*test*.rs; do
    echo "Updating $file..."
    # Manual updates with pattern matching
done
```

---

## 📚 References

### Key Files
- `crates/main/src/discovery/capability_resolver.rs` - Discovery engine
- `crates/main/src/discovery/types.rs` - Capability types
- `crates/main/src/ecosystem/mod.rs` - Ecosystem manager
- `crates/main/src/ecosystem/registry/discovery.rs` - Service discovery

### Documentation
- `EVOLUTION_EXECUTION_PLAN_JAN_27_2026.md` - Phase 2 details
- `PRIMAL_IPC_PROTOCOL.md` - Protocol standards
- `SEMANTIC_METHOD_NAMING_STANDARD.md` - Capability naming

---

**Status**: 🔄 **IN PROGRESS**  
**Phase**: Week 1 - Core Method Updates  
**Progress**: 1/667 (0.15%) - `songbird_endpoint` fixed  
**Next**: Update `EcosystemManager` core methods  
**Timeline**: 2 weeks to completion

