<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# Squirrel Current Status

**Last Updated**: March 17, 2026
**Version**: 0.1.0-alpha.10
**License**: AGPL-3.0-only (scyBorg: ORC + CC-BY-SA 4.0 for docs)

## Build

| Metric | Value |
|--------|-------|
| Build | GREEN — default features: 0 errors; `--all-features`: 0 errors |
| Tests | 5,228 passing / 0 failed across 21 crates |
| Edition | 2024 (Rust 1.93.0) |
| Clippy | CLEAN — `pedantic + nursery + deny(unwrap/expect)` on `--all-features --all-targets`; zero warnings |
| Docs | All crates `#![warn(missing_docs)]`; `doc_markdown` clean |
| Formatting | `cargo fmt --all -- --check` passes |
| Unsafe Code | 0 in production — `#![forbid(unsafe_code)]` in all crate entry points |
| Pure Rust | 100% default features (zero C deps); `ring`/`openssl` banned in `deny.toml` |
| ecoBin | Compliant — `deny.toml` explicitly bans `ring` and `openssl`; `sysinfo` behind `system-metrics` |
| Coverage | 67.6% line coverage via `cargo-llvm-cov` (target: 90%) |
| Crates | 21 workspace members |
| Files >1000 lines | 0 (max: 991 — `router.rs`, likely dead code pending review) |
| Property tests | 17 (proptest round-trip for all JSON-RPC types + niche + 7 wire-format fuzz) |
| Mocks in production | 0 — `InMemoryMonitoringClient` documented as intentional fallback; all test mocks behind `#[cfg(test)]` |
| Legacy aliases | Removed — only semantic `{domain}.{verb}` method names accepted |

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

**JSON-RPC batch support**: Full Section 6 compliance — array of requests → array of responses.

**Legacy aliases removed**: Flat names (`query_ai`, `health`, `ping`, etc.) no longer
accepted. All clients must use the semantic `{domain}.{verb}` names above.

## tarpc Service

tarpc 0.37 (upgraded from 0.34). All JSON-RPC methods mirrored as tarpc service
methods with typed request/response structs. `TarpcRpcServer` delegates to
`JsonRpcServer` for shared handler logic. Protocol negotiation selects tarpc or
JSON-RPC per-connection.

## Niche Self-Knowledge (`niche.rs`)

Follows the groundSpring/wetSpring/airSpring niche pattern:

| Constant | What |
|----------|------|
| `CAPABILITIES` | 21 exposed methods (ai, capability, system, discovery, tool, context, lifecycle) |
| `CONSUMED_CAPABILITIES` | 14 external capabilities from BearDog, Songbird, ToadStool, NestGate |
| `COST_ESTIMATES` | Per-method latency and GPU hints for Pathway Learner scheduling |
| `DEPENDENCIES` | 4 primals (beardog, songbird required; toadstool, nestgate optional) |
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
| `JWT_ISSUER` | `"squirrel-mcp"` | JWT token `iss` claim |
| `JWT_AUDIENCE` | `"squirrel-mcp-api"` | JWT token `aud` claim |
| `JWT_SIGNING_KEY_ID` | `"squirrel-jwt-signing-key"` | BearDog key lookup |

Runtime discovery uses capabilities, not primal names. Names are only for socket
file naming conventions and logging.

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
| `system-metrics` | sysinfo (C dependency) | OFF |
| `monitoring` | Prometheus metrics (brings hyper) | OFF |
| `nvml` | NVIDIA GPU detection via nvml-wrapper | OFF |
| `local-jwt` | Local JWT signing (brings ring C dep) | OFF |

## Zero-Copy Patterns

| Pattern | Where |
|---------|-------|
| `Arc<str>` for identifiers | `jsonrpc_handlers.rs` (`AnnouncedPrimal`), `self_knowledge.rs`, `EcosystemServiceRegistration` |
| `Arc<dyn ValidationRule>` | `validation.rs` — eliminates `Box::new(self.clone())` |
| `bytes::Bytes` for payloads | `transport/frame.rs` — O(1) clone on frame data |
| `&'static str` for constants | `self_knowledge.rs` — default capabilities |
| Struct update syntax | Builder patterns use `..Default::default()` throughout |

## Error Handling

| Crate | Error Type | Pattern |
|-------|-----------|---------|
| `squirrel-commands` | `CommandError` (thiserror) | Typed variants: Io, Serialization, Validation, Hook, Lifecycle, etc. |
| `squirrel-cli` | `FormatterError` (thiserror) | Serialization, UnknownFormat |
| `squirrel-mcp` | `MCPError` (thiserror) | Protocol, transport, context, plugin errors |
| `universal-error` | `UniversalError` | Cross-crate error type |
| `universal-patterns` | `IpcClientError` + `IpcErrorPhase` | Phase-tagged IPC errors (Connect, Write, Read, JsonRpcError, NoResult) with `is_retryable()` — absorbed from rhizoCrypt v0.13 |
| `universal-patterns` | `DispatchOutcome<T>` | Protocol vs application error separation at RPC dispatch — absorbed from groundSpring/loamSpine/sweetGrass |
| `universal-patterns` | `CircuitBreaker` + `RetryPolicy` | IPC resilience with exponential backoff gated by `IpcErrorPhase` — absorbed from petalTongue v1.6.6 |
| `universal-patterns` | `RpcError` + `extract_rpc_error()` | Structured JSON-RPC error extraction — absorbed from loamSpine/petalTongue |

## Logging

Production code uses `tracing` (`info!`, `warn!`, `error!`, `debug!`).
`println!` reserved for CLI user-facing output and startup banner only.

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
| Primal Names | `primal_names::*` constants for all 13 ecosystem primals |

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
| rustfmt | `.rustfmt.toml` — edition 2024, max_width 100 |
| clippy | `clippy.toml` — pedantic + nursery + deny(unwrap/expect) via `[workspace.lints.clippy]` |
| cargo-deny | `deny.toml` — license allowlist, advisory audit, ban wildcards, deny yanked |
| cargo-llvm-cov | Installed, 67.6% line coverage measured |
| proptest | Round-trip + wire-format fuzz for all JSON-RPC types (17 properties) |

## Known Issues

1. `test_load_from_json_file` flaky under full workspace runs (env var pollution) — needs `#[serial]`
2. `chaos_07_memory_pressure` flaky under parallel test load (environment-sensitive)
3. `model_splitting/` redirect stub — functionality moved to ToadStool; module retained as navigation aid
4. `unified_manager` — Phase 2 placeholder for unified plugin system
5. Coverage at 67.6% — gap to 90% target; incremental expansion underway
6. `redis` v0.23 behind optional `persistence` feature — upgrade to 0.25+ when ecosystem stabilizes
7. `router.rs` (991 lines) — likely dead code, pending investigation before refactoring or removal

## Changes Since Last Handoff (March 17, 2026)

### Deep Audit & Evolution Sprint

- **Clippy**: Full `--all-features --all-targets -D warnings` pass — zero warnings across all 21 crates
- **Formatting**: `cargo fmt` clean across entire workspace
- **Auth tests**: Complete rewrite of `auth_tests.rs` to align with current `squirrel_mcp_auth` API
- **Doctests**: Fixed all doctest failures in `squirrel-core` service discovery; marked malformed WASM example as `ignore`
- **Smart refactoring**: `performance_optimizer.rs` (996L → 10 focused modules); `ecosystem/mod.rs` (985L → 4 files)
- **Hardcoding evolution**: All ports now environment-overridable via `get_port()` helpers; capability-based delegation replaces hardcoded primal names in `send_to_primal` and `delegate_to_songbird`
- **deny.toml**: Explicit bans for `ring` and `openssl` with documented Pure Rust alternatives
- **Float comparisons**: Epsilon-based assertions replace `assert_eq!` on floats
- **Struct initialization**: Struct update syntax replaces `Default::default()` + field reassignment
- **Doc quality**: `doc_markdown` backtick compliance across all crates; wildcard imports replaced with explicit imports
- **Stubs evolved**: `execute_capability` now sends JSON-RPC over Unix sockets; `send_to_primal` discovers endpoints via capability registry
