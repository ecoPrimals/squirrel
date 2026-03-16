<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# Squirrel Current Status

**Last Updated**: March 16, 2026
**Version**: 0.1.0-alpha.7
**License**: AGPL-3.0-only (scyBorg: ORC + CC-BY-SA 4.0 for docs)

## Build

| Metric | Value |
|--------|-------|
| Build | GREEN — default features: 0 errors; `--all-features`: 0 errors |
| Tests | 4,819 passing / 0 failed / 140 ignored across 22 crates |
| Edition | 2024 (Rust 1.93.0) |
| Clippy | CLEAN — `pedantic + nursery + deny(unwrap/expect)` on all lib targets |
| Docs | All crates `#![warn(missing_docs)]` |
| Formatting | `cargo fmt --all -- --check` passes |
| Unsafe Code | 0 in production — `#![forbid(unsafe_code)]`; tests use `temp_env` (no `unsafe` env ops) |
| Pure Rust | 100% default features (zero C deps); `ring`/`zstd-sys`/`sysinfo` behind optional features only |
| ecoBin | Compliant — `openssl`/`native-tls` removed from all features; `sysinfo` behind `system-metrics` |
| Coverage | 69% line coverage via `cargo-llvm-cov` (target: 90%) |
| Crates | 22 workspace members |
| Files >1000 lines | 0 (enhanced/mod.rs refactored 992→701; benchmarking/mod.rs 988→477) |
| Property tests | 10 (proptest round-trip for all JSON-RPC types + niche) |

## JSON-RPC Methods

Source of truth: [`capability_registry.toml`](capability_registry.toml)

| Domain | Methods |
|--------|---------|
| AI | `ai.query`, `ai.list_providers`, `ai.complete`, `ai.chat` |
| Capability | `capability.announce`, `capability.discover`, **`capability.list`** |
| Context | `context.create`, `context.update`, `context.summarize` |
| System | `system.health`, `system.status`, `system.metrics`, `system.ping` |
| Discovery | `discovery.peers` |
| Tool | `tool.execute`, `tool.list` |
| Lifecycle | `lifecycle.register`, `lifecycle.status` |

**JSON-RPC batch support**: Full Section 6 compliance — array of requests → array of responses.

## tarpc Service

All JSON-RPC methods mirrored as tarpc service methods with typed request/response
structs. `TarpcRpcServer` delegates to `JsonRpcServer` for shared handler logic.
Protocol negotiation selects tarpc or JSON-RPC per-connection.

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

`capability.list` returns per-method cost/dependency detail for PathwayLearner scheduling.

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
| `Arc<str>` for identifiers | `jsonrpc_handlers.rs` (`AnnouncedPrimal`), `self_knowledge.rs` (capabilities) |
| `Arc<dyn ValidationRule>` | `validation.rs` — eliminates `Box::new(self.clone())` |
| `bytes::Bytes` for payloads | `transport/frame.rs` — O(1) clone on frame data |
| `&'static str` for constants | `self_knowledge.rs` — default capabilities |

## Error Handling

| Crate | Error Type | Pattern |
|-------|-----------|---------|
| `squirrel-commands` | `CommandError` (thiserror) | Typed variants: Io, Serialization, Validation, Hook, Lifecycle, etc. |
| `squirrel-cli` | `FormatterError` (thiserror) | Serialization, UnknownFormat |
| `squirrel-mcp` | `MCPError` (thiserror) | Protocol, transport, context, plugin errors |
| `universal-error` | `UniversalError` | Cross-crate error type |

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

## Socket Configuration

Injectable `SocketConfig` pattern (absorbed from airSpring):

```
Tier 1: SQUIRREL_SOCKET (primal-specific override)
Tier 2: BIOMEOS_SOCKET_PATH (Neural API orchestration)
Tier 3: PRIMAL_SOCKET + family suffix
Tier 4: XDG runtime: /run/user/<uid>/biomeos/squirrel.sock
Tier 5: /tmp/squirrel-<family>-<node>.sock (dev only)
```

All tiers testable via `SocketConfig` DI without `temp_env` or `#[serial]`.

## Tooling

| Tool | Config |
|------|--------|
| rustfmt | `.rustfmt.toml` — edition 2024, max_width 100 |
| clippy | `clippy.toml` — pedantic + nursery + deny(unwrap/expect) via `[workspace.lints.clippy]` |
| cargo-deny | `deny.toml` — license allowlist, advisory audit, ban wildcards |
| cargo-llvm-cov | Installed, 69% line coverage measured |
| proptest | Round-trip invariants for all JSON-RPC types |

## Known Issues

1. `test_load_from_json_file` flaky under full workspace runs (env var pollution) — needs `#[serial]`
2. `chaos_07_memory_pressure` flaky under parallel test load (environment-sensitive)
3. `model_splitting/` stub module — waiting on ToadStool integration
4. 140 ignored tests — doc-tests for `universal-patterns` examples requiring runtime services
5. Coverage at 69% — gap to 90% target (~27K uncovered lines remaining)
6. `redis` v0.23.3 will be rejected by future Rust — upgrade needed
