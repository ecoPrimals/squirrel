<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# Squirrel Current Status

**Last Updated**: March 16, 2026
**Version**: 0.1.0-alpha.3
**License**: AGPL-3.0-only (scyBorg: ORC + CC-BY-SA 4.0 for docs)

## Build

| Metric | Value |
|--------|-------|
| Build | GREEN (0 errors, 0 warnings) |
| Tests | 4,465 passing / 0 failed across 22 crates |
| Edition | 2024 (Rust 1.93.0) |
| Clippy | CLEAN (pedantic + nursery lints enabled) |
| Docs | All 22 crates `#![warn(missing_docs)]` — zero doc warnings |
| Formatting | `cargo fmt --all -- --check` passes |
| Unsafe Code | 0 — `#![forbid(unsafe_code)]` unconditional on all 22 crates |
| Pure Rust | 100% default features (zero C deps; reqwest/ring only behind optional dev features) |
| Coverage | 66% line coverage via `cargo-llvm-cov` (target: 90%) |
| Crates | 22 workspace members |
| Files >1000 lines | 0 |

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

## tarpc Service

All 18 JSON-RPC methods mirrored as tarpc service methods with typed request/response
structs. `TarpcRpcServer` delegates to `JsonRpcServer` for shared handler logic.
Protocol negotiation selects tarpc or JSON-RPC per-connection.

## Feature Gates

| Feature | What it gates | Default |
|---------|---------------|---------|
| `capability-ai` | Capability-based AI routing (Pure Rust) | ON |
| `ecosystem` | Ecosystem integration | ON |
| `tarpc-rpc` | High-performance binary RPC via tarpc | ON |
| `delegated-jwt` | Capability-based JWT delegation | ON |
| `system-metrics` | sysinfo C dependency | OFF |
| `monitoring` | Prometheus metrics (brings hyper) | OFF |
| `nvml` | NVIDIA GPU detection via nvml-wrapper | OFF |
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

## Crypto Migration

See [docs/CRYPTO_MIGRATION.md](docs/CRYPTO_MIGRATION.md) for the path from reqwest 0.11 (ring) toward pure Rust. ecosystem-api uses reqwest 0.12 as proof of concept.

## Tooling

| Tool | Config |
|------|--------|
| rustfmt | `.rustfmt.toml` — edition 2024, max_width 100 |
| clippy | `clippy.toml` — pedantic + nursery via `[workspace.lints.clippy]` |
| cargo-deny | `deny.toml` — license allowlist, advisory audit, ban wildcards |
| cargo-llvm-cov | Installed, coverage measurable |

## Known Issues

1. Coverage at 66% — needs targeted test expansion for cli, auth, mcp crates (<40%)
2. Context methods (`context.create`/`update`/`summarize`) use stub storage — persistence via NestGate planned
3. `universal-patterns` `test_discover_peers` timing-sensitive under CI pressure
4. reqwest 0.11 → 0.12 migration incomplete (1 of 10 crates upgraded)
