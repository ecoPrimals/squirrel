<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# Squirrel Current Status

**Last Updated**: March 15, 2026
**Version**: 0.1.0-alpha
**License**: AGPL-3.0-only (scyBorg: ORC + CC-BY-SA 4.0 for docs)

## Build

| Metric | Value |
|--------|-------|
| Build | GREEN (0 errors, 0 warnings) |
| Tests | 3,749+ passing / 0 failed across 22 crates |
| Edition | 2024 (Rust 1.93.0) |
| Clippy | CLEAN (`-D warnings` with pedantic — zero errors) |
| Docs | All 22 crates `#![warn(missing_docs)]` — zero doc warnings |
| Formatting | `cargo fmt --all -- --check` passes |
| Unsafe Code | 0 (`#![cfg_attr(not(test), forbid(unsafe_code))]` — test-only `unsafe` for `set_var`) |
| Pure Rust | 100% default features (zero C deps; reqwest/ring only behind optional dev features) |
| Coverage | ~66% line coverage via `cargo-llvm-cov` (target: 90%) |
| Unique Deps | 272 |
| Crates | 22 workspace members |

## JSON-RPC Methods

Source of truth: [`capability_registry.toml`](capability_registry.toml)

| Domain | Methods |
|--------|---------|
| AI | `ai.query`, `ai.list_providers`, `ai.complete`, `ai.chat` |
| Capability | `capability.announce`, `capability.discover` |
| Context | `context.create`, `context.update`, `context.summarize` |
| System | `system.health`, `system.status`, `system.metrics`, `system.ping` |
| Discovery | `discovery.peers` |
| Tool | `tool.execute`, `tool.list` |
| Lifecycle | `lifecycle.register`, `lifecycle.status` |

## Feature Gates

| Feature | What it gates | Default |
|---------|---------------|---------|
| `capability-ai` | Capability-based AI routing (Pure Rust) | ON |
| `ecosystem` | Ecosystem integration | ON |
| `tarpc-rpc` | High-performance binary RPC via tarpc | ON |
| `delegated-jwt` | Capability-based JWT delegation via BearDog | ON |
| `system-metrics` | sysinfo C dependency | OFF |
| `monitoring` | Prometheus metrics (brings hyper) | OFF |
| `gpu-detection` | ToadStool GPU detection | OFF |
| `local-jwt` | Local JWT signing (brings ring C dep) | OFF |

## Ecosystem Integration

| Component | Status |
|-----------|--------|
| Capability Registry | `capability_registry.toml` loaded at startup |
| Deploy Graph | `squirrel_deploy.toml` (BYOB pattern) |
| biomeOS Lifecycle | `lifecycle.register` + 30s heartbeat (when orchestrator detected) |
| BearDog Crypto | Discovery via biomeOS socket scan |
| ToadStool AI | Auto-discovered via biomeOS socket scan for local inference |
| Signal Handling | SIGTERM + SIGINT → socket cleanup + graceful shutdown |

## Known Issues

1. `universal-patterns` `test_discover_peers` is timing-sensitive (flaky under CI pressure)
2. Integration test suites (`--features integration-tests`) need rewrite for current API
3. Context methods (`context.create`/`update`/`summarize`) use stub storage — persistence via NestGate planned
4. Coverage at 66% — needs more tests to reach 90% target (cli, auth, mcp at <40%)
5. One env-var test (`test_fallback_chain`) is racy under parallel execution — needs `serial_test`
