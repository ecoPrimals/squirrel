<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# Squirrel Evolution Handoff — Wave 155g

**Date**: Jul 28, 2026 | **Wave**: 155g | **From**: squirrel code team on westGate
**To**: overwatch + upstream primal teams

## Current State

| Metric | Value |
|--------|-------|
| Tests | 763 passing / 0 failures (16 crates, `--workspace --lib --tests`) |
| `#[test]` attrs | 4,946 (balance behind feature gates) |
| Clippy | 0 warnings (`pedantic + nursery + cargo`) |
| Formatting | `cargo fmt --check` clean |
| Unsafe | 0 blocks (`unsafe_code = "forbid"`) |
| Files >800L (prod) | 0 |
| `.rs` files | 986 |
| Lines | ~306k |
| Mocks in prod | 0 (all `#[cfg(test)]`) |
| Hardcoded primal names | 0 in prod (capability stems only; deprecated aliases preserved) |
| TODO/FIXME | 0 |

## Evolution Completed (Wave 155g)

### Capability-Based Naming Purification
- All `beardog`/`BearDog` references in production code evolved to `security_provider`/`SecurityProvider` capability stems.
- `mod beardog` → `mod security_provider` in `universal-patterns/security/providers/`.
- Config builder: `beardog_endpoint()` → `security_provider_endpoint()`, `beardog_auth()` → `security_provider_auth()`.
- Env var chain: `SECURITY_ENDPOINT` (primary) → `BEARDOG_ENDPOINT` (fallback with deprecation warning).
- `BEARDOG_SECURITY_SERVICE_ID` → `SECURITY_SERVICE_ID`.
- Connection handler: `"awaiting_beardog_keys"` → `"awaiting_security_keys"`.
- Doctor: `"bearDog"` → `"security provider (secrets.store)"`.
- Context state: `"nestGate IPC"` → `"storage capability provider"`.
- Deprecated aliases preserved for backward compatibility with `#[deprecated]` attrs.

### Overstep Elimination
- **Local crypto removed**: blake3 password hashing, XOR "encryption", local rand key generation all removed from production. These belong to the security capability provider.
- **Anomaly detection delegation**: Fake anomaly scores removed. `AnomalyDetection` now delegates to `defense.*` capability via IPC.
- **Security monitoring delegation**: `SecurityMonitor` local threat logic replaced with IPC delegation to `defense.*` capability.

### Adapter IPC Wiring
- `UniversalComputeAdapter`: resolves `compute.*` provider socket via `ipc_client`, forwards JSON-RPC.
- `StorageAdapter`: resolves `storage.*` provider socket, delegates operations.
- `SecurityAdapter`: resolves `security.*` provider socket, delegates operations.
- `OrchestrationAdapter`: resolves `orchestration.*` provider socket, delegates operations.
- All adapters return `OperationFailed` with clear messages when no provider is discovered (not `NotImplemented`).

### Type System Evolution
- `EcosystemPrimalType` enum: `#[deprecated]` — replaced by string-based `CapabilityIdentifier`.
- `PrimalType` enum in `ecosystem-api`: `#[deprecated]`.
- `CapabilityIdentifier` now supports arbitrary capability domain strings.

### Dependency Cleanup
- `wiremock` removed from `universal-patterns` (unused dev-dep).
- `tempfile` moved to `[dev-dependencies]` in `commands` and `ai-tools`.
- `strum` added for `Display` derives (replacing manual `impl Display`).

### Lint & Format
- 5 files re-formatted.
- Stale `#[expect]` suppressions cleaned in `context/lib.rs` and `security/monitoring/types.rs`.
- Test assertions updated: 14 tests evolved from `NotImplemented` → `OperationFailed`.

## Active IPC Integrations

| Upstream Capability | Method | Status |
|---------------------|--------|--------|
| `security.*` | `secrets.store/retrieve/list/delete` | WIRED — `SecurityProviderSecretStore` |
| `security.*` | BTSP `ClientHello` handshake | WIRED — `btsp_client.rs` |
| `network.*` | `http.request` delegation | WIRED — capability endpoint discovery |
| `compute.*` | `compute.execute` | WIRED — `UniversalComputeAdapter` IPC delegation |
| `storage.*` | `storage.*` operations | WIRED — `StorageAdapter` IPC delegation |
| `defense.*` | `defense.detect_anomaly`, `defense.classify_threat` | WIRED — `SecurityMonitor` + `AnomalyDetection` delegation |
| any | `capabilities.list` / `primal.announce` | WIRED — JSON-RPC handlers |

## Gaps for Upstream Review

1. **Adapter end-to-end validation**: All four universal adapters are wired for IPC delegation but need integration testing with live capability providers (compute, storage, security, orchestration).
2. **Defense capability provider**: `SecurityMonitor` and `AnomalyDetection` now delegate to `defense.*` — needs a primal advertising this capability.
3. **`send_to_primal` endpoint resolution**: Still depends on env vars for capability provider socket paths. Runtime registry discovery would eliminate this.
4. **Feature-gated tests**: 4,946 `#[test]` attrs total but only 763 run with default features. Feature-gated test coverage should be validated in CI with `--all-features`.
5. **Large test files**: Three test files exceed 800 lines (`jsonrpc_server_unit_tests.rs` 1,293L, `mcp/tests.rs` 915L, `adapter_integration_tests.rs` 814L) — test-only, not gated by 800L prod rule, but candidates for structural refactoring.
6. **`EcosystemPrimalType` / `PrimalType` deprecation**: Deprecated but still referenced at ~20 use sites with `#[allow(deprecated)]`. Full removal blocked on ecosystem-wide migration to `CapabilityIdentifier`.

## Docs Updated

- `README.md`: Fresh metrics, capability-based compute delegation, standalone mode description.
- `CONTEXT.md`: Fresh file/line counts, test counts.
- `CURRENT_STATUS.md`: Wave 155g header, test counts, `wiremock` removal noted.
- `CHANGELOG.md`: Wave 155g summary added to `[Unreleased]`.
