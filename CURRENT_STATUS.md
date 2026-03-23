<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# Squirrel Current Status

**Last Updated**: March 23, 2026
**Version**: 0.1.0-alpha.22
**License**: AGPL-3.0-only (scyBorg: ORC + CC-BY-SA 4.0 for docs)

## Build

| Metric | Value |
|--------|-------|
| Build | GREEN — default features: 0 errors; `--all-features`: 0 errors |
| Tests | 6,720 passing / 0 failures across 22 workspace members |
| Edition | 2024 (Rust 1.94+) |
| Clippy | CLEAN — `pedantic + nursery + cargo + deny(unwrap/expect)` on `--all-targets`; zero warnings under `-D warnings` |
| Docs | All crates `#![warn(missing_docs)]`; `cargo doc --no-deps` clean |
| Formatting | `cargo fmt --all -- --check` passes |
| Unsafe Code | 0 in production — `#![forbid(unsafe_code)]` in **all** crate `lib.rs`, `main.rs`, and `bin/*.rs` files workspace-wide |
| Pure Rust | 100% default features (zero C deps); 14 C-dep crates banned in `deny.toml`; `sysinfo` removed |
| ecoBin | Compliant v3.0 — `deny.toml` bans 14 C-dep crates (groundSpring V115 standard); pure Rust `sys_info` via `/proc` parsing |
| Coverage | 86.0% line coverage via `cargo-llvm-cov` (target: 90%); remaining gap is IPC/network code requiring real socket connections and binary entry points |
| Crates | 22 workspace members |
| Files >1000 lines | 0 (max: 987 — learning/integration_tests.rs); all 19 previously >1000-line files smart-refactored |
| `#[expect(reason)]` | Workspace migrated from `#[allow]` to `#[expect(reason)]` — dead suppressions caught automatically |
| Cargo metadata | All crates have `repository`, `readme`, `keywords`, `categories`, `description` — zero `clippy::cargo` warnings |
| Property tests | 23 proptest properties + 2 TOML sync + identity invariant tests + Unix socket IPC tests |
| cargo deny | `advisories ok, bans ok, licenses ok, sources ok` |
| Mocks in production | 0 — `InMemoryMonitoringClient` documented as intentional fallback; all test mocks behind `#[cfg(test)]` |
| Legacy aliases | Removed — only semantic `{domain}.{verb}` method names accepted; `capabilities.list` canonical per SEMANTIC_METHOD_NAMING_STANDARD v2.1 |
| TODO/FIXME in code | 0 (documented `STUB` comments only in performance optimizer batch/optimizer — Phase 2 deferred; swarm, coordination, and crypto are production implementations) |
| Dev credentials | 0 hardcoded — all via env vars (`SQUIRREL_DEV_JWT_SECRET`, `SQUIRREL_DEV_API_KEY`) |
| Zero-copy | Hot-path clones audited; `Arc::clone()` for intent clarity; `mem::take` for payload moves; `String` → borrow in MCP task client |

## JSON-RPC Methods

Source of truth: [`capability_registry.toml`](capability_registry.toml)

| Domain | Methods |
|--------|---------|
| AI | `ai.query`, `ai.list_providers`, `ai.complete`, `ai.chat` |
| Capability | **`capabilities.list`** (canonical), `capability.announce`, `capability.discover`, `capability.list` (alias) |
| Context | `context.create`, `context.update`, `context.summarize` |
| System | `system.health`, `system.status`, `system.metrics`, `system.ping` |
| Health | `health.liveness`, `health.readiness` (PRIMAL_IPC_PROTOCOL v3.0) |
| Discovery | `discovery.peers` |
| Tool | `tool.execute`, `tool.list` |
| Lifecycle | `lifecycle.register`, `lifecycle.status` |
| Graph | `graph.parse`, `graph.validate` (primalSpring BYOB) |

**JSON-RPC batch support**: Full Section 6 compliance — array of requests → array of responses.

**Legacy aliases removed**: Flat names (`query_ai`, `health`, `ping`, etc.) no longer
accepted. All clients must use the semantic `{domain}.{verb}` names above.

## tarpc Service

tarpc 0.37 (upgraded from 0.34). All JSON-RPC methods mirrored as tarpc service
methods with typed request/response structs. `TarpcRpcServer` delegates to
`JsonRpcServer` for shared handler logic. Protocol negotiation (client + server)
selects tarpc or JSON-RPC per-connection.

## Niche Self-Knowledge (`niche.rs`)

Follows the groundSpring/wetSpring/airSpring niche pattern:

| Constant | What |
|----------|------|
| `CAPABILITIES` | 24 exposed methods (ai, capabilities, capability, system, discovery, tool, context, lifecycle, graph) |
| `CONSUMED_CAPABILITIES` | 32 external capabilities from BearDog, Songbird, ToadStool, NestGate, domain springs, rhizoCrypt, sweetGrass, primalSpring |
| `COST_ESTIMATES` | Per-method latency and GPU hints for Pathway Learner scheduling |
| `DEPENDENCIES` | 6 primals (beardog, songbird required; toadstool, nestgate, primalspring, petaltongue optional) |
| `SEMANTIC_MAPPINGS` | Short name → fully qualified capability mapping |
| `operation_dependencies()` | DAG inputs per operation for parallelization |

`capability.discover` response includes `cost_estimates`, `operation_dependencies`, and `consumed_capabilities`.

`capabilities.list` (canonical) / `capability.list` (alias) returns per-method cost/dependency detail for PathwayLearner scheduling,
plus a flat `capabilities` array, `domains` list, and `locality` (local/external) for
ecosystem-consensus introspection (absorbed from sweetGrass/rhizoCrypt).

## Primal Identity

Centralized in `universal-constants::identity`:

| Constant | Value | Usage |
|----------|-------|-------|
| `PRIMAL_ID` | `"squirrel"` | Socket naming, logging |
| `PRIMAL_DOMAIN` | `"ai"` | biomeOS Neural API domain registration |
| `JWT_ISSUER` | `"squirrel-mcp"` | JWT token `iss` claim |
| `JWT_AUDIENCE` | `"squirrel-mcp-api"` | JWT token `aud` claim |
| `JWT_SIGNING_KEY_ID` | `"squirrel-jwt-signing-key"` | BearDog key lookup |

Runtime discovery uses capabilities, not primal names. Names are only for socket
file naming conventions and logging. `CapabilityIdentifier` replaces the deprecated
`EcosystemPrimalType` enum.

## Context Management

Context handlers use real in-memory `DashMap` storage (not stubs). Each context session
has a unique ID, version tracking, and metadata. NestGate persistence will be wired when
NestGate's `storage.put` / `storage.get` capabilities are discovered at runtime.

## Service Registration

| Target | Protocol | Status |
|--------|----------|--------|
| biomeOS | `lifecycle.register` + 30s heartbeat | Active (when orchestrator detected) |
| Songbird | `discovery.register` + 30s heartbeat | Active (when Songbird socket detected) |

## Orchestration

`DeploymentGraphDef` types (from ludoSpring exp054) absorbed for multi-primal
composition awareness. Squirrel can parse deployment graphs and identify nodes
requiring AI capabilities.

## Feature Gates

| Feature | What it gates | Default |
|---------|---------------|---------|
| `capability-ai` | Capability-based AI routing (Pure Rust) | ON |
| `ecosystem` | Ecosystem integration | ON |
| `tarpc-rpc` | High-performance binary RPC via tarpc | ON |
| `delegated-jwt` | Capability-based JWT delegation | ON |
| `monitoring` | Prometheus metrics (brings hyper) | OFF |
| `nvml` | NVIDIA GPU detection via nvml-wrapper | OFF |
| `local-jwt` | Local JWT signing (brings ring C dep) | OFF |

## Human Dignity Evaluation

AI routing operations pass through `DignityEvaluator` checks:

| Check | What |
|-------|------|
| Discrimination risk | Flags operations involving employment, credit, housing, insurance, criminal justice |
| Human oversight | Requires human-in-the-loop for high-stakes decisions |
| Manipulation prevention | Detects urgency, scarcity, and dark-pattern language |
| Explainability | Flags black-box models used for consequential decisions |

`DignityGuard` wraps the evaluator with configurable enforcement (block vs warn).

## Zero-Copy Patterns

| Pattern | Where |
|---------|-------|
| `Arc<str>` for identifiers | `jsonrpc_handlers.rs` (`AnnouncedPrimal`), `self_knowledge.rs`, `EcosystemServiceRegistration` |
| `Arc<dyn ValidationRule>` | `validation.rs` — eliminates `Box::new(self.clone())` |
| `bytes::Bytes` for payloads | `transport/frame.rs` — O(1) clone on frame data |
| `&'static str` for constants | `self_knowledge.rs` — default capabilities |
| `Cow<str>` | IPC paths, configuration values |
| Struct update syntax | Builder patterns use `..Default::default()` throughout |

## Pure Rust System Info

`universal-constants::sys_info` provides OS-level metrics without C dependencies:

| Function | Implementation |
|----------|---------------|
| `memory_info()` | `/proc/meminfo` parsing on Linux; graceful fallback elsewhere |
| `process_rss_mb()` | `/proc/self/status` VmRSS parsing |
| `cpu_count()` | `std::thread::available_parallelism()` |
| `uptime_seconds()` | `/proc/uptime` parsing |
| `hostname()` | `rustix::system::uname()` |
| `system_cpu_usage_percent()` | `/proc/stat` delta sampling |

Replaces the `sysinfo` crate (C dependency) for ecoBin v3.0 compliance.

## Error Handling

| Crate | Error Type | Pattern |
|-------|-----------|---------|
| `squirrel-commands` | `CommandError` (thiserror) | Typed variants: Io, Serialization, Validation, Hook, Lifecycle, etc. |
| `squirrel-cli` | `FormatterError` (thiserror) | Serialization, UnknownFormat |
| `squirrel-mcp` | `MCPError` (thiserror) | Protocol, transport, context, plugin errors |
| `universal-error` | `UniversalError` | Cross-crate error type |
| `universal-patterns` | `IpcClientError` + `IpcErrorPhase` | Phase-tagged IPC errors with `.context()` chains |
| `universal-patterns` | `DispatchOutcome<T>` | Protocol vs application error separation at RPC dispatch |
| `universal-patterns` | `CircuitBreaker` + `RetryPolicy` | IPC resilience with exponential backoff gated by `IpcErrorPhase` |
| `universal-patterns` | `RpcError` + `extract_rpc_result()` + `extract_rpc_error()` | Centralized JSON-RPC result/error extraction |
| `squirrel` (main) | `PrimalError` | `From<anyhow::Error>` for seamless `.context()` chains |

## Logging

Production code uses `tracing` (`info!`, `warn!`, `error!`, `debug!`).
`println!` reserved for CLI user-facing output only.

## Plugin System

`UnifiedPluginManager` provides real plugin lifecycle:

| Component | Status |
|-----------|--------|
| `UnifiedPluginManager` | Implemented — load, unload, list, get, shutdown |
| `PluginEventBus` | Implemented — pub/sub with topic-based routing |
| `PluginSecurityManager` | Implemented — capability-based permission checks |
| `ManagerMetrics` | Implemented — load/unload/error counters |
| Performance optimizer | Batch/optimizer stubs deferred to Phase 2 (`batch_processor`, `optimizer`) |

## Ecosystem Integration

| Component | Status |
|-----------|--------|
| Capability Registry | `capability_registry.toml` loaded at startup |
| Niche Self-Knowledge | `niche.rs` with capabilities, costs, deps, consumed capabilities |
| Primal Identity | `universal-constants::identity` — centralized JWT/primal constants |
| Deploy Graph | `squirrel_deploy.toml` (BYOB pattern) |
| Orchestration Types | `DeploymentGraphDef`, `GraphNode`, `TickConfig` (ludoSpring wire-compatible) |
| biomeOS Lifecycle | `lifecycle.register` + 30s heartbeat (when orchestrator detected) |
| Songbird Discovery | `discovery.register` + 30s heartbeat (when Songbird detected) |
| BearDog Crypto | Discovery via biomeOS socket scan |
| ToadStool AI | Auto-discovered via capability-based biomeOS socket scan |
| Signal Handling | SIGTERM + SIGINT → socket cleanup + graceful shutdown |
| Health Probes v3.0 | `health.liveness` + `health.readiness` — PRIMAL_IPC_PROTOCOL v3.0 |
| Circuit Breaker | `CircuitBreaker` + `RetryPolicy` + `ResilientCaller` for IPC resilience |
| Manifest Discovery | `PrimalManifest` scan at `$XDG_RUNTIME_DIR/ecoPrimals/*.json` — Songbird fallback |
| OrExit Pattern | `OrExit<T>` trait + centralized `exit_codes` for zero-panic binary entry points |
| DispatchOutcome | `DispatchOutcome<T>` for protocol vs application error separation |
| Validation Harness | `ValidationHarness` for multi-check binary validation (doctor, validate) |
| 4-Format Capability Parsing | flat, object, nested, double-nested response formats |
| Primal Names | `primal_names::*` machine IDs + `display` submodule for all 13 ecosystem primals |
| Spring Tool Discovery | `spring_tools::SpringToolDiscovery` — runtime MCP tool aggregation from domain springs; `SpringToolDef` aligned with biomeOS `McpToolDefinition` V251 |
| Human Dignity | `DignityEvaluator` + `DignityGuard` for AI operation checks |
| BYOB Deploy Graphs | `graphs/squirrel_ai_niche.toml` + `ai_continuous_tick.toml` — primalSpring-compatible BYOB niche graphs |
| NicheDeployGraph Types | Wire-compatible with primalSpring `deploy.rs` — `[graph]` + `[[graph.node]]` TOML format |
| Graph Handlers | `graph.parse` + `graph.validate` — RPC endpoints for graph introspection |
| Capability Identifiers | `CapabilityIdentifier` type replacing deprecated `EcosystemPrimalType` enum |

## Socket Configuration

Injectable `SocketConfig` pattern (absorbed from airSpring). `FAMILY_ID`-compliant
per `PRIMAL_IPC_PROTOCOL.md`:

```
Tier 1: SQUIRREL_SOCKET (primal-specific override)
Tier 2: BIOMEOS_SOCKET_PATH (Neural API orchestration)
Tier 3: PRIMAL_SOCKET + family suffix
Tier 4: XDG runtime: /run/user/<uid>/biomeos/squirrel-${FAMILY_ID}.sock
Tier 5: /tmp/squirrel-<family>-<node>.sock (dev only)
```

When `FAMILY_ID` is not set, Tier 4 falls back to `squirrel.sock` (single-instance).
All tiers testable via `SocketConfig` DI without `temp_env` or `#[serial]`.

## Tooling

| Tool | Config |
|------|--------|
| just | `justfile` — ci, check, fmt, clippy, test, coverage, build-release, build-ecobin-all (x86_64+aarch64 musl), audit, doctor |
| rustfmt | `.rustfmt.toml` — edition 2024, max_width 100 |
| clippy | `clippy.toml` — pedantic + nursery + deny(unwrap/expect) via `[workspace.lints.clippy]` |
| cargo-deny | `deny.toml` — license allowlist, advisory audit, ban wildcards, deny yanked, 14-crate ecoBin C-dep ban |
| cargo-llvm-cov | 86.0% line coverage (target: 90%) |
| proptest | Round-trip + wire-format fuzz + IPC fuzz for all JSON-RPC types (23 properties) + Unix socket IPC tests |
| rust-toolchain | `rust-toolchain.toml` — pinned stable + clippy + rustfmt + llvm-tools-preview |

## Known Issues

1. Coverage at 86.0% — remaining ~4% gap to 90% is primarily IPC/network code, binary entry points, and WASM-dependent SDK paths
2. Performance optimizer `batch_processor` / `optimizer` stubs remain deferred to Phase 2 (swarm coordination, coordination service, and crypto are now production implementations, not stubs)

## Changes Since Last Handoff (March 23, 2026)

### alpha.22 Sprint (Deep Debt Resolution, Lint Pedantry & Cross-Ecosystem Absorption)

- **`#![forbid(unsafe_code)]` workspace-wide**: Applied to all `lib.rs`, `main.rs`, and `bin/*.rs` files across the entire workspace (previously only in select crate roots)
- **19 files >1000 lines smart-refactored**: Extracted types, handlers, and tests into submodules with re-exports for backward compatibility. Examples: `web/api.rs` (1266→183+endpoints+handlers+websocket+tests), `universal_primal_ecosystem/mod.rs` (1221→461+cache+discovery+ipc+tests), `primal_provider/core.rs` (1166→684+universal_trait+tests), all RPC servers, plugin managers, CLI modules, AI tools
- **`#[allow]` → `#[expect(reason)]` migration**: 59 files migrated; dead suppressions caught and removed; crate-level lint policies consolidated; unfulfilled expectations cleaned across auth, context, mcp, plugins, universal-patterns, interfaces, config, ecosystem-integration
- **Cargo metadata complete**: All 22 crates now have `repository`, `readme`, `keywords`, `categories`, `description` — zero `clippy::cargo` warnings
- **Clippy nursery/pedantic full clean**: Fixed `unnecessary_literal_bound` (→ `&'static str`), `manual_let_else`, `manual_string_new`, `strict_f32_comparison`, `redundant_clone`, `items_after_test_module`, and all unfulfilled lint expectations
- **Zero-copy clone audit**: Removed unnecessary clones in MCP task client (per-RPC String→borrow), auth provider discovery (move instead of clone), consensus messaging (`Arc::clone` for clarity), biomeOS context state (single-clone session IDs)
- **Config test hardening**: Pinned all timeout values in validation tests to resist env var pollution from parallel test runs under llvm-cov
- **Test count**: 6,717→6,720 (+28 targeted tests for AI routing, IPC, RPC handlers, capabilities, compute providers, transport)
- **Coverage**: 86.0% line coverage (86.6% region coverage) — remaining gap is IPC/network code and binary entry points
- **Files**: 1,318 `.rs` files, 445K total lines, max file 987 lines (all under 1000)
- **Quality gates**: `fmt` ✓, `clippy -D warnings` ✓, `doc` ✓, `deny` ✓, `test` ✓ (6,720/0)

### alpha.21 Sprint (Coverage Push & Zero-Copy Evolution)

- **Coverage 74.8% → 86.8%**: 12 percentage point increase via 22 targeted test waves across all workspace crates
- **Test count 5,828 → 6,717**: +889 new tests covering MCP security, context learning, services, SDK, AI tools, CLI, RPC handlers, universal adapters, biomeos integration, primal providers, transport, and more
- **Zero-copy evolution**: `MetricType`/`ConsensusStatus` made `Copy`; `Arc::clone` clarity; `mem::take` replaces payload clone in consensus messaging; redundant clones removed from collector, federation, RPC handlers
- **Production bug fixes discovered via tests**:
  - `task/manager.rs`: deadlock in `assign_task` — write lock held across async prerequisite check now resolved via snapshot-check-relock pattern
  - `web/api.rs`: `/api/plugins/health` and `/metrics` were shadowed by generic plugin-details route
  - `handlers_tool.rs`: spring tools were hijacking built-in `system.health`; built-ins now resolve first
  - `resource_manager/core.rs`: `get_usage_stats` now reports live background task count
  - `dispatch.rs`: flaky test from HashMap iteration order under llvm-cov instrumentation
- **Clippy**: CLEAN — `pedantic + nursery + deny(warnings)` on full workspace; zero warnings
- **Files**: All <1000 lines

### alpha.20 Sprint (Deep Debt Resolution, Semantic Compliance & Lint Tightening)

- **`capabilities.list` canonical**: Added per SEMANTIC_METHOD_NAMING_STANDARD v2.1; `capability.list` retained as required alias; niche self-knowledge, capability registry TOML, cost estimates, operation dependencies all updated; 24 exposed methods (was 23)
- **definitions.rs smart refactor**: 1121→585 lines by extracting `service.rs` (service mesh, load balancing, circuit breaker, database types) and `definitions_tests.rs`; zero files >1000 lines
- **Flaky llvm-cov tests fixed**: `test_config_validate_security_*` hardened with explicit port values to resist coverage-instrumentation variance
- **#[allow] suppression tightening**: Removed crate-level `#![allow(...)]` from `ecosystem-api` and `squirrel-core` entirely; reduced `universal-patterns` from ~40 to 16 allows; reduced `squirrel-cli` to 21 targeted allows; removed `items_after_test_module` from `ai-tools`
- **Dead code cleanup**: All `#[allow(dead_code)]` without `reason` evolved to documented `reason` strings; unused parse functions gated behind `#[cfg(test)]`; `PluginManifest::to_metadata` exercised via new test
- **Production unwrap audit**: All 5 hotspot files confirmed test-only unwrap/expect; zero production panics
- **Coverage wave 3**: +51 new tests across core/monitoring, main/alerts, universal messages/context/helpers, security rate_limiter, ecosystem types/registration, error paths, niche JSON validation
- **Test count**: 5,777→5,828 (+51 tests)
- **Coverage**: 74.6%→74.8% line coverage
- **Semantic consistency fix**: `semantic_mappings_json()` missing `list_capabilities → capabilities.list` entry corrected
- **Clippy**: CLEAN — `pedantic + nursery + deny(warnings)` on workspace; zero warnings
- **Files**: 1,293 `.rs` files, 427K total lines, max file 965 lines

### alpha.19 Sprint (Coverage, Refactoring & Dependency Modernization)

- **base64 0.21→0.22**: Unified across workspace (`squirrel-mcp`, `squirrel` main, workspace root); fixed 1 legacy `base64::encode` call → `Engine::encode`; `squirrel-mcp-auth` was already 0.22
- **web/api.rs smart refactor**: 977→859 lines by extracting 8 DTO types (`PluginInfo`, `EndpointInfo`, `PluginInstallRequest`, etc.) into `web/api_types.rs` (131 lines); re-exported from `web/mod.rs` for backward compatibility
- **ai-tools lib.rs tightened**: Removed 10 blanket clippy suppressions (`unused_imports`, `uninlined_format_args`, `use_self`, `redundant_closure_for_method_calls`, `redundant_else`, `manual_string_new`, `redundant_clone`, `assigning_clones`, `cloned_instead_of_copied`, `needless_raw_string_hashes`); 67 auto-fixes applied across 11 files
- **Coverage wave 2**: New test suites for `config/unified/types/definitions.rs` (30 tests), `core/auth/auth.rs`, `mcp/security/token.rs` (18 tests), `core/routing/balancer.rs` (18 tests), `mcp/protocol/websocket.rs` (15 tests), `mcp/enhanced/session.rs`
- **Test count**: 5,729→5,777 (+48 tests)
- **Coverage**: 74.3%→74.6% line coverage
- **Clippy fixes**: `missing_panics_doc` in `concurrent_test_helpers.rs`, `too_many_lines` + `redundant_clone` in config definitions tests
- **Dependency analysis**: `rand 0.8→0.9` (23 files, moderate effort — deferred for focused PR); `mockall 0.11→latest` (1 file, trivial — deferred); documented upgrade paths

### alpha.18 Sprint (Deep Debt Resolution & Compliance Sprint)

- **Clippy blocker fixed**: `ipc_routed_providers.rs` dead code gated behind `#[cfg(any(feature = "openai", feature = "anthropic", feature = "gemini"))]` — clippy now CLEAN on all features/targets
- **llvm-cov fixed**: `test_write_and_discover_tcp_endpoint` stabilised with deterministic temp directory + unique service names — coverage now measurable
- **Coverage**: 73% → 74.3% via new test suites for MCP error types (6 files), task server (7 files), plugin types (5 files), transport types (3 files)
- **License files**: Added `LICENSE-ORC` and `LICENSE-CC-BY-SA` per scyBorg triple-copyleft standard (matches Songbird, biomeOS)
- **CONTRIBUTING.md**: Created per PUBLIC_SURFACE_STANDARD
- **Rate limiter whitelist**: Evolved from hardcoded `127.0.0.1`/`::1` to env-configurable via `SQUIRREL_RATE_LIMIT_WHITELIST`
- **Plugin loader paths**: Evolved from hardcoded directories to env-configurable via `SQUIRREL_PLUGIN_DIRS`
- **SongbirdProvider → IPC-wired**: Evolved from stub to real IPC discovery via `universal-patterns::IpcClient`; gracefully degrades to tracing when monitoring socket unavailable
- **Workspace dep cleanup**: Removed dead `lazy_static` and `once_cell` from workspace `Cargo.toml` (already evolved to `std::sync::LazyLock` in prior sprints)
- **squirrel-core**: Added `universal-patterns` dependency for IPC monitoring integration
- **Clippy fixes**: Fixed `uninlined_format_args`, `redundant_clone`, `single_char_pattern`, `strict_f64_comparison`, `similar_names` in new test code

### alpha.17 Sprint (Alpha.17 Audit Sprint)

- **Clippy**: All clippy errors fixed (13+ in monitoring_tests, auth, ecosystem-api, commands, and 20+ more across the workspace).
- **Chaos**: `chaos_07_memory_pressure` fixed (no longer flaky).
- **CONTEXT.md**: Created per PUBLIC_SURFACE_STANDARD.
- **Hardcoded ports**: Evolved to capability discovery in the SDK and config defaults.
- **Production implementations**: SwarmCoordinator (peer tracking), CoordinationService (lifecycle FSM), DefaultCryptoProvider (ed25519+BLAKE3), web/api (real metrics), dashboard (live registry + /proc), discovery/registry (typed errors); prior swarm, coordination, and crypto stubs are now real implementations.
- **Clone proliferation**: Reduced via `HealthStatus: Copy`, `Arc::clone` clarity, and scan-then-remove patterns.
- **Modular refactoring**: `rate_limiter` (5 modules), `monitoring` (6 modules), `streaming` (4 modules), `transport` (5 modules).
- **Dead code**: Suppressions cleaned in 10+ files; upgraded `allow` to `expect(reason)` where appropriate.
- **SPDX**: 100% coverage (one missing file fixed).
- **Documentation**: `warn(missing_docs)` un-suppressed on squirrel-core, squirrel-mcp, and squirrel-cli; 400+ doc comments added.
- **JSON-RPC**: Semantic naming is 100% `domain.verb` compliant (22 methods).
- **cargo deny**: Clean (advisories ok, bans ok, licenses ok, sources ok).
- **Metrics**: Test count 5,574→5,775 (+201); coverage 71%→73%.
- **Unwrap audit**: All production `unwrap` verified test-only with `cfg_attr` gating.
- **New tests**: Unix socket IPC, RPC error paths, timeout coverage, and lifecycle edge cases.

### alpha.16 Sprint (Deep Debt Resolution & Compliance Audit)

- **Clippy pedantic**: Zero warnings on `cargo clippy --all-features -- -D warnings` — `#[must_use]` on 11+ functions, `# Errors` docs on 123+ Result-returning functions, removed blanket `must_use_candidate`/`missing_errors_doc` allows
- **Dependency evolution**: `serde_yml` (unsound/unmaintained) → `serde_yaml_ng` v0.10 (maintained fork); removed unused `config` v0.13 and `yaml-rust` v0.4
- **cargo-deny clean**: `advisories ok, bans ok, licenses ok, sources ok` — pinned all 22 wildcard internal deps, documented `cc` build-time exceptions, advisory ignores for tarpc-transitive `bincode`
- **Capability-based discovery**: Hardcoded ports/IPs evolved to `DiscoveredEndpoint` pattern + env-var discovery chain; primal only has self-knowledge
- **File refactoring**: `ipc_client.rs` (999L → 6-module split), `types.rs` (972L → 4-file split), `traits.rs` (960L → 6-file split), `monitoring.rs` tests extracted; all files <1000 lines
- **Production stub evolution**: `PlaceholderPlugin` → `NoOpPlugin`/`DefaultPlugin` (null object), federation → `CapabilityUnavailable` error variant, AI providers → IPC-routed delegation via `IpcRoutedVendorClient`
- **Unwrap audit**: Removed blanket `#![allow(clippy::unwrap_used)]` from `universal-patterns`, fixed production unwraps in config/presets/security; all crates use `cfg_attr(test, allow(…))`
- **Test expansion**: 5,440 → 5,574 tests; core/core 0% → 86-100% coverage; new tests across main (shutdown, rate_limiter, rpc, biome), SDK, ecosystem-api, cli, ai-tools
- **Coverage**: 69.95% → 71.05% lines (72.79% regions, 70.83% functions)
- **Doc fixes**: 12 intra-doc link warnings fixed, zero doc warnings on `cargo doc --all-features --no-deps`
- **SPDX**: 100% (1,287/1,287 `.rs` files)
- **Files**: 1,287 `.rs` files, 425K total lines, max file 985 lines

### Prior: alpha.15 Sprint (BYOB Graph Coordination)

- **`NicheDeployGraph` types**: primalSpring-compatible `[graph]` + `[[graph.node]]` TOML with structural validation, capability queries, JSON roundtrip
- **2 BYOB deploy graphs**: `squirrel_ai_niche.toml` (Sequential: Tower → Squirrel → petalTongue), `ai_continuous_tick.toml` (Continuous: 10 Hz AI → viz loop)
- **`graph.parse` + `graph.validate` RPC handlers**: Accept TOML, return parsed/validated graphs — enables primalSpring to send graphs for introspection
- **Coordination consumed capabilities**: `coordination.validate_composition`, `coordination.deploy_atomic`, `composition.nucleus_health` (primalSpring)
- **Dependencies**: 4 → 6 (+ primalSpring, petalTongue optional)
- **Capabilities**: 21 → 23 exposed, 29 → 32 consumed
- **10 new graph tests**: Parse, validate, capability query, roundtrip, all-graphs sweep
- **Tests**: 5,440 passing, 0 failures

### Prior (alpha.14)

- Capability registry TOML sync test, `identity::PRIMAL_DOMAIN`, `SpringToolDef` McpToolDefinition alignment, 7 consumed capabilities, aarch64-musl CI

### Prior (alpha.13)

- Spring tool discovery, centralized `extract_rpc_result()`, capability-first sockets, ecoBin 14-crate ban, primal display names, 6 proptest IPC fuzz tests

### Prior (alpha.12)

- Smart file refactoring, hardcoded URL extraction, discovery stubs evolved, 346+ new tests, redis 0.23→1.0.5

### Prior (alpha.11)

- Lint tightening, 170+ clippy fixes, tarpc negotiation, sysinfo removal, plugin manager, human dignity, capability identifiers
