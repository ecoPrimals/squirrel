<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# Squirrel Current Status

**Last Updated**: March 15, 2026
**Version**: 0.1.0-alpha
**License**: scyBorg (AGPL-3.0-or-later + ORC + CC-BY-SA 4.0)

## Build

| Metric | Value |
|--------|-------|
| Build | GREEN (0 errors, 0 warnings) |
| Tests | 3,749+ passing / 0 failed across 22 crates |
| Clippy | CLEAN |
| Unsafe Code | 0 (`#![forbid(unsafe_code)]`) |
| Pure Rust | 100% (zero C deps default) |
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
| `delegated-jwt` | Capability-based JWT delegation | ON |
| `mesh` | Songbird federation/routing/swarm | OFF |
| `http-api` | Axum/Tower/Hyper HTTP stack | OFF |
| `gpu-detection` | ToadStool GPU detection | OFF |
| `system-metrics` | sysinfo C dependency | OFF |
| `local-jwt` | Local JWT signing (brings ring C dep) | OFF |
| `websocket` | WebSocket transport | OFF |

## Ecosystem Integration

| Component | Status |
|-----------|--------|
| Capability Registry | `capability_registry.toml` loaded at startup |
| Deploy Graph | `squirrel_deploy.toml` (BYOB pattern from airSpring) |
| biomeOS Lifecycle | `lifecycle.register` + 30s heartbeat (when orchestrator detected) |
| BearDog Crypto | Discovery via biomeOS socket scan (`/run/user/<uid>/biomeos/beardog.sock`) |
| ToadStool AI | Auto-discovered via biomeOS socket scan for local inference |
| Signal Handling | SIGTERM + SIGINT → socket cleanup + graceful shutdown |

## Known Issues

1. `squirrel-context` `test_experience_replay_*` tests may be slow under load (async locking)
2. `universal-patterns` `test_discover_peers` is timing-sensitive (flaky under CI pressure)
3. Integration test suites (`--features integration-tests`) need rewrite for current API
4. Context methods (`context.create`/`update`/`summarize`) use stub storage — persistence via NestGate planned
