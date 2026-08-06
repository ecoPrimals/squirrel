# Squirrel Status — Wave 157b: C8 Upstream Absorption Excision

**Date**: Aug 6, 2026 | **From**: eastGate overwatch | **HEAD**: TBD (pending commit)

## Summary

C8 — squirrel's only remaining cephalization work — is **P1+P2 COMPLETE**. ~67K lines and 216 files of Songbird/BearDog/ToadStool scaffolding excised from squirrel. Zero compilation errors, 4,090 tests passing, 0 failures.

## What Was Excised

### P1 — crates/main/src (~30,224 lines)

| Target | Lines | Notes |
|--------|------:|-------|
| `ecosystem/` | 6,497 | `EcosystemManager` immediately discarded in main.rs |
| `biomeos_integration/` | 6,365 | Only tests referenced it |
| `compute_client/` + `storage_client/` + `security_client/` | 8,169 | ToadStool/BearDog/NestGate client SDKs, zero handler refs |
| `primal_provider/` | 4,044 | Never instantiated in prod |
| `universal/` + `universal_primal_ecosystem/` + `universal_adapter_v2.rs` | 4,582 | Duplicates of ecosystem-api traits |
| `error_handling/` | 67 | Empty module, replaced by `error/` |

Also removed: `SquirrelSystem`, `initialize_squirrel_system`, `create_default_squirrel_system` from `lib.rs`. Dead `EcosystemManager` init from `main.rs`.

### P2 — ecosystem-api crate (4,722 lines)

Entire crate dropped — zero production imports after P1 excision. Was only used by the removed `ecosystem/` module.

### P2 — universal-patterns (~19,500 lines)

| Module | Lines | Action |
|--------|------:|--------|
| `federation/` | 5,993 | EXCISED — zero external consumers |
| `security/` | 7,234 | EXCISED — zero external consumers |
| `registry/` | 1,723 | EXCISED — zero external consumers |
| `config/` | 6,600 | EXCISED — `CredentialStorage` inlined into squirrel-mcp |
| `traits/` | 3,514 | EXCISED — zero external consumers |
| `builder.rs`, `circuit_breaker.rs`, `compute_dispatch.rs`, `dispatch_outcome.rs`, `streaming.rs`, `validation_harness.rs` | ~1,827 | EXCISED — orchestration scaffolding |
| `capabilities.rs` (orphan), `testing/` (orphan) | 971 | DELETED — never compiled |

**Kept**: `transport/`, `ipc_client/`, `manifest_discovery.rs`, `or_exit.rs`, `provenance.rs`

### Migration Artifacts

- `CredentialStorage` enum moved from `universal-patterns::config` → `squirrel-mcp::security::secret_store`
- `CrossPlatform::get_runtime_dir()` moved from `federation::cross_platform` → `transport::types::get_runtime_dir()`
- 13 integration test files + 1 example file deleted (3,943 lines of test code for excised modules)

## Codebase Snapshot

| Metric | Before (157a) | After (157b) | Delta |
|--------|---------------|--------------|-------|
| Crates | 13 | 12 | -1 |
| Files (.rs) | 838 | 622 | -216 |
| Lines (.rs) | ~257K | ~190K | ~-67K |
| Tests passing | 5,668 | 4,090 | -1,578 (all removed tests were for excised code) |
| Compilation | 0 errors | 0 errors | — |
| Test failures | 0 | 0 | — |

## What Remains

### P3 — Monitoring/Observability Consolidation (~4,481 lines)

Lower priority. Three overlapping modules (`monitoring/`, `observability/`, `metrics/`) could be consolidated to one. Production code is functional — this is a refactor, not an excision.

### Squirrel True Domain (190K lines)

AI coordination, tool routing, signal dispatch, G65 RPC (JSON-RPC + tarpc), agent panel, context management, provider management, security, transport.

## Archives

Excised code archived at:
- `/tmp/squirrel-c8-excision-archive.tar.gz` (P1 targets from crates/main/src)
- `/tmp/squirrel-c8-ecosystem-api-archive.tar.gz` (ecosystem-api crate)
- `/tmp/squirrel-c8-universal-patterns-archive.tar.gz` (universal-patterns excised modules)

These are available for upstream primal teams (Songbird, BearDog, ToadStool) if they need reference implementations.
