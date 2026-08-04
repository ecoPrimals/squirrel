+++
title = "squirrel Validation Summary"
description = "AI inference routing, context management, capability discovery, signal graph dispatch, provenance proxy. 7,142 tests (--all-features), 44 IPC methods."
date = 2026-08-04

[taxonomies]
primals = ["squirrel"]
springs = []
+++

## Wave 156d — Sovereignty + Logging Hygiene + Test Isolation (Aug 4, 2026)

- Hardcoded primal names ("BearDog", "Songbird") removed from production error/status messages — replaced with capability-based language
- Inline `/biomeos/` path constructions centralized to `BIOMEOS_SOCKET_SUBDIR` / `get_socket_dir()` (5 files)
- Security client stubs evolved: `apply_ai_security_routing()` and `get_ai_security_insights()` now provider-count-aware
- `#[cfg(test)] pub` field removed from `ModelRegistry` — now private with test-only accessor
- Test isolation: `monitoring_service_provider_trait_methods` race condition fixed via env isolation
- 260 emoji removed from log macros across 33 production files (grep-friendly tracing)
- 7,142 tests passing, 0 warnings

## Status

- **Gate**: CLEAR (stadial readiness confirmed May 17, 2026)
- **Phase**: 3 (BTSP Phase 3 AEAD encrypted framing)
- **Edition**: 2024 (Rust 1.94+)
- **Tests**: **7,142** passing / 5 ignored across 16 workspace crates (`--all-features`), full suite ~60s
- **Source**: 986 `.rs` files, ~306k lines
- **Clippy**: 0 warnings (`pedantic` + `nursery` + `cargo`, `-D warnings`, `--all-features`)
- **Docs**: 0 warnings (`-D warnings`)
- **deny.toml**: ring, openssl, reqwest, native-tls, aws-lc-sys all banned; pure Rust enforced
- **Coverage**: 90.14% region / 89.67% line (cargo-llvm-cov)
- **Binary**: 4.4 MB static-pie musl, stripped, BLAKE3 checksummed, zero host paths
- **Transport**: Full Phase 2 — `TRANSPORT_ENDPOINT` accepted + `connect_transport()` for all outbound IPC + Eukaryotic riboCipher: MitoBeacon (`0xEC`/`0xED`) accepted + outbound `[0xEC, 0x01]` preamble on all UDS
- **HTTP IPC**: Raw TCP JSON-RPC delegation (zero external HTTP deps, uniBin compliant)
- **Files >800L (prod)**: 0 — all production files under 800 lines
- **Hardcoding**: Evolved — 14 production files migrated from literal localhost/ports to capability-based discovery
- **TRUE PRIMAL**: `niche::REQUIRED_CAPABILITIES` replaces named-primal `DEPENDENCIES`; `capability_id` field on `EcosystemServiceRegistration`; `EcosystemPrimalType` production uses annotated `#[expect(deprecated)]`
- **Metrics**: Real `/proc` reads (CPU, memory, disk I/O, network I/O) replace simulated values; `RequestTracker` unified between `JsonRpcServer` and `MetricsCollector` — single `Arc` shared at startup. `context_state.active_sessions` live from `ContextManager`. Dead helpers (`get_cpu_usage`, `get_memory_usage`, `get_memory_percentage`) wired, `#[expect(dead_code)]` removed.
- **Security Health**: Capability-discovery probe replaces simulated endpoint check
- **BTSP Phase 3 Transport Switch**: Server auto-transitions to encrypted frame loop after `btsp.negotiate` with `chacha20-poly1305`; 3 integration tests on live Unix socket pairs (previously orphaned, now wired)
- **Provenance Proxy**: `dag.*`, `anchoring.*`, `attribution.*`, `provenance.*` methods routed to discovered primals via capability-based socket discovery; `forward_jsonrpc` E2E-tested with mock UDS round-trips (happy path, remote error, invalid JSON, missing result)
- **Context Persistence**: Shared `ContextManager` on `JsonRpcServer` — `context.create` → `context.update` → `context.summarize` persists across requests; session count synced to `MetricsCollector`
- **tarpc Parity**: `provider.*` and `btsp.negotiate` tarpc stubs delegated to JSON-RPC handlers (mirrors lifecycle pattern)
- **Identity**: Single canonical source (`universal_constants::capabilities::SELF_PRIMAL_NAME`); `niche::PRIMAL_ID` and `core::PRIMAL_TYPE` are re-exports. Zero hardcoded self-identity string literals in production.
- **Feature gating**: Context learning subsystem (~14.6k lines, 625 tests) behind `context-learning` feature. Context visualization (~3.1k lines) behind `context-visualization`.
- **SecretStore**: `InMemorySecretStore` (dev), `FileSecretStore` (explicit path), `PlatformSecretStore` (OS-native cache path), `SecurityProvider` (security capability IPC — production authority). Native credential stores are the security provider's domain; squirrel caches, security provider stores.
- **Nuclear Lineage (0xEE)**: Protocol-aware; NDJSON clients receive JSON-RPC -32050 with `resolution:"awaiting_security_keys"`; BTSP closes silently. Full encrypted channel awaits security provider key material.
- **Discovery**: Socket registry is canonical for LAN. DNS-SD and mDNS announce/register return explicit `MechanismFailed` errors (no more silent no-ops); discovery falls back to socket registry. Ready for `discovery-mdns` feature flag with hickory-dns.
- **Security middleware**: `SecurityOrchestrator` wired as pre-dispatch middleware — rate limiting, input validation, and threat detection active when orchestrator attached. Method prefix → `EndpointType` tiering; denied requests receive JSON-RPC `-32003`.
- **Constraint routing**: `ai.query` now parses routing constraints from raw request params (`privacy_level`, `cost_preference`, `quality`, `speed_preference`, `constraints[]`) and feeds them to `select_provider_with_constraints`.
- **Feature gating (hygiene)**: Vestigial `capability-ai`, `ecosystem`, and `deprecated-adapters` features removed; `benchmarking` module gated behind its feature; defaults trimmed to `["tarpc-rpc"]`.
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

## Wave 156b — Test Performance: 400s → 16s (Aug 3, 2026)

- Root cause: unit tests probing live Unix sockets with 10s timeouts per capability (40 caps × 10s = 400s)
- `discovery.rs`, `discovery_error_tests.rs`: replaced live `discover_services` calls with instant `perform_service_discovery` registry population
- `defense_client.rs`: `without_discovery()` constructor skips socket probing in tests
- `security/health.rs`: `without_discovery()` skips `check_discovered_endpoints` in tests
- `security/monitoring`: `new_without_discovery()` injects discovery-free client
- Dignity enforcement env var race condition fixed with `serial_test::serial`
- squirrel lib: 400s → 16s (25×); security: 45s → 0.26s; full workspace: 8+ min → ~80s
- 4,613 tests passing, 0 failures, 5 ignored

## Wave 156a — PrimalType Deprecation + Test Consolidation (Aug 3, 2026)

- `PrimalType` eliminated from `squirrel-core`: 7 deprecated usages → capability-domain `String` types
- 34 integration test files consolidated into single `tests/main.rs` binary (build artifacts 9.5 GB → 4.1 GB)
- 7,243 tests passing at time of wave (count normalized to 4,613 after test dedup in 156b)

## Wave 155g — Deep Debt Evolution + Capability Purification (July 28, 2026)

- All `beardog`/`BearDog` production references → `security_provider`/`SecurityProvider` capability stems (deprecated aliases preserved)
- Local crypto oversteps eliminated: blake3 hashing, XOR encryption, rand key generation → delegation errors
- Universal adapters (compute, storage, security, orchestration) wired for IPC delegation via `ipc_client`
- Anomaly detection + security monitoring → `defense.*` capability delegation
- `EcosystemPrimalType` / `PrimalType` enums deprecated → `CapabilityIdentifier` string-based
- `wiremock` removed, `tempfile` → dev-deps, `strum` added for Display derives
- 14 tests evolved from `NotImplemented` → `OperationFailed` assertions
- 763 tests passing, 0 failures, clippy clean, fmt clean

## Wave 152a — Deep Debt Sweep + SDK Alignment (July 26, 2026)

- Deleted 11 dead_code items (5 unused plugin consts, ExampleData struct, 2 dead example handlers, 3 empty recovery stubs)
- Aligned 9 SDK deps to `workspace = true` (serde, serde_json, thiserror, futures, tokio, tracing, uuid, chrono, semver)
- Fixed pre-existing timeout test race condition (env var pollution between concurrent tests)
- Clippy: `map_or` → `is_some_and`, redundant closures → method refs, stale `#[expect]` → `#[cfg_attr]`
- Audit confirmed: all mocks `#[cfg(test)]`, all hardcoded hosts via `universal-constants`, 0 unsafe, 0 TODO/FIXME

## Wave 151b — BTSP Client Handshake for bearDog Strict Mode (July 26, 2026)

- Client-side 4-step BTSP handshake: ClientHello → ServerHello → ChallengeResponse → HandshakeComplete
- HMAC-SHA256 with FAMILY_SEED (env-tiered: FAMILY_SEED → BEARDOG_FAMILY_SEED → BIOMEOS_FAMILY_SEED)
- Wired into `SecurityProviderSecretStore.rpc_call` via `maybe_client_handshake`
- Strict mode detection: `BEARDOG_UDS_REQUIRE_BTSP=1` or `BTSP_STRICT_MODE=1`
- 10 new tests (strict mode, seed resolution, HMAC computation, wire serialization)

## Wave 156c — Deprecated Surface Cleanup: 40 → 13 items (Aug 4, 2026)

- Beardog → SecurityProvider: all 20+ deprecated aliases, re-exports, builder methods, factory functions removed (15 files)
- 7 dead port constants removed (DEFAULT_BIND_ADDRESS, DEFAULT_WEBSOCKET_PORT, etc.)
- `get_port_from_env()` evolved to `port_from_env()` — legitimate self-knowledge utility, not deprecated
- `DefaultConfigManager`, `Config` aliases, `HttpMethod::parse_method()` removed
- `discover_services_by_primal_types()` removed (zero callers)
- Remaining: PrimalType (42 files, multi-wave), PluginError (SDK, 600+ refs), AIError (3 refs but crate::Result alias)

## Wave 150u — CredentialStore Integration via bearDog secrets.* JSON-RPC (July 22, 2026)

- `SecurityProviderSecretStore`: IPC backend for bearDog's `secrets.store/retrieve/list/delete`
- `SecretStoreBackend::SecurityProvider` variant wired in `from_config`
- `api_key_resolver`: AI key resolution via `secrets.retrieve` with env-var fallback
- `discover_http_providers` tries bearDog credential store before legacy env vars
- 14 new tests (endpoint parsing, discovery priority, store/env fallback, key filtering)

## Wave 129 — Mock Evolution + Timeout Threading + Dead Module Purge (June 28, 2026)

- Deleted `chaos/mod.rs` (682L), `universal_provider.rs` + tests (1,128L) — zero callers
- Deleted 6 zero-caller deprecated items (niche, primal_names, tarpc_client, security config, discovery) + 7 associated tests
- Self-healing evolved from simulated to staleness-based health checks; auto-recovery resets components for re-evaluation
- Universal adapters evolved from fabricated success JSON to honest `NotImplemented` errors
- mDNS/DNS-SD stubs evolved from silent no-ops to explicit `MechanismFailed` errors
- Stale `#![expect(deprecated)]` removed from `universal/endpoints.rs`
- `ServerConfig` timeouts threaded to: heartbeat loops, UDS connection handler, inference adapter (env override)

## Wave 124 — Adapter Consolidation + Config Evolution (June 23, 2026)

- SecurityOrchestrator wired at startup in `main.rs` (was middleware-only)
- Deleted 5 dead parallel adapter files (1,811 lines): `adapter.rs`, `adapter_tests.rs`, `bridge.rs`, `discovery.rs`, `universal.rs`
- Single adapter path: `AiRouter` → `router_discovery` → `adapters/universal.rs`
- Removed no-op `ecosystem` feature flag; default features trimmed to `["tarpc-rpc"]`
- Added configurable timeout fields to `ServerConfig` (connection, heartbeat, inference, probe)
- Replaced hardcoded `0.0.0.0` fallbacks with `LOCALHOST_IPV4`
- Expanded `extract_input_data` to recognize `system_message`, `query`, `content`, `input`
- 13 new depth tests (security concurrency, RPC edge cases, malformed input)

## Wave 144a — Phase 2 Transport SHIPPED + Orphan Purge (July 16, 2026)

- Phase 2 transport SHIPPED: TransportEndpoint extracted to universal-patterns/transport/endpoint.rs
- 12 call sites across 6 crates migrated from raw #[cfg] blocks to connect_transport_with_timeout()
- ~564 lines of duplicated platform-gating code eliminated
- MCP task client: missing connect timeout added, EOF-on-split bug fixed
- TimeoutConfig::global() lazy accessor; 8 production hot paths wired from inline Duration literals to env-overridable config
- Split capability_ai.rs (803→700L) — types extracted to capability_ai_types.rs
- Removed 4 orphan crates (-14,535 lines): adapter-pattern-examples, adapter-pattern-tests, integration/*, rule-system
- Centralized hardcoded ports to named constants; IPC service ID to identity constant
- Security client stubs evolved to honest behavior (NotImplemented / empty / real validation)
- Clone-in-loop fix in self_healing/mod.rs

## Degradation

When squirrel is down: AI inference unavailable, context operations fail.
Other primals continue operating — squirrel is intelligence routing, not a gate.
Discovery, health, lifecycle registrations degrade gracefully (standalone mode).
