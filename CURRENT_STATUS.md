<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# Squirrel Current Status

**Last Updated**: March 22, 2026
**Version**: 0.1.0-alpha.17
**License**: AGPL-3.0-only (scyBorg: ORC + CC-BY-SA 4.0 for docs)

## Build

| Metric | Value |
|--------|-------|
| Build | GREEN — default features: 0 errors; `--all-features`: 0 errors |
| Tests | 5,775 passing / 0 failures across 21 workspace members |
| Edition | 2024 (Rust 1.94+) |
| Clippy | CLEAN — `pedantic + nursery + deny(unwrap/expect)` on `--all-features --all-targets`; zero warnings |
| Docs | All crates `#![warn(missing_docs)]`; `doc_markdown` clean |
| Formatting | `cargo fmt --all -- --check` passes |
| Unsafe Code | 0 in production — `#![forbid(unsafe_code)]` in all crate entry points |
| Pure Rust | 100% default features (zero C deps); 14 C-dep crates banned in `deny.toml`; `sysinfo` removed |
| ecoBin | Compliant v3.0 — `deny.toml` bans 14 C-dep crates (groundSpring V115 standard); pure Rust `sys_info` via `/proc` parsing |
| Coverage | 73% line coverage via `cargo-llvm-cov` (target: 90%) |
| Crates | 21 workspace members |
| Files >1000 lines | 0 (max: 977 — web/api.rs) |
| Property tests | 23 proptest properties + 2 TOML sync + identity invariant tests + Unix socket IPC tests |
| redis | 1.0.5 (upgraded from 0.23) |
| Mocks in production | 0 — `InMemoryMonitoringClient` documented as intentional fallback; all test mocks behind `#[cfg(test)]` |
| Legacy aliases | Removed — only semantic `{domain}.{verb}` method names accepted |
| TODO/FIXME in code | 0 (documented `STUB` comments only in performance optimizer batch/optimizer — Phase 2 deferred; swarm, coordination, and crypto are production implementations) |
| Dev credentials | 0 hardcoded — all via env vars (`SQUIRREL_DEV_JWT_SECRET`, `SQUIRREL_DEV_API_KEY`) |

## JSON-RPC Methods

Source of truth: [`capability_registry.toml`](capability_registry.toml)

| Domain | Methods |
|--------|---------|
| AI | `ai.query`, `ai.list_providers`, `ai.complete`, `ai.chat` |
| Capability | `capability.announce`, `capability.discover`, **`capability.list`** |
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
| `CAPABILITIES` | 23 exposed methods (ai, capability, system, discovery, tool, context, lifecycle, graph) |
| `CONSUMED_CAPABILITIES` | 32 external capabilities from BearDog, Songbird, ToadStool, NestGate, domain springs, rhizoCrypt, sweetGrass, primalSpring |
| `COST_ESTIMATES` | Per-method latency and GPU hints for Pathway Learner scheduling |
| `DEPENDENCIES` | 6 primals (beardog, songbird required; toadstool, nestgate, primalspring, petaltongue optional) |
| `SEMANTIC_MAPPINGS` | Short name → fully qualified capability mapping |
| `operation_dependencies()` | DAG inputs per operation for parallelization |

`capability.discover` response includes `cost_estimates`, `operation_dependencies`, and `consumed_capabilities`.

`capability.list` returns per-method cost/dependency detail for PathwayLearner scheduling,
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
| cargo-llvm-cov | 73% line coverage (target: 90%) |
| proptest | Round-trip + wire-format fuzz + IPC fuzz for all JSON-RPC types (23 properties) + Unix socket IPC tests |
| rust-toolchain | `rust-toolchain.toml` — pinned stable + clippy + rustfmt + llvm-tools-preview |

## Known Issues

1. Coverage at 73% — gap to 90% target; incremental expansion underway
2. Performance optimizer `batch_processor` / `optimizer` stubs remain deferred to Phase 2 (swarm coordination, coordination service, and crypto are now production implementations, not stubs)

## Changes Since Last Handoff (March 22, 2026)

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
