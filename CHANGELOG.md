<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# Changelog

All notable changes to Squirrel will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

Pre-alpha history is preserved as fossil record in
`ecoPrimals/archive/squirrel-pre-alpha-fossil-mar15-2026/docs/CHANGELOG.pre-alpha.md`.

## [0.1.0-alpha.10] - 2026-03-16

Deep ecosystem absorption: patterns from all springs and primals (toadStool S157b,
coralReef Iter 52, biomeOS v2.48, neuralSpring V112, groundSpring V112, loamSpine v0.9.3,
sweetGrass v0.7.19, barraCuda v0.3.5, petalTongue v1.6.6, airSpring v0.8.7,
rhizoCrypt v0.13, hotSpring v0.6.32). 4,925 tests passing.

### Added

- **`OrExit<T>`** — zero-panic binary entry point trait with structured exit codes
  and human-readable error messages — ecosystem consensus from 6+ primals
- **`DispatchOutcome<T>`** — protocol vs application error separation at RPC dispatch
  — absorbed from groundSpring V112, loamSpine v0.9.3, sweetGrass v0.7.19
- **`CircuitBreaker` + `RetryPolicy` + `ResilientCaller`** — IPC resilience with
  exponential backoff gated by `IpcErrorPhase.is_retryable()` — absorbed from
  petalTongue v1.6.6
- **`health.liveness` + `health.readiness`** — PRIMAL_IPC_PROTOCOL v3.0 health probes
  — absorbed from sweetGrass v0.7.19, petalTongue v1.6.6, coralReef Iter 52
- **4-format capability parsing** — flat, object, nested, double-nested+wrapper
  response formats — absorbed from airSpring v0.8.7
- **`PrimalManifest` discovery** — `$XDG_RUNTIME_DIR/ecoPrimals/*.json` manifest scan
  as fallback when Songbird unavailable — absorbed from rhizoCrypt v0.13
- **`extract_rpc_error()`** — structured JSON-RPC error extraction with
  `RpcError` type — absorbed from loamSpine v0.9.3, petalTongue v1.6.6
- **`ValidationHarness`** — multi-check validation runner with pass/fail/skip/warn
  reporting (sync + async) — absorbed from rhizoCrypt v0.13
- **Centralized `exit_codes`** — `universal-patterns::exit_codes` module with
  SUCCESS/ERROR/CONFIG/NETWORK/PERMISSION/RESOURCE/INTERRUPTED constants
- **Phase 2 primal names** — `primal_names::RHIZOCRYPT`, `PETALTONGUE`,
  `SWEETGRASS`, `LOAMSPINE`, `SKUNKBAT` added to complete the ecosystem catalogue
- **7 JSON-RPC wire-format proptest fuzz tests** — request validity, success
  response roundtrip, error extractability, capability parsing, reserved code ranges

### Changed

- **CLI exit codes** now re-export from `universal-patterns::exit_codes` instead
  of defining inline — single source of truth across all binary entry points

## [0.1.0-alpha.9] - 2026-03-16

Ecosystem absorption: cross-primal patterns from rhizoCrypt, sweetGrass, coralReef,
petalTongue, and wetSpring integrated. Modern idiomatic Rust evolution across IPC,
error handling, dependency management, and capability introspection.

### Added

- **`IpcErrorPhase`** — phase-tagged IPC errors (Connect, Write, Read, JsonRpcError,
  NoResult) with `is_retryable()` — absorbed from rhizoCrypt v0.13 structured error pattern
- **`StreamItem` / `StreamKind`** — NDJSON streaming types for pipeline coordination
  (data, progress, error, done, heartbeat) — absorbed from rhizoCrypt v0.13
- **`ComputeDispatchRequest` / `ComputeDispatchResponse`** — typed `compute.dispatch` client
  for ToadStool GPU routing — absorbed from coralReef v0.4.18
- **`parse_capabilities_from_response()`** — dual-format capability parsing (flat array +
  legacy methods-object) for interop with primals at different evolution stages
- **`socket_env_var()` / `address_env_var()`** — generic primal discovery helpers
  replacing hardcoded per-primal environment variable names — absorbed from sweetGrass v0.7.17
- **`from_env_reader(F)`** — DI config reader pattern for testable env-driven config
  without mutating process state — absorbed from rhizoCrypt v0.13
- **`capability.list` ecosystem fields** — flat `capabilities` array, `domains` list,
  and `locality` (local/external) for cross-primal introspection consensus
- **6 cross-primal IPC e2e tests** — health exchange, capability list format validation,
  error propagation, concurrent requests, graceful disconnect
- **27 new unit tests** across streaming, compute dispatch, capability parsing, and socket helpers

### Changed

- **tarpc 0.34 → 0.37** — aligned with rhizoCrypt ecosystem; `Context::deadline` updated
  from `SystemTime` to `Instant`
- **`#[allow(dead_code)]` → `#[expect(dead_code, reason)]`** — 52 attributes migrated to
  modern Rust `#[expect]` with descriptive reasons; unfulfilled expectations automatically cleaned
- **`deny.toml` hardened** — `yanked = "deny"` (was "warn") per ecosystem consensus
- **`IpcClientError` restructured** — all variants now carry `IpcErrorPhase` for retry-aware
  error handling; `is_retryable()` method added

### Metrics

| Metric | alpha.8 | alpha.9 |
|--------|---------|---------|
| Tests | 4,835 | 4,862 (+27) |
| tarpc | 0.34 | 0.37 |
| `#[allow(dead_code)]` in prod | 52 | 0 (all migrated to `#[expect]`) |
| deny.toml yanked | warn | deny |
| New modules | — | streaming, compute_dispatch |
| Cross-primal e2e tests | 0 | 6 |

## [0.1.0-alpha.8] - 2026-03-16

Deep debt execution: file refactoring, mock isolation, legacy alias removal,
FAMILY_ID socket compliance, clippy --all-targets, and documentation alignment.

### Added

- **`handlers_ai.rs`** — AI domain handlers extracted from `jsonrpc_handlers.rs`
- **`handlers_capability.rs`** — Capability domain handlers extracted
- **`handlers_system.rs`** — System/Discovery/Lifecycle handlers extracted
- **`biomeos_integration/types.rs`** — data types extracted from `biomeos_integration/mod.rs`
- **`sdk/core/manager.rs`** — `PluginManager`, `PluginFactory`, `register_plugin!` extracted from `plugin.rs`
- **`universal-constants::zero_copy`** and **`config_helpers`** modules exposed publicly
- **16 new tests** for handler refactoring verification

### Changed

- **Clippy `--all-targets`** — `cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))`
  applied systematically across 109 files; test code can use `unwrap()`/`expect()` while
  production code remains denied
- **File refactoring** — `jsonrpc_handlers.rs` (1094→~400), `biomeos_integration/mod.rs`
  (1101→658), `plugin.rs` (1012→838) — all now under 1000 lines
- **Legacy aliases removed** — flat names (`query_ai`, `health`, `ping`, etc.) no longer
  dispatched; only semantic `{domain}.{verb}` method names accepted
- **Mock isolation** — `MockServiceMeshClient` changed from `cfg(any(test, feature = "testing"))`
  to strict `#[cfg(test)]`; MCP `mock` module gated behind `#[cfg(test)]`
- **FAMILY_ID socket compliance** — `get_socket_path` and `get_xdg_socket_path` now include
  `${FAMILY_ID}` suffix per `PRIMAL_IPC_PROTOCOL.md`
- **`capability.discover`** method name — `probe_socket` now sends semantic name instead of
  legacy `discover_capabilities`
- **`unified_manager.rs`** docs updated to Phase 2 placeholder language

### Removed

- **Legacy JSON-RPC aliases** — dispatch arms for `query_ai`, `list_providers`, `announce_capabilities`,
  `discover_capabilities`, `health`, `metrics`, `ping`, `discover_peers`, `list_tools`,
  `execute_tool`
- **Stale planning docs** — 11 analysis/strategy/migration markdown files archived

### Metrics

| Metric | alpha.7 | alpha.8 |
|--------|---------|---------|
| Tests | 4,819 | 4,835 (+16) |
| Coverage | 69% | 69% |
| Clippy (`--all-targets`) | FAIL (test unwrap) | PASS (0 errors) |
| `cargo doc` warnings | 0 | 0 |
| Files >1000 lines | 0 | 0 (max: 996) |
| Mocks in production | ~2 | 0 |
| Legacy aliases | Active | Removed |

## [0.1.0-alpha.7] - 2026-03-16

Comprehensive audit execution: ecoBin compliance, clippy zero-error, typed errors,
structured logging, zero-copy evolution, test expansion, and documentation alignment.

### Added

- **`universal-constants::identity`** — centralized `PRIMAL_ID`, `JWT_ISSUER`,
  `JWT_AUDIENCE`, `JWT_SIGNING_KEY_ID` constants. Auth crates import from here
  instead of hardcoding strings.
- **`CommandError` (thiserror)** — typed error enum replacing `Box<dyn Error>` in
  `squirrel-commands` (~80 instances). Variants: Io, Serialization, Validation,
  Hook, Lifecycle, ResourceNotFound, Allocation, Lock.
- **`FormatterError` (thiserror)** — typed error for CLI formatter.
- **152 new tests** — MCP error handling, transport framing, plugin state,
  performance optimizer, visualization system, SDK types, config validation,
  environment detection.
- **`enhanced/platform_types.rs`** — extracted from `enhanced/mod.rs` (992→701 lines).
- **`benchmarking/runners.rs`** — extracted from `benchmarking/mod.rs` (988→477 lines).

### Changed

- **ecoBin compliance** — removed `openssl-sys`, `native-tls`, `anthropic-sdk` from
  all feature paths. Gated `sysinfo` behind `system-metrics` feature. Default build
  has zero chimeric C dependencies.
- **Structured logging** — ~50 `println!/eprintln!` calls in production evolved to
  `tracing::{info,warn,error,debug}`. `println!` reserved for CLI and startup banner.
- **Zero-copy patterns** — `Arc<str>` for primal identifiers and capabilities in
  `jsonrpc_handlers.rs` and `self_knowledge.rs`. `bytes::Bytes` for frame payloads.
  `Arc<dyn ValidationRule>` replacing `Box::new(self.clone())` (11 sites).
- **Clippy zero-error** — all lib targets pass `cargo clippy --all-features --lib
  -- -D warnings` with pedantic + nursery. Hundreds of lint fixes applied.
- **Unsafe elimination** — all `unsafe { env::set_var }` calls in 4 test files
  migrated to `temp_env`. Added `temp-env` to MCP crate dev-deps.
- **`--all-features` build** — fixed 12 compile errors in `ai-tools/clients` module,
  cleaned MCP `build.rs`, fixed doc-markdown lints in `universal-constants`.
- **Stubs documented** — `unified_manager.rs` STUB comments replaced with proper docs.
  Mocks verified behind `#[cfg(test)]`.

### Removed

- **TODO comment** in MCP Cargo.toml (wateringHole violation: no TODOs in committed code)
- **Stale `anthropic-sdk` dep** from `ai-tools` (pulled `native-tls`/`openssl`)
- **Stale `openai-api-rs` dep** from MCP crate (pulled `reqwest` 0.11)
- **`CODEBASE_STRUCTURE.md`** — obsolete spec (described layout from September 2024)
- **`LEGACY_PROVIDERS_DEPRECATED.md`** — superseded by capability-ai migration
- **`README_MOVED.md`** — stale redirect doc in model_splitting/

### Metrics

| Metric | alpha.6 | alpha.7 |
|--------|---------|---------|
| Tests | 4,667 | 4,819 (+152) |
| Coverage | 67% | 69% |
| Clippy (lib) | FAIL (350+ errors) | PASS (0 errors) |
| `cargo fmt` | FAIL (10+ files) | PASS |
| `--all-features` build | FAIL (125+ errors) | PASS |
| C deps (default) | 0 (claimed) | 0 (verified) |
| `Box<dyn Error>` in libs | ~80 | 0 (commands, cli) |
| `println!` in production | ~50 | 0 |
| `unsafe` in tests | 4 files | 0 |
| Files >1000 lines | 0 | 0 (two refactored) |
| Hardcoded JWT strings | 8 | 0 (centralized) |

## [0.1.0-alpha.6] - 2026-03-16

Test coverage expansion, reqwest 0.12 migration, disabled test re-enablement.

### Added

- **Auth crate tests** — 51 new tests for `errors.rs` (19), `types.rs` (21),
  `session.rs` (6), `lib.rs` (5). Covers all error variants, From impls, serde
  round-trips, session lifecycle, and env-based initialization.
- **Plugins crate tests** — 31 new tests for `manager.rs` (9), `types.rs` (7),
  `discovery.rs` (6), `default_manager.rs` (9). Covers plugin registration,
  status transitions, manifest deserialization, serde round-trips, and discovery.
- **Config crate tests** — 10 new tests for `merge_config` (4), `health_check` (5),
  `ConfigLoader::load()` integration (1). Full pipeline test with temp file + env.
- **Re-enabled tests** — 16 tests re-enabled: 14 MCP propagation tests (removed
  `disabled_until_rewrite` feature gate, fixed API mismatches), rate limiter test
  (fixed nested runtime), resource manager test (updated for current API).

### Changed

- **reqwest 0.11 → 0.12** — All 9 remaining crates migrated. Now using rustls 0.23
  with pluggable crypto providers. No API changes needed — existing usage compatible.
- **universal_adapter_tests** — 12 tests fixed from `block_on` inside tokio runtime
  to `#[test] fn` with explicit `Runtime::new()` inside `temp_env` closures.
- **Chaos test clarity** — `chaos_09` and `chaos_10` ignore reasons documented.

### Removed

- **Orphaned test files** — 7 dead test files removed from config crate (referenced
  removed `core` module, deprecated `environment_config`, unwired test modules).
- **`test_primal_analyze_e2e_mock`** — deleted (HTTP handlers removed, test was no-op).

### Metrics

| Metric | Before | After |
|--------|--------|-------|
| Tests | 4,600+ | 4,667 passing |
| Auth tests | 19 | 70 |
| Plugins tests | 22 | 53 |
| Config tests | 102 | 112 |
| reqwest version | 0.11 (9 crates) / 0.12 (1 crate) | 0.12 (all 10 crates) |
| Re-enabled tests | — | 16 |
| Orphaned files | 7 | 0 |

## [0.1.0-alpha.5] - 2026-03-16

Deep debt resolution: modern idiomatic Rust, production mock cleanup,
capability-based discovery, JSON-RPC batch support, handler refactoring.

### Added

- **`primal_names.rs`** — centralized primal name constants for socket discovery
  (groundSpring V106 / wetSpring V119 pattern). All socket path construction
  now uses typed constants instead of raw strings.
- **`capability.list` handler** — per-method cost/dependency info for biomeOS
  PathwayLearner scheduling (LoamSpine v0.8.8 / sweetGrass v0.7.12 pattern).
- **JSON-RPC 2.0 batch support** — full Section 6 compliance. Array of requests
  → array of responses. Notification-only batches return no response per spec.
- **Context in-memory persistence** — `ContextManager` evolved from stubs to real
  `DashMap`-backed storage with create/read/update/delete/list operations.
- **Batch handler tests** — 3 new tests for empty, single, and multi-request batches.
- **`capability.list` test** — verifies per-method cost/deps structure.

### Changed

- **Handler refactoring** — `jsonrpc_handlers.rs` (1019 lines) split into 3 domain
  files: `jsonrpc_handlers.rs` (utility + AI + capability + system + discovery +
  lifecycle), `handlers_context.rs` (context domain), `handlers_tool.rs` (tool domain).
  Main file now ~550 lines.
- **Production mock cleanup** — `MCPAdapter` mock fields gated behind `#[cfg(test)]`.
  `stream_request` evolved from fake-data return to honest error signaling.
- **`#[allow]` → `#[expect]` migration** — ~44 item-level `#[allow(dead_code)]`
  migrated to `#[expect(dead_code, reason = "...")]` across 7 crates.
- **Unsafe test evolution** — `unsafe { env::set_var }` replaced with `temp_env`
  in 5 test files. Tests restructured to avoid `block_on` inside tokio runtime.
- **Hardcoded socket paths** — security, lifecycle, songbird, discovery, and AI
  router now use `primal_names::*` constants for socket directory/name construction.
- **AI router** — ToadStool scanning evolved from primal-name-specific to
  capability-based discovery hints.

### Fixed

- `capability_discovery_error_tests` — fixed `block_on` inside tokio runtime
  by restructuring to sync tests with explicit `Runtime::new()`.
- Two unfulfilled `#[expect]` warnings resolved.

### Metrics

| Metric | Before | After |
|--------|--------|-------|
| Tests | 4,552 | 4,600+ (new batch + capability.list + primal_names) |
| JSON-RPC methods | 20 | 21 (+capability.list) |
| Files >1000 lines | 0 (maintained) | 0 (handlers split to 3 files) |
| Production mocks | ~5 | 0 (gated or evolved) |
| Unsafe in tests | ~30 | 0 (migrated to temp_env) |
| #[allow] without reason | ~50 | ~6 (remaining are clippy/deprecated) |
| Hardcoded primal names | ~25 | ~5 (legacy display mappings only) |

## [0.1.0-alpha.4] - 2026-03-16

Spring absorption: niche self-knowledge, Songbird announcement, proptest,
deployment graph types, SocketConfig DI, deny(unwrap/expect).

### Added

- **`niche.rs`** — structured self-knowledge module (groundSpring/wetSpring/airSpring pattern):
  `CAPABILITIES`, `CONSUMED_CAPABILITIES`, `COST_ESTIMATES`, `DEPENDENCIES`,
  `SEMANTIC_MAPPINGS`, `FEATURE_GATES`, plus JSON functions `operation_dependencies()`,
  `cost_estimates_json()`, `semantic_mappings_json()` — 8 invariant tests
- **Songbird announcement** — `capabilities/songbird.rs` implements `discovery.register` +
  `discovery.heartbeat` loop (wetSpring pattern); wired into main server startup
- **`orchestration/` module** — `DeploymentGraphDef`, `GraphNode`, `TickConfig` types
  wire-compatible with ludoSpring exp054 and biomeOS TOML; includes topological sort,
  cycle detection, `requires_squirrel()` — 7 tests
- **`SocketConfig` DI pattern** — injectable config struct for socket path resolution
  (airSpring pattern); `_with` variants avoid `temp_env`/`#[serial]` — 8 tests
- **`proptest` round-trip tests** — `tests/proptest_roundtrip.rs` with 10 property tests
  covering all JSON-RPC types and niche JSON serialization
- `PartialEq` derive on all JSON-RPC request/response types

### Changed

- **`deny(clippy::expect_used, clippy::unwrap_used)`** in `[workspace.lints.clippy]`
- All 22 crates now inherit `[lints] workspace = true`
- `capability.discover` response now includes `cost_estimates`, `operation_dependencies`,
  and `consumed_capabilities` from `niche.rs`
- `send_jsonrpc` in lifecycle module made `pub(crate)` for reuse by songbird module

### Fixed

- Pre-existing doctest in `squirrel-mcp-auth` (`DelegatedJwtClient::new` signature change)
- Pre-existing doctest in `universal-error` (`Arc<str>` conversion)
- Removed conflicting `[lints.clippy]` sections from 4 crates (config, plugins, core, mcp)

### Metrics

| Metric | Before | After |
|--------|--------|-------|
| Tests | 4,465 | 4,552 (+87) |
| Niche self-knowledge | None | `niche.rs` with 20 capabilities, 14 consumed |
| Songbird registration | Not implemented | `discovery.register` + heartbeat |
| Property tests | 0 | 10 (proptest round-trip) |
| Deployment graph types | None | `DeploymentGraphDef` + topo sort |
| SocketConfig DI tests | 0 | 8 (no temp_env needed) |
| Workspace lint inheritance | 11/22 crates | 22/22 crates |
| deny(unwrap/expect) | Not enforced | Enforced workspace-wide |

## [0.1.0-alpha.3] - 2026-03-16

Deep debt evolution, modern idiomatic Rust, and ecosystem standards alignment.

### Changed

- **`#![forbid(unsafe_code)]` unconditional** — removed `cfg_attr(not(test), ...)` from all 22 crates; all `unsafe { env::set_var }` in tests replaced with `temp_env` crate
- **tarpc service deepened** — 18 typed methods mirroring all JSON-RPC handlers; `TarpcRpcServer` delegates to `JsonRpcServer`; protocol negotiation per-connection
- **Production mocks evolved** — `ecosystem.rs` now uses capability discovery, `federation.rs` uses config-driven defaults, `registry.rs` loads from embedded `capability_registry.toml`
- **Constants centralized** — `DEFAULT_JSON_RPC_PORT`, `DEFAULT_BIOMEOS_PORT`, `MAX_TRANSPORT_FRAME_SIZE`, plugin limits, context TTL moved to `universal-constants`
- **Zero-copy expanded** — `UniversalError` stores `Arc<str>` instead of `String`; `#[must_use]`, `#[non_exhaustive]`, `#[inline]` on key types
- **Crypto migration documented** — `docs/CRYPTO_MIGRATION.md`; `ecosystem-api` upgraded to reqwest 0.12 as proof of concept
- **Clippy pedantic + nursery** — enabled via `[workspace.lints.clippy]` in workspace `Cargo.toml`

### Added

- `.rustfmt.toml` — edition 2024, max_width 100
- `clippy.toml` — cognitive complexity, function length, argument count thresholds
- `deny.toml` — cargo-deny license allowlist, advisory audit, ban wildcards
- `docs/CRYPTO_MIGRATION.md` — reqwest 0.11→0.12, ring→rustls-rustcrypto path
- `nvml-wrapper` optional dep for GPU detection (behind `nvml` feature)
- `temp-env` dev-dep across 7 crates for safe env var testing

### Fixed

- All compilation errors under `--all-features` (ecosystem-api `Arc<str>`, squirrel-plugins `reqwest`, squirrel-core `f64: Eq`, squirrel-sdk `NetworkConfig`, squirrel-ai-tools missing modules, squirrel `nvml-wrapper`)
- License: `AGPL-3.0-or-later` → `AGPL-3.0-only` in `LICENSE` file SPDX header and body
- Flaky tests: `test_graceful_degradation` tolerance, `test_fallback_chain` env isolation, all `temp_env` + `#[tokio::test]` nested-runtime conflicts
- Doctest failure in `squirrel-mcp-auth` (feature-gated `AuthService`)
- `manifest.rs` (1070→578+303+223), `orchestrator.rs` (1014→778+269), `jsonrpc_handlers.rs` (1002→997) — all files now under 1,000 lines

### Removed

- Orphaned modules: `infrastructure/`, `core/`, `client/`, `communication/` stubs in main crate
- Duplicate `specs/current/CURRENT_STATUS.md`
- Orphaned root `examples/` (9 files — relocated to archive)
- Stale `crates/config/production.toml`

### Metrics

| Metric | Before | After |
|--------|--------|-------|
| Tests | 3,749+ (some failing) | 4,465 (0 failures) |
| `#![forbid(unsafe_code)]` | Conditional (test-exempt) | Unconditional |
| Files >1000 lines | 2 | 0 |
| Production mocks | 3 files | 0 |
| Hardcoded ports/IPs | 7+ sites | Centralized in universal-constants |
| tarpc methods | Minimal | 18 (matching all JSON-RPC) |
| Tooling configs | 0 | 3 (.rustfmt.toml, clippy.toml, deny.toml) |
| Workspace lint level | warn only | pedantic + nursery |

## [0.1.0-alpha.2] - 2026-03-15

Comprehensive audit and standards alignment session. All wateringHole quality
gates now pass.

### Changed

- **Edition 2024**: All 22 crates upgraded from edition 2021 to 2024
  - Fixed `gen` reserved keyword (7 files)
  - Wrapped `std::env::set_var`/`remove_var` in `unsafe {}` for test code
  - `#![forbid(unsafe_code)]` → `#![cfg_attr(not(test), forbid(unsafe_code))]`
  - Collapsed nested `if` statements using let-chains (~50+ instances)
- **License**: `AGPL-3.0-or-later` → `AGPL-3.0-only` in all 23 Cargo.toml and 1,280 SPDX headers
- **Documentation**: Added `#![warn(missing_docs)]` to all 22 library crates; ~1,600 doc comments added
- **Clippy**: All code quality lints resolved — workspace passes `clippy -- -D warnings` clean

### Fixed

- 8 formatting violations (`cargo fmt`)
- 3 doc warnings (HTML tags in doc comments, unresolved links)
- 5 TODO/FIXME comments removed from committed code
- 8 failing plugin loading tests (implemented `load_plugins_from_directory`)
- 5 broken doctests (wrong crate name, uncompilable examples)
- Journal clone bug in `squirrel-commands` (invalid JSON in `SerializationError` clone)
- Redundant closure in capability registry
- Dead code in websocket transport (removed unused stubs)

### Removed

- Stale `run_examples.sh` script from adapter-pattern-examples

### Metrics

| Metric | Before | After |
|--------|--------|-------|
| Edition | 2021 | 2024 |
| License | AGPL-3.0-or-later | AGPL-3.0-only |
| `cargo fmt` | FAIL (8 files) | PASS |
| `cargo clippy -D warnings` | FAIL (couldn't run) | PASS (0 errors) |
| `cargo doc` warnings | 3 | 0 |
| Missing docs enforcement | 5/22 crates | 22/22 crates |
| Tests | 8 failing | All pass |
| Coverage (llvm-cov) | Unmeasured | ~66% |
| TODO/FIXME in code | 5 | 0 |

## [0.1.0-alpha] - 2026-03-15

First public alpha release. Squirrel is the AI Coordination Primal of the
ecoPrimals ecosystem — a sovereign MCP service for routing AI requests,
managing context, and coordinating multiple model providers.

### Highlights

- **3,749+ tests** passing across 22 crates, 0 failures
- **Zero C dependencies** in default build (pure Rust)
- **Zero unsafe code** (`#![forbid(unsafe_code)]` on all crates)
- **scyBorg license** — AGPL-3.0-only + CC-BY-SA 4.0
- **Capability registry** — `capability_registry.toml` as single source of truth
- **biomeOS lifecycle** — `lifecycle.register` + 30s heartbeat + SIGTERM cleanup
- **Context RPC methods** — `context.create`, `context.update`, `context.summarize`

### Architecture

- TRUE PRIMAL design: self-knowledge only, runtime capability discovery
- JSON-RPC 2.0 over Unix sockets (default IPC)
- tarpc binary protocol with automatic negotiation
- Transport hierarchy: Unix sockets → named pipes → TCP
- HTTP/WebSocket feature-gated OFF by default
- Vendor-agnostic AI: OpenAI, Anthropic, Gemini, local models (Ollama, llama.cpp, vLLM)
- Capability-based tool definitions with JSON Schema (`input_schema`) — McpToolDef pattern
- Deploy graph (`squirrel_deploy.toml`) for BYOB biomeOS deployment

### Feature Gates

| Feature | Purpose | Default |
|---------|---------|---------|
| `capability-ai` | Capability-based AI routing | ON |
| `ecosystem` | Ecosystem integration | ON |
| `tarpc-rpc` | High-performance binary RPC | ON |
| `delegated-jwt` | Capability-based JWT delegation | ON |
| `system-metrics` | sysinfo C dependency | OFF |
| `monitoring` | Prometheus metrics | OFF |
| `gpu-detection` | ToadStool GPU detection | OFF |
| `local-jwt` | Local JWT (brings ring C dep) | OFF |

### Spring Absorption & Primal Integration

- Added `capability_registry.toml` (wetSpring pattern) — replaces hardcoded capability lists
- Added `squirrel_deploy.toml` (airSpring pattern) — BYOB deploy graph with germination order
- Registry loader (`capabilities/registry.rs`) — TOML→JSON schema conversion, compiled fallback
- `handle_discover_capabilities` reads from registry instead of hardcoded vec
- `handle_list_tools` enriched with `input_schema` from registry (neuralSpring McpToolDef pattern)
- `capability.announce` treats `capabilities` as tool routing fallback (neuralSpring adapter fix)
- biomeOS lifecycle module: `lifecycle.register`, `lifecycle.status` heartbeat, signal handlers
- Context RPC methods wired: `context.create`, `context.update`, `context.summarize`
- BearDog crypto discovery aligned to biomeOS socket scan
- ToadStool AI provider auto-discovered via biomeOS socket scan
- SIGTERM/SIGINT signal handlers with socket file cleanup

### Cleanup from pre-alpha

- Reduced unique dependencies from 314 to 272
- Eliminated HTTP stack from default build
- Feature-gated all cross-primal code (Songbird, ToadStool, BearDog, NestGate)
- Replaced deprecated crates (`serde_yaml` → `serde_yml`, `log` → `tracing`)
- Purged PII, large artifacts, and stale code from git history
- Fixed deadlock in ExperienceReplay (RwLock re-entrance)
- Fixed all MCPError Display formatting (missing `#[error]` attributes)
- Fixed squirrel-mcp-auth feature interaction (delegated-jwt vs local-jwt)
- Resolved all build warnings across workspace
- Archived 420+ stale docs, scripts, and showcase files
