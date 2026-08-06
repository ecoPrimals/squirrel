<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# Squirrel Current Status

**Last Updated**: Aug 6, 2026 (Wave 156r — PluginMetadata Migration + Deprecated Enum Deletion)
**Version**: 0.1.0
**License**: AGPL-3.0-or-later (scyBorg: ORC + CC-BY-SA 4.0 for docs)

> **Live metrics**: See [CHANGELOG.md](CHANGELOG.md) `[Unreleased]` and [sporeprint/validation-summary.md](sporeprint/validation-summary.md) for authoritative test counts and gate status.

## Build

| Metric | Value |
|--------|-------|
| Build | GREEN — default features: 0 errors; `--all-features`: 0 errors, **0 warnings** |
| Tests | **6,366** passing / 0 failures across 16 workspace crates (default features) |
| Edition | 2024 (Rust 1.94+) |
| async-trait | **0 usage** — all `#[async_trait]` annotations removed; dyn-safe traits use explicit `Pin<Box<dyn Future>>`, non-dyn traits use native `async fn`; `async-trait` only remains as transitive dep from external crate `config` |
| Clippy | CLEAN — `pedantic + nursery + cargo`, `expect_used/unwrap_used = deny` workspace-wide; zero warnings under `-D warnings` |
| Docs | All crates `#![warn(missing_docs)]`; `cargo doc --no-deps` clean |
| Formatting | `cargo fmt --all -- --check` passes |
| Unsafe Code | 0 in production — `unsafe_code = "forbid"` in workspace `[lints.rust]` (all 16 crates) |
| Pure Rust | 100% default features **and** `--all-features` (zero C deps, zero non-Rust crypto); 14 C-dep crates banned in `deny.toml`; `sysinfo` removed; `ed25519-dalek` feature-gated behind `local-crypto`; `blake3` → `features = ["pure"]` (no SIMD assembly); `pprof`, `openai`, `libloading`, `nvml-wrapper` removed; `nix` → `rustix` (pure Rust syscalls); `rand` upgraded 0.8→0.9.4 (RUSTSEC-2026-0097); `ring`/`reqwest`/`jsonwebtoken`/`rustls` **ELIMINATED** from Cargo.lock (stadial gate); `zstd`/`flate2`/`lz4_flex` **ELIMINATED** from Cargo.lock (compression feature emptied: `CompressionFormat` is metadata-only, no codec wired) |
| ecoBin | Compliant v3.0 — 4.4 MB static-pie musl binary, stripped, BLAKE3 checksummed, zero host paths (`--remap-path-prefix`), zero dynamic deps; `deny.toml` bans 14 C-dep crates + `tokio-tungstenite` (Tower Atomic) + `reqwest` (Tower Atomic); pure Rust `sys_info` via `/proc` parsing |
| Coverage | **90.1%** region coverage / 89.6% line coverage via `cargo-llvm-cov` (**target met**); remaining uncovered: binary entry points, demo bins, WASM-only SDK paths, live IPC server loops |
| `.unwrap()` in code | 0 — workspace-wide elimination; all Results use `?` or `.expect("invariant")` |
| `panic!()` in code | 0 — replaced with `unreachable!()` or proper assertions |
| `Box<dyn Error>` | 0 in production APIs — replaced with typed errors + `anyhow::Result` (`PrimalError`, `AIError`, `SquirrelError`, `ContextError`, `MCPError`, `EcosystemError`, `anyhow::Error`) |
| Crates | 16 workspace members |
| Files >800 lines (prod) | 0 — `compute_client/types.rs` split (788→482L); `federation/service.rs` split (784→449L); `universal_executor.rs` split (794→633L); `routing/agent.rs` split (794→479L); `jsonrpc_server.rs` split (829→336L); `provider_trait.rs` refactored 983→728L; `env_vars.rs` (1091L) → module tree (36 files, max 107L); visualization thinned (~1,800L removed); largest prod file: 777L |
| `#[expect(reason)]` | Workspace migrated from `#[allow]` to `#[expect(reason)]`; 1 `#[allow]` remains with documented reason: 1 `dead_code` on trait method (Rust 1.94: `#[expect]` unfulfilled in check but needed under test compilation); `PrimalDependency.primal_type` evolved to `String` — no deprecated field, no `#[allow(deprecated)]` needed; dead suppressions caught automatically |
| Cargo metadata | All crates have `repository`, `readme`, `keywords`, `categories`, `description` — zero `clippy::cargo` warnings |
| Property tests | 23 proptest properties + 2 TOML sync + identity invariant tests + Unix socket IPC tests |
| cargo deny | `advisories ok, bans ok, licenses ok, sources ok` |
| Mocks in production | 0 — all production stubs evolved to honest capability-based patterns: `SecurePluginStub` rejects execution (security sandbox, documented); `NoOpPluginManager` returns errors; plugin web API returns 501 (Phase 2); `WebVisualizationServer` logs capability-pending; `UnavailableServiceRegistry` returns empty (honest); learning integration wired to live `ContextManager` data; neural engine evolved from tanh stub to ReLU MLP; federation `dead_code` fields wired to real diagnostics; all test mocks behind `#[cfg(any(test, feature = "testing"))]` |
| Primal self-knowledge | All hardcoded primal names evolved to capability-based: `BearDog*` → `SecurityProvider*`, `Songbird*` → `Discovery*`/`ServiceMesh*`, `NestGate` → `ContentAddressed`; deprecated type aliases for backward compat; env var chains prefer capability names (`SECURITY_ENDPOINT` → `BEARDOG_ENDPOINT` fallback); `SECURITY_SERVICE_ID` / `SECURITY_PRIMARY_SERVICE_ID` constants replace all `format!("{}-security", primal_names::BEARDOG)` calls |
| Legacy aliases | Backward-compatible aliases for ecosystem compat; `capabilities.list` canonical per SEMANTIC_METHOD_NAMING_STANDARD v2.1 |
| TODO/FIXME in code | 0 — no TODO/FIXME/HACK markers in committed code; Phase 2 placeholders wired with capability fallback or documented with `#[expect(dead_code, reason)]` (5 false positives: `TaskStatus::Todo` enum variant, `sk-xxx` test fixtures) |
| Dev credentials | 0 hardcoded — all via env vars (`SQUIRREL_DEV_JWT_SECRET`, `SQUIRREL_DEV_API_KEY`) |
| Zero-copy | Hot-path clones audited; `ServiceInfo` string fields evolved to `Arc<str>`; `Arc::clone()` for intent clarity; `mem::take` for payload moves; `String` → borrow in MCP task client |

## JSON-RPC Methods

Source of truth: [`config/capability_registry.toml`](config/capability_registry.toml)

| Domain | Methods |
|--------|---------|
| Inference | **`inference.complete`**, **`inference.embed`**, **`inference.models`**, **`inference.register_provider`**, **`inference.unregister_provider`** (canonical per SEMANTIC_METHOD_NAMING_STANDARD v2.0 §7) |
| AI | `ai.query`, `ai.list_providers`, `ai.complete`, `ai.chat` (backward-compat aliases → `inference.*` handlers) |
| Signal | `signal.plan` (Neural API composition collapse) |
| Capability | **`capabilities.list`** (canonical), `capabilities.announce`, `capabilities.discover`, `capability.announce` (alias), `capability.discover` (alias), `capability.list` (alias), `primal.announce` (stadial standard), `primal.capabilities` (alias) |
| Identity | `identity.get` (CAPABILITY_BASED_DISCOVERY_STANDARD v1.0) |
| Context | `context.create`, `context.update`, `context.summarize` |
| System | **`system.metrics`** (canonical), `system.health`, `system.status`, `system.ping` (backward-compat aliases) |
| Health | **`health.check`**, **`health.liveness`**, **`health.readiness`** (canonical — PRIMAL_IPC_PROTOCOL v3.0) |
| Discovery | `discovery.peers`, `discovery.list` (alias) |
| Tool | `tool.execute`, `tool.list` |
| BTSP | `btsp.negotiate` (Phase 3 FULL: encrypted framing + key derivation) |
| Lifecycle | `lifecycle.register`, `lifecycle.status` |
| Graph | `graph.parse`, `graph.validate` (primalSpring BYOB) |
| Provider | `provider.register`, `provider.list`, `provider.deregister` (spring registration) |
| Provenance | `provenance.*`, `dag.*`, `anchoring.*`, `attribution.*` (dynamic proxy → discovered primals) |

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
| `CAPABILITIES` | 35 exposed methods (inference, ai, capabilities, capability, identity, system, health, discovery, tool, context, provider, btsp, lifecycle, graph) |
| `CONSUMED_CAPABILITIES` | 32 external capabilities from security, service-mesh, compute, content-storage providers, domain springs, rhizoCrypt, sweetGrass, primalSpring |
| `COST_ESTIMATES` | Per-method latency and GPU hints for Pathway Learner scheduling |
| `DEPENDENCIES` | 6 primals (security-provider, service-mesh required; compute, content-storage, primalspring, petaltongue optional) |
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
| `JWT_SIGNING_KEY_ID` | `"squirrel-jwt-signing-key"` | Security provider key lookup |

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
| Discovery service | `ipc.register` + 30s heartbeat | Active (when discovery socket detected) |

## Orchestration

`DeploymentGraphDef` types (from ludoSpring exp054) absorbed for multi-primal
composition awareness. Squirrel can parse deployment graphs and identify nodes
requiring AI capabilities.

## Feature Gates

| Feature | What it gates | Default |
|---------|---------------|---------|
| `tarpc-rpc` | High-performance binary RPC via tarpc | ON |
| `delegated-jwt` | Capability-based JWT delegation | ON |
| `monitoring` | Prometheus metrics (brings hyper) | OFF |
| `benchmarking` | Criterion benchmark harness | OFF |
| `context-learning` | Context learning subsystem (~14.6k lines, 625 tests) | OFF |

> **Removed features (fossil record)**: `capability-ai` (Wave 124 — always-on, absorbed into core), `ecosystem` (Wave 128 — always-on, absorbed), `deprecated-adapters` (Wave 128 — vendor adapters deleted).

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
| Capability Registry | `config/capability_registry.toml` loaded at startup |
| Niche Self-Knowledge | `niche.rs` with capabilities, costs, deps, consumed capabilities |
| Primal Identity | `universal-constants::identity` — centralized JWT/primal constants |
| Deploy Graph | `squirrel_deploy.toml` (BYOB pattern) |
| Orchestration Types | `DeploymentGraphDef`, `GraphNode`, `TickConfig` (ludoSpring wire-compatible) |
| biomeOS Lifecycle | `lifecycle.register` + 30s heartbeat (when orchestrator detected) |
| Discovery Service | `ipc.register` + 30s heartbeat (when discovery socket detected) |
| Security Provider Crypto | Discovery via capability-based biomeOS socket scan |
| Compute Provider AI | Auto-discovered via capability-based biomeOS socket scan |
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
per `PRIMAL_IPC_PROTOCOL.md` (ecosystem spec, see `ecoPrimals/infra/wateringHole/`):

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
| cargo-llvm-cov | **90.1%** region coverage / 89.6% line coverage (**target met**) |
| proptest | Round-trip + wire-format fuzz + IPC fuzz for all JSON-RPC types (23 properties) + Unix socket IPC tests |
| rust-toolchain | `rust-toolchain.toml` — pinned stable + clippy + rustfmt + llvm-tools-preview |

## Known Issues

1. **Coverage target met** — 90.1% region coverage (89.6% line). Remaining uncovered: binary entry points, demo binaries, WASM-only SDK paths, live IPC server loops. All production modules have test coverage.
2. Performance optimizer `batch_processor` / `optimizer` are complete (no deferred stubs)
3. `base64` duplicate (0.21 via `config`/`ron`, 0.22 direct) — transitive, benign
4. `async-trait` — **0 annotations** in Squirrel code (migrated from 228 → 0); dyn-safe traits use `Pin<Box<dyn Future>>`, non-dyn traits use native `async fn in trait`; `async-trait` remains only as transitive dep from external crate `config`

---

> **Session history (Wave 107–129)** archived to `ecoPrimals/fossilRecord/squirrel_current_status_wave129/`

