<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# Squirrel Current Status

**Last Updated**: March 15, 2026
**Version**: 0.1.0-alpha
**License**: scyBorg (AGPL-3.0-or-later + ORC + CC-BY-SA 4.0)

## Build

| Metric | Value |
|--------|-------|
| Build | GREEN (0 errors, 0 warnings) |
| Tests | 3,749 passing / 0 failed across 22 crates |
| Clippy | CLEAN |
| Unsafe Code | 0 (`#![forbid(unsafe_code)]`) |
| Pure Rust | 100% (zero C deps default) |
| Unique Deps | 272 |
| Crates | 22 workspace members |

## Feature Gates

| Feature | What it gates | Default |
|---------|---------------|---------|
| `mesh` | Songbird federation/routing/swarm | OFF |
| `http-api` | Axum/Tower/Hyper HTTP stack | OFF |
| `gpu-detection` | ToadStool GPU detection | OFF |
| `system-metrics` | sysinfo C dependency | OFF |
| `local-jwt` | Local JWT signing (brings ring C dep) | OFF |
| `websocket` | WebSocket transport | OFF |
| `delegated-jwt` | Capability-based JWT delegation | ON |

## Known Issues

1. `squirrel-context` `test_experience_replay_*` tests may be slow under load (async locking)
2. `universal-patterns` `test_discover_peers` is timing-sensitive (flaky under CI pressure)
3. Integration test suites (`--features integration-tests`) need rewrite for current API
