# Hardcoding Audit - January 19, 2026

## Executive Summary

**Status**: ⚠️ Significant hardcoding identified requiring evolution

**Scope**: Complete codebase scan for hardcoded primal names, ports, and constants

## Key Findings

### 1. Primal Name Hardcoding
**Severity**: HIGH - Violates TRUE PRIMAL pattern

**Count**: 195 instances across 45 files

**Pattern**: Direct references to primal names instead of capability discovery

**Impact**: 
- Prevents dynamic ecosystem composition
- Violates TRUE PRIMAL self-knowledge principle
- Creates tight coupling between primals

**Example Violations**:
```rust
// ❌ WRONG: Hardcoded primal name
if primal_name == "beardog" {
    connect_to_beardog()
}

// ✅ CORRECT: Capability-based discovery
if service.capabilities.contains("crypto.signing") {
    connect_to_service(&service)
}
```

**Top Files with Hardcoding** (by count):
1. `crates/main/src/cli.rs` - 38 instances
2. `crates/main/src/ecosystem/mod.rs` - 17 instances
3. `crates/main/src/ecosystem/ecosystem_types_tests.rs` - 12 instances
4. `crates/main/src/ecosystem/types.rs` - 11 instances
5. `crates/main/src/optimization/zero_copy/arc_str.rs` - 8 instances

**Analysis by Context**:
- **Test files**: Acceptable for testing specific scenarios
- **Documentation/Examples**: Acceptable for clarity
- **Production code**: ⚠️ MUST EVOLVE to capability discovery

### 2. Port Hardcoding
**Severity**: MEDIUM - Reduces deployment flexibility

**Count**: 91 instances across 29 files

**Common Hardcoded Ports**:
- `:9200` - Elasticsearch style (likely test data)
- `:9300` - Alternative service port
- `:8080` - HTTP service port
- `:3000` - Development server port
- `:5000` - API server port

**Files with DEFAULT_*_PORT Pattern**: 12 files identified

**Key Files**:
```
crates/universal-constants/src/network.rs         ← Core constants
crates/universal-patterns/src/config/port_resolver.rs  ← Port resolution
crates/config/src/unified/network.rs              ← Network config
crates/core/mcp/src/constants.rs                  ← MCP constants
```

**Evolution Path**:
```rust
// ❌ WRONG: Hardcoded port
const SERVICE_PORT: u16 = 9200;

// ✅ CORRECT: Runtime discovery
let port = config.get_service_port("service_name")
    .or_else(|| discover_service_port(&service_id))
    .unwrap_or(DEFAULT_FALLBACK_PORT);
```

### 3. Mock/Stub Code Analysis
**Severity**: LOW - Minimal mocks found

**Count**: 1 file with mock patterns

**File**: `crates/main/src/observability/tracing_utils_tests.rs`

**Context**: Test file (appropriate use case) ✅

**Status**: No mocks in production code! ✅

### 4. Placeholder Code Analysis
**Severity**: NONE ✅

**todo!() / unimplemented!() Count**: 0 in production code

**Found Instances**: 9 total (all in documentation examples)
- `crates/main/src/universal/traits.rs` - 4 instances (doc examples)
- `crates/universal-patterns/src/security/traits.rs` - 5 instances (doc examples)

**Status**: ✅ ALL PLACEHOLDERS ARE IN DOCUMENTATION ONLY

**Analysis**: Documentation examples appropriately use placeholder macros for clarity. Production code has no placeholders!

## Detailed Breakdown

### Primal Name Hardcoding by File

#### Production Code (HIGH PRIORITY)

**`crates/main/src/cli.rs`** - 38 instances
- **Context**: CLI interface and subcommands
- **Risk**: User-facing hardcoded primal names
- **Evolution**: Support generic primal operations via capability

**`crates/main/src/ecosystem/mod.rs`** - 17 instances  
- **Context**: Ecosystem management and coordination
- **Risk**: Assumes specific primal presence
- **Evolution**: Query capabilities instead of names

**`crates/main/src/ecosystem/types.rs`** - 11 instances
- **Context**: Type definitions for ecosystem
- **Risk**: Enums/types tied to specific primals
- **Evolution**: Generic service types with capability metadata

**`crates/main/src/primal_provider/core.rs`** - 5 instances
- **Context**: Core primal provider logic
- **Risk**: Direct primal name references
- **Evolution**: Capability-based service resolution

**`crates/main/src/universal_primal_ecosystem/mod.rs`** - 4 instances
- **Context**: Universal ecosystem integration
- **Risk**: Hardcoded ecosystem assumptions
- **Evolution**: Dynamic service discovery

#### Test Code (MEDIUM PRIORITY)

**Test files** have many hardcoded names for specific test scenarios.
- **Status**: Acceptable but could use constants for DRY principle
- **Evolution**: Optional - consider test data fixtures

#### Documentation (LOW PRIORITY)

**Doc comments and examples** use specific primal names for clarity.
- **Status**: Appropriate for documentation
- **Evolution**: None required, but consider noting "Example primal names"

### Port Hardcoding by Category

#### Constants Files (MEDIUM PRIORITY)

**`crates/universal-constants/src/network.rs`**
- Contains `DEFAULT_*_PORT` constants
- **Status**: Appropriate for fallback defaults
- **Enhancement**: Ensure runtime override capability

**`crates/universal-constants/src/lib.rs`**
- Exports network constants
- **Status**: Proper abstraction
- **Evolution**: Add deprecation warnings to guide runtime discovery

#### Configuration Files (MEDIUM PRIORITY)

**`crates/config/src/unified/network.rs`**
- Network configuration with port definitions
- **Status**: Appropriate for config layer
- **Evolution**: Ensure environment variable overrides work

**`crates/tools/cli/src/mcp/config.rs`**
- MCP configuration with ports
- **Status**: Config-based (good)
- **Enhancement**: Document runtime override patterns

#### Test Files (LOW PRIORITY)

**Multiple test files** use hardcoded ports for test servers
- **Status**: Appropriate for tests
- **Enhancement**: Use random available ports to avoid conflicts

## Evolution Roadmap

### Phase 1: Audit Complete ✅
- [x] Identify all hardcoded primal names
- [x] Identify all hardcoded ports
- [x] Identify mocks in production
- [x] Identify todo!/unimplemented! in production

### Phase 2: Critical Path (Week 1)
**Target**: High-impact production code

1. **CLI Evolution** (4 hours)
   - File: `crates/main/src/cli.rs`
   - Replace hardcoded primal names with capability queries
   - Support generic primal operations

2. **Ecosystem Core Evolution** (3 hours)
   - Files: `crates/main/src/ecosystem/*.rs`
   - Replace primal name matching with capability matching
   - Implement service type abstraction

3. **Provider Core Evolution** (2 hours)
   - File: `crates/main/src/primal_provider/core.rs`
   - Replace direct name references with service discovery
   - Use capability-based resolution

### Phase 3: Enhanced Discovery (Week 2)
**Target**: Port resolution and dynamic discovery

1. **Port Resolution Enhancement** (3 hours)
   - Add deprecation warnings to port constants
   - Implement runtime port discovery
   - Support environment variable overrides

2. **Universal Ecosystem** (2 hours)
   - Complete dynamic service discovery
   - Remove hardcoded assumptions
   - Test capability-based routing

### Phase 4: Cleanup and Polish (Week 3)
**Target**: Test improvements and documentation

1. **Test Refactoring** (2 hours)
   - Extract hardcoded values to test fixtures
   - Use constants for DRY principle
   - Add capability discovery tests

2. **Documentation Update** (1 hour)
   - Add notes on example primal names
   - Document capability discovery patterns
   - Update architecture docs

## Metrics

### Before Evolution
- **Primal name instances**: 195 (45 files)
- **Port hardcoding**: 91 (29 files)
- **Production mocks**: 0 ✅
- **Production placeholders**: 0 ✅

### Target After Evolution (Week 3)
- **Primal name instances**: <20 (test/doc only)
- **Port hardcoding**: <30 (config/defaults only)
- **Production mocks**: 0 ✅ (maintained)
- **Production placeholders**: 0 ✅ (maintained)

### Success Criteria
- [ ] CLI supports generic primal operations
- [ ] Ecosystem uses capability-based matching
- [ ] All service connections via discovery
- [ ] Port resolution has runtime override
- [ ] No primal names in business logic
- [ ] Deprecation warnings guide developers

## Implementation Priority

### HIGH (Do First)
1. CLI primal name removal → capability-based CLI
2. Ecosystem core → capability matching
3. Provider core → service discovery

### MEDIUM (Do Second)  
1. Port resolution → runtime discovery
2. Universal ecosystem → dynamic composition
3. Test refactoring → fixture-based

### LOW (Do Last)
1. Documentation updates
2. Example improvements
3. Additional tests

## Notes

### TRUE PRIMAL Principle
> A primal has only self-knowledge and discovers other primals at runtime via capabilities.

**Current Violations**:
- Direct primal name references in business logic
- Hardcoded service endpoints
- Assumption of specific primal presence

**Evolution Goal**:
- Zero primal names in production logic
- All connections via capability discovery
- Dynamic ecosystem composition
- Runtime service resolution

### Concentrated Gap Strategy
> Only Songbird handles external HTTP/TLS. Other primals are Pure Rust ecoBins.

**Current Status**: ✅ Architecture aligned
- HTTP delegation to Songbird implemented
- Unix socket communication primary
- No HTTP dependencies in default build

**Validation**: No conflicts with hardcoding audit

---

**Audit Completed**: January 19, 2026  
**Next Review**: Post Phase 2 evolution  
**Estimated Evolution Time**: 17 hours (3 weeks)

