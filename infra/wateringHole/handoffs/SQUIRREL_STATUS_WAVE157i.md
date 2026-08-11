<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# Squirrel Status — Wave 157i (Darwin Fix + Clippy Sweep)

**Date**: Aug 11, 2026
**Gate**: eastGate
**Posture**: GREEN — darwin upstream merge applied, clippy clean, flaky test fixed

## Changes

### Darwin Build Fix (graftGate upstream merge)

Removed hardcoded `[build] target = "x86_64-unknown-linux-musl"` from `.cargo/config.toml`.
Previously, every `cargo build/check/test` was forced to target musl regardless of host platform.
On graftGate (aarch64-apple-darwin), this required explicit `--target aarch64-apple-darwin` for
every command.

**Fix**: No default target — cargo uses the host triple. ecoBin builds already specify `--target`
explicitly in the justfile (`build-ecobin`, `build-ecobin-arm`). Target-specific config sections
(musl rustflags, darwin dead_strip, RISC-V linker) remain and activate only when selected.

**Verification**: `cargo check --workspace` passes on all three architectures:
- `x86_64-unknown-linux-gnu` (eastGate native) — clean
- `aarch64-apple-darwin` (graftGate) — clean
- `x86_64-pc-windows-gnu` (cross-arch) — clean
- `x86_64-unknown-linux-musl` (ecoBin) — clean

### Flaky Test Fixed

`coordinate_storage_fails_without_provider` — env-dependent since Wave 157g. Root cause: test
endpoint `https://storage.test` fell through socket resolution to convention-based path
(`storage.sock`), connecting to live NestGate on eastGate. Fixed: endpoint changed to
`unix:///tmp/squirrel-test-nonexistent-storage.sock` for deterministic IPC failure.

### Clippy Sweep

40+ clippy warnings fixed across 4 crates (`universal-patterns`, `squirrel-context`,
`squirrel-commands`, `squirrel`, `squirrel-mcp`, `squirrel-ai-tools`). Major categories:
- 14 unnecessary `Result` wrappings removed
- 8 `write!` → `writeln!`
- 5 unfulfilled `#[expect]` annotations removed
- `io::Error::other()`, `is_ok_and()`, `pub(crate)` → `pub`, let-chains, `#[must_use]`

## Metrics

| Metric | Value |
|--------|-------|
| Workspace crates | 12 |
| Rust files | 616 |
| Lines | ~188,000 |
| Tests | **4,127** passing / **0 failures** / 0 flaky |
| Cargo.lock packages | 333 |
| Clippy | CLEAN — `--workspace -- -D warnings` |
| Cross-arch | darwin + windows + musl all clean |

## Upstream Status

| Fix | Status |
|-----|--------|
| squirrel `.cargo/config.toml` darwin fix | **APPLIED** — ready for upstream merge |
| bearDog ios.rs import | Needs upstream (bearDog code team) |
| toadStool cfg gate | Needs upstream (toadStool code team) |
| petalTongue rustix API | Needs upstream (petalTongue code team) |

## Remaining

- `context-learning` + `context-visualization` deprecated (feature-gated OFF). Future extraction
  when compute/petalTongue capability contracts are available.
- G72 Tier 2 items (HTTP→songBird, axum→0.8) — fleet-wide, not squirrel-specific.
- Gossip injection not yet wired for squirrel (coordination-only primal, not a data producer).
