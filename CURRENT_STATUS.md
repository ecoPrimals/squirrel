<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# Squirrel Current Status

**Last Updated**: March 15, 2026
**Version**: 0.1.0-alpha
**License**: AGPL-3.0-or-later (scyBorg)

## Build

| Metric | Value |
|--------|-------|
| Build | GREEN (0 errors) |
| Tests | 1,622 passing / 0 failed |
| Clippy | CLEAN |
| Unsafe Code | 0 (`#![forbid(unsafe_code)]`) |
| Pure Rust | 100% (zero C deps default) |
| Unique Deps | 272 |

## Feature Gates

| Feature | What it gates | Default |
|---------|---------------|---------|
| `mesh` | Songbird federation/routing/swarm | OFF |
| `http-api` | Axum/Tower/Hyper HTTP stack | OFF |
| `gpu-detection` | ToadStool GPU detection | OFF |
| `system-metrics` | sysinfo C dependency | OFF |
| `local-jwt` | BearDog local JWT signing | OFF |
| `websocket` | WebSocket transport | OFF |

## Known Issues

1. `squirrel-mcp-auth` test suite has pre-existing compile errors
2. `squirrel-plugins` test suite has pre-existing API mismatch
3. `squirrel-mcp` test suite has pre-existing type annotation errors
