<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# Squirrel Current Status

**Last Updated**: April 16, 2026
**Version**: 0.1.0
**License**: AGPL-3.0-or-later (scyBorg: ORC + CC-BY-SA 4.0 for docs)

## Build

| Metric | Value |
|--------|-------|
| Build | GREEN — default features: 0 errors; `--all-features`: 0 errors |
| Tests | 7,156 passing / 0 failures across 22 workspace members |
| Edition | 2024 (Rust 1.94+) |
| async-trait | **0 usage** — all 64 `#[async_trait]` annotations removed; dyn-safe traits use explicit `Pin<Box<dyn Future>>`, non-dyn traits use native `async fn` + `#[expect(async_fn_in_trait)]`; `async-trait` only remains as transitive dep from external crates (`config`, `wiremock`, `test-context`) |
| Clippy | CLEAN — `pedantic + nursery + cargo + deny(unwrap/expect)` on `--all-targets`; zero warnings under `-D warnings` |
| Docs | All crates `#![warn(missing_docs)]`; `cargo doc --no-deps` clean |
| Formatting | `cargo fmt --all -- --check` passes |
| Unsafe Code | 0 in production — `unsafe_code = "forbid"` in workspace `[lints.rust]` (all 22 crates) |
| Pure Rust | 100% default features (zero C deps, zero non-Rust crypto); 14 C-dep crates banned in `deny.toml`; `sysinfo` removed; `ed25519-dalek` feature-gated behind `local-crypto`; `flate2` → pure Rust `miniz_oxide` backend; `blake3` → `features = ["pure"]` (no SIMD assembly); `pprof`, `openai`, `libloading` removed; `rand` upgraded 0.8→0.9.4 (RUSTSEC-2026-0097); `ring`/`reqwest`/`zstd-sys` only resolve under `--all-features` (not in default/ecoBin build) |
| ecoBin | Compliant v3.0 — 3.5 MB static-pie musl binary, stripped, BLAKE3 checksummed, zero host paths (`--remap-path-prefix`), zero dynamic deps; `deny.toml` bans 14 C-dep crates + `tokio-tungstenite` (Tower Atomic) + `reqwest` (Tower Atomic); pure Rust `sys_info` via `/proc` parsing |
| Coverage | **90.1%** region coverage / 89.6% line coverage via `cargo-llvm-cov` (**target met**); remaining uncovered: binary entry points, demo bins, WASM-only SDK paths, live IPC server loops |
| `.unwrap()` in code | 0 — workspace-wide elimination; all Results use `?` or `.expect("invariant")` |
| `panic!()` in code | 0 — replaced with `unreachable!()` or proper assertions |
| `Box<dyn Error>` | 0 in production APIs — replaced with typed errors + `anyhow::Result` (`PrimalError`, `AIError`, `SquirrelError`, `ContextError`, `MCPError`, `EcosystemError`, `anyhow::Error`) |
| Crates | 22 workspace members |
| Files >800 lines (prod) | 0 — all production `.rs` files under 800 lines; max production file ~798L (`btsp_handshake.rs`); test files up to 965L (expected). Smart-refactored: `workflow_manager.rs` (831→403), `server/mod.rs` (840→647), `mcp/client.rs` (836→605), `ecosystem client.rs` (824→659), `plugins/manager.rs` (816→706); types extracted to sibling modules |
| `#[expect(reason)]` | Workspace migrated from `#[allow]` to `#[expect(reason)]` — dead suppressions caught automatically |
| Cargo metadata | All crates have `repository`, `readme`, `keywords`, `categories`, `description` — zero `clippy::cargo` warnings |
| Property tests | 23 proptest properties + 2 TOML sync + identity invariant tests + Unix socket IPC tests |
| cargo deny | `advisories ok, bans ok, licenses ok, sources ok` |
| Mocks in production | 0 — all production stubs evolved to honest capability-based patterns: `SecurePluginStub` rejects execution (security sandbox, documented); `NoOpPluginManager` returns errors; plugin web API returns 501 (Phase 2); `WebVisualizationServer` logs capability-pending; `UnavailableServiceRegistry` returns empty (honest); learning integration wired to live `ContextManager` data; neural engine evolved from tanh stub to ReLU MLP; federation `dead_code` fields wired to real diagnostics; all test mocks behind `#[cfg(any(test, feature = "testing"))]` |
| Primal self-knowledge | All hardcoded primal names evolved to capability-based: `BearDog*` → `SecurityProvider*`, `Songbird*` → `Discovery*`/`ServiceMesh*`, `NestGate` → `ContentAddressed`; deprecated type aliases for backward compat; env var chains prefer capability names (`SECURITY_ENDPOINT` → `BEARDOG_ENDPOINT` fallback) |
| Legacy aliases | Backward-compatible aliases for ecosystem compat; `capabilities.list` canonical per SEMANTIC_METHOD_NAMING_STANDARD v2.1 |
| TODO/FIXME in code | 0 — no TODO/FIXME/HACK markers in committed code; Phase 2 placeholders wired with capability fallback or documented with `#[expect(dead_code, reason)]` |
| Dev credentials | 0 hardcoded — all via env vars (`SQUIRREL_DEV_JWT_SECRET`, `SQUIRREL_DEV_API_KEY`) |
| Zero-copy | Hot-path clones audited; `ServiceInfo` string fields evolved to `Arc<str>`; `Arc::clone()` for intent clarity; `mem::take` for payload moves; `String` → borrow in MCP task client |

## JSON-RPC Methods

Source of truth: [`capability_registry.toml`](capability_registry.toml)

| Domain | Methods |
|--------|---------|
| Inference | **`inference.complete`**, **`inference.embed`**, **`inference.models`**, **`inference.register_provider`** (canonical per SEMANTIC_METHOD_NAMING_STANDARD v2.0 §7) |
| AI | `ai.query`, `ai.list_providers`, `ai.complete`, `ai.chat` (backward-compat aliases → `inference.*` handlers) |
| Capability | **`capabilities.list`** (canonical), `capabilities.announce`, `capabilities.discover`, `capability.announce` (alias), `capability.discover` (alias), `capability.list` (alias), `primal.capabilities` (alias) |
| Identity | `identity.get` (CAPABILITY_BASED_DISCOVERY_STANDARD v1.0) |
| Context | `context.create`, `context.update`, `context.summarize` |
| System | **`system.metrics`** (canonical), `system.health`, `system.status`, `system.ping` (backward-compat aliases) |
| Health | **`health.check`**, **`health.liveness`**, **`health.readiness`** (canonical — PRIMAL_IPC_PROTOCOL v3.0) |
| Discovery | `discovery.peers`, `discovery.list` (alias) |
| Tool | `tool.execute`, `tool.list` |
| Lifecycle | `lifecycle.register`, `lifecycle.status` |
| Graph | `graph.parse`, `graph.validate` (primalSpring BYOB) |

**JSON-RPC batch support**: Full Section 6 compliance — array of requests → array of responses.

**Legacy prefix normalization**: `normalize_method()` strips `squirrel.` and `mcp.` prefixes
for ecosystem backward compatibility (e.g. `squirrel.system.health` → `system.health`).

**Health tiering**: `health.check` (canonical; `system.health` alias) returns `HealthTier` (alive/ready/healthy) per
CAPABILITY_BASED_DISCOVERY_STANDARD v1.0 — alive (process running), ready (providers
initialized), healthy (fully operational with served requests).

## tarpc Service

tarpc 0.37 (upgraded from 0.34). All JSON-RPC methods mirrored as tarpc service
methods with typed request/response structs. `TarpcRpcServer` delegates to
`JsonRpcServer` for shared handler logic. Protocol negotiation (client + server)
selects tarpc or JSON-RPC per-connection.

## Niche Self-Knowledge (`niche.rs`)

Follows the groundSpring/wetSpring/airSpring niche pattern:

| Constant | What |
|----------|------|
| `CAPABILITIES` | 29 exposed methods (inference, ai, capabilities, capability, identity, system, health, discovery, tool, context, lifecycle, graph) |
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
| Discovery service | `discovery.register` + 30s heartbeat | Active (when discovery socket detected) |

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
| `deprecated-adapters` | Vendor-specific HTTP adapters (Anthropic, OpenAI) — v0.3.0 removal. Use `UniversalAiAdapter` + `LOCAL_AI_ENDPOINT`. | OFF |

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
| `squirrel-interfaces` | `SquirrelError` (thiserror) | Cross-crate trait error type — replaces `Box<dyn Error>` in all trait signatures |
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
| Performance optimizer | Implemented — `batch_processor`, `optimizer` completed (NOTE(phase2) stubs resolved) |

## Ecosystem Integration

| Component | Status |
|-----------|--------|
| Capability Registry | `capability_registry.toml` loaded at startup |
| Niche Self-Knowledge | `niche.rs` with capabilities, costs, deps, consumed capabilities |
| Primal Identity | `universal-constants::identity` — centralized JWT/primal constants |
| Deploy Graph | `squirrel_deploy.toml` (BYOB pattern) |
| Orchestration Types | `DeploymentGraphDef`, `GraphNode`, `TickConfig` (ludoSpring wire-compatible) |
| biomeOS Lifecycle | `lifecycle.register` + 30s heartbeat (when orchestrator detected) |
| Discovery Service | `discovery.register` + 30s heartbeat (when discovery socket detected) |
| BearDog Crypto | Discovery via biomeOS socket scan |
| ToadStool AI | Auto-discovered via capability-based biomeOS socket scan |
| Signal Handling | SIGTERM + SIGINT → socket cleanup + graceful shutdown |
| Health Probes v3.0 | `health.liveness` + `health.readiness` — PRIMAL_IPC_PROTOCOL v3.0 |
| Circuit Breaker | `CircuitBreaker` + `RetryPolicy` + `ResilientCaller` for IPC resilience; `StandardRetryPolicy::from_env()` with primal→ecosystem→default chain |
| Manifest Discovery | `PrimalManifest` scan at `$XDG_RUNTIME_DIR/ecoPrimals/*.json` — discovery service fallback |
| TCP JSON-RPC listener | TCP JSON-RPC listener for remote/tooling access alongside Unix socket transport |
| Capability domain symlink | `ai.sock` capability-domain symlink for Neural API / biomeOS alignment |
| Workspace dependency centralization | Shared `[workspace.dependencies]` + `{ workspace = true }` in member crates |
| Smart file refactoring | Large modules split with tests extracted; file-size compliance maintained |
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
| rustfmt | `rustfmt.toml` — edition 2024, max_width 100 |
| clippy | `clippy.toml` — pedantic + nursery + deny(unwrap/expect) via `[workspace.lints.clippy]` |
| cargo-deny | `deny.toml` — license allowlist, advisory audit, ban wildcards, deny yanked, 14-crate ecoBin C-dep ban |
| cargo-llvm-cov | **90.1%** region coverage / 89.6% line coverage (**target met**) |
| proptest | Round-trip + wire-format fuzz + IPC fuzz for all JSON-RPC types (23 properties) + Unix socket IPC tests |
| rust-toolchain | `rust-toolchain.toml` — pinned stable + clippy + rustfmt + llvm-tools-preview |

## Known Issues

1. **Coverage target met** — 90.1% region coverage (89.6% line). Remaining uncovered: binary entry points, demo binaries, WASM-only SDK paths, live IPC server loops. All production modules have test coverage.
2. Performance optimizer `batch_processor` / `optimizer` are complete (no deferred stubs)
3. `ring` present as transitive dependency via `rustls`/`sqlx`/`jsonwebtoken` — tracked in `docs/CRYPTO_MIGRATION.md` for future crypto provider evolution
4. `base64` duplicate (0.21 via `config`/`ron`, 0.22 direct) — transitive, benign
5. `async-trait` — **0 annotations** in Squirrel code (migrated from 228 → 0); dyn-safe traits use `Pin<Box<dyn Future>>`, non-dyn traits use native `async fn in trait`; `async-trait` remains only as transitive dep from external crates (`config`, `wiremock`, `test-context`)

## Changes Since Last Handoff (April 16, 2026)

### April 16, 2026 session X (coverage 86%→90.1%: 144 targeted tests across 15 production modules)

- **Coverage target met**: 90.1% region coverage / 89.6% line coverage (was 86.0% / 88.98%). 144 new tests (7,012→7,156) targeting: jsonrpc_server, sdk/mcp/client, ai/router, sdk/plugin, btsp_handshake, monitoring_provider, learning/manager, discovery, cli/config, cli/security, ai/adapter, rule-system/manager, rule-system/evaluator, universal-patterns/config, interning, universal_adapter_v2, ipc_routed_providers, shutdown, sdk/logging, sdk/fs, sdk/http
- **SDK error tests wired**: `sdk/infrastructure/error/tests.rs` was 0% (334 missed) because tests used `#[wasm_bindgen_test]` only — fixed with dual `#[test]` + `#[wasm_bindgen_test]` macro
- **Production bugs found via tests**: `set_rule_manager` held `RwLock::write()` across `await` (deadlock risk) — fixed; `load_from_file` nested JSON branch didn't update `self.models` — fixed
- **Quality gates**: `fmt` ✓, `clippy -D warnings` ✓ (0 warnings), `test` ✓ (7,156 passed / 0 failures), `deny` ✓

### April 15, 2026 session W (deep debt: primal self-knowledge, smart refactoring, mock evolution, dependency purity)

- **Primal self-knowledge — BearDog→SecurityProvider**: `capability_crypto.rs` socket stem → `"security"`; `errors.rs` `BeardogIntegration` → `SecurityProviderIntegration`; `lib.rs` exports → `SecurityProviderJwtConfig`, `SecurityProviderJwtService`, `SecurityProviderClient`; `security_coordinator.rs` → `authenticate_with_security_provider`, `requires_security_provider`. All primal-named symbols have `#[deprecated(since = "0.2.0")]` aliases.
- **Primal self-knowledge — Songbird→Discovery**: `SongbirdLoadBalancerConfig` → deprecated, `DEFAULT_SONGBIRD_PORT` → `DEFAULT_DISCOVERY_PORT`; `DISCOVERY_ENDPOINT`/`DISCOVERY_PORT` canonical env vars with `SONGBIRD_*` as fallback.
- **Primal self-knowledge — NestGate→ContentAddressed**: `ContextStorage::NestGate` → `ContentAddressed` with `serde(alias = "nestgate")`; `DatabaseBackend::NestGate` → `ContentAddressed` with serde backward compat.
- **Smart refactoring — 5 production files under 800L**: `workflow_manager.rs` (831→403, tests+helpers extracted), `server/mod.rs` (840→647, handlers extracted), `mcp/client.rs` (836→605, listener+interactive extracted), `ecosystem client.rs` (824→659, DTOs+mock extracted), `plugins/manager.rs` (816→706, metadata+test_plugin extracted)
- **Production mocks evolved**: Learning integration wired to live `ContextManager` data (session count, intervention detection); neural engine evolved from tanh stub to ReLU MLP (`new_mlp`, `forward_scores`); federation `dead_code` fields wired to real `find_leader_node` + diagnostics; all stubs documented as intentional deny-policy or honest capability-fallback
- **blake3 → pure Rust**: `blake3 = { default-features = false, features = ["pure"] }` — no SIMD assembly compilation, no C code in default build
- **Dependency verification**: `ring`/`reqwest`/`zstd-sys` confirmed absent from default build (only resolve under `--all-features`); `cargo deny` clean
- **Quality gates**: `fmt` ✓, `clippy -D warnings` ✓ (0 warnings), `test` ✓ (7,012 passed / 0 failures), `deny` ✓, `doc` ✓

### April 15, 2026 session V (primalSpring audit: async-trait elimination, genetics prep, smart refactoring, rand upgrade)

- **async-trait fully eliminated**: All 64 remaining `#[async_trait]` annotations removed across 8 crates (`squirrel-interfaces`, `squirrel-plugins`, `squirrel-context`, `squirrel-rule-system`, `squirrel-cli`, `squirrel`, `adapter-pattern-examples`, `adapter-pattern-tests`). Dyn-safe traits (`Plugin`, `DynPlugin`, `DynContext*`, `ContextPlugin`, `ConditionEvaluator`, `ActionExecutor`, `WebPlugin`, `ZeroCopyPlugin`, `CommandAdapter`, `UniversalServiceRegistry`) use explicit `Pin<Box<dyn Future<Output = …> + Send + '_>>`. Non-dyn traits use native `async fn in trait` + `#[expect(async_fn_in_trait)]`. `async-trait` removed from workspace `[dependencies]` and all 8 crate `Cargo.toml`s. Zero Squirrel code imports it; only remains as transitive dep from `config`, `wiremock`, `test-context`.
- **Three-tier genetics / mito-beacon prep**: Assessed readiness for `primalspring >= 0.10.0` `mito_beacon_from_env()`. BTSP handshake code (`btsp_handshake.rs`) annotated with evolution roadmap: `family_seed_ref` → mito-beacon fields; Phase 3 cipher negotiation → `BTSP_CHACHA20_POLY1305` when BearDog server-side ready. FAMILY_ID env var chain and discovery already clean. No local code action needed until primalspring 0.10.0 ships (currently 0.9.14).
- **BLAKE3 content curation**: Assessed — blocked on NestGate content-addressed storage API stability. Squirrel already uses BLAKE3 for ecoBin checksums.
- **Phase 3 cipher negotiation**: Assessed — blocked on BearDog `btsp.negotiate` server-side readiness. Current NULL cipher post-handshake is per-spec.
- **Smart refactoring**: `client.rs` (844→664L), `dependency_resolver.rs` (814→731L) extracted DTOs to sibling modules
- **rand 0.8→0.9.4**: Upgraded per RUSTSEC-2026-0097; ed25519-dalek compat via `rand::fill` + `SigningKey::from_bytes`
- **Quality gates**: `fmt` ✓, `clippy -D warnings` ✓ (0 warnings), `test` ✓ (7,011 passed / 0 failures), `deny` ✓, `doc` ✓

### April 14, 2026 session T (primalSpring audit: CLI bind gap, hardcoding evolution, production stubs, smart refactoring)

- **SQ-04 RESOLVED — CLI bind gap**: `squirrel server --port 9500` was unreachable from Docker containers because TCP bound to `127.0.0.1`. Added `--bind` CLI flag + `SQUIRREL_BIND`/`SQUIRREL_IPC_HOST` env vars with precedence: CLI > env > config > default (`127.0.0.1`). Docker pattern: `--bind 0.0.0.0 --port 9500`
- **Hardcoded primal names eliminated**: `"toadstool"` → `"compute"` capability stem in AI router socket discovery; `SONGBIRD_SOCKET` fallback removed from discovery service (prefer `DISCOVERY_SOCKET`); `"petalTongue"` → "visualization capability discovery" in web visualization
- **Hardcoded `127.0.0.1`** in universal listener → `universal_constants::network::LOCALHOST_IPV4`
- **Hardcoded `/tmp/` paths** evolved across 5 files: discovery.rs, lifecycle.rs, status.rs, local.rs, discovery_service.rs → `universal_constants::network::get_socket_dir()` / `BIOMEOS_SOCKET_FALLBACK_DIR`
- **Production stubs evolved**: RL policy `get_training_iterations`/`get_last_loss`/`get_performance_metrics`/`load_weights` → use real `training_state`/`metrics` fields + file I/O; context learning `extract_features` → JSON-aware extraction from context state
- **Unused `hostname` dependency removed** from workspace
- **Smart refactoring — 9 production files under 800L**: `integration.rs` (881→700), `dashboard.rs` (856→605), `zero_copy.rs` (851→670), `service.rs` (828→723), `builder.rs` (827→768), `jsonrpc_server.rs` (872→756), `router.rs` (828→701), `sync.rs` (819→733) — types/configs/impl blocks extracted to sibling modules
- **Orphaned files removed**: `crates/config/src/unified/security.rs` (not in build graph, dead code), `zero_copy_types.rs` (duplicate artifact)
- **Quality gates**: `fmt` ✓, `clippy -D warnings` ✓ (0 warnings), `test` ✓ (7,003 passed / 0 failures), `check --workspace` ✓

### April 11, 2026 session S (Deep debt cleanup: test extraction, stub evolution, hardcoding elimination, dead code removal)

- **9 large mixed files refactored** — inline `#[cfg(test)] mod tests { ... }` extracted to sibling `*_tests.rs` files via `#[path]` pattern: `self_healing_tests.rs` (470 lines), `ecosystem_jwt_tests.rs` (442), `error_tests.rs` (380), `history_tests.rs` (296), `hardening_tests.rs`, `monitoring_tests.rs`, `unix_socket_tests.rs`, `retry_tests.rs`, `plugin_tests.rs`; all production files now well under 600 lines
- **Plugin web API evolved** — `install_plugin`, `get_plugin_config`, `execute_plugin_command` now return honest 501 (Not Implemented) with structured error JSON instead of placeholder fake success responses; marketplace `install_plugin` aligned
- **No-op stubs evolved to capability-based patterns** — `WebVisualizationServer::start()` logs capability-pending; `ContextPluginManager::load_plugins_from_path` uses capability.call dispatch; `discover_via_service_mesh` logs endpoint and explains capability wiring
- **AI intelligence evolved** — `process_intelligence_request` now measures real `Instant` timing, returns `confidence: 0.0` and `"engine_status": "awaiting_capability_wiring"` instead of fake 0.9 confidence
- **Predictive loader evolved** — `generate_predictions()` checks usage patterns map, logs when empty, returns honest empty Vec
- **Identity auth evolved** — `authenticate()` now `warn!` on password skip with username context (security risk visibility)
- **State SVG evolved** — returns proper SVG with capability-pending message instead of minimal `<text>` stub
- **Swarm service documented** — `#[expect(dead_code)]` with Phase 2 capability.call reason; placeholder wording replaced with capability description
- **Federation capabilities evolved** — `get_node_capabilities()` now reads from `SQUIRREL_EXPOSED_CAPABILITIES` (shared with `niche.rs`) instead of hardcoded strings; single source of truth via `universal_constants::capabilities`
- **`env_name()` removed** — deprecated method with hardcoded `TOADSTOOL`/`SONGBIRD`/`BEARDOG`/`NESTGATE` strings replaced by `endpoint_env_prefix()` derived from `capability()` (e.g. `service-mesh` → `SERVICE_MESH`); all callers migrated
- **Crypto socket paths evolved** — `capability_crypto.rs` hardcoded `/run/user/.../beardog.sock` and `/tmp/beardog.sock` replaced with tiered `candidate_crypto_signing_socket_paths()`: `SECURITY_SOCKET` → `BEARDOG_SOCKET` (legacy) → `resolve_capability_unix_socket("CRYPTO_CAPABILITY_SOCKET", "beardog")` → `/tmp/beardog.sock` last resort; `nix` dependency removed from auth crate
- **AI router hardcoding eliminated** — `primal_names::TOADSTOOL` socket construction replaced with `resolve_capability_unix_socket("COMPUTE_SOCKET", "toadstool")`; `localhost:11434` replaced with `deployment::endpoints::ollama()`; log messages use capability descriptions ("compute primal", "service mesh") instead of marketing names
- **Ecosystem-api env vars documented** — legacy `SONGBIRD_*`/`TOADSTOOL_*`/`NESTGATE_*` fallbacks annotated with "prefer `SERVICE_MESH_*`/`COMPUTE_*`/`STORAGE_*`" comments
- **Orphan visualization files removed** — `state_viz.rs`, `rule_viz.rs`, `metrics_viz.rs` deleted (not in module tree, referenced non-existent `Visualizable` trait)
- **async-trait audit** — all 73 remaining `#[async_trait]` usages confirmed necessary (all on `dyn`-dispatched traits); no zero-dyn migration candidates
- **Quality gates**: `fmt` ✓, `clippy -D warnings` ✓ (0 warnings), `test` ✓ (6,881 passed / 0 failures), `check --workspace` ✓

### April 11, 2026 session R (Tower Atomic WebSocket removal, canonical inference namespace, provider registration)

- **WebSocket transport removed from squirrel-mcp**: `websocket` feature, `tokio-tungstenite` dependency, and all gated modules (protocol/websocket, transport/websocket, websocket_tests) removed; Tower Atomic pattern — WebSocket provided by Songbird service mesh, not embedded in primals
- **`tokio-tungstenite` banned in deny.toml**: Tower Atomic compliance; `squirrel-sdk` exempted via `wrappers` (migration debt: SDK MCP client evolve to IPC transport)
- **`tokio-tungstenite` removed from workspace deps**: Comment documents Tower Atomic rationale
- **Canonical `inference.*` namespace**: Per SEMANTIC_METHOD_NAMING_STANDARD v2.0 §7 — `inference.complete`, `inference.embed`, `inference.models`, `inference.register_provider` wired as first-class JSON-RPC methods
- **`inference.register_provider` implemented**: Springs (neuralSpring, healthSpring, ludoSpring) can register as inference backends via JSON-RPC; creates `RemoteInferenceAdapter` that forwards `inference.complete` calls over UDS
- **`RemoteInferenceAdapter` created**: New `adapters/remote_inference.rs` — forwards inference to registered springs via `send_jsonrpc_public` over Unix domain sockets
- **`ai.*` methods → backward-compat aliases**: `ai.query`, `ai.complete`, `ai.chat` now route to `handle_inference_complete`; `ai.list_providers` unchanged
- **`handlers_inference.rs` wired**: Previously orphan source file now declared in `rpc/mod.rs` and fully active in dispatch
- **niche.rs updated**: `CAPABILITIES` (25→29), `SEMANTIC_MAPPINGS`, `COST_ESTIMATES`, `operation_dependencies()`, `cost_estimates_json()`, `semantic_mappings_json()` all include `inference.*` methods
- **`capability_registry.toml` updated**: 4 new `inference.*` capability definitions with input schemas
- **Test update**: `ai_query_dispatches_to_router_and_returns_echo` updated for `inference.complete` response format (`text` field instead of `response`)
- **Quality gates**: `fmt` ✓, `clippy -D warnings` ✓ (0 warnings), `test` ✓ (2145 passed / 0 failures), `check --workspace` ✓

### April 8, 2026 session P (Deep Debt: Self-Knowledge Violations, Production Mocks, Dependency Cleanup)

- **BEARDOG_API_KEY → SECURITY_API_KEY** — `core/auth/providers.rs` EncryptionService and ComplianceMonitor now read `SECURITY_API_KEY` first, falling back to `BEARDOG_API_KEY` for legacy compat; eliminates primal-specific env var violation
- **`/tmp/token` → env-based resolution** — `universal-patterns/security/providers/local.rs` LocalSecurityProvider token path now resolves via `SECURITY_TOKEN_FILE` → `$XDG_RUNTIME_DIR/biomeos/security.token` → `/tmp/biomeos-security.token` fallback
- **`"primalspring"` → `primal_names::PRIMALSPRING`** — Added `PRIMALSPRING` constant + display name to `universal-constants/primal_names.rs`; niche.rs DEPENDENCIES now uses the constant instead of raw string
- **`DummyPluginManager` → `NoOpPluginManager`** — Renamed across 5 files (mod.rs, actions.rs, plugin.rs, tests.rs, plugin_tests.rs) with honest documentation: "returns errors for all lookups" rather than "for testing purposes"; changed to unit struct
- **SDK fs.rs WASM stubs** — `exists()` now returns `false` (was `true`); `read_file_internal()` returns empty binary (was "Hello World"); `upload_file()` returns error; all documented as WASM sandbox stubs pending host wiring
- **10 orphan workspace dependencies removed** — `hex`, `uuid-serde`, `lru`, `indexmap`, `argon2`, `simple_logger`, `secrecy`, `tarpc-mcp`, `axum-mcp`, `axum-extra-mcp` were declared in workspace but unused by any member crate
- **rule-system version alignment** — `toml = "0.7"` → `toml.workspace = true` (0.8); `glob = "0.3"` → `glob.workspace = true`
- **Unfulfilled lint cleanup** — Removed stale `clippy::unnested_or_patterns` expectation from SDK lib.rs
- Quality: 6,875 tests passing, 0 failures, 0 clippy warnings, 0 fmt diffs, 0 doc errors

### April 8, 2026 session O (BTSP §Security Model — BIOMEOS_INSECURE guard, GAP-MATRIX-12)

- **`validate_insecure_guard()`** — BTSP Protocol Standard compliance: primal refuses to start when both `FAMILY_ID` (non-default) and `BIOMEOS_INSECURE=1` are set; checks `SQUIRREL_FAMILY_ID` first, then `FAMILY_ID` (primal-specific env var precedence per `PRIMAL_SELF_KNOWLEDGE_STANDARD.md` §4)
- **Injectable guard** — `validate_insecure_guard_with(has_family, insecure)` for pure-function testing without env var side effects; `SocketConfig` extended with `biomeos_insecure: Option<bool>` field
- **Startup gate** — guard fires first in `run_server()` before config load, socket resolution, or daemon fork; returns `exit_code::CONFIG` (2) on violation
- **9 new tests** — 4 injectable unit tests (neither, family-only, insecure-only, both-rejected) + 5 env-based tests via `temp_env` (no conflict, family-only, family+insecure, primal-family+insecure, default-family-is-not-production)
- **Quality gates** — `fmt` ✓, `clippy -D warnings` ✓ (0 warnings), `test` ✓ (6,875/0/107), `doc` ✓

### April 8, 2026 session N (Wire Standard compliance, deep debt, dead code removal)

- **Wire Standard L2 compliance**: `capabilities.list` returns flat `methods` array; `identity.get` returns `primal`/`version`/`domain`/`license`; `health.liveness` returns `"status": "alive"` — all per CAPABILITY_WIRE_STANDARD.md
- **Daemon mode implemented**: Safe re-exec pattern via `std::process::Command` (no `unsafe`); `--daemon` flag spawns detached child with `SQUIRREL_DAEMONIZED=1`; parent prints PID and exits
- **reqwest banned in `deny.toml`**: Tower Atomic pattern enforced — all HTTP routes through Songbird service mesh via IPC
- **Production mocks eliminated**: SDK MCP `OperationHandler` (6 methods) evolved from fake hardcoded data to honest empty/error returns with `connected: bool` for future IPC wiring; web adapter `get_component_markup` evolved from placeholder HTML to proper error
- **Socket-first endpoint resolution**: `DefaultEndpoints::socket_path(service)` added as primary tier in ecosystem-api defaults — Unix socket before HTTP fallback (Tower Atomic)
- **Dead code removed**: `orchestration/mod.rs` (791 lines) discovered never compiled (not in `lib.rs` module tree); used banned `reqwest` directly — removed entirely
- **Smart refactoring**: `severity.rs` (803→275 lines production) — 550+ lines of tests extracted to `severity_tests.rs` via `#[path]` pattern
- **SDK lint cleanup**: Removed unfulfilled `clippy::if_not_else` from lint expectations — clippy now zero warnings across workspace
- **Quality gates** — `fmt` ✓, `clippy -D warnings` ✓ (0 warnings), `test` ✓ (6,850/0/101), `doc` ✓

### April 5, 2026 session M (async-trait wave 3: deep dyn→generics across all tiers)

- **async-trait annotations reduced 168 → 129** (39 more removed): 15+ traits migrated across Tiers A/B/C
- **NetworkConnection consolidated**: 3 duplicate trait definitions → 1 canonical def with re-exports; `FederationNetwork`/`FederationNetworkManager` genericized
- **Sovereignty traits genericized**: `DefaultSovereignDataManager<E, A>` generic over `EncryptionKeyManager`/`AccessControlManager`
- **PlatformExecutor**: `RegisteredPlatformExecutor` enum dispatch, `Box<dyn>` eliminated
- **SessionManager**: `SquirrelPrimalProvider<S: SessionManager = SessionManagerImpl>` — production code unchanged, tests use concrete mock
- **PluginRegistry**: `WebPluginRegistry<R>` / `PluginManagementInterface<R>` genericized, `dyn PluginRegistry` removed from web boundary
- **MCPInterface**: `AIRouter<M: MCPInterface = NoMcpInterface>` / `McpAiToolsAdapter<M>` genericized, all `dyn MCPInterface` eliminated
- **AiCapability**: `BridgeAdapter<C: AiCapability>` generic, RPITIT for Send-safe futures, `BoxedAiCapability` removed
- **ServiceMeshClient**: `HealthMonitor<C>` / `ServiceDiscovery<C>` genericized, all `dyn ServiceMeshClient` eliminated
- **KeyStorage**: `SecurityManagerImpl<K: KeyStorage = InMemoryKeyStorage>` genericized
- **AuthenticationService**: `SecurityMiddleware<A: AuthenticationService>` genericized
- **ContextAdapter**: RPITIT + `ContextAdapterDyn` blanket for dyn-safe wrapper, `async_trait` removed from trait def
- **CommandsPlugin/MessageHandler**: migrated to native async, concrete types replace `dyn`
- **Dependency hygiene**: `async-trait` removed from `squirrel-mcp`, `squirrel-mcp-auth`, `squirrel-commands` Cargo.toml
- **Deferred** (heterogeneous collections require `dyn`): `MonitoringProvider`, `PrimalProvider`, `WebPlugin`, `ConditionEvaluator`, `ZeroCopyPlugin`, `ActionPlugin`, `ActionExecutor`, `RepositoryProvider`
- **Quality gates** — `fmt` ✓, `clippy -D warnings` ✓ (default + `--all-features --all-targets`), `test` ✓, `doc` ✓, `deny` ✓

### April 5, 2026 session L (deep async-trait migration wave 2 + dyn-to-generics evolution)

- **async-trait annotations reduced from 205 → 168** (37 more removed): 26 zero-dyn traits migrated to native async; 5 low-dyn traits converted via dyn→generics/enum dispatch
- **Zero-dyn wave 2** (26 trait defs + impl blocks):
  - `core/core`: `PrimalCoordinator`, `McpRouter`, `SwarmManager`, `ServiceMeshLoadBalancerIntegration`, `EnhancedMcpRouter`
  - `core/mcp`: `Transport` (+ 3 impls: SimpleTransport, WebSocketTransport, MockTransport)
  - `core/plugins`: `AppPlugin`, `CliPlugin`, `PluginLoader`, `PluginDiscovery`, `PluginDistribution`, `MonitoringPlugin`, `WebPluginExt`, `TestUtilsPlugin`, `ToolPlugin`, `PluginManagerTrait`, `LegacyWebPluginTrait`
  - `universal-patterns/federation`: `FederationNetwork`, `ConsensusManager`, `SovereignDataManager`, `CrossPlatformExecutor`, `UniversalExecutor`
  - `universal-patterns/security`: `ZeroCopySecurityProvider`
  - `main/monitoring`: `MetricsExporter` (converted to enum dispatch)
  - `main/tests/chaos`: `ChaosScenario` (+ 6 test impls)
  - `tools/rule-system`: `FileWatcher`
- **dyn→generics evolution** (5 traits with 1-3 dyn usages):
  - `MetricsExporter` → `MetricsExporterHandle` enum dispatch (Prometheus + JSON variants)
  - `ShutdownHandler` → `RegisteredShutdownHandler` enum dispatch
  - `IpcHttpDelegate` → generic `IpcRoutedVendorClient<D: IpcHttpDelegate>` with RPITIT `+ Send` bounds
  - `SecurityProvider` + `UniversalSecurityProvider` + `UniversalSecurityService` → `UniversalSecurityProviderBox` enum + blanket impls; `UniversalSecurityClient` no longer uses `dyn`
  - `ComputeProvider` → `ComputeProviderImpl` enum dispatch
  - `ServiceRegistryProvider` → `UnavailableServiceRegistry` concrete type
- **Dependency hygiene**: `async-trait` moved from `[dependencies]` to `[dev-dependencies]` for `squirrel-context-adapter` and `squirrel-integration` (test-only usage)
- **Clippy fixes**: Elidable lifetimes in `ZeroCopySecurityProvider`, `unnecessary_literal_bound` in `UnavailableServiceRegistry`, `use_self` in `IpcRoutedVendorClient`
- **Doc examples updated**: `security/traits.rs` doc examples removed `#[async_trait]` + `use async_trait::async_trait`
- **`LegacyWebPluginTrait`**: Methods use RPITIT (`fn handle_request() -> impl Future<Output = ...> + Send`) for `Send` guarantee
- **Quality gates** — `fmt` ✓, `clippy -D warnings` ✓ (default + `--all-features --all-targets`), `doc` ✓, `deny` ✓

### April 5, 2026 session K (async-trait → native Rust 2024 async fn in trait migration)

- **23 `#[async_trait]` annotations removed** (228 → 205): 10 trait definitions + 13 impl blocks migrated to native `async fn` in trait across 11 files
- **Tier 1 traits migrated** (zero `dyn` dispatch — safe drop-in):
  - `AIProvider` (`ecosystem-api/src/traits/ai.rs`)
  - `EcosystemIntegration` (`ecosystem-api/src/traits/primal.rs`) + 1 impl in `universal_provider.rs`
  - `Primal` (`universal-patterns/src/traits/primal.rs`) + 4 test impls + 1 in `primal_tests.rs`
  - `GpuInferenceCapability` (`universal-patterns/src/capabilities.rs`)
  - `ServiceMeshCapability` (`universal-patterns/src/capabilities.rs`)
  - `OrchestrationProvider` (`universal-patterns/src/orchestration/mod.rs`) + 2 impls
  - `TryFlattenStreamExt` (`tools/ai-tools/src/router/types.rs`) + 1 impl
  - `ContextManager` (`core/interfaces/src/context.rs`) + 1 impl in `core/context/src/manager/mod.rs`
  - `MockAdapter` (`adapter-pattern-tests/src/integration.rs`) + 3 impls
- **Tier 2 trait migrated** (`AuthenticationCapability` — `dyn` only in doc example + 2 tests):
  - `AuthenticationCapability` (`universal-patterns/src/capabilities.rs`) + 1 mock impl
  - Doc example updated: `&dyn AuthenticationCapability` → `&impl AuthenticationCapability`
  - Tests updated: `&dyn` → concrete `MockAuthService`
  - `async_trait` import fully removed from `capabilities.rs`
- **Tier 2 deferred** (production `dyn` dispatch — requires architectural refactoring):
  - `UniversalPrimalProvider` (production `Box<dyn>` in config.rs)
  - `AuthenticationService` (production `Arc<dyn>` in middleware.rs)
- **Lint strategy**: `#[expect(async_fn_in_trait, reason = "...")]` on migrated traits — suppresses `async_fn_in_trait` warning since all impls guarantee `Send + Sync`
- **Dead imports cleaned**: Removed `use async_trait::async_trait` from 4 files where it was the sole user
- **Quality gates** — `fmt` ✓, `clippy -D warnings` ✓ (default + `--all-features --all-targets`), `test` ✓ (all pass), `doc` ✓, `deny` ✓

### April 3, 2026 session J (deep debt execution, stub evolution, self-reference cleanup, zero-copy)

- **Production stubs evolved to complete implementations**:
  - `create_compute_from_type` — removed vendor-specific match arms (k8s/docker/nomad); added `LocalProcessProvider` with workload tracking for dev/test; all non-local providers delegate via `compute.execute` capability discovery
  - `auto_detect_compute_provider` — removed ToadStool-specific detection; uses `COMPUTE_ENDPOINT` env var for capability-based detection, falls back to local
  - `SecurePluginStub::execute` — returns `SecurityError` instead of fake success; sandbox plugins reject direct execution
  - `AiIntelligence::analyze_ecosystem_state` — uses actual engine telemetry (active predictions, automation count, prediction accuracy) instead of hardcoded values
  - `AiIntelligence::generate_optimizations` — derives recommendations from `OptimizationEngine` strategies and history
  - `AiIntelligence::generate_ecosystem_report` — delegates to `analyze_ecosystem_state` + `generate_optimizations` for real data
  - `IntelligenceEngine/OptimizationEngine/PredictionEngine/AutomationEngine/FederationIntelligence::initialize` — log actual engine state (model counts, strategy counts, accuracy); clear stale state
  - `is_healthy()` on OptimizationEngine/PredictionEngine — `const fn` checking actual model/strategy availability
  - `ContextAnalytics::initialize/update_analytics/shutdown` — resets counters, logs metrics snapshots
  - `StateVersioning::initialize/cleanup_old_versions` — tracks version history size, logs audit info
- **Hardcoded "squirrel" self-references → `niche::PRIMAL_ID`**: 20+ production references across `universal_adapters/` (storage, compute, orchestration, security), `primal_provider/` (core, health_monitoring, ecosystem_integration), `rpc/` (jsonrpc_server, unix_socket), `tool/executor`, `security/beardog_coordinator`, `ecosystem/manager`, `biomeos_integration/mod`, `universal_provider`, `discovery/self_knowledge`
- **Removed `primal_names` import** from `compute_client/provider_trait.rs` — no vendor-primal coupling in compute detection
- **Dead code cleanup**: Removed 42KB of orphaned `sync/manager.rs` (917 lines) and `sync/types.rs` (368 lines) — never compiled (not declared as submodules); actual sync module is `sync.rs` (826 lines, under 1000)
- **Zero-copy evolution**: `ServiceInfo` string fields (`service_id`, `name`, `category`, `endpoints`) evolved from `String` → `Arc<str>` — eliminates deep copies in high-frequency capability discovery queries
- **Unfulfilled lint expectation fixed**: `capability_jwt_integration_tests.rs` — `#[expect(clippy::expect_used)]` removed (no violations); replaced with `#[allow]`
- **Quality gates** — `fmt` ✓, `clippy -D warnings` ✓ (default + `--all-features --all-targets`), `test 6,856/0/107` ✓, `doc` ✓, `deny` ✓

### April 3, 2026 session I (primalSpring audit compliance, domain sovereignty, overstep resolution)

- **primalSpring audit resolution**: Reviewed wateringHole gap registry and primalSpring downstream audit findings
- **MockAIClient cfg gate hardened**: Removed blanket `#[allow(warnings)]` from `ai-tools/tests/basic_test.rs` that was hiding lint violations; replaced with targeted `#[allow(missing_docs, clippy::unwrap_used, clippy::expect_used)]`; all `MockAIClient` usages properly gated behind `#[cfg(any(test, feature = "testing"))]`
- **ed25519-dalek overstep resolved (BearDog domain)**: `ed25519-dalek` moved from required to **optional** dependency behind `local-crypto` feature; `DefaultCryptoProvider` (crypto.rs) and `SecurityManagerImpl` crypto paths gated with `#[cfg(feature = "local-crypto")]`; encrypt/decrypt return helpful error when crypto feature absent (directing to BearDog capability discovery); `enhanced`/`full` features include `local-crypto` for backward compat
- **sled/sqlx overstep confirmed clean**: `sled` not present in dependency tree; `sqlx` properly optional behind `persistence` feature in `rule-system` only (not in default build)
- **Default build is now zero-crypto**: No `ed25519-dalek` or signing code compiled in default features — TRUE PRIMAL sovereignty (delegates crypto to BearDog at runtime)
- **Quality gates** — `fmt` ✓, `clippy -D warnings` ✓ (default + `--all-features --all-targets`), `test 6,855/0/107` ✓, `doc` ✓, `deny` ✓

### April 3, 2026 session H (ORC-Notice compliance, hardcode evolution, smart refactoring)

- **ORC-Notice headers**: Added `// ORC-Notice:` to all 16 crate `lib.rs`/`main.rs` files that were missing them; 25/25 entry points now have consistent SPDX + ORC + Copyright headers
- **Hardcoded values evolved to env-configurable**: `trust_domain` now reads `SQUIRREL_TRUST_DOMAIN` / `SECURITY_TRUST_DOMAIN` with `"biome.local"` fallback; resource requirements (`cpu`, `memory`, `storage`, `network`, `gpu`) configurable via `SQUIRREL_RESOURCE_*` env vars; `mod.rs` uses `Default::default()` instead of re-stating literals
- **Smart refactoring of large files**:
  - `shutdown.rs` (917→517 lines): tests extracted to `shutdown_tests.rs` (395 lines) as sibling test module; added `pub(crate) phase_timeout()` accessor to avoid leaking field visibility
  - `integration_tests.rs` (988→668 lines): LearningIntegration lifecycle tests extracted to `integration_lifecycle_tests.rs` (323 lines)
- **Ignored tests reviewed**: Only 6 `#[ignore]` in codebase — 3 network-dependent (MCP server), 2 destructive chaos (FD/disk exhaustion), 1 external crypto provider — all legitimately gated
- **Dependency audit**: `cargo deny check` passes (advisories ok, bans ok, licenses ok, sources ok); `base64` duplicate (0.21 via `ron`/`config`, 0.22 direct) is transitive; `bincode` unmaintained tracked via `RUSTSEC-2025-0141` ignore
- **Quality gates** — `fmt` ✓, `clippy -D warnings` ✓, `test 6,859/0/107` ✓, `doc` ✓, `deny` ✓

### April 3, 2026 session G (Dead-code removal, test idiomacy, concurrency-model improvements)

- **65,910 lines of orphan dead code removed** from `squirrel-mcp` — ~246 files existed on disk but were never compiled (not declared in `mod.rs`); entire orphan module trees removed: `observability/`, `tool/`, `monitoring/`, `plugins/`, `integration/`, `sync/`, `context_manager/`, `client/`, `session/`, `server/`, plus orphan protocol adapter, transport TCP/memory/stdio, resilience circuit-breaker/bulkhead/recovery/state-sync, and 12 loose root-level `.rs` files
- **`CommandRegistry` `Mutex` → `RwLock`** — `commands` and `resources` maps converted for concurrent reads; `execute` signature fixed (`&Vec<String>` → `&[String]`)
- **IPC client timeout test** — 60s `tokio::time::sleep` → `std::future::pending()` (zero wasted time)
- **Context adapter TTL test** — 3s → 2.1s sleep with 1s TTL
- **Learning integration test** — 120ms → 50ms background sync wait
- **Remaining sleep audit** — all `thread::sleep` in compiled code confirmed legitimate (sync tests, wall-clock timestamp resolution); all `tokio::time::sleep` in compiled tests confirmed necessary (rate limiter refill, chaos harnesses, security alerting pipelines)
- **Quality gates** — `fmt` ✓, `clippy -D warnings` ✓, `test 7,165/0/110` ✓, `deny` ✓

### April 3, 2026 session F (primalSpring audit: build fix, capability-domain decoupling wave 2)

- **Integration test build fix** — `MockAIClient` `cfg(test)` gate invisible to integration binaries; mock-dependent tests now `cfg(feature = "testing")`; E0282 type inference resolved
- **Flaky `find_biomeos_socket` test fixed** — no longer asserts `is_none()` when real sockets may exist on host
- **`register_songbird_service` → `register_orchestration_service`** — public API renamed to capability-domain
- **`delegate_to_songbird` → `delegate_to_http_proxy`** — IPC HTTP delegation uses `http.proxy` capability
- **`metric_names::songbird` → `metric_names::orchestration`** — metric namespace generalized
- **`SongbirdIntegration` → `ServiceMeshIntegration`** — orchestration provider type renamed
- **`ConfigBuilder::songbird()` → `ConfigBuilder::orchestration()`** — config preset generalized
- **Examples generalized** — `universal_adapters_demo.rs` and `observability_demo.rs` use capability-domain names
- **Quality gates** — `fmt` ✓, `clippy -D warnings` ✓, `test 7,165/0/110` ✓, `deny` ✓

### April 3, 2026 session D (Deep debt execution — lint hygiene, trait evolution, stub maturity)

- **`#[allow(` → `#[expect(reason)]`** — 93 suppressions across 62 files migrated to `#[expect(reason)]`; dead suppressions now caught automatically
- **`KeyStorage` trait extracted** — `InMemoryKeyStorage` now implements `KeyStorage` trait; `SecurityManagerImpl` accepts `Arc<dyn KeyStorage>` via `with_key_storage()` constructor; production deployments can inject HSM/BearDog backends
- **Hardcoded localhost elimination (wave 2)** — 7 more production modules evolved: `service_mesh_client`, `tcp transport`, `websocket config`, `auth init`, `endpoint_resolver`, `PrimalEndpoints`, `url_builders`; all resolved via `universal_constants` helpers
- **`get_task_status` stub evolved** — returns HTTP 404 "unknown" instead of fake "completed"; documents Phase 2 persistence requirement honestly
- **`discover_capabilities` documented** — `tracing::debug!` on empty map, Phase 2 noted in non-test build path
- **`Box<dyn Error>` audited** — all usages confirmed correct: generic framework (bulkhead), binary entry points (ai-config), test helpers (cli); blanket `From` impls documented
- **Clone patterns audited** — top-5 clone-heavy files confirmed idiomatic (Arc/String clones for async task movement)
- **`println!` audit** — all 17 instances in `main.rs`/`doctor.rs` confirmed intentional CLI output
- **Quality gates** — `fmt` ✓, `clippy -D warnings` ✓, `test 7,165/0/110` ✓, `deny` ✓

### April 2, 2026 session C (Capability-based discovery compliance — primalSpring PRIORITY 3)

- **Songbird name decoupled from discovery** — `capabilities/songbird.rs` → `capabilities/discovery_service.rs`; public API `discover_songbird_socket` → `discover_discovery_socket`
- **Monitoring types renamed** — `SongbirdProvider`/`SongbirdConfig`/`SongbirdMonitoringClient` → `MonitoringServiceProvider`/`MonitoringServiceConfig`/`ServiceMeshMonitoringClient`
- **Config fields renamed** — `songbird_endpoint` → `discovery_endpoint` across `OrchestrationConfig` and `DiscoveryConfig`; `SongbirdConfig` → `ServiceMeshConfig` in ecosystem-api
- **All SONGBIRD_* env vars deprecated** — zero direct reads; all behind `.or_else()` fallbacks to new `DISCOVERY_*`/`SERVICE_MESH_*`/`MONITORING_*` primary names
- **Bootstrap documented** — `discovery.sock` symlink pattern for chicken-and-egg resolution
- **Quality gates** — `fmt` ✓, `clippy -D warnings` ✓, `test 7,162/0/110` ✓, `doc` ✓

### April 2, 2026 session B (Deep debt execution, dependency evolution, discovery-first)

- **50+ unused dependencies removed** across 13 crates — massive supply chain reduction via cargo-machete + manual verification
- **Production mock isolation** — `MockAIClient` gated behind `#[cfg(any(test, feature = "testing"))]`; no mock code compiled into production
- **Port unification** — Conflicting `DEFAULT_MCP_PORT` (8778 vs 8444) resolved to 8444; doc comments corrected
- **Hardcoded localhost → discovery** — `ecosystem_service`, `federation/service`, `dashboard_integration`, `presets` all evolved from `"localhost"/"127.0.0.1"` to `universal_constants::get_bind_address()`/`get_host()`/`build_http_url()`
- **Hardcoded primal endpoints → capability discovery** — 4 universal adapters evolved from `*.ecosystem.local` URLs to env-discoverable `get_host("SERVICE_ENDPOINT", ...)` patterns
- **Primal schema neutralized** — Hardcoded primal chain example in `schemas.rs` replaced with generic role-based description
- **Smart refactoring** — `optimization.rs` (919 lines) → 4-module directory (selector/scorer/utils/tests)
- **Orphan module audit** — Identified uncompiled modules in mcp, ai-tools, and main crates (documented for future cleanup)
- **Doc example TODOs resolved** — Replaced `todo!()`/`unimplemented!()` in doc examples with illustrative error returns
- **deny.toml cleanup** — Removed stale `RUSTSEC-2026-0002` advisory ignore (lru removed); cleaned unused license allowances
- **justfile** — `cargo test` now runs `--all-features` to enable testing feature for integration test mock access
- **Quality gates** — `fmt` ✓, `clippy -D warnings` ✓, `test 7,161/0/110` ✓, `doc` ✓, `deny` ✓

### April 2, 2026 session A (SQ-04 audit, workspace unsafe lint, rustdoc fixes)

- **SQ-02 RESOLVED**: `LOCAL_AI_ENDPOINT` / `OLLAMA_ENDPOINT` / `OLLAMA_URL` now wired into `AiRouter::new_with_discovery` — local AI always discovered alongside cloud providers; default Ollama probe at `localhost:11434` as implicit fallback; `ai.query` routes to local inference when available
- **SQ-03 socket path**: Confirmed conformant — `$XDG_RUNTIME_DIR/biomeos/squirrel.sock` with `ai.sock` symlink; primalSpring audit finding was from stale deployment
- **SQ-03 `deprecated-adapters` feature**: Documented in CURRENT_STATUS.md feature gates table
- **Clippy fix**: `clippy::type_complexity` in federation test with `#[expect(reason)]`
- **Test fix**: `test_validation` hardened with `temp_env::with_vars` to pin all timeout env vars (prevents pollution from parallel runs)
- **Quality gates**: `fmt` ✓, `clippy --all-features -D warnings` ✓, `doc --no-deps` ✓, `test` ✓ (6,839/0/107)

### March 31, 2026 session (TCP JSON-RPC, capability symlink, workspace deps, refactoring)

- **TCP JSON-RPC listener**: TCP JSON-RPC listener alongside Unix socket — remote clients and tooling can attach without a local socket path
- **Capability domain symlink (`ai.sock`)**: Symlink aligns the Neural API / biomeOS capability domain with the canonical socket name for discovery and orchestration
- **Workspace dependency centralization**: `[workspace.dependencies]` in the root manifest with `{ workspace = true }` in member crates — fewer version skews and simpler upgrades
- **Smart file refactoring**: Large modules split with tests extracted; ecosystem/core/plugin surfaces kept under file-size limits
- **Health RPC naming**: `health.check`, `health.liveness`, `health.readiness` canonical; `system.health`, `system.status`, `system.ping` backward-compat aliases; `system.metrics` remains canonical for system metrics
- **Performance optimizer**: `batch_processor` / `optimizer` NOTE(phase2) work completed; TODO/FIXME/HACK sweep clean in committed code
- **Tests**: 6,839 passing / 0 failures / 107 ignored across 22 workspace members (accurate post-llvm-cov reconciliation)

### alpha.25b Sprint (Deep Debt Evolution & Modern Idiomatic Rust)

- **License SPDX reconciled**: All 22 crate `Cargo.toml`, `.rustfmt.toml`, `clippy.toml`, `justfile`, and `LICENSE` updated from `AGPL-3.0-only` to `AGPL-3.0-or-later` per wateringHole standard
- **File size compliance**: `jsonrpc_handlers_tests.rs` (1,034→715 lines) split via `jsonrpc_ai_router_tests.rs` (195 lines) with `TestAiAdapter` abstraction; `config/validation.rs` (1,122→600 lines) split via `validation_tests.rs` (521 lines); **zero files >1,000 lines**
- **Production stubs evolved**: `state_sync::process_state_update` → full validation/serialization/storage/metrics; `sovereign_data` crypto → `blake3` XOF keystream + `rand` CSPRNG; security providers → `blake3` keyed hash + capability-based auth; `mcp_adapter::send_request` → explicit error (not mock response)
- **Dependency evolution**: `sha2` → `blake3` (pure Rust) in CLI checksums; `libloading` removed (secure plugin stub); security providers use `blake3` + `rand` instead of toy XOR
- **JSON-RPC semantic compliance**: Added `health.check`, `primal.capabilities`, `discovery.list` aliases; unified `capability.*` → `capabilities.*` canonical with backward-compatible aliases
- **Dead code cleanup**: Removed Phase 2 placeholder structs from `mcp_adapter.rs`; conditional `#[cfg(test)]` imports to silence unused warnings
- **Doc tests**: 33 ignored doc tests fixed and now passing
- **Coverage**: 85.4% → 86.5% line coverage (+78 new tests, 33 fixed doc tests)
- **Test count**: 6,761 → 6,839 passing, 0 failures, 107 ignored
- **Root docs updated**: README.md, CONTEXT.md, CURRENT_STATUS.md synced with accurate metrics (22 crates, 6,839 tests, 86.5% coverage, AGPL-3.0-or-later)
- **justfile cleaned**: Removed dead `archive/` path references; SPDX header corrected
- **Quality gates**: `fmt` ✓, `clippy pedantic+nursery -D warnings` ✓, `doc --no-deps` ✓, `test` ✓ (6,839/0)

### alpha.25 Sprint (Ecosystem Absorption & Modern Idiomatic Rust Evolution)

- **`identity.get` handler**: New JSON-RPC handler returning primal self-knowledge (id, domain, version, transport, protocol, license, JWT issuer/audience) per CAPABILITY_BASED_DISCOVERY_STANDARD v1.0
- **`normalize_method()`**: Strips `squirrel.` and `mcp.` prefixes for ecosystem backward compatibility (BearDog v0.9.0, barraCuda v0.3.7 pattern)
- **Health tiering**: `system.health` returns 3-tier `HealthTier` (alive/ready/healthy) — alive (process running), ready (providers initialized), healthy (fully operational with served requests); extends `HealthCheckResponse` with `tier`, `alive`, `ready`, `healthy` booleans
- **JSON-RPC 2.0 strictness**: Validates `method` field (present, non-empty, string), `params` type (object/array only when present); proper single-request notification handling (no response body); standard error codes defined (-32700 through -32099)
- **Cast safety lints**: Added `cast_possible_truncation`, `cast_sign_loss`, `cast_precision_loss` to workspace clippy lints — zero violations found
- **`Arc<Box<dyn>>` → `Arc<dyn>`**: Eliminated double indirection in `circuit_breaker/breaker.rs` and `plugins/registry.rs` per rhizoCrypt pattern
- **Env-configurable retry**: `StandardRetryPolicy::from_env()` with primal→ecosystem→default chain (`SQUIRREL_RETRY_*` → `IPC_RETRY_*` → defaults) per SweetGrass `RetryPolicy::from_env()` pattern
- **Capability registry**: 24 → 25 methods (added `identity.get` with domain `identity.self`)
- **Niche self-knowledge**: Updated `CAPABILITIES`, `SEMANTIC_MAPPINGS`, `COST_ESTIMATES`, `operation_dependencies()` for `identity.get`
- **Tests**: New tests for identity.get handler, normalize_method (3 cases), health tiering (3 tiers), JSON-RPC validation (5 cases), retry from_env (5 cases)
- **Quality gates**: `fmt` ✓, `clippy --all-features -D warnings` ✓, `check --all-targets --all-features` ✓

### alpha.24 Sprint (Comprehensive Debt Resolution & Sovereignty Evolution)

- **Zero `.unwrap()` workspace-wide**: Eliminated all ~5,600 `.unwrap()` calls across 551 files — Results use `?` propagation, Options use `.expect("invariant message")`, locks use `.expect("lock poisoned")`
- **Zero `panic!()` workspace-wide**: Replaced all 137 `panic!("Expected X")` patterns in tests with `unreachable!()` or proper assertions
- **`Box<dyn Error>` → typed errors**: Replaced in ~15 production APIs across 6 crates — `SquirrelError` in interfaces traits, `PrimalError` in main, `AIError` in ai-tools, `ContextError` in context, `MCPError` in mcp, `EcosystemError` in integration
- **Sovereignty evolution**: `SongbirdClient` → `ServiceMeshHttpClient`, `SongbirdConfig` → `ServiceMeshConfig` with deprecated aliases; primal-specific env vars (`SONGBIRD_*`, `TOADSTOOL_*`, `NESTGATE_*`) emit deprecation warnings when used as fallbacks
- **Port centralization**: Hardcoded `8080`/`8500`/`8081`/`8082` replaced with `universal_constants::network::get_service_port()` calls
- **Mock isolation**: `MockServiceMeshClient` and `MCPAdapter` mock fields gated behind `#[cfg(any(test, feature = "testing"))]`
- **`#[allow]` → `#[expect]` expansion**: 217 `#[expect()]` attributes; remaining 130 `#[allow()]` only where lint is conditional across targets
- **Smart refactoring**: `ecosystem.rs` (1000→799 lines) split into `coordinator.rs` + `ecosystem_types.rs`; `federation/service.rs` (973→732 lines) split into `swarm.rs` + `service_tests.rs`
- **Clone reduction**: `sync/manager.rs` — `HashMap<String, SyncMessage>` → `HashSet<String>` for pending ops; `transport/memory` — conditional history clone; `monitoring/clients` — `Arc<Mutex>` sharing, move-then-insert patterns
- **License alignment**: `AGPL-3.0-only` → `AGPL-3.0-or-later` per wateringHole standard
- **Workspace member**: Added `crates/integration` umbrella to workspace
- **Duplicate config removed**: Removed `rustfmt.toml` (kept `.rustfmt.toml` with SPDX header)
- **Rustdoc clean**: Fixed `private_intra_doc_links` warning on `SecurityRequest`
- **New tests**: service_discovery validate/matches/sort/paginate, transaction edge cases, web integration framework, history formatted, lifecycle no-hooks
- **Files**: 1,331 `.rs` files, 450K total lines
- **Quality gates**: `fmt` ✓, `clippy --all-features -D warnings` ✓, `doc --no-deps` ✓, `test --all-features` ✓ (7,035/0)

### alpha.23 Sprint (Comprehensive Audit, Modern Idiomatic Rust & Coverage Push)

- **Build fully green with `--all-features`**: Fixed 15 compile errors in `squirrel-ai-tools` (missing imports), 12 clippy errors in `ecosystem-api` (missing docs, `use_self`), 123 pedantic clippy errors in `squirrel-core` (unused_async, significant_drop, cast safety, etc.), 3 unfulfilled lint expectations in `squirrel-commands`, 1 dead code in `squirrel-plugins`, 2 errors in `squirrel-ai-tools` (unused import, inefficient clone)
- **Blanket lint suppression eliminated**: Removed 28-lint blanket `#![expect(...)]` from `ai-tools/lib.rs`; every underlying issue fixed with proper per-item docs, `#[must_use]`, `const fn`, removed `unused_async`, proper cast conversions
- **`#[allow]` → `#[expect(reason)]` migration**: Completed across workspace; remaining `#[allow]` only where lint is conditional
- **Primal name centralization**: Raw `"songbird"`/`"toadstool"`/`"beardog"` literals replaced with `universal_constants::primal_names::*` constants across 10+ production files
- **Production `panic!()` eliminated**: `deploy_graph.rs` and `sdk/error/conversions.rs` evolved to proper error returns
- **Hardcoded socket paths evolved**: New `resolve_capability_unix_socket()` in `universal-constants/network.rs` with tiered env-var resolution; `capability_ai.rs`, `delegated_jwt_client.rs`, `security_provider_client.rs` all migrated
- **Clone audit**: 27+ redundant clones eliminated across 5 hot-path files; patterns: `swap_remove`, `Arc::clone`, borrow + `from_ref`, move-then-fetch
- **Large file refactoring**: `federation.rs` → module tree (types.rs + service.rs), `auth.rs` → module tree (discovery.rs + operations.rs + tests.rs), `cli/mcp/mod.rs` → extracted test module
- **Production stubs evolved**: `api.rs` `/info` returns real uptime + federation stats; `/federation/join` calls `SwarmManager`; Phase 2 items documented with proper `#[expect(dead_code, reason)]`
- **82 new tests**: 57 for `squirrel-core` mesh modules (federation, ecosystem, api, routing), 12 for `ai-tools` ipc_routed_providers, 7 for main (router + jsonrpc), 6 for ecosystem-api
- **`rustfmt.toml` added**: edition 2024, max_width 100
- **reqwest verified rustls-only**: All reqwest deps use `default-features = false, features = ["rustls-tls"]`; `deny.toml` bans openssl/ring/native-tls
- **SPDX header**: Fixed 1 missing file (`engine_tests/mod.rs`); all `.rs` files now have AGPL-3.0-or-later header
- **Doctest fixes**: 3 doctests updated for sync `start_heartbeat_loop` signature
- **Migration script cleaned**: `scripts/migrate_allow_to_expect.py` removed (migration complete)
- **Test count**: 6,720 → 7,035 (+315 tests)
- **Coverage**: 85.4% line coverage with full `--all-features` (comprehensive — includes previously untested mesh code)
- **Files**: 1,327 `.rs` files, 447K total lines, max file 1000 lines
- **Quality gates**: `fmt` ✓, `clippy --all-features -D warnings` ✓, `doc --all-features` ✓, `test --all-features` ✓ (7,035/0)

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
