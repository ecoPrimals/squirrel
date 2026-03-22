# Squirrel Absorption Analysis — Handoff Files Mar 18, 2026

**Date**: 2026-03-18  
**Source**: 8 handoff files from `/home/eastgate/Development/ecoPrimals/wateringHole/handoffs/`  
**Squirrel baseline**: spring tool discovery via `mcp.tools.list`, `extract_rpc_result()`, 14-crate ecoBin ban, primal display names, capability-first sockets, proptest IPC fuzz, 22 consumed capabilities, zero-copy patterns

---

## 1. Each Spring's Key Evolution (2–3 Bullets)

### healthSpring V37
- **FMA sweep** — 8 sites converted to `mul_add()` for IEEE 754 fused multiply-add
- **Centralized RPC extraction** — `extract_rpc_result()` and `extract_rpc_result_owned()` replace 6 ad-hoc `val.get("result")` sites
- **Provenance registry** — `PROVENANCE_REGISTRY` with 49 `ProvenanceRecord` entries + completeness test walking `control/`

### wetSpring V128
- **`cast` module** — 9 safe numeric helpers (`usize_f64`, `f64_usize`, etc.) as `const fn` where possible
- **`PRIMAL_DOMAIN`** — `pub const PRIMAL_DOMAIN: &str = "science.ecology"` for biomeOS Neural API registration
- **`FAMILY_ID` socket paths** — `family_id()` helper; socket names evolved to `{primal}-{family_id}.sock`

### groundSpring V115
- **API evolution** — `assert!` → `Result<T, InputError>` for bootstrap, quasispecies, drift modules
- **`resilient_call()`** — CircuitBreaker + RetryPolicy wrapper; `BiomeOsError` query methods (`is_recoverable`, `is_retriable`, `is_method_not_found`)
- **`NESTGATE_ADDRESS` env-var** — Environment-first discovery for NestGate service

### neuralSpring V118
- **`primal_names::display`** — 12 mixed-case display-name constants; `#[allow()]` → `#[expect(reason)]` in fossils
- **Capability registry TOML** — `config/capability_registry.toml` with 16 capabilities; sync-tested against Rust constants
- **ecoBin cross-compile CI** — musl + ARM cross-compile + banned C crate detection

### toadStool S158b
- **`temp_env` migration** — `unsafe { env::set_var }` → `temp_env::with_var` (zero unsafe env in tests)
- **Zero-copy `Arc<str>`** — Build fix for `str_as_str` API replaced with `&*arc` dereference pattern
- **Scope & aims** — Four-phase silicon-targeting roadmap ("Every Piece of Silicon")

### biomeOS V251
- **`biomeos-types` modules** — `ipc.rs` (IpcErrorPhase, extract_rpc_result, extract_rpc_error), `or_exit.rs`, `cast.rs`, `validation.rs`, `mcp.rs`, `primal_capabilities.rs`
- **`capability.list` enhancements** — `cost_estimates`, `operation_dependencies`, `locality` ("local" vs "mesh"), `domains`
- **Socket registry discovery** — `DiscoveryMethod::SocketRegistry` variant; `discover_via_socket_registry()` reads `$XDG_RUNTIME_DIR/biomeos/socket-registry.json`

### primalSpring V0.30
- **5-tier discovery** — env → XDG → temp → manifest → socket-registry
- **Structured Provenance** — `Provenance { source, baseline_date, description }`; `with_provenance()` constructors
- **TOCTOU fix** — `deploy::validate_live()` `.expect()` replaced with graceful `Result` propagation

### airSpring V0.10
- **MCP tool definitions** — 10 ecology tools with JSON Schema; `list_tools()`, `tool_to_method()`, validation tests
- **Python baseline provenance registry** — `PythonBaseline` records with commit hashes, categories

---

## 2. Absorption Opportunities for Squirrel (Prioritized)

### P1 — High Impact, Not Yet in Squirrel

| Opportunity | Source | Detail |
|-------------|--------|--------|
| **`extract_rpc_result_owned()`** | healthSpring V37 | Owned variant when `extract_rpc_result()` borrows; `Option<Value>` for clone-on-need. |
| **`cast` module** | airSpring, wetSpring, biomeOS | 9 type-safe numeric cast helpers (`usize_f64`, `f64_usize`, etc.) with `CastError`; ecosystem standard. |
| **`PRIMAL_DOMAIN` constant** | wetSpring, healthSpring | `pub const PRIMAL_DOMAIN: &str = "ai"` for biomeOS Neural API registration and Songbird discovery. |
| **`locality` field in capability.list** | biomeOS V251 | Add `"local"` vs `"mesh"` per operation for PathwayLearner routing. |
| **`display::for_id()` lookup** | biomeOS | `display_name()` exists; add `for_id(id)` for reverse lookup if needed. |

### P2 — Medium Impact, Alignment

| Opportunity | Source | Detail |
|-------------|--------|--------|
| **`tool.invoke` coordination flow** | healthSpring | Squirrel has `tool.execute`; ensure healthSpring's 23 tools (PK/PD, microbiome, biosignal, etc.) are routable via discovery. |
| **Provenance registry completeness test** | healthSpring | If Squirrel has control scripts or configs, add registry + test that walks `control/` and asserts every `.py` has an entry. |
| **Capability registry TOML sync test** | neuralSpring, primalSpring | `capability_registry.toml` loaded at startup; add unit test that `CAPABILITIES` matches TOML entries. |
| **`NESTGATE_ADDRESS` env-var discovery** | groundSpring | `primal_names::address_env_var()` convention; add before host/port fallback for NestGate. |
| **`resilient_call()` wrapper** | groundSpring | Squirrel has CircuitBreaker + RetryPolicy; consider a single `resilient_call()` helper that wraps both. |
| **`BiomeOsError` query methods** | groundSpring | `is_recoverable()`, `is_retriable()`, `is_method_not_found()` — align with `IpcErrorPhase` if applicable. |

### P3 — Lower Priority, Nice-to-Have

| Opportunity | Source | Detail |
|-------------|--------|--------|
| **`mul_add()` FMA sweep** | healthSpring, neuralSpring | Audit numeric hot paths for `a * b + c` → `a.mul_add(b, c)`; no API changes. |
| **ecoBin cross-compile CI** | neuralSpring | `cargo check --target x86_64-unknown-linux-musl` + `aarch64-unknown-linux-gnu`. |
| **`deny.toml` expansion** | biomeOS | 15 vs 14 banned crates; add `libgit2-sys`, `libssh2-sys` if needed. |
| **`#[allow()]` → `#[expect(reason)]`** | neuralSpring | Where lint is known to fire; avoids unfulfilled-expectation errors. |
| **TOCTOU-safe file reparse** | primalSpring | If Squirrel validates files twice, use graceful `Result` instead of `.expect()`. |

---

## 3. Breaking Changes Affecting Squirrel

| Change | Source | Impact |
|--------|--------|--------|
| **groundSpring API evolution** | groundSpring V115 | `bootstrap_mean`, `quasispecies_simulation`, `wright_fisher_fixation`, `neutral_diversity_trajectory` now return `Result<T, InputError>`. If Squirrel calls these directly, callers must handle `Result`. |
| **Squirrel-specific** | — | None identified; handoffs are additive. |

---

## 4. Patterns Squirrel Already Has (No Action)

- `mcp.tools.list` spring tool discovery
- `extract_rpc_result()` and `extract_rpc_error()`
- 14-crate ecoBin ban in `deny.toml`
- Primal display names (`display` submodule)
- Capability-first sockets
- Proptest IPC fuzz (6 tests)
- `OrExit<T>` trait
- `FAMILY_ID` / `SQUIRREL_FAMILY_ID` socket paths
- `IpcErrorPhase` + `CircuitBreaker` + `RetryPolicy`
- `health.liveness` / `health.readiness`
- `cost_estimates` / `operation_dependencies` in capability.list
- `SocketRegistryDiscovery` (socket-registry.json)
- `capability_registry.toml`
- `universal_patterns::provenance`
- `temp_env` (for tests)

---

## 5. Summary of New Capabilities / Method Names

| Method / Capability | Source | Notes |
|--------------------|--------|--------|
| `mcp.tools.list` (healthSpring) | healthSpring V37 | 23 tools (PK/PD, microbiome, biosignal, endocrine, compute, model, health) |
| `mcp.tools.list` (airSpring) | airSpring V0.10 | 10 ecology tools |

---

## 6. Absorption-Worthy Code Patterns (Summary)

1. **`cast` module** — 9 safe numeric helpers; ecosystem standard (airSpring, wetSpring, biomeOS).
2. **`extract_rpc_result_owned()`** — Owned variant for clone-on-need paths.
3. **`PRIMAL_DOMAIN`** — Constant for biomeOS registration; Squirrel domain = `"ai"`.
4. **Provenance registry + completeness test** — If Squirrel has control scripts or configs.
5. **Capability registry TOML sync test** — Assert `CAPABILITIES` matches TOML.
6. **`locality` in capability.list** — For PathwayLearner routing.

---

*Generated from handoff analysis of 8 ecosystem springs. Squirrel v0.1.0-a12 baseline.*
