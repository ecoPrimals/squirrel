<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# Squirrel — Waves 156i–156j: Deep Debt Sweep + EcosystemPrimalType Migration

**Date**: Aug 5, 2026
**Host**: eastGate
**Tests**: 7,140 passing / 0 failures / 0 warnings (`--all-features`)
**Covers**: Waves 156i, 156j (consolidated handoff)

---

## Wave 156i — AIError Migration + PrimalType Dedup + Hardcoded Port Elimination

### AIError → AIToolsError migration (`squirrel-ai-tools`)
- `crate::error::Result<T>` and `crate::error::Error` now resolve to `universal_error::tools::AIToolsError`
- 8 construction sites updated across `mcp_adapter.rs`, `mock.rs`, `rate_limiter.rs`, `router/types.rs`
- 5 bare `?` operators in `config/core.rs` and `model_registry.rs` converted to explicit `.map_err()`
- Variant renames: `Runtime` → `Provider`, `RateLimit` → `RateLimitExceeded`

### PrimalType deduplication (`universal-patterns`)
- `config::types::PrimalType` (6 variants) removed → `pub use crate::traits::PrimalType` (8 variants)
- Zero duplicate enum definitions within the crate

### Hardcoded port elimination (`endpoint_resolver.rs`)
- Inline literals `8081`, `8082`, `9090`, `8500` → `ports::metrics()` + `get_service_port()`
- All ports now env-overridable via `universal_constants::network`

### Debris cleanup
- Dangling `PLUGIN_METADATA_MIGRATION_PLAN.md` references fixed (inline migration note)
- Dead `ADR-008` doc link removed from `ENVIRONMENT_GUIDE.md`

---

## Wave 156j — EcosystemPrimalType → String Migration + Context Quality

### Core migration (P1 — 9 fields across 5 files)
- `EcosystemServiceRegistration.primal_type` → `String`
- `DiscoveredService.primal_type` → `String` (was redundant with `primary_capability`)
- `PrimalApiRequest.from_primal` / `.to_primal` → `String` (`impl Into<String>` on `new()`)
- `PrimalStatus.primal_type` → `String`
- `EcosystemRegistryEvent` — all 4 variants with `primal_type` → `String`

### Construction sites updated
- `manager.rs`, `ecosystem_integration.rs`, `optimized_implementations.rs`, `discovery.rs` use `crate::niche::DOMAIN` / `primary_capability.to_string()`
- `metrics_tests.rs`, `types_tests.rs`, `manager_tests.rs` updated to use capability domain strings
- Removed 3 `#[allow(deprecated)]` / `#![expect(deprecated)]` module attributes

### Context quality
- `VisualizationAction` derives `Copy` (fieldless enum)
- `cache_visualization` redundant clone eliminated
- `count_enabled_components` code smell fixed (7 duplicate if-checks → arithmetic)

### Deprecated enum retained as fossil
- `EcosystemPrimalType` definition + `FromStr`/`as_str`/`capability`/`endpoint_env_prefix` kept for backward-compat deserialization
- Zero production struct consumers remain — only tests + serde roundtrip coverage

---

## Remaining high-priority debt for subsequent waves

| Item | Scope | Priority |
|------|-------|----------|
| `PluginMetadata` Uuid→String key migration | PluginManager, Registry, DependencyResolver | P2 |
| `ecosystem_api::PrimalType` deprecation sweep | 42 files, ~500 refs → `CapabilityDomain` | P1 |
| Production mock evolution | DNS-SD, mDNS, remote HTTP registry stubs | P2 |
| Clone hot spots in context layer | visualization/learning (marginal — mostly Arc already) | P3 |

---

## Upstream review gaps for primals teams

- **biomeOS**: `EcosystemPrimalType` is fully eliminated from squirrel struct fields; if biomeOS orchestrator consumes `primal_type` fields via serde, it now receives capability domain strings (`"ai"`, `"security"`) instead of enum variant names (`"Squirrel"`, `"BearDog"`). Wire format change.
- **ecosystem-api**: `ecosystem_api::PrimalType` enum still has ~500 refs across ecosystem — next migration wave target.
- **primalSpring**: No impact from 156i–j changes.
