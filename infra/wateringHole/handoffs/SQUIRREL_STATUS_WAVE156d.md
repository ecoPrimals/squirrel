<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# Squirrel — Wave 156d: Sovereignty + Logging Hygiene + Test Isolation

**Date**: Aug 4, 2026
**Host**: eastGate
**Tests**: 7,142 passing / 0 failures / 0 warnings (`--all-features`)
**Full suite wall-clock**: ~60s

## Changes

### Hardcoded Primal Names → Capability-Based Language

Removed all remaining hardcoded primal names from production error/status messages:

| File | Before | After |
|------|--------|-------|
| `security_client/client.rs` | "BearDog security provider IPC" | "discovered security provider via IPC" |
| `transport/endpoint.rs` | "route via Songbird" | "route via service mesh" |
| `capability_crypto.rs` | "Set ... BEARDOG_SOCKET" | "Set ... SECURITY_SOCKET" |
| `security_client/client.rs` doc | "`BearDog`, enterprise security" | "discovered via capability-based IPC" |

### Socket Path Centralization

Inline `/biomeos/` path constructions replaced with canonical constants:

| File | Change |
|------|--------|
| `rpc/unix_socket.rs` `get_xdg_socket_path` | `format!("{}/biomeos/{}")` → uses `BIOMEOS_SOCKET_SUBDIR` |
| `rpc/unix_socket.rs` `ensure_biomeos_directory` | Hardcoded `/run/user/{uid}/biomeos` → uses `get_socket_dir()` |
| `config/endpoint_resolver.rs` | Two inline constructions → use `BIOMEOS_SOCKET_SUBDIR` |
| `capabilities/discovery.rs` | Inline UID-based paths → `get_socket_dir()` |

### Security Client Evolution

- `apply_ai_security_routing()`: Now checks `self.providers.len()` — returns `NotImplemented` only when no providers discovered; succeeds (with debug log) when providers available
- `get_ai_security_insights()`: Returns distinct JSON for `"no_provider"` vs `"providers_available"` states instead of static "not available"

### Test Isolation Fix

`monitoring_service_provider_trait_methods` intermittent failure:
- **Root cause**: Test called `MonitoringServiceProvider::new()` which runs `IpcClient::discover("monitoring")`. A concurrently running test set `XDG_RUNTIME_DIR` to a temp dir containing `monitoring.sock`, causing this test to attempt real socket I/O
- **Fix**: Converted from `#[tokio::test]` to `#[test]` with `temp_env::with_vars_unset(["XDG_RUNTIME_DIR", "MONITORING_SOCKET", "BIOMEOS_SOCKET_DIR"])` and local runtime

### Production Struct Hygiene

`ModelRegistry.models` (`ai-tools`):
- **Before**: `#[cfg(test)] pub models` / `#[cfg(not(test))] models` dual definition
- **After**: Single private `models` field + `#[cfg(test)] pub(crate) fn models(&self)` accessor

### Emoji Removed from Logging

260 `info!`/`warn!`/`error!`/`debug!` invocations across 33 production files cleaned of emoji characters. Improves grep-ability and aligns with idiomatic Rust tracing conventions.

## Remaining Debt (tracked)

| Item | Scope | Notes |
|------|-------|-------|
| `PrimalType`/`EcosystemPrimalType` | 42 files, ~500 refs | Multi-wave migration to capability-domain strings |
| `PluginError` in SDK | 600+ refs | Needs coordinated migration with plugin consumers |
| `AIError` in ai-tools | 3 consumer lines | Touches `crate::Result` alias — small but careful change |
| Emoji in test/example files | ~170 remaining | Lower priority — test output, not production logs |
| Large file refactoring | No files >800L | Threshold already met; largest prod file is 774L |

## Cascade Notes

Ready for golgiBody cascade. All changes are backward compatible.
