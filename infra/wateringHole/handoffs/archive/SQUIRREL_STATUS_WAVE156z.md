<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# Squirrel Status — Wave 156z (Aug 6, 2026)

## Gate Status

| Metric | Value |
|--------|-------|
| Build | GREEN — 0 errors, **0 warnings** |
| Tests | **5,753** passing / 0 failures across **13** workspace crates |
| Workspace | 13 crates (was 16 at start of session) |
| Scale | ~257k lines across ~838 `.rs` files |
| Clippy | CLEAN — pedantic + nursery + cargo, zero warnings |
| Unsafe | 0 — `unsafe_code = "forbid"` workspace-wide |
| Pure Rust | 100% default features and `--all-features` |
| `.unwrap()` | 0 in production code |
| `todo!()` / `unimplemented!()` | 0 |
| `#[async_trait]` | 0 |

## Session Summary (Waves 156y–156z)

### Wave 156y — squirrel-plugins Crate Excise
- **squirrel-plugins** removed (15,573 lines, 65 files, 0 reverse deps)
- Plugin hosting belongs to ToadStool; context plugin functionality already in squirrel-interfaces + squirrel-context
- Dead `with-plugins` feature gate removed from squirrel-context
- -14,537 net lines, 189 self-tests removed

### Wave 156z — Orphan Crate Excise + Debt Cleanup
- **squirrel-core** removed (13,907 lines, 0 reverse deps) — Songbird mesh/federation/swarm code absorbed during early development
- **squirrel-sdk** removed (11,912 lines, 0 reverse deps) — ToadStool WASM plugin authoring SDK
- 12 timeout/duration literals centralized to `universal-constants` (5 new constants)
- `DignityViolation` migrated to `#[derive(thiserror::Error)]`
- `jsonrpc_server_unit_tests.rs` (1,293L) split into 3 focused files
- Lint hygiene: missing `reason` on `#[expect]` fixed

### Cumulative (Waves 156q–156z)
- **~55,000 net lines removed** (PluginError migration, deprecated fossils, orphan crate excise)
- 3 orphan crates excised (squirrel-plugins, squirrel-core, squirrel-sdk)
- 157 functions de-asynced, 26 Pin<Box<dyn Future>> eliminated, 6 dep crates removed
- All hardcoded ports/DNS/timeouts centralized
- Workspace reduced from 16 → 13 crates

## Remaining Upstream Absorption Candidates

The following modules in `crates/main/src/` are Songbird/BearDog/ToadStool scaffolding that was absorbed into squirrel during early ecosystem development. They compile and have tests but are **not called from the production startup path** (`main.rs` → `JsonRpcServer` → `AiRouter`).

**Confirm with upstream primal teams before removal:**

| Module | Lines | Likely Owner | Evidence |
|--------|------:|--------------|----------|
| `compute_client/` + `storage_client/` + `security_client/` | ~8,832 | ToadStool/BearDog/Nestgate client SDKs | Zero `rpc/` handler references |
| `ecosystem/` | 6,497 | Songbird service mesh | `EcosystemManager` constructed then immediately discarded in main.rs |
| `biomeos_integration/` | 6,365 | Songbird/primalSpring | Only tests reference it |
| `primal_provider/` | 4,044 | Songbird provider interface | `SquirrelPrimalProvider` never instantiated in prod |
| `universal/` | 2,026 | Songbird orchestration types | Duplicate of ecosystem-api traits |
| `universal_primal_ecosystem/` | 1,893 | Songbird | Only dead ecosystem layer uses it |
| `universal_adapter_v2.rs` | 663 | Songbird | Only primal_provider tests use it |
| **Subtotal** | **~30,320** | | |

### Crate-level candidates
| Crate | Lines | Likely Owner | Notes |
|-------|------:|--------------|-------|
| `ecosystem-api` | 4,715 | Songbird shared API | Only 2 types used (CapabilityDomain, CapabilityIdentifier) — could inline |
| `universal-patterns` (partial) | ~18-22K | sourDough + Songbird | Transport/IPC (~11K) is legit; federation/registry/security/config (~18K) is Songbird |

## Archives

Excised crate archives (for upstream primal repos):
- `/tmp/squirrel-plugins-archive-156x.tar.gz` → ToadStool
- `/tmp/squirrel-core-archive-156z.tar.gz` → Songbird
- `/tmp/squirrel-sdk-archive-156z.tar.gz` → ToadStool

## Next Steps

1. **Upstream triage**: Share absorption candidates with Songbird/BearDog/ToadStool teams for confirmation
2. **Dead main modules**: Once confirmed, excise `ecosystem/`, `primal_provider/`, `universal/`, `*_client/` (~30K lines)
3. **ecosystem-api**: Inline the 2 used types, drop the crate (~4.7K lines)
4. **universal-patterns split**: Keep transport/IPC, move federation/registry/security to Songbird
5. **Continue debt sweep**: Large test files, remaining timeout literals, discovery stubs
