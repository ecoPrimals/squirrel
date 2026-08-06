+++
title = "squirrel Validation Summary"
description = "AI inference routing, context management, capability discovery, signal graph dispatch, provenance proxy. 5,668 tests (default features), 44 IPC methods."
date = 2026-08-06

[taxonomies]
primals = ["squirrel"]
springs = []
+++

## Wave 157a — Dep Cleanup + Lint Hygiene + Retry Modernization (Aug 6, 2026)

- 5 stale workspace deps removed (tower, tower-http, async-recursion, semver, test-case)
- `reason =` added to all 25 `#[allow(...)]` blocks (14 crate roots + 11 test/example files)
- Retry mechanism: `Pin<Box<dyn Future>>` → generic `Fut` param (eliminates forced heap alloc)
- Dead code: unused ViolationType variants, dead import cleaned
- Root docs updated: 13 crates, 838 files, 257K lines, 5,668 tests passing

## Wave 156z — Orphan Crate Excise + Debt Cleanup (Aug 6, 2026)

- squirrel-core (13,907L) removed — Songbird mesh/federation/swarm code, 0 reverse deps
- squirrel-sdk (11,912L) removed — ToadStool WASM plugin SDK, 0 reverse deps
- 12 timeout literals centralized to universal-constants (5 new constants)
- DignityViolation → thiserror, test file split (1293→759+280+254), lint hygiene
- Workspace: 13 crates, net ~-26K lines, 0 warnings, 5,753 tests passing

## Wave 156y — squirrel-plugins Crate Excise (Aug 6, 2026)

- Fully orphaned squirrel-plugins crate removed (15,573 lines, 0 reverse deps, never linked into production binary)
- Plugin hosting (sandbox, marketplace, dynamic loading, web dashboard) belongs to ToadStool
- Context plugin functionality already in squirrel-interfaces + squirrel-context
- Archived to tarball for ToadStool future use
- Net -14,246 lines, 0 warnings, 6,104 tests passing (15 workspace crates)

## Wave 156x — Plugin Dead Code Elimination (Aug 6, 2026)

- PluginV2 + 5 dead extension scaffolds + ZeroCopyPlugin trait + dead interfaces scaffolds deleted
- WebPluginExt naming collision resolved
- Net -677 lines, 0 warnings, 6,293 tests passing

## Wave 156w — Config Excise + PrimalType Fossil + Bincode Cleanup (Aug 6, 2026)

- External `config` 0.14 crate replaced with hand-rolled loader (-234 lockfile lines)
- Deprecated ecosystem-api PrimalType enum deleted (3 struct fields → String)
- bincode direct dep moved to dev-only, documented tarpc upstream constraint
- 0 warnings, 6,294 tests passing

## Wave 156v — Deduplication + thiserror + Default Derives (Aug 6, 2026)

- ModelRegistry deduplicated (config/model_registry.rs 760→11L shim, -749 lines)
- CapabilityDomain/Identifier unified (canonical in ecosystem-api, main re-exports)
- 4 error types migrated to thiserror (JsonRpcError, RpcError, WireFormatError, CLI PluginError)
- 14 manual Default impls → #[derive(Default)] (-161 lines)
- 0 warnings, 6,298 tests passing

## Wave 156u — Legacy Pattern Modernization + Error Consolidation (Aug 6, 2026)

- PrimalError: 5 duplicate variants eliminated, ~85 call sites migrated
- 6 error types migrated to thiserror (-82 net lines of manual Display/Error impls)
- 26 Pin<Box<dyn Future>> eliminated (UniversalServiceRegistry + PluginStateManager)
- async-recursion dependency removed (function was iterative, not recursive)
- 0 warnings, 6,302 tests passing

## Wave 156t — Timeout Consolidation + De-Async + Smart Refactoring (Aug 6, 2026)

- 157 functions de-asynced (session, metrics, biomeos, discovery/rpc/providers)
- 8 timeout literals consolidated to universal_constants::timeouts
- DiscoveryConfig capabilities aligned with niche::CAPABILITIES
- learning/engine.rs refactored (773→504L): NeuralNetwork + types extracted
- doctor.rs refactored to directory module (checks + tests extracted)
- 5 stale lint suppressions removed from squirrel-context
- 0 errors, 6,302 tests passing

## Wave 156s — Fossil Cleanup + Dependency Pruning + E2 Prep (Aug 6, 2026)

- EcosystemPrimalType fossil deleted (deprecated enum + 16 tests, zero prod callers)
- ecosystem-api orphan dependency pruned from 3 consumer crates
- Main crate #![expect(deprecated)] blanket removed
- systemd service polished for E2 ironGate deploy (ai.sock cleanup, LimitNOFILE)
- 0 warnings, 6,302 tests passing

## Wave 156r — PluginMetadata Migration + Deprecated Enum Deletion (Aug 6, 2026)

- Full `PluginMetadata` migration: 28 files migrated from deprecated `plugin::PluginMetadata` (Uuid id) to canonical `squirrel_interfaces::plugins::PluginMetadata` (String id). Deprecated struct deleted.
- `AIError` enum deleted (345 lines, zero production callers)
- `EcosystemPrimalType` de-exported from public API
- 3 crate-level `#![expect(deprecated)]` blankets eliminated (ai-tools, SDK, plugins)
- DNS constant extracted, cfg_attr lint hygiene, error type Uuid→String migration
- 0 warnings, 6,366 tests passing

## Wave 156q — Port Constants + Lint Hygiene + Dead Code (Aug 6, 2026)

- 6 inline port literals wired to `universal_constants::network` named constants across 4 crates
- 3 new constants: `DEFAULT_HTTP_SERVICE_PORT` (8081), `DEFAULT_ADMIN_PORT` (8082), `DEFAULT_MCP_TCP_PORT` (9000)
- 5 `#[allow]`/`#[expect]` attributes converted to include `reason = "..."`
- 1 unfulfilled `#![allow(deprecated)]` removed (no deprecated items in module)
- Dead `infer_primal_type_from_capability()` removed (zero callers)
- 0 warnings, 6,371 tests passing

## Wave 156p — PluginError → SDKError Migration (Aug 6, 2026)

- SDK error system fully migrated from deprecated `PluginError` (44 variants, ~580 refs) to `SDKError` hierarchy
- 15 consumer files migrated (config, operations, manager, plugin_config, utils, events, connection, http, fs, commands, http_types, context, message)
- Common `From` impls moved to `universal-error/src/sdk.rs` (orphan-rule compliant)
- 50KB dead infrastructure deleted: severity.rs, severity_tests.rs, context.rs, macros.rs
- `PluginError` enum quarantined as `pub(crate)` fossil for serde backward compat
- 0 warnings, 6,077 tests passing

## Wave 156o — Pre-sized Collections + String Builder + API Ergonomics (Aug 6, 2026)

- 7 `with_capacity` additions in hot paths (task manager, health monitor, batch processor, discovery, monitoring)
- `rule_to_mdc` String builder: `push_str(&format!)` → `write!` + `with_capacity(512)` — zero intermediate allocations
- 10 more constructor/builder params evolved to `impl Into<String>` (SecurityContext, EcosystemConfig, Event, PluginContext)
- `vec![]` initializer for known-size recommendations
- Neural policy init: 3 inner vecs pre-sized with `with_capacity`
- 0 warnings, 6,455 tests passing

## Wave 156n — Allocation Hygiene + API Ergonomics (Aug 6, 2026)

- 2 `Option<&String>` params evolved to `Option<&str>` in `dignity.rs`; call sites: `.as_ref()` → `.as_deref()`
- 6 redundant `.clone()` on `Copy` types removed; `monitoring::HealthState` given `Copy` derive
- 15 `vec.contains(&x.to_string())` → `vec.iter().any(|e| e == x)` — zero-allocation comparisons across 11 files
- 11 constructor/builder params in `ServiceDefinition`/`ServiceQuery` evolved to `impl Into<String>`
- `ServiceDefinition::has_capability` allocation eliminated
- 0 warnings, 6,455 tests passing

## Wave 156m — PluginError→SDKError Bridge + 20 More Copy Derives (Aug 5, 2026)

- `From<PluginError> for SDKError` bridge added: maps all 40+ deprecated variants to hierarchical SDKError
- 20 additional enums derive `Copy` across core, context, plugins, mcp, main, sdk, universal-patterns
- Total: 45 Copy-derived enums across Waves 156l–m
- 0 warnings, 6,455 tests passing

## Wave 156l — PrimalDependency Migration, Copy Derives, Unwrap Elimination (Aug 5, 2026)

- `ecosystem_api::PrimalDependency.primal_type` migrated from deprecated `PrimalType` enum to `String`
- `ecosystem_api::UniversalPrimalProvider::primal_type()` evolved from `PrimalType` return to `&str`
- `persist_session_context` evolved from `NotImplemented` error to graceful degradation (`Ok(())`)
- 4 production `unwrap()` eliminated in `TaskManager` via direct `Arc` reference cloning
- 25 unit-variant enums across 14 files derive `Copy` — trait, core, API, config, security types
- 0 warnings, 6,455 tests passing

## Wave 156j — EcosystemPrimalType → String Migration + Context Quality (Aug 5, 2026)

- 9 struct/event fields across 5 files migrated from `EcosystemPrimalType` enum to `String` capability-domain values
- Construction sites use `crate::niche::DOMAIN` / `primary_capability.to_string()` — zero deprecated enum usage in production struct consumers
- `PrimalApiRequest::new()` signature changed to `impl Into<String>` for caller flexibility
- `#![expect(deprecated)]` module attributes removed from `optimized_implementations.rs` and `registry/types.rs`
- `VisualizationAction` derives `Copy`; `cache_visualization` clone reduction; `count_enabled_components` code smell fixed
- 0 warnings, 7,140 tests passing

## Wave 156i — AIError Migration + PrimalType Dedup + Hardcoded Port Elimination (Aug 5, 2026)

- `AIError` type alias migrated to `AIToolsError` in `squirrel-ai-tools` — 8 construction sites + 5 bare `?` operators converted
- Config `PrimalType` deduplicated: `config::types::PrimalType` removed → `pub use crate::traits::PrimalType`
- Hardcoded ports (`8081`, `8082`, `9090`, `8500`) eliminated from `endpoint_resolver.rs` → `ports::metrics()` + `get_service_port()`
- Dangling `PLUGIN_METADATA_MIGRATION_PLAN.md` references and dead `ADR-008` doc link fixed
- 0 warnings, 7,140 tests passing

## Wave 156h — Clone-to-Arc Evolution + Copy Derives + Debris Cleanup (Aug 5, 2026)

- TaskManager evolved to `Arc<Task>` storage with `Arc::make_mut` CoW mutations — 12 of 16 `.clone()` calls eliminated
- SyncManager broadcast evolved to `Arc<SyncEvent>` fan-out — N deep clones per event → single Arc wrap + cheap Arc::clone
- `Copy` derived on `SyncStatus` and `ConflictResolutionStrategy` enums (zero heap data)
- `ResourceValidationRule` dead_code warning fixed (system-metrics feature gate)
- Orphaned `sync_manager_tests.rs` deleted (548 lines, never compiled, used nonexistent enum variants)
- 0 warnings, 0 test regressions

## Wave 156g — RegistryType Elimination + Discovery Modernization (Aug 5, 2026)

- `RegistryType` enum (6 variants) deleted — replaced with `registry_backend: String` on `RegistryDiscovery`
- 3 module-level `#![expect(deprecated)]` removed from discovery modules
- Duplicate env-var-to-enum match blocks eliminated in runtime_engine.rs and self_knowledge.rs
- 0 warnings, 7,238 tests passing

## Wave 156f — Deep Scaffolding Audit + Dead Code Elimination (Aug 5, 2026)

- `ClientRequestCounter` deleted (superseded by `RateLimitBucket`, zero consumers)
- `ConnectionMetadata` + `metadata` field deleted (universal_adapter_v2, zero readers)
- `ProviderScore.meets_requirements` field deleted (always true, never read)
- 14 write-only struct fields evolved to `_` prefix convention across rate limiter, security hardening, federation, BTSP client, transport frames, viz cache
- `ViolationType` test-only variants gated behind `#[cfg(test)]`
- `FilePluginDiscovery::new()` deleted (zero callers)
- 0 warnings, 7,240 tests passing

## Wave 156e — Deployment + Safety + Dead Code Elimination (Aug 5, 2026)

- systemd service unit created (`infra/squirrel.service`) for ironGate deployment
- `JsonRpcServer::new()` and `with_ai_router()` evolved to `impl Into<String>`
- 6 SDK WASM `Reflect::set(...).expect(...)` → safe `let _ = ...`
- `StringPool::get_or_create` → `entry().or_insert_with()` (eliminates expect)
- Federation dead code deleted (queue_message, ConnectionState, transition_to)
- 8 reserved-for-future fields evolved to `_` prefix convention
- 0 warnings, 7,241 tests passing

## Wave 156d — Sovereignty + Logging Hygiene + Test Isolation (Aug 4, 2026)

- Hardcoded primal names ("BearDog", "Songbird") removed from production error/status messages — replaced with capability-based language
- Inline `/biomeos/` path constructions centralized to `BIOMEOS_SOCKET_SUBDIR` / `get_socket_dir()` (5 files)
- Security client stubs evolved: `apply_ai_security_routing()` and `get_ai_security_insights()` now provider-count-aware
- `#[cfg(test)] pub` field removed from `ModelRegistry` — now private with test-only accessor
- Test isolation: `monitoring_service_provider_trait_methods` race condition fixed via env isolation
- 260 emoji removed from log macros across 33 production files (grep-friendly tracing)
- 7,241 tests passing, 0 warnings

## Status

- **Gate**: CLEAR (stadial readiness confirmed May 17, 2026)
- **Phase**: 3 (BTSP Phase 3 AEAD encrypted framing)
- **Edition**: 2024 (Rust 1.94+)
- **Tests**: **6,302** passing across 16 workspace crates (default features), full suite ~75s
- **Source**: ~980 `.rs` files, ~300k lines
- **Clippy**: 0 warnings (`pedantic` + `nursery` + `cargo`, `-D warnings`, `--all-features`)
- **Docs**: 0 warnings (`-D warnings`)
- **deny.toml**: ring, openssl, reqwest, native-tls, aws-lc-sys all banned; pure Rust enforced
- **Coverage**: 90.14% region / 89.67% line (cargo-llvm-cov)
- **Binary**: 4.4 MB static-pie musl, stripped, BLAKE3 checksummed, zero host paths
- **Transport**: Full Phase 2 — `TRANSPORT_ENDPOINT` accepted + `connect_transport()` for all outbound IPC + Eukaryotic riboCipher: MitoBeacon (`0xEC`/`0xED`) accepted + outbound `[0xEC, 0x01]` preamble on all UDS
- **HTTP IPC**: Raw TCP JSON-RPC delegation (zero external HTTP deps, uniBin compliant)
- **Files >800L (prod)**: 0 — all production files under 800 lines
- **Hardcoding**: Evolved — 14 production files migrated from literal localhost/ports to capability-based discovery
- **TRUE PRIMAL**: `niche::REQUIRED_CAPABILITIES` replaces named-primal `DEPENDENCIES`; `capability_id` field on `EcosystemServiceRegistration`; all struct fields migrated from `EcosystemPrimalType` enum → `String` capability domains (Wave 156j); deprecated enum retained as fossil for serde compat
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
