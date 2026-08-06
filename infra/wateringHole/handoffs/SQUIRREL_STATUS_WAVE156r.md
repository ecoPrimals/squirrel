# Wave 156r — PluginMetadata Migration + Deprecated Enum Deletion + Lint Cleanup

**Date**: Aug 6, 2026
**Tests**: 6,366 passing / 0 failures / 0 warnings

## Changes

### PluginMetadata Migration (28 files)
- Migrated `crates/core/plugins` from deprecated `plugin::PluginMetadata` (`id: Uuid`, `dependencies: Vec<Uuid>`) to canonical `squirrel_interfaces::plugins::PluginMetadata` (`id: String`, `dependencies: Vec<String>`)
- Deleted deprecated struct and all associated impls from `plugin.rs`
- All `HashMap<Uuid, ...>` / `DashMap<Uuid, ...>` plugin ID keys → `String` keys
- `Plugin::id()` returns `&str` (was `Uuid`)
- `PluginManagerTrait` methods: `Uuid` params → `&str`
- `PluginError::NotFound`, `AlreadyRegistered`, `DependencyCycle`: `Uuid` → `String`
- `EnhancedPluginDependency.id`: `Uuid` → `String`
- `dependency_id_from_name()` deleted (deps stored as names directly)
- Added `dependencies: Vec<String>`, `with_dependency()`, `with_name()` to canonical `PluginMetadata`
- Web API layer: `extract_plugin_id()` returns `String` (no UUID parsing)

### AIError Enum Deleted
- Removed 345-line deprecated `AIError` enum from `ai-tools/src/error.rs`
- Zero production callers — bridge + tests only
- Module now re-exports `universal_error::tools::AIToolsError` aliases

### EcosystemPrimalType De-exported
- Removed from `ecosystem/mod.rs` re-exports (was exposing deprecated type)
- Test imports updated to `ecosystem::types::EcosystemPrimalType`

### 3 Crate-level Deprecated Blankets Eliminated
- **ai-tools**: `#![expect(deprecated)]` removed (no deprecated items remain)
- **SDK**: blanket removed; targeted `#[expect(deprecated)]` on 3 fossil sites (`core.rs` module, `conversions.rs` import + impl)
- **plugins**: blanket removed after PluginMetadata migration completed

### Constants & Lint Hygiene
- `8.8.8.8` → `universal_constants::network::DEFAULT_DNS_SERVER`
- 2 `cfg_attr` `#[allow]` attributes given `reason` parameter

## Remaining Debt (Post-156r)

| Item | Priority | Notes |
|------|----------|-------|
| `PrimalType` wire-format migration | Medium | Deprecated enum in `ecosystem-api`, used in serde structs — needs wire compat window |
| `PluginError` fossil (SDK) | Low | `core.rs` ~100 lines, kept for serde backward compat |
| `EcosystemPrimalType` definition | Low | Deprecated fossil in `types.rs`, tests-only usage |
| 36 files at 700-774 lines | Low | Approaching 800-line refactor threshold |
| 12 patch-level dependency updates | Low | Minor semver bumps |
