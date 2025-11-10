# Week 1 Execution Plan: Legacy Import Cleanup
**Date**: November 10, 2025  
**Status**: IN PROGRESS  
**Branch**: cleanup-modernization-nov10

---

## 🎯 REVISED STRATEGY

### Key Discovery
After analysis, `squirrel-mcp-config` **IS** the canonical package name. The issue is NOT the package name, but:

1. **Deprecated type aliases** being used instead of new names
2. **Inconsistent import patterns** across codebase
3. **Minor cleanup opportunities** for better organization

### What We Found
- ✅ Package name is correct: `squirrel-mcp-config`
- ⚠️ Deprecated type aliases in use: `Config`, `DefaultConfigManager`
- ✅ New names available: `SquirrelUnifiedConfig`, `ConfigLoader`
- ✅ Only 13 import statements to review

---

## 📋 TASKS

### Task 1: Update Deprecated Type Aliases (2 hours)

#### Files Using Deprecated `Config`:
1. `crates/main/src/biomeos_integration/ecosystem_client.rs`
2. `crates/core/mcp/src/client/config.rs`
3. `crates/core/mcp/src/client/mod.rs`  
4. `crates/sdk/src/communication/mcp/client.rs`

**Action**: Replace `Config` with `SquirrelUnifiedConfig` where appropriate, or verify if the deprecated alias is intentional for backward compatibility.

#### File Using Deprecated `DefaultConfigManager`:
5. `crates/main/src/biomeos_integration/mod.rs`

**Action**: Replace with `ConfigLoader`

---

### Task 2: Verify Correct Usage (30 minutes)

#### Files Using Correct Patterns:
- `crates/core/mcp/src/enhanced/config_manager.rs` ✅ (uses `unified::*`)
- `crates/core/mcp/src/transport/tcp/mod.rs` ✅ (uses `unified::ConfigLoader`)
- `crates/core/mcp/src/transport/memory/mod.rs` ✅ (uses `unified::SquirrelUnifiedConfig`)
- `crates/core/mcp/src/transport/tcp/connection.rs` ✅ (uses `unified::ConfigLoader`)

**Action**: Document as examples of correct usage

---

### Task 3: Clean Up Comments (15 minutes)

Many files have:
```rust
// Removed: use squirrel_mcp_config::get_service_endpoints;
```

**Action**: These can stay (document historical changes) or be removed if cluttering code

---

### Task 4: Create ADR-008 (1 hour)

Document:
- Configuration standardization decisions
- Deprecated → New type mappings
- Migration strategy
- Backward compatibility approach

---

## 🔧 EXECUTION STEPS

### Step 1: Update deprecated Config usage

```bash
# Example for one file
# crates/main/src/biomeos_integration/ecosystem_client.rs
```

**Before**:
```rust
use squirrel_mcp_config::Config;
```

**After** (Option A - Use new name):
```rust
use squirrel_mcp_config::SquirrelUnifiedConfig;
```

**After** (Option B - Use specific imports):
```rust
use squirrel_mcp_config::unified::SquirrelUnifiedConfig;
```

---

### Step 2: Update DefaultConfigManager

**Before**:
```rust
use squirrel_mcp_config::DefaultConfigManager;
```

**After**:
```rust
use squirrel_mcp_config::ConfigLoader;
```

---

### Step 3: Test After Each Change

```bash
# After each file update
cargo check --package <affected-package>

# After all changes
cargo check --workspace
cargo test --workspace
```

---

## ⚠️ IMPORTANT CONSIDERATIONS

### Backward Compatibility
The deprecated aliases exist for a reason - gradual migration. Need to decide:

**Option A: Aggressive Update**
- Replace all deprecated usage immediately
- May break external code depending on these
- Faster, cleaner

**Option B: Conservative Approach**
- Keep deprecated aliases in place
- Only update new code to use new names
- Document migration path
- Slower, safer

**Recommendation**: **Option B - Conservative**
- The deprecation warnings serve as gentle nudges
- External code gets time to migrate
- We can clean up in future versions

---

## 📊 REVISED WEEK 1 GOALS

### Minimal Changes (Conservative) - 1-2 hours
1. ✅ Document current state
2. ✅ Verify which imports are deprecated vs. correct
3. ⚡ Update ONLY the most obvious deprecated usage (2-3 files)
4. 📝 Create ADR-008 documenting strategy
5. ✅ Commit and document

### Complete Update (Aggressive) - 3-4 hours
1. Update all 13 import locations
2. Replace deprecated aliases with new names
3. Test comprehensively
4. Create ADR-008
5. Update all documentation
6. Commit and verify

---

## 🎯 DECISION POINT

**Question for Review**: 
Should we proceed with:
- **A) Conservative** (1-2 hours, document + minimal changes)
- **B) Aggressive** (3-4 hours, update all deprecated usage)

**Current Recommendation**: **Conservative**
- Maintains backward compatibility
- Lower risk
- Still achieves documentation goal
- Deprecated items will naturally phase out

---

## 📝 NEXT STEPS (If Conservative)

1. **Create ADR-008** (30 min)
   - Document config standardization
   - Explain deprecated → new mappings
   - Define migration timeline

2. **Update 2-3 Critical Files** (30 min)
   - Pick highest-impact files
   - Update to new names
   - Test thoroughly

3. **Document Current State** (30 min)
   - Update START_HERE.md
   - Create migration guide
   - List all deprecated usage for future cleanup

4. **Commit & Push** (15 min)
   - Create clear commit message
   - Update progress log
   - Mark Week 1 complete

**Total Time**: ~2 hours  
**Risk**: Very low  
**Value**: High (documentation + foundation for future work)

---

**Status**: READY TO EXECUTE  
**Recommended Path**: Conservative approach  
**Next Action**: Create ADR-008


