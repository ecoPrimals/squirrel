# Wave 156t — Timeout Consolidation + De-Async + Smart Refactoring

**Date**: Aug 6, 2026
**Tests**: 6,302 passing / 0 failures / 0 warnings

## Changes

### 157 Functions De-Asynced
- Removed `async` from functions with no `.await`, not trait-constrained
- **session/mod.rs** (7): `create_session`, `get_session`, `get_session_metadata`, `update_session`, `terminate_session`, `cleanup_expired_sessions`, `get_active_session_count`
- **monitoring/metrics/collector.rs** (19): `register_custom_metric`, `record_metric`, `get_component_metrics`, `get_metric_info`, `list_metric_definitions`, system info helpers
- **biomeos_integration/** (90): `validate_*`, `start_*_agent`, `stop_*_agent`, context/state/MCP/intelligence handlers
- **discovery/rpc/providers** (41): registry/mdns/dnssd CRUD, RPC handlers, compute/storage/security provider scoring
- All callers updated across 35+ files; trait impls remain `async` per trait contract

### Timeout Literals Consolidated (8 sites → constants)
- `handlers_provenance.rs`: 500ms → `DEFAULT_PROBE_TIMEOUT_MS`
- `socket_registry.rs`: 30s → `DEFAULT_SOCKET_REGISTRY_CACHE_TTL` (new)
- `runtime_engine.rs`: 300s → `DEFAULT_CAPABILITY_DISCOVERY_TTL_SECS`
- `registry.rs`, `mdns.rs`, `dnssd.rs`: 5s → `DEFAULT_DISCOVERY_QUERY_TIMEOUT` (new)
- `jsonrpc_server.rs`: 30s → `DEFAULT_CONNECTION_TIMEOUT`
- `spring_tools.rs`: 60s → `DEFAULT_REQUEST_TIMEOUT`

### Capability Config Aligned
- `DiscoveryConfig::default()` now uses `niche::CAPABILITIES` (39 entries) instead of 4 stale hardcoded strings
- Drift-detection test added

### learning/engine.rs Refactored (773→504L)
- `types.rs` (171L): `LearningEngineConfig`, `RLState`, `RLAction`, `RLExperience`, `QValue`, `LearningAlgorithm`
- `neural_network.rs` (114L): `NeuralNetwork` MLP
- Public re-exports maintained via `learning/mod.rs`

### doctor.rs Refactored (765→directory module)
- `doctor/mod.rs`: types + orchestration + reporting
- `doctor/checks.rs`: 6 health check functions
- `doctor/doctor_tests.rs`: 37 tests

### 5 Stale Lint Suppressions Removed
- `squirrel-context/lib.rs`: `cast_precision_loss`, `format_push_string`, `too_many_lines`, `default_trait_access`, `significant_drop_in_scrutinee`

## Remaining Debt

| Item | Priority | Notes |
|------|----------|-------|
| `PrimalType` wire-format migration | Medium | Deprecated enum in `ecosystem-api`, used in serde structs |
| `PluginError` fossil (SDK) | Low | `core.rs` ~100 lines, kept for serde backward compat |
| `config` 0.14 crate replacement | Medium | Replace with hand-rolled loader; drops `async-trait`, `encoding_rs`, `yaml-rust2` |
| `bincode` 1.x migration | Low | RUSTSEC-2025-0141 unmaintained; tarpc transitive |
| Remaining ~unused_async (trait-constrained) | Low | 92 trait-constrained fns need trait redesign |
| 30+ files at 700-774 lines | Low | Approaching refactor threshold |
