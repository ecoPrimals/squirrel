<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# Squirrel Status — Wave 157g: G72 Dependency Pandemic Tier 1

**Date**: Aug 10, 2026
**From**: squirrel team @ eastGate
**Posture**: STADIAL SHIFT — shedding vestigial dependencies

## Summary

G72 Tier 1 executed for squirrel. Feature flags trimmed from `["full"]` to
explicit minimal sets. 18 dead dependencies excised across 6 crates. 6 workspace
dependencies removed entirely. Dev dependency versions aligned to workspace
inheritance. Cargo.lock reduced from 384 to 333 unique packages (13% reduction).

## Changes

### Feature Trimming

| Dependency | Before | After |
|-----------|--------|-------|
| `tokio` | `["full"]` | 9 features: `rt-multi-thread`, `macros`, `sync`, `time`, `io-util`, `net`, `fs`, `signal`, `process` |
| `tokio-util` | `["full"]` | `["codec"]` |
| `tarpc` | `["full"]` (`serde-transport` + `tcp` + `unix`) | `["serde-transport"]` (squirrel uses own transport adapter) |

### Dead Dependencies Removed (18)

| Crate | Removed | Reason |
|-------|---------|--------|
| squirrel (main) | `strum`, `prometheus`, `metrics`, `metrics-exporter-prometheus` | Zero `use` imports; monitoring has own exposition format |
| squirrel-mcp | `dashmap`, `futures-util`, `tokio-stream`, `toml` | Zero `use` imports |
| squirrel-context | `futures`, `glob`, `tracing-subscriber` | Zero `use` imports |
| squirrel-mcp-auth | `blake3`, `rand`, `squirrel-mcp-config` | Zero `use` imports; also switched to workspace tokio |
| squirrel-mcp-config | `zeroize` | Zero `use` imports |
| squirrel-ai-tools | `axum`, `chrono`, `tokio-stream` | Zero `use` imports |

### Workspace Dependencies Removed (7)

`glob`, `metrics`, `prometheus`, `metrics-exporter-prometheus`, `tokio-stream`,
`futures-util`, `axum` — no remaining crate consumers.

### Version Alignment

20+ dev-dependency declarations migrated from local version strings to
`workspace = true`: `tokio-test`, `tempfile`, `temp-env`, `serial_test`, `insta`.
Added `temp-env`, `serial_test`, `insta` to workspace `[dependencies]`.

## Metrics

| Metric | Before (157f) | After (157g) | Delta |
|--------|---------------|--------------|-------|
| Cargo.lock packages | 384 | 333 | -51 (13%) |
| Cargo.lock lines | ~2,600 | ~2,027 | -584 lines |
| Tests passing | 4,100 | 4,159 | +59 |
| Cross-arch (Windows) | PASS | PASS | — |
| Build warnings | 0 | 0 | — |

## Remaining G72 Tier 2 (not squirrel-specific)

Per cascade blurb, Tier 2 items are fleet-wide:
- HTTP → songBird/capability.call (6+ projects)
- axum → 0.8
- wgpu → 28
- YAML unify
- tokio::sync → std::sync audit

Squirrel has no HTTP dependencies to migrate (Tower Atomic pattern already
applied — all HTTP eliminated in prior waves).

## Status

- Build: GREEN
- Cross-arch: PASS
- Tests: 4,159 / 0 failures
- G72 Tier 1: COMPLETE
