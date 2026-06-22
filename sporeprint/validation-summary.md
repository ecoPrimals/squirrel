+++
title = "squirrel Validation Summary"
description = "AI inference routing, context management, capability discovery, signal composition, provenance proxy. 7,539+ tests, 42+ IPC methods, 90% coverage."
date = 2026-06-22

[taxonomies]
primals = ["squirrel"]
springs = []
+++

## Status

- **Gate**: CLEAR (stadial readiness confirmed May 17, 2026)
- **Phase**: 3 (BTSP Phase 3 AEAD encrypted framing)
- **Edition**: 2024 (Rust 1.94+)
- **Tests**: 7,539 passing across 22 workspace crates
- **Source**: ~1,035 `.rs` files, ~328k lines
- **Clippy**: 0 warnings (`pedantic` + `nursery` + `cargo`, `-D warnings`, `--all-features`)
- **Docs**: 0 warnings (`-D warnings`)
- **deny.toml**: ring, openssl, reqwest, native-tls, aws-lc-sys all banned; pure Rust enforced
- **Coverage**: 90.14% region / 89.67% line (cargo-llvm-cov)
- **Binary**: 3.5 MB static-pie musl, stripped, BLAKE3 checksummed, zero host paths
- **Transport**: Full Phase 2 — `TRANSPORT_ENDPOINT` accepted + `connect_transport()` for all outbound IPC + Eukaryotic riboCipher: MitoBeacon (`0xEC`/`0xED`) accepted + outbound `[0xEC, 0x01]` preamble on all UDS
- **HTTP IPC**: Raw TCP JSON-RPC delegation (zero external HTTP deps, uniBin compliant)
- **Files >800L (prod)**: 0 — `jsonrpc_server.rs` split (829L → 339L server + 474L connection handler); `env_vars.rs` refactored to module tree
- **Hardcoding**: Evolved — 14 production files migrated from literal localhost/ports to capability-based discovery
- **TRUE PRIMAL**: `niche::REQUIRED_CAPABILITIES` replaces named-primal `DEPENDENCIES`; `capability_id` field on `EcosystemServiceRegistration`; `EcosystemPrimalType` production uses annotated `#[expect(deprecated)]`
- **Metrics**: Real `/proc` reads (CPU, memory, disk I/O, network I/O) replace simulated values; `RequestTracker` unified between `JsonRpcServer` and `MetricsCollector` — single `Arc` shared at startup. `context_state.active_sessions` live from `ContextManager`. Dead helpers (`get_cpu_usage`, `get_memory_usage`, `get_memory_percentage`) wired, `#[expect(dead_code)]` removed.
- **Security Health**: Capability-discovery probe replaces simulated endpoint check
- **BTSP Phase 3 Transport Switch**: Server auto-transitions to encrypted frame loop after `btsp.negotiate` with `chacha20-poly1305`; 3 integration tests on live Unix socket pairs (previously orphaned, now wired)
- **Provenance Proxy**: `dag.*`, `anchoring.*`, `attribution.*`, `provenance.*` methods routed to discovered primals via capability-based socket discovery; `forward_jsonrpc` E2E-tested with mock UDS round-trips (happy path, remote error, invalid JSON, missing result)
- **Context Persistence**: Shared `ContextManager` on `JsonRpcServer` — `context.create` → `context.update` → `context.summarize` persists across requests; session count synced to `MetricsCollector`
- **tarpc Parity**: `provider.*` and `btsp.negotiate` tarpc stubs delegated to JSON-RPC handlers (mirrors lifecycle pattern)
- **Identity**: Single canonical source (`universal_constants::capabilities::SELF_PRIMAL_NAME`); `niche::PRIMAL_ID` and `core::PRIMAL_TYPE` are re-exports. Zero hardcoded self-identity string literals in production.
- **Feature gating**: Context learning subsystem (~14.6k lines, 625 tests) behind `context-learning` feature. Default build: 6,914 tests; `--all-features`: 7,539 tests.
- **Nuclear Lineage (0xEE)**: Protocol-aware; NDJSON clients receive JSON-RPC -32050 with `resolution:"awaiting_beardog_keys"`; BTSP closes silently. Full encrypted channel awaits BearDog key material.
- **Discovery**: Socket registry is canonical for LAN. DNS-SD and mDNS stubs documented; fallback paths tested. Ready for `discovery-mdns` feature flag with hickory-dns.
- **Security middleware**: `SecurityOrchestrator` wired as pre-dispatch middleware — rate limiting, input validation, and threat detection active when orchestrator attached. Method prefix → `EndpointType` tiering; denied requests receive JSON-RPC `-32003`.
- **Constraint routing**: `ai.query` now parses routing constraints from raw request params (`privacy_level`, `cost_preference`, `quality`, `speed_preference`, `constraints[]`) and feeds them to `select_provider_with_constraints`.
- **Feature gating (hygiene)**: Vestigial `capability-ai` feature removed; `benchmarking` module gated behind its feature; defaults trimmed to `["ecosystem", "tarpc-rpc"]`.
- **Dead-code attrs narrowed**: 5 module-level `#![expect(dead_code)]` replaced with targeted per-item `#[expect(dead_code, reason)]` where code IS wired but specific fields/variants await downstream consumers.
- **Lint policy**: `clippy::expect_used` + `clippy::unwrap_used` = `deny` workspace-wide (evolved from `warn`); zero `#[allow(` remaining (all converted to `#[expect(reason)]`); zero unfulfilled lint expectations
- **CI**: `fmt` + `clippy -D warnings` + `test` + `cargo deny check` (supply-chain audit added)
- **Dignity**: Configurable enforcement (`SQUIRREL_DIGNITY_ENFORCEMENT`: warn/enforce/audit)
- **AuthService**: Complete standalone implementation (was missing module; now compiles under `--all-features`)

## Capabilities

| Capability | Description |
|-----------|-------------|
| `inference` | Multi-provider AI inference routing (complete, embed, models) |
| `context` | Session context creation, update, and summarization |
| `discovery` | Capability-based peer discovery (zero hardcoded names) |
| `signal` | Neural API composition collapse (signal.plan) |
| `tool` | Plugin tool execution and listing |
| `health` | Standard health triad (liveness, readiness, check) |
| `btsp` | Phase 3 cipher negotiation + encrypted framing |
| `graph` | Dependency graph parsing and validation (primalSpring BYOB) |
| `lifecycle` | biomeOS lifecycle registration + heartbeat |
| `provider` | Spring provider registration/deregistration (LIVE — Wave 116) |
| `provenance` | Proxy layer for DAG/anchoring/attribution routing to discovered primals |

## Methods (42 registered + dynamic provenance proxy)

- `inference.complete`, `inference.embed`, `inference.models`, `inference.register_provider`, `inference.unregister_provider`
- `ai.query`, `ai.list_providers`, `ai.complete`, `ai.chat`
- `signal.plan`
- `capabilities.list`, `capabilities.announce`, `capabilities.discover`, `capability.announce`, `capability.discover`, `capability.list`, `primal.announce`, `primal.capabilities`
- `identity.get`
- `context.create`, `context.update`, `context.summarize`
- `system.metrics`, `system.health`, `system.status`, `system.ping`
- `health` (bare — Wave 113), `health.check`, `health.liveness`, `health.readiness`
- `discovery.peers`, `discovery.list`
- `tool.execute`, `tool.list`
- `provider.register`, `provider.list`, `provider.deregister`
- `btsp.negotiate`
- `lifecycle.register`, `lifecycle.status`
- `graph.parse`, `graph.validate`
- `provenance.*`, `dag.*`, `anchoring.*`, `attribution.*` (dynamic proxy → discovered primals)

## Composition Role

Squirrel is the **intelligence router** for all compositions requiring AI inference:
- Meta-tier member (biomeOS + squirrel + petalTongue)
- Provides inference routing to any composition needing LLM/embedding access
- Delegates compute to neuralSpring providers, routes via capability discovery
- Context management for multi-turn conversations across compositions
- Human dignity evaluation with configurable enforcement

## Downstream Pairing

- esotericWebb (inference consumer — web UI)
- projectFOUNDATION (inference consumer — code generation)
- neuralSpring (inference provider — model hosting)
- primalSpring (graph validation, coordination)
- wetSpring (sovereign pipeline — inference for Barrick clone)
- NestGate (model weight storage)

## Degradation

When squirrel is down: AI inference unavailable, context operations fail.
Other primals continue operating — squirrel is intelligence routing, not a gate.
Discovery, health, lifecycle registrations degrade gracefully (standalone mode).
