<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# Squirrel — Waves 156f–156h: Deep Debt Sweep + Arc Evolution

**Date**: Aug 5, 2026
**Host**: eastGate
**Tests**: 7,241 passing / 0 failures / 0 warnings (`--all-features`)
**Covers**: Waves 156f, 156g, 156h (consolidated handoff)

## Wave 156h — Clone-to-Arc Evolution + Copy Derives

### TaskManager → `Arc<Task>` Storage
- `tasks: RwLock<HashMap<Arc<str>, Arc<Task>>>` — zero-copy reads
- All mutator methods use `Arc::make_mut` (copy-on-write) instead of clone-out/mutate/re-insert
- 12 of 16 `.clone()` calls eliminated
- `assign_task` triple-clone → single `Arc::make_mut`
- All public APIs return `Arc<Task>` (bulk lists: `Vec<Arc<Task>>`)
- `task_to_json_task` handler accepts `&Task` instead of owned `Task`

### SyncManager → `Arc<SyncEvent>` Broadcast
- Events wrapped in `Arc` once at emission, fan out via `Arc::clone`
- Eliminates per-subscriber deep clone of `ConflictInfo`, `PartitionInfo`, `ContextState`
- `Sender<SyncEvent>` → `Sender<Arc<SyncEvent>>` (2 test files updated)

### Small Enum Copy Derives
- `SyncStatus`: `#[derive(Copy)]` — zero heap data
- `ConflictResolutionStrategy`: `#[derive(Copy)]` — zero heap data
- `.clone()` calls on these types now zero-cost

### Cleanup
- `ResourceValidationRule` dead_code warning fixed (`system-metrics` feature gate)
- Orphaned `sync_manager_tests.rs` deleted (548 lines, never compiled)

## Wave 156g — RegistryType Elimination

- `RegistryType` enum (6 variants) → `registry_backend: String` on `RegistryDiscovery`
- Only `"biomeos"` variant had working implementation (socket-registry.json)
- 3 module-level `#![expect(deprecated)]` removed
- Env-var-to-enum match blocks eliminated in `runtime_engine.rs`, `self_knowledge.rs`
- `capability_resolver.rs`: `with_registry(backend: impl Into<String>)` API

## Wave 156f — Deep Scaffolding Audit + Dead Code

### Deleted
| Item | File | Reason |
|------|------|--------|
| `ClientRequestCounter` | rate_limiter/types.rs | Superseded by `RateLimitBucket` |
| `ConnectionMetadata` + field | universal_adapter_v2.rs | Zero readers |
| `ProviderScore.meets_requirements` | constraint_router.rs | Always true, never read |
| `FilePluginDiscovery::new()` | discovery.rs | Zero callers |

### Evolved (14 write-only fields → `_` prefix)
- `SecurityViolation`, `ClientInfo`, `AdaptiveRateLimitState` (rate limiter)
- `AuthAttempt`, `AccountLockout` (security hardening)
- `FederationLoadBalancer` (federation)
- `ServerHello`, `HandshakeError` (BTSP client — with `#[serde(rename)]`)
- `FrameTransport`, `FramedStream` (MCP transport)
- `CachedVisualization` (visualization)

### Gated
- `ViolationType::{SuspiciousActivity, RepeatedViolations, MaliciousRequest}` → `#[cfg(test)]`

## Remaining Debt (tracked, multi-wave)

| Item | Scope | Priority |
|------|-------|----------|
| `EcosystemPrimalType` / `PrimalType` | 42 files, ~500 refs | P1 (multi-wave) |
| `PluginMetadata` migration | Uuid→String key refactor across PluginManager/Registry/DependencyResolver | P2 |
| `PluginError` in SDK | 600+ refs → `universal_error::sdk::SDKError` | P3 |
| `AIError` in ai-tools | 59 production refs → `universal_error::tools::AIToolsError` | P3 |

## Cascade Notes

Pushed to golgiBody. Ready for upstream overwatch audit.
