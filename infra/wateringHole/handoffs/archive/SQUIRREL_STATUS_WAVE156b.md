<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# Squirrel Status Handoff — Wave 156b

**Date**: Aug 3, 2026 | **Wave**: 156b | **From**: squirrel team on eastGate
**To**: overwatch + upstream primal teams

## Current State

| Metric | Value |
|--------|-------|
| Tests | **4,613** passing / 5 ignored (`--all-features`), 0 failures |
| Test time | **~80s** full workspace (was 8+ min) |
| Clippy | CLEAN |
| Formatting | CLEAN |
| Health | GREEN |
| IPC Methods | **44** registered |
| Capabilities | **39** in `niche::CAPABILITIES` |

## Wave 156b — Test Performance: 400s → 16s

### Root cause identified and fixed

Unit tests were calling live socket discovery (`discover_services`, `discover_capability`,
`discover_all_capabilities`) which probed real Unix sockets with 10-second timeouts per
capability. With 40 consumed capabilities, this produced **400 seconds of wall-clock
blocking with <1 second of CPU** — entirely timeout wait.

### What was done

1. **ecosystem::registry::discovery** — 5 async tests that called `discover_services` with
   live socket probing replaced with instant `perform_service_discovery` + direct registry
   assertions. `perform_service_discovery` promoted to `pub(crate)`.

2. **ecosystem::registry::discovery_error_tests** — Added `populate_registry()` helper.
   Removed all 10 `discover_services` calls. Tests populate via instant registry writes.

3. **security::monitoring::defense_client** — `DefenseClient::without_discovery()` returns
   `ResourceNotFound` immediately instead of probing 3 defense capabilities × 10s.

4. **security::health** — `UniversalSecurityHealthChecker::without_discovery()` skips
   `check_discovered_endpoints` in test context.

5. **security::monitoring** — `new_without_discovery()` injects discovery-free client.

6. **Dignity race condition** — `temp_env::with_var` env mutations leaked across parallel
   tests. Fixed with `#[serial_test::serial(dignity_enforcement)]` on all 6 dignity tests.

### Results

| Module | Before | After |
|--------|--------|-------|
| squirrel lib (2,277 tests) | 400+ seconds | **16 seconds** |
| security tests (293 tests) | 45 seconds | **0.26 seconds** |
| Full workspace (4,613 tests) | 8+ minutes | **~80 seconds** |

### Architectural note

The `TimeoutConfig` uses `OnceLock` — env var overrides after first init have no effect.
Setting `SQUIRREL_DISCOVERY_TIMEOUT_SECS` in tests doesn't help. The fix correctly avoids
the discovery pathway entirely in unit tests rather than trying to adjust timeouts.

## Wave 156a — PrimalType Deprecation + Test Consolidation (same day)

- `PrimalType` eliminated from `squirrel-core` (7 usages → `String` capability domains)
- 34 integration test binaries → 1 via `tests/main.rs` (artifacts 9.5 GB → 4.1 GB)
- `chaos_testing.rs` shim deleted

## For Upstream

- **Test count normalized**: Previous 7,243 count included duplicate tests across 34
  integration test binaries. Actual unique test count is **4,613**.
- **No API changes**: All fixes are test-internal. Production code unchanged except
  `perform_service_discovery` visibility (`async fn` → `pub(crate) async fn`).
- **Integration test consolidation**: If downstream crates reference specific test binary
  names from `crates/main/tests/`, they now live under `tests/integration/` and compile
  as modules of a single `tests/main.rs` binary.

## Known issues

- **Dignity enforcement test isolation**: Fixed with `serial_test` but root cause is
  `temp_env::with_var` not being thread-safe. Consider migrating to explicit config
  injection for sovereignty guard enforcement level.
- **Integration tests still 20s**: The consolidated integration test binary takes ~20s.
  No live I/O timeouts found; likely legitimate test workload.
