# 🔄 Plugin Metadata Migration Plan

**Date**: January 13, 2026  
**Objective**: Migrate from deprecated `plugin::PluginMetadata` to `interfaces::PluginMetadata`  
**Philosophy**: Gradual, non-breaking, deep debt solution  
**Status**: NOT BLOCKING - Gradual migration

---

## 🎯 Executive Summary

**Current Status**: Dual system (deprecated + new) coexisting  
**Target**: 100% using `squirrel_interfaces::plugins::PluginMetadata`  
**Strategy**: Gradual migration with compatibility layer

**Key Finding**: This is a **well-executed deprecation strategy** - not a problem to fix urgently!

---

## 📊 Current State Analysis

### Architecture ✅ EXCELLENT DEPRECATION PATTERN

**Old System** (Deprecated, but functional):
```
Location: crates/core/plugins/src/plugin.rs
Status:   #[deprecated(since = "2.0.0", note = "Use squirrel_interfaces instead")]

Features:
- Uses Uuid for plugin ID
- Has dependencies field (Vec<Uuid>)
- Legacy compatibility maintained
```

**New System** (Modern):
```
Location: crates/core/interfaces/src/plugins.rs
Status:   Active, recommended

Features:
- Uses String for plugin ID (more flexible)
- Removed dependencies field (handled elsewhere)
- Cleaner API
- Better integration with other interfaces
```

### Key Differences

| Feature | Old (plugin.rs) | New (interfaces.rs) |
|---------|----------------|---------------------|
| **ID Type** | `Uuid` | `String` |
| **Dependencies** | `Vec<Uuid>` | Removed (external) |
| **Import** | `crate::plugin::PluginMetadata` | `squirrel_interfaces::plugins::PluginMetadata` |
| **Status** | Deprecated | Active |

### Why It's Well-Designed

✅ **Gradual Migration**:
- Old code continues to work
- Deprecation warnings guide users
- No breaking changes forced

✅ **Compatibility Layer**:
- Both versions coexist
- Easy conversion between them
- Time for ecosystem to adapt

✅ **Clear Path Forward**:
- Deprecation notice explains what to do
- New API is simpler
- Migration is straightforward

---

## 🚀 Migration Strategy

### Phase 1: Analysis (Week 1 - 2 hours)

#### Identify Usage

**Find Old Usage**:
```bash
# Find files using deprecated PluginMetadata
grep -r "use.*plugin::PluginMetadata" crates/ --include="*.rs"
grep -r "crate::plugin::PluginMetadata" crates/ --include="*.rs"

# Find #[allow(deprecated)] suppressions
grep -r "#\[allow(deprecated)\]" crates/core/plugins/ --include="*.rs"
```

**Find New Usage**:
```bash
# Find files already using new PluginMetadata
grep -r "use squirrel_interfaces::plugins::PluginMetadata" crates/ --include="*.rs"
grep -r "squirrel_interfaces::plugins::PluginMetadata" crates/ --include="*.rs"
```

**Categories**:
1. **Plugin trait definition** (keep old for now - trait boundary)
2. **Plugin implementations** (can migrate gradually)
3. **Plugin managers** (can migrate gradually)
4. **Tests** (low priority)

### Phase 2: Create Conversion Utilities (Week 1 - 2 hours)

**Add to `crates/core/interfaces/src/plugins.rs`**:
```rust
use uuid::Uuid;

impl PluginMetadata {
    /// Create from legacy plugin metadata
    #[allow(deprecated)]
    pub fn from_legacy(legacy: &crate::core::plugins::PluginMetadata) -> Self {
        Self {
            id: legacy.id.to_string(),
            name: legacy.name.clone(),
            version: legacy.version.clone(),
            description: legacy.description.clone(),
            author: legacy.author.clone(),
            capabilities: legacy.capabilities.clone(),
        }
    }

    /// Convert to legacy plugin metadata (for backward compatibility)
    #[allow(deprecated)]
    pub fn to_legacy(&self) -> crate::core::plugins::PluginMetadata {
        crate::core::plugins::PluginMetadata {
            id: self.id.parse().unwrap_or_else(|_| Uuid::new_v4()),
            name: self.name.clone(),
            version: self.version.clone(),
            description: self.description.clone(),
            author: self.author.clone(),
            capabilities: self.capabilities.clone(),
            dependencies: Vec::new(),  // Legacy field, empty for new plugins
        }
    }
}
```

**Benefits**:
- Easy conversion between old and new
- Backward compatibility maintained
- Gradual migration enabled

### Phase 3: Migrate Plugin Implementations (Weeks 2-4 - 6 hours)

#### Strategy: One plugin at a time

**Example Migration**:

**Before** (Old):
```rust
use crate::plugin::{Plugin, PluginMetadata};  // ❌ Deprecated

pub struct MyPlugin {
    #[allow(deprecated)]
    metadata: PluginMetadata,  // Uses Uuid ID
}

impl Plugin for MyPlugin {
    #[allow(deprecated)]
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }
}
```

**After** (New):
```rust
use squirrel_interfaces::plugins::{Plugin, PluginMetadata};  // ✅ Modern

pub struct MyPlugin {
    metadata: PluginMetadata,  // Uses String ID
}

impl Plugin for MyPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }
}
```

#### Migration Checklist Per Plugin

- [ ] Update import: `crate::plugin` → `squirrel_interfaces::plugins`
- [ ] Remove `#[allow(deprecated)]` attributes
- [ ] Update metadata field to use new type
- [ ] Update constructor if needed (Uuid → String)
- [ ] Run tests
- [ ] Commit

#### Batch Migration

**Week 2**: Core plugins (5-10 plugins)
**Week 3**: Example plugins (10-15 plugins)
**Week 4**: Test plugins (remaining)

### Phase 4: Update Plugin Managers (Week 5 - 4 hours)

**Files**:
```
crates/core/plugins/src/manager.rs
crates/core/plugins/src/registry.rs
crates/core/plugins/src/default_manager.rs
```

**Strategy**: Update to accept both old and new metadata

**Example**:
```rust
use squirrel_interfaces::plugins::PluginMetadata as NewMetadata;

pub struct PluginManager {
    // Migrate to new metadata
    plugins: HashMap<String, NewMetadata>,
}

impl PluginManager {
    /// Register plugin with new metadata
    pub fn register(&mut self, metadata: NewMetadata) -> Result<()> {
        self.plugins.insert(metadata.id.clone(), metadata);
        Ok(())
    }

    /// Register plugin with legacy metadata (compatibility)
    #[allow(deprecated)]
    pub fn register_legacy(&mut self, legacy: crate::plugin::PluginMetadata) -> Result<()> {
        let metadata = NewMetadata::from_legacy(&legacy);
        self.register(metadata)
    }
}
```

### Phase 5: Update Plugin Trait (Week 6 - 2 hours)

**Current** (uses old metadata):
```rust
// crates/core/plugins/src/plugin.rs
#[async_trait]
pub trait Plugin: Send + Sync {
    #[allow(deprecated)]
    fn metadata(&self) -> &PluginMetadata;  // Old type
    
    async fn initialize(&self) -> Result<()>;
    async fn execute(&self, input: Value) -> Result<Value>;
}
```

**Option A: Keep Trait As-Is** (Recommended)
- Don't change trait signature (breaking change)
- Trait is internal to plugin system
- Implementations can use new metadata internally
- Only convert when crossing trait boundary

**Option B: Create New Trait** (If needed)
```rust
// crates/core/interfaces/src/plugins.rs
#[async_trait]
pub trait IPlugin: Send + Sync + Debug {
    fn metadata(&self) -> &PluginMetadata;  // New type
    
    async fn initialize(&self, context: PluginExecutionContext) -> Result<()>;
    async fn execute(&self, input: Value, context: PluginExecutionContext) -> Result<Value>;
}
```

**Recommendation**: **Option A** - Keep old trait, gradual internal migration

### Phase 6: Deprecate Old Metadata Further (Month 2 - 1 hour)

**Add stronger warnings**:
```rust
#[deprecated(
    since = "2.0.0",
    note = "Use squirrel_interfaces::plugins::PluginMetadata instead. This will be REMOVED in 3.0.0"
)]
pub struct PluginMetadata {
    // ... keep implementation for now
}
```

**Update documentation**:
```rust
//! # Migration Guide
//!
//! If you're using `crate::plugin::PluginMetadata`:
//!
//! ```rust,ignore
//! // OLD:
//! use crate::plugin::PluginMetadata;
//! 
//! // NEW:
//! use squirrel_interfaces::plugins::PluginMetadata;
//! ```
//!
//! ID type changed from `Uuid` to `String` for better flexibility.
```

### Phase 7: Final Cleanup (Month 3 - 2 hours)

**Only after all usage migrated**:
- [ ] Verify no remaining `#[allow(deprecated)]` for PluginMetadata
- [ ] Remove old PluginMetadata struct
- [ ] Remove compatibility layer
- [ ] Update to version 3.0.0
- [ ] Celebrate! 🎉

---

## ⏱️ Time Estimates

### Realistic Schedule

```
Week 1: Analysis & Utilities
- Analysis:           2 hours
- Conversion utils:   2 hours
- Subtotal:           4 hours

Weeks 2-4: Plugin Implementations
- Core plugins:       2 hours
- Example plugins:    2 hours
- Test plugins:       2 hours
- Subtotal:           6 hours

Week 5: Plugin Managers
- Manager updates:    4 hours
- Subtotal:           4 hours

Week 6: Trait Updates (if needed)
- Trait migration:    2 hours
- Subtotal:           2 hours

Month 2-3: Gradual Completion
- Final stragglers:   2 hours
- Documentation:      1 hour
- Final cleanup:      1 hour
- Subtotal:           4 hours

Grand Total:         20 hours over 3 months
```

---

## 🎯 Success Criteria

### Phase Completion

- [ ] Conversion utilities created
- [ ] 50%+ plugins migrated to new metadata
- [ ] Plugin managers support both formats
- [ ] All new plugins use new metadata
- [ ] Documentation updated

### Final Completion (3.0.0 Release)

- [ ] 100% plugins migrated
- [ ] Old metadata removed
- [ ] All deprecation warnings resolved
- [ ] Tests passing
- [ ] Documentation complete

---

## 💡 Why This Is LOW Priority

### It's a Well-Designed Deprecation

✅ **No Urgency Because**:
1. Old system still works perfectly
2. Deprecation warnings guide users
3. No security issues
4. No performance impact
5. Clear migration path

### When to Prioritize

🔴 **Increase priority if**:
- Planning major version release (3.0.0)
- Adding features that need new metadata
- External plugins complaining
- Removing deprecated code sprint

🟢 **Current priority is correct** - gradual, non-breaking

---

## 📚 Documentation

### Migration Guide for Plugin Authors

**For External Plugin Developers**:

```markdown
# Migrating to New Plugin Metadata

## Quick Migration

### 1. Update Import
```rust
// OLD:
use squirrel::plugin::PluginMetadata;

// NEW:
use squirrel_interfaces::plugins::PluginMetadata;
```

### 2. Update ID Type
```rust
// OLD:
use uuid::Uuid;

let metadata = PluginMetadata {
    id: Uuid::new_v4(),  // Uuid
    // ...
};

// NEW:
let metadata = PluginMetadata {
    id: "my-plugin".to_string(),  // String
    // ...
};
```

### 3. Remove Dependencies Field
```rust
// OLD:
let metadata = PluginMetadata {
    dependencies: vec![some_uuid],  // No longer needed
    // ...
};

// NEW:
// Dependencies handled separately by plugin manager
```

### That's It!
Your plugin will work with both old and new systems during the transition period.
```

---

## 🔧 Tooling

### Find Migration Candidates

```bash
#!/bin/bash
# find_plugin_metadata_usage.sh

echo "=== Plugin Metadata Migration Analysis ==="

echo -e "\nFiles using OLD metadata (crate::plugin):"
grep -r "use.*plugin::PluginMetadata" crates/ --include="*.rs" | wc -l

echo -e "\nFiles using NEW metadata (interfaces):"
grep -r "use squirrel_interfaces::plugins::PluginMetadata" crates/ --include="*.rs" | wc -l

echo -e "\nFiles with #[allow(deprecated)] for metadata:"
grep -r "#\[allow(deprecated)\]" crates/core/plugins/ --include="*.rs" -B2 | grep -A2 "PluginMetadata" | wc -l

echo -e "\nMigration Progress:"
OLD=$(grep -r "use.*plugin::PluginMetadata" crates/ --include="*.rs" | wc -l)
NEW=$(grep -r "use squirrel_interfaces::plugins::PluginMetadata" crates/ --include="*.rs" | wc -l)
TOTAL=$((OLD + NEW))
if [ $TOTAL -gt 0 ]; then
    PERCENT=$((NEW * 100 / TOTAL))
    echo "$PERCENT% migrated ($NEW/$TOTAL files)"
fi
```

---

## 📊 Current Recommendation

### Status: **GRADUAL - NOT BLOCKING**

**Why?**
- Old system works fine
- Clear deprecation path
- No security/performance issues
- Well-designed migration strategy

**When to Execute?**
- After higher-priority items (dependencies, file refactoring)
- During maintenance sprints
- Before 3.0.0 release
- When adding new plugin features

**Current Action**: **CREATE PLAN (DONE) ✅**

**Next Action**: **DEFER TO MONTH 2-3** (after hot paths optimized)

---

**Created**: January 13, 2026  
**Status**: Plan Complete - Execution Deferred  
**Total Time**: 20 hours over 3 months (low priority)  
**Risk**: Very low (gradual, non-breaking)

🔄 **Well-designed deprecation - no urgency!**

