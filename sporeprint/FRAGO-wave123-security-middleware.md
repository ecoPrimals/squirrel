<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# FRAGO — Wave 123: Security Middleware + Lint Hygiene + Constraint Plumbing

**Date**: June 22, 2026
**Gate**: eastGate
**Primal**: squirrel
**Wave**: 123
**Prior**: [FRAGO-wave120-deep-debt.md](FRAGO-wave120-deep-debt.md)

---

## Executed

### SecurityOrchestrator wired to RPC hot path (P1 — M effort)
- `check_security()` now runs as pre-dispatch middleware in `handle_single_request_object`
- Method prefix → `EndpointType` tiering: `health.*` → HealthCheck, `ai.*`/`inference.*` → Compute, `btsp.*` → Authentication, `deploy.*` → Admin
- Input extraction: prompt/text/message → Text, url/endpoint → Url, path/file → FilePath
- Denied requests receive JSON-RPC error `-32003` with `risk_level` data
- Builder: `JsonRpcServer::with_security_orchestrator(Arc<SecurityOrchestrator>)`
- **Design decision**: UDS connections default to `127.0.0.1` as client IP; BTSP session ID integration deferred until cross-gate trust ships

### Constraint router plumbed to RPC (P2 → completed)
- `ai.query` now parses routing constraints from raw request JSON via `constraints::from_request()`
- Added `ConstraintSet::into_vec()` for flattening into `Vec<RoutingConstraint>`
- Clients can send: `privacy_level`, `cost_preference`, `quality`, `speed_preference`, `constraints[]`

### Dead-code attr hygiene (P2 → completed)
- 5 module-level `#![expect(dead_code)]` blanket suppressions replaced with targeted per-item attrs
- Files: `adapters/universal.rs`, `constraint_router.rs`, `universal_adapter_v2.rs`, `monitoring/types.rs`, `rate_limiter/mod.rs`
- Uses `#[cfg_attr(not(test), expect(...))]` where items are used in tests but not production paths

### Feature-flag cleanup (P3 → completed)
- Removed vestigial `capability-ai` feature (zero `cfg` references)
- Gated `benchmarking` module behind `#[cfg(feature = "benchmarking")]`
- Default features trimmed: `["ecosystem", "tarpc-rpc"]`

### Hardcoded 9200 announce port replaced
- `announce_capabilities()` now resolves via `get_service_port(PRIMAL_ID)` (env → discovery → fallback)

---

## Metrics

| Metric | Before | After |
|--------|--------|-------|
| Tests | 7,534 | 7,539 (+5 security middleware) |
| Clippy warnings | 0 | 0 |
| Unfulfilled lint expects | 0 | 0 |
| Module-level dead_code blankets | 7 | 2 (`action_registry.rs`, `benchmarking/mod.rs` — genuinely unwired) |
| Default features | 3 | 2 (dropped `capability-ai`) |
| Hardcoded ports (prod path) | 1 (9200) | 0 |

---

## Residual Debt (for upstream audit)

### P1 — Medium effort
1. **Wire `SecurityOrchestrator` at startup in `main.rs`**: Middleware plumbing complete; `main.rs` still needs `SecuritySystemBuilder::new().build().await?` + `with_security_orchestrator()` at server construction. Blocked on: config loading decision (file vs env vs defaults).
2. **UDS client identity**: Currently defaults to `127.0.0.1`. BTSP session ID or `SO_PEERCRED` would give real per-client identity for rate limiting. Blocked on: cross-gate BTSP trust (flockGate team).

### P2 — Architecture evolution
3. **Dual `UniversalAiAdapter` consolidation**: `api/ai/adapter.rs` + `discovery.rs` + `bridge.rs` overlap with `adapters/universal.rs`. Should consolidate on one adapter path.
4. **`ActionRegistry` wiring**: Complete CRUD registry exists but is orphaned. Needs integration into `AiRouter` or new `action.*` RPC surface.
5. **`universal_adapter_v2` as sole cross-primal client**: Used by `primal_provider` and `universal_provider` but not all client paths.

### P3 — Operational tuning
6. **Configurable heartbeat/RPC timeouts**: 30s heartbeat, 120s inference, 5s probe — all hardcoded. Should read from `SquirrelConfig`.
7. **`ecosystem` feature effectively no-op**: Only gates one re-export in `lib.rs`. Wire or remove.

---

## Upstream Audit Requests

1. **flockGate/bearDog**: When is cross-gate BTSP trust shipping? Squirrel's security middleware needs real client identity for rate-limit-by-session.
2. **sporeGate/cellMembrane**: Transport Envelope Phase 1 — does it affect JSON-RPC dispatch? Squirrel's pre-dispatch security check may need envelope metadata.
3. **eastGate/primalSpring**: Can primalSpring add a scenario testing `ai.query` with routing constraints (e.g. `privacy_level: "local"`) to validate constraint plumbing end-to-end?
